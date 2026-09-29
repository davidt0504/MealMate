import 'dart:math';

import 'package:flutter/foundation.dart' show protected;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/planning/experiment_provider.dart';
import 'package:meal_mate/features/planning/planner_provider.dart';
import 'package:meal_mate/src/rust/api/decisions.dart';
import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/planning_drafts.dart';

/// One Cover My Week draft, whole from Rust (OPT-007): the screen holds nothing durable
/// (invariants 17, 21). Alternatives, Lock in, Choose, Undo and Review edit the draft; only
/// [CoverNotifier.accept] writes the saved plan, and it writes exactly what is shown.
class CoverView {
  const CoverView({required this.draft});

  final DraftViewDto draft;

  bool get accepted => draft.state == DraftStateDto.accepted;
  bool get needsReview => draft.state == DraftStateDto.needsReview;
  bool get open =>
      draft.state == DraftStateDto.active ||
      draft.state == DraftStateDto.needsReview;
}

/// A fresh id per user intent: Rust replays a repeated id instead of applying it twice.
String newRequestId() {
  final random = Random.secure();
  return List.generate(
    16,
    (_) => random.nextInt(256).toRadixString(16).padLeft(2, '0'),
  ).join();
}

/// Kinds after which the view on screen is out of date: reload rather than guess.
bool reloadsView(Object error) =>
    error is KimattaError_Draft &&
    const {
      DraftErrorKind.stale,
      DraftErrorKind.needsReview,
      DraftErrorKind.closed,
      DraftErrorKind.sessionChanged,
    }.contains(error.kind);

/// Keyed by cycle offset, exactly as [plannerProvider] and `shoppingProvider` are, reading the
/// shared anchor and never committing one. Opening resumes this window's draft without a
/// search, or starts one from the saved meals.
class CoverNotifier extends AutoDisposeFamilyAsyncNotifier<CoverView, int> {
  bool _inFlight = false;

  @override
  Future<CoverView> build(int arg) async {
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    final draft = await openDraft(_context(householdId));
    return CoverView(draft: draft);
  }

  DraftContextDto _context(String householdId) => DraftContextDto(
    householdId: householdId,
    today: ref.read(plannerAnchorProvider),
    offsetCycles: arg,
  );

  DraftEnvelopeDto _envelope(DraftViewDto draft, String householdId) =>
      DraftEnvelopeDto(
        context: _context(householdId),
        databaseSession: draft.databaseSession,
        draftId: draft.draftId,
        expectedRevision: draft.revision,
        requestId: newRequestId(),
      );

  /// The bridge calls behind overridable seams, as `PlannerNotifier`'s are.
  @protected
  Future<DraftViewDto> openDraft(DraftContextDto context) =>
      openPlanningDraft(context: context);

  @protected
  Future<DraftViewDto> mutateDraft(
    DraftEnvelopeDto envelope,
    DraftActionDto action,
  ) => mutatePlanningDraft(envelope: envelope, action: action);

  @protected
  Future<DraftViewDto> acceptDraft(DraftEnvelopeDto envelope) =>
      acceptPlanningDraft(envelope: envelope);

  @protected
  Future<MealExclusionOutcomeDto> addExclusions(AddMealExclusionsDto request) =>
      addMealExclusions(request: request);

  @protected
  Future<DraftViewDto> inspectDraft(DraftContextDto context, String draftId) =>
      inspectPlanningDraft(context: context, draftId: draftId);

  @protected
  Future<PlanDecisionOutcomeDto> recordDecision(
    PlanDecisionRequestDto request,
  ) => recordPlanDecision(request: request);

  /// Runs one command against the draft on screen. A second tap while one is in flight is
  /// dropped here as well as disabled in the widget. A response is published only if the view
  /// it was built on is still the one on screen — same session, same revision — so a late
  /// answer from before a restore or a reload never overwrites a newer view. A refusal that
  /// means the view is stale reloads it and still reports the refusal.
  Future<DraftViewDto?> _run(
    Future<DraftViewDto?> Function(DraftViewDto draft, String householdId) call,
  ) async {
    final before = state.valueOrNull?.draft;
    if (before == null || _inFlight) return null;
    _inFlight = true;
    try {
      final householdId = await ref.read(
        householdProvider.selectAsync((h) => h.id),
      );
      final next = await call(before, householdId);
      final now = state.valueOrNull?.draft;
      final current =
          now != null &&
          now.databaseSession == before.databaseSession &&
          now.revision == before.revision;
      if (next != null && current) state = AsyncData(CoverView(draft: next));
      return next;
    } catch (error) {
      if (reloadsView(error)) ref.invalidateSelf();
      rethrow;
    } finally {
      _inFlight = false;
    }
  }

  Future<void> mutate(DraftActionDto action) => _run(
    (draft, householdId) => mutateDraft(_envelope(draft, householdId), action),
  );

  /// Accept: one tap writes exactly the proposal on screen. The saved plan and the list derived
  /// from it are refreshed only on the returned receipt. Returns the accepted view, or `null`
  /// when nothing was sent.
  Future<DraftViewDto?> accept() async {
    final accepted = await _run(
      (draft, householdId) => acceptDraft(_envelope(draft, householdId)),
    );
    if (accepted?.receipt != null) ref.invalidate(plannerProvider(arg));
    return accepted;
  }

  /// "Never suggest": a standing rule, committed at once and kept whatever happens to this
  /// draft. The draft comes back with its choices kept and the conflict shown.
  Future<void> neverSuggest(List<String> dishes) async {
    await _run((draft, householdId) async {
      final outcome = await addExclusions(
        AddMealExclusionsDto(
          context: _context(householdId),
          dishes: dishes,
          acting: _envelope(draft, householdId),
        ),
      );
      ref.invalidate(mealExclusionsProvider);
      final next = outcome.draft;
      if (next == null) ref.invalidateSelf();
      return next;
    });
  }

  /// After Accept the screen stays settled; editing again opens a new draft from the saved
  /// meals.
  void editAgain() => ref.invalidateSelf();

  /// An expired or set-aside draft: read it for its revision, discard it, reopen this window.
  Future<void> discardExpired(String draftId) async {
    final householdId = await ref.read(
      householdProvider.selectAsync((h) => h.id),
    );
    final old = await inspectDraft(_context(householdId), draftId);
    await mutateDraft(
      _envelope(old, householdId),
      const DraftActionDto.discard(),
    );
    ref.invalidateSelf();
  }

  /// "We don't have any restrictions": a household fact recorded with its evidence row. It
  /// settles wording only, so the draft keeps every choice and needs no review.
  Future<void> confirmNoRestrictions() async {
    final householdId = await ref.read(
      householdProvider.selectAsync((h) => h.id),
    );
    await recordDecision(
      PlanDecisionRequestDto(
        householdId: householdId,
        today: ref.read(plannerAnchorProvider),
        offsetCycles: arg,
        decision: const PlanDecisionDto.restrictionsReviewed(),
      ),
    );
    ref.invalidateSelf();
  }
}

/// The week action's label: `New mix` unless this visit's tester experiment session says
/// otherwise (OPT-007 §10). The label never changes what the action does — the command sent is
/// the same whichever words are on the button.
final weekActionLabelProvider = Provider.autoDispose<String>(
  (ref) =>
      ref.watch(weekActionExperimentProvider).valueOrNull?.label ?? 'New mix',
);

final coverProvider = AsyncNotifierProvider.autoDispose
    .family<CoverNotifier, CoverView, int>(CoverNotifier.new);

/// Settings › Meal exclusions: the household's standing rules, identity and legacy phrase.
class MealExclusionsNotifier
    extends AutoDisposeAsyncNotifier<List<MealExclusionDto>> {
  @override
  Future<List<MealExclusionDto>> build() async {
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    return listExclusions(householdId);
  }

  @protected
  Future<List<MealExclusionDto>> listExclusions(String householdId) =>
      listMealExclusions(householdId: householdId);

  @protected
  Future<MealExclusionOutcomeDto> removeExclusion(
    RemoveMealExclusionDto request,
  ) => removeMealExclusion(request: request);

  /// Disables one rule; the dish becomes eligible for future suggestions and nothing is
  /// regenerated. Open drafts meet the change as a review on their next command.
  Future<void> remove(String policyId) async {
    final householdId = await ref.read(
      householdProvider.selectAsync((h) => h.id),
    );
    final outcome = await removeExclusion(
      RemoveMealExclusionDto(
        context: DraftContextDto(
          householdId: householdId,
          today: ref.read(plannerAnchorProvider),
          offsetCycles: 0,
        ),
        policyId: policyId,
      ),
    );
    state = AsyncData(outcome.exclusions);
  }
}

final mealExclusionsProvider =
    AsyncNotifierProvider.autoDispose<
      MealExclusionsNotifier,
      List<MealExclusionDto>
    >(MealExclusionsNotifier.new);
