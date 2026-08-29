import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

/// The library: active recipes only (PRD §16). Archived ones live one tap away.
class RecipeListScreen extends ConsumerWidget {
  const RecipeListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
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
        AsyncData(:final value) => ListView(
          children: [
            for (final r in value)
              ListTile(
                title: Text(r.title),
                trailing: const Icon(Icons.chevron_right),
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
