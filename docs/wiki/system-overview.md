# System overview

## Responsibility

The repository builds one Android-first Flutter application backed by a Rust static/dynamic library. The installed app is local-only today: authoritative state lives in an app-private SQLite database, with explicit user export and restore. Cloud sync, durable remote auth, public sharing, billing, production activation, and iOS packaging are roadmap work rather than current adapters.

## Package and build shape

`pubspec.yaml` defines the private Dart package `meal_mate`, Flutter/Riverpod/go_router dependencies, FRB 2.13.0, platform directory lookup, and sharing support. `flutter_rust_bridge.yaml` maps Rust's `crate::api` to generated Dart under `lib/src/rust`. `hook/build.dart` delegates native-assets production to FRB during Flutter builds.

`rust/Cargo.toml` is both the `kimatta_bridge` package and the workspace root. It builds `cdylib` and `staticlib` artifacts and depends directly on the application service, storage adapter, FRB runtime, and UUID generation. Workspace members are the four crates beneath `rust/crates`: `household-core`, `food-domain`, `kimatta-application`, and `kimatta-storage`.

The repository pins Rust 1.98.0 and FRB 2.13.0. Lockfiles make Dart and Rust dependency resolution reproducible. `analysis_options.yaml` applies Flutter lint defaults. Root Git metadata excludes generated/build artifacts while retaining configuration and generated bridge source required by the project.

## Architectural boundaries

- Flutter owns widgets, accessibility, navigation, transient presentation state, and platform-facing file/share APIs.
- Rust owns identities, domain validation, policy, planner behavior, durable data, backup validation, and transactional mutations.
- The bridge publishes task-sized commands and DTOs. It does not publish SQLite handles or field-by-field persistence operations.
- `household-core` is domain-neutral. `food-domain` may depend on it, but the kernel may not depend on food types.
- Planner output is deterministic for the same snapshot, algorithm version, and search parameters. User locks and hard restrictions are constraints, not compensating score terms.
- Planned does not mean cooked, pantry is an optional binary mark, and coverage claims are limited to what stored evidence supports.

## Data and control lifecycle

On app start, Dart initializes the native library and mounts Riverpod. The health provider resolves the app support path and asks Rust to open the database. Rust owns one process-wide connection behind a mutex; opening applies migrations and produces a schema/health report. The household provider then creates or loads the anonymous household. Feature providers read whole DTOs and send explicit commands back through the bridge.

The planner path loads a consistent snapshot from storage, filters infeasible candidates with Tier-0 rules, performs deterministic bounded beam search, assesses coverage and uncertainty, creates attention requests and an action proposal, and records an immutable controller-ledger entry. Applying a plan is conditional on its status and occurs transactionally with its evidence row.

## Operational contract

The documented core gate is Rust formatting, clippy with warnings denied, workspace tests, a fresh release Rust build, Flutter analysis/tests, and Android APK construction. Binding generation is required after changes to `rust/src/api`. A stale native library can make Flutter tests misleading, so the Rust release build precedes them.

Release APK publication is tag-driven through GitHub Actions. The repository still uses a temporary Android identity and debug signing for internal sideloading; production identity/signing are explicit future decisions.

## Coverage evidence

This page owns 13 repository-level manifests and configuration files. Exact paths, modes, hashes, sizes, kinds, and discovered symbols are in `../coverage-manifest.json`. No architecture claim on this page depends on the modified working-tree roadmap.

