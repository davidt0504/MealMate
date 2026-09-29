import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/planning/cover_copy.dart';
import 'package:meal_mate/features/planning/cover_provider.dart';
import 'package:meal_mate/features/planning/experiment_provider.dart';
import 'package:meal_mate/features/planning/planner_copy.dart'
    show emptyCellCopy, kindLabel, slotLabel;
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/src/rust/api/experiment.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planner.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/planning_drafts.dart';

/// The Cover My Week draft (OPT-007): an exception console, not a feed. Everything rendered
/// comes from one `CoverView`; the screen holds no durable state and issues coarse commands
/// through the provider (invariants 17, 21). Accept is the one accent action; every other
/// control edits the draft, which Accept writes exactly as shown.
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

String _slotKey(String date, MealSlotDto slot) => '$date:${slot.name}';

bool _hasDish(List<MealComponentDto> components) => components.any(
  (c) =>
      c.kind == 'recipe' ||
      (c.kind == 'freeform' && (c.note ?? '').startsWith('starter:')),
);

class _CoverScreenState extends ConsumerState<CoverScreen> {
  bool _busy = false;
  bool _exposed = false;

  /// Review answers in progress, keyed by date and slot: `true` keeps the saved meal.
  final Map<String, bool> _review = {};

  CoverNotifier get _notifier =>
      ref.read(coverProvider(widget.offset).notifier);

  @override
  Widget build(BuildContext context) {
    final view = ref.watch(coverProvider(widget.offset));
    // One experiment session per visit, alive for the whole screen: review or an accepted
    // view in between does not end it.
    ref.watch(weekActionExperimentProvider);
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
              Text(describeFailure(error, subject: coverTitle)),
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
    final draft = view.draft;
    final attention = draft.status == OutcomeStatusDto.needsAttention;
    final firstRun =
        ref.watch(householdProvider).valueOrNull?.onboarded == false;
    final assumptionLines = [
      for (final code in draft.assumptions) ?assumptionCopy(code),
    ];
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        if (_busy)
          const Padding(
            padding: EdgeInsets.only(bottom: 8),
            child: LinearProgressIndicator(semanticsLabel: 'Updating plan'),
          ),
        Text(
          _banner(draft),
          style: textTheme.titleMedium?.copyWith(
            color: attention ? Theme.of(context).colorScheme.tertiary : null,
          ),
        ),
        if (firstRun) ...[const SizedBox(height: 4), const Text(firstRunCopy)],
        if (view.accepted)
          ..._acceptedSection(draft, firstRun)
        else if (view.needsReview)
          ..._reviewSection(draft)
        else if (draft.state == DraftStateDto.active)
          ..._actions(draft),
        if (draft.pastChangesDropped > 0)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(pastDroppedCopy(draft.pastChangesDropped)),
          ),
        for (final expired in draft.expired) _expiredTile(expired),
        const SizedBox(height: 8),
        for (final slot in draft.slots) _slotCard(view, slot),
        if (!view.accepted &&
            draft.assumptions.contains('RESTRICTIONS_NOT_CONFIGURED'))
          _reviewedRow(),
        if (assumptionLines.isNotEmpty)
          ExpansionTile(
            title: const Text(assumptionsHeading),
            children: [
              for (final line in assumptionLines)
                ListTile(dense: true, title: Text(line)),
            ],
          ),
      ],
    );
  }

  String _banner(DraftViewDto draft) => switch (draft.status) {
    OutcomeStatusDto.covered => coveredCopy,
    OutcomeStatusDto.tentativelyCovered => tentativeCopy,
    OutcomeStatusDto.needsAttention => needsAttentionCopy,
    OutcomeStatusDto.unresolved => unresolvedCopy,
  };

  WeekActionExperiment get _experiment =>
      ref.read(weekActionExperimentProvider.notifier);

  List<Widget> _actions(DraftViewDto draft) {
    final label = ref.watch(weekActionLabelProvider);
    // Exposure is the enabled control actually on screen, once per session: rebuilds do not
    // repeat it (and Rust drops a repeat anyway).
    if (draft.weekTargets > 0 &&
        !_exposed &&
        ref.watch(weekActionExperimentProvider).hasValue) {
      _exposed = true;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _experiment.record(ExperimentEventKindDto.exposure);
      });
    }
    final editable = draft.slots.where((s) => s.editable).toList();
    final committed = editable.any((s) => s.committed);
    final allLocked = editable.isNotEmpty && editable.every((s) => s.committed);
    return [
      if (draft.pendingChanges)
        const Padding(
          padding: EdgeInsets.only(top: 8),
          child: Text(pendingCopy),
        ),
      if (committed)
        const Padding(
          padding: EdgeInsets.only(top: 4),
          child: Text(lockedStayCopy),
        ),
      Padding(
        padding: const EdgeInsets.only(top: 12),
        child: Wrap(
          spacing: 8,
          runSpacing: 8,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            Semantics(
              label: acceptSemanticsCopy,
              child: FilledButton(
                key: const ValueKey('cover:accept'),
                style: FilledButton.styleFrom(
                  backgroundColor: Theme.of(context).colorScheme.tertiary,
                  foregroundColor: Theme.of(context).colorScheme.onTertiary,
                ),
                onPressed: _busy || !draft.acceptAllowed ? null : _accept,
                child: const Text('Accept'),
              ),
            ),
            OutlinedButton(
              key: const ValueKey('cover:week'),
              onPressed: _busy || draft.weekTargets == 0
                  ? null
                  : () => _mutate(const DraftActionDto.alternatives()),
              child: Text(label),
            ),
            if (draft.undoAvailable)
              TextButton(
                key: const ValueKey('cover:undo'),
                onPressed: _busy
                    ? null
                    : () => _mutate(const DraftActionDto.undo()),
                child: const Text('Undo'),
              ),
            if (draft.pendingChanges)
              TextButton(
                key: const ValueKey('cover:discard'),
                onPressed: _busy
                    ? null
                    : () => _mutate(const DraftActionDto.discard()),
                child: const Text('Discard changes'),
              ),
          ],
        ),
      ),
      if (draft.weekTargets == 0)
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Text(allLocked ? allLockedCopy : noSuggestionsToChangeCopy),
        ),
    ];
  }

  List<Widget> _acceptedSection(DraftViewDto draft, bool firstRun) => [
    const SizedBox(height: 4),
    const Text(acceptedCopy),
    Wrap(
      spacing: 8,
      children: [
        TextButton(
          onPressed: () => context.go('/shopping'),
          child: const Text(shoppingReadyCopy),
        ),
        TextButton(
          key: const ValueKey('cover:edit'),
          onPressed: _busy ? null : _notifier.editAgain,
          child: const Text('Edit meals'),
        ),
      ],
    ),
    // Accepted, but the onboarding flag did not flip: retry that alone — the meals are written
    // and never sent again.
    if (firstRun && draft.receipt != null) ...[
      const Text(finishSetupCopy),
      Align(
        alignment: Alignment.centerLeft,
        child: TextButton(
          key: const ValueKey('cover:finish-setup'),
          onPressed: _busy ? null : _finishSetup,
          child: const Text('Finish setup'),
        ),
      ),
    ],
  ];

  List<Widget> _reviewSection(DraftViewDto draft) {
    final conflicts = draft.slots.where((s) => s.inReview).toList();
    if (conflicts.isEmpty) {
      return [
        const Padding(
          padding: EdgeInsets.only(top: 8),
          child: Text(reviewFactsOnlyCopy),
        ),
        Align(
          alignment: Alignment.centerLeft,
          child: FilledButton(
            key: const ValueKey('cover:review'),
            onPressed: _busy
                ? null
                : () => _mutate(const DraftActionDto.review(resolutions: [])),
            child: const Text('Review changes'),
          ),
        ),
      ];
    }
    final complete = conflicts.every(
      (s) => _review.containsKey(_slotKey(s.date, s.slot)),
    );
    return [
      const Padding(
        padding: EdgeInsets.only(top: 8),
        child: Text(reviewIntroCopy),
      ),
      for (final slot in conflicts) _reviewCard(slot),
      Align(
        alignment: Alignment.centerLeft,
        child: FilledButton(
          key: const ValueKey('cover:review'),
          onPressed: _busy || !complete
              ? null
              : () => _mutate(
                  DraftActionDto.review(
                    resolutions: [
                      for (final s in conflicts)
                        ReviewResolutionDto(
                          date: s.date,
                          slot: s.slot,
                          useSaved: _review[_slotKey(s.date, s.slot)]!,
                        ),
                    ],
                  ),
                ),
          child: const Text('Done reviewing'),
        ),
      ),
    ];
  }

  Widget _reviewCard(DraftSlotDto slot) {
    final key = _slotKey(slot.date, slot.slot);
    final saved = slot.saved;
    return Card(
      key: ValueKey('cover:review:$key'),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('${slot.date} · ${slotLabel(slot.slot)}'),
            const SizedBox(height: 4),
            Text(
              savedMealHeading,
              style: Theme.of(context).textTheme.labelLarge,
            ),
            if (saved == null)
              const Text(nothingSavedCopy)
            else ...[
              for (final c in saved.components) Text(_componentLine(c)),
              if (saved.locked) const Text('Locked in'),
            ],
            const SizedBox(height: 4),
            Text(
              yourChoiceHeading,
              style: Theme.of(context).textTheme.labelLarge,
            ),
            for (final c in slot.components) Text(_componentLine(c)),
            const SizedBox(height: 8),
            SegmentedButton<bool>(
              emptySelectionAllowed: true,
              segments: const [
                ButtonSegment(value: true, label: Text('Use saved meal')),
                ButtonSegment(value: false, label: Text('Use my choice')),
              ],
              selected: {?_review[key]},
              onSelectionChanged: _busy
                  ? null
                  : (choice) => setState(() {
                      if (choice.isEmpty) {
                        _review.remove(key);
                      } else {
                        _review[key] = choice.first;
                      }
                    }),
            ),
          ],
        ),
      ),
    );
  }

  Widget _expiredTile(ExpiredDraftDto expired) => ListTile(
    key: ValueKey('cover:expired:${expired.draftId}'),
    contentPadding: EdgeInsets.zero,
    title: Text(expiredDraftCopy(expired.anchor)),
    trailing: TextButton(
      onPressed: _busy ? null : () => _discardExpired(expired.draftId),
      child: const Text('Discard'),
    ),
  );

  Widget _slotCard(CoverView view, DraftSlotDto slot) {
    final theme = Theme.of(context);
    final problem =
        slot.state == CoverageStateDto.needsAttention ||
            slot.reasonCodes.contains('HARD_CONSTRAINT_UNRESOLVED')
        ? slotProblemCopy(slot.reasonCodes)
        : null;
    final actionable =
        view.draft.state == DraftStateDto.active && slot.editable;
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
                if (slot.committed)
                  Semantics(
                    label: 'Locked in',
                    child: const ExcludeSemantics(child: Icon(Icons.lock)),
                  ),
              ],
            ),
            if (slot.components.isEmpty) const Text(emptyCellCopy),
            for (final component in slot.components)
              Text(_componentLine(component)),
            if (problem != null)
              Text(
                problem,
                style: TextStyle(color: theme.colorScheme.tertiary),
              ),
            if (slot.outcome == SlotOutcomeDto.exhausted) ...[
              Text(exhaustedCopy(slot.date)),
              if (actionable)
                Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: _busy ? null : () => _openChooser(slot),
                      child: const Text('Choose'),
                    ),
                    TextButton(
                      key: ValueKey(
                        'cover:reconsider:${slot.date}:${slot.slot.name}',
                      ),
                      onPressed: _busy
                          ? null
                          : () => _mutate(
                              DraftActionDto.reconsider(
                                date: slot.date,
                                slot: slot.slot,
                              ),
                            ),
                      child: const Text('Reconsider'),
                    ),
                  ],
                ),
            ],
            if (slot.outcome == SlotOutcomeDto.blocked)
              const Text(blockedAnotherCopy),
            if (actionable) _slotActions(slot),
          ],
        ),
      ),
    );
  }

  Widget _slotActions(DraftSlotDto slot) {
    final where = '${slot.date} ${slotLabel(slot.slot)}';
    final dish = _hasDish(slot.components);
    return Wrap(
      spacing: 8,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        // The spoken names ride on the text, so they land on the tappable node itself: its tap
        // action, enabled state and selection come from the control and cannot drift from it.
        if (!slot.committed && dish)
          TextButton(
            key: ValueKey('cover:another:${slot.date}:${slot.slot.name}'),
            onPressed: _busy
                ? null
                : () => _mutate(
                    DraftActionDto.another(date: slot.date, slot: slot.slot),
                  ),
            child: Text('Another', semanticsLabel: 'Another meal for $where'),
          ),
        FilterChip(
          key: ValueKey('cover:lock:${slot.date}:${slot.slot.name}'),
          label: Text(
            slot.committed ? 'Locked in' : 'Lock in',
            semanticsLabel: slot.committed
                ? 'Locked in, $where'
                : 'Lock in $where',
          ),
          selected: slot.committed,
          onSelected: _busy
              ? null
              : (locked) => _mutate(
                  DraftActionDto.setCommitment(
                    date: slot.date,
                    slot: slot.slot,
                    locked: locked,
                  ),
                ),
        ),
        PopupMenuButton<String>(
          key: ValueKey('cover:more:${slot.date}:${slot.slot.name}'),
          tooltip: 'More for $where',
          enabled: !_busy,
          onSelected: (choice) => switch (choice) {
            'choose' => _openChooser(slot),
            'unlock' => _mutate(
              DraftActionDto.setCommitment(
                date: slot.date,
                slot: slot.slot,
                locked: false,
              ),
            ),
            'never' => _confirmNeverSuggest(slot),
            _ => null,
          },
          itemBuilder: (_) => [
            PopupMenuItem(
              value: 'choose',
              child: Text(slot.committed ? 'Choose a replacement' : 'Choose'),
            ),
            if (slot.committed)
              const PopupMenuItem(value: 'unlock', child: Text('Unlock')),
            if (dish)
              const PopupMenuItem(value: 'never', child: Text('Never suggest')),
          ],
        ),
      ],
    );
  }

  String? _recipeTitle(String? recipeId) => ref
      .watch(recipeLibraryProvider)
      .valueOrNull
      ?.where((r) => r.id == recipeId)
      .map((r) => r.title)
      .firstOrNull;

  String _componentLine(MealComponentDto component) {
    if (component.kind == 'recipe') {
      return _recipeTitle(component.recipeId) ?? kindLabel(component.kind);
    }
    final note = component.note;
    if (component.kind == 'freeform' &&
        note != null &&
        note.startsWith('starter:')) {
      return _starterTitle(note);
    }
    final label = kindLabel(component.kind);
    return note == null || note.isEmpty ? label : '$label — $note';
  }

  /// A starter not yet installed is a stub naming its slug; the slug is written to be read.
  String _starterTitle(String note) {
    final slug = note.substring('starter:'.length).replaceAll('-', ' ');
    return slug.isEmpty
        ? kindLabel('freeform')
        : slug[0].toUpperCase() + slug.substring(1);
  }

  /// Dish identities and names for a "Never suggest" confirmation. Fallback kinds and free
  /// notes name no dish, so they contribute nothing.
  List<({String identity, String title})> _dishes(DraftSlotDto slot) => [
    for (final c in slot.components)
      if (c.kind == 'recipe' && c.recipeId != null)
        (
          identity: 'recipe:${c.recipeId}',
          title: _recipeTitle(c.recipeId) ?? kindLabel('recipe'),
        )
      else if (c.kind == 'freeform' && (c.note ?? '').startsWith('starter:'))
        (
          identity: 'starter:${c.note!.substring('starter:'.length)}',
          title: _starterTitle(c.note!),
        ),
  ];

  Future<void> _run(Future<void> Function() action) async {
    if (!mounted || _busy) return;
    setState(() => _busy = true);
    try {
      await action();
    } catch (e) {
      _report(e);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _mutate(DraftActionDto action) => _run(() async {
    final before = ref.read(coverProvider(widget.offset)).valueOrNull?.draft;
    if (action is DraftActionDto_Alternatives) {
      _experiment.record(ExperimentEventKindDto.alternativeRequested);
    }
    await _notifier.mutate(action);
    if (action is DraftActionDto_Review) _review.clear();
    _measure(action);
    _announce(before);
  });

  /// Coarse counts only (OPT-007 §10): how many meals changed or ran out, never which.
  void _measure(DraftActionDto action) {
    final operation = ref
        .read(coverProvider(widget.offset))
        .valueOrNull
        ?.draft
        .operation;
    switch (action) {
      case DraftActionDto_Alternatives():
        _experiment.record(
          ExperimentEventKindDto.alternativeResult,
          changed:
              operation?.slots
                  .where((s) => s.outcome == SlotOutcomeDto.changed)
                  .length ??
              0,
          exhausted:
              operation?.slots
                  .where((s) => s.outcome == SlotOutcomeDto.exhausted)
                  .length ??
              0,
        );
      case DraftActionDto_Undo():
        _experiment.record(ExperimentEventKindDto.undo);
      case DraftActionDto_Discard():
        _experiment.record(ExperimentEventKindDto.discard);
      default:
        break;
    }
  }

  /// Says which meals changed, so a screen-reader user hears the result of Another or a week
  /// request without hunting for it.
  void _announce(DraftViewDto? before) {
    final after = ref.read(coverProvider(widget.offset)).valueOrNull?.draft;
    final operation = after?.operation;
    if (!mounted ||
        after == null ||
        operation == null ||
        identical(before, after)) {
      return;
    }
    final lines = <String>[
      for (final s in operation.slots)
        if (s.outcome == SlotOutcomeDto.changed)
          '${s.date}: ${after.slots.where((x) => x.date == s.date && x.slot == s.slot).expand((x) => x.components).map(_componentLine).join(', ')}'
        else if (s.outcome == SlotOutcomeDto.exhausted)
          exhaustedCopy(s.date),
    ];
    if (lines.isEmpty) return;
    SemanticsService.sendAnnouncement(
      View.of(context),
      lines.join('. '),
      Directionality.of(context),
    );
  }

  Future<void> _accept() => _run(() async {
    final accepted = await _notifier.accept();
    // First-run ends on the persisted receipt only: a refused or unsent Accept never
    // completes onboarding.
    if (accepted?.receipt != null) {
      _experiment.record(ExperimentEventKindDto.acceptSuccess, completed: true);
      await widget.completeFirstRun();
    }
  });

  Future<void> _finishSetup() => _run(widget.completeFirstRun);

  Future<void> _discardExpired(String draftId) =>
      _run(() => _notifier.discardExpired(draftId));

  void _report(Object e) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(describeFailure(e, subject: coverTitle))),
    );
  }

  /// The picker: the recipe library plus the fallback kinds. Cancelling changes nothing; a pick
  /// on a locked-in slot was labelled as a replacement before anything was chosen.
  Future<void> _openChooser(DraftSlotDto slot) async {
    final replacing = slot.committed;
    final components = await showModalBottomSheet<List<MealComponentDto>>(
      context: context,
      isScrollControlled: true,
      builder: (sheet) => Consumer(
        builder: (context, ref, _) {
          final recipes = ref.watch(recipeLibraryProvider);
          return SafeArea(
            child: ListView(
              shrinkWrap: true,
              children: [
                ListTile(
                  title: Text(replacing ? replaceSheetTitle : chooseSheetTitle),
                  subtitle: Text(
                    replacing ? replaceWarningCopy : chooseLocksCopy,
                  ),
                ),
                switch (recipes) {
                  AsyncData(:final value) => Column(
                    children: [
                      for (final recipe in value)
                        ListTile(
                          key: ValueKey('cover:pick:${recipe.id}'),
                          title: Text(recipe.title),
                          onTap: () => Navigator.pop(sheet, [
                            MealComponentDto(
                              kind: 'recipe',
                              recipeId: recipe.id,
                            ),
                          ]),
                        ),
                    ],
                  ),
                  AsyncError() => const ListTile(
                    title: Text(chooseLibraryUnavailableCopy),
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
                    onTap: () =>
                        Navigator.pop(sheet, [MealComponentDto(kind: kind)]),
                  ),
              ],
            ),
          );
        },
      ),
    );
    if (components == null) return;
    await _mutate(
      DraftActionDto.choose(
        date: slot.date,
        slot: slot.slot,
        components: components,
        explicitReplace: replacing,
      ),
    );
  }

  Future<void> _confirmNeverSuggest(DraftSlotDto slot) async {
    final dishes = _dishes(slot);
    if (dishes.isEmpty) return;
    final picked = await showDialog<List<String>>(
      context: context,
      builder: (dialog) => _NeverSuggestDialog(dishes: dishes),
    );
    if (picked == null || picked.isEmpty) return;
    await _run(() => _notifier.neverSuggest(picked));
  }

  /// "We don't have any" is a household fact, not a draft edit: it goes through the decision
  /// command and the draft is re-read, keeping every choice.
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
                      : () => _run(_notifier.confirmNoRestrictions),
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

/// Lists every dish the meal names, all ticked; confirming returns the ticked identities.
class _NeverSuggestDialog extends StatefulWidget {
  const _NeverSuggestDialog({required this.dishes});

  final List<({String identity, String title})> dishes;

  @override
  State<_NeverSuggestDialog> createState() => _NeverSuggestDialogState();
}

class _NeverSuggestDialogState extends State<_NeverSuggestDialog> {
  late final Set<String> _ticked = {for (final d in widget.dishes) d.identity};

  @override
  Widget build(BuildContext context) {
    final titles = [
      for (final d in widget.dishes)
        if (_ticked.contains(d.identity)) d.title,
    ];
    return AlertDialog(
      title: const Text(neverSuggestTitle),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (widget.dishes.length > 1) ...[
              const Text(neverSuggestPickCopy),
              for (final d in widget.dishes)
                CheckboxListTile(
                  key: ValueKey('cover:never:${d.identity}'),
                  value: _ticked.contains(d.identity),
                  title: Text(d.title),
                  onChanged: (on) => setState(() {
                    if (on ?? false) {
                      _ticked.add(d.identity);
                    } else {
                      _ticked.remove(d.identity);
                    }
                  }),
                ),
            ],
            if (titles.isNotEmpty) Text(neverSuggestBody(titles)),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: _ticked.isEmpty
              ? null
              : () => Navigator.pop(context, [
                  for (final d in widget.dishes)
                    if (_ticked.contains(d.identity)) d.identity,
                ]),
          child: const Text('Never suggest'),
        ),
      ],
    );
  }
}
