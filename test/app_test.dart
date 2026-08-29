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
import 'package:meal_mate/features/restrictions/restrictions_provider.dart';
import 'package:meal_mate/features/restrictions/restrictions_screen.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/restrictions.dart';

const okReport = HealthReport(dbPath: '/x/kimatta.db', schemaVersion: 4);
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
}) => ProviderScope(
  overrides: [
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
    expect(find.textContaining('schema v4 at /x/kimatta.db'), findsOneWidget);
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
    expect(find.textContaining('schema v4 at /x/kimatta.db'), findsOneWidget);
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
