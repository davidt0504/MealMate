import 'package:flutter/foundation.dart' show protected;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

/// The household's active recipe library. Every write goes through here, so the list a
/// screen shows is always what Rust says is stored — and the notifier outlives any screen,
/// as `HouseholdNotifier` records.
class RecipeLibraryNotifier extends AsyncNotifier<List<RecipeSummaryDto>> {
  @override
  Future<List<RecipeSummaryDto>> build() async {
    // Only the id, for the reason `RestrictionsNotifier` records.
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    return fetchRecipes(householdId);
  }

  /// The four bridge calls behind overridable seams. `_republish`'s failure path — a re-list
  /// that fails after a write has already committed — is otherwise unreachable in a test,
  /// because the generated commands are free functions on `RustLib`.
  @protected
  Future<List<RecipeSummaryDto>> fetchRecipes(String householdId) =>
      listRecipes(householdId: householdId);

  @protected
  Future<RecipeDto> storeRecipe(RecipeDto recipe) => saveRecipe(recipe: recipe);

  @protected
  Future<RecipeDto> markArchived(
    String householdId,
    String recipeId,
    String archivedOn,
  ) => archiveRecipe(
    householdId: householdId,
    recipeId: recipeId,
    archivedOn: archivedOn,
  );

  @protected
  Future<RecipeDto> markRestored(String householdId, String recipeId) =>
      restoreRecipe(householdId: householdId, recipeId: recipeId);

  /// Saves the whole recipe and returns what was stored (an empty `id` is minted in Rust).
  Future<RecipeDto> save(RecipeDto recipe) async {
    final stored = await storeRecipe(recipe);
    await _republish(stored.householdId);
    return stored;
  }

  /// "Delete": archives, never removes (owner decision 2026-08-28). The date is today's local
  /// civil date — Rust never reads the clock.
  Future<RecipeDto> archive(String householdId, String recipeId) async {
    final stored = await markArchived(householdId, recipeId, todayCivilDate());
    await _republish(householdId);
    ref.invalidate(archivedRecipesProvider);
    return stored;
  }

  Future<RecipeDto> restore(String householdId, String recipeId) async {
    final stored = await markRestored(householdId, recipeId);
    await _republish(householdId);
    ref.invalidate(archivedRecipesProvider);
    return stored;
  }

  /// A re-list that fails *after* the write committed is the list's problem, not the write's:
  /// reporting it out of `save` tells the user the recipe was not stored, and the retry —
  /// because a create sends an empty id, minted fresh in Rust — stores a second copy.
  Future<void> _republish(String householdId) async {
    state = await AsyncValue.guard(() => fetchRecipes(householdId));
  }
}

final recipeLibraryProvider =
    AsyncNotifierProvider<RecipeLibraryNotifier, List<RecipeSummaryDto>>(
      RecipeLibraryNotifier.new,
    );

/// Recipes the household has archived; invalidated by archive/restore above.
class ArchivedRecipesNotifier extends AsyncNotifier<List<RecipeSummaryDto>> {
  @override
  Future<List<RecipeSummaryDto>> build() async {
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    return listArchivedRecipes(householdId: householdId);
  }
}

final archivedRecipesProvider =
    AsyncNotifierProvider<ArchivedRecipesNotifier, List<RecipeSummaryDto>>(
      ArchivedRecipesNotifier.new,
    );

/// One recipe by id, or `null` when it is not this household's. Watches the library so a
/// save or archive re-runs it and the detail shows the stored truth.
final recipeDetailProvider = FutureProvider.family<RecipeDto?, String>((
  ref,
  recipeId,
) async {
  final householdId = await ref.watch(
    householdProvider.selectAsync((h) => h.id),
  );
  ref.watch(recipeLibraryProvider);
  return loadRecipe(householdId: householdId, recipeId: recipeId);
});

/// The unit vocabulary, read from Rust so the dropdown cannot drift from the domain (as
/// `knownRestrictionKindsProvider` does for restrictions).
final knownUnitKindsProvider = FutureProvider<List<String>>(
  (_) async => knownUnitKinds(),
);
