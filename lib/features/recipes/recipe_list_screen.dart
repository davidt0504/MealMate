import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/recipes/restriction_warnings.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

/// The warning line under a title, or nothing: absence of a subtitle means "no known
/// conflict", which the detail screen spells out; it never means "checked and clear".
Widget? _conflictSubtitle(RecipeSummaryDto r) => r.assessment.conflicts.isEmpty
    ? null
    : Text(summariseConflicts(r.assessment));

/// The library: active recipes only (PRD §16). Archived ones live one tap away.
class RecipeListScreen extends ConsumerStatefulWidget {
  const RecipeListScreen({super.key});

  @override
  ConsumerState<RecipeListScreen> createState() => _RecipeListScreenState();
}

class _RecipeListScreenState extends ConsumerState<RecipeListScreen> {
  /// View state, not stored: Flutter owns it (PRD §13); Rust owns the assessment.
  bool _hideConflicts = false;

  List<Widget> _library(List<RecipeSummaryDto> value) {
    // With no restrictions set nothing was checked, and a filter control would imply a
    // check exists (PRD §10, invariant 19) — so the chip only appears once one does.
    final anyChecked = value.any((r) => r.assessment.restrictionsChecked > 0);
    final visible = _hideConflicts
        ? value.where((r) => r.assessment.conflicts.isEmpty).toList()
        : value;
    final hidden = value.length - visible.length;
    return [
      if (anyChecked)
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 0),
          child: Align(
            alignment: Alignment.centerLeft,
            child: FilterChip(
              label: const Text(hideConflictsLabel),
              selected: _hideConflicts,
              onSelected: (v) => setState(() => _hideConflicts = v),
            ),
          ),
        ),
      // Whenever the filter is engaged, not only when it hid something: an engaged filter
      // over a full list is the state that reads most like "checked and cleared", so it is
      // the one that most needs the disclaimer (AC-4, invariant 10). Still gated on
      // `anyChecked` for the same reason the chip is — with nothing checked, a line saying
      // nothing matched would itself imply a check ran (PRD §10, invariant 19).
      if (anyChecked && _hideConflicts)
        ListTile(
          subtitle: Text(
            hidden > 0 ? hiddenCountCopy(hidden) : nothingHiddenCopy,
          ),
        ),
      for (final r in visible)
        ListTile(
          title: Text(r.title),
          subtitle: _conflictSubtitle(r),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => context.go('/recipes/${r.id}'),
        ),
    ];
  }

  @override
  Widget build(BuildContext context) {
    final recipes = ref.watch(recipeLibraryProvider);
    return Scaffold(
      appBar: AppBar(
        title: const Text('Recipes'),
        actions: [
          IconButton(
            icon: const Icon(Icons.inventory_2_outlined),
            tooltip: 'Archived recipes',
            onPressed: () => context.go('/recipes/archived'),
          ),
        ],
      ),
      body: switch (recipes) {
        AsyncData(value: final list) when list.isEmpty => const Center(
          child: Text('No recipes yet.'),
        ),
        AsyncData(:final value) => ListView(children: _library(value)),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Text(describeFailure(error, subject: 'Recipes')),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => context.go('/recipes/new'),
        icon: const Icon(Icons.add),
        label: const Text('New recipe'),
      ),
    );
  }
}

class ArchivedRecipesScreen extends ConsumerStatefulWidget {
  const ArchivedRecipesScreen({super.key});

  @override
  ConsumerState<ArchivedRecipesScreen> createState() =>
      _ArchivedRecipesScreenState();
}

class _ArchivedRecipesScreenState extends ConsumerState<ArchivedRecipesScreen> {
  bool _busy = false;

  Future<void> _restore(RecipeSummaryDto r) async {
    final householdId = ref.read(householdProvider).valueOrNull?.id;
    if (householdId == null) return;
    setState(() => _busy = true);
    try {
      await ref.read(recipeLibraryProvider.notifier).restore(householdId, r.id);
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
    final archived = ref.watch(archivedRecipesProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Archived recipes')),
      body: switch (archived) {
        AsyncData(value: final list) when list.isEmpty => const Center(
          child: Text('Nothing archived.'),
        ),
        AsyncData(:final value) => ListView(
          children: [
            for (final r in value)
              ListTile(
                title: Text(r.title),
                subtitle: _conflictSubtitle(r),
                trailing: TextButton(
                  onPressed: _busy ? null : () => _restore(r),
                  child: const Text('Restore'),
                ),
                onTap: () => context.go('/recipes/${r.id}'),
              ),
          ],
        ),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Text(describeFailure(error, subject: 'Recipes')),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }
}
