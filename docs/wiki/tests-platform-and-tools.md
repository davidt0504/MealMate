# Tests, platform, and tools

## Test architecture

Flutter tests cover three levels. Small copy/format tests pin wording, quantities, civil dates, and restriction summaries. Provider and widget seams replace bridge calls while exercising notifier merge/refresh behavior and screen states. `bridge_native_test.dart` loads the real native library against temporary databases, testing the generated boundary and durable behavior rather than a Dart imitation.

`app_test.dart` is the broad UI regression suite. It drives navigation, onboarding, recipes, planner, Cover My Week, pantry, shopping, settings, async failures, and accessibility-sized layouts through injectable providers and fakes. Rust tests live beside their modules and therefore appear on the domain/storage pages rather than under `test/`.

The important two-language gate is sequential: build/test current Rust first, then run Flutter tests against that native artifact. Binding regeneration is a separate freshness obligation after bridge API changes.

## Android host and packaging

Android is the only checked-in platform host. `MainActivity` is a minimal `FlutterActivity`; application logic stays in Flutter/Rust. Kotlin DSL config wires Flutter, the temporary `dev.mealmate.temp` application ID, SDK levels, ABI packaging, and current internal release signing.

The main manifest names the app and launch activity, declares no Internet permission, disables platform backup, and uses explicit data-extraction rules. Debug/profile manifests provide development allowances. Styles and launch-background XML define splash behavior. Launcher PNG density variants and adaptive-icon XML are asset families, not executable source.

## Release automation

`.github/workflows/release-apk.yml` builds and publishes an Android APK for app-code pushes to master, version tags, or manual workflow dispatch. It prepares Flutter/Rust/Android toolchains, runs the Rust build and Flutter tests, installs the shared debug signing key from a secret, builds the universal release artifact and checks its signing certificate in a read-only build job; a separate publish job releases it as a `build-<run number>` (or tag-named) GitHub release. This is an internal distribution path; production signing and store activation remain separately gated.

## Device and emulator tools

`tools/emulator.sh` coordinates WSL with the Windows-host Android SDK/adb server, starts or stops the configured emulator, checks boot readiness, proxies adb, and resets app data. It distinguishes a connected physical device from an emulator before launch, but callers with multiple devices may need an explicit serial.

`tools/device.sh` builds, installs, launches, logs, clears, or exports the APK for a selected physical device. `tools/windows/setup-adb-bridge.ps1` configures the Windows-side adb/firewall bridge used from WSL. These scripts operate developer infrastructure; they are not runtime dependencies.

`tools/make_icons.py` generates launcher densities from the source brand image. Its Python AST parsed successfully; the manifest records `round_rect_mask`, `save_png`, `fit_within`, `composite_foreground`, `main`, and the `Metrics` class. `.githooks/pre-commit` optionally removes Windows `Zone.Identifier` metadata and refuses risky staged collisions.

## Coverage evidence

This page owns 42 files: nine Dart tests, Android/Kotlin/Gradle configuration and assets, release workflow, Git hook, and four developer tools. The ten launcher PNGs are individually fingerprinted binary assets but discussed as one density family. Exact evidence is in `../coverage-manifest.json`.

