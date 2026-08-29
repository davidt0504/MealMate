import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/router.dart';
import 'package:meal_mate/app/theme.dart';
import 'package:meal_mate/features/household/household_provider.dart';

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
    ref.listen(householdProvider, (_, next) {
      if (_gated) return;
      final value = next.valueOrNull;
      if (value == null) return;
      _gated = true;
      if (!value.onboarded) _router.go(welcomeLocation);
    });
    return MaterialApp.router(
      title: 'MealMate (dev)',
      restorationScopeId: 'app',
      theme: lightTheme,
      darkTheme: darkTheme,
      routerConfig: _router,
    );
  }
}
