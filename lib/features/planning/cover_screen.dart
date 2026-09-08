import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/planning/cover_copy.dart';
import 'package:meal_mate/features/planning/cover_provider.dart';
import 'package:meal_mate/features/planning/planner_copy.dart'
    show lockLabel, slotLabel, kindLabel;
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/src/rust/api/decisions.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planner.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

/// The Cover My Week result view (MVP-024): an exception console, not a feed. Everything
/// rendered comes from one `CoverView`; the screen holds no durable state and issues coarse
/// commands through the provider (invariants 17, 21).
typedef CompleteFirstRun = Future<void> Function();

class CoverScreen extends ConsumerStatefulWidget {
  const CoverScreen({
    super.key,
    required this.offset,
    required this.completeFirstRun,
  });

  final int offset;
  final CompleteFirstRun completeFirstRun;

  @override
  ConsumerState<CoverScreen> createState() => _CoverScreenState();
}

class _CoverScreenState extends ConsumerState<CoverScreen> {
  bool _busy = false;
  bool _accepted = false;

  CoverNotifier get _notifier =>
      ref.read(coverProvider(widget.offset).notifier);

  @override
  Widget build(BuildContext context) {
    final view = ref.watch(coverProvider(widget.offset));
    return Scaffold(
      appBar: AppBar(title: const Text(coverTitle)),
      body: switch (view) {
        AsyncData(:final value) => _body(value),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(describeFailure(error, subject: 'Cover My Week')),
              const SizedBox(height: 16),
              FilledButton(
                onPressed: () => ref.invalidate(coverProvider(widget.offset)),
                child: const Text('Try again'),
              ),
            ],
          ),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Widget _body(CoverView view) {
    final textTheme = Theme.of(context).textTheme;
    final result = view.result;
    final high = result.attention
        .where((a) => a.urgency == UrgencyDto.high)
        .toList();
    final low = result.attention
        .where((a) => a.urgency == UrgencyDto.low)
        .toList();
    final needsYou = high.isNotEmpty;
    final assumptionLines = [
      for (final code in result.assumptions) ?assumptionCopy(code),
    ];
    final showAccept = result.status != OutcomeStatusDto.needsAttention;
    final firstRun =
        ref.watch(householdProvider).valueOrNull?.onboarded == false;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(
          _banner(view),
          style: textTheme.titleMedium?.copyWith(
            color: needsYou ? Theme.of(context).colorScheme.tertiary : null,
          ),
        ),
        if (firstRun) ...[const SizedBox(height: 4), const Text(firstRunCopy)],
        if (_accepted && view.outcome.applied) ...[
          const SizedBox(height: 4),
          Align(
            alignment: Alignment.centerLeft,
            child: TextButton(
              onPressed: () => context.go('/shopping'),
              child: const Text(shoppingReadyCopy),
            ),
          ),
        ],
        if (high.isNotEmpty) ...[
          Padding(
            padding: const EdgeInsets.only(top: 16, bottom: 4),
            child: Text(questionsHeading, style: textTheme.titleSmall),
          ),
          for (final request in high) _questionCard(request, result.slots),
        ],
        if (showAccept)
          Padding(
            padding: const EdgeInsets.only(top: 12),
            child: Semantics(
              label: acceptSemanticsCopy,
              child: FilledButton(
                style: FilledButton.styleFrom(
                  backgroundColor: Theme.of(context).colorScheme.tertiary,
                  foregroundColor: Theme.of(context).colorScheme.onTertiary,
                ),
                onPressed: _busy ? null : _accept,
                child: const Text('Accept'),
              ),
            ),
          ),
        for (final slot in result.slots) _slotTile(slot),
        if (result.assumptions.contains('RESTRICTIONS_NOT_CONFIGURED'))
          _reviewedRow(),
        if (assumptionLines.isNotEmpty)
          ExpansionTile(
            title: Text(assumptionsHeading),
            children: [
              for (final line in assumptionLines)
                ListTile(dense: true, title: Text(line)),
            ],
          ),
        if (low.isNotEmpty) ...[
          Padding(
            padding: const EdgeInsets.only(top: 16, bottom: 4),
            child: Text(quietHeading, style: textTheme.titleSmall),
          ),
          for (final request in low)
            ListTile(dense: true, title: Text(_questionText(request))),
        ],
      ],
    );
  }

  String _banner(CoverView view) {
    final n = view.highUrgencyCount;
    if (n > 0) return needsYouCopy(n);
    return switch (view.result.status) {
      OutcomeStatusDto.covered => coveredCopy,
      OutcomeStatusDto.tentativelyCovered => tentativeCopy,
      OutcomeStatusDto.needsAttention => needsYouCopy(n),
      OutcomeStatusDto.unresolved => unresolvedCopy,
    };
  }

  String _questionText(AttentionRequestDto request) {
    if (request.reasonCodes.contains('PLAN_INFEASIBLE')) {
      return infeasibleQuestionCopy;
    }
    if (request.reasonCodes.contains('HARD_CONSTRAINT_UNRESOLVED')) {
      return wordingQuestionCopy;
    }
    // The Low `PREFERENCES_SPARSE` options are already user language from the engine.
    if (request.options.isNotEmpty &&
        request.reasonCodes.contains('PREFERENCES_SPARSE')) {
      return request.options.first;
    }
    return fallbackQuestionCopy;
  }

  /// A wording question is raised from the chosen candidates with no filter on slot state, and
  /// a locked candidate stays feasible at Tier 0, so a locked slot reaches this card. Every
  /// affordance here ends in a swap, which `record_decision` refuses on a locked row
  /// (invariant 18), so the whole [Wrap] is gated the way [_slotTile] gates its own Row —
  /// offering an action that can only fail is worse than offering none. A question addressing
  /// a slot [slots] does not carry has no lock state to read and keeps its affordances.
  Widget _questionCard(
    AttentionRequestDto request,
    List<SlotCoverageDto> slots,
  ) {
    final at = questionSlot(request.id);
    final titles = ref.watch(recipeLibraryProvider).valueOrNull;
    final addressed = at == null
        ? null
        : slots
              .where((s) => s.date == at.date && s.slot == at.slot)
              .firstOrNull;
    final locked = addressed?.state == CoverageStateDto.lockedByUser;
    return Card(
      key: ValueKey('cover:question:${request.id}'),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (at != null) Text('${at.date} · ${slotLabel(at.slot)}'),
            Text(_questionText(request)),
            if (at != null && locked)
              Row(
                children: [
                  // Decorative: the sentence beside it carries the meaning, so a second
                  // semantic node here would read the lock out twice.
                  const ExcludeSemantics(child: Icon(Icons.lock)),
                  const SizedBox(width: 8),
                  const Expanded(child: Text(questionLockedCopy)),
                ],
              ),
            if (at != null && !locked)
              Wrap(
                spacing: 8,
                children: [
                  for (final option in request.options)
                    ?_optionChip(option, at, titles),
                  ActionChip(
                    label: const Text('Swap'),
                    onPressed: _busy
                        ? null
                        : () => _openSwapPicker(at.date, at.slot),
                  ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  /// A `recipe:` option whose title the library holds swaps directly — the id is mechanical.
  /// Every other option kind embeds free text or a multi-component join, neither of which is
  /// renderable as a label (raw tokens never render), so it contributes no chip at all: the
  /// card's own "Swap" chip is the affordance for those, and a chip repeating the card's own
  /// date·slot header named neither the option nor the action.
  Widget? _optionChip(
    String option,
    ({String date, MealSlotDto slot}) at,
    List<RecipeSummaryDto>? titles,
  ) {
    final recipeId = optionRecipeId(option);
    final title = recipeId == null
        ? null
        : titles
              ?.where((r) => r.id == recipeId)
              .map((r) => r.title)
              .firstOrNull;
    if (recipeId == null || title == null) return null;
    return ActionChip(
      label: Text(swapToLabel(title)),
      onPressed: _busy
          ? null
          : () => _decide(
              PlanDecisionDto.swap(
                date: at.date,
                slot: at.slot,
                components: [
                  MealComponentDto(kind: 'recipe', recipeId: recipeId),
                ],
              ),
            ),
    );
  }

  Widget _slotTile(SlotCoverageDto slot) {
    final locked = slot.state == CoverageStateDto.lockedByUser;
    return Card(
      key: ValueKey('cover:${slot.date}:${slot.slot.name}'),
      child: Padding(
        padding: const EdgeInsets.all(8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(child: Text('${slot.date} · ${slotLabel(slot.slot)}')),
                if (locked)
                  Semantics(
                    label: lockLabel(true),
                    child: const ExcludeSemantics(child: Icon(Icons.lock)),
                  ),
              ],
            ),
            for (final component in slot.components)
              Text(_componentLine(component)),
            if (!locked)
              Row(
                children: [
                  TextButton(
                    onPressed: _busy
                        ? null
                        : () => _openSwapPicker(slot.date, slot.slot),
                    child: const Text('Swap'),
                  ),
                  if (_vetoSubject(slot) case final subject?)
                    TextButton(
                      onPressed: _busy
                          ? null
                          : () => _confirmVeto(subject, slot.date, slot.slot),
                      child: const Text('Never suggest'),
                    ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  String _componentLine(MealComponentDto component) {
    if (component.kind == 'recipe') {
      final titles = ref.watch(recipeLibraryProvider).valueOrNull;
      final title = titles
          ?.where((r) => r.id == component.recipeId)
          .map((r) => r.title)
          .firstOrNull;
      return title ?? kindLabel(component.kind);
    }
    final note = component.note;
    final label = kindLabel(component.kind);
    return note == null || note.isEmpty ? label : '$label — $note';
  }

  /// A veto needs a subject in the household's words; the chosen recipe's title is that
  /// subject. Placeholder kinds have no dish to veto. The title is not a dish handle once it
  /// reaches the engine: Tier 0 matches it as a whole-token phrase against every candidate's
  /// title and every ingredient line name, so a one-word title rejects every recipe listing
  /// that word. [vetoConfirmBody] states that before consent is taken — this is the only
  /// surface that mints `food.hard_veto` rows, and no surface removes them.
  String? _vetoSubject(SlotCoverageDto slot) {
    for (final component in slot.components) {
      if (component.kind == 'recipe') {
        final titles = ref.watch(recipeLibraryProvider).valueOrNull;
        final title = titles
            ?.where((r) => r.id == component.recipeId)
            .map((r) => r.title)
            .firstOrNull;
        if (title != null) return title;
      }
    }
    return null;
  }

  Future<void> _accept() async {
    if (!mounted) return;
    setState(() => _busy = true);
    try {
      await _notifier.accept();
      await widget.completeFirstRun();
      if (mounted) setState(() => _accepted = true);
    } catch (e) {
      _report(e);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _decide(PlanDecisionDto decision) async {
    // Reached after an await that can outlive the route — the swap sheet and the veto dialog
    // both resolve into here — so the opening `setState` guards like every other one in this
    // file. The decision is deliberately abandoned rather than written through a disposed
    // state: `_notifier` is a `ref.read` on this `ConsumerState`, so preserving the write
    // would mean capturing the notifier before each await, which is more plumbing than the
    // race is worth. Today the same race throws instead, and loses the decision anyway.
    if (!mounted) return;
    setState(() => _busy = true);
    try {
      await _notifier.decide(decision);
    } catch (e) {
      _report(e);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _report(Object e) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(describeFailure(e, subject: 'Cover My Week'))),
    );
  }

  /// The MVP-013-style picker: the recipe library plus the fallback kinds, feeding one
  /// swap decision. [swapLocksCopy] states the lock effect before anything is chosen.
  Future<void> _openSwapPicker(String date, MealSlotDto slot) async {
    final decision = await showModalBottomSheet<PlanDecisionDto>(
      context: context,
      builder: (sheet) => Consumer(
        builder: (context, ref, _) {
          final recipes = ref.watch(recipeLibraryProvider);
          return SafeArea(
            child: ListView(
              shrinkWrap: true,
              children: [
                ListTile(
                  title: Text(swapSheetTitle),
                  subtitle: const Text(swapLocksCopy),
                ),
                switch (recipes) {
                  AsyncData(:final value) => Column(
                    children: [
                      for (final recipe in value)
                        ListTile(
                          key: ValueKey('cover:pick:${recipe.id}'),
                          title: Text(recipe.title),
                          onTap: () => Navigator.pop(
                            sheet,
                            PlanDecisionDto.swap(
                              date: date,
                              slot: slot,
                              components: [
                                MealComponentDto(
                                  kind: 'recipe',
                                  recipeId: recipe.id,
                                ),
                              ],
                            ),
                          ),
                        ),
                    ],
                  ),
                  AsyncError() => const ListTile(
                    title: Text(swapLibraryUnavailableCopy),
                  ),
                  _ => const Center(child: CircularProgressIndicator()),
                },
                for (final kind in const [
                  'leftovers',
                  'dining_out',
                  'frozen_quick',
                  'open',
                ])
                  ListTile(
                    key: ValueKey('cover:pick:$kind'),
                    title: Text(kindLabel(kind)),
                    onTap: () => Navigator.pop(
                      sheet,
                      PlanDecisionDto.swap(
                        date: date,
                        slot: slot,
                        components: [MealComponentDto(kind: kind)],
                      ),
                    ),
                  ),
              ],
            ),
          );
        },
      ),
    );
    if (decision != null) await _decide(decision);
  }

  Future<void> _confirmVeto(
    String subject,
    String date,
    MealSlotDto slot,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialog) => AlertDialog(
        title: Text(vetoConfirmTitle(subject)),
        content: Text(vetoConfirmBody(subject)),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialog, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialog, true),
            child: const Text('Never suggest'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    await _decide(
      PlanDecisionDto.veto(subject: subject, date: date, slot: slot),
    );
  }

  Widget _reviewedRow() {
    return Card(
      key: const ValueKey('cover:reviewed-row'),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(reviewedRowCopy),
            Wrap(
              spacing: 8,
              children: [
                TextButton(
                  onPressed: () => context.go('/settings/restrictions'),
                  child: const Text(reviewedRowSetLabel),
                ),
                TextButton(
                  onPressed: _busy
                      ? null
                      : () => _decide(
                          const PlanDecisionDto.restrictionsReviewed(),
                        ),
                  child: const Text(reviewedRowNoneLabel),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
