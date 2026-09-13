# Meal Mate architecture wiki

Snapshot: 2026-09-07 at commit `c834724eee35997c7481a92a652d6414ec2aecdd` on `master`.

Meal Mate, branded in-product as Kimatta while the repository retains its temporary package identity, is an offline-first household meal-planning application. Flutter owns presentation and platform interaction. Rust owns durable household and food state, planning policy, deterministic plan generation, shopping derivation, and SQLite persistence. `flutter_rust_bridge` exposes coarse service calls between them.

## Read this first

- [System overview](system-overview.md): deployment shape, dependencies, build contract, and architectural invariants.
- [Flutter shell](flutter-shell.md): startup, routing, navigation shell, theming, and native build integration.
- [Flutter features](flutter-features.md): screens, Riverpod ownership, and user workflows.
- [Bridge and storage](bridge-and-storage.md): DTO boundary, connection lifecycle, SQLite schema, transactions, export, and restore.
- [Rust domain and planner](rust-domain-and-planner.md): kernel types, food model, deterministic search, coverage, attention, and application orchestration.
- [Tests, platform, and tools](tests-platform-and-tools.md): test layers, Android packaging, release automation, and device tooling.
- [Product and governance](product-and-governance.md): PRDs, roadmap, task cards, research, reviews, and reference content.
- [Writing style](style.md): conventions for future updates.

## Runtime flow

```text
Android FlutterActivity
  -> Dart main -> RustLib.init
  -> ProviderScope -> App -> GoRouter shell
  -> feature screen -> Riverpod notifier
  -> generated Dart FRB API
  -> rust/src/api service adapter
  -> process-wide Mutex<SQLite connection>
  -> kimatta-storage and kimatta-application
  -> household-core + food-domain
```

The startup dependency is deliberate: `healthReportProvider` opens the database, `householdProvider` waits for it and bootstraps the anonymous household, and downstream providers select the household ID. The app-level listener performs first-run routing and starts idempotent starter-content installation without blocking navigation.

## Dependency rule

```text
Flutter presentation -> generated bridge -> bridge API
                                         -> kimatta-application -> food-domain -> household-core
                                         -> kimatta-storage ----^             -> household-core
```

The kernel does not import food concepts. Flutter does not mutate SQLite tables. Database handles never cross the bridge. Generated bindings are outputs, not hand-maintained source.

## Sole path-to-page routing map

The exact per-file evidence is `../coverage-manifest.json`. These routes are disjoint and exhaustive at this snapshot:

| Tracked path | Page |
|---|---|
| `.gitattributes`, `.gitignore`, `.metadata`, `LICENSE`, `README.md`, `analysis_options.yaml`, `flutter_rust_bridge.yaml`, `pubspec.lock`, `pubspec.yaml`, `rust/.gitignore`, `rust/Cargo.lock`, `rust/Cargo.toml`, `rust/rust-toolchain.toml` | `system-overview.md` |
| `hook/**`, `lib/main.dart`, `lib/app/**` | `flutter-shell.md` |
| `lib/features/**` | `flutter-features.md` |
| `lib/src/rust/**`, `rust/src/**`, `rust/crates/kimatta-storage/**` | `bridge-and-storage.md` |
| `rust/crates/household-core/**`, `rust/crates/food-domain/**`, `rust/crates/kimatta-application/**` | `rust-domain-and-planner.md` |
| `.githooks/**`, `.github/**`, `android/**`, `test/**`, `tools/**` | `tests-platform-and-tools.md` |
| `.impeccable.md`, `KNOWN_ISSUES*`, `docs/**`, `refs/**` | `product-and-governance.md` |

Complement result: 240 tracked files, 240 assignments, zero uncovered paths, zero duplicate assignments. The inventory contains 225 text files and 15 binary image assets; it contains no symlinks or submodules. `tools/make_icons.py` parsed successfully and its top-level definitions are recorded in the manifest.

## Snapshot caveats

`docs/ROADMAP.md` was modified in the working tree when this report started. It is inventoried and its current bytes are fingerprinted, but this wiki does not treat the edit as committed architecture. There are no WIP-backed architecture claims, page failures, deferred pages, or omitted tracked files.
