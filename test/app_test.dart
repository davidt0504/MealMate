import 'dart:async';

import 'package:flutter_rust_bridge/flutter_rust_bridge.dart' show Int64List;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/app/app.dart';
import 'package:meal_mate/app/router.dart';
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/settings/backup_copy.dart';
import 'package:meal_mate/features/settings/backup_provider.dart';
import 'package:meal_mate/features/settings/health_provider.dart';
import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/api/household.dart';
import 'package:meal_mate/features/planning/planner_copy.dart';
import 'package:meal_mate/features/planning/planner_provider.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/recipes/restriction_warnings.dart';
import 'package:meal_mate/features/pantry/pantry_copy.dart';
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/features/restrictions/restrictions_provider.dart';
import 'package:meal_mate/features/restrictions/restriction_copy.dart';
import 'package:meal_mate/src/rust/api/pantry.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/restrictions.dart';
import 'package:meal_mate/src/rust/api/starter.dart';
import 'package:meal_mate/features/recipes/starter_provider.dart';
import 'package:meal_mate/features/planning/cover_copy.dart';
import 'package:meal_mate/features/planning/cover_provider.dart';
import 'package:meal_mate/src/rust/api/decisions.dart';
import 'package:meal_mate/src/rust/api/planner.dart';
import 'package:meal_mate/features/shopping/shopping_copy.dart';
import 'package:meal_mate/features/shopping/shopping_provider.dart';
import 'package:meal_mate/src/rust/api/shopping.dart';

const okReport = HealthReport(dbPath: '/x/kimatta.db', schemaVersion: 5);

/// The report an install returns today: the catalog seeds, and nothing installs because
/// nothing has a recorded cook review (MVP-011 AC-3).
const okStarterReport = StarterInstallReportDto(
  installed: 0,
  skipped: 0,
  catalogInstalled: 37,
  available: 0,
  pendingCookReview: 10,
);

/// A steady-state install: it short-circuited and wrote nothing, so nothing downstream needs
/// re-reading. `okStarterReport` models a *first* install (`catalogInstalled: 37`), which is
/// why every harness launch re-reads the pantry unless a test opts out with this.
const noCatalogStarterReport = StarterInstallReportDto(
  installed: 0,
  skipped: 0,
  catalogInstalled: 0,
  available: 0,
  pendingCookReview: 10,
);

/// Two lines, one with every structured field and one with only its text, so the detail and
/// edit tests exercise both shapes.
const okRecipe = RecipeDto(
  id: 'r-1',
  householdId: 'h-1',
  title: 'Pancakes',
  servings: 4,
  prepMinutes: 20,
  instructions: 'Mix. Fry.',
  lines: [
    IngredientLineDto(
      originalText: '  1/2 cup Flour, sifted ',
      name: 'Flour',
      quantity: QuantityDto.exact(numer: 1, denom: 2),
      unit: UnitDto.known(unit: 'cup'),
      preparation: 'sifted',
      optional: false,
    ),
    IngredientLineDto(
      originalText: 'a splash of something',
      name: 'something',
      quantity: QuantityDto.unknown(),
      unit: UnitDto.none(),
      optional: false,
    ),
  ],
  provenance: RecipeProvenanceDto(kind: 'authored'),
  assessment: emptyAssessment,
);

/// What a read-back reports with no restrictions stored: checked nothing, found nothing.
const emptyAssessment = RestrictionAssessmentDto(
  ruleVersion: 1,
  restrictionsChecked: 0,
  linesChecked: 2,
  conflicts: [],
  wordingOnly: [],
);
const dairyConflict = ConflictDto(
  restriction: RestrictionDto.known(kind: 'dairy'),
  linePosition: 0,
  lineName: 'butter',
  term: 'butter',
);
const dairyAssessment = RestrictionAssessmentDto(
  ruleVersion: 1,
  restrictionsChecked: 1,
  linesChecked: 1,
  conflicts: [dairyConflict],
  wordingOnly: [],
);
const checkedCleanAssessment = RestrictionAssessmentDto(
  ruleVersion: 1,
  restrictionsChecked: 1,
  linesChecked: 1,
  conflicts: [],
  wordingOnly: [],
);
const okSummary = RecipeSummaryDto(
  id: 'r-1',
  title: 'Pancakes',
  assessment: emptyAssessment,
);
const conflictSummary = RecipeSummaryDto(
  id: 'r-2',
  title: 'Butter toast',
  assessment: dairyAssessment,
);
const checkedCleanSummary = RecipeSummaryDto(
  id: 'r-3',
  title: 'Rice',
  assessment: checkedCleanAssessment,
);

/// One line that resolves to a catalog identity and one that resolves to nothing, so the
/// detail's pantry affordance has both arms to exercise. `okRecipe`'s lines are both
/// unresolved, which is why this cannot reuse it.
const recipeWithResolvedLine = RecipeDto(
  id: 'r-1',
  householdId: 'h-1',
  title: 'Hummus',
  servings: 4,
  instructions: 'Mix. Fry.',
  lines: [
    IngredientLineDto(
      originalText: 'chickpeas',
      name: 'chickpeas',
      ingredient: chickpeasRef,
      quantity: QuantityDto.unknown(),
      unit: UnitDto.none(),
      optional: false,
    ),
    IngredientLineDto(
      originalText: 'a splash of something',
      name: 'something',
      quantity: QuantityDto.unknown(),
      unit: UnitDto.none(),
      optional: false,
    ),
  ],
  provenance: RecipeProvenanceDto(kind: 'authored'),
  assessment: emptyAssessment,
);

/// `okRecipe` with no prep estimate, for the absence case.
final okRecipeNoPrep = RecipeDto(
  id: okRecipe.id,
  householdId: okRecipe.householdId,
  title: okRecipe.title,
  servings: okRecipe.servings,
  instructions: okRecipe.instructions,
  lines: okRecipe.lines,
  provenance: okRecipe.provenance,
  assessment: okRecipe.assessment,
);

/// `okRecipe` with two conflicts — one known kind, one wording-only `Other` — and the
/// wording-only note, for the detail tests.
const okRecipeWithConflicts = RecipeDto(
  id: 'r-1',
  householdId: 'h-1',
  title: 'Pancakes',
  servings: 4,
  instructions: 'Mix. Fry.',
  lines: [
    IngredientLineDto(
      originalText: '  1/2 cup Flour, sifted ',
      name: 'Flour',
      quantity: QuantityDto.exact(numer: 1, denom: 2),
      unit: UnitDto.known(unit: 'cup'),
      preparation: 'sifted',
      optional: false,
    ),
    IngredientLineDto(
      originalText: 'a splash of something (optional)',
      name: 'something',
      quantity: QuantityDto.unknown(),
      unit: UnitDto.none(),
      optional: true,
    ),
  ],
  provenance: RecipeProvenanceDto(kind: 'authored'),
  assessment: RestrictionAssessmentDto(
    ruleVersion: 1,
    restrictionsChecked: 2,
    linesChecked: 2,
    conflicts: [
      ConflictDto(
        restriction: RestrictionDto.known(kind: 'gluten'),
        linePosition: 0,
        lineName: 'Flour',
        term: 'flour',
      ),
      ConflictDto(
        restriction: RestrictionDto.other(text: 'something'),
        linePosition: 1,
        lineName: 'something',
        term: 'something',
      ),
    ],
    wordingOnly: ['something'],
  ),
);
const okCycle = PlanningCycleDto(
  householdId: 'h-1',
  anchorDate: '2026-08-29',
  lengthDays: 7,
  mealSlots: [MealSlotDto.dinner],
  dates: [
    '2026-08-29',
    '2026-08-30',
    '2026-08-31',
    '2026-09-01',
    '2026-09-02',
    '2026-09-03',
    '2026-09-04',
  ],
);

/// `onboarded: true` throughout, so every pre-existing test keeps exercising the
/// already-welcomed path and the first-run gate stays the concern of the tests that
/// deliberately opt out of it.
const okHousehold = HouseholdDto(
  id: 'h-1',
  name: null,
  members: [MemberDto(id: 'm-1', displayName: 'Me')],
  onboarded: true,
);
const twoMemberHousehold = HouseholdDto(
  id: 'h-2',
  name: 'Casa',
  members: [
    MemberDto(id: 'm-1', displayName: 'Me'),
    MemberDto(id: 'm-2', displayName: 'Ada'),
  ],
  onboarded: true,
);

/// All four providers are always overridden, which is why `flutter test` never
/// loads the native library: no test reaches `openDatabase`,
/// `bootstrapHousehold`, `ensurePlanningCycle` or `loadRestrictions`. The save paths go
/// through the notifiers, so `rename:`, `completeOnboarding:`, `saveCycle:` and
/// `saveRestrictions:` fake them at the same seam; a test that taps a save button
/// without supplying one fails loudly rather than reaching the real bridge.
/// `bridge_native_test.dart` is what proves the writes themselves.
class _FakeHouseholdNotifier extends HouseholdNotifier {
  _FakeHouseholdNotifier(this._build, this._rename, this._complete);

  final FutureOr<HouseholdDto> Function()? _build;
  final Future<HouseholdDto> Function(String, String?)? _rename;
  final Future<HouseholdDto> Function(String)? _complete;

  @override
  Future<HouseholdDto> completeOnboarding(String householdId) async {
    final fake = _complete;
    if (fake == null) {
      throw StateError(
        'this test completes onboarding without a harness `completeOnboarding:` hook',
      );
    }
    final updated = await fake(householdId);
    state = AsyncData(updated);
    return updated;
  }

  @override
  Future<HouseholdDto> build() async {
    // Keep the real provider's dependency edge: App now holds only
    // householdProvider, so a stub that skipped this await would leave
    // healthReportProvider uncreated and test 13 would read 0.
    await ref.watch(healthReportProvider.future);
    return (_build ?? () => okHousehold)();
  }

  @override
  Future<HouseholdDto> rename(String householdId, String? name) async {
    final fake = _rename;
    if (fake == null) {
      throw StateError(
        'this test taps Save name without a harness `rename:` hook',
      );
    }
    final updated = await fake(householdId, name);
    state = AsyncData(updated);
    return updated;
  }
}

/// Keeps the real provider's dependency edge on `householdProvider`, so the
/// "bootstrapped once" counting tests still read 1 with the planning tile watching.
class _FakePlanningCycleNotifier extends PlanningCycleNotifier {
  _FakePlanningCycleNotifier(this._build, this._save);

  final FutureOr<PlanningCycleDto> Function()? _build;
  final Future<PlanningCycleDto> Function(
    String,
    String,
    int,
    List<MealSlotDto>,
  )?
  _save;

  @override
  Future<PlanningCycleDto> build() async {
    await ref.watch(householdProvider.selectAsync((h) => h.id));
    return (_build ?? () => okCycle)();
  }

  @override
  Future<PlanningCycleDto> save(
    String householdId, {
    required String anchorDate,
    required int lengthDays,
    required List<MealSlotDto> mealSlots,
  }) async {
    final fake = _save;
    if (fake == null) {
      throw StateError('this test taps Save cycle without a `saveCycle:` hook');
    }
    final stored = await fake(householdId, anchorDate, lengthDays, mealSlots);
    state = AsyncData(stored);
    return stored;
  }
}

class _FakeRestrictionsNotifier extends RestrictionsNotifier {
  _FakeRestrictionsNotifier(this._build, this._save);

  final FutureOr<List<RestrictionDto>> Function()? _build;
  final Future<List<RestrictionDto>> Function(String, List<RestrictionDto>)?
  _save;

  @override
  Future<List<RestrictionDto>> build() async {
    await ref.watch(householdProvider.selectAsync((h) => h.id));
    return (_build ?? () => const <RestrictionDto>[])();
  }

  @override
  Future<List<RestrictionDto>> save(
    String householdId,
    List<RestrictionDto> restrictions,
  ) async {
    final fake = _save;
    if (fake == null) {
      throw StateError(
        'this test taps Save restrictions without a `saveRestrictions:` hook',
      );
    }
    final stored = await fake(householdId, restrictions);
    state = AsyncData(stored);
    return stored;
  }
}

/// Same seam as the restriction fakes: `save`, `archive` and `restore` are faked whole, and
/// the two lifecycle calls re-run `build` (and drop the archived list) so a hook returning a
/// changed list is what the screen shows next — the real notifier re-lists the same way.
class _FakeRecipeLibraryNotifier extends RecipeLibraryNotifier {
  _FakeRecipeLibraryNotifier(
    this._build,
    this._save,
    this._archive,
    this._restore,
  );

  final FutureOr<List<RecipeSummaryDto>> Function()? _build;
  final Future<RecipeDto> Function(RecipeDto)? _save;
  final Future<RecipeDto> Function(String, String)? _archive;
  final Future<RecipeDto> Function(String, String)? _restore;

  @override
  Future<List<RecipeSummaryDto>> build() async {
    await ref.watch(householdProvider.selectAsync((h) => h.id));
    return (_build ?? () => const <RecipeSummaryDto>[])();
  }

  @override
  Future<RecipeDto> save(RecipeDto recipe) async {
    final fake = _save;
    if (fake == null) {
      throw StateError(
        'this test taps Save recipe without a `saveRecipe:` hook',
      );
    }
    return fake(recipe);
  }

  @override
  Future<RecipeDto> archive(String householdId, String recipeId) async {
    final fake = _archive;
    if (fake == null) {
      throw StateError('this test archives without an `archiveRecipe:` hook');
    }
    final stored = await fake(householdId, recipeId);
    ref.invalidateSelf();
    ref.invalidate(archivedRecipesProvider);
    return stored;
  }

  @override
  Future<RecipeDto> restore(String householdId, String recipeId) async {
    final fake = _restore;
    if (fake == null) {
      throw StateError('this test restores without a `restoreRecipe:` hook');
    }
    final stored = await fake(householdId, recipeId);
    ref.invalidateSelf();
    ref.invalidate(archivedRecipesProvider);
    return stored;
  }
}

/// The *real* `RecipeLibraryNotifier` with only its bridge seams replaced, so `save`'s,
/// `archive`'s and `restore`'s own re-list runs. `_FakeRecipeLibraryNotifier` above overrides
/// all three wholesale and therefore cannot reach any of them.
class _SeamedRecipeLibraryNotifier extends RecipeLibraryNotifier {
  _SeamedRecipeLibraryNotifier(this._onFetch);

  final Future<List<RecipeSummaryDto>> Function(int call) _onFetch;
  int fetches = 0;
  int archives = 0;
  int restores = 0;
  String? archivedOn;

  @override
  Future<List<RecipeSummaryDto>> fetchRecipes(String householdId) =>
      _onFetch(++fetches);

  @override
  Future<RecipeDto> storeRecipe(RecipeDto recipe) async => okRecipe;

  @override
  Future<RecipeDto> markArchived(
    String householdId,
    String recipeId,
    String on,
  ) async {
    archives++;
    archivedOn = on;
    return okRecipe;
  }

  @override
  Future<RecipeDto> markRestored(String householdId, String recipeId) async {
    restores++;
    return okRecipe;
  }
}

class _FakeArchivedRecipesNotifier extends ArchivedRecipesNotifier {
  _FakeArchivedRecipesNotifier(this._build);

  final FutureOr<List<RecipeSummaryDto>> Function()? _build;

  @override
  Future<List<RecipeSummaryDto>> build() async {
    await ref.watch(householdProvider.selectAsync((h) => h.id));
    return (_build ?? () => const <RecipeSummaryDto>[])();
  }
}

/// The *real* `ArchivedRecipesNotifier` with only its bridge seam replaced, as
/// `_SeamedRecipeLibraryNotifier` is for the library, so its own `build` — and therefore its
/// restriction watch — runs. The fake above overrides `build` wholesale and cannot reach it.
class _SeamedArchivedRecipesNotifier extends ArchivedRecipesNotifier {
  int fetches = 0;

  @override
  Future<List<RecipeSummaryDto>> fetchArchived(String householdId) async {
    fetches++;
    return const [okSummary];
  }
}

/// The *real* `PantryNotifier` with only its two bridge seams replaced, in the
/// `_SeamedArchivedRecipesNotifier` pattern: its own `build`, `refresh` and `setMark` all run,
/// so a regression in any of them is visible here. Overriding `setMark` wholesale — as this
/// double used to — meant every row assertion verified the test file's own copy of the merge.
class _FakePantryNotifier extends PantryNotifier {
  _FakePantryNotifier(this._build, this._setMark);

  final FutureOr<List<PantryEntryDto>> Function()? _build;
  final Future<PantryEntryDto> Function(String, IngredientRefDto, bool)?
  _setMark;

  @override
  Future<List<PantryEntryDto>> fetchPantry(String householdId) async =>
      (_build ?? () => pantryEntries)();

  @override
  Future<PantryEntryDto> writeMark(
    String householdId,
    IngredientRefDto ingredient,
    bool marked,
  ) async {
    final fake = _setMark;
    if (fake == null) {
      throw StateError(
        'this test toggles a pantry row without a `setMark:` hook',
      );
    }
    return fake(householdId, ingredient, marked);
  }
}

/// The *real* `PlannerNotifier` with only its five bridge seams replaced, as
/// `_FakePantryNotifier` is: its own `build`, `refresh`, `save`, `setLock` and `delete` run,
/// so the republish logic is what every planner widget assertion verifies.
class _FakePlannerNotifier extends PlannerNotifier {
  _FakePlannerNotifier({
    this.window,
    this.meals,
    this.save_,
    this.lock,
    this.delete_,
    this.seenWindow,
  });

  final FutureOr<PlanningCycleDto> Function(int offset)? window;

  /// Records what `_fetch` anchored the read to. Separate from [window] so the existing
  /// `cycleWindow:` hooks keep their one-argument shape.
  final void Function(String today, int offset)? seenWindow;
  final FutureOr<List<PlannedMealDto>> Function()? meals;
  final Future<PlannedMealDto> Function(PlannedMealDto)? save_;
  final Future<PlannedMealDto> Function(String, String, bool)? lock;
  final Future<void> Function(String, String)? delete_;

  @override
  Future<PlanningCycleDto> fetchWindow(
    String householdId,
    String today,
    int offset,
  ) async {
    seenWindow?.call(today, offset);
    return (window ?? (_) => okCycle)(offset);
  }

  @override
  Future<List<PlannedMealDto>> fetchMeals(
    String householdId,
    String from,
    String to,
  ) async => (meals ?? () => const <PlannedMealDto>[])();

  @override
  Future<PlannedMealDto> storeMeal(PlannedMealDto meal) {
    final fake = save_;
    if (fake == null) {
      throw StateError('this test saves a meal without a `savePlanned:` hook');
    }
    return fake(meal);
  }

  @override
  Future<PlannedMealDto> writeLock(
    String householdId,
    String mealId,
    bool locked,
  ) {
    final fake = lock;
    if (fake == null) {
      throw StateError('this test locks a meal without a `lockPlanned:` hook');
    }
    return fake(householdId, mealId, locked);
  }

  @override
  Future<void> removeMeal(String householdId, String mealId) {
    final fake = delete_;
    if (fake == null) {
      throw StateError(
        'this test deletes a meal without a `deletePlanned:` hook',
      );
    }
    return fake(householdId, mealId);
  }
}

/// The *real* `ShoppingNotifier` with only its seven bridge seams replaced, as
/// `_FakePlannerNotifier` is: its own `build`, `refresh` and every republish run, so the
/// merge-by-key logic is what every shopping widget assertion verifies.
class _FakeShoppingNotifier extends ShoppingNotifier {
  _FakeShoppingNotifier({
    this.window,
    this.view,
    this.lineState,
    this.manualItem,
    this.deleteItem,
    this.reset_,
    this.pantryMarks,
  });

  final FutureOr<PlanningCycleDto> Function(int offset)? window;
  final FutureOr<ShoppingViewDto> Function(String from, String to)? view;
  final Future<ShoppingLineStateDto> Function(ShoppingLineStateDto)? lineState;
  final Future<ShoppingManualItemDto> Function(ShoppingManualItemDto)?
  manualItem;
  final Future<void> Function(String id)? deleteItem;
  final Future<void> Function(String from, String to)? reset_;
  final Future<List<IngredientRefDto>> Function(List<IngredientRefDto>, bool)?
  pantryMarks;

  @override
  Future<PlanningCycleDto> fetchWindow(
    String householdId,
    String today,
    int offset,
  ) async => (window ?? (_) => okCycle)(offset);

  @override
  Future<ShoppingViewDto> fetchView(
    String householdId,
    String from,
    String to,
  ) async => (view ?? (_, _) => emptyShoppingView)(from, to);

  @override
  Future<ShoppingLineStateDto> writeLineState(
    String householdId,
    String from,
    String to,
    ShoppingLineStateDto state,
  ) {
    final fake = lineState;
    if (fake == null) {
      throw StateError('this test writes a line state without a hook');
    }
    return fake(state);
  }

  @override
  Future<ShoppingManualItemDto> writeManualItem(ShoppingManualItemDto item) {
    final fake = manualItem;
    if (fake == null) {
      throw StateError('this test saves a manual item without a hook');
    }
    return fake(item);
  }

  @override
  Future<void> removeManualItem(String householdId, String itemId) {
    final fake = deleteItem;
    if (fake == null) {
      throw StateError('this test deletes a manual item without a hook');
    }
    return fake(itemId);
  }

  @override
  Future<void> writeReset(String householdId, String from, String to) {
    final fake = reset_;
    if (fake == null) throw StateError('this test resets without a hook');
    return fake(from, to);
  }

  @override
  Future<List<IngredientRefDto>> writePantryMarks(
    String householdId,
    List<IngredientRefDto> ingredients,
    bool marked,
  ) {
    final fake = pantryMarks;
    if (fake == null) {
      throw StateError('this test marks the pantry in bulk without a hook');
    }
    return fake(ingredients, marked);
  }
}

const emptyShoppingList = ShoppingListDto(
  algorithmVersion: 1,
  fromDate: '2026-08-29',
  toDate: '2026-09-04',
  groups: [],
  nonRecipeComponents: 0,
  contributionCount: 0,
);

const emptyShoppingView = ShoppingViewDto(
  list: emptyShoppingList,
  lineStates: [],
  manualItems: [],
  orphanedLineStateCount: 0,
);

const flourKey = 'm:catalog:i-chickpeas:volume_us:req:known';
const onionKey = 'm:catalog:i-onion:count:req:known';
const mysteryKey = 's:pm-1:0:2';

ContributionDto contribution(String meal, String date, String text) =>
    ContributionDto(
      plannedMealId: meal,
      date: date,
      slot: MealSlotDto.dinner,
      componentPosition: 0,
      recipeId: 'r-1',
      recipeTitle: 'Pancakes',
      linePosition: 0,
      originalText: text,
    );

/// Two store groups: `baking` holds a merged line with two contributions (resolved to
/// `chickpeasRef`, so a bulk mark shows on the Pantry tab) and a pantry-omitted line; the
/// uncategorised group holds one unresolved `s:` line.
final okShoppingList = ShoppingListDto(
  algorithmVersion: 1,
  fromDate: '2026-08-29',
  toDate: '2026-09-04',
  groups: [
    ShoppingGroupDto(
      category: 'baking',
      lines: [
        ShoppingLineDto(
          key: flourKey,
          name: 'flour',
          ingredient: chickpeasRef,
          quantity: const QuantityDto.exact(numer: 5, denom: 2),
          unit: const UnitDto.known(unit: 'cup'),
          optional: false,
          status: ShoppingLineStatusDto.needed,
          contributions: [
            contribution('pm-1', '2026-08-29', '1 cup flour'),
            contribution('pm-2', '2026-08-30', '1 1/2 cup flour'),
          ],
        ),
        ShoppingLineDto(
          key: onionKey,
          name: 'onion',
          ingredient: const IngredientRefDto.catalog(id: 'i-onion'),
          quantity: const QuantityDto.exact(numer: 2, denom: 1),
          unit: const UnitDto.known(unit: 'piece'),
          optional: false,
          status: ShoppingLineStatusDto.omittedPantryMarked,
          contributions: [contribution('pm-1', '2026-08-29', '2 onions')],
        ),
      ],
    ),
    ShoppingGroupDto(
      lines: [
        ShoppingLineDto(
          key: mysteryKey,
          name: 'mystery',
          quantity: const QuantityDto.unknown(),
          unit: const UnitDto.none(),
          optional: true,
          status: ShoppingLineStatusDto.needed,
          separateReason: SeparateReasonDto.unresolved,
          contributions: [contribution('pm-1', '2026-08-29', 'a mystery')],
        ),
      ],
    ),
  ],
  nonRecipeComponents: 0,
  contributionCount: 4,
);

ShoppingViewDto shoppingView({
  List<ShoppingLineStateDto> states = const [],
  List<ShoppingManualItemDto> items = const [],
  int orphaned = 0,
}) => ShoppingViewDto(
  list: okShoppingList,
  lineStates: states,
  manualItems: items,
  orphanedLineStateCount: orphaned,
);

ShoppingLineStateDto lineStateOf(
  String key, {
  bool checked = false,
  bool hidden = false,
  bool restored = false,
  bool changed = false,
  String? checkedAgainst,
}) => ShoppingLineStateDto(
  key: key,
  checked: checked,
  hidden: hidden,
  restored: restored,
  changed: changed,
  checkedAgainst: checkedAgainst,
);

/// Echoes a line-state write back as stored: `changed` off, and a token attached iff
/// checked — what Rust returns.
Future<ShoppingLineStateDto> echoLineState(ShoppingLineStateDto s) async =>
    lineStateOf(
      s.key,
      checked: s.checked,
      hidden: s.hidden,
      restored: s.restored,
      checkedAgainst: s.checked ? 'exact:5/2|known:cup' : null,
    );

const batteries = ShoppingManualItemDto(
  id: 'mi-1',
  householdId: 'h-1',
  name: 'batteries',
  note: 'AA',
  checked: false,
);

/// The six kind tokens the real bridge returns (`known_meal_component_kinds`).
const componentKindTokens = [
  'recipe',
  'leftovers',
  'dining_out',
  'frozen_quick',
  'freeform',
  'open',
];

/// A stored occurrence: `okSummary`'s recipe on the first dinner of `okCycle`.
const okPlanned = PlannedMealDto(
  id: 'pm-1',
  householdId: 'h-1',
  date: '2026-08-29',
  slot: MealSlotDto.dinner,
  components: [MealComponentDto(kind: 'recipe', recipeId: 'r-1')],
  locked: false,
);

PlannedMealDto planned({
  String id = 'pm-1',
  String date = '2026-08-29',
  MealSlotDto slot = MealSlotDto.dinner,
  List<MealComponentDto> components = const [
    MealComponentDto(kind: 'recipe', recipeId: 'r-1'),
  ],
  bool locked = false,
}) => PlannedMealDto(
  id: id,
  householdId: 'h-1',
  date: date,
  slot: slot,
  components: components,
  locked: locked,
);

/// One catalog identity carrying an alias that is not a substring of its canonical name, and
/// one custom identity, so alias search and the custom subtitle both have something to bite on.
const chickpeasRef = IngredientRefDto.catalog(id: 'i-chickpeas');
const houseMixRef = IngredientRefDto.custom(id: 'c-1');
const pantryEntries = [
  PantryEntryDto(
    ingredient: chickpeasRef,
    name: 'chickpeas',
    aliases: ['garbanzo beans'],
    marked: false,
  ),
  PantryEntryDto(
    ingredient: houseMixRef,
    name: "nana's mix",
    aliases: [],
    marked: false,
  ),
];

PantryEntryDto pantryEntryMarked(IngredientRefDto ingredient, bool marked) {
  final entry = pantryEntries.firstWhere((e) => e.ingredient == ingredient);
  return PantryEntryDto(
    ingredient: entry.ingredient,
    name: entry.name,
    aliases: entry.aliases,
    marked: marked,
  );
}

/// The eleven unit tokens the real bridge returns (`known_unit_kinds`).
const unitKindTokens = [
  'tsp',
  'tbsp',
  'cup',
  'fl_oz',
  'ml',
  'l',
  'g',
  'kg',
  'oz',
  'lb',
  'piece',
];

/// The eleven tokens the real bridge returns, so the editor's checkbox list is the same
/// list in tests as in production without loading the native library.
const knownKinds = [
  'peanuts',
  'tree_nuts',
  'dairy',
  'eggs',
  'gluten',
  'soy',
  'fish',
  'shellfish',
  'sesame',
  'vegetarian',
  'vegan',
];

Widget harness({
  String initial = '/plan',
  FutureOr<HealthReport> Function()? health,
  FutureOr<HouseholdDto> Function()? household,
  Future<HouseholdDto> Function(String, String?)? rename,
  Future<HouseholdDto> Function(String)? completeOnboarding,
  FutureOr<PlanningCycleDto> Function()? cycle,
  Future<PlanningCycleDto> Function(String, String, int, List<MealSlotDto>)?
  saveCycle,
  FutureOr<List<RestrictionDto>> Function()? restrictions,
  Future<List<RestrictionDto>> Function(String, List<RestrictionDto>)?
  saveRestrictions,
  FutureOr<List<String>> Function()? kinds,
  FutureOr<List<RecipeSummaryDto>> Function()? recipes,
  FutureOr<List<RecipeSummaryDto>> Function()? archived,
  FutureOr<RecipeDto?> Function(String)? recipe,
  Future<RecipeDto> Function(RecipeDto)? saveRecipe,
  Future<RecipeDto> Function(String, String)? archiveRecipe,
  Future<RecipeDto> Function(String, String)? restoreRecipe,
  FutureOr<List<String>> Function()? unitKinds,
  Future<StarterInstallReportDto> Function(String)? starterInstall,
  FutureOr<List<PantryEntryDto>> Function()? pantry,
  Future<PantryEntryDto> Function(String, IngredientRefDto, bool)? setMark,
  FutureOr<PlanningCycleDto> Function(int)? cycleWindow,
  FutureOr<List<PlannedMealDto>> Function()? planner,
  Future<PlannedMealDto> Function(PlannedMealDto)? savePlanned,
  Future<PlannedMealDto> Function(String, String, bool)? lockPlanned,
  Future<void> Function(String, String)? deletePlanned,
  FutureOr<List<String>> Function()? componentKinds,
  FutureOr<ShoppingViewDto> Function(String, String)? shopping,
  Future<ShoppingLineStateDto> Function(ShoppingLineStateDto)? setLineState,
  Future<ShoppingManualItemDto> Function(ShoppingManualItemDto)? saveItem,
  Future<void> Function(String)? deleteItem,
  Future<void> Function(String, String)? resetShopping,
  Future<List<IngredientRefDto>> Function(List<IngredientRefDto>, bool)?
  setPantryMarks,
  FutureOr<CoverCycleOutcomeDto> Function(CoverCycleRequestDto)? cover,
  Future<PlanDecisionOutcomeDto> Function(PlanDecisionRequestDto)? decide,
  BackupActions? backup,
}) => ProviderScope(
  overrides: [
    // Unconditional, like the planner's: the Cover route is reachable from Plan, and an
    // un-overridden provider would reach the real bridge.
    coverProvider.overrideWith(() => _FakeCoverNotifier(cover, decide)),
    // Unconditional, as the planner override is: the destinations, restoration,
    // accessibility and text-scale tests all visit Shopping.
    shoppingProvider.overrideWith(
      () => _FakeShoppingNotifier(
        window: cycleWindow,
        view: shopping,
        lineState: setLineState,
        manualItem: saveItem,
        deleteItem: deleteItem,
        reset_: resetShopping,
        pantryMarks: setPantryMarks,
      ),
    ),
    // Unconditional: Plan is home, so every launch reaches the planner, and an un-overridden
    // provider would reach the real bridge.
    plannerProvider.overrideWith(
      () => _FakePlannerNotifier(
        window: cycleWindow,
        meals: planner,
        save_: savePlanned,
        lock: lockPlanned,
        delete_: deletePlanned,
      ),
    ),
    mealComponentKindsProvider.overrideWith(
      (_) async => (componentKinds ?? () => componentKindTokens)(),
    ),
    // Unconditional, like every other bridge seam here: `App` subscribes to this on the
    // first frame, and an un-overridden provider would reach the real bridge, which
    // `flutter test` never initialises.
    starterInstallProvider.overrideWithValue(
      starterInstall ?? (_) async => okStarterReport,
    ),
    // Unconditional, as the restriction overrides are: the "all five destinations" test
    // visits Recipes, and an un-overridden provider would reach the real bridge.
    recipeLibraryProvider.overrideWith(
      () => _FakeRecipeLibraryNotifier(
        recipes,
        saveRecipe,
        archiveRecipe,
        restoreRecipe,
      ),
    ),
    archivedRecipesProvider.overrideWith(
      () => _FakeArchivedRecipesNotifier(archived),
    ),
    recipeDetailProvider.overrideWith(
      (_, id) async => (recipe ?? (id) => id == 'r-1' ? okRecipe : null)(id),
    ),
    knownUnitKindsProvider.overrideWith(
      (_) async => (unitKinds ?? () => unitKindTokens)(),
    ),
    healthReportProvider.overrideWith((_) => (health ?? () => okReport)()),
    // Unconditional, like the other bridge seams: the backup buttons live on Settings,
    // and an un-overridden provider would reach the real bridge and the filesystem.
    backupActionsProvider.overrideWithValue(backup ?? _FakeBackupActions()),
    householdProvider.overrideWith(
      () => _FakeHouseholdNotifier(household, rename, completeOnboarding),
    ),
    planningCycleProvider.overrideWith(
      () => _FakePlanningCycleNotifier(cycle, saveCycle),
    ),
    // Unconditional: Settings watches this, so an un-overridden provider would send every
    // Settings-scoped test to the real bridge.
    restrictionsProvider.overrideWith(
      () => _FakeRestrictionsNotifier(restrictions, saveRestrictions),
    ),
    knownRestrictionKindsProvider.overrideWith(
      (_) async => (kinds ?? () => knownKinds)(),
    ),
    // Unconditional, as the restriction overrides are: the destinations, restoration,
    // accessibility and text-scale tests all visit Pantry, and an un-overridden provider
    // would reach the real bridge.
    pantryProvider.overrideWith(() => _FakePantryNotifier(pantry, setMark)),
  ],
  child: App(initialLocation: initial),
);

/// A Cover outcome, defaults benign: one covered slot, no attention, quiet assumptions.
CoverCycleOutcomeDto coverOutcome({
  OutcomeStatusDto status = OutcomeStatusDto.covered,
  List<SlotCoverageDto> slots = const [
    SlotCoverageDto(
      date: '2026-08-29',
      slot: MealSlotDto.dinner,
      state: CoverageStateDto.covered,
      components: [MealComponentDto(kind: 'recipe', recipeId: 'r-1')],
      reasonCodes: [],
    ),
  ],
  List<AttentionRequestDto> attention = const [],
  List<String> assumptions = const ['PANTRY_INCOMPLETE'],
  bool applied = false,
}) => CoverCycleOutcomeDto(
  result: PlanningResultDto(
    algorithmVersion: 2,
    snapshotHash: 'aaaaaaaaaaaaaaaa',
    status: status,
    horizonFrom: '2026-08-29',
    horizonTo: '2026-09-04',
    slots: slots,
    proposed: const [],
    rejections: const [],
    unresolvedIssues: const [],
    assumptions: assumptions,
    reasonCodes: const [],
    proposals: const [],
    attention: attention,
    search: const SearchTraceDto(
      beamWidth: 8,
      candidatesPerSlot: 12,
      slotOrder: 'chronological',
      statesScored: 1,
    ),
    scoreTiers: Int64List(6),
  ),
  ledgerEntryId: 'le-1',
  applied: applied,
  changedSlots: applied ? 1 : 0,
);

AttentionRequestDto attentionRequest({
  String id = 'food:infeasible:2026-08-29:dinner',
  UrgencyDto urgency = UrgencyDto.high,
  List<String> options = const [],
  List<String> codes = const ['PLAN_INFEASIBLE'],
}) => AttentionRequestDto(
  id: id,
  controllerId: 'food',
  urgency: urgency,
  decisionBenefitBand: BandDto.high,
  estimatedEffortBand: BandDto.low,
  options: options,
  reasonCodes: codes,
);

/// The Cover seams, faked like every other notifier: `cover` maps `apply` to an outcome;
/// `decide` captures the decision requests the screen issues.
class _FakeCoverNotifier extends CoverNotifier {
  _FakeCoverNotifier(this._cover, this._decide);

  final FutureOr<CoverCycleOutcomeDto> Function(CoverCycleRequestDto)? _cover;
  final Future<PlanDecisionOutcomeDto> Function(PlanDecisionRequestDto)?
  _decide;

  @override
  Future<CoverCycleOutcomeDto> runCover(CoverCycleRequestDto request) async =>
      (_cover ?? (_) => coverOutcome())(request);

  @override
  Future<PlanDecisionOutcomeDto> recordDecision(
    PlanDecisionRequestDto request,
  ) {
    final fake = _decide;
    if (fake == null) {
      throw StateError('this test records a decision without a `decide:` hook');
    }
    return fake(request);
  }
}

const labels = ['Recipes', 'Plan', 'Pantry', 'Shopping', 'Settings'];

/// Upper bound for the traversal test. The bar is entered one press after the
/// last focusable ahead of it, and Plan now contributes a count that scales with
/// the window: two cycle arrows, `Cover My Week`, then per cell either `Add` or a
/// lock switch and its component menus. Deliberately not restated as a number —
/// pinning one is what went stale when the planner replaced the placeholder. The
/// bound exists only to stop a broken traversal looping forever, so it is set
/// well above any plausible count rather than just above today's.
const maxTabPresses = 60;

Finder tab(String label) =>
    find.descendant(of: find.byType(NavigationBar), matching: find.text(label));
Finder title(String text) =>
    find.descendant(of: find.byType(AppBar), matching: find.text(text));

/// Pixel-5 view. Called at the top of each test body rather than from `setUp`,
/// where `tester` is out of scope. The guideline matchers in test 6 depend on
/// view size and device pixel ratio, so the geometry cannot be left to default.
void usePixel5(WidgetTester tester) {
  tester.view.physicalSize = const Size(1080, 2400);
  tester.view.devicePixelRatio = 2.75;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  addTearDown(tester.platformDispatcher.clearPlatformBrightnessTestValue);
  addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
}

/// Same device pixel ratio as [usePixel5] but tall enough for the whole Restrictions form —
/// eleven checkboxes, the chip row, the free-text entry and two buttons — to be laid out at
/// once. `ListView` builds lazily, so on a phone-height viewport the buttons below the fold
/// do not exist in the tree and a tap fails on a missing widget rather than on the behaviour
/// under test. Scrolling to them instead leaves the target under the app bar, where the tap
/// lands on the wrong widget. The tests that genuinely depend on phone geometry — the
/// accessibility guidelines and text scale 2.0 — keep [usePixel5].
void useTallView(WidgetTester tester) {
  tester.view.physicalSize = const Size(1080, 3600);
  tester.view.devicePixelRatio = 2.75;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  addTearDown(tester.platformDispatcher.clearPlatformBrightnessTestValue);
  addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
}

void main() {
  // `?offset=` is the only place a String becomes a cycle offset, and the value crosses to
  // Rust as an i32: `sse_encode_i_32` is `putInt32`, which keeps the low 32 bits silently.
  // Saturating is what stops an out-of-range deep link from previewing an unrelated window.
  group('coverOffset', () {
    test('an unreadable value is the active cycle, as before', () {
      expect(coverOffset(null), 0);
      expect(coverOffset(''), 0);
      expect(coverOffset('abc'), 0);
    });

    test('an in-range value passes through untouched', () {
      expect(coverOffset('-3'), -3);
      // Past the ±520 bound but inside i32: deliberately *not* clamped here, so the bridge
      // still refuses it in prose rather than the route silently moving the user to week 520.
      expect(coverOffset('1000'), 1000);
    });

    test('a value outside i32 saturates rather than narrowing', () {
      // Unsaturated these truncate to 1 and -1 — an in-range window the URL never asked for,
      // which then passes the ±520 bound.
      expect(coverOffset('4294967297'), 2147483647);
      expect(coverOffset('-4294967297'), -2147483648);
    });
  });

  testWidgets('all five destinations are reachable', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness());
    await tester.pumpAndSettle();
    for (final label in labels) {
      await tester.tap(tab(label));
      await tester.pumpAndSettle();
      expect(title(label), findsOneWidget, reason: 'destination $label');
    }
  });

  testWidgets('Cover My Week is reachable under Plan and returns', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness());
    await tester.pumpAndSettle();

    await tester.tap(find.text('Cover My Week'));
    await tester.pumpAndSettle();
    expect(title('Cover My Week'), findsOneWidget);
    // The real result view (AC-1): a coherent preview from one action, no generate step.
    expect(find.text(coveredCopy), findsOneWidget);
    expect(
      find.byKey(const ValueKey('cover:2026-08-29:dinner')),
      findsOneWidget,
    );

    await tester.tap(find.byType(BackButton));
    await tester.pumpAndSettle();
    expect(title('Plan'), findsOneWidget);
  });

  testWidgets('cover surfaces exactly the material questions (AC-3)', (
    tester,
  ) async {
    usePixel5(tester);
    // Material side: two High requests render two question cards and the count banner.
    await tester.pumpWidget(
      harness(
        cover: (req) => coverOutcome(
          status: OutcomeStatusDto.needsAttention,
          attention: [
            attentionRequest(),
            attentionRequest(
              id: 'food:wording:2026-08-30:dinner',
              codes: ['HARD_CONSTRAINT_UNRESOLVED'],
            ),
          ],
        ),
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text(needsYouCopy(2)), findsOneWidget);
    expect(find.text(infeasibleQuestionCopy), findsOneWidget);
    expect(find.text(wordingQuestionCopy), findsOneWidget);
    // NeedsAttention hides Accept (AC-4 refusal path).
    expect(find.text('Accept'), findsNothing);
  });

  testWidgets('a low-value-only cover result asks no question (AC-3)', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        cover: (req) => coverOutcome(assumptions: const ['PANTRY_INCOMPLETE']),
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text(coveredCopy), findsOneWidget);
    expect(find.text(questionsHeading), findsNothing);
    // Pantry stays quiet: non-blocking, no disclosure entry (invariant 6).
    expect(find.text(assumptionsHeading), findsNothing);
    expect(find.text('Accept'), findsOneWidget);
  });

  testWidgets(
    'accept reaches the seam with apply and links shopping (AC-4/5)',
    (tester) async {
      usePixel5(tester);
      final applies = <bool>[];
      await tester.pumpWidget(
        harness(
          cover: (req) {
            applies.add(req.apply);
            return coverOutcome(applied: req.apply);
          },
          initial: '/plan/cover',
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Accept'));
      await tester.pumpAndSettle();
      expect(applies, [false, true], reason: 'preview then one-tap apply');
      expect(find.text(shoppingReadyCopy), findsOneWidget);
      await tester.tap(find.text(shoppingReadyCopy));
      await tester.pumpAndSettle();
      expect(title('Shopping'), findsOneWidget);
    },
  );

  testWidgets(
    'a locked cover slot is marked and offers no swap or veto (AC-2)',
    (tester) async {
      usePixel5(tester);
      await tester.pumpWidget(
        harness(
          cover: (req) => coverOutcome(
            slots: const [
              SlotCoverageDto(
                date: '2026-08-29',
                slot: MealSlotDto.dinner,
                state: CoverageStateDto.lockedByUser,
                components: [MealComponentDto(kind: 'recipe', recipeId: 'r-1')],
                reasonCodes: [],
              ),
            ],
          ),
          initial: '/plan/cover',
        ),
      );
      await tester.pumpAndSettle();
      expect(
        find.byWidgetPredicate(
          (w) => w is Semantics && w.properties.label == lockLabel(true),
        ),
        findsOneWidget,
      );
      expect(find.byIcon(Icons.lock), findsOneWidget);
      expect(find.text('Swap'), findsNothing);
      expect(find.text('Never suggest'), findsNothing);
    },
  );

  testWidgets('swap and reviewed-marker reach the decision seam (AC-4)', (
    tester,
  ) async {
    usePixel5(tester);
    final decisions = <PlanDecisionDto>[];
    await tester.pumpWidget(
      harness(
        cover: (req) => coverOutcome(
          status: OutcomeStatusDto.tentativelyCovered,
          assumptions: const [
            'PANTRY_INCOMPLETE',
            'RESTRICTIONS_NOT_CONFIGURED',
          ],
        ),
        decide: (req) async {
          decisions.add(req.decision);
          return const PlanDecisionOutcomeDto(
            ledgerEntryId: 'le-2',
            priorStatus: OutcomeStatusDto.tentativelyCovered,
            resultingStatus: OutcomeStatusDto.tentativelyCovered,
          );
        },
        recipes: () => const [okSummary],
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    // Two taps: Swap on the tile, then a recipe in the picker.
    await tester.tap(find.text('Swap'));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('cover:pick:r-1')));
    await tester.pumpAndSettle();
    expect(decisions, hasLength(1));
    expect(
      decisions.single,
      const PlanDecisionDto.swap(
        date: '2026-08-29',
        slot: MealSlotDto.dinner,
        components: [MealComponentDto(kind: 'recipe', recipeId: 'r-1')],
      ),
    );
    // The reviewed-marker row shows only while the assumption is present; one tap records.
    await tester.tap(find.text(reviewedRowNoneLabel));
    await tester.pumpAndSettle();
    expect(decisions.last, const PlanDecisionDto.restrictionsReviewed());
  });

  // The one irreversible action on the screen. The native test drives the bridge with a
  // hand-built DTO, so only this can catch a wrong `subject` — the recipe id instead of the
  // title would tokenise to nothing and silently veto nothing.
  testWidgets('the veto path confirms, cancels, and reaches the seam (AC-4)', (
    tester,
  ) async {
    usePixel5(tester);
    final decisions = <PlanDecisionDto>[];
    await tester.pumpWidget(
      harness(
        cover: (req) => coverOutcome(),
        decide: (req) async {
          decisions.add(req.decision);
          return const PlanDecisionOutcomeDto(
            ledgerEntryId: 'le-3',
            priorStatus: OutcomeStatusDto.covered,
            resultingStatus: OutcomeStatusDto.covered,
          );
        },
        recipes: () => const [okSummary],
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Never suggest'));
    await tester.pumpAndSettle();
    expect(find.text(vetoConfirmTitle('Pancakes')), findsOneWidget);
    expect(find.text(vetoConfirmBody('Pancakes')), findsOneWidget);
    // Cancelling is a real answer: nothing reaches the seam.
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(decisions, isEmpty);
    // Both the tile button and the dialog's confirm read "Never suggest".
    await tester.tap(find.text('Never suggest'));
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.text('Never suggest'),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      decisions.single,
      const PlanDecisionDto.veto(
        subject: 'Pancakes',
        date: '2026-08-29',
        slot: MealSlotDto.dinner,
      ),
    );
  });

  // An option the screen cannot name is not a control: `wording_only` options are the
  // household's own restriction phrases, which never parse as `recipe:` tokens.
  testWidgets('unnameable options render no chip of their own', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        cover: (req) => coverOutcome(
          status: OutcomeStatusDto.needsAttention,
          attention: [
            attentionRequest(
              codes: const ['HARD_CONSTRAINT_UNRESOLVED'],
              options: const ['no shellfish', 'nothing with peanuts'],
            ),
          ],
        ),
        recipes: () => const [okSummary],
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    final card = find.byKey(
      const ValueKey('cover:question:food:infeasible:2026-08-29:dinner'),
    );
    // Only the card's own generic Swap chip, and the header label is not duplicated.
    expect(
      find.descendant(of: card, matching: find.byType(ActionChip)),
      findsOneWidget,
    );
    expect(
      find.descendant(of: card, matching: find.text('2026-08-29 · Dinner')),
      findsOneWidget,
    );
  });

  // A wording question is raised from `chosen` with no filter on slot state, and Tier 0 keeps
  // a locked candidate feasible, so a locked slot does reach this card. Its Swap could never
  // succeed — `record_decision` refuses a swap onto a locked row (invariant 18) — so offering
  // it was a dead end. `_slotTile` has always gated its own Swap on the lock; the card now
  // does the same, and says why in the household's terms rather than automation's.
  testWidgets('a question on a locked slot offers no swap, and says why', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        cover: (req) => coverOutcome(
          status: OutcomeStatusDto.tentativelyCovered,
          slots: const [
            SlotCoverageDto(
              date: '2026-08-29',
              slot: MealSlotDto.dinner,
              state: CoverageStateDto.lockedByUser,
              components: [MealComponentDto(kind: 'recipe', recipeId: 'r-1')],
              reasonCodes: [],
            ),
          ],
          attention: [
            attentionRequest(
              id: 'food:wording:2026-08-29:dinner',
              codes: const ['HARD_CONSTRAINT_UNRESOLVED'],
              options: const ['no shellfish'],
            ),
          ],
        ),
        recipes: () => const [okSummary],
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    final card = find.byKey(
      const ValueKey('cover:question:food:wording:2026-08-29:dinner'),
    );
    expect(
      find.descendant(of: card, matching: find.byType(ActionChip)),
      findsNothing,
    );
    expect(
      find.descendant(of: card, matching: find.text(questionLockedCopy)),
      findsOneWidget,
    );
    // The question itself still stands — the card explains the missing action, it does not
    // withdraw the question.
    expect(
      find.descendant(of: card, matching: find.text(wordingQuestionCopy)),
      findsOneWidget,
    );
  });

  // Expected-to-pass: pins the gate's deliberate open branch. A question whose `(date, slot)`
  // is not among the rendered slots has no lock state to read, so it keeps its Swap chip —
  // the refusal below it is now honest prose either way.
  testWidgets('a question addressing no rendered slot keeps its swap chip', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        cover: (req) => coverOutcome(
          status: OutcomeStatusDto.needsAttention,
          attention: [
            attentionRequest(
              id: 'food:wording:2026-09-01:dinner',
              codes: const ['HARD_CONSTRAINT_UNRESOLVED'],
            ),
          ],
        ),
        recipes: () => const [okSummary],
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.descendant(
        of: find.byKey(
          const ValueKey('cover:question:food:wording:2026-09-01:dinner'),
        ),
        matching: find.byType(ActionChip),
      ),
      findsOneWidget,
    );
  });

  // Expected-to-pass: `_report` and the `_busy` release in `finally` already work; this is the
  // coverage pin they never had, on a screen where every mutation is irreversible or
  // lock-bearing. The message is the one `error.rs` maps `SwapOntoLockedSlot` to — the test
  // constructs the error itself, so a reword there must be mirrored here.
  testWidgets(
    'a refused decision surfaces as a snackbar and frees the screen',
    (tester) async {
      usePixel5(tester);
      const refusal = KimattaError.planning(
        message: 'that meal is locked, so it cannot be swapped',
      );
      await tester.pumpWidget(
        harness(
          cover: (req) => coverOutcome(),
          decide: (req) async => throw refusal,
          recipes: () => const [okSummary],
          initial: '/plan/cover',
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Swap'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey('cover:pick:r-1')));
      await tester.pumpAndSettle();
      expect(
        find.descendant(
          of: find.byType(SnackBar),
          matching: find.text(
            describeFailure(refusal, subject: 'Cover My Week'),
          ),
        ),
        findsOneWidget,
      );
      // `_busy` is released in `finally`, so the screen is not left inert after a refusal.
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Swap'))
            .onPressed,
        isNotNull,
      );
    },
  );

  testWidgets('only a resolvable recipe option becomes a swap chip', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        cover: (req) => coverOutcome(
          status: OutcomeStatusDto.needsAttention,
          attention: [
            // Resolvable, unresolvable id, and a multi-component join.
            attentionRequest(
              options: const [
                'recipe:r-1:1/1',
                'recipe:r-missing:1/1',
                'recipe:r-1:1/1,recipe:r-2:1/1',
              ],
            ),
          ],
        ),
        recipes: () => const [okSummary],
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    final card = find.byKey(
      const ValueKey('cover:question:food:infeasible:2026-08-29:dinner'),
    );
    expect(
      find.descendant(of: card, matching: find.text(swapToLabel('Pancakes'))),
      findsOneWidget,
    );
    // The swap chip plus the card's own Swap — the two unnameable options add nothing.
    expect(
      find.descendant(of: card, matching: find.byType(ActionChip)),
      findsNWidgets(2),
    );
  });

  testWidgets('the cover screen renders a load failure and Try again retries', (
    tester,
  ) async {
    usePixel5(tester);
    var reads = 0;
    await tester.pumpWidget(
      harness(
        cover: (req) =>
            ++reads == 1 ? throw const KimattaError.notOpen() : coverOutcome(),
        initial: '/plan/cover',
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text(
        describeFailure(const KimattaError.notOpen(), subject: 'Cover My Week'),
      ),
      findsOneWidget,
    );
    await tester.tap(find.text('Try again'));
    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey('cover:2026-08-29:dinner')),
      findsOneWidget,
    );
  });

  // AC-7 — the cover screen through the same bounded checks every destination passes,
  // over a result that renders every affordance: a question card with options, an
  // unlocked tile with Swap/veto, the reviewed row and the disclosure.
  testWidgets('the cover screen meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(
        harness(
          cover: (req) => coverOutcome(
            status: OutcomeStatusDto.needsAttention,
            attention: [
              attentionRequest(options: const ['recipe:r-1:1/1']),
            ],
            assumptions: const [
              'PANTRY_INCOMPLETE',
              'RESTRICTIONS_NOT_CONFIGURED',
            ],
          ),
          recipes: () => const [okSummary],
          initial: '/plan/cover',
        ),
      );
      await tester.pumpAndSettle();
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
    }
  });

  testWidgets('/plan/cover survives text scale 2.0', (tester) async {
    usePixel5(tester);
    tester.platformDispatcher.textScaleFactorTestValue = 2.0;
    await tester.pumpWidget(harness(initial: '/plan/cover'));
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  });

  testWidgets('an unknown route renders not-found with a return action', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/nope'));
    await tester.pumpAndSettle();
    expect(title('Page not found'), findsOneWidget);
    expect(find.text('No screen at /nope.'), findsOneWidget);

    await tester.tap(find.text('Go to Plan'));
    await tester.pumpAndSettle();
    expect(title('Plan'), findsOneWidget);
  });

  // Expected-to-pass: a regression pin, not a fail-first test. The location is
  // restored by Flutter's own `Router` under `MaterialApp.router`'s 'app' scope
  // (`widgets/app.dart` hard-codes `restorationScopeId: 'router'`, and
  // `_RouterState.restoreState` prefers `setRestoredRoutePath`), so this does
  // not establish that go_router's own restoration ids are load-bearing. What
  // it pins is this card's end-to-end composition: restored location →
  // go_router re-parse → correct branch selected.
  testWidgets('the selected destination survives restoration', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness());
    await tester.pumpAndSettle();

    await tester.tap(tab('Pantry'));
    await tester.pumpAndSettle();
    expect(title('Pantry'), findsOneWidget);

    await tester.restartAndRestore();
    await tester.pumpAndSettle();
    expect(title('Pantry'), findsOneWidget);
  });

  testWidgets('Settings reports the database schema version and path', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness());
    await tester.pumpAndSettle();

    await tester.tap(tab('Settings'));
    await tester.pumpAndSettle();
    expect(find.textContaining('schema v5 at /x/kimatta.db'), findsOneWidget);
  });

  testWidgets('Settings reports a storage failure instead of the report', (
    tester,
  ) async {
    usePixel5(tester);
    // Explicit initial location, so the assertion does not depend on whether
    // the shell builds the Settings branch eagerly or lazily.
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () => throw const KimattaError.storage(message: 'disk full'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Local database unavailable: disk full'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Settings reports a damaged database honestly', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () =>
            throw const KimattaError.corrupt(message: 'file is not a database'),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text(
        'Local database unavailable: '
        "the local database is damaged and can't be opened",
      ),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('Settings reports an invalid database path in prose', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () => throw const KimattaError.invalidPath(),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('Local database unavailable: the database path is invalid'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  // Expected-to-pass: pins the `_` fallback arm, which the InvalidPath arm
  // narrows to genuinely foreign errors. The interpolated `toString()` is
  // deliberate and is what is being pinned — for an error the bridge contract
  // does not define, a raw rendering is the honest one, and pinning it makes a
  // later change to it a visible diff rather than a silent one.
  testWidgets('Settings falls back for a non-KimattaError failure', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () => throw StateError('platform channel returned null'),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.textContaining('Local database unavailable: Bad state:'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  // Expected-to-pass: pins the AsyncLoading arm, untouched by the InvalidPath
  // arm. `pumpAndSettle` would hang on the uncompleted future, so the pending
  // state is reached with a bare `pump`.
  testWidgets('Settings shows a checking state before the report arrives', (
    tester,
  ) async {
    usePixel5(tester);
    final pending = Completer<HealthReport>();
    await tester.pumpWidget(
      harness(initial: '/settings', health: () => pending.future),
    );
    await tester.pump();
    expect(find.text('Local database: checking…'), findsOneWidget);

    pending.complete(okReport);
    await tester.pumpAndSettle();
    expect(find.textContaining('schema v5 at /x/kimatta.db'), findsOneWidget);
  });

  // AC-3's bounded UI/state inspection: the default arrives with no setup step.
  testWidgets(
    'Settings shows the dinner-only default cycle without any setup',
    (tester) async {
      usePixel5(tester);
      await tester.pumpWidget(harness(initial: '/settings'));
      await tester.pumpAndSettle();
      expect(find.text('Dinner only · 7 days from 2026-08-29'), findsOneWidget);
    },
  );

  testWidgets('Settings reports a planning failure in prose', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        cycle: () => throw const KimattaError.storage(message: 'locked'),
      ),
    );
    await tester.pumpAndSettle();
    // "Planning cycle", not "Household" — the subject parameter is what makes this honest.
    expect(find.text('Planning cycle unavailable: locked'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  // Expected-to-pass: pins the AsyncLoading arm. `pumpAndSettle` would hang on the
  // uncompleted future, so the pending state is reached with a bare `pump`.
  testWidgets('Settings shows a loading state before the cycle arrives', (
    tester,
  ) async {
    usePixel5(tester);
    final pending = Completer<PlanningCycleDto>();
    await tester.pumpWidget(
      harness(initial: '/settings', cycle: () => pending.future),
    );
    await tester.pump();
    expect(find.text('Loading planning cycle…'), findsOneWidget);

    pending.complete(okCycle);
    await tester.pumpAndSettle();
    expect(find.text('Dinner only · 7 days from 2026-08-29'), findsOneWidget);
  });

  testWidgets('every destination meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(harness());
      await tester.pumpAndSettle();
      for (final label in labels) {
        await tester.tap(tab(label));
        await tester.pumpAndSettle();
        await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
        await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
        await expectLater(tester, meetsGuideline(textContrastGuideline));
      }
    }
  });

  testWidgets('destination screens survive text scale 2.0', (tester) async {
    usePixel5(tester);
    // `NavigationBar` labels are clamped to 1.3 by the framework; what this
    // exercises is the destination bodies and AppBar titles at a genuine 2.0 — including
    // the live pantry screen's disclaimer paragraph, search field and `SwitchListTile`s on
    // Pixel-5 geometry, which is exactly where a switch row overflows.
    tester.platformDispatcher.textScaleFactorTestValue = 2.0;
    await tester.pumpWidget(harness());
    await tester.pumpAndSettle();
    for (final label in labels) {
      await tester.tap(tab(label));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull, reason: 'destination $label');
    }
  });

  testWidgets('tab traversal reaches the navigation bar', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness());
    await tester.pumpAndSettle();

    // Expected-to-pass: a brittleness fix, not a bug fix. Bounded traversal
    // rather than a fixed count, because how many focusables sit ahead of the
    // bar depends on the Plan screen's tree, which MVP-013 has replaced. What is
    // asserted is that the bar is reachable at all, not where a magic number
    // happens to land.
    var reachedBar = false;
    for (var i = 0; i < maxTabPresses && !reachedBar; i++) {
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump();
      final context = FocusManager.instance.primaryFocus?.context;
      reachedBar =
          context != null &&
          find
              .ancestor(
                of: find.byWidget(context.widget),
                matching: find.byType(NavigationBar),
              )
              .evaluate()
              .isNotEmpty;
    }
    expect(
      reachedBar,
      isTrue,
      reason:
          'the NavigationBar was not focused within $maxTabPresses tab presses',
    );
  });

  testWidgets('the health provider is created once per launch', (tester) async {
    usePixel5(tester);
    var creations = 0;
    await tester.pumpWidget(
      harness(
        health: () {
          creations++;
          return okReport;
        },
      ),
    );
    await tester.pumpAndSettle();
    // The load-bearing assertion: created at startup by App's subscription, not
    // deferred until Settings asked. A post-tap-only check reads 1 either way.
    expect(creations, 1);

    await tester.tap(tab('Settings'));
    await tester.pumpAndSettle();
    expect(creations, 1);
  });

  testWidgets('the household is bootstrapped once at launch', (tester) async {
    usePixel5(tester);
    var bootstraps = 0;
    await tester.pumpWidget(
      harness(
        household: () {
          bootstraps++;
          return okHousehold;
        },
      ),
    );
    await tester.pumpAndSettle();
    expect(bootstraps, 1);
    await tester.tap(tab('Settings'));
    await tester.pumpAndSettle();
    expect(bootstraps, 1);
  });

  testWidgets('Settings summarises an unnamed solo household honestly', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/settings'));
    await tester.pumpAndSettle();
    expect(
      find.text('Unnamed household · Just you for now — a household of one.'),
      findsOneWidget,
    );
  });

  testWidgets('the Settings tile opens the Household screen', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/settings'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(ListTile, 'Household'));
    await tester.pumpAndSettle();
    expect(title('Household'), findsOneWidget);
    // Back returns to Settings: nested under the settings branch, not a new tab.
    await tester.tap(find.byType(BackButton));
    await tester.pumpAndSettle();
    expect(title('Settings'), findsOneWidget);
  });

  testWidgets('Household screen shows the identity and the rename controls', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/settings/household'));
    await tester.pumpAndSettle();
    expect(title('Household'), findsOneWidget);
    expect(find.text('Unnamed household'), findsOneWidget);
    expect(find.text('Me'), findsOneWidget);
    expect(find.widgetWithText(FilledButton, 'Save name'), findsOneWidget);
    expect(find.widgetWithText(TextField, 'Household name'), findsOneWidget);
  });

  testWidgets('Household screen reports a storage failure in prose', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/household',
        household: () => throw const KimattaError.storage(message: 'locked'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Household unavailable: locked'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Household screen meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(harness(initial: '/settings/household'));
      await tester.pumpAndSettle();
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
    }
  });

  // Pins `describeFailure`'s NotOpen arm — the one variant the process-wide
  // connection makes genuinely reachable in production, since every bridge call
  // returns it when `open_database` has not run.
  testWidgets('Household screen reports a closed database in prose', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/household',
        household: () => throw const KimattaError.notOpen(),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('Household unavailable: the database is not open'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  // Pins `describeFailure`'s InvalidPath arm.
  testWidgets('Household screen reports an invalid database path in prose', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/household',
        household: () => throw const KimattaError.invalidPath(),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('Household unavailable: the database path is invalid'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  // Pins `describeFailure`'s `_` arm, which the two typed arms above narrow to
  // genuinely foreign errors. As in the Settings fallback test, the raw
  // `toString()` is what is deliberately pinned.
  testWidgets('Household screen falls back for a non-KimattaError failure', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/household',
        household: () => throw StateError('platform channel returned null'),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.textContaining('Household unavailable: Bad state:'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  // Pins `describeFailure`'s Planning arm, which the `_` arm above would
  // otherwise absorb silently — the raw freezed `toString()` that arm was
  // added to prevent. Routed through the planning tile, the only subject a
  // `KimattaError.planning` can reach today.
  testWidgets('Settings reports a planning-domain failure in prose', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        cycle: () => throw const KimattaError.planning(message: 'locked'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Planning cycle unavailable: locked'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  // Pins the Settings health switch's NotOpen arm, added by this card alongside
  // the Household screen's.
  testWidgets('Settings reports a closed database in prose', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () => throw const KimattaError.notOpen(),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('Local database unavailable: the database is not open'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  // Pins `describeHousehold`'s plural arm; `okHousehold` has exactly one member,
  // so no other test reaches it.
  testWidgets('Settings summarises a household with several members', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(initial: '/settings', household: () => twoMemberHousehold),
    );
    await tester.pumpAndSettle();
    expect(find.text('Casa · 2 members.'), findsOneWidget);
  });

  // The finding this pins: a save that outlives the screen. `renameHousehold`
  // commits, then the route is popped before the future returns. Through the
  // notifier the new name still reaches every listener; through the old
  // `ref.invalidate` it raised a swallowed StateError and the UI kept the stale
  // name for the rest of the session.
  testWidgets('a save that outlives the screen still refreshes the UI', (
    tester,
  ) async {
    usePixel5(tester);
    final pending = Completer<HouseholdDto>();
    await tester.pumpWidget(
      harness(initial: '/settings/household', rename: (_, _) => pending.future),
    );
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField), 'Casa');
    await tester.tap(find.widgetWithText(FilledButton, 'Save name'));
    await tester.pump();

    // Leave the Household screen while the write is still in flight.
    await tester.tap(find.byType(BackButton));
    await tester.pumpAndSettle();
    expect(title('Settings'), findsOneWidget);

    pending.complete(
      const HouseholdDto(
        id: 'h-1',
        name: 'Casa',
        members: [MemberDto(id: 'm-1', displayName: 'Me')],
        onboarded: true,
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(
      find.text('Casa · Just you for now — a household of one.'),
      findsOneWidget,
    );
  });

  // The broad `catch` in `_save` still renders every save failure as prose.
  testWidgets('a failed save reports the reason in a snackbar', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/household',
        rename: (_, _) async =>
            throw const KimattaError.storage(message: 'database is locked'),
      ),
    );
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField), 'Casa');
    await tester.tap(find.widgetWithText(FilledButton, 'Save name'));
    await tester.pumpAndSettle();
    expect(
      find.text('Household unavailable: database is locked'),
      findsOneWidget,
    );
    // The button is re-enabled: the `finally` ran.
    expect(
      tester
          .widget<FilledButton>(find.widgetWithText(FilledButton, 'Save name'))
          .onPressed,
      isNotNull,
    );
  });

  // Storage trims the name, so the field has to agree with what was persisted —
  // otherwise the field reads `  Casa  ` while the header reads `Casa`, and
  // re-tapping Save is a no-op the user cannot tell apart from a failure.
  testWidgets('the name field resyncs to the trimmed, stored value', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/household',
        rename: (id, name) async => HouseholdDto(
          id: id,
          name: name?.trim().isEmpty ?? true ? null : name!.trim(),
          members: const [MemberDto(id: 'm-1', displayName: 'Me')],
          onboarded: true,
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField), '  Casa  ');
    await tester.tap(find.widgetWithText(FilledButton, 'Save name'));
    await tester.pumpAndSettle();
    expect(
      tester.widget<TextField>(find.byType(TextField)).controller?.text,
      'Casa',
    );
    expect(find.text('Casa'), findsWidgets);
  });

  // --- MVP-006 -------------------------------------------------------------

  const newHousehold = HouseholdDto(
    id: 'h-1',
    name: null,
    members: [MemberDto(id: 'm-1', displayName: 'Me')],
    onboarded: false,
  );

  // AC-1: a first launch is met by Welcome, outside the shell so no nav bar shows.
  testWidgets('a first launch lands on Welcome', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(household: () => newHousehold));
    await tester.pumpAndSettle();
    expect(title('Welcome'), findsOneWidget);
    expect(find.byType(NavigationBar), findsNothing);
  });

  // Expected-to-pass: the regression pin for every pre-existing test, all of which use an
  // already-onboarded household.
  testWidgets('an onboarded household stays on Plan', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness());
    await tester.pumpAndSettle();
    expect(title('Plan'), findsOneWidget);
  });

  // AC-1: skip reaches the usable dinner-first app, and marks onboarding done once.
  testWidgets('Get started marks onboarding done and reaches Plan', (
    tester,
  ) async {
    usePixel5(tester);
    final completed = <String>[];
    await tester.pumpWidget(
      harness(
        household: () => newHousehold,
        completeOnboarding: (id) async {
          completed.add(id);
          return okHousehold;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Get started'));
    await tester.pumpAndSettle();
    expect(completed, ['h-1']);
    expect(title('Plan'), findsOneWidget);
  });

  testWidgets('Set up first reaches Settings', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        household: () => newHousehold,
        completeOnboarding: (_) async => okHousehold,
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(TextButton, 'Set up first'));
    await tester.pumpAndSettle();
    expect(title('Settings'), findsOneWidget);
  });

  // AC-3: no account creation and no intake — nothing to type, nothing to sign into.
  testWidgets('Welcome asks for no account', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(household: () => newHousehold));
    await tester.pumpAndSettle();
    expect(find.byType(TextField), findsNothing);
    for (final word in ['sign in', 'account', 'password', 'email']) {
      expect(
        find.textContaining(RegExp(word, caseSensitive: false)),
        findsNothing,
        reason: 'Welcome must not mention "$word"',
      );
    }
  });

  // AC-1 again, in the failure direction: a database failure must not trap the user.
  testWidgets('a failed completion still lets the user in', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        household: () => newHousehold,
        completeOnboarding: (_) async =>
            throw const KimattaError.storage(message: 'disk full'),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Get started'));
    await tester.pumpAndSettle();
    expect(find.text('Setup unavailable: disk full'), findsOneWidget);
    expect(title('Plan'), findsOneWidget);
  });

  testWidgets('Welcome meets the accessibility guidelines', (tester) async {
    usePixel5(tester);
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(harness(household: () => newHousehold));
      await tester.pumpAndSettle();
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
    }
  });

  // AC-2: the cycle is editable and the exact request is what the user built.
  testWidgets('the cycle editor saves the edited scope and length', (
    tester,
  ) async {
    usePixel5(tester);
    ({String id, String anchor, int days, List<MealSlotDto> slots})? sent;
    await tester.pumpWidget(
      harness(
        initial: '/settings/cycle',
        saveCycle: (id, anchor, days, slots) async {
          sent = (id: id, anchor: anchor, days: days, slots: slots);
          return okCycle;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(CheckboxListTile, 'Lunch'));
    await tester.pump();
    await tester.tap(find.byTooltip('More days'));
    await tester.pump();
    await tester.tap(find.byTooltip('More days'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, 'Save cycle'));
    await tester.pumpAndSettle();
    // Index order, not tap order: the editor sorts before sending so the round-trip
    // cannot reorder under the user.
    expect(sent?.slots, [MealSlotDto.lunch, MealSlotDto.dinner]);
    expect(sent?.days, 9);
    expect(sent?.anchor, '2026-08-29');
    expect(sent?.id, 'h-1');
  });

  testWidgets('Save cycle is disabled with no meal selected', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/settings/cycle'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(CheckboxListTile, 'Dinner'));
    await tester.pump();
    expect(find.text('Pick at least one meal.'), findsOneWidget);
    expect(
      tester
          .widget<FilledButton>(find.widgetWithText(FilledButton, 'Save cycle'))
          .onPressed,
      isNull,
    );
  });

  // Pins the wipe path shut: no form, so no Save, before the cycle has loaded.
  testWidgets('the cycle editor shows progress and no Save while loading', (
    tester,
  ) async {
    usePixel5(tester);
    final pending = Completer<PlanningCycleDto>();
    await tester.pumpWidget(
      harness(initial: '/settings/cycle', cycle: () => pending.future),
    );
    await tester.pump();
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    expect(find.widgetWithText(FilledButton, 'Save cycle'), findsNothing);
    pending.complete(okCycle);
    await tester.pumpAndSettle();
    expect(find.widgetWithText(FilledButton, 'Save cycle'), findsOneWidget);
  });

  testWidgets('the cycle editor renders a load failure and no form', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/cycle',
        cycle: () => throw const KimattaError.storage(message: 'locked'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Planning cycle unavailable: locked'), findsOneWidget);
    expect(find.widgetWithText(FilledButton, 'Save cycle'), findsNothing);
  });

  testWidgets('Start from today sets the anchor to the local civil date', (
    tester,
  ) async {
    usePixel5(tester);
    String? sentAnchor;
    await tester.pumpWidget(
      harness(
        initial: '/settings/cycle',
        saveCycle: (_, anchor, _, _) async {
          sentAnchor = anchor;
          return okCycle;
        },
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Starts 2026-08-29'), findsOneWidget);
    await tester.tap(find.widgetWithText(TextButton, 'Start from today'));
    await tester.pump();
    final today = todayCivilDate();
    expect(find.text('Starts $today'), findsOneWidget);
    await tester.tap(find.widgetWithText(FilledButton, 'Save cycle'));
    await tester.pumpAndSettle();
    expect(sentAnchor, today);
  });

  testWidgets('a failed cycle save reports the reason and re-enables Save', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/cycle',
        saveCycle: (_, _, _, _) async =>
            throw const KimattaError.planning(message: 'bad anchor'),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Save cycle'));
    await tester.pumpAndSettle();
    expect(find.text('Planning cycle unavailable: bad anchor'), findsOneWidget);
    // The `finally` ran, so the user can retry.
    expect(
      tester
          .widget<FilledButton>(find.widgetWithText(FilledButton, 'Save cycle'))
          .onPressed,
      isNotNull,
    );
  });

  testWidgets('the cycle editor meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(harness(initial: '/settings/cycle'));
      await tester.pumpAndSettle();
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
    }
  });

  // One test per location rather than a loop: re-pumping `harness` with a different
  // `initial` trips `App.didUpdateWidget`, and that assertion leaves the element tree
  // half-updated for every test after it in this file (see the closing test's note).
  for (final location in const [
    '/welcome',
    '/settings/cycle',
    '/settings/restrictions',
    '/recipes',
    '/recipes/new',
    '/recipes/r-1',
  ]) {
    testWidgets('$location survives text scale 2.0', (tester) async {
      usePixel5(tester);
      tester.platformDispatcher.textScaleFactorTestValue = 2.0;
      await tester.pumpWidget(harness(initial: location));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
    });
  }

  testWidgets('the Settings tile opens the cycle editor and Back returns', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/settings'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(ListTile, 'Planning cycle'));
    await tester.pumpAndSettle();
    expect(title('Planning cycle'), findsOneWidget);
    await tester.tap(find.byType(BackButton));
    await tester.pumpAndSettle();
    expect(title('Settings'), findsOneWidget);
  });

  // Invariant 10, verbatim: the screen never claims safety.
  testWidgets('the restrictions editor leads with the warnings-only line', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/settings/restrictions'));
    await tester.pumpAndSettle();
    expect(find.text(restrictionsDisclaimer), findsOneWidget);
  });

  // AC-2: restrictions are saved, and the exact set is what the user built.
  testWidgets('the restrictions editor saves checked kinds and free text', (
    tester,
  ) async {
    useTallView(tester);
    List<RestrictionDto>? sent;
    await tester.pumpWidget(
      harness(
        initial: '/settings/restrictions',
        saveRestrictions: (_, r) async {
          sent = r;
          return r;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(CheckboxListTile, 'Peanuts'));
    await tester.pump();
    await tester.enterText(find.byType(TextField), 'nightshades');
    await tester.tap(find.widgetWithText(OutlinedButton, 'Add'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, 'Save restrictions'));
    await tester.pumpAndSettle();
    expect(sent, [
      const RestrictionDto.known(kind: 'peanuts'),
      const RestrictionDto.other(text: 'nightshades'),
    ]);
  });

  testWidgets('a blank free-text restriction cannot be added', (tester) async {
    useTallView(tester);
    await tester.pumpWidget(harness(initial: '/settings/restrictions'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), '   ');
    await tester.tap(find.widgetWithText(OutlinedButton, 'Add'));
    await tester.pump();
    expect(find.byType(Chip), findsNothing);
  });

  /// A deletion must survive the rebuild a save triggers — the `_seedOnce` guard is what
  /// stops the republished provider re-adding the chip the user just removed.
  testWidgets('a deleted chip stays deleted across the save rebuild', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/restrictions',
        restrictions: () => const [
          RestrictionDto.other(text: 'nightshades'),
          RestrictionDto.other(text: 'cilantro'),
        ],
        saveRestrictions: (_, r) async => r,
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(
        of: find.widgetWithText(Chip, 'nightshades'),
        matching: find.byIcon(Icons.cancel),
      ),
    );
    await tester.pump();
    expect(find.widgetWithText(Chip, 'nightshades'), findsNothing);
    await tester.tap(find.widgetWithText(FilledButton, 'Save restrictions'));
    await tester.pumpAndSettle();
    expect(find.widgetWithText(Chip, 'nightshades'), findsNothing);
    expect(find.widgetWithText(Chip, 'cilantro'), findsOneWidget);
  });

  // The two tests that pin the wipe path shut: `_known`/`_other` start empty and Save
  // writes the whole set, so a Save reachable before the load resolved would clear
  // everything with nothing on screen to show the loss.
  testWidgets(
    'the restrictions editor shows progress and no Save while loading',
    (tester) async {
      useTallView(tester);
      final pending = Completer<List<RestrictionDto>>();
      await tester.pumpWidget(
        harness(
          initial: '/settings/restrictions',
          restrictions: () => pending.future,
        ),
      );
      await tester.pump();
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      // The whole form is absent, not merely the button below the fold — asserting on the
      // checkbox list keeps this falsifiable, since a lazy `ListView` would report the
      // off-screen Save as absent either way.
      expect(find.byType(CheckboxListTile), findsNothing);
      pending.complete(const []);
      await tester.pumpAndSettle();
      expect(find.byType(CheckboxListTile), findsWidgets);
      expect(
        find.widgetWithText(FilledButton, 'Save restrictions'),
        findsOneWidget,
      );
    },
  );

  testWidgets('the restrictions editor renders a load failure and no form', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/restrictions',
        restrictions: () =>
            throw const KimattaError.restriction(message: 'bad token'),
      ),
    );
    await tester.pumpAndSettle();
    // Also pins `describeFailure`'s new Restriction arm, which the `_` arm would
    // otherwise absorb as a raw freezed `toString()`.
    expect(find.text('Restrictions unavailable: bad token'), findsOneWidget);
    // As above: the form is absent entirely, which is what stops an empty Save wiping the
    // stored set from an error screen.
    expect(find.byType(CheckboxListTile), findsNothing);
  });

  testWidgets('a failed restriction save reports the reason in a snackbar', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings/restrictions',
        saveRestrictions: (_, _) async =>
            throw const KimattaError.storage(message: 'database is locked'),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Save restrictions'));
    await tester.pumpAndSettle();
    expect(
      find.text('Restrictions unavailable: database is locked'),
      findsOneWidget,
    );
  });

  testWidgets('Settings summarises an empty restriction list honestly', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/settings'));
    await tester.pumpAndSettle();
    expect(find.text('None set — nothing is filtered out.'), findsOneWidget);
  });

  // A separate test rather than a second `pumpWidget`: re-pumping the harness reuses the
  // already-initialised notifier override, so the new list never reaches the tile.
  testWidgets('Settings summarises a set restriction list by label', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        restrictions: () => const [
          RestrictionDto.known(kind: 'tree_nuts'),
          RestrictionDto.other(text: 'nightshades'),
        ],
      ),
    );
    await tester.pumpAndSettle();
    // The label, not the raw token — `tree_nuts` reaching a user is the regression the
    // bridge cross-check test exists to prevent.
    expect(find.text('Tree nuts, nightshades'), findsOneWidget);
  });

  testWidgets('the restrictions editor meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(harness(initial: '/settings/restrictions'));
      await tester.pumpAndSettle();
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
    }
  });

  // --- MVP-006 fix pass ----------------------------------------------------

  /// The `Known` arm of `_seedOnce` — the one branch on the wipe path no test reached. The
  /// wipe-path tests above assert on the *absence* of the form, so a regression here would
  /// render a stored allergen unchecked and the next Save would drop it, with the checkbox
  /// state on screen looking self-consistent throughout. Expected-to-pass: this closes a
  /// coverage gap rather than pinning a fix.
  testWidgets(
    'a stored known restriction seeds the editor and survives a no-op save',
    (tester) async {
      useTallView(tester);
      List<RestrictionDto>? sent;
      await tester.pumpWidget(
        harness(
          initial: '/settings/restrictions',
          restrictions: () => const [
            RestrictionDto.known(kind: 'peanuts'),
            RestrictionDto.other(text: 'nightshades'),
          ],
          saveRestrictions: (_, r) async {
            sent = r;
            return r;
          },
        ),
      );
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<CheckboxListTile>(
              find.widgetWithText(CheckboxListTile, 'Peanuts'),
            )
            .value,
        isTrue,
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Save restrictions'));
      await tester.pumpAndSettle();
      expect(sent, const [
        RestrictionDto.known(kind: 'peanuts'),
        RestrictionDto.other(text: 'nightshades'),
      ]);
    },
  );

  /// `_save` re-seeds `_known`/`_other` from what the bridge returned, so anything ticked,
  /// added or deleted during the await would be discarded without a message. The three edit
  /// affordances are dead for exactly that window.
  testWidgets('the edit affordances are disabled while a save is in flight', (
    tester,
  ) async {
    useTallView(tester);
    final pending = Completer<List<RestrictionDto>>();
    await tester.pumpWidget(
      harness(
        initial: '/settings/restrictions',
        restrictions: () => const [RestrictionDto.other(text: 'nightshades')],
        saveRestrictions: (_, _) => pending.future,
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Save restrictions'));
    await tester.pump();
    expect(
      tester
          .widget<CheckboxListTile>(
            find.widgetWithText(CheckboxListTile, 'Peanuts'),
          )
          .onChanged,
      isNull,
    );
    expect(
      tester
          .widget<OutlinedButton>(find.widgetWithText(OutlinedButton, 'Add'))
          .onPressed,
      isNull,
    );
    expect(
      tester.widget<Chip>(find.widgetWithText(Chip, 'nightshades')).onDeleted,
      isNull,
    );
    pending.complete(const [RestrictionDto.other(text: 'nightshades')]);
    await tester.pumpAndSettle();
    expect(
      tester
          .widget<CheckboxListTile>(
            find.widgetWithText(CheckboxListTile, 'Peanuts'),
          )
          .onChanged,
      isNotNull,
    );
  });

  /// `is_same` never collapses a `Known`/`Other` pair, so free text naming a kind the list
  /// already covers would be stored, shown and matched twice.
  testWidgets(
    'free text naming a known kind ticks its box instead of adding a chip',
    (tester) async {
      useTallView(tester);
      List<RestrictionDto>? sent;
      await tester.pumpWidget(
        harness(
          initial: '/settings/restrictions',
          saveRestrictions: (_, r) async {
            sent = r;
            return r;
          },
        ),
      );
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextField), 'Peanuts');
      await tester.tap(find.widgetWithText(OutlinedButton, 'Add'));
      await tester.pump();
      expect(find.byType(Chip), findsNothing);
      expect(
        tester
            .widget<CheckboxListTile>(
              find.widgetWithText(CheckboxListTile, 'Peanuts'),
            )
            .value,
        isTrue,
      );
      // The same entry against an already-ticked box: absorbed, not duplicated. `_known` is
      // a set, so the user's intent — warn about peanuts — is already satisfied.
      await tester.enterText(find.byType(TextField), 'peanuts');
      await tester.tap(find.widgetWithText(OutlinedButton, 'Add'));
      await tester.pump();
      expect(find.byType(Chip), findsNothing);
      await tester.tap(find.widgetWithText(FilledButton, 'Save restrictions'));
      await tester.pumpAndSettle();
      expect(sent, const [RestrictionDto.known(kind: 'peanuts')]);
    },
  );

  /// `_finish` needs the household's id, so a tap before it resolves would complete nothing
  /// while still navigating on — reachable through Android state restoration, which restores
  /// `/welcome` while Riverpod rebuilds from scratch.
  testWidgets('Welcome waits for the household before it can complete', (
    tester,
  ) async {
    usePixel5(tester);
    final pending = Completer<HouseholdDto>();
    await tester.pumpWidget(
      harness(initial: '/welcome', household: () => pending.future),
    );
    await tester.pump();
    expect(
      tester
          .widget<FilledButton>(
            find.widgetWithText(FilledButton, 'Get started'),
          )
          .onPressed,
      isNull,
    );
    expect(
      tester
          .widget<TextButton>(find.widgetWithText(TextButton, 'Set up first'))
          .onPressed,
      isNull,
    );
    pending.complete(newHousehold);
    await tester.pumpAndSettle();
    expect(
      tester
          .widget<FilledButton>(
            find.widgetWithText(FilledButton, 'Get started'),
          )
          .onPressed,
      isNotNull,
    );
  });

  /// The error arm the gate above deliberately lets through: buttons dead on a screen with no
  /// navigation bar would trap the user, so they stay live and the failure is said out loud
  /// rather than navigating on as if setup had been recorded.
  testWidgets(
    'Welcome explains a household failure instead of skipping silently',
    (tester) async {
      usePixel5(tester);
      await tester.pumpWidget(
        harness(
          initial: '/welcome',
          household: () =>
              throw const KimattaError.storage(message: 'disk full'),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.widgetWithText(FilledButton, 'Get started'));
      await tester.pumpAndSettle();
      expect(
        find.text('Setup unavailable: the household did not load'),
        findsOneWidget,
      );
      expect(title('Plan'), findsOneWidget);
    },
  );

  // --- MVP-008 -------------------------------------------------------------

  // Pins `describeFailure`'s Recipe arm, added as the Planning and Restriction arms were.
  // Routed through the Restrictions tile: the test needs no recipe screen to be falsifiable.
  testWidgets('Settings reports a recipe-domain failure in prose', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        restrictions: () =>
            throw const KimattaError.recipe(message: 'bad line'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Restrictions unavailable: bad line'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Settings reports a planned-meal failure in prose', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        restrictions: () => throw const KimattaError.plannedMeal(
          message: 'open must stand alone',
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('Restrictions unavailable: open must stand alone'),
      findsOneWidget,
    );
    expect(tester.takeException(), isNull);
  });

  testWidgets('the recipe library shows an honest empty state', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/recipes'));
    await tester.pumpAndSettle();
    expect(title('Recipes'), findsOneWidget);
    expect(find.text('No recipes yet.'), findsOneWidget);
    expect(find.text('New recipe'), findsOneWidget);
  });

  testWidgets('the recipe library shows progress while loading', (
    tester,
  ) async {
    usePixel5(tester);
    final pending = Completer<List<RecipeSummaryDto>>();
    await tester.pumpWidget(
      harness(initial: '/recipes', recipes: () => pending.future),
    );
    await tester.pump();
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    expect(find.byType(ListTile), findsNothing);
    pending.complete(const [okSummary]);
    await tester.pumpAndSettle();
    expect(find.widgetWithText(ListTile, 'Pancakes'), findsOneWidget);
  });

  testWidgets('the recipe library lists summaries and opens the detail', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(initial: '/recipes', recipes: () => const [okSummary]),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(ListTile, 'Pancakes'));
    await tester.pumpAndSettle();
    expect(title('Pancakes'), findsOneWidget);
    await tester.tap(find.byType(BackButton));
    await tester.pumpAndSettle();
    expect(title('Recipes'), findsOneWidget);
  });

  testWidgets('the recipe library renders a load failure and no list', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes',
        recipes: () => throw const KimattaError.recipe(message: 'bad row'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Recipes unavailable: bad row'), findsOneWidget);
    expect(find.byType(ListTile), findsNothing);
  });

  // --- MVP-009: warnings and the hard filter on the library -----------------------------

  testWidgets('the library shows a conflict subtitle per recipe', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes',
        recipes: () => const [okSummary, conflictSummary],
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('May conflict: Dairy'), findsOneWidget);
    expect(
      find.descendant(
        of: find.widgetWithText(ListTile, 'Butter toast'),
        matching: find.text('May conflict: Dairy'),
      ),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: find.widgetWithText(ListTile, 'Pancakes'),
        matching: find.textContaining('May conflict'),
      ),
      findsNothing,
    );
  });

  testWidgets(
    'Hide known conflicts hides only conflicting recipes and says how many',
    (tester) async {
      usePixel5(tester);
      await tester.pumpWidget(
        harness(
          initial: '/recipes',
          recipes: () => const [okSummary, conflictSummary],
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(FilterChip), findsOneWidget);
      expect(find.text(hiddenCountCopy(1)), findsNothing);
      await tester.tap(find.widgetWithText(FilterChip, hideConflictsLabel));
      await tester.pumpAndSettle();
      expect(find.widgetWithText(ListTile, 'Butter toast'), findsNothing);
      expect(find.widgetWithText(ListTile, 'Pancakes'), findsOneWidget);
      expect(find.text(hiddenCountCopy(1)), findsOneWidget);
      await tester.tap(find.widgetWithText(FilterChip, hideConflictsLabel));
      await tester.pumpAndSettle();
      expect(find.widgetWithText(ListTile, 'Butter toast'), findsOneWidget);
      expect(find.text(hiddenCountCopy(1)), findsNothing);
    },
  );

  // AC-4: an engaged filter over a full list is the state that reads most like "checked and
  // cleared", so it carries the disclaimer rather than no line at all.
  testWidgets('hiding nothing says nothing was hidden', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(initial: '/recipes', recipes: () => const [checkedCleanSummary]),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilterChip, hideConflictsLabel));
    await tester.pumpAndSettle();
    expect(find.widgetWithText(ListTile, 'Rice'), findsOneWidget);
    expect(find.textContaining('hidden for known conflicts'), findsNothing);
    expect(find.text(nothingHiddenCopy), findsOneWidget);
  });

  // PRD §10 / invariant 19: the line above must not outlive the check that justifies it. The
  // filter is engaged while a checked recipe is listed, then the library re-lists with nothing
  // checked — `_hideConflicts` survives (the State is not disposed), `anyChecked` does not.
  testWidgets('the hidden-count line goes with the last checked recipe', (
    tester,
  ) async {
    usePixel5(tester);
    var checked = true;
    await tester.pumpWidget(
      harness(
        initial: '/recipes',
        recipes: () =>
            checked ? const [checkedCleanSummary] : const [okSummary],
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilterChip, hideConflictsLabel));
    await tester.pumpAndSettle();
    expect(find.text(nothingHiddenCopy), findsOneWidget);
    checked = false;
    ProviderScope.containerOf(tester.element(find.byType(FilterChip)))
        .invalidate(recipeLibraryProvider);
    await tester.pumpAndSettle();
    expect(find.byType(FilterChip), findsNothing);
    expect(find.text(nothingHiddenCopy), findsNothing);
    expect(find.textContaining('hidden for known conflicts'), findsNothing);
  });

  // PRD §10 / invariant 19: with nothing checked, no control may imply a check exists.
  testWidgets('no restrictions set shows no filter chip', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(initial: '/recipes', recipes: () => const [okSummary]),
    );
    await tester.pumpAndSettle();
    expect(find.byType(FilterChip), findsNothing);
    expect(find.textContaining('May conflict'), findsNothing);
  });

  testWidgets('the archived list shows the conflict subtitle', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes/archived',
        archived: () => const [conflictSummary],
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('May conflict: Dairy'), findsOneWidget);
    expect(find.byType(FilterChip), findsNothing);
  });

  testWidgets('the library with the chip survives text scale 2.0', (
    tester,
  ) async {
    usePixel5(tester);
    tester.platformDispatcher.textScaleFactorTestValue = 2.0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes',
        recipes: () => const [okSummary, conflictSummary],
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(find.byType(FilterChip), findsOneWidget);
  });

  testWidgets('the library with the chip meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(
        harness(
          initial: '/recipes',
          recipes: () => const [okSummary, conflictSummary],
        ),
      );
      await tester.pumpAndSettle();
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
    }
  });

  // --- MVP-009: the detail's warnings and uncertainty copy -----------------------------

  testWidgets('the detail lists each conflict with its line and term', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(initial: '/recipes/r-1', recipe: (_) => okRecipeWithConflicts),
    );
    await tester.pumpAndSettle();
    expect(find.text(warningsHeading), findsOneWidget);
    for (final c in okRecipeWithConflicts.assessment!.conflicts) {
      expect(find.text(describeConflict(c)), findsOneWidget);
    }
    expect(find.text('Gluten — "Flour" matched "flour"'), findsOneWidget);
    expect(find.text(noRestrictionsCopy), findsNothing);
    expect(find.text(noKnownConflictCopy), findsNothing);
  });

  testWidgets('no restrictions set reads as unchecked, not clear', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/recipes/r-1'));
    await tester.pumpAndSettle();
    expect(find.text(noRestrictionsCopy), findsOneWidget);
    expect(find.text(noKnownConflictCopy), findsNothing);
    expect(find.text(warningsHeading), findsNothing);
  });

  testWidgets('a checked recipe with nothing found is not called safe', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        recipe: (_) => const RecipeDto(
          id: 'r-1',
          householdId: 'h-1',
          title: 'Rice',
          instructions: '',
          lines: [],
          provenance: RecipeProvenanceDto(kind: 'authored'),
          assessment: checkedCleanAssessment,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text(noKnownConflictCopy), findsOneWidget);
    expect(find.text(noRestrictionsCopy), findsNothing);
  });

  testWidgets('wording-only restrictions are named', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(initial: '/recipes/r-1', recipe: (_) => okRecipeWithConflicts),
    );
    await tester.pumpAndSettle();
    expect(find.text(wordingOnlyCopy(const ['something'])), findsOneWidget);
  });

  // Adversarial: an optional line is still a line the household would cook with.
  testWidgets('a conflict on an optional line is still shown', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(initial: '/recipes/r-1', recipe: (_) => okRecipeWithConflicts),
    );
    await tester.pumpAndSettle();
    expect(okRecipeWithConflicts.lines[1].optional, isTrue);
    expect(
      find.text('something — "something" matched "something"'),
      findsOneWidget,
    );
  });

  testWidgets('the detail with two conflicts survives text scale 2.0', (
    tester,
  ) async {
    usePixel5(tester);
    tester.platformDispatcher.textScaleFactorTestValue = 2.0;
    await tester.pumpWidget(
      harness(initial: '/recipes/r-1', recipe: (_) => okRecipeWithConflicts),
    );
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  });

  testWidgets('the archived list shows an honest empty state', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/recipes/archived'));
    await tester.pumpAndSettle();
    expect(title('Archived recipes'), findsOneWidget);
    expect(find.text('Nothing archived.'), findsOneWidget);
  });

  testWidgets('the archived list renders a load failure', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes/archived',
        archived: () => throw const KimattaError.storage(message: 'locked'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Recipes unavailable: locked'), findsOneWidget);
    expect(find.byType(ListTile), findsNothing);
  });

  testWidgets('the archived list restores a recipe', (tester) async {
    usePixel5(tester);
    final shelf = [okSummary];
    final restored = <String>[];
    await tester.pumpWidget(
      harness(
        initial: '/recipes/archived',
        archived: () => List.of(shelf),
        restoreRecipe: (household, id) async {
          restored.add('$household/$id');
          shelf.removeWhere((s) => s.id == id);
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(TextButton, 'Restore'));
    await tester.pumpAndSettle();
    expect(restored, ['h-1/r-1']);
    expect(find.widgetWithText(ListTile, 'Pancakes'), findsNothing);
    expect(find.text('Nothing archived.'), findsOneWidget);
  });

  testWidgets('the recipe detail shows lines verbatim and the instructions', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/recipes/r-1'));
    await tester.pumpAndSettle();
    expect(title('Pancakes'), findsOneWidget);
    expect(find.text('Serves 4'), findsOneWidget);
    expect(find.text('  1/2 cup Flour, sifted '), findsOneWidget);
    expect(find.text('a splash of something'), findsOneWidget);
    expect(find.text('Mix. Fry.'), findsOneWidget);
    expect(find.byTooltip('Edit'), findsOneWidget);
    expect(find.widgetWithText(OutlinedButton, 'Delete'), findsOneWidget);
  });

  // AC-3: "delete" is archive, behind a confirmation that says what archiving means.
  testWidgets('Delete asks for confirmation and archives on confirm', (
    tester,
  ) async {
    usePixel5(tester);
    final archived = <String>[];
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        recipes: () => archived.isEmpty ? const [okSummary] : const [],
        archiveRecipe: (household, id) async {
          archived.add('$household/$id');
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(OutlinedButton, 'Delete'));
    await tester.pumpAndSettle();
    expect(find.text('Archive this recipe?'), findsOneWidget);
    expect(find.textContaining('can be restored'), findsOneWidget);
    await tester.tap(find.widgetWithText(TextButton, 'Cancel'));
    await tester.pumpAndSettle();
    expect(archived, isEmpty);
    expect(find.text('Archive this recipe?'), findsNothing);
    expect(title('Pancakes'), findsOneWidget);

    await tester.tap(find.widgetWithText(OutlinedButton, 'Delete'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Archive'));
    await tester.pumpAndSettle();
    expect(archived, ['h-1/r-1']);
    expect(title('Recipes'), findsOneWidget);
    expect(find.text('No recipes yet.'), findsOneWidget);
  });

  testWidgets('an archived recipe shows the archived marker and Restore', (
    tester,
  ) async {
    usePixel5(tester);
    final restored = <String>[];
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        recipe: (_) => const RecipeDto(
          id: 'r-1',
          householdId: 'h-1',
          title: 'Pancakes',
          instructions: '',
          lines: [],
          provenance: RecipeProvenanceDto(kind: 'authored'),
          archivedAt: '2026-08-29',
        ),
        restoreRecipe: (household, id) async {
          restored.add('$household/$id');
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Archived on 2026-08-29'), findsOneWidget);
    expect(find.widgetWithText(OutlinedButton, 'Delete'), findsNothing);
    await tester.tap(find.widgetWithText(FilledButton, 'Restore'));
    await tester.pumpAndSettle();
    expect(restored, ['h-1/r-1']);
    expect(title('Recipes'), findsOneWidget);
  });

  testWidgets('a failed archive reports the reason in a snackbar', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        archiveRecipe: (_, _) async =>
            throw const KimattaError.storage(message: 'locked'),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(OutlinedButton, 'Delete'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Archive'));
    await tester.pumpAndSettle();
    expect(find.text('Recipes unavailable: locked'), findsOneWidget);
    expect(title('Pancakes'), findsOneWidget);
    expect(
      tester
          .widget<OutlinedButton>(find.widgetWithText(OutlinedButton, 'Delete'))
          .onPressed,
      isNotNull,
    );
  });

  testWidgets('the detail renders a load failure', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        recipe: (_) => throw const KimattaError.storage(message: 'locked'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Recipes unavailable: locked'), findsOneWidget);
    expect(find.byTooltip('Edit'), findsNothing);
  });

  testWidgets('an unknown recipe id is reported, not crashed on', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/recipes/ghost'));
    await tester.pumpAndSettle();
    expect(find.text('Recipe not found.'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  Finder field(String label, [int index = 0]) =>
      find.widgetWithText(TextField, label).at(index);

  // The form is a scrolled `Column`, so off-screen controls exist but cannot be tapped
  // until scrolled into view.
  Future<void> tapVisible(WidgetTester tester, Finder finder) async {
    // A focused text field scrolls itself back on screen after layout, undoing the
    // `ensureVisible` below; drop focus first (a real user's keyboard dismissal does the same).
    FocusManager.instance.primaryFocus?.unfocus();
    await tester.pump();
    await tester.ensureVisible(finder);
    await tester.pumpAndSettle();
    await tester.tap(finder);
  }

  Future<void> pickUnit(WidgetTester tester, int row, String label) async {
    await tapVisible(
      tester,
      find.byType(DropdownButtonFormField<String>).at(row),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text(label).last);
    await tester.pumpAndSettle();
  }

  // AC-1/AC-2: the exact DTO, with every string sent verbatim — no `.trim()` on the form,
  // because silent normalisation is this card's stop condition. Rust validates.
  testWidgets('a new recipe with two ingredient rows is saved as entered', (
    tester,
  ) async {
    useTallView(tester);
    RecipeDto? sent;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (r) async {
          sent = r;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), '  Pancakes ');
    await tester.enterText(field('Servings'), '4');
    await tester.enterText(field('Instructions'), 'Mix. Fry.');
    await tester.enterText(field('As written'), '  1/2 cup Flour, sifted ');
    await tester.enterText(field('Name'), 'Flour');
    await tester.enterText(field('Amount'), '1/2');
    await pickUnit(tester, 0, 'cup');
    await tester.enterText(field('Preparation'), 'sifted');
    await tapVisible(
      tester,
      find.widgetWithText(OutlinedButton, 'Add ingredient'),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('As written', 1), "2-3 handfuls nana's mix");
    await tester.enterText(field('Name', 1), "nana's mix");
    await tester.enterText(field('Amount', 1), '2-3');
    await pickUnit(tester, 1, 'Other…');
    await tester.enterText(field('Unit name'), 'handful');
    await tapVisible(
      tester,
      find.widgetWithText(SwitchListTile, 'Optional').at(1),
    );
    await tester.pump();
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    // Field by field first, so a mismatch names the field (the DTO has no `toString`).
    expect(sent?.title, '  Pancakes ');
    expect(sent?.servings, 4);
    expect(sent?.instructions, 'Mix. Fry.');
    expect(sent?.lines.length, 2);
    expect(sent?.lines[0].originalText, '  1/2 cup Flour, sifted ');
    expect(
      sent?.lines[0].quantity,
      const QuantityDto.exact(numer: 1, denom: 2),
    );
    expect(sent?.lines[0].unit, const UnitDto.known(unit: 'cup'));
    expect(sent?.lines[0].preparation, 'sifted');
    expect(sent?.lines[1].originalText, "2-3 handfuls nana's mix");
    expect(sent?.lines[1].unit, const UnitDto.other(text: 'handful'));
    expect(sent?.lines[1].optional, isTrue);
    expect(sent?.lines[1].preparation, isNull);
    // Not a whole-DTO `==`: the generated `RecipeDto.==` compares `lines` by List identity,
    // so the list goes through the deep matcher and the scalars are asserted above/below.
    expect(sent?.id, '');
    expect(sent?.householdId, 'h-1');
    expect(sent?.provenance, const RecipeProvenanceDto(kind: 'authored'));
    expect(sent?.archivedAt, isNull);
    expect(sent?.lines, const [
      IngredientLineDto(
        originalText: '  1/2 cup Flour, sifted ',
        name: 'Flour',
        quantity: QuantityDto.exact(numer: 1, denom: 2),
        unit: UnitDto.known(unit: 'cup'),
        preparation: 'sifted',
        optional: false,
      ),
      IngredientLineDto(
        originalText: "2-3 handfuls nana's mix",
        name: "nana's mix",
        quantity: QuantityDto.range(
          minNumer: 2,
          minDenom: 1,
          maxNumer: 3,
          maxDenom: 1,
        ),
        unit: UnitDto.other(text: 'handful'),
        optional: true,
      ),
    ]);
    expect(title('Recipes'), findsOneWidget);
  });

  // --- MVP-011 step 10(1): the once-per-launch starter install -----------------------

  testWidgets(
    'starter content installs once per launch for an already-onboarded household',
    (tester) async {
      // Adversarial: this is the regression an onboarding-wired install would have
      // shipped. `okHousehold.onboarded` is true, so Welcome is unreachable and a
      // one-shot tied to it would never run again on this device.
      usePixel5(tester);
      final calls = <String>[];
      await tester.pumpWidget(
        harness(
          starterInstall: (id) async {
            calls.add(id);
            return okStarterReport;
          },
        ),
      );
      await tester.pumpAndSettle();
      expect(calls, ['h-1']);
    },
  );

  testWidgets('renaming the household does not re-run the starter install', (
    tester,
  ) async {
    // Pins the `_installStarted` latch: `HouseholdNotifier.rename` publishes `AsyncData`,
    // so the listener fires again on every Save name.
    usePixel5(tester);
    var calls = 0;
    await tester.pumpWidget(
      harness(
        initial: '/settings/household',
        starterInstall: (_) async {
          calls++;
          return okStarterReport;
        },
        rename: (id, name) async => HouseholdDto(
          id: id,
          name: name,
          members: okHousehold.members,
          onboarded: true,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(calls, 1);

    await tester.enterText(find.byType(TextField), 'Casa');
    await tester.tap(find.widgetWithText(FilledButton, 'Save name'));
    await tester.pumpAndSettle();
    expect(calls, 1);
  });

  testWidgets('the install is not called before the household resolves', (
    tester,
  ) async {
    usePixel5(tester);
    var calls = 0;
    final pending = Completer<HouseholdDto>();
    await tester.pumpWidget(
      harness(
        household: () => pending.future,
        starterInstall: (_) async {
          calls++;
          return okStarterReport;
        },
      ),
    );
    await tester.pump();
    expect(calls, 0);

    pending.complete(okHousehold);
    await tester.pumpAndSettle();
    expect(calls, 1);
  });

  testWidgets('a failed starter install does not block the app', (
    tester,
  ) async {
    // Pins the `scaffoldMessengerKey` path: `_AppState`'s context sits above
    // `MaterialApp.router`, so without the key this throws rather than reporting.
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        starterInstall: (_) async =>
            throw const KimattaError.storage(message: 'disk full'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Starter recipes unavailable: disk full'), findsOneWidget);
    // Navigation is untouched: the app is still usable.
    expect(title('Plan'), findsOneWidget);
  });

  testWidgets('editing an existing recipe preserves its prep time', (
    tester,
  ) async {
    // Risk 11: FRB emits a nullable DTO field as an *optional* named parameter, so the form
    // compiles unchanged while sending `prepMinutes: null` and wiping the stored value.
    // Nothing else catches this.
    useTallView(tester);
    RecipeDto? sent;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1/edit',
        recipe: (_) => okRecipe,
        saveRecipe: (dto) async {
          sent = dto;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(sent!.prepMinutes, 20);
  });

  testWidgets('the form saves an entered prep time', (tester) async {
    useTallView(tester);
    RecipeDto? sent;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (dto) async {
          sent = dto;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Soup');
    await tester.enterText(field('Prep time (minutes)'), '35');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(sent!.prepMinutes, 35);
  });

  testWidgets('a zero prep time blocks save and names the field', (
    tester,
  ) async {
    useTallView(tester);
    var saves = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async {
          saves++;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Soup');
    await tester.enterText(field('Prep time (minutes)'), '0');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(
      find.textContaining('Prep time must be a whole number of minutes'),
      findsOneWidget,
    );
    expect(saves, 0);
  });

  testWidgets('the detail screen shows prep time when there is one', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(harness(initial: '/recipes/r-1'));
    await tester.pumpAndSettle();
    expect(find.text('Prep 20 min'), findsOneWidget);
  });

  testWidgets('the detail screen omits prep time when there is none', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(initial: '/recipes/r-1', recipe: (_) => okRecipeNoPrep),
    );
    await tester.pumpAndSettle();
    expect(find.textContaining('Prep'), findsNothing);
  });

  testWidgets('a blank title blocks save with an inline error', (tester) async {
    useTallView(tester);
    var saves = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async {
          saves++;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), '   ');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(find.text('Give the recipe a title.'), findsOneWidget);
    expect(saves, 0);
  });

  testWidgets('an unreadable quantity blocks save and names the row', (
    tester,
  ) async {
    useTallView(tester);
    var saves = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async {
          saves++;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('As written'), 'abc bread');
    await tester.enterText(field('Name'), 'bread');
    await tester.enterText(field('Amount'), 'abc');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(
      find.textContaining('Ingredient 1: cannot read "abc"'),
      findsOneWidget,
    );
    expect(saves, 0);
  });

  // The `u32` the bridge carries is the accepted range. Before the bound, `int.parse` threw
  // out of `_validate` — which `_save` called outside its `try` — so the tap did nothing at
  // all: no inline error, no snackbar, no write.
  testWidgets('an over-long amount blocks save and names the row', (
    tester,
  ) async {
    useTallView(tester);
    var saves = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async {
          saves++;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('As written'), 'lots of bread');
    await tester.enterText(field('Name'), 'bread');
    await tester.enterText(field('Amount'), '99999999999999999999');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(
      find.textContaining('Ingredient 1: cannot read "99999999999999999999"'),
      findsOneWidget,
    );
    expect(saves, 0);
  });

  // Above `u32` the FRB encoder masks rather than range-checks, so `4294967297` would be
  // stored as `1` and read back as `Serves 1`.
  testWidgets('a servings count above the bridge range blocks save', (
    tester,
  ) async {
    useTallView(tester);
    var saves = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async {
          saves++;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('Servings'), '4294967297');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(
      find.text('Servings must be a whole number from 1 to 4294967295.'),
      findsOneWidget,
    );
    expect(saves, 0);
  });

  // The reject path's only signal is an inline error, and by the time the user reaches Save
  // they have scrolled past the offending field — so on phone geometry the error is painted
  // above the fold, clipped, and the tap looks inert. These three run at [usePixel5], not
  // [useTallView]: the tall viewport fits the whole form, which is the one geometry in which
  // the error is always visible and the defect therefore invisible.
  //
  // The bounds come from the scroll viewport, never from the view height: every recipe route
  // is inside the shell's `StatefulShellRoute`, so the bottom 80 logical px are its
  // `NavigationBar` and an error painted there is behind it, not on screen.
  Rect scrollViewport(WidgetTester tester) =>
      tester.getRect(find.byType(SingleChildScrollView));

  void expectVisible(WidgetTester tester, Finder error) {
    final viewport = scrollViewport(tester);
    final rect = tester.getRect(error);
    expect(
      rect.top,
      greaterThanOrEqualTo(viewport.top),
      reason: 'error is clipped above the scroll viewport',
    );
    expect(
      rect.bottom,
      lessThanOrEqualTo(viewport.bottom),
      reason: 'error is below the fold or behind the navigation bar',
    );
  }

  testWidgets('a blank title scrolls the error into view on a phone screen', (
    tester,
  ) async {
    usePixel5(tester);
    var saves = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async {
          saves++;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expectVisible(tester, find.text('Give the recipe a title.'));
    expect(saves, 0);
  });

  // The row branch, and the case the two-row form cannot show: `tapVisible` scrolls Save —
  // the last widget — into view, which necessarily reveals the row just above it. Only an
  // error near the top of a long form is clipped, so the offending row is the *first* of
  // three.
  testWidgets(
    'the first of three ingredient rows scrolls its error into view',
    (tester) async {
      usePixel5(tester);
      var saves = 0;
      await tester.pumpWidget(
        harness(
          initial: '/recipes/new',
          saveRecipe: (_) async {
            saves++;
            return okRecipe;
          },
        ),
      );
      await tester.pumpAndSettle();
      await tester.enterText(field('Title'), 'Toast');
      await tester.enterText(field('As written'), 'abc bread');
      await tester.enterText(field('Name'), 'bread');
      await tester.enterText(field('Amount'), 'abc');
      for (var i = 0; i < 2; i++) {
        await tapVisible(
          tester,
          find.widgetWithText(OutlinedButton, 'Add ingredient'),
        );
        await tester.pumpAndSettle();
      }
      await tapVisible(
        tester,
        find.widgetWithText(FilledButton, 'Save recipe'),
      );
      await tester.pumpAndSettle();
      expectVisible(
        tester,
        find.textContaining('Ingredient 1: cannot read "abc"'),
      );
      expect(saves, 0);
    },
  );

  // The anchor has to be the error text itself, not the row `Card`: at this scale the card is
  // taller than the viewport, so aligning its *leading* edge leaves the error — the card's
  // last child — below the fold, reproducing the defect on the accessibility path where the
  // form is hardest to read.
  testWidgets('a row error is scrolled into view at text scale 2.0', (
    tester,
  ) async {
    usePixel5(tester);
    tester.platformDispatcher.textScaleFactorTestValue = 2.0;
    await tester.pumpWidget(harness(initial: '/recipes/new'));
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('As written'), 'abc bread');
    await tester.enterText(field('Name'), 'bread');
    await tester.enterText(field('Amount'), 'abc');
    for (var i = 0; i < 2; i++) {
      await tapVisible(
        tester,
        find.widgetWithText(OutlinedButton, 'Add ingredient'),
      );
      await tester.pumpAndSettle();
    }
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expectVisible(
      tester,
      find.textContaining('Ingredient 1: cannot read "abc"'),
    );
  });

  // The Servings arm of `_firstErrorKey`, which the three above never reach: they leave Servings
  // empty, so the title and row arms answer first.
  //
  // Expected-to-pass — the fix is already in — but not vacuous, and the two extra rows are what
  // make it so. On the default one-row form a servings error renders inside the fold whether or
  // not the scroll runs, so the obvious version of this test passes with `key: _servingsKey`
  // deleted. Three rows make the form long enough that `tapVisible`'s scroll to Save leaves
  // Servings well above the viewport, so the assertion holds only if the arm returns a key whose
  // `currentContext` resolves: drop the key from the field, or reorder the arm behind the row
  // scan, and this goes red while the other 152 stay green.
  //
  // The rows are left empty on purpose. An untouched row is skipped by `_validate`, so they add
  // height without adding a competing row error — the rejection is servings-only.
  testWidgets('a rejected servings count scrolls its error into view', (
    tester,
  ) async {
    usePixel5(tester);
    var saves = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async {
          saves++;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('Servings'), '0');
    for (var i = 0; i < 2; i++) {
      await tapVisible(
        tester,
        find.widgetWithText(OutlinedButton, 'Add ingredient'),
      );
      await tester.pumpAndSettle();
    }
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expectVisible(
      tester,
      find.text('Servings must be a whole number from 1 to 4294967295.'),
    );
    expect(saves, 0);
  });

  // `preparation` was the one field whose emptiness test (`.isEmpty`) disagreed with the
  // domain's (`trim().is_empty()`), so a lone space passed the form and failed the whole save
  // in Rust — one snackbar, no row named, nothing highlighted.
  testWidgets('a whitespace-only preparation blocks save and names the row', (
    tester,
  ) async {
    useTallView(tester);
    var saves = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async {
          saves++;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('As written'), 'sifted flour');
    await tester.enterText(field('Name'), 'flour');
    await tester.enterText(field('Preparation'), ' ');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(
      find.text('Ingredient 1: write a preparation or leave it blank.'),
      findsOneWidget,
    );
    expect(saves, 0);
  });

  // Expected-to-pass: the guard above rejects *only* whitespace-only text. Padding around
  // real words is content, and the domain keeps it (`preparation() == Some(' sifted ')`), so
  // it must still reach Rust with both spaces on.
  testWidgets('a padded preparation is still saved verbatim', (tester) async {
    useTallView(tester);
    RecipeDto? sent;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (r) async {
          sent = r;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('As written'), 'sifted flour');
    await tester.enterText(field('Name'), 'flour');
    await tester.enterText(field('Preparation'), ' sifted ');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(sent?.lines.single.preparation, ' sifted ');
  });

  testWidgets('Remove ingredient drops the row and saves without it', (
    tester,
  ) async {
    useTallView(tester);
    RecipeDto? sent;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (r) async {
          sent = r;
          return okRecipe;
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('As written'), 'bread');
    await tester.enterText(field('Name'), 'bread');
    await tapVisible(
      tester,
      find.widgetWithText(OutlinedButton, 'Add ingredient'),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('As written', 1), 'butter');
    await tester.enterText(field('Name', 1), 'butter');
    await tapVisible(tester, find.byTooltip('Remove ingredient 2'));
    await tester.pumpAndSettle();
    expect(find.widgetWithText(TextField, 'As written'), findsOneWidget);
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(sent?.lines.length, 1);
    expect(sent?.lines.first.name, 'bread');
  });

  // Tapping the ✕ does not unfocus the row's own field, and disposing inside `setState` tore
  // the controller down before the frame that unmounts its `TextField`. Deliberately a bare
  // `tap`, not `tapVisible`: that helper unfocuses first, which is the condition under test.
  testWidgets('Remove ingredient works while the row holds focus', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(harness(initial: '/recipes/new'));
    await tester.pumpAndSettle();
    await tapVisible(
      tester,
      find.widgetWithText(OutlinedButton, 'Add ingredient'),
    );
    await tester.pumpAndSettle();
    await tester.tap(field('Name', 1));
    await tester.pumpAndSettle();
    await tester.enterText(field('Name', 1), 'butter');
    await tester.pumpAndSettle();
    expect(FocusManager.instance.primaryFocus, isNotNull);
    await tester.tap(find.byTooltip('Remove ingredient 2'));
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(find.widgetWithText(TextField, 'As written'), findsOneWidget);
  });

  // A removal renumbers every row below it, so an error written before the removal must not
  // keep the number it was written with. The two removal tests above both delete from a form
  // that has never been rejected, so neither of them has an error to renumber.
  testWidgets('Remove ingredient renumbers the errors of the rows below it', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(harness(initial: '/recipes/new'));
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tester.enterText(field('Name'), 'bread');
    for (var row = 1; row < 3; row++) {
      await tapVisible(
        tester,
        find.widgetWithText(OutlinedButton, 'Add ingredient'),
      );
      await tester.pumpAndSettle();
      await tester.enterText(field('Name', row), 'filling $row');
    }
    // Every row carries a name and no "As written", so `_validate` rejects all three rather
    // than skipping any of them as untouched.
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    for (final n in [1, 2, 3]) {
      expect(
        find.text('Ingredient $n: write it as you would say it.'),
        findsOneWidget,
      );
    }
    await tapVisible(tester, find.byTooltip('Remove ingredient 2'));
    await tester.pumpAndSettle();
    // Row 1 sits above the removal and keeps both its number and its message — clearing every
    // surviving row's error would drop a rejection signal that is still true, which is the
    // silent-rejection shape this card already fixed once. The row that was 3 is now 2.
    expect(
      find.text('Ingredient 1: write it as you would say it.'),
      findsOneWidget,
    );
    expect(
      find.text('Ingredient 2: write it as you would say it.'),
      findsOneWidget,
    );
    expect(find.textContaining('Ingredient 3:'), findsNothing);
  });

  // A re-list that fails after the write committed must not be reported as a save failure:
  // `_save`'s catch would say "Recipes unavailable: …" for a recipe that *was* stored, and
  // the natural retry sends `id: ''` again, minting a second copy.
  group('a re-list failure after a write', () {
    ProviderContainer containerWith(_SeamedRecipeLibraryNotifier notifier) {
      final container = ProviderContainer(
        overrides: [
          // `HouseholdNotifier.build` awaits the health report before bootstrapping, so both
          // are overridden or the notifier reaches the real bridge.
          healthReportProvider.overrideWith((_) => okReport),
          householdProvider.overrideWith(
            () => _FakeHouseholdNotifier(() => okHousehold, null, null),
          ),
          // The real `build` watches the restriction set (MVP-009), so this is overridden
          // too or the seamed notifier reaches the native `loadRestrictions`.
          restrictionsProvider.overrideWith(
            () => _FakeRestrictionsNotifier(null, null),
          ),
          recipeLibraryProvider.overrideWith(() => notifier),
        ],
      );
      addTearDown(container.dispose);
      return container;
    }

    Future<_SeamedRecipeLibraryNotifier> failingOnRepublish() async {
      final notifier = _SeamedRecipeLibraryNotifier(
        (call) async => call == 1
            ? const [okSummary]
            : throw const KimattaError.recipe(message: 'bad row'),
      );
      final container = containerWith(notifier);
      await container.read(recipeLibraryProvider.future);
      return notifier;
    }

    test('does not escape save as a write failure', () async {
      final notifier = await failingOnRepublish();
      final stored = await notifier.save(okRecipe);
      expect(stored.id, okRecipe.id);
      expect(notifier.fetches, 2);
    });

    test('degrades the list state instead of leaving it stale', () async {
      final notifier = await failingOnRepublish();
      await notifier.save(okRecipe);
      expect(notifier.state, isA<AsyncError<List<RecipeSummaryDto>>>());
      final error = (notifier.state as AsyncError).error;
      expect(error, isA<KimattaError_Recipe>());
      expect((error as KimattaError_Recipe).message, 'bad row');
    });
  });

  // The widget harness overrides `archive` and `restore` *wholesale* and re-lists with
  // `ref.invalidateSelf()`, so no test reached the real bodies — both could be gutted with the
  // suite still green. These run the real notifier with only its bridge seams replaced.
  //
  // Expected-to-pass: the production bodies are already correct. What the group buys is that
  // dropping `_republish` or `ref.invalidate(archivedRecipesProvider)` now fails. `fetches`
  // read straight after the await is the discriminator — `_republish` fetches eagerly, whereas
  // `invalidateSelf` would leave it at 1 until something read the provider.
  group('archive and restore refresh both lists', () {
    late int archivedBuilds;

    ProviderContainer containerWith(_SeamedRecipeLibraryNotifier notifier) {
      archivedBuilds = 0;
      final container = ProviderContainer(
        overrides: [
          healthReportProvider.overrideWith((_) => okReport),
          householdProvider.overrideWith(
            () => _FakeHouseholdNotifier(() => okHousehold, null, null),
          ),
          restrictionsProvider.overrideWith(
            () => _FakeRestrictionsNotifier(null, null),
          ),
          recipeLibraryProvider.overrideWith(() => notifier),
          archivedRecipesProvider.overrideWith(
            () => _FakeArchivedRecipesNotifier(() {
              archivedBuilds++;
              return const <RecipeSummaryDto>[];
            }),
          ),
        ],
      );
      addTearDown(container.dispose);
      return container;
    }

    // Both providers are read *before* the write: invalidating one that was never built
    // cannot show a second build, so the counter would not discriminate.
    Future<(_SeamedRecipeLibraryNotifier, ProviderContainer)> ready([
      Future<List<RecipeSummaryDto>> Function(int call)? onFetch,
    ]) async {
      final notifier = _SeamedRecipeLibraryNotifier(
        onFetch ?? (_) async => const [okSummary],
      );
      final container = containerWith(notifier);
      await container.read(recipeLibraryProvider.future);
      await container.read(archivedRecipesProvider.future);
      expect(archivedBuilds, 1);
      return (notifier, container);
    }

    test('archive re-lists the library and drops the archived list', () async {
      final (notifier, container) = await ready();
      final stored = await notifier.archive('h-1', 'r-1');
      expect(stored.id, okRecipe.id);
      expect(notifier.archives, 1);
      expect(notifier.archivedOn, todayCivilDate());
      expect(notifier.fetches, 2);
      expect(notifier.state, isA<AsyncData<List<RecipeSummaryDto>>>());
      expect(notifier.state.value, const [okSummary]);
      await container.read(archivedRecipesProvider.future);
      expect(archivedBuilds, 2);
    });

    test('restore re-lists the library and drops the archived list', () async {
      final (notifier, container) = await ready();
      final stored = await notifier.restore('h-1', 'r-1');
      expect(stored.id, okRecipe.id);
      expect(notifier.restores, 1);
      expect(notifier.fetches, 2);
      expect(notifier.state, isA<AsyncData<List<RecipeSummaryDto>>>());
      expect(notifier.state.value, const [okSummary]);
      await container.read(archivedRecipesProvider.future);
      expect(archivedBuilds, 2);
    });

    // The same contract `save` has: the archive committed, so a failed re-list is the list's
    // problem and must not escape as though the write failed.
    test('a re-list failure after an archive degrades the list instead of throwing', () async {
      final (notifier, _) = await ready(
        (call) async => call == 1
            ? const [okSummary]
            : throw const KimattaError.recipe(message: 'bad row'),
      );
      await notifier.archive('h-1', 'r-1');
      expect(notifier.archives, 1);
      expect(notifier.state, isA<AsyncError<List<RecipeSummaryDto>>>());
      final error = (notifier.state as AsyncError).error;
      expect(error, isA<KimattaError_Recipe>());
      expect((error as KimattaError_Recipe).message, 'bad row');
    });
  });

  // AC-3, Dart half. The widget-harness fake overrides `build` wholesale, so only the real
  // notifier can show that a restriction save re-runs the listing.
  test('a restriction save re-lists the library', () async {
    final notifier = _SeamedRecipeLibraryNotifier(
      (_) async => const [okSummary],
    );
    final container = ProviderContainer(
      overrides: [
        healthReportProvider.overrideWith((_) => okReport),
        householdProvider.overrideWith(
          () => _FakeHouseholdNotifier(() => okHousehold, null, null),
        ),
        restrictionsProvider.overrideWith(
          () => _FakeRestrictionsNotifier(null, (_, r) async => r),
        ),
        recipeLibraryProvider.overrideWith(() => notifier),
      ],
    );
    addTearDown(container.dispose);
    await container.read(recipeLibraryProvider.future);
    expect(notifier.fetches, 1);
    await container.read(restrictionsProvider.notifier).save('h-1', const [
      RestrictionDto.known(kind: 'dairy'),
    ]);
    await container.pump();
    await container.read(recipeLibraryProvider.future);
    expect(notifier.fetches, 2);
  });

  // AC-3, archived half. Without this the archived notifier's restriction watch is free to be
  // deleted and nothing fails — the library half is pinned above, the archived half was not.
  test('a restriction save re-lists the archived recipes', () async {
    final notifier = _SeamedArchivedRecipesNotifier();
    final container = ProviderContainer(
      overrides: [
        healthReportProvider.overrideWith((_) => okReport),
        householdProvider.overrideWith(
          () => _FakeHouseholdNotifier(() => okHousehold, null, null),
        ),
        restrictionsProvider.overrideWith(
          () => _FakeRestrictionsNotifier(null, (_, r) async => r),
        ),
        archivedRecipesProvider.overrideWith(() => notifier),
      ],
    );
    addTearDown(container.dispose);
    await container.read(archivedRecipesProvider.future);
    expect(notifier.fetches, 1);
    await container.read(restrictionsProvider.notifier).save('h-1', const [
      RestrictionDto.known(kind: 'dairy'),
    ]);
    await container.pump();
    await container.read(archivedRecipesProvider.future);
    expect(notifier.fetches, 2);
  });

  // Both dependencies moving in the same turn is the case that put the restriction watch on
  // the far side of the household await: a `ref` call after that gap can land on an element
  // the household emission has already outdated. Expected-to-pass — a container cannot force
  // the microtask interleaving that trips Riverpod's own assert, so this pins the outcome
  // (AC-3 still fires, no error state) rather than the mechanism.
  test(
    'a household and restriction change in one turn still re-lists',
    () async {
      final notifier = _SeamedRecipeLibraryNotifier(
        (_) async => const [okSummary],
      );
      final container = ProviderContainer(
        overrides: [
          healthReportProvider.overrideWith((_) => okReport),
          householdProvider.overrideWith(
            () => _FakeHouseholdNotifier(() => okHousehold, null, null),
          ),
          restrictionsProvider.overrideWith(
            () => _FakeRestrictionsNotifier(null, (_, r) async => r),
          ),
          recipeLibraryProvider.overrideWith(() => notifier),
        ],
      );
      addTearDown(container.dispose);
      await container.read(recipeLibraryProvider.future);
      expect(notifier.fetches, 1);
      container.invalidate(householdProvider);
      await container.read(restrictionsProvider.notifier).save('h-1', const [
        RestrictionDto.known(kind: 'dairy'),
      ]);
      await container.pump();
      await container.read(recipeLibraryProvider.future);
      expect(notifier.fetches, greaterThan(1));
      expect(
        container.read(recipeLibraryProvider),
        isA<AsyncData<List<RecipeSummaryDto>>>(),
      );
    },
  );

  testWidgets(
    'editing seeds the form from the loaded recipe and saves with the same id',
    (tester) async {
      useTallView(tester);
      RecipeDto? sent;
      await tester.pumpWidget(
        harness(
          initial: '/recipes/r-1/edit',
          saveRecipe: (r) async {
            sent = r;
            return r;
          },
        ),
      );
      await tester.pumpAndSettle();
      expect(
        tester.widget<TextField>(field('Title')).controller?.text,
        'Pancakes',
      );
      expect(
        tester.widget<TextField>(field('As written')).controller?.text,
        '  1/2 cup Flour, sifted ',
      );
      expect(tester.widget<TextField>(field('Amount')).controller?.text, '1/2');
      expect(tester.widget<TextField>(field('Amount', 1)).controller?.text, '');
      await tester.enterText(field('Title'), 'Pancakes v2');
      await tapVisible(
        tester,
        find.widgetWithText(FilledButton, 'Save recipe'),
      );
      await tester.pumpAndSettle();
      expect(sent?.id, 'r-1');
      expect(sent?.householdId, 'h-1');
      expect(sent?.title, 'Pancakes v2');
      expect(sent?.servings, 4);
      expect(sent?.lines, okRecipe.lines);
      expect(sent?.provenance, okRecipe.provenance);
      expect(title('Pancakes'), findsOneWidget);
    },
  );

  testWidgets('a failed save reports the reason and re-enables Save', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes/new',
        saveRecipe: (_) async =>
            throw const KimattaError.recipe(message: 'bad line'),
      ),
    );
    await tester.pumpAndSettle();
    await tester.enterText(field('Title'), 'Toast');
    await tapVisible(tester, find.widgetWithText(FilledButton, 'Save recipe'));
    await tester.pumpAndSettle();
    expect(find.text('Recipes unavailable: bad line'), findsOneWidget);
    expect(
      tester
          .widget<FilledButton>(
            find.widgetWithText(FilledButton, 'Save recipe'),
          )
          .onPressed,
      isNotNull,
    );
  });

  testWidgets(
    'the form shows progress and no Save while the unit vocabulary loads',
    (tester) async {
      useTallView(tester);
      final pending = Completer<List<String>>();
      await tester.pumpWidget(
        harness(initial: '/recipes/new', unitKinds: () => pending.future),
      );
      await tester.pump();
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.byType(TextField), findsNothing);
      pending.complete(unitKindTokens);
      await tester.pumpAndSettle();
      expect(find.widgetWithText(FilledButton, 'Save recipe'), findsOneWidget);
    },
  );

  testWidgets('the edit form renders a load failure and no form', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1/edit',
        recipe: (_) => throw const KimattaError.storage(message: 'locked'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Recipes unavailable: locked'), findsOneWidget);
    expect(find.byType(TextField), findsNothing);
  });

  for (final location in const [
    '/recipes',
    '/recipes/new',
    '/recipes/r-1',
    '/recipes/archived',
  ]) {
    testWidgets('$location meets the accessibility guidelines', (tester) async {
      usePixel5(tester);
      for (final brightness in Brightness.values) {
        tester.platformDispatcher.platformBrightnessTestValue = brightness;
        await tester.pumpWidget(
          harness(initial: location, recipes: () => const [okSummary]),
        );
        await tester.pumpAndSettle();
        await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
        await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
        await expectLater(tester, meetsGuideline(textContrastGuideline));
      }
    });
  }

  // AC-4, keyboard: bounded traversal as the navigation-bar test, asserting both of the
  // form's action buttons take focus.
  testWidgets('tab traversal reaches Add ingredient and Save on the form', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(harness(initial: '/recipes/new'));
    await tester.pumpAndSettle();
    final targets = {
      'Add ingredient': find.widgetWithText(OutlinedButton, 'Add ingredient'),
      'Save recipe': find.widgetWithText(FilledButton, 'Save recipe'),
    };
    final reached = <String>{};
    for (var i = 0; i < 40 && reached.length < targets.length; i++) {
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump();
      final context = FocusManager.instance.primaryFocus?.context;
      if (context == null) continue;
      for (final entry in targets.entries) {
        if (find
            .ancestor(of: find.byWidget(context.widget), matching: entry.value)
            .evaluate()
            .isNotEmpty) {
          reached.add(entry.key);
        }
      }
    }
    expect(reached, targets.keys.toSet());
  });

  // --- MVP-014 pantry ------------------------------------------------------

  testWidgets('the pantry screen lists identities and marks one', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <(String, IngredientRefDto, bool)>[];
    await tester.pumpWidget(
      harness(
        initial: '/pantry',
        setMark: (household, ingredient, marked) async {
          sent.add((household, ingredient, marked));
          return pantryEntryMarked(ingredient, marked);
        },
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text(pantryDisclaimer), findsOneWidget);
    expect(find.byType(SwitchListTile), findsNWidgets(2));
    expect(find.text('Your ingredient'), findsOneWidget);

    await tester.tap(find.widgetWithText(SwitchListTile, 'chickpeas'));
    await tester.pumpAndSettle();
    expect(sent, [('h-1', chickpeasRef, true)]);
    expect(
      tester
          .widget<SwitchListTile>(
            find.widgetWithText(SwitchListTile, 'chickpeas'),
          )
          .value,
      isTrue,
    );
  });

  testWidgets('unmarking sends marked false and the row returns to unmarked', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <bool>[];
    await tester.pumpWidget(
      harness(
        initial: '/pantry',
        pantry: () => [pantryEntryMarked(chickpeasRef, true)],
        setMark: (_, ingredient, marked) async {
          sent.add(marked);
          return pantryEntryMarked(ingredient, marked);
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(SwitchListTile, 'chickpeas'));
    await tester.pumpAndSettle();
    expect(sent, [false]);
    expect(
      tester
          .widget<SwitchListTile>(
            find.widgetWithText(SwitchListTile, 'chickpeas'),
          )
          .value,
      isFalse,
    );
  });

  testWidgets('the pantry search filters by name', (tester) async {
    useTallView(tester);
    await tester.pumpWidget(harness(initial: '/pantry'));
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField), 'nana');
    await tester.pumpAndSettle();
    expect(find.byType(SwitchListTile), findsOneWidget);
    expect(find.text("nana's mix"), findsOneWidget);

    await tester.enterText(find.byType(TextField), 'zzz');
    await tester.pumpAndSettle();
    expect(find.byType(SwitchListTile), findsNothing);
    expect(find.text(pantryNoMatchCopy('zzz')), findsOneWidget);
  });

  /// Edge: `ingredient_alias` ships names that are not substrings of their canonical name, so
  /// a name-only filter would return nothing for the word the user actually knows.
  testWidgets('the pantry search finds an identity by its alias', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(harness(initial: '/pantry'));
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextField), 'garbanzo');
    await tester.pumpAndSettle();
    expect(find.byType(SwitchListTile), findsOneWidget);
    expect(find.text('chickpeas'), findsOneWidget);
  });

  /// AC-2: an empty pantry is a screen that still works and says what it does not know,
  /// never a claim that the household is out of anything.
  testWidgets('an empty pantry is honest about what it does not know', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(initial: '/pantry', pantry: () => const <PantryEntryDto>[]),
    );
    await tester.pumpAndSettle();
    expect(title('Pantry'), findsOneWidget);
    expect(find.text(pantryEmptyCopy), findsOneWidget);
    expect(find.byType(SwitchListTile), findsNothing);
    expect(find.text(pantryDisclaimer), findsOneWidget);
  });

  testWidgets('the pantry screen renders a load failure and no list', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/pantry',
        pantry: () =>
            Future<List<PantryEntryDto>>.error(const KimattaError.notOpen()),
      ),
    );
    await tester.pumpAndSettle();
    expect(title('Pantry'), findsOneWidget);
    expect(
      find.text(
        describeFailure(const KimattaError.notOpen(), subject: 'Pantry'),
      ),
      findsOneWidget,
    );
    expect(find.byType(SwitchListTile), findsNothing);
  });

  testWidgets('a failed pantry mark reports the reason in a snackbar and '
      'leaves the row alone', (tester) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/pantry',
        setMark: (_, _, _) async => throw const KimattaError.notOpen(),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(SwitchListTile, 'chickpeas'));
    await tester.pumpAndSettle();
    expect(
      find.descendant(
        of: find.byType(SnackBar),
        matching: find.text(
          describeFailure(const KimattaError.notOpen(), subject: 'Pantry'),
        ),
      ),
      findsOneWidget,
    );
    expect(
      tester
          .widget<SwitchListTile>(
            find.widgetWithText(SwitchListTile, 'chickpeas'),
          )
          .value,
      isFalse,
      reason: 'a failed write must leave the row at its stored value',
    );
  });

  /// Every `okRecipe` line is unresolved, so a resolved line needs its own fixture — the
  /// arm that carries a pantry control cannot be reached otherwise.
  testWidgets('the recipe detail toggles pantry for a resolved line only', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <(String, IngredientRefDto, bool)>[];
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        recipe: (_) => recipeWithResolvedLine,
        setMark: (household, ingredient, marked) async {
          sent.add((household, ingredient, marked));
          return pantryEntryMarked(ingredient, marked);
        },
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byType(Switch), findsOneWidget);

    await tester.tap(find.byType(Switch));
    await tester.pumpAndSettle();
    expect(sent, [('h-1', chickpeasRef, true)]);
  });

  /// Expected-to-pass by construction — the recipe detail had no pantry dependency before
  /// this step — and given teeth: with the pantry in `AsyncError`, the recipe still renders
  /// in full and nothing on screen reports a pantry failure (AC-2).
  testWidgets('a pantry failure does not block the recipe detail', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        recipe: (_) => recipeWithResolvedLine,
        pantry: () =>
            Future<List<PantryEntryDto>>.error(const KimattaError.notOpen()),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('chickpeas'), findsOneWidget);
    expect(find.text('a splash of something'), findsOneWidget);
    expect(find.text('Mix. Fry.'), findsOneWidget);
    expect(
      find.text(
        describeFailure(const KimattaError.notOpen(), subject: 'Pantry'),
      ),
      findsNothing,
    );
    expect(tester.takeException(), isNull);
  });

  /// AC-3's accessible half. The claim is that the accessible *name* states the meaning
  /// before the platform's own toggle state — not that "off" is never announced, which no
  /// label can prevent and which a widget-test semantics tree could never detect.
  testWidgets('the pantry screen meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    // Disposed inline rather than via `addTearDown`: the framework verifies no handle is
    // live at the end of the test body, which runs before tear-downs.
    final handle = tester.ensureSemantics();
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(harness(initial: '/pantry'));
      await tester.pumpAndSettle();
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
      for (final entry in pantryEntries) {
        expect(
          find.bySemanticsLabel(pantryRowLabel(entry.name, entry.marked)),
          findsOneWidget,
          reason: '${entry.name} must carry its meaning as its exact label',
        );
      }
    }
    handle.dispose();
  });

  /// The reachable stale-list path: `installStarterContent` grows the catalog after
  /// `pantryProvider` has already read it, so a recipe line can resolve to an identity the
  /// list has never seen. The write commits either way — the question is whether the UI
  /// learns. The recipe detail is the surface because an empty pantry renders
  /// `pantryEmptyCopy` with no rows to tap.
  testWidgets('a mark for an identity the list has not seen re-reads the list', (
    tester,
  ) async {
    useTallView(tester);
    var reads = 0;
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        recipe: (_) => recipeWithResolvedLine,
        // No launch re-read, so `reads` counts only what this finding is about.
        starterInstall: (_) async => noCatalogStarterReport,
        pantry: () {
          reads++;
          return reads == 1
              ? const <PantryEntryDto>[]
              : [pantryEntryMarked(chickpeasRef, true)];
        },
        setMark: (_, ingredient, marked) async =>
            pantryEntryMarked(ingredient, marked),
      ),
    );
    await tester.pumpAndSettle();
    expect(reads, 1);

    await tester.tap(find.byType(Switch));
    await tester.pumpAndSettle();
    expect(reads, 2, reason: 'a miss must re-read rather than drop the write');
    expect(
      tester.widget<Switch>(find.byType(Switch)).value,
      isTrue,
      reason: 'the committed mark must reach the control the user tapped',
    );
  });

  /// The re-list above can itself fail. The write has already committed, so the failure must
  /// not be published over a usable list: `_lineTile` drops the control entirely when the
  /// pantry has no value, which would cost the user the switch as the result of a *successful*
  /// mark. `pantryEntries[1]` alone — `nana's mix` — leaves chickpeas absent, so the tap
  /// genuinely reaches the miss path.
  testWidgets(
    'a re-list that fails after a mark commits keeps the list usable',
    (tester) async {
      useTallView(tester);
      var reads = 0;
      await tester.pumpWidget(
        harness(
          initial: '/recipes/r-1',
          recipe: (_) => recipeWithResolvedLine,
          starterInstall: (_) async => noCatalogStarterReport,
          pantry: () {
            reads++;
            if (reads == 1) return [pantryEntries[1]];
            throw const KimattaError.notOpen();
          },
          setMark: (_, ingredient, marked) async =>
              pantryEntryMarked(ingredient, marked),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byType(Switch));
      await tester.pumpAndSettle();
      expect(reads, 2, reason: 'the miss path must have attempted the re-read');
      expect(
        find.byType(Switch),
        findsOneWidget,
        reason:
            'a failed re-list must not strip the control off a committed mark',
      );
      expect(
        find.byType(SnackBar),
        findsNothing,
        reason: 'AC-2: a pantry failure is not reported on the recipe detail',
      );
      expect(tester.takeException(), isNull);
    },
  );

  /// The starter install grows the catalog after `pantryProvider` has already read it, and
  /// `okStarterReport` models a first install (`catalogInstalled: 37`). Without the launch
  /// invalidate the screen shows `pantryEmptyCopy` until the app is killed.
  testWidgets('a starter install that grew the catalog re-reads the pantry', (
    tester,
  ) async {
    useTallView(tester);
    var reads = 0;
    await tester.pumpWidget(
      harness(
        initial: '/pantry',
        pantry: () {
          reads++;
          return reads == 1 ? const <PantryEntryDto>[] : pantryEntries;
        },
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byType(SwitchListTile), findsNWidgets(2));
  });

  /// The other half of the gate: a steady-state launch short-circuits and writes nothing, so
  /// re-reading would be pure cost. `initial: '/pantry'` is load-bearing — on the harness
  /// default route nothing watches `pantryProvider` and the count is 0 either way.
  testWidgets(
    'a starter install that wrote no catalog does not re-read the pantry',
    (tester) async {
      useTallView(tester);
      var reads = 0;
      await tester.pumpWidget(
        harness(
          initial: '/pantry',
          starterInstall: (_) async => noCatalogStarterReport,
          pantry: () {
            reads++;
            return pantryEntries;
          },
        ),
      );
      await tester.pumpAndSettle();
      expect(reads, 1);
    },
  );

  /// `pantryEmptyCopy` tells the user to pull down, so the gesture is pinned rather than only
  /// the callback. A flag rather than a call count, so the launch invalidate cannot silently
  /// satisfy the refresh.
  testWidgets('pulling down on an empty pantry re-reads it', (tester) async {
    useTallView(tester);
    var installed = false;
    await tester.pumpWidget(
      harness(
        initial: '/pantry',
        starterInstall: (_) async => noCatalogStarterReport,
        pantry: () => installed ? pantryEntries : const <PantryEntryDto>[],
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text(pantryEmptyCopy), findsOneWidget);

    installed = true;
    await tester.fling(find.byType(ListView), const Offset(0, 300), 1000);
    await tester.pumpAndSettle();
    expect(find.byType(SwitchListTile), findsNWidgets(2));
  });

  testWidgets('the pantry error arm retries', (tester) async {
    useTallView(tester);
    var installed = false;
    await tester.pumpWidget(
      harness(
        initial: '/pantry',
        starterInstall: (_) async => noCatalogStarterReport,
        pantry: () {
          if (installed) return pantryEntries;
          throw const KimattaError.notOpen();
        },
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text(
        describeFailure(const KimattaError.notOpen(), subject: 'Pantry'),
      ),
      findsOneWidget,
    );

    installed = true;
    await tester.tap(find.text('Try again'));
    await tester.pumpAndSettle();
    expect(find.byType(SwitchListTile), findsNWidgets(2));
  });

  /// The other half of `refresh`: a pull-to-refresh that fails over a list the user is already
  /// reading reports the reason without replacing that list with an error screen. Publishing
  /// the failure would cost the user every row to tell them a re-read did not work.
  testWidgets('a refresh that fails keeps the list and says why', (
    tester,
  ) async {
    useTallView(tester);
    var reads = 0;
    await tester.pumpWidget(
      harness(
        initial: '/pantry',
        starterInstall: (_) async => noCatalogStarterReport,
        pantry: () {
          reads++;
          if (reads == 1) return pantryEntries;
          throw const KimattaError.notOpen();
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.fling(find.byType(ListView), const Offset(0, 300), 1000);
    await tester.pumpAndSettle();
    expect(
      find.descendant(
        of: find.byType(SnackBar),
        matching: find.text(
          describeFailure(const KimattaError.notOpen(), subject: 'Pantry'),
        ),
      ),
      findsOneWidget,
    );
    expect(
      find.byType(SwitchListTile),
      findsNWidgets(2),
      reason: 'a failed refresh must not cost the user the list',
    );
  });

  /// AC-3's accessible half on the *second* pantry control. This is the only
  /// `Semantics`-wrapped `Switch` in `lib/`, and Key Decision 4 records that an annotation
  /// outside the tappable node leaves that node unnamed. Each expectation is kept on its own
  /// line so a failure names itself — this screen has never been checked against any of the
  /// three guidelines, so a red run does not on its own implicate the semantics shape.
  testWidgets(
    'the recipe detail pantry control meets the accessibility guidelines',
    (tester) async {
      usePixel5(tester);
      final handle = tester.ensureSemantics();
      for (final brightness in Brightness.values) {
        tester.platformDispatcher.platformBrightnessTestValue = brightness;
        await tester.pumpWidget(
          harness(
            initial: '/recipes/r-1',
            recipe: (_) => recipeWithResolvedLine,
            // No launch re-read, so the tree under assertion is the one read produced.
            starterInstall: (_) async => noCatalogStarterReport,
          ),
        );
        await tester.pumpAndSettle();
        await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
        await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
        await expectLater(tester, meetsGuideline(textContrastGuideline));
        expect(
          find.bySemanticsLabel(pantryRowLabel('chickpeas', false)),
          findsOneWidget,
          reason:
              'the control must carry its ingredient as its accessible name',
        );
      }
      handle.dispose();
    },
  );

  /// The label must track the value, not be fixed at "not marked".
  testWidgets('the recipe detail pantry label states the marked state', (
    tester,
  ) async {
    usePixel5(tester);
    final handle = tester.ensureSemantics();
    await tester.pumpWidget(
      harness(
        initial: '/recipes/r-1',
        recipe: (_) => recipeWithResolvedLine,
        starterInstall: (_) async => noCatalogStarterReport,
        pantry: () => [pantryEntryMarked(chickpeasRef, true)],
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.bySemanticsLabel(pantryRowLabel('chickpeas', true)),
      findsOneWidget,
    );
    handle.dispose();
  });

  // --- MVP-013: planner ---------------------------------------------------------------

  Finder cell(String date, MealSlotDto slot) =>
      find.byKey(ValueKey('cell:$date:${slot.name}'));
  Finder inCell(String date, MealSlotDto slot, Finder matching) =>
      find.descendant(of: cell(date, slot), matching: matching);
  // `byType` matches the exact generic instantiation, which is private to the screen.
  final menuButton = find.byWidgetPredicate((w) => w is PopupMenuButton);

  /// Opens the overflow menu of the only component in a cell and picks one entry.
  Future<void> pickAction(
    WidgetTester tester,
    String date,
    MealSlotDto slot,
    String action,
  ) async {
    await tester.tap(inCell(date, slot, menuButton));
    await tester.pumpAndSettle();
    await tester.tap(find.text(action).last);
    await tester.pumpAndSettle();
  }

  group('PlannerNotifier', () {
    /// `overrideWith` runs its argument once per family member, and a notifier instance binds
    /// to exactly one element for its whole life (`arg` and `_element` are `late final`), so a
    /// test that touches two offsets must hand over a factory rather than an instance — and
    /// even a single-offset test must keep its element alive, since under `autoDispose` the
    /// element is disposed the moment the read that built it lets go, and the next read builds
    /// a *second* one that the same instance cannot bind to.
    ///
    /// [clock] replaces the wall clock the anchor is built and re-read from; it is called
    /// once per read, so a counter closure hands out a different date each time.
    ProviderContainer containerFrom(
      PlannerNotifier Function() create, {
      List<int> offsets = const [0],
      String Function()? clock,
    }) {
      final container = ProviderContainer(
        overrides: [
          healthReportProvider.overrideWith((_) => okReport),
          householdProvider.overrideWith(
            () => _FakeHouseholdNotifier(() => okHousehold, null, null),
          ),
          planningCycleProvider.overrideWith(
            () =>
                _FakePlanningCycleNotifier(null, (_, a, l, s) async => okCycle),
          ),
          if (clock != null) plannerTodayProvider.overrideWith((_) => clock()),
          plannerProvider.overrideWith(create),
        ],
      );
      addTearDown(container.dispose);
      for (final offset in offsets) {
        container.listen(plannerProvider(offset), (_, _) {});
      }
      return container;
    }

    ProviderContainer containerWith(_FakePlannerNotifier notifier) =>
        containerFrom(() => notifier);

    test('save on an unseen id appends in date then slot order', () async {
      final notifier = _FakePlannerNotifier(
        meals: () => [planned(id: 'b', date: '2026-08-31')],
        save_: (m) async => m,
      );
      final container = containerWith(notifier);
      await container.read(plannerProvider(0).future);
      await notifier.save(
        planned(id: 'c', date: '2026-08-30', slot: MealSlotDto.lunch),
      );
      await notifier.save(
        planned(id: 'a', date: '2026-08-30', slot: MealSlotDto.breakfast),
      );
      final ids = container
          .read(plannerProvider(0))
          .value!
          .meals
          .map((m) => m.id);
      expect(ids, ['a', 'c', 'b']);
    });

    test('save on a known id replaces it and re-sorts a move', () async {
      final notifier = _FakePlannerNotifier(
        meals: () => [planned(id: 'a'), planned(id: 'b', date: '2026-08-30')],
        save_: (m) async => m,
      );
      final container = containerWith(notifier);
      await container.read(plannerProvider(0).future);
      await notifier.save(planned(id: 'a', date: '2026-09-01'));
      final meals = container.read(plannerProvider(0)).value!.meals;
      expect(meals.map((m) => m.id), ['b', 'a']);
      expect(meals.last.date, '2026-09-01');
    });

    test('setLock republishes the stored lock state', () async {
      final notifier = _FakePlannerNotifier(
        meals: () => [planned()],
        lock: (_, id, locked) async => planned(id: id, locked: locked),
      );
      final container = containerWith(notifier);
      await container.read(plannerProvider(0).future);
      await notifier.setLock('h-1', 'pm-1', true);
      expect(
        container.read(plannerProvider(0)).value!.meals.single.locked,
        isTrue,
      );
    });

    test('delete drops the occurrence', () async {
      final notifier = _FakePlannerNotifier(
        meals: () => [planned(id: 'a'), planned(id: 'b', date: '2026-08-30')],
        delete_: (_, _) async {},
      );
      final container = containerWith(notifier);
      await container.read(plannerProvider(0).future);
      await notifier.delete('h-1', 'a');
      expect(container.read(plannerProvider(0)).value!.meals.single.id, 'b');
    });

    test(
      'a refresh that fails over a value returns the error unpublished',
      () async {
        var reads = 0;
        final notifier = _FakePlannerNotifier(
          meals: () =>
              ++reads == 1 ? [planned()] : throw const KimattaError.notOpen(),
        );
        final container = containerWith(notifier);
        await container.read(plannerProvider(0).future);
        final error = await notifier.refresh();
        expect(error, isA<KimattaError_NotOpen>());
        expect(container.read(plannerProvider(0)).value!.meals, hasLength(1));
      },
    );

    test(
      'a cycle save re-runs build so the window follows the rhythm',
      () async {
        var windows = 0;
        final notifier = _FakePlannerNotifier(
          window: (_) {
            windows++;
            return okCycle;
          },
        );
        final container = containerWith(notifier);
        await container.read(plannerProvider(0).future);
        expect(windows, 1);
        await container
            .read(planningCycleProvider.notifier)
            .save(
              'h-1',
              anchorDate: '2026-09-01',
              lengthDays: 3,
              mealSlots: [MealSlotDto.dinner],
            );
        await container.read(plannerProvider(0).future);
        expect(windows, 2);
      },
    );

    test('every offset is anchored to the one shared today', () async {
      var clockReads = 0;
      final anchors = <String>[];
      final container = ProviderContainer(
        overrides: [
          healthReportProvider.overrideWith((_) => okReport),
          householdProvider.overrideWith(
            () => _FakeHouseholdNotifier(() => okHousehold, null, null),
          ),
          planningCycleProvider.overrideWith(
            () =>
                _FakePlanningCycleNotifier(null, (_, a, l, s) async => okCycle),
          ),
          plannerTodayProvider.overrideWith((_) {
            clockReads++;
            return '1999-01-01';
          }),
          plannerProvider.overrideWith(
            () => _FakePlannerNotifier(
              seenWindow: (today, _) => anchors.add(today),
            ),
          ),
        ],
      );
      addTearDown(container.dispose);
      container.listen(plannerProvider(0), (_, _) {});
      container.listen(plannerProvider(1), (_, _) {});
      await container.read(plannerProvider(0).future);
      await container.read(plannerProvider(1).future);
      // Both offsets report the injected date, not the wall clock each build happened to see:
      // the equality is what makes Previous/Next a sequence instead of two unrelated windows.
      expect(anchors, ['1999-01-01', '1999-01-01']);
      expect(clockReads, 1);
    });

    test(
      'a refresh re-reads the clock so a stale anchor cannot persist',
      () async {
        var clockReads = 0;
        final anchors = <String>[];
        final container = ProviderContainer(
          overrides: [
            healthReportProvider.overrideWith((_) => okReport),
            householdProvider.overrideWith(
              () => _FakeHouseholdNotifier(() => okHousehold, null, null),
            ),
            planningCycleProvider.overrideWith(
              () => _FakePlanningCycleNotifier(
                null,
                (_, a, l, s) async => okCycle,
              ),
            ),
            plannerTodayProvider.overrideWith(
              (_) => '1999-01-0${++clockReads}',
            ),
            plannerProvider.overrideWith(
              () => _FakePlannerNotifier(
                seenWindow: (today, _) => anchors.add(today),
              ),
            ),
          ],
        );
        addTearDown(container.dispose);
        container.listen(plannerProvider(0), (_, _) {});
        await container.read(plannerProvider(0).future);
        // The only affordance that re-reads the clock. Without it the shared anchor, being
        // kept alive for the container, would hold a session open across a cycle boundary on
        // last cycle's window with nothing able to correct it.
        await container.read(plannerProvider(0).notifier).refresh();
        expect(anchors, ['1999-01-01', '1999-01-02']);
      },
    );

    test('a failed refresh leaves the anchor where it was', () async {
      var clockReads = 0;
      final anchors = <String>[];
      var failing = false;
      final container = containerFrom(
        () => _FakePlannerNotifier(
          window: (_) => failing ? throw const KimattaError.notOpen() : okCycle,
          seenWindow: (today, _) => anchors.add(today),
        ),
        clock: () => '1999-01-0${++clockReads}',
      );
      await container.read(plannerProvider(0).future);
      failing = true;
      expect(
        await container.read(plannerProvider(0).notifier).refresh(),
        isA<KimattaError_NotOpen>(),
      );
      failing = false;
      // Offset 1 is built only now, so what it anchors to is the anchor as the failed
      // refresh left it rather than as it stood before.
      container.listen(plannerProvider(1), (_, _) {});
      await container.read(plannerProvider(1).future);
      // The failed read tried '1999-01-02' but never committed it: the window on screen is
      // still the one built from '1999-01-01', so Next has to be W+1 measured from that. A
      // committed-anyway anchor would make this the *third* date and skip W+1 entirely.
      expect(anchors, ['1999-01-01', '1999-01-02', '1999-01-01']);
    });

    test(
      'a refresh that succeeds after a failed one commits the newer anchor',
      () async {
        var clockReads = 0;
        final anchors = <String>[];
        var failing = false;
        final container = containerFrom(
          () => _FakePlannerNotifier(
            window: (_) =>
                failing ? throw const KimattaError.notOpen() : okCycle,
            seenWindow: (today, _) => anchors.add(today),
          ),
          clock: () => '1999-01-0${++clockReads}',
        );
        await container.read(plannerProvider(0).future);
        final notifier = container.read(plannerProvider(0).notifier);
        failing = true;
        await notifier.refresh();
        failing = false;
        await notifier.refresh();
        container.listen(plannerProvider(1), (_, _) {});
        await container.read(plannerProvider(1).future);
        // Holding the anchor back on failure must not strand it there: the next refresh that
        // comes back is what the following offsets are measured from. Expected to pass on
        // either side of the fix — it pins the half of the contract the fix must not break.
        expect(anchors, [
          '1999-01-01',
          '1999-01-02',
          '1999-01-03',
          '1999-01-03',
        ]);
      },
    );
  });

  testWidgets(
    'the planner renders one cell per date and slot with Add and no weekday',
    (tester) async {
      useTallView(tester);
      await tester.pumpWidget(harness());
      await tester.pumpAndSettle();
      expect(title('Plan'), findsOneWidget);
      expect(find.text(windowHeading(okCycle)), findsOneWidget);
      expect(find.text(plannerEmptyCopy), findsOneWidget);
      for (final date in okCycle.dates) {
        expect(
          inCell(date, MealSlotDto.dinner, find.text('Add')),
          findsOneWidget,
        );
        expect(
          inCell(date, MealSlotDto.dinner, find.text(emptyCellCopy)),
          findsOneWidget,
        );
      }
      expect(find.byType(Card), findsNWidgets(7));
      // The card's stop condition: no hard-coded weekday anywhere on the screen.
      for (final day in ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']) {
        expect(find.textContaining(day), findsNothing, reason: day);
      }
    },
  );

  testWidgets('adding a recipe from the picker saves one recipe component', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <PlannedMealDto>[];
    await tester.pumpWidget(
      harness(
        recipes: () => const [okSummary],
        savePlanned: (m) async {
          sent.add(m);
          return planned(
            id: 'pm-new',
            date: m.date,
            slot: m.slot,
            components: m.components,
          );
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(
      inCell('2026-08-30', MealSlotDto.dinner, find.text('Add')),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Pancakes'));
    await tester.pumpAndSettle();
    expect(sent.single.id, '');
    expect(sent.single.date, '2026-08-30');
    expect(sent.single.slot, MealSlotDto.dinner);
    expect(sent.single.components, [
      const MealComponentDto(kind: 'recipe', recipeId: 'r-1'),
    ]);
    expect(find.text(plannerEmptyCopy), findsNothing);
    expect(
      inCell('2026-08-30', MealSlotDto.dinner, find.text('Pancakes · 1×')),
      findsOneWidget,
    );
  });

  /// The sheet is driven with bare pumps, never `pumpAndSettle`: an unresolved section holds
  /// a `CircularProgressIndicator`, which schedules frames forever.
  Future<void> openPicker(WidgetTester tester) async {
    await tester.tap(
      inCell('2026-08-30', MealSlotDto.dinner, find.text('Add')),
    );
    await tester.pump();
    await tester.pump(const Duration(seconds: 1));
  }

  testWidgets('the picker does not call an unread library empty', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(recipes: () => Completer<List<RecipeSummaryDto>>().future),
    );
    await tester.pumpAndSettle();
    await openPicker(tester);
    // The first-run path: an empty plan builds no component tile, so the sheet is opened
    // before anything has read the library. "You have no recipes" is then false, not merely
    // incomplete.
    expect(find.text(emptyLibraryCopy), findsNothing);
    expect(find.byType(CircularProgressIndicator), findsWidgets);
  });

  testWidgets('the picker reports reads that failed rather than spinning', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        recipes: () => throw const KimattaError.notOpen(),
        componentKinds: () => throw const KimattaError.notOpen(),
      ),
    );
    await tester.pumpAndSettle();
    await openPicker(tester);
    // A spinner over a failed read promises an arrival that is not coming, and with the kinds
    // gone the sheet would offer no way to pick anything with no stated reason.
    expect(find.text(libraryUnreadCopy), findsOneWidget);
    expect(find.text(kindsUnreadCopy), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsNothing);
    expect(find.text(emptyLibraryCopy), findsNothing);
  });

  testWidgets('the picker still calls a resolved empty library empty', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(harness(recipes: () => const []));
    await tester.pumpAndSettle();
    await openPicker(tester);
    // Expected-to-pass: pins that distinguishing unresolved from empty did not trade one
    // false statement for a permanent spinner over a library that really is empty.
    expect(find.text(emptyLibraryCopy), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });

  testWidgets('Remove on the last component confirms and deletes the meal', (
    tester,
  ) async {
    useTallView(tester);
    final deleted = <String>[];
    await tester.pumpWidget(
      harness(
        recipes: () => const [okSummary],
        planner: () => [okPlanned],
        deletePlanned: (_, id) async => deleted.add(id),
      ),
    );
    await tester.pumpAndSettle();
    await pickAction(tester, '2026-08-29', MealSlotDto.dinner, 'Remove');
    expect(find.text(removeLastComponentTitle), findsOneWidget);
    await tester.tap(find.widgetWithText(FilledButton, 'Remove'));
    await tester.pumpAndSettle();
    expect(deleted, ['pm-1']);
    expect(
      inCell('2026-08-29', MealSlotDto.dinner, find.text(emptyCellCopy)),
      findsOneWidget,
    );
  });

  testWidgets(
    'Remove on one of two components re-saves without it, unconfirmed',
    (tester) async {
      useTallView(tester);
      final sent = <PlannedMealDto>[];
      await tester.pumpWidget(
        harness(
          recipes: () => const [okSummary],
          planner: () => [
            planned(
              components: const [
                MealComponentDto(kind: 'recipe', recipeId: 'r-1'),
                MealComponentDto(kind: 'leftovers', note: 'chili'),
              ],
            ),
          ],
          savePlanned: (m) async {
            sent.add(m);
            return m;
          },
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(menuButton.last);
      await tester.pumpAndSettle();
      await tester.tap(find.text('Remove').last);
      await tester.pumpAndSettle();
      expect(find.text(removeLastComponentTitle), findsNothing);
      expect(sent.single.components, [
        const MealComponentDto(kind: 'recipe', recipeId: 'r-1'),
      ]);
    },
  );

  testWidgets('Move lists only empty cells and re-saves with the chosen one', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <PlannedMealDto>[];
    await tester.pumpWidget(
      harness(
        recipes: () => const [okSummary],
        planner: () => [okPlanned, planned(id: 'pm-2', date: '2026-08-30')],
        savePlanned: (m) async {
          sent.add(m);
          return m;
        },
      ),
    );
    await tester.pumpAndSettle();
    await pickAction(tester, '2026-08-29', MealSlotDto.dinner, 'Move…');
    expect(find.text('2026-08-30 · Dinner'), findsNothing);
    expect(find.text('2026-08-29 · Dinner'), findsNothing);
    await tester.tap(find.text('2026-08-31 · Dinner'));
    await tester.pumpAndSettle();
    expect(sent.single.id, 'pm-1');
    expect(sent.single.date, '2026-08-31');
    expect(
      inCell('2026-08-31', MealSlotDto.dinner, find.text('Pancakes · 1×')),
      findsOneWidget,
    );
    expect(
      inCell('2026-08-29', MealSlotDto.dinner, find.text(emptyCellCopy)),
      findsOneWidget,
    );
  });

  testWidgets('Move with no free cell says so', (tester) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        cycleWindow: (_) => const PlanningCycleDto(
          householdId: 'h-1',
          anchorDate: '2026-08-29',
          lengthDays: 1,
          mealSlots: [MealSlotDto.dinner],
          dates: ['2026-08-29'],
        ),
        planner: () => [okPlanned],
      ),
    );
    await tester.pumpAndSettle();
    await pickAction(tester, '2026-08-29', MealSlotDto.dinner, 'Move…');
    expect(find.text(noFreeSlotCopy), findsOneWidget);
  });

  testWidgets('Scale 1½× sends a 3/2 scale on the recipe component', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <PlannedMealDto>[];
    await tester.pumpWidget(
      harness(
        recipes: () => const [okSummary],
        planner: () => [okPlanned],
        savePlanned: (m) async {
          sent.add(m);
          return m;
        },
      ),
    );
    await tester.pumpAndSettle();
    await pickAction(tester, '2026-08-29', MealSlotDto.dinner, 'Scale…');
    await tester.tap(find.text('1½×'));
    await tester.pumpAndSettle();
    expect(
      sent.single.components.single.scale,
      const ScaleDto(numer: 3, denom: 2),
    );
    expect(find.text('Pancakes · 1½×'), findsOneWidget);
  });

  testWidgets('a non-recipe component offers no Scale', (tester) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        planner: () => [
          planned(components: const [MealComponentDto(kind: 'dining_out')]),
        ],
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Dining out'), findsOneWidget);
    await tester.tap(menuButton);
    await tester.pumpAndSettle();
    expect(find.text('Scale…'), findsNothing);
    expect(find.text('Move…'), findsOneWidget);
  });

  testWidgets(
    'the lock switch writes the lock and is labelled by its meaning',
    (tester) async {
      useTallView(tester);
      final sent = <(String, String, bool)>[];
      await tester.pumpWidget(
        harness(
          recipes: () => const [okSummary],
          planner: () => [okPlanned],
          lockPlanned: (h, id, locked) async {
            sent.add((h, id, locked));
            return planned(locked: locked);
          },
        ),
      );
      await tester.pumpAndSettle();
      expect(find.bySemanticsLabel(lockLabel(false)), findsOneWidget);
      await tester.tap(find.byType(SwitchListTile));
      await tester.pumpAndSettle();
      expect(sent, [('h-1', 'pm-1', true)]);
      expect(find.bySemanticsLabel(lockLabel(true)), findsOneWidget);
      expect(
        tester.widget<SwitchListTile>(find.byType(SwitchListTile)).value,
        isTrue,
      );
    },
  );

  testWidgets('a locked meal keeps Add and the component menu enabled', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        recipes: () => const [okSummary],
        planner: () => [planned(locked: true)],
      ),
    );
    await tester.pumpAndSettle();
    final add = inCell(
      '2026-08-29',
      MealSlotDto.dinner,
      find.widgetWithText(FilledButton, 'Add'),
    );
    expect(tester.widget<FilledButton>(add).enabled, isTrue);
    expect(tester.widget<PopupMenuButton>(menuButton).enabled, isTrue);
  });

  testWidgets(
    'a component whose recipe conflicts shows the warning and opens it',
    (tester) async {
      useTallView(tester);
      await tester.pumpWidget(
        harness(
          recipes: () => const [conflictSummary],
          planner: () => [
            planned(
              components: const [
                MealComponentDto(kind: 'recipe', recipeId: 'r-2'),
              ],
            ),
          ],
          recipe: (_) => okRecipeWithConflicts,
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text(summariseConflicts(dairyAssessment)), findsOneWidget);
      await tester.tap(find.text('Butter toast · 1×'));
      await tester.pumpAndSettle();
      expect(title('Pancakes'), findsOneWidget);
    },
  );

  testWidgets(
    'open is offered for an empty cell and hidden beside a component',
    (tester) async {
      useTallView(tester);
      await tester.pumpWidget(
        harness(recipes: () => const [okSummary], planner: () => [okPlanned]),
      );
      await tester.pumpAndSettle();
      await tester.tap(
        inCell('2026-08-30', MealSlotDto.dinner, find.text('Add')),
      );
      await tester.pumpAndSettle();
      expect(find.text(kindLabel('open')), findsOneWidget);
      // Dismiss the sheet by tapping the barrier, then open the occupied cell's picker.
      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();
      await tester.tap(
        inCell('2026-08-29', MealSlotDto.dinner, find.text('Add')),
      );
      await tester.pumpAndSettle();
      expect(find.text(kindLabel('leftovers')), findsOneWidget);
      expect(find.text(kindLabel('open')), findsNothing);
    },
  );

  testWidgets('freeform needs a note before it saves', (tester) async {
    useTallView(tester);
    final sent = <PlannedMealDto>[];
    await tester.pumpWidget(
      harness(
        savePlanned: (m) async {
          sent.add(m);
          return planned(id: 'pm-f', components: m.components);
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(
      inCell('2026-08-29', MealSlotDto.dinner, find.text('Add')),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text(kindLabel('freeform')));
    await tester.pumpAndSettle();
    expect(find.text(noteRequiredCopy), findsOneWidget);
    expect(sent, isEmpty);
    await tester.enterText(find.byType(TextField), ' tacos ');
    await tester.tap(find.text(kindLabel('freeform')));
    await tester.pumpAndSettle();
    expect(
      sent.single.components.single,
      const MealComponentDto(kind: 'freeform', note: 'tacos'),
    );
  });

  testWidgets('an archived recipe is marked and an unknown one is honest', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        archived: () => const [conflictSummary],
        planner: () => [
          planned(
            components: const [
              MealComponentDto(kind: 'recipe', recipeId: 'r-2'),
            ],
          ),
          planned(
            id: 'pm-2',
            date: '2026-08-30',
            components: const [
              MealComponentDto(kind: 'recipe', recipeId: 'ghost'),
            ],
          ),
        ],
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('${archivedRecipeCopy('Butter toast')} · 1×'),
      findsOneWidget,
    );
    expect(find.text('$unknownRecipeCopy · 1×'), findsOneWidget);
    // The assessment travels with the recipe, so archiving a recipe cannot quietly clear the
    // warning on the meals it is still planned on (AC-2, invariant 10).
    expect(find.text(summariseConflicts(dairyAssessment)), findsOneWidget);
  });

  testWidgets('an archived recipe\'s warning is still explainable in context', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        archived: () => const [conflictSummary],
        planner: () => [
          planned(
            components: const [
              MealComponentDto(kind: 'recipe', recipeId: 'r-2'),
            ],
          ),
        ],
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text(summariseConflicts(dairyAssessment)));
    await tester.pumpAndSettle();
    // `recipe:` is left at its default, which knows only `r-1`, so the detail's own honest
    // arm is the assertion: reaching it at all is what proves the tile navigated.
    expect(find.text('Recipe not found.'), findsOneWidget);
    expect(title('Plan'), findsNothing);
  });

  testWidgets('an archived recipe without conflicts stays quiet and inert', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        archived: () => const [checkedCleanSummary],
        planner: () => [
          planned(
            components: const [
              MealComponentDto(kind: 'recipe', recipeId: 'r-3'),
            ],
          ),
        ],
      ),
    );
    await tester.pumpAndSettle();
    // Expected-to-pass: pins that reaching the archived assessment did not make every
    // archived tile a warning or a link — the conflict list is still what decides both.
    expect(find.text('${archivedRecipeCopy('Rice')} · 1×'), findsOneWidget);
    expect(find.textContaining('May conflict'), findsNothing);
    await tester.tap(find.text('${archivedRecipeCopy('Rice')} · 1×'));
    await tester.pumpAndSettle();
    expect(title('Plan'), findsOneWidget);
  });

  testWidgets('a planned recipe says nothing while the library is unread', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        recipes: () => Completer<List<RecipeSummaryDto>>().future,
        planner: () => [planned()],
      ),
    );
    await tester.pumpAndSettle();
    // The frame that paints a component is the frame that starts the library read, so this
    // is every cold start — and `Recipe unavailable` there is a false statement about a
    // recipe that is in the library, with the same missing conflict line beside it.
    expect(find.text('$pendingRecipeCopy · 1×'), findsOneWidget);
    expect(find.text('$unknownRecipeCopy · 1×'), findsNothing);
  });

  testWidgets('a planned recipe is honest when the library read fails', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        recipes: () => throw const KimattaError.notOpen(),
        planner: () => [planned()],
      ),
    );
    await tester.pumpAndSettle();
    // A failure is not a slow read: nothing on this screen retries the library, so promising
    // the name is on its way would be the same false statement in the other direction.
    expect(find.text('$unreadRecipeCopy · 1×'), findsOneWidget);
    expect(find.text('$pendingRecipeCopy · 1×'), findsNothing);
    expect(find.text('$unknownRecipeCopy · 1×'), findsNothing);
  });

  testWidgets('the planner renders a load failure and Try again retries', (
    tester,
  ) async {
    useTallView(tester);
    var reads = 0;
    await tester.pumpWidget(
      harness(
        planner: () => ++reads == 1
            ? throw const KimattaError.notOpen()
            : const <PlannedMealDto>[],
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text(describeFailure(const KimattaError.notOpen(), subject: 'Plan')),
      findsOneWidget,
    );
    expect(find.byType(Card), findsNothing);
    await tester.tap(find.text('Try again'));
    await tester.pumpAndSettle();
    expect(find.byType(Card), findsNWidgets(7));
  });

  testWidgets('a failed planner save reports the reason in a snackbar', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        recipes: () => const [okSummary],
        savePlanned: (_) async =>
            throw const KimattaError.plannedMeal(message: 'occupied'),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(
      inCell('2026-08-29', MealSlotDto.dinner, find.text('Add')),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Pancakes'));
    await tester.pumpAndSettle();
    expect(
      find.descendant(
        of: find.byType(SnackBar),
        matching: find.text(
          describeFailure(
            const KimattaError.plannedMeal(message: 'occupied'),
            subject: 'Plan',
          ),
        ),
      ),
      findsOneWidget,
    );
    expect(
      inCell('2026-08-29', MealSlotDto.dinner, find.text(emptyCellCopy)),
      findsOneWidget,
    );
  });

  testWidgets(
    'Previous and Next cycle request the neighbouring offsets and re-read on '
    'return',
    (tester) async {
      useTallView(tester);
      final offsets = <int>[];
      await tester.pumpWidget(
        harness(
          cycleWindow: (offset) {
            offsets.add(offset);
            return okCycle;
          },
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Previous cycle'));
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Next cycle'));
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Next cycle'));
      await tester.pumpAndSettle();
      // The second 0 is the `autoDispose` lifetime, not a redundant read: the screen watches
      // one offset at a time, so paging away frees the one it left. What the re-read must
      // *not* change is the anchor, which the notifier tests pin separately.
      expect(offsets, [0, -1, 0, 1]);
    },
  );

  testWidgets('the planner with a meal meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(recipes: () => const [okSummary], planner: () => [okPlanned]),
    );
    await tester.pumpAndSettle();
    await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
    await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
    await expectLater(tester, meetsGuideline(textContrastGuideline));
  });

  testWidgets('the planner with a meal survives text scale 2.0', (
    tester,
  ) async {
    usePixel5(tester);
    tester.platformDispatcher.textScaleFactorTestValue = 2.0;
    await tester.pumpWidget(
      harness(
        recipes: () => const [conflictSummary],
        planner: () => [
          planned(
            components: const [
              MealComponentDto(kind: 'recipe', recipeId: 'r-2'),
            ],
          ),
        ],
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  });

  // --- MVP-016: the shopping screen ------------------------------------------------------

  Finder checkboxFor(String name) => find.ancestor(
    of: find.text(name),
    matching: find.byType(CheckboxListTile),
  );

  Future<void> openMenu(WidgetTester tester) async {
    await tester.tap(find.byTooltip('More'));
    await tester.pumpAndSettle();
  }

  /// AC-1: lines land under their store category, the uncategorised line under `Other`, the
  /// omitted line under Already have, and the explain sheet names every contribution.
  testWidgets('the shopping screen groups lines and explains one', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(initial: '/shopping', shopping: (_, _) => shoppingView()),
    );
    await tester.pumpAndSettle();
    expect(title(shoppingTitle), findsOneWidget);
    expect(
      find.text(shoppingHeading('2026-08-29', '2026-09-04')),
      findsOneWidget,
    );
    expect(find.text(countsCopy(2, 1, 0)), findsOneWidget);
    expect(find.text(neededHeading), findsOneWidget);
    expect(find.text('baking'), findsOneWidget);
    expect(find.text(uncategorisedHeading), findsOneWidget);
    expect(find.text('flour'), findsOneWidget);
    expect(find.text('mystery'), findsOneWidget);
    expect(find.text('$alreadyHaveHeading (1)'), findsOneWidget);
    expect(find.text('5/2 cup'), findsOneWidget);
    expect(find.text('amount not known · $optionalCopy'), findsOneWidget);
    expect(find.text(manualEditPolicyCopy), findsOneWidget);
    // The omitted line is not in the To buy list until expanded.
    expect(find.text('onion'), findsNothing);

    await tester.tap(find.byTooltip('Why is this here?').first);
    await tester.pumpAndSettle();
    for (final c in okShoppingList.groups[0].lines[0].contributions) {
      expect(find.text(explainContribution(c)), findsOneWidget);
    }
    expect(find.text('Remove from list'), findsOneWidget);
    expect(find.text('Back to pantry'), findsNothing);
  });

  /// AC-2: a tap sends the line's key and flags, and the row renders what Rust stored.
  testWidgets('checking a line sends its state and renders what was stored', (
    tester,
  ) async {
    useTallView(tester);
    final handle = tester.ensureSemantics();
    final sent = <ShoppingLineStateDto>[];
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(),
        setLineState: (s) {
          sent.add(s);
          return echoLineState(s);
        },
      ),
    );
    await tester.pumpAndSettle();
    expect(find.bySemanticsLabel(lineLabel('flour', false)), findsOneWidget);
    await tester.tap(checkboxFor('flour'));
    await tester.pumpAndSettle();
    expect(sent, hasLength(1));
    expect(sent.single.key, flourKey);
    expect(sent.single.checked, isTrue);
    expect(sent.single.hidden, isFalse);
    expect(sent.single.restored, isFalse);
    expect(find.bySemanticsLabel(lineLabel('flour', true)), findsOneWidget);
    expect(tester.widget<CheckboxListTile>(checkboxFor('flour')).value, isTrue);
    // Unchecking clears the row: Rust deletes it, and the view drops the state.
    await tester.tap(checkboxFor('flour'));
    await tester.pumpAndSettle();
    expect(sent.last.checked, isFalse);
    expect(find.bySemanticsLabel(lineLabel('flour', false)), findsOneWidget);
    handle.dispose();
  });

  /// Decision 2: a check made against a different amount is shown as changed and rendered
  /// unchecked — never kept.
  testWidgets('a changed line renders unchecked with the changed copy', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(
          states: [
            lineStateOf(
              flourKey,
              checked: true,
              changed: true,
              checkedAgainst: 'exact:1/1|known:cup',
            ),
          ],
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      tester.widget<CheckboxListTile>(checkboxFor('flour')).value,
      isFalse,
    );
    expect(find.text('5/2 cup · ${changedCopy('1 cup')}'), findsOneWidget);
    // Not eligible for the pantry while changed.
    await openMenu(tester);
    final item = tester.widget<PopupMenuItem<String>>(
      find.widgetWithText(PopupMenuItem<String>, 'Add checked to pantry'),
    );
    expect(item.enabled, isFalse);
  });

  testWidgets('removing a line moves it to Removed and Put back restores it', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <ShoppingLineStateDto>[];
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(),
        setLineState: (s) {
          sent.add(s);
          return echoLineState(s);
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Why is this here?').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Remove from list'));
    await tester.pumpAndSettle();
    expect(sent.single.key, flourKey);
    expect(sent.single.hidden, isTrue);
    expect(find.text(countsCopy(1, 1, 1)), findsOneWidget);
    expect(checkboxFor('flour'), findsNothing);
    expect(find.text('$removedHeading (1)'), findsOneWidget);

    await tester.tap(find.text('$removedHeading (1)'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Put back'));
    await tester.pumpAndSettle();
    expect(sent.last.hidden, isFalse);
    expect(sent.last.restored, isFalse);
    expect(find.text(countsCopy(2, 1, 0)), findsOneWidget);
    expect(checkboxFor('flour'), findsOneWidget);
  });

  testWidgets('an omitted line can be added anyway and sent back to pantry', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <ShoppingLineStateDto>[];
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(),
        setLineState: (s) {
          sent.add(s);
          return echoLineState(s);
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('$alreadyHaveHeading (1)'));
    await tester.pumpAndSettle();
    expect(find.textContaining(omittedCopy), findsOneWidget);
    await tester.tap(find.text('Add anyway'));
    await tester.pumpAndSettle();
    expect(sent.single.key, onionKey);
    expect(sent.single.restored, isTrue);
    expect(find.text(countsCopy(3, 0, 0)), findsOneWidget);
    expect(checkboxFor('onion'), findsOneWidget);

    // onion now sits second in the baking group, so its explain button is the second.
    await tester.tap(find.byTooltip('Why is this here?').at(1));
    await tester.pumpAndSettle();
    expect(find.text(omittedCopy), findsOneWidget);
    await tester.tap(find.text('Back to pantry'));
    await tester.pumpAndSettle();
    expect(sent.last.key, onionKey);
    expect(sent.last.restored, isFalse);
    expect(find.text(countsCopy(2, 1, 0)), findsOneWidget);
    expect(checkboxFor('onion'), findsNothing);
  });

  testWidgets('manual items can be added, checked, edited and deleted', (
    tester,
  ) async {
    useTallView(tester);
    final saved = <ShoppingManualItemDto>[];
    final deleted = <String>[];
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(items: [batteries]),
        saveItem: (item) async {
          saved.add(item);
          return ShoppingManualItemDto(
            id: item.id.isEmpty ? 'mi-2' : item.id,
            householdId: item.householdId,
            name: item.name.trim(),
            note: item.note,
            checked: item.checked,
          );
        },
        deleteItem: (id) async => deleted.add(id),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text(yourItemsHeading), findsOneWidget);
    expect(find.text('batteries'), findsOneWidget);
    expect(find.text('AA'), findsOneWidget);

    // A blank name is refused inside the dialog; nothing is sent.
    await tester.tap(find.text('Add item'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(find.text(manualNameRequiredCopy), findsOneWidget);
    expect(saved, isEmpty);
    await tester.enterText(find.widgetWithText(TextField, 'Name'), ' candles ');
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(saved.single.id, isEmpty, reason: 'a blank id lets Rust mint one');
    expect(saved.single.householdId, 'h-1');
    expect(saved.single.name, ' candles ');
    expect(saved.single.note, isNull);
    expect(find.text('candles'), findsOneWidget);
    // Sorted case-insensitively: batteries before candles.
    expect(
      tester.getTopLeft(find.text('batteries')).dy,
      lessThan(tester.getTopLeft(find.text('candles')).dy),
    );

    await tester.tap(checkboxFor('candles'));
    await tester.pumpAndSettle();
    expect(saved.last.id, 'mi-2');
    expect(saved.last.checked, isTrue);
    expect(
      tester.widget<CheckboxListTile>(checkboxFor('candles')).value,
      isTrue,
    );

    await tester.tap(find.byTooltip('Edit item').at(1));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.widgetWithText(TextField, 'Name'),
      'tea lights',
    );
    await tester.enterText(find.widgetWithText(TextField, 'Note'), 'unscented');
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(saved.last.id, 'mi-2');
    expect(saved.last.name, 'tea lights');
    expect(saved.last.note, 'unscented');
    expect(saved.last.checked, isTrue, reason: 'an edit keeps the check');
    expect(find.text('tea lights'), findsOneWidget);
    expect(find.text('candles'), findsNothing);

    await tester.tap(find.byTooltip('Edit item').at(1));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete'));
    await tester.pumpAndSettle();
    expect(deleted, ['mi-2']);
    expect(find.text('tea lights'), findsNothing);
    expect(find.text('batteries'), findsOneWidget);
  });

  /// AC-3 and decision 8: only the eligible refs are sent, Undo sends exactly the returned
  /// set, the Pantry tab shows the mark, and the shopping view re-reads after each.
  testWidgets(
    'add checked to pantry sends only the eligible refs and Undo sends the returned set',
    (tester) async {
      useTallView(tester);
      var fetches = 0;
      var chickpeasMarked = false;
      final calls = <(List<IngredientRefDto>, bool)>[];
      await tester.pumpWidget(
        harness(
          initial: '/shopping',
          shopping: (_, _) {
            fetches++;
            return shoppingView(
              states: [
                lineStateOf(flourKey, checked: true),
                // Unresolved, so never eligible however it is checked.
                lineStateOf(mysteryKey, checked: true),
              ],
            );
          },
          pantry: () => [
            pantryEntryMarked(chickpeasRef, chickpeasMarked),
            pantryEntries[1],
          ],
          setPantryMarks: (refs, marked) async {
            calls.add((refs, marked));
            chickpeasMarked = marked;
            // A pre-existing mark would be absent here; the caller must send this back.
            return [chickpeasRef];
          },
        ),
      );
      await tester.pumpAndSettle();
      final before = fetches;
      await openMenu(tester);
      await tester.tap(find.text('Add checked to pantry'));
      await tester.pumpAndSettle();
      expect(calls, hasLength(1));
      expect(calls.single.$1, [chickpeasRef]);
      expect(calls.single.$2, isTrue);
      expect(find.text(addedToPantryCopy(1)), findsOneWidget);
      expect(
        fetches,
        greaterThan(before),
        reason: 'invalidating the pantry re-runs the shopping build',
      );

      await tester.tap(tab('Pantry'));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<SwitchListTile>(
              find.ancestor(
                of: find.text('chickpeas'),
                matching: find.byType(SwitchListTile),
              ),
            )
            .value,
        isTrue,
      );

      await tester.tap(tab('Shopping'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Undo'));
      await tester.pumpAndSettle();
      expect(calls, hasLength(2));
      expect(calls.last.$1, [chickpeasRef]);
      expect(calls.last.$2, isFalse);
    },
  );

  testWidgets('Start over names the counts and clears only after confirm', (
    tester,
  ) async {
    useTallView(tester);
    var cleared = false;
    final resets = <(String, String)>[];
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => cleared
            ? shoppingView(items: [batteries])
            : shoppingView(
                states: [
                  lineStateOf(flourKey, checked: true),
                  lineStateOf(mysteryKey, hidden: true),
                ],
                items: [
                  batteries,
                  const ShoppingManualItemDto(
                    id: 'mi-2',
                    householdId: 'h-1',
                    name: 'candles',
                    checked: true,
                  ),
                ],
              ),
        resetShopping: (from, to) async {
          resets.add((from, to));
          cleared = true;
        },
      ),
    );
    await tester.pumpAndSettle();
    await openMenu(tester);
    await tester.tap(find.text('Start over'));
    await tester.pumpAndSettle();
    expect(find.text(resetTitle), findsOneWidget);
    expect(find.text(resetBody(2, 1)), findsOneWidget);
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(resets, isEmpty);
    expect(find.text('candles'), findsOneWidget);

    await openMenu(tester);
    await tester.tap(find.text('Start over'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Start over'));
    await tester.pumpAndSettle();
    expect(resets, [('2026-08-29', '2026-09-04')]);
    expect(find.text('candles'), findsNothing);
    expect(find.text('batteries'), findsOneWidget);
    expect(find.text(countsCopy(2, 1, 0)), findsOneWidget);
  });

  /// The reset write commits before the read that follows it, so a failed read must not leave
  /// the pre-reset list standing as a successful-looking view of storage that no longer
  /// matches it.
  testWidgets('a failed read after Start over does not leave the old list up', (
    tester,
  ) async {
    useTallView(tester);
    var reset = false;
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) {
          if (reset) throw const KimattaError.storage(message: 'db closed');
          return shoppingView(states: [lineStateOf(flourKey, checked: true)]);
        },
        resetShopping: (_, _) async => reset = true,
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.widget<CheckboxListTile>(checkboxFor('flour')).value, isTrue);
    await openMenu(tester);
    await tester.tap(find.text('Start over'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Start over'));
    await tester.pumpAndSettle();
    expect(checkboxFor('flour'), findsNothing);
    // Reported twice on purpose: the screen's error branch and `_startOver`'s snackbar.
    expect(find.text('Shopping unavailable: db closed'), findsWidgets);
    expect(find.text('Try again'), findsOneWidget);
  });

  /// The bridge counts the states it dropped for exactly this: the screen says the edits no
  /// longer apply rather than showing an unchecked list with no explanation.
  testWidgets('states dropped by the window are named, not silently missing', (
    tester,
  ) async {
    useTallView(tester);
    var orphaned = 0;
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(orphaned: orphaned),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.textContaining('no longer has'), findsNothing);

    // Paging cycles is a fresh read, so one harness can produce a window that did drop rows.
    orphaned = 6;
    await tester.tap(find.byTooltip('Next cycle'));
    await tester.pumpAndSettle();
    expect(find.text(orphanedStatesCopy(6)), findsOneWidget);

    orphaned = 1;
    await tester.tap(find.byTooltip('Next cycle'));
    await tester.pumpAndSettle();
    expect(find.text(orphanedStatesCopy(1)), findsOneWidget);
  });

  /// `reset_shopping_list` deletes every row for the window, including the ones the bridge
  /// filtered out of `line_states`, so the confirmation must count those too.
  testWidgets('Start over counts the dropped rows it also deletes', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(
          states: [
            lineStateOf(flourKey, checked: true),
            lineStateOf(mysteryKey, hidden: true),
          ],
          orphaned: 3,
        ),
        resetShopping: (_, _) async {},
      ),
    );
    await tester.pumpAndSettle();
    await openMenu(tester);
    await tester.tap(find.text('Start over'));
    await tester.pumpAndSettle();
    expect(find.text(resetBody(5, 0)), findsOneWidget);
  });

  /// `restored` qualifies a pantry-omitted line and nothing else. Carried onto a line the
  /// derivation now calls Needed it would override a later pantry mark from storage.
  testWidgets('a restored flag is dropped on a line that is merely needed', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <ShoppingLineStateDto>[];
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) =>
            shoppingView(states: [lineStateOf(flourKey, restored: true)]),
        setLineState: (s) {
          sent.add(s);
          return echoLineState(s);
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(checkboxFor('flour'));
    await tester.pumpAndSettle();
    expect(sent.single.key, flourKey);
    expect(sent.single.checked, isTrue);
    expect(
      sent.single.restored,
      isFalse,
      reason: 'the inert flag is dropped rather than carried forward',
    );

    // The line the flag exists for still sets it.
    await tester.tap(find.text('$alreadyHaveHeading (1)'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Add anyway'));
    await tester.pumpAndSettle();
    expect(sent.last.key, onionKey);
    expect(sent.last.restored, isTrue);
  });

  /// A section change is a move, not an edit: the line key does not change when a pantry mark
  /// is added or a line is removed, so the check on it must not be destroyed on the way.
  testWidgets('a check survives Add anyway and a remove/put-back round trip', (
    tester,
  ) async {
    useTallView(tester);
    final sent = <ShoppingLineStateDto>[];
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        // The Already have row renders no checkbox, so this check is invisible before it is
        // destroyed — exactly why "Add anyway" must carry it.
        shopping: (_, _) =>
            shoppingView(states: [lineStateOf(onionKey, checked: true)]),
        setLineState: (s) {
          sent.add(s);
          return echoLineState(s);
        },
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('$alreadyHaveHeading (1)'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Add anyway'));
    await tester.pumpAndSettle();
    expect(sent.single.key, onionKey);
    expect(sent.single.restored, isTrue);
    expect(
      sent.single.checked,
      isTrue,
      reason: 'the stored check is carried, not discarded',
    );
    expect(tester.widget<CheckboxListTile>(checkboxFor('onion')).value, isTrue);

    await tester.tap(checkboxFor('flour'));
    await tester.pumpAndSettle();
    expect(sent.last.checked, isTrue);
    await tester.tap(find.byTooltip('Why is this here?').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Remove from list'));
    await tester.pumpAndSettle();
    expect(sent.last.key, flourKey);
    expect(sent.last.hidden, isTrue);
    expect(sent.last.checked, isTrue);
    await tester.tap(find.text('$removedHeading (1)'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Put back'));
    await tester.pumpAndSettle();
    expect(sent.last.hidden, isFalse);
    expect(sent.last.checked, isTrue);
    expect(tester.widget<CheckboxListTile>(checkboxFor('flour')).value, isTrue);

    // The fourth arm: sending onion back under Already have keeps its check too, so a
    // second "Add anyway" returns it checked rather than blank.
    await tester.tap(find.byTooltip('Why is this here?').at(1));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Back to pantry'));
    await tester.pumpAndSettle();
    expect(sent.last.key, onionKey);
    expect(sent.last.restored, isFalse);
    expect(sent.last.checked, isTrue);
  });

  /// Line keys carry no window bound, so the same key routinely appears in adjacent cycles:
  /// the next cycle's row must not inherit the previous cycle's in-flight write.
  testWidgets('paging cycles mid-write does not freeze the next cycle row', (
    tester,
  ) async {
    useTallView(tester);
    final gate = Completer<ShoppingLineStateDto>();
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(),
        setLineState: (_) => gate.future,
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(checkboxFor('flour'));
    await tester.pump();
    expect(
      tester.widget<CheckboxListTile>(checkboxFor('flour')).onChanged,
      isNull,
    );
    await tester.tap(find.byTooltip('Next cycle'));
    await tester.pumpAndSettle();
    expect(
      tester.widget<CheckboxListTile>(checkboxFor('flour')).onChanged,
      isNotNull,
    );
    // The old cycle's write settling must not unfreeze — or refreeze — this cycle's row.
    gate.complete(lineStateOf(flourKey, checked: true));
    await tester.pumpAndSettle();
    expect(
      tester.widget<CheckboxListTile>(checkboxFor('flour')).onChanged,
      isNotNull,
    );
  });

  /// Rust mints a fresh id for every blank-id save, so nothing deduplicates a repeated submit:
  /// the button carries the guard instead.
  testWidgets('Add item is disabled while the save it started is in flight', (
    tester,
  ) async {
    useTallView(tester);
    final gate = Completer<ShoppingManualItemDto>();
    Finder addButton() => find.widgetWithText(TextButton, 'Add item');
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => shoppingView(),
        saveItem: (_) => gate.future,
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.widget<TextButton>(addButton()).onPressed, isNotNull);
    await tester.tap(find.text('Add item'));
    await tester.pumpAndSettle();
    await tester.enterText(find.widgetWithText(TextField, 'Name'), 'candles');
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(tester.widget<TextButton>(addButton()).onPressed, isNull);
    gate.complete(
      const ShoppingManualItemDto(
        id: 'mi-2',
        householdId: 'h-1',
        name: 'candles',
        checked: false,
      ),
    );
    await tester.pumpAndSettle();
    expect(tester.widget<TextButton>(addButton()).onPressed, isNotNull);
  });

  testWidgets('the shopping screen renders a load failure in prose', (
    tester,
  ) async {
    useTallView(tester);
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) => throw const KimattaError.storage(message: 'boom'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Shopping unavailable: boom'), findsOneWidget);
    expect(find.text('Try again'), findsOneWidget);
    expect(find.byType(CheckboxListTile), findsNothing);
  });

  testWidgets('an empty derivation shows the empty copy', (tester) async {
    useTallView(tester);
    await tester.pumpWidget(harness(initial: '/shopping'));
    await tester.pumpAndSettle();
    expect(find.text(shoppingEmptyCopy), findsOneWidget);
    expect(find.text(countsCopy(0, 0, 0)), findsOneWidget);
    expect(find.text('Add item'), findsOneWidget);
  });

  /// Decision 8 and the reload pin: a pantry toggle re-runs the shopping build, and while
  /// that re-read is in flight the list the user was reading stays on screen.
  testWidgets('a pantry toggle re-reads shopping and a reload keeps the list', (
    tester,
  ) async {
    useTallView(tester);
    var fetches = 0;
    final pending = Completer<ShoppingViewDto>();
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) {
          fetches++;
          return fetches == 1 ? shoppingView() : pending.future;
        },
        setMark: (_, ref, marked) async => pantryEntryMarked(ref, marked),
      ),
    );
    await tester.pumpAndSettle();
    expect(fetches, 1);

    await tester.tap(tab('Pantry'));
    await tester.pumpAndSettle();
    await tester.tap(
      find.ancestor(
        of: find.text('chickpeas'),
        matching: find.byType(SwitchListTile),
      ),
    );
    await tester.pump();
    await tester.tap(tab('Shopping'));
    await tester.pump();
    expect(fetches, 2, reason: 'the pantry toggle re-ran the build');
    expect(find.text('flour'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsNothing);

    pending.complete(shoppingView(items: [batteries]));
    await tester.pumpAndSettle();
    expect(find.text('batteries'), findsOneWidget);
  });

  testWidgets('a planner save re-reads shopping', (tester) async {
    useTallView(tester);
    var fetches = 0;
    await tester.pumpWidget(
      harness(
        initial: '/shopping',
        shopping: (_, _) {
          fetches++;
          return shoppingView();
        },
        savePlanned: (meal) async => meal,
      ),
    );
    await tester.pumpAndSettle();
    expect(fetches, 1);
    final container = ProviderScope.containerOf(
      tester.element(find.byType(NavigationBar)),
    );
    await container.read(plannerProvider(0).notifier).save(okPlanned);
    await tester.pumpAndSettle();
    expect(fetches, 2);
    expect(find.text('flour'), findsOneWidget);
  });

  /// AC-4's accessible half, the pantry precedent: every row's name states the meaning
  /// before the platform's own state, and each explain button is its own labelled node.
  testWidgets('the shopping screen meets the accessibility guidelines', (
    tester,
  ) async {
    usePixel5(tester);
    final handle = tester.ensureSemantics();
    for (final brightness in Brightness.values) {
      tester.platformDispatcher.platformBrightnessTestValue = brightness;
      await tester.pumpWidget(
        harness(
          initial: '/shopping',
          shopping: (_, _) => shoppingView(
            states: [lineStateOf(flourKey, checked: true)],
            items: [batteries],
          ),
        ),
      );
      await tester.pumpAndSettle();
      await expectLater(tester, meetsGuideline(androidTapTargetGuideline));
      await expectLater(tester, meetsGuideline(labeledTapTargetGuideline));
      await expectLater(tester, meetsGuideline(textContrastGuideline));
      expect(find.bySemanticsLabel(lineLabel('flour', true)), findsOneWidget);
      expect(
        find.bySemanticsLabel(lineLabel('mystery', false)),
        findsOneWidget,
      );
      expect(
        find.bySemanticsLabel(lineLabel('batteries', false)),
        findsOneWidget,
      );
      // One explain node per Needed row, outside the tile's merged node.
      expect(find.byTooltip('Why is this here?'), findsNWidgets(2));
    }
    handle.dispose();
  });

  testWidgets('Export reports where the export landed', (tester) async {
    usePixel5(tester);
    final backup = _FakeBackupActions();
    await tester.pumpWidget(harness(initial: '/settings', backup: backup));
    await tester.pumpAndSettle();
    await tester.tap(find.text(exportButtonLabel));
    await tester.pumpAndSettle();
    expect(backup.exports, 1);
    expect(find.text(exportedCopy(_fakeExportPath)), findsOneWidget);
    // The share sheet is offered with the file that was just written.
    expect(backup.shared, [_fakeExportPath]);
  });

  testWidgets('Restore names the export, restores and refreshes', (
    tester,
  ) async {
    usePixel5(tester);
    var healthFetches = 0;
    final backup = _FakeBackupActions(latest: _fakeExportPath);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () {
          healthFetches++;
          return okReport;
        },
        backup: backup,
      ),
    );
    await tester.pumpAndSettle();
    expect(healthFetches, 1);
    await tester.tap(find.text(restoreButtonLabel));
    await tester.pumpAndSettle();
    expect(
      find.text(restoreConfirmBody('kimatta-export-20260902-010203.db')),
      findsOneWidget,
    );
    await tester.tap(find.text(restoreConfirmAction));
    await tester.pumpAndSettle();
    expect(backup.restores, [_fakeExportPath]);
    // The invalidation set ran: the health provider re-fetched even though the
    // household id is unchanged — nothing cascades it.
    expect(healthFetches, 2);
    expect(find.text(restoredCopy), findsOneWidget);
  });

  testWidgets('Restore with no exports says so and restores nothing', (
    tester,
  ) async {
    usePixel5(tester);
    final backup = _FakeBackupActions();
    await tester.pumpWidget(harness(initial: '/settings', backup: backup));
    await tester.pumpAndSettle();
    await tester.tap(find.text(restoreButtonLabel));
    await tester.pumpAndSettle();
    expect(find.text(noExportsCopy), findsOneWidget);
    expect(backup.restores, isEmpty);
  });

  // Adversarial: a non-corrupt failure must not offer start-fresh — trading
  // recoverable data for nothing on, say, a disk-full error.
  testWidgets('a non-corrupt failure offers retry only', (tester) async {
    usePixel5(tester);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () => throw const KimattaError.storage(message: 'disk full'),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text(tryAgainLabel), findsOneWidget);
    expect(find.text(startFreshLabel), findsNothing);
  });

  testWidgets('a corrupt database offers retry and a confirmed start fresh', (
    tester,
  ) async {
    usePixel5(tester);
    var healthFetches = 0;
    final backup = _FakeBackupActions();
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () {
          healthFetches++;
          if (healthFetches == 1) {
            throw const KimattaError.corrupt(message: 'file is not a database');
          }
          return okReport;
        },
        backup: backup,
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text(tryAgainLabel), findsOneWidget);
    // The recovery buttons sit below the diagnostics tile, off the Pixel-5
    // viewport; an off-screen tap silently misses.
    await tester.ensureVisible(find.text(startFreshLabel));
    await tester.pumpAndSettle();
    await tester.tap(find.text(startFreshLabel));
    await tester.pumpAndSettle();
    expect(find.text(startFreshConfirmBody), findsOneWidget);
    await tester.tap(find.text(startFreshConfirmAction));
    await tester.pumpAndSettle();
    expect(backup.freshes, 1);
    // The reset invalidated the health provider, which now reports the fresh db.
    expect(healthFetches, 2);
    expect(find.textContaining('schema v5'), findsOneWidget);
  });

  // The confirmation dialogs are the only guard between a stray tap and an irreversible
  // replacement, so the refusing branch is tested as deliberately as the confirming one.
  testWidgets('Restore cancelled leaves the database alone', (tester) async {
    usePixel5(tester);
    var healthFetches = 0;
    final backup = _FakeBackupActions(latest: _fakeExportPath);
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () {
          healthFetches++;
          return okReport;
        },
        backup: backup,
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text(restoreButtonLabel));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(TextButton, 'Cancel'));
    await tester.pumpAndSettle();
    expect(backup.restores, isEmpty);
    expect(find.text(restoredCopy), findsNothing);
    // The invalidation set did not run either: a refused restore swapped nothing.
    expect(healthFetches, 1);
  });

  testWidgets('Start fresh cancelled leaves the database alone', (
    tester,
  ) async {
    usePixel5(tester);
    final backup = _FakeBackupActions();
    await tester.pumpWidget(
      harness(
        initial: '/settings',
        health: () =>
            throw const KimattaError.corrupt(message: 'file is not a database'),
        backup: backup,
      ),
    );
    await tester.pumpAndSettle();
    // The recovery buttons sit below the diagnostics tile, off the Pixel-5
    // viewport; an off-screen tap silently misses.
    await tester.ensureVisible(find.text(startFreshLabel));
    await tester.pumpAndSettle();
    await tester.tap(find.text(startFreshLabel));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(TextButton, 'Cancel'));
    await tester.pumpAndSettle();
    expect(backup.freshes, 0);
    expect(find.text(startedFreshCopy), findsNothing);
  });

  /// A dismissal is a refusal, not an unanswered question: `showDialog` completes with
  /// `null` when the barrier is tapped, and that must not read as consent.
  testWidgets('dismissing the restore dialog counts as a refusal', (
    tester,
  ) async {
    usePixel5(tester);
    final backup = _FakeBackupActions(latest: _fakeExportPath);
    await tester.pumpWidget(harness(initial: '/settings', backup: backup));
    await tester.pumpAndSettle();
    await tester.tap(find.text(restoreButtonLabel));
    await tester.pumpAndSettle();
    expect(find.byType(AlertDialog), findsOneWidget);
    await tester.tapAt(const Offset(10, 10));
    await tester.pumpAndSettle();
    expect(find.byType(AlertDialog), findsNothing);
    expect(backup.restores, isEmpty);
    expect(find.text(restoredCopy), findsNothing);
  });

  // The three destructive `catch` arms had no coverage at all: every backup test drove the
  // success path, so the failure branches — the only place a user is told a destructive
  // operation did not happen — were never executed. These three go here, above the
  // must-be-last test below.
  testWidgets(
    'a failed restore reports it and re-fetches the health provider',
    (tester) async {
      usePixel5(tester);
      var healthFetches = 0;
      const failure = KimattaError.storage(message: 'disk full');
      final backup = _FakeBackupActions(
        latest: _fakeExportPath,
        onRestore: () async => throw failure,
      );
      await tester.pumpWidget(
        harness(
          initial: '/settings',
          health: () {
            healthFetches++;
            // The restore emptied the connection slot and `recover_original` did not refill
            // it: the second read of the live database fails where the first succeeded.
            if (healthFetches > 1) {
              throw const KimattaError.notOpen();
            }
            return okReport;
          },
          backup: backup,
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text(restoreButtonLabel));
      await tester.pumpAndSettle();
      await tester.tap(find.text(restoreConfirmAction));
      await tester.pumpAndSettle();

      expect(backup.restores, [_fakeExportPath]);
      expect(
        find.text(describeFailure(failure, subject: 'Restore')),
        findsOneWidget,
      );
      // The invalidation runs on failure too, so the diagnostics tile stops advertising a
      // database the app can no longer reach and the recovery row appears with it.
      expect(healthFetches, 2);
      expect(find.text(tryAgainLabel), findsOneWidget);
    },
  );

  testWidgets(
    'a failed start fresh reports it and re-fetches the health provider',
    (tester) async {
      usePixel5(tester);
      var healthFetches = 0;
      const failure = KimattaError.storage(message: 'disk full');
      final backup = _FakeBackupActions(
        onStartFresh: () async => throw failure,
      );
      await tester.pumpWidget(
        harness(
          initial: '/settings',
          health: () {
            healthFetches++;
            throw const KimattaError.corrupt(message: 'file is not a database');
          },
          backup: backup,
        ),
      );
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.text(startFreshLabel));
      await tester.pumpAndSettle();
      await tester.tap(find.text(startFreshLabel));
      await tester.pumpAndSettle();
      await tester.tap(find.text(startFreshConfirmAction));
      await tester.pumpAndSettle();

      expect(backup.freshes, 1);
      expect(
        find.text(describeFailure(failure, subject: 'Start fresh')),
        findsOneWidget,
      );
      expect(healthFetches, 2);
    },
  );

  // The export succeeded and was reported before the share sheet was offered, so a share
  // that fails is not a data failure and must not overwrite that message.
  testWidgets('a failed share leaves the export success message standing', (
    tester,
  ) async {
    usePixel5(tester);
    final backup = _FakeBackupActions(
      onShareExport: () async => throw Exception('no share sheet'),
    );
    await tester.pumpWidget(harness(initial: '/settings', backup: backup));
    await tester.pumpAndSettle();
    await tester.tap(find.text(exportButtonLabel));
    await tester.pumpAndSettle();

    expect(backup.shared, [_fakeExportPath]);
    expect(find.text(exportedCopy(_fakeExportPath)), findsOneWidget);
    expect(find.textContaining('Export failed'), findsNothing);
  });

  // Only one `.pre-restore` generation is kept, so a second confirmed restore would move the
  // first restore's result into it and destroy the database the user started with. The modal
  // dialog is no guard: it closes as soon as it is answered.
  testWidgets('a restore in flight disables the destructive buttons', (
    tester,
  ) async {
    usePixel5(tester);
    final pending = Completer<HealthReport>();
    final backup = _FakeBackupActions(
      latest: _fakeExportPath,
      onRestore: () => pending.future,
    );
    await tester.pumpWidget(harness(initial: '/settings', backup: backup));
    await tester.pumpAndSettle();
    await tester.tap(find.text(restoreButtonLabel));
    await tester.pumpAndSettle();
    await tester.tap(find.text(restoreConfirmAction));
    // Not `pumpAndSettle`: the restore is deliberately still in flight.
    await tester.pump();

    expect(
      tester
          .widget<FilledButton>(
            find.widgetWithText(FilledButton, restoreButtonLabel),
          )
          .onPressed,
      isNull,
      reason: 'the button is disabled while the swap is in flight',
    );
    await tester.tap(find.text(restoreButtonLabel));
    await tester.pump();
    expect(backup.restores, [_fakeExportPath], reason: 'no second restore');

    pending.complete(okReport);
    await tester.pumpAndSettle();
    expect(find.text(restoredCopy), findsOneWidget);
    expect(
      tester
          .widget<FilledButton>(
            find.widgetWithText(FilledButton, restoreButtonLabel),
          )
          .onPressed,
      isNotNull,
      reason: 'and enabled again once it finishes',
    );
  });

  // Keep this test last: the assertion it provokes leaves the element tree
  // half-updated, and every test pumped after it in the same file fails on a
  // framework "dependent is not our descendant" assertion (measured).
  testWidgets('changing initialLocation on a live App element is rejected', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness(initial: '/shopping'));
    await tester.pumpAndSettle();

    // `App` carries no key, so this updates the existing element rather than
    // building a fresh one: `_router` is already built and the new location
    // would otherwise be dropped in silence. The assertion is `App`'s and does not depend
    // on the route shown, but the half-updated tree it leaves makes a focus scope report a
    // second framework assertion when the tree is torn down, and a test that reports two
    // exceptions in one frame fails regardless of `takeException`. MVP-013 dodged that by
    // provoking it from the then-inert Shopping branch; with Shopping live (MVP-016) no
    // route is inert, so the tree is unmounted here, in its own frame, and that teardown
    // assertion is taken separately from the one under test.
    await tester.pumpWidget(harness(initial: '/nope'));
    expect(tester.takeException(), isA<AssertionError>());
    await tester.pumpWidget(const SizedBox.shrink());
    tester.takeException();
  });
}

const _fakeExportPath = '/x/exports/kimatta-export-20260902-010203.db';

/// Same seam as the recipe fakes: each call is recorded and then handed to an optional hook,
/// so a test can make the destructive operations fail (or hang) without the bridge. Recording
/// happens before the hook runs, so a throwing hook still proves the call was made.
class _FakeBackupActions extends BackupActions {
  _FakeBackupActions({
    this.latest,
    this.onRestore,
    this.onStartFresh,
    this.onShareExport,
  });

  final String? latest;
  final Future<HealthReport> Function()? onRestore;
  final Future<HealthReport> Function()? onStartFresh;
  final Future<void> Function()? onShareExport;
  int exports = 0;
  final List<String> shared = [];
  final List<String> restores = [];
  int freshes = 0;

  @override
  Future<ExportReport> export() async {
    exports++;
    return ExportReport(path: _fakeExportPath, schemaVersion: 5);
  }

  @override
  Future<void> shareExport(String path) async {
    shared.add(path);
    await (onShareExport ?? () async {})();
  }

  @override
  Future<String?> latestExport() async => latest;

  @override
  Future<HealthReport> restore(String exportPath) async {
    restores.add(exportPath);
    return (onRestore ?? () async => okReport)();
  }

  @override
  Future<HealthReport> startFresh() async {
    freshes++;
    return (onStartFresh ?? () async => okReport)();
  }
}
