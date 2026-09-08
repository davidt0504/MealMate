import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/appearance_provider.dart';
import 'package:meal_mate/app/router.dart';
import 'package:meal_mate/app/theme.dart';
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/recipes/starter_provider.dart';
import 'package:meal_mate/features/settings/backup_provider.dart';
import 'package:meal_mate/src/rust/api/household.dart';

class App extends ConsumerStatefulWidget {
  const App({super.key, this.initialLocation = homeLocation});

  /// Read once, when `_router` is first built. Changing it on a later rebuild
  /// of the same element does nothing; `didUpdateWidget` asserts against that.
  final String initialLocation;

  @override
  ConsumerState<App> createState() => _AppState();
}

class _AppState extends ConsumerState<App> {
  static const _startupTimeout = Duration(seconds: 10);

  bool _startupHandled = false;
  late bool _startupSettled;

  Future<void>? _firstRunCompletion;

  /// `_AppState`'s own context sits *above* `MaterialApp.router`, so
  /// `ScaffoldMessenger.of(context)` there throws `No ScaffoldMessenger widget found` — every
  /// other `describeFailure` snackbar in the app is raised from a screen below it. The key is
  /// how a failure raised at this level reaches a surface at all.
  final _messengerKey = GlobalKey<ScaffoldMessengerState>();

  late final GoRouter _router = buildRouter(
    initialLocation: widget.initialLocation,
    completeFirstRun: _completeFirstRun,
  );

  @override
  void initState() {
    super.initState();
    // Only the ordinary launch can flash the empty Plan grid before the first-run decision.
    // Explicit/restored routes keep their own loading and error surfaces reachable.
    _startupSettled = widget.initialLocation != homeLocation;
  }

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
    final appearance = ref.watch(appearanceProvider);
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
    // The same subscription carries the first-run gate. Until the household resolves, the
    // router stays mounted but offstage behind a neutral startup surface. A new household waits
    // for starter content before Cover can read it; an existing household is revealed at once
    // and receives the same once-per-database-generation install in the background. A household
    // error reveals Plan's in-shell error arm, where it can be explained without trapping
    // navigation.
    ref.listen(databaseGenerationProvider, (_, _) {
      _startupHandled = false;
      _firstRunCompletion = null;
      if (mounted && _startupSettled) {
        setState(() => _startupSettled = false);
      }
    });
    ref.listen(householdProvider, (_, next) {
      switch (next) {
        case AsyncError():
          // Reveal the route's own explanation without consuming the success transition. A
          // later health/database retry must still install and route a recovered first run.
          if (!_startupHandled) _settleStartup();
        case AsyncData(:final value):
          if (_startupHandled) return;
          _startupHandled = true;
          // After an ordinary-launch error, cover the empty Plan again while the recovered new
          // household waits for starters. Explicit initial routes retain their own surfaces.
          if (!value.onboarded &&
              widget.initialLocation == homeLocation &&
              _startupSettled &&
              mounted) {
            setState(() => _startupSettled = false);
          }
          unawaited(_handleStartup(value));
        case AsyncLoading():
          // Invalidation can carry the previous database's value through loading. It is stale
          // by definition and must not consume this generation's startup transition.
          return;
      }
    });
    return MaterialApp.router(
      title: 'Kimatta (dev)',
      restorationScopeId: 'app',
      scaffoldMessengerKey: _messengerKey,
      theme: buildTheme(appearance.palette, appearance.type, Brightness.light),
      darkTheme: buildTheme(
        appearance.palette,
        appearance.type,
        Brightness.dark,
      ),
      themeMode: ThemeMode.system,
      routerConfig: _router,
      builder: (context, child) => Stack(
        children: [
          Positioned.fill(
            child: Offstage(
              offstage: !_startupSettled,
              child: child ?? const SizedBox.shrink(),
            ),
          ),
          if (!_startupSettled)
            Positioned.fill(
              child: ColoredBox(
                color: Theme.of(context).scaffoldBackgroundColor,
                child: const ExcludeSemantics(
                  child: Center(child: CircularProgressIndicator()),
                ),
              ),
            ),
        ],
      ),
    );
  }

  Future<void> _handleStartup(HouseholdDto household) async {
    final install = _installStarterContent(household.id);
    if (household.onboarded) {
      _settleStartup();
      final failure = await install;
      if (failure != null) _reportFailure(failure, subject: 'Starter recipes');
      return;
    }

    final failure = await install.timeout(
      _startupTimeout,
      onTimeout: () => _StartupTimeout(
        'installation took longer than ${_startupTimeout.inSeconds} seconds',
      ),
    );
    if (!mounted) return;
    _router.go('/plan/cover');
    _settleStartup(failure: failure, subject: 'Starter recipes');
  }

  void _settleStartup({Object? failure, String? subject}) {
    if (!mounted) return;
    if (!_startupSettled) setState(() => _startupSettled = true);
    if (failure != null && subject != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _reportFailure(failure, subject: subject);
      });
    }
  }

  /// One shared future per database generation closes the race between Accept and leaving through
  /// the shell while the bridge write is in flight. A stuck local write must not turn Cover into
  /// a trap.
  Future<void> _completeFirstRun() =>
      _firstRunCompletion ??= _completeFirstRunOnce();

  Future<void> _completeFirstRunOnce() async {
    final household = ref.read(householdProvider).valueOrNull;
    if (household == null || household.onboarded) return;
    try {
      await ref
          .read(householdProvider.notifier)
          .completeOnboarding(household.id)
          .timeout(
            _startupTimeout,
            onTimeout: () {
              throw _StartupTimeout(
                'saving setup took longer than ${_startupTimeout.inSeconds} seconds',
              );
            },
          );
    } catch (error) {
      _reportFailure(error, subject: 'Setup');
    }
  }

  void _reportFailure(Object error, {required String subject}) {
    if (!mounted) return;
    _messengerKey.currentState?.showSnackBar(
      SnackBar(content: Text(describeFailure(error, subject: subject))),
    );
  }

  /// Awaited, with a bound, only for a new household; later launches keep the non-blocking shape
  /// MVP-006 pinned. The steady-state short-circuit is cheap and writes no rows.
  Future<Object?> _installStarterContent(String householdId) async {
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
      return null;
    } catch (error) {
      return error;
    }
  }
}

class _StartupTimeout implements Exception {
  const _StartupTimeout(this.message);

  final String message;

  @override
  String toString() => message;
}
