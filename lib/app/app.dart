import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/router.dart';
import 'package:meal_mate/app/theme.dart';
import 'package:meal_mate/features/settings/health_provider.dart';

class App extends ConsumerStatefulWidget {
  const App({super.key, this.initialLocation = homeLocation});

  /// Read once, when `_router` is first built. Changing it on a later rebuild
  /// of the same element does nothing; `didUpdateWidget` asserts against that.
  final String initialLocation;

  @override
  ConsumerState<App> createState() => _AppState();
}

class _AppState extends ConsumerState<App> {
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
    // Holds healthReportProvider for the app's lifetime, so the DB is opened and
    // migrated exactly once. `ref.read` registers no listener: under Riverpod 3,
    // which auto-disposes by default, the provider would be disposed right after
    // a startup read and re-created by SettingsScreen's `ref.watch` — a second
    // open/migrate, and the ground the KNOWN_ISSUES.md connection-ownership
    // MEDIUM was deferred on ("exactly one storage-touching call") would be gone.
    // `ref.listen` subscribes without rebuilding, and behaves the same on 2.x.
    ref.listen(healthReportProvider, (_, _) {});
    return MaterialApp.router(
      title: 'MealMate (dev)',
      restorationScopeId: 'app',
      theme: lightTheme,
      darkTheme: darkTheme,
      routerConfig: _router,
    );
  }
}
