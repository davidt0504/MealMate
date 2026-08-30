import 'package:flutter/foundation.dart' show protected;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/features/planning/planner_provider.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';
import 'package:meal_mate/src/rust/api/pantry.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/shopping.dart';

/// One cycle window and the shopping view Rust derived for it. Nothing here is durable:
/// the list is re-derived on every read and the overlay is what Rust stored (invariants
/// 17, 21).
class ShoppingView {
  const ShoppingView({required this.window, required this.view});

  final PlanningCycleDto window;
  final ShoppingViewDto view;

  String get from => window.dates.first;
  String get to => window.dates.last;

  /// The stored state for one line key, or `null` for "no record".
  ShoppingLineStateDto? stateFor(String key) {
    for (final s in view.lineStates) {
      if (s.key == key) return s;
    }
    return null;
  }
}

/// Keyed by cycle offset from the window containing [plannerAnchorProvider], exactly as
/// [plannerProvider] is, so the two screens page through the same sequence of windows.
class ShoppingNotifier
    extends AutoDisposeFamilyAsyncNotifier<ShoppingView, int> {
  @override
  Future<ShoppingView> build(int arg) async {
    // Three dependencies, resolved values deliberately unused — the watch itself is the
    // point (MVP-016 decision 8): a cycle save moves the window; a meal save or a pantry
    // toggle changes what the derivation returns, so either re-runs this build. Do not
    // "simplify" them away.
    final cycle = ref.watch(planningCycleProvider.future);
    final planner = ref.watch(plannerProvider(arg).future);
    final pantry = ref.watch(pantryProvider.future);
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    await cycle;
    await planner;
    await pantry;
    // `read`, for the planner's reason: re-anchoring is the planner's job, and this
    // notifier never commits an anchor.
    return _fetch(householdId, arg, ref.read(plannerAnchorProvider));
  }

  Future<ShoppingView> _fetch(
    String householdId,
    int offset,
    String today,
  ) async {
    final window = await fetchWindow(householdId, today, offset);
    final view = await fetchView(
      householdId,
      window.dates.first,
      window.dates.last,
    );
    return ShoppingView(window: window, view: view);
  }

  /// The bridge calls behind overridable seams, as `PlannerNotifier`'s are: a test double
  /// replaces these and nothing else, so the republish logic below runs for real under
  /// every widget assertion.
  @protected
  Future<PlanningCycleDto> fetchWindow(
    String householdId,
    String today,
    int offset,
  ) => planningCycleWindow(
    householdId: householdId,
    today: today,
    offsetCycles: offset,
  );

  @protected
  Future<ShoppingViewDto> fetchView(
    String householdId,
    String from,
    String to,
  ) => loadShoppingView(householdId: householdId, fromDate: from, toDate: to);

  @protected
  Future<ShoppingLineStateDto> writeLineState(
    String householdId,
    String from,
    String to,
    ShoppingLineStateDto state,
  ) => setShoppingLineState(
    householdId: householdId,
    fromDate: from,
    toDate: to,
    state: state,
  );

  @protected
  Future<ShoppingManualItemDto> writeManualItem(ShoppingManualItemDto item) =>
      saveShoppingManualItem(item: item);

  @protected
  Future<void> removeManualItem(String householdId, String itemId) =>
      deleteShoppingManualItem(householdId: householdId, itemId: itemId);

  @protected
  Future<void> writeReset(String householdId, String from, String to) =>
      resetShoppingList(householdId: householdId, fromDate: from, toDate: to);

  @protected
  Future<List<IngredientRefDto>> writePantryMarks(
    String householdId,
    List<IngredientRefDto> ingredients,
    bool marked,
  ) => setPantryMarks(
    householdId: householdId,
    ingredients: ingredients,
    marked: marked,
  );

  /// Re-reads the window and its view; returns the failure, or `null`. Same contract as
  /// `PantryNotifier.refresh`. Unlike the planner's, this reads the shared anchor and never
  /// commits one — re-anchoring stays the planner's job.
  Future<Object?> refresh() async {
    final today = ref.read(plannerAnchorProvider);
    final result = await AsyncValue.guard(() async {
      final household = await ref.read(householdProvider.future);
      return _fetch(household.id, arg, today);
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

  /// Writes one line's flags and republishes the view with what Rust stored, keyed by
  /// line key. A state with every flag off is a clear: Rust deleted the row, so it leaves
  /// the list rather than being kept as an all-false record.
  Future<ShoppingLineStateDto> setLineState(ShoppingLineStateDto state) async {
    final current = await future;
    final household = await ref.read(householdProvider.future);
    final stored = await writeLineState(
      household.id,
      current.from,
      current.to,
      state,
    );
    final cleared = !stored.checked && !stored.hidden && !stored.restored;
    _republish(
      (view) => ShoppingViewDto(
        list: view.list,
        lineStates: [
          for (final s in view.lineStates)
            if (s.key != stored.key) s,
          if (!cleared) stored,
        ],
        manualItems: view.manualItems,
        orphanedLineStateCount: view.orphanedLineStateCount,
      ),
    );
    return stored;
  }

  /// Saves the item and republishes with what Rust stored — replacing by id, else
  /// inserting — kept in the order the list read uses (name, case-folded, then id).
  Future<ShoppingManualItemDto> saveManualItem(
    ShoppingManualItemDto item,
  ) async {
    final stored = await writeManualItem(item);
    _republish((view) {
      final next = [
        for (final i in view.manualItems)
          if (i.id != stored.id) i,
        stored,
      ];
      next.sort(_byNameThenId);
      return ShoppingViewDto(
        list: view.list,
        lineStates: view.lineStates,
        manualItems: next,
        orphanedLineStateCount: view.orphanedLineStateCount,
      );
    });
    return stored;
  }

  Future<void> deleteManualItem(String itemId) async {
    final household = await ref.read(householdProvider.future);
    await removeManualItem(household.id, itemId);
    _republish(
      (view) => ShoppingViewDto(
        list: view.list,
        lineStates: view.lineStates,
        manualItems: [
          for (final i in view.manualItems)
            if (i.id != itemId) i,
        ],
        orphanedLineStateCount: view.orphanedLineStateCount,
      ),
    );
  }

  /// Marks the given identities in one bridge call and returns exactly the refs whose mark
  /// changed — the set [undoPantryMarks] must be sent, so a pre-existing mark is never
  /// cleared by an Undo. Invalidating the pantry is what re-runs this build (decision 8),
  /// so the omitted lines move without a second fetch being written here.
  Future<List<IngredientRefDto>> addCheckedToPantry(
    List<IngredientRefDto> refs,
  ) async {
    final household = await ref.read(householdProvider.future);
    final changed = await writePantryMarks(household.id, refs, true);
    ref.invalidate(pantryProvider);
    return changed;
  }

  Future<List<IngredientRefDto>> undoPantryMarks(
    List<IngredientRefDto> refs,
  ) async {
    final household = await ref.read(householdProvider.future);
    final changed = await writePantryMarks(household.id, refs, false);
    ref.invalidate(pantryProvider);
    return changed;
  }

  /// "Start over" for the window on screen, then a fresh read: the derivation is untouched,
  /// so what comes back is the same list with its overlay cleared.
  ///
  /// The read is published whatever it returns, unlike [refresh], which keeps a usable view on
  /// failure. Here the write has already committed, so the view being kept would be a
  /// successful-looking list that storage no longer holds; a failure is the honest state. The
  /// error is rethrown as well, so the caller's own reporting still runs.
  Future<void> reset() async {
    final current = await future;
    final household = await ref.read(householdProvider.future);
    await writeReset(household.id, current.from, current.to);
    final today = ref.read(plannerAnchorProvider);
    final result = await AsyncValue.guard(
      () => _fetch(household.id, arg, today),
    );
    state = result;
    if (result case AsyncError(:final error, :final stackTrace)) {
      Error.throwWithStackTrace(error, stackTrace);
    }
  }

  /// Only over a usable view: a write that lands while the view is loading or failed has
  /// nothing to merge into, and the next build reads it back anyway.
  void _republish(ShoppingViewDto Function(ShoppingViewDto view) change) {
    final current = state.valueOrNull;
    if (current == null) return;
    state = AsyncData(
      ShoppingView(window: current.window, view: change(current.view)),
    );
  }

  static int _byNameThenId(ShoppingManualItemDto a, ShoppingManualItemDto b) {
    final byName = a.name.toLowerCase().compareTo(b.name.toLowerCase());
    return byName != 0 ? byName : a.id.compareTo(b.id);
  }
}

final shoppingProvider = AsyncNotifierProvider.autoDispose
    .family<ShoppingNotifier, ShoppingView, int>(ShoppingNotifier.new);
