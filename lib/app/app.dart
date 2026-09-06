import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/router.dart';
import 'package:meal_mate/app/theme.dart';
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/recipes/starter_provider.dart';

class App extends ConsumerStatefulWidget {
  const App({super.key, this.initialLocation = homeLocation});

  /// Read once, when `_router` is first built. Changing it on a later rebuild
  /// of the same element does nothing; `didUpdateWidget` asserts against that.
  final String initialLocation;

  @override
  ConsumerState<App> createState() => _AppState();
}

class _AppState extends ConsumerState<App> {
  bool _gated = false;

  /// Sibling of `_gated`, and needed for the same reason: `HouseholdNotifier.rename` and
  /// `.completeOnboarding` both publish `AsyncData`, so the listener fires several times per
  /// launch. Without this, every Save-name in Settings would re-run the install.
  bool _installStarted = false;

  /// `_AppState`'s own context sits *above* `MaterialApp.router`, so
  /// `ScaffoldMessenger.of(context)` there throws `No ScaffoldMessenger widget found` — every
  /// other `describeFailure` snackbar in the app is raised from a screen below it. The key is
  /// how a failure raised at this level reaches a surface at all.
  final _messengerKey = GlobalKey<ScaffoldMessengerState>();

  late final GoRouter _router = buildRouter(
    initialLocation: widget.initialLocation,
  );

  @override
  void didUpdateWidget(App oldWidget) {
    super.didUpdateWidget(oldWidget);
    assert(
      oldWidget.initialLocation == widget.initialLocation,
      'initialLocation is read once at first build. Give App a unique Key to '
      'force a fresh router instead of changing it on an existing element.',
    );
  }

  @override
  void dispose() {
    _router.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // Holds householdProvider — and through it healthReportProvider — for the
    // app's lifetime, so the DB is opened, migrated and bootstrapped exactly
    // once. `ref.read` registers no listener: under a Riverpod major that
    // auto-disposes by default, the provider would be disposed right after
    // a startup read and re-created by SettingsScreen's `ref.watch` — a second
    // open/migrate, and the ground the KNOWN_ISSUES.md connection-ownership
    // MEDIUM was deferred on ("exactly one storage-touching call") would be gone.
    // `pubspec.yaml` pins `flutter_riverpod: ^2.6.1` and the resolved major is 2,
    // where `AsyncNotifierProvider` does not auto-dispose, so that is a
    // forward-looking reason rather than a current one; the conclusion — App holds
    // the provider so the database opens once — stands either way.
    // `ref.listen` subscribes without rebuilding.
    //
    // The same subscription now also carries the first-run gate. The router is
    // still built immediately at `initialLocation`, so Settings' own loading arms
    // stay reachable; a genuine first launch shows Plan for the frames before the
    // DTO arrives and then goes to Welcome. An error deliberately does not route
    // here — `complete_onboarding` would fail too, and Settings is where the
    // failure is explained.
    // The starter install runs here rather than at onboarding, and the distinction is
    // load-bearing: `welcomeLocation` is reachable only while `onboarded == false`, so an
    // install wired to Welcome is a one-shot that no already-onboarded device — the owner's
    // emulator, every dev and beta build — could ever reach again. Those devices would never
    // get the ingredient catalog, and a cook review recorded later could never install
    // anywhere. (`WelcomeScreen._finish` is also the single handler behind *both* Welcome
    // buttons, so "the Get started handler" is not even a well-defined site.)
    //
    // It is latched separately from `_gated` so neither gate can swallow the other.
    ref.listen(householdProvider, (_, next) {
      final value = next.valueOrNull;
      if (value == null) return;
      if (!_installStarted) {
        _installStarted = true;
        _installStarterContent(value.id);
      }
      if (_gated) return;
      _gated = true;
      if (!value.onboarded) _router.go(welcomeLocation);
    });
    return MaterialApp.router(
      title: 'Kimatta (dev)',
      restorationScopeId: 'app',
      scaffoldMessengerKey: _messengerKey,
      theme: lightTheme,
      darkTheme: darkTheme,
      routerConfig: _router,
    );
  }

  /// Non-blocking and never gates navigation — the shape MVP-006 pinned with "a failed
  /// completion still lets the user in". Step 6's read-only short-circuit makes the
  /// steady-state call one `SELECT` for the household and two more for the ids and slugs,
  /// writing no rows, so this is cheap to run on every launch.
  Future<void> _installStarterContent(String householdId) async {
    try {
      final report = await ref.read(starterInstallProvider)(householdId);
      // `catalogInstalled` is the size of the catalog the install *wrote*, not a count of new
      // rows: `install_starter_content` reports `catalog.len()` whenever it takes its write
      // branch and 0 when it short-circuits. So `> 0` reads as "the ingredient table was
      // rewritten this run" — the condition under which `pantryProvider`'s once-per-launch
      // list can be missing identities. Deliberately a superset: a launch that installs a new
      // starter *recipe* rewrites the catalog too and costs one redundant read. Steady-state
      // launches short-circuit and cost nothing, which is the case that matters.
      //
      // `mounted` because this future is deliberately unawaited — the element can be gone by
      // the time the install lands, and `ref` would throw. The `catch` below dodges the same
      // hazard with `_messengerKey.currentState?.`. `invalidate` rather than the notifier's
      // `refresh()` precisely because it must not instantiate a provider nothing has read.
      if (mounted && report.catalogInstalled > 0) {
        ref.invalidate(pantryProvider);
      }
      // Same hazard, the recipe half. Until MVP-032 `installed` was always 0, so a Recipes tab
      // opened before this unawaited future landed could not be stale. Now it can:
      // `RecipeLibraryNotifier.build` depends only on the household and its restrictions, so
      // nothing else re-reads it, and a first launch would otherwise cache an empty library —
      // which is exactly what MVP-032 AC-5 asks to see filled.
      if (mounted && report.installed > 0) {
        ref.invalidate(recipeLibraryProvider);
      }
    } catch (error) {
      _messengerKey.currentState?.showSnackBar(
        SnackBar(
          content: Text(describeFailure(error, subject: 'Starter recipes')),
        ),
      );
    }
  }
}
