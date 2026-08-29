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

const okReport = HealthReport(dbPath: '/x/kimatta.db', schemaVersion: 1);
const okHousehold = HouseholdDto(
  id: 'h-1',
  name: null,
  members: [MemberDto(id: 'm-1', displayName: 'Me')],
);
const twoMemberHousehold = HouseholdDto(
  id: 'h-2',
  name: 'Casa',
  members: [
    MemberDto(id: 'm-1', displayName: 'Me'),
    MemberDto(id: 'm-2', displayName: 'Ada'),
  ],
);

/// Both providers are always overridden, which is why `flutter test` never
/// loads the native library: no test reaches `openDatabase` or
/// `bootstrapHousehold`. The save path goes through `HouseholdNotifier.rename`,
/// so `rename:` fakes it at the same seam; a test that taps **Save name**
/// without supplying one fails loudly rather than reaching the real bridge.
/// `bridge_native_test.dart` is what proves the rename write itself.
class _FakeHouseholdNotifier extends HouseholdNotifier {
  _FakeHouseholdNotifier(this._build, this._rename);

  final FutureOr<HouseholdDto> Function()? _build;
  final Future<HouseholdDto> Function(String, String?)? _rename;

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

Widget harness({
  String initial = '/plan',
  FutureOr<HealthReport> Function()? health,
  FutureOr<HouseholdDto> Function()? household,
  Future<HouseholdDto> Function(String, String?)? rename,
}) => ProviderScope(
  overrides: [
    healthReportProvider.overrideWith((_) => (health ?? () => okReport)()),
    householdProvider.overrideWith(
      () => _FakeHouseholdNotifier(household, rename),
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
    expect(find.textContaining('schema v1 at /x/kimatta.db'), findsOneWidget);
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
    expect(find.textContaining('schema v1 at /x/kimatta.db'), findsOneWidget);
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
