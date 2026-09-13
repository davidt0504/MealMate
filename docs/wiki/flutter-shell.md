# Flutter shell

## Responsibility

The Flutter shell initializes the Rust runtime, creates the app-scoped provider container, owns the router, selects themes, and presents the persistent navigation structure. It contains no durable food model.

## Startup and app lifetime

`main` calls `WidgetsFlutterBinding.ensureInitialized`, awaits `RustLib.init`, and mounts `ProviderScope(child: App())`. Native initialization therefore happens before widgets exist; later database failures are represented through providers and settings surfaces.

`App` is a `ConsumerStatefulWidget` because it owns a single `GoRouter` for the element lifetime and two launch latches. Its household listener routes a not-yet-onboarded household to Welcome exactly once. A separate latch starts starter-content installation for every resolved household, including devices that were onboarded before new starter data shipped. Installation is non-blocking and reports failure through the scaffold messenger rather than gating navigation.

Holding and listening to `householdProvider` at app scope also keeps the database bootstrap chain alive. Renames and onboarding completion republish household DTOs, but the latches prevent either operation from replaying startup side effects.

## Navigation

`buildRouter` defines named locations for the onboarding screen, main shell branches, planning cycle editor, Cover My Week, recipe create/detail/edit, household settings, and restriction settings. `coverOffset` validates the cycle-offset query parameter and falls back to zero for malformed values.

The stateful shell presents the primary Plan, Recipes, Pantry, Shop, and Settings destinations. Detail/editor routes sit outside or above branch roots as appropriate, while each feature owns its own async loading and failure arms. `App.initialLocation` is a test seam and is intentionally read only when the router is created.

## Presentation frame

`theme.dart` defines the Material light and dark themes. `placeholder_screen.dart` is a small reusable empty/deferred surface. The shell owns navigation affordances but delegates domain copy and state transitions to feature modules.

## Native build hook

`hook/build.dart` is the Flutter native-assets entry point and delegates to `flutter_rust_bridge_hooks`. It makes Rust compilation part of Flutter's build graph; it is not an application runtime module.

## Failure boundaries

App-level starter installation uses a `GlobalKey<ScaffoldMessengerState>` because the `App` state's context is above `MaterialApp.router`. Database-open and domain failures are rendered by feature/provider states. Native library initialization itself remains a pre-widget failure boundary.

## Coverage evidence

This page owns seven hand-written Dart files: the entry point, five app-shell files, and the native build hook. Exact fingerprints and symbols are in `../coverage-manifest.json`.

