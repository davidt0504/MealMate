import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/pantry/pantry_copy.dart';
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/features/recipes/recipe_fields.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/recipes/restriction_warnings.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

class RecipeDetailScreen extends ConsumerStatefulWidget {
  const RecipeDetailScreen({super.key, required this.recipeId});

  final String recipeId;

  @override
  ConsumerState<RecipeDetailScreen> createState() => _RecipeDetailScreenState();
}

class _RecipeDetailScreenState extends ConsumerState<RecipeDetailScreen> {
  bool _busy = false;
  // Per-row, and separate from `_busy`: marking must not gate Delete/Restore, and one line's
  // write must not freeze the others.
  final Set<IngredientRefDto> _marking = {};

  /// "Delete" is archive (owner decision 2026-08-28), and the dialog says exactly what that
  /// means so the confirmation is informed rather than ritual.
  Future<void> _confirmArchive(RecipeDto r) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Archive this recipe?'),
        content: const Text(
          'It leaves your library and pickers, stays on any past or planned '
          'meals, and can be restored later.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Archive'),
          ),
        ],
      ),
    );
    if (confirmed ?? false) {
      await _run(
        () => ref
            .read(recipeLibraryProvider.notifier)
            .archive(r.householdId, r.id),
      );
    }
  }

  /// One ingredient line, with a pantry control when — and only when — the line resolves to
  /// an identity and the pantry is loaded. An unresolved line has nothing to mark, and
  /// inventing an identity for it would break "identity matching is explicit". A pantry
  /// failure costs the control and nothing else: the recipe is never blocked by pantry state
  /// (AC-2), so the failure is not reported here either.
  ///
  /// Deliberately not routed through `_run`: that navigates to `/recipes` on *success*, which
  /// would throw the reader off the recipe after a successful toggle, and its `_busy` gates
  /// Delete/Restore. Marking keeps its own per-row busy state.
  Widget _lineTile(String householdId, IngredientLineDto line) {
    final plain = ListTile(title: Text(describeLine(line)));
    final reference = line.ingredient;
    if (reference == null) return plain;
    final entries = ref.watch(pantryProvider).valueOrNull;
    if (entries == null) return plain;
    final marked = entries.any((e) => e.ingredient == reference && e.marked);
    return ListTile(
      title: Text(describeLine(line)),
      trailing: Semantics(
        label: pantryRowLabel(line.name, marked),
        child: Switch(
          value: marked,
          onChanged: _marking.contains(reference)
              ? null
              : (next) => _mark(householdId, reference, next),
        ),
      ),
    );
  }

  Future<void> _mark(
    String householdId,
    IngredientRefDto ingredient,
    bool marked,
  ) async {
    setState(() => _marking.add(ingredient));
    try {
      await ref
          .read(pantryProvider.notifier)
          .setMark(householdId, ingredient, marked);
    } catch (e) {
      // The switch renders from provider state, which a failed write never changed, so the
      // row is already back at its stored value; the snackbar says why.
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Pantry'))),
        );
      }
    } finally {
      if (mounted) setState(() => _marking.remove(ingredient));
    }
  }

  Future<void> _run(Future<RecipeDto> Function() action) async {
    setState(() => _busy = true);
    try {
      await action();
      if (mounted) context.go('/recipes');
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Recipes'))),
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final detail = ref.watch(recipeDetailProvider(widget.recipeId));
    final recipe = detail.valueOrNull;
    return Scaffold(
      appBar: AppBar(
        title: Text(recipe?.title ?? 'Recipe'),
        actions: [
          if (recipe != null)
            IconButton(
              icon: const Icon(Icons.edit_outlined),
              tooltip: 'Edit',
              onPressed: () => context.go('/recipes/${recipe.id}/edit'),
            ),
        ],
      ),
      body: switch (detail) {
        AsyncData(value: null) => const Padding(
          padding: EdgeInsets.all(24),
          child: Text('Recipe not found.'),
        ),
        AsyncData(value: final r?) => _body(r),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Text(describeFailure(error, subject: 'Recipes')),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  /// Exactly one of: the warnings card (conflicts found), the unchecked line (no
  /// restrictions set), or the not-a-safety-check line (checked, nothing known found) — plus
  /// the wording-only note whenever an `Other` restriction was in the set.
  List<Widget> _assessment(RestrictionAssessmentDto a) => [
    if (a.conflicts.isNotEmpty)
      Card(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                warningsHeading,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              for (final c in a.conflicts)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Text(describeConflict(c)),
                ),
            ],
          ),
        ),
      )
    else if (a.restrictionsChecked == 0)
      const Text(noRestrictionsCopy)
    else
      const Text(noKnownConflictCopy),
    if (a.wordingOnly.isNotEmpty) Text(wordingOnlyCopy(a.wordingOnly)),
    const SizedBox(height: 16),
  ];

  Widget _body(RecipeDto r) => ListView(
    padding: const EdgeInsets.all(16),
    children: [
      if (r.archivedAt case final date?) ...[
        Text('Archived on $date'),
        const SizedBox(height: 8),
        FilledButton(
          onPressed: _busy
              ? null
              : () => _run(
                  () => ref
                      .read(recipeLibraryProvider.notifier)
                      .restore(r.householdId, r.id),
                ),
          child: const Text('Restore'),
        ),
        const SizedBox(height: 16),
      ],
      // A `null` assessment is impossible on a read-back (`every_read_back_is_some`); it
      // renders nothing rather than a fabricated state.
      if (r.assessment case final a?) ..._assessment(a),
      if (r.servings case final n?) Text('Serves $n'),
      // Rendered only when there is an estimate: no "0 min", no "unknown" (PRD §10).
      if (r.prepMinutes case final n?) Text('Prep $n min'),
      const SizedBox(height: 8),
      Text('Ingredients', style: Theme.of(context).textTheme.titleMedium),
      if (r.lines.isEmpty) const Text('No ingredients listed.'),
      for (final line in r.lines) _lineTile(r.householdId, line),
      const SizedBox(height: 8),
      Text('Instructions', style: Theme.of(context).textTheme.titleMedium),
      Text(r.instructions.isEmpty ? 'No instructions yet.' : r.instructions),
      const SizedBox(height: 24),
      if (r.archivedAt == null)
        OutlinedButton(
          onPressed: _busy ? null : () => _confirmArchive(r),
          child: const Text('Delete'),
        ),
    ],
  );
}
