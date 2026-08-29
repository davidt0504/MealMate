import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/app/app.dart';
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/settings/health_provider.dart';
import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/api/household.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/recipes/restriction_warnings.dart';
import 'package:meal_mate/features/restrictions/restrictions_provider.dart';
import 'package:meal_mate/features/restrictions/restriction_copy.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/restrictions.dart';
import 'package:meal_mate/src/rust/api/starter.dart';
import 'package:meal_mate/features/recipes/starter_provider.dart';

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
}) => ProviderScope(
  overrides: [
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
  ],
  child: App(initialLocation: initial),
);

const labels = ['Recipes', 'Plan', 'Pantry', 'Shopping', 'Settings'];

/// Upper bound for the traversal test. The bar is entered one press after the
/// last focusable ahead of it — two presses today, since Plan contributes only
/// `Cover My Week`. 12 leaves room for a real planner screen's controls before
/// the traversal has diverged far enough that failing is the right answer.
const maxTabPresses = 12;

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
    expect(find.textContaining('MVP-024'), findsOneWidget);

    await tester.tap(find.byType(BackButton));
    await tester.pumpAndSettle();
    expect(title('Plan'), findsOneWidget);
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
    // exercises is the placeholder bodies and AppBar titles at a genuine 2.0.
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
    // bar depends on the Plan screen's tree, which MVP-013 replaces. What is
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

  // Keep this test last: the assertion it provokes leaves the element tree
  // half-updated, and every test pumped after it in the same file fails on a
  // framework "dependent is not our descendant" assertion (measured).
  testWidgets('changing initialLocation on a live App element is rejected', (
    tester,
  ) async {
    usePixel5(tester);
    await tester.pumpWidget(harness());
    await tester.pumpAndSettle();

    // `App` carries no key, so this updates the existing element rather than
    // building a fresh one: `_router` is already built and the new location
    // would otherwise be dropped in silence.
    await tester.pumpWidget(harness(initial: '/nope'));
    expect(tester.takeException(), isA<AssertionError>());
  });
}
