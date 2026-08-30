import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/planning/planner_copy.dart';
import 'package:meal_mate/features/planning/planner_provider.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/recipes/restriction_warnings.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

/// The five scale presets the sheet offers; `null` is the bridge's "as written".
const scalePresets = <ScaleDto?>[
  ScaleDto(numer: 1, denom: 2),
  null,
  ScaleDto(numer: 3, denom: 2),
  ScaleDto(numer: 2, denom: 1),
  ScaleDto(numer: 3, denom: 1),
];

enum _ComponentAction { move, scale, remove }

/// Manual planning: the configured cycle × its enabled slots, one card per cell, every
/// action a button (card: "accessible without drag-only dependence"). The window and the
/// occurrences come from Rust whole; the only view state here is which cycle is shown.
class PlannerScreen extends ConsumerStatefulWidget {
  const PlannerScreen({super.key});

  @override
  ConsumerState<PlannerScreen> createState() => _PlannerScreenState();
}

class _PlannerScreenState extends ConsumerState<PlannerScreen> {
  int _offset = 0;
  // Per-cell, as the pantry's per-row set: one cell's write must not freeze the others.
  final Set<(String, MealSlotDto)> _writing = {};

  PlannerNotifier get _notifier => ref.read(plannerProvider(_offset).notifier);

  void _report(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(describeFailure(error, subject: 'Plan'))),
    );
  }

  /// Both refresh affordances; `PlannerNotifier.refresh` returns the failure rather than
  /// publishing it over a plan the user is reading, so it lands in the same snackbar.
  Future<void> _refresh() async {
    final error = await _notifier.refresh();
    if (error != null) _report(error);
  }

  Future<void> _run(
    String date,
    MealSlotDto slot,
    Future<void> Function() write,
  ) async {
    setState(() => _writing.add((date, slot)));
    try {
      await write();
    } catch (e) {
      _report(e);
    } finally {
      if (mounted) setState(() => _writing.remove((date, slot)));
    }
  }

  Future<void> _add(
    String householdId,
    String date,
    MealSlotDto slot,
    PlannedMealDto? existing,
  ) async {
    final picked = await showModalBottomSheet<MealComponentDto>(
      context: context,
      isScrollControlled: true,
      builder: (_) =>
          _ComponentPicker(cellEmpty: existing?.components.isEmpty ?? true),
    );
    if (picked == null) return;
    await _run(
      date,
      slot,
      () => _notifier.save(
        PlannedMealDto(
          id: existing?.id ?? '',
          householdId: householdId,
          date: date,
          slot: slot,
          components: [...?existing?.components, picked],
          locked: existing?.locked ?? false,
        ),
      ),
    );
  }

  Future<void> _move(PlannerView view, PlannedMealDto meal) async {
    final target = await showModalBottomSheet<(String, MealSlotDto)>(
      context: context,
      builder: (_) => _MoveSheet(view: view, from: (meal.date, meal.slot)),
    );
    if (target == null) return;
    await _run(
      meal.date,
      meal.slot,
      () => _notifier.save(
        PlannedMealDto(
          id: meal.id,
          householdId: meal.householdId,
          date: target.$1,
          slot: target.$2,
          components: meal.components,
          locked: meal.locked,
        ),
      ),
    );
  }

  Future<void> _scale(PlannedMealDto meal, int index) async {
    final current = meal.components[index];
    final chosen = await showModalBottomSheet<(ScaleDto?,)>(
      context: context,
      builder: (_) => _ScaleSheet(current: current.scale),
    );
    if (chosen == null) return;
    final components = [...meal.components];
    components[index] = MealComponentDto(
      kind: current.kind,
      recipeId: current.recipeId,
      note: current.note,
      scale: chosen.$1,
    );
    await _run(
      meal.date,
      meal.slot,
      () => _notifier.save(
        PlannedMealDto(
          id: meal.id,
          householdId: meal.householdId,
          date: meal.date,
          slot: meal.slot,
          components: components,
          locked: meal.locked,
        ),
      ),
    );
  }

  /// Removing the last component deletes the occurrence (the domain refuses an empty one),
  /// which is a different thing from trimming a meal — so it is confirmed.
  Future<void> _remove(PlannedMealDto meal, int index) async {
    if (meal.components.length == 1) {
      final confirmed = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text(removeLastComponentTitle),
          content: const Text(removeLastComponentBody),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(false),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(true),
              child: const Text('Remove'),
            ),
          ],
        ),
      );
      if (!(confirmed ?? false)) return;
      await _run(
        meal.date,
        meal.slot,
        () => _notifier.delete(meal.householdId, meal.id),
      );
      return;
    }
    final components = [
      for (final (i, c) in meal.components.indexed)
        if (i != index) c,
    ];
    await _run(
      meal.date,
      meal.slot,
      () => _notifier.save(
        PlannedMealDto(
          id: meal.id,
          householdId: meal.householdId,
          date: meal.date,
          slot: meal.slot,
          components: components,
          locked: meal.locked,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final view = ref.watch(plannerProvider(_offset));
    // Warmed here, not at first use: nothing else on the way to `/plan` (it is `homeLocation`)
    // reads them, so without this the first frame that paints a recipe component is the frame
    // that *starts* the library read, and the picker on an empty plan opens colder still.
    // Resolving them is not required for correctness — `_lookup` and the picker both
    // distinguish unresolved from empty — this only shortens the window in which they are.
    ref.watch(recipeLibraryProvider);
    ref.watch(archivedRecipesProvider);
    ref.watch(mealComponentKindsProvider);
    return Scaffold(
      appBar: AppBar(
        title: const Text(plannerTitle),
        actions: [
          IconButton(
            tooltip: 'Previous cycle',
            icon: const Icon(Icons.chevron_left),
            onPressed: () => setState(() => _offset--),
          ),
          IconButton(
            tooltip: 'Next cycle',
            icon: const Icon(Icons.chevron_right),
            onPressed: () => setState(() => _offset++),
          ),
        ],
      ),
      body: switch (view) {
        AsyncData(:final value) => _body(value),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(describeFailure(error, subject: 'Plan')),
              const SizedBox(height: 16),
              FilledButton(onPressed: _refresh, child: const Text('Try again')),
            ],
          ),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Widget _body(PlannerView view) {
    final householdId = ref.watch(householdProvider).valueOrNull?.id;
    // Unreachable today for the reason the pantry gives; decided once here.
    if (householdId == null) {
      return const Center(child: CircularProgressIndicator());
    }
    final textTheme = Theme.of(context).textTheme;
    return RefreshIndicator(
      onRefresh: _refresh,
      child: ListView(
        padding: const EdgeInsets.all(16),
        // As the pantry: an empty plan is the shortest list, and default physics would make
        // pull-to-refresh inert exactly there.
        physics: const AlwaysScrollableScrollPhysics(),
        children: [
          Text(windowHeading(view.window), style: textTheme.titleMedium),
          // In the body, not the app bar: with the two cycle arrows, the app bar overflows
          // at text scale 2.0 on Pixel-5 width (measured, 85px).
          Align(
            alignment: Alignment.centerLeft,
            child: TextButton(
              onPressed: () => context.go('/plan/cover'),
              child: const Text('Cover My Week'),
            ),
          ),
          if (view.meals.isEmpty) ...[
            const SizedBox(height: 8),
            const Text(plannerEmptyCopy),
          ],
          for (final date in view.window.dates) ...[
            Padding(
              padding: const EdgeInsets.only(top: 16, bottom: 4),
              child: Text(date, style: textTheme.titleSmall),
            ),
            for (final slot in view.window.mealSlots)
              _cell(householdId, view, date, slot),
          ],
        ],
      ),
    );
  }

  Widget _cell(
    String householdId,
    PlannerView view,
    String date,
    MealSlotDto slot,
  ) {
    final meal = view.at(date, slot);
    final busy = _writing.contains((date, slot));
    final addButton = FilledButton.tonal(
      onPressed: busy ? null : () => _add(householdId, date, slot, meal),
      child: const Text('Add'),
    );
    return Card(
      key: ValueKey('cell:$date:${slot.name}'),
      child: Padding(
        padding: const EdgeInsets.all(8),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Text(slotLabel(slot)),
            ),
            if (meal == null) ...[
              const Padding(
                padding: EdgeInsets.symmetric(horizontal: 8),
                child: Text(emptyCellCopy),
              ),
              addButton,
            ] else ...[
              // The label goes *inside* the tile for the reason `pantry_screen.dart` gives.
              SwitchListTile(
                title: Semantics(
                  label: lockLabel(meal.locked),
                  child: const ExcludeSemantics(child: Text('Locked')),
                ),
                value: meal.locked,
                // A locked meal stays editable by the user: the lock binds automation only
                // (invariant 18), so nothing below is gated on it.
                onChanged: busy
                    ? null
                    : (locked) => _run(
                        date,
                        slot,
                        () => _notifier.setLock(householdId, meal.id, locked),
                      ),
              ),
              for (final (i, c) in meal.components.indexed)
                _componentTile(view, meal, i, c, busy),
              if (!meal.components.any((c) => c.kind == 'open')) addButton,
            ],
          ],
        ),
      ),
    );
  }

  Widget _componentTile(
    PlannerView view,
    PlannedMealDto meal,
    int index,
    MealComponentDto c,
    bool busy,
  ) {
    final found = c.recipeId == null ? null : _lookup(c.recipeId!);
    final summary = found?.summary;
    final conflicts =
        summary != null && summary.assessment.conflicts.isNotEmpty;
    final title = c.kind == 'recipe'
        ? '${found!.title} · ${describeScale(c.scale)}'
        : kindLabel(c.kind);
    final subtitle = conflicts
        ? summariseConflicts(summary.assessment)
        : c.note;
    return ListTile(
      title: Text(title),
      subtitle: subtitle == null ? null : Text(subtitle),
      // A warning is explainable from the recipe it came from (AC-2).
      onTap: conflicts ? () => context.go('/recipes/${c.recipeId}') : null,
      trailing: PopupMenuButton<_ComponentAction>(
        enabled: !busy,
        itemBuilder: (_) => [
          const PopupMenuItem(
            value: _ComponentAction.move,
            child: Text('Move…'),
          ),
          if (c.kind == 'recipe')
            const PopupMenuItem(
              value: _ComponentAction.scale,
              child: Text('Scale…'),
            ),
          const PopupMenuItem(
            value: _ComponentAction.remove,
            child: Text('Remove'),
          ),
        ],
        onSelected: (action) => switch (action) {
          _ComponentAction.move => _move(view, meal),
          _ComponentAction.scale => _scale(meal, index),
          _ComponentAction.remove => _remove(meal, index),
        },
      ),
    );
  }

  /// Both recipe providers, in one pass. An archived recipe keeps its assessment: it stays on
  /// the meal it was planned for (MVP-012 decision gate), so its restriction warning has to
  /// stay with it too (AC-2, invariant 10) — searching only the active library is what made
  /// the planner the one surface that dropped it.
  _PlannedRecipe _lookup(String recipeId) {
    final library = ref.watch(recipeLibraryProvider);
    final archived = ref.watch(archivedRecipesProvider);
    for (final r in library.valueOrNull ?? const <RecipeSummaryDto>[]) {
      if (r.id == recipeId) return _PlannedRecipe(r.title, r);
    }
    for (final r in archived.valueOrNull ?? const <RecipeSummaryDto>[]) {
      if (r.id == recipeId) {
        return _PlannedRecipe(archivedRecipeCopy(r.title), r);
      }
    }
    // Only both providers together can say "in neither", and only once both have produced a
    // list. Until then the id may be in the half that has not arrived — and a failed read
    // knows nothing at all. Carrying no summary is what keeps either case from implying
    // "no conflicts", which is the same silence the archived case was.
    if (library.hasError || archived.hasError) {
      return const _PlannedRecipe(unreadRecipeCopy, null);
    }
    if (library.valueOrNull == null || archived.valueOrNull == null) {
      return const _PlannedRecipe(pendingRecipeCopy, null);
    }
    return const _PlannedRecipe(unknownRecipeCopy, null);
  }
}

/// One planned recipe id resolved against both libraries.
class _PlannedRecipe {
  const _PlannedRecipe(this.title, this.summary);

  /// Already marked if archived, already honest if unknown, unread or still arriving.
  final String title;

  /// `null` unless a library actually holds the recipe, so nothing downstream can read a
  /// missing assessment as an absent conflict.
  final RecipeSummaryDto? summary;
}

/// Recipe first, then the other kinds. `open` is offered only for an empty cell — the
/// domain refuses it beside anything else — and `freeform` needs its note before it pops.
class _ComponentPicker extends ConsumerStatefulWidget {
  const _ComponentPicker({required this.cellEmpty});

  final bool cellEmpty;

  @override
  ConsumerState<_ComponentPicker> createState() => _ComponentPickerState();
}

class _ComponentPickerState extends ConsumerState<_ComponentPicker> {
  final _note = TextEditingController();
  bool _noteMissing = false;

  @override
  void dispose() {
    _note.dispose();
    super.dispose();
  }

  void _pick(String kind) {
    final note = _note.text.trim();
    if (kind == 'freeform' && note.isEmpty) {
      setState(() => _noteMissing = true);
      return;
    }
    Navigator.of(context)
        .pop(MealComponentDto(kind: kind, note: note.isEmpty ? null : note));
  }

  /// A section whose provider has not produced a list: the failure if it failed, otherwise a
  /// spinner. Never the empty-state line — "you have none" is a claim only a finished read
  /// can make, and on the first-run path this sheet is opened before either read has started.
  Widget _placeholder(AsyncValue<Object?> value, String unread) =>
      value.hasError
      ? Text(unread)
      : const Padding(
          padding: EdgeInsets.symmetric(vertical: 8),
          child: Center(child: CircularProgressIndicator()),
        );

  @override
  Widget build(BuildContext context) {
    final library = ref.watch(recipeLibraryProvider);
    final kindVocabulary = ref.watch(mealComponentKindsProvider);
    final recipes = library.valueOrNull;
    final kinds = kindVocabulary.valueOrNull;
    final textTheme = Theme.of(context).textTheme;
    return SafeArea(
      child: SizedBox(
        height: MediaQuery.sizeOf(context).height * 0.8,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text(recipesHeading, style: textTheme.titleMedium),
            if (recipes == null)
              _placeholder(library, libraryUnreadCopy)
            else if (recipes.isEmpty)
              const Text(emptyLibraryCopy),
            for (final r in recipes ?? const <RecipeSummaryDto>[])
              ListTile(
                title: Text(r.title),
                subtitle: r.assessment.conflicts.isEmpty
                    ? null
                    : Text(summariseConflicts(r.assessment)),
                onTap: () =>
                    Navigator.of(context)
                        .pop(MealComponentDto(kind: 'recipe', recipeId: r.id)),
              ),
            const SizedBox(height: 16),
            Text(otherHeading, style: textTheme.titleMedium),
            TextField(
              controller: _note,
              decoration: InputDecoration(
                labelText: 'Note',
                errorText: _noteMissing ? noteRequiredCopy : null,
              ),
            ),
            if (kinds == null) _placeholder(kindVocabulary, kindsUnreadCopy),
            for (final kind in kinds ?? const <String>[])
              if (kind != 'recipe' && (kind != 'open' || widget.cellEmpty))
                ListTile(
                  title: Text(kindLabel(kind)),
                  onTap: () => _pick(kind),
                ),
          ],
        ),
      ),
    );
  }
}

/// Only empty enabled cells in the shown window: storage refuses an occupied `(date, slot)`.
class _MoveSheet extends StatelessWidget {
  const _MoveSheet({required this.view, required this.from});

  final PlannerView view;
  final (String, MealSlotDto) from;

  @override
  Widget build(BuildContext context) {
    final targets = [
      for (final date in view.window.dates)
        for (final slot in view.window.mealSlots)
          if ((date, slot) != from && view.at(date, slot) == null) (date, slot),
    ];
    return SafeArea(
      child: ListView(
        shrinkWrap: true,
        padding: const EdgeInsets.all(16),
        children: [
          Text(moveToHeading, style: Theme.of(context).textTheme.titleMedium),
          if (targets.isEmpty) const Text(noFreeSlotCopy),
          for (final t in targets)
            ListTile(
              title: Text('${t.$1} · ${slotLabel(t.$2)}'),
              onTap: () => Navigator.of(context).pop(t),
            ),
        ],
      ),
    );
  }
}

class _ScaleSheet extends StatelessWidget {
  const _ScaleSheet({required this.current});

  final ScaleDto? current;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: ListView(
        shrinkWrap: true,
        padding: const EdgeInsets.all(16),
        children: [
          Text(scaleHeading, style: Theme.of(context).textTheme.titleMedium),
          for (final preset in scalePresets)
            ListTile(
              title: Text(describeScale(preset)),
              trailing: preset == current ? const Icon(Icons.check) : null,
              // Wrapped in a record so "as written" (`null`) survives the pop.
              onTap: () => Navigator.of(context).pop((preset,)),
            ),
        ],
      ),
    );
  }
}
