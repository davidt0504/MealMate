import 'package:flutter/foundation.dart' show protected;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/planning/planner_provider.dart';
import 'package:meal_mate/src/rust/api/decisions.dart';
import 'package:meal_mate/src/rust/api/planner.dart';

/// One Cover My Week outcome, whole from Rust; the screen holds nothing durable
/// (invariants 17, 21). Two rules the widgets must keep:
/// - the UI never reads `result.reasonCodes` — slot state and the attention list carry
///   everything material, and the raw field still contains engine tokens (`LOCK_CONFLICT`
///   is filtered at the bridge for slots, not here);
/// - every preview appends one ledger `propose` row by design — cheap, honest evidence of
///   what was shown.
class CoverView {
  const CoverView({required this.outcome});

  final CoverCycleOutcomeDto outcome;

  PlanningResultDto get result => outcome.result;

  int get highUrgencyCount =>
      result.attention.where((a) => a.urgency == UrgencyDto.high).length;
}

/// Keyed by cycle offset, exactly as [plannerProvider] and `shoppingProvider` are, so all
/// three screens page through the same sequence of windows. Reads the shared anchor and
/// never commits one — re-anchoring stays the planner's job (the shopping precedent). On a
/// cold entry (deep link to `/plan/cover` before the grid ever fetched) the anchor's own
/// `build` resolves from `plannerTodayProvider`, the same date the planner would anchor to,
/// so offset-0 windows cannot diverge.
class CoverNotifier extends AutoDisposeFamilyAsyncNotifier<CoverView, int> {
  @override
  Future<CoverView> build(int arg) async {
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    final outcome = await runCover(_request(householdId, apply: false));
    return CoverView(outcome: outcome);
  }

  CoverCycleRequestDto _request(String householdId, {required bool apply}) =>
      CoverCycleRequestDto(
        householdId: householdId,
        today: ref.read(plannerAnchorProvider),
        offsetCycles: arg,
        apply: apply,
      );

  /// The two bridge calls behind overridable seams, as `PlannerNotifier`'s are.
  @protected
  Future<CoverCycleOutcomeDto> runCover(CoverCycleRequestDto request) =>
      coverCycle(request: request);

  @protected
  Future<PlanDecisionOutcomeDto> recordDecision(
    PlanDecisionRequestDto request,
  ) => recordPlanDecision(request: request);

  /// One tap from the result view to a written plan. Publishes the returned outcome —
  /// apply re-plans on the current snapshot, so what is shown afterwards is always what was
  /// applied, and a raced declined apply (`applied: false`) re-renders honestly from the
  /// same result. Invalidating the planner cascades into shopping (its build watches the
  /// planner), so the derived list is ready with no separate generate step (AC-5).
  Future<CoverView> accept() async {
    final householdId = await ref.read(
      householdProvider.selectAsync((h) => h.id),
    );
    final outcome = await runCover(_request(householdId, apply: true));
    final view = CoverView(outcome: outcome);
    state = AsyncData(view);
    ref.invalidate(plannerProvider(arg));
    return view;
  }

  /// Records one decision (mutation + evidence in one command), then re-previews so the
  /// view reflects it. The planner grid is invalidated too: a swap writes a stored row.
  Future<void> decide(PlanDecisionDto decision) async {
    final householdId = await ref.read(
      householdProvider.selectAsync((h) => h.id),
    );
    await recordDecision(
      PlanDecisionRequestDto(
        householdId: householdId,
        today: ref.read(plannerAnchorProvider),
        offsetCycles: arg,
        decision: decision,
      ),
    );
    final outcome = await runCover(_request(householdId, apply: false));
    state = AsyncData(CoverView(outcome: outcome));
    ref.invalidate(plannerProvider(arg));
  }
}

final coverProvider = AsyncNotifierProvider.autoDispose
    .family<CoverNotifier, CoverView, int>(CoverNotifier.new);
