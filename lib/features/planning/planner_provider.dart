import 'package:flutter/foundation.dart' show protected;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planning.dart';

/// One cycle window and the occurrences dated inside it. Both come from Rust whole; the
/// screen holds nothing durable (invariants 17, 21).
class PlannerView {
  const PlannerView({required this.window, required this.meals});

  final PlanningCycleDto window;
  final List<PlannedMealDto> meals;

  /// The occurrence at one cell, or `null`. Storage refuses two at the same `(date, slot)`,
  /// so the first match is the only one.
  PlannedMealDto? at(String date, MealSlotDto slot) {
    for (final meal in meals) {
      if (meal.date == date && meal.slot == slot) return meal;
    }
    return null;
  }
}

/// The wall clock, behind a provider so a test can inject a date and so re-reading it is one
/// named operation. Only [PlannerAnchor] reads it: a date this returns is a *candidate*
/// anchor, not the anchor.
final plannerTodayProvider = Provider<String>((_) => todayCivilDate());

/// The one `today` every offset is measured from, resolved once per container. Without a
/// shared anchor each family member reads its own clock, so a session open across a cycle
/// boundary builds offset 1 from a `today` already inside the next window: the user pages
/// from W to W+2 and W+1 cannot be reached at all. Deliberately not `autoDispose` — it has to
/// outlive the offsets that share it.
///
/// [PlannerNotifier.refresh] is the only thing that moves it, and only once a read anchored
/// to the new date has come back: a refresh that fails deliberately leaves the old window on
/// screen, and an anchor that moved anyway would put the arrows one cycle out of step with
/// what is rendered — the same unreachable-W+1 skip, reached through the failure path. So
/// offset 0 is the window containing the `today` of the last *successful* read, which after a
/// boundary is last cycle until a refresh succeeds. That was already true of every cached
/// offset before the anchor was shared; what is new is that all of them are now wrong
/// together rather than each in its own direction.
class PlannerAnchor extends Notifier<String> {
  @override
  String build() => ref.read(plannerTodayProvider);

  /// Moves the anchor to [today]; called only from the success branch of a refresh.
  void commit(String today) => state = today;
}

final plannerAnchorProvider = NotifierProvider<PlannerAnchor, String>(
  PlannerAnchor.new,
);

/// Keyed by cycle offset from the window containing [plannerAnchorProvider]: 0 is that window,
/// ±1 its neighbours. `autoDispose`, because the screen watches exactly one offset at a time
/// and a plain family would grow a member per arrow press and free none of them. Paging back
/// therefore re-reads — every read anchored to the same `today`, so the offsets stay one
/// consistent sequence rather than each its own clock's.
class PlannerNotifier extends AutoDisposeFamilyAsyncNotifier<PlannerView, int> {
  @override
  Future<PlannerView> build(int arg) async {
    // Watching the stored cycle first, as the library watches restrictions: a cycle save
    // re-runs this build so the window follows the new rhythm. The resolved value is
    // deliberately unused — the watch itself is the dependency; do not "simplify" it away.
    final cycle = ref.watch(planningCycleProvider.future);
    // Only the id, for the reason `RestrictionsNotifier` records.
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    await cycle;
    // `read`, not `watch`: committing a new anchor must not rebuild every live offset into a
    // spinner behind a plan the user is reading. `refresh` is what re-anchors, and it
    // republishes on its own terms.
    return _fetch(householdId, arg, ref.read(plannerAnchorProvider));
  }

  /// [today] is passed rather than read here so that `refresh` can fetch against a candidate
  /// anchor it has not committed yet.
  Future<PlannerView> _fetch(
    String householdId,
    int offset,
    String today,
  ) async {
    final window = await fetchWindow(householdId, today, offset);
    final meals = await fetchMeals(
      householdId,
      window.dates.first,
      window.dates.last,
    );
    return PlannerView(window: window, meals: meals);
  }

  /// The five bridge calls behind overridable seams, as `PantryNotifier`'s are: a test
  /// double replaces these and nothing else, so the republish logic below runs for real
  /// under every widget assertion.
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
  Future<List<PlannedMealDto>> fetchMeals(
    String householdId,
    String from,
    String to,
  ) => listPlannedMeals(householdId: householdId, fromDate: from, toDate: to);

  @protected
  Future<PlannedMealDto> storeMeal(PlannedMealDto meal) =>
      savePlannedMeal(meal: meal);

  @protected
  Future<PlannedMealDto> writeLock(
    String householdId,
    String mealId,
    bool locked,
  ) => setPlannedMealLock(
    householdId: householdId,
    mealId: mealId,
    locked: locked,
  );

  @protected
  Future<void> removeMeal(String householdId, String mealId) =>
      deletePlannedMeal(householdId: householdId, mealId: mealId);

  /// Re-reads the window and its meals; returns the failure, or `null`. Same contract as
  /// `PantryNotifier.refresh`: a failure is only published when there is no usable view to
  /// lose, so a pull-to-refresh that fails leaves the plan on screen and reports in a
  /// snackbar, while a retry from the error arm publishes.
  Future<Object?> refresh() async {
    // Read the clock again — a session open across a cycle boundary is exactly where the
    // shared anchor has gone stale, and this is the only affordance that corrects it — but
    // fetch against the new date before committing it, so the failure arm below leaves the
    // anchor on the window it is still showing.
    final today = ref.refresh(plannerTodayProvider);
    final result = await AsyncValue.guard(() async {
      final household = await ref.read(householdProvider.future);
      return _fetch(household.id, arg, today);
    });
    switch (result) {
      case AsyncData(:final value):
        ref.read(plannerAnchorProvider.notifier).commit(today);
        state = AsyncData(value);
        return null;
      case AsyncError(:final error):
        if (state.valueOrNull == null) state = result;
        return error;
      case _:
        return null;
    }
  }

  /// Saves the whole occurrence and republishes the view with what Rust says was stored —
  /// replacing by id (a move changes `date`/`slot` under the same id), else appending —
  /// kept in `(date, slot)` order so a new cell lands where the list read would put it.
  Future<PlannedMealDto> save(PlannedMealDto meal) async {
    final stored = await storeMeal(meal);
    _republish((meals) {
      final next = [
        for (final m in meals)
          if (m.id != stored.id) m,
        stored,
      ];
      next.sort(_byDateThenSlot);
      return next;
    });
    return stored;
  }

  /// Republishes the stored lock state; the row's switch is rendered from provider state,
  /// so a failed write leaves it at the stored value.
  Future<PlannedMealDto> setLock(
    String householdId,
    String mealId,
    bool locked,
  ) async {
    final stored = await writeLock(householdId, mealId, locked);
    _republish(
      (meals) => [
        for (final m in meals)
          if (m.id == stored.id) stored else m,
      ],
    );
    return stored;
  }

  Future<void> delete(String householdId, String mealId) async {
    await removeMeal(householdId, mealId);
    _republish(
      (meals) => [
        for (final m in meals)
          if (m.id != mealId) m,
      ],
    );
  }

  /// Only over a usable view: a write that lands while the view is loading or failed has
  /// nothing to merge into, and the next build reads it back anyway.
  void _republish(
    List<PlannedMealDto> Function(List<PlannedMealDto> meals) change,
  ) {
    final current = state.valueOrNull;
    if (current == null) return;
    state = AsyncData(
      PlannerView(window: current.window, meals: change(current.meals)),
    );
  }

  static int _byDateThenSlot(PlannedMealDto a, PlannedMealDto b) {
    final byDate = a.date.compareTo(b.date);
    return byDate != 0 ? byDate : a.slot.index.compareTo(b.slot.index);
  }
}

final plannerProvider = AsyncNotifierProvider.autoDispose
    .family<PlannerNotifier, PlannerView, int>(PlannerNotifier.new);

/// The component kind vocabulary, read from Rust so the picker cannot drift from the domain
/// (as `knownUnitKindsProvider` does for units).
final mealComponentKindsProvider = FutureProvider<List<String>>(
  (_) async => knownMealComponentKinds(),
);
