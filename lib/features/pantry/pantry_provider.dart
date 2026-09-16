import 'package:flutter/foundation.dart' show protected;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/src/rust/api/pantry.dart';
// Prefixed too: the notifier's own `setRestockFlag`/`setPantryUsedUp` methods share a name
// with these bridge functions, and an unqualified call inside the class would resolve to the
// method (member lookup wins over a top-level function of the same name), not the bridge.
import 'package:meal_mate/src/rust/api/pantry.dart' as pantry_bridge;
import 'package:meal_mate/src/rust/api/recipe.dart';

class PantryNotifier extends AsyncNotifier<List<PantryEntryDto>> {
  @override
  Future<List<PantryEntryDto>> build() async {
    // Only the id, for the reason `RestrictionsNotifier` records: watching the whole household
    // would re-run this build on every rename.
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    return fetchPantry(householdId);
  }

  /// The two bridge calls behind overridable seams, as `RecipeLibraryNotifier`'s are: a test
  /// double replaces these and nothing else, so `setMark`'s merge — this notifier's only
  /// non-trivial behaviour — runs for real under every widget assertion.
  @protected
  Future<List<PantryEntryDto>> fetchPantry(String householdId) =>
      listPantry(householdId: householdId);

  @protected
  Future<PantryEntryDto> writeMark(
    String householdId,
    IngredientRefDto ingredient,
    bool marked,
  ) => setPantryMark(
    householdId: householdId,
    ingredient: ingredient,
    marked: marked,
  );

  @protected
  Future<PantryEntryDto> writeRestockFlag(
    String householdId,
    IngredientRefDto ingredient,
    bool flagged,
  ) => pantry_bridge.setRestockFlag(
    householdId: householdId,
    ingredient: ingredient,
    flagged: flagged,
  );

  @protected
  Future<PantryEntryDto> writeUsedUp(
    String householdId,
    IngredientRefDto ingredient,
  ) => pantry_bridge.setPantryUsedUp(
    householdId: householdId,
    ingredient: ingredient,
  );

  /// Re-reads the list from Rust and returns the failure, or `null`. `AsyncValue.guard`
  /// rather than `ref.invalidateSelf()`: the screen renders a spinner for any non-`AsyncData`
  /// state, so invalidating would blank a list the user is reading back to a spinner, and the
  /// returned future would not track the reload for a `RefreshIndicator`.
  ///
  /// A failure is only *published* when there is no usable list to lose. A refresh that fails
  /// over a list the user is reading returns the error for the caller to put in a snackbar,
  /// rather than replacing that list with an error screen; a retry from the error arm has
  /// nothing to protect, so it publishes.
  Future<Object?> refresh() async {
    final result = await AsyncValue.guard(() async {
      final household = await ref.read(householdProvider.future);
      return fetchPantry(household.id);
    });
    switch (result) {
      case AsyncData(:final value):
        state = AsyncData(value);
        return null;
      case AsyncError(:final error):
        if (state.valueOrNull == null) state = result;
        return error;
      case _:
        return null;
    }
  }

  /// Writes one mark and republishes the list with the entry the bridge says was stored, so
  /// the UI shows persisted truth. The state write goes through the notifier, which outlives
  /// any screen. Only the toggled entry is replaced — the pantry is edited one tap at a time,
  /// so refetching the whole list per tap would cost a bridge read for nothing.
  Future<PantryEntryDto> setMark(
    String householdId,
    IngredientRefDto ingredient,
    bool marked,
  ) => _applyWrite(householdId, writeMark(householdId, ingredient, marked));

  /// Flags or unflags restock for one identity — same merge-into-state contract as [setMark],
  /// independent of the have/none mark (OPT-006 gate 1).
  Future<PantryEntryDto> setRestockFlag(
    String householdId,
    IngredientRefDto ingredient,
    bool flagged,
  ) => _applyWrite(
    householdId,
    writeRestockFlag(householdId, ingredient, flagged),
  );

  /// "Used it up" (OPT-006 gate 4): clears the have-mark and sets the restock flag together.
  Future<PantryEntryDto> setUsedUp(
    String householdId,
    IngredientRefDto ingredient,
  ) => _applyWrite(householdId, writeUsedUp(householdId, ingredient));

  /// Shared by [setMark], [setRestockFlag] and [setUsedUp]: awaits the bridge write, then
  /// republishes the list with the entry the bridge says was stored, so the UI shows persisted
  /// truth. Only the touched entry is replaced — the pantry is edited one tap at a time, so
  /// refetching the whole list per tap would cost a bridge read for nothing.
  Future<PantryEntryDto> _applyWrite(
    String householdId,
    Future<PantryEntryDto> write,
  ) async {
    final stored = await write;
    final current = state.valueOrNull;
    if (current == null) return stored;
    if (current.any((e) => e.ingredient == stored.ingredient)) {
      state = AsyncData([
        for (final entry in current)
          entry.ingredient == stored.ingredient ? stored : entry,
      ]);
    } else {
      // The bridge confirmed an identity this list has never seen: the list was built before
      // the starter install that added it. Re-read rather than append — `list_pantry_entries`
      // orders by name, so an appended row would land last.
      //
      // A re-list that fails is discarded, not published: the write committed either way, and
      // publishing `AsyncError` over a usable list would strip the recipe detail's pantry
      // control entirely (`_lineTile` falls back to a plain tile whenever the list has no
      // value) — losing a control as the result of a *successful* write. Staying stale is what
      // the caller already had.
      final relisted = await AsyncValue.guard(() => fetchPantry(householdId));
      if (relisted case AsyncData(:final value)) state = AsyncData(value);
    }
    return stored;
  }

  /// Custom ingredients this household has not yet categorized (OPT-006 gate 5 remediation).
  @protected
  Future<List<CustomIngredientMissingCategoryDto>> fetchMissingCategory(
    String householdId,
  ) => listCustomIngredientsMissingCategory(householdId: householdId);

  /// Assigns a category to a pre-existing custom ingredient, then refreshes the pantry list so
  /// it renders grouped. Unlike [setMark]/[setRestockFlag]/[setUsedUp], a full refetch is
  /// correct here: categorizing changes which group the entry belongs to, and (per
  /// `list_pantry_entries`) it may not have appeared in the list at all beforehand.
  Future<void> categorize(
    String householdId,
    String customIngredientId,
    String storeCategory,
  ) async {
    await setCustomIngredientCategory(
      householdId: householdId,
      id: customIngredientId,
      storeCategory: storeCategory,
    );
    final error = await refresh();
    if (error != null) throw error;
  }
}

/// Everything this household can mark, with its current mark. Depends on `householdProvider`,
/// so holding this holds the open database.
final pantryProvider =
    AsyncNotifierProvider<PantryNotifier, List<PantryEntryDto>>(
      PantryNotifier.new,
    );

/// The store-category vocabulary (OPT-006 gate 5), read from Rust so the dropdown cannot drift
/// from the domain (as `knownUnitKindsProvider` does for units).
final knownStoreCategoriesProvider = FutureProvider<List<String>>(
  (_) async => knownStoreCategories(),
);
