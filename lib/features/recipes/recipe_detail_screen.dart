import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/recipes/recipe_fields.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

class RecipeDetailScreen extends ConsumerStatefulWidget {
  const RecipeDetailScreen({super.key, required this.recipeId});

  final String recipeId;

  @override
  ConsumerState<RecipeDetailScreen> createState() => _RecipeDetailScreenState();
}

class _RecipeDetailScreenState extends ConsumerState<RecipeDetailScreen> {
  bool _busy = false;

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
      if (r.servings case final n?) Text('Serves $n'),
      const SizedBox(height: 8),
      Text('Ingredients', style: Theme.of(context).textTheme.titleMedium),
      if (r.lines.isEmpty) const Text('No ingredients listed.'),
      for (final line in r.lines) ListTile(title: Text(describeLine(line))),
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
