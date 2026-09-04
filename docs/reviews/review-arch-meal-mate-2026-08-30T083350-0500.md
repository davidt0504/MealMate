# Architecture Review — Meal Mate repository

**Reviewed:** 2026-08-30 08:33:50 CDT  
**Repository:** `/home/davidlinux/projects/personal/meal_mate`  
**Scope:** Entire Git repository at `241f3d852ba7411869e1008a71cf155a894c4fcd` (`master`)  
**Review mode:** Read-only architecture review; no project files changed  
**Working tree at review start/end:** Existing untracked `KNOWN_ISSUES-audit.md`; untouched  

## Executive verdict

The repository is a coherent, deliberately narrow Flutter/Rust foundation. Its intended dependency direction is visible in code, the SQLite foundation is normalized and transactional, the generated FFI boundary is coarse, and the documented Rust/Flutter gates pass. There are no CRITICAL or HIGH defects at the scaffold's present maturity.

The architecture is **not ready to carry real household data or support a beta yet**. Seven MEDIUM findings need explicit disposition. M-1 through M-5 belong before or during the first real persistence card (`MVP-004`): a cross-household write invariant is unenforced, Android's default backup behavior implicitly exports the database, connection/concurrency ownership is undefined, the first write API is not callable through the currently allowed dependency seam, and startup has no recovery boundary. M-6 and M-7 should close before the bridge grows materially or beta evidence is trusted: the two-language build contract is manual-only, and the global error/init contract is misplaced under the health probe.

| Area | Verdict | Reason |
|---|---|---|
| Structure and dependency direction | PASS WITH CAVEATS | Rust kernel and storage edges point inward correctly; the transitional bridge/storage seam is incomplete. |
| State ownership | PASS FOR CURRENT SPIKE | Dart owns only transient probe state; Rust owns identity and SQLite. No real application state exists yet. |
| Persistence and integrity | BLOCK BEFORE USER DATA | Migrations, foreign keys, and transactions are sound, but household/member pairing is not enforced. |
| Concurrency | NOT YET DEFINED | Every storage-touching bridge call opens/migrates/drops a connection; safe only because one valid call exists today. |
| Public interfaces and failure boundaries | NEEDS REMEDIATION | The global error/init contract is nested under `health`; bootstrap failures occur before any recoverable UI. |
| Security and privacy | BLOCK BEFORE USER DATA | Android Auto Backup defaults apply to the app-private database because no backup policy is declared. |
| Deployment | SCAFFOLD-READY, NOT BETA-READY | Clean debug/release builds work and carry all three ABIs; release is intentionally debug-signed and uses a temporary ID. |
| Observability | INTENTIONALLY DEFERRED | No telemetry exists; `MVP-019` defines privacy constraints and the beta gate requires operational readiness. |
| Test architecture | GREEN BUT MANUAL | 9 Rust and 5 Flutter tests pass; there is no CI or repository-enforced generated/native freshness gate. |

## Scoped inventory

The repository contains 101 tracked files:

| Surface | Inventory and role |
|---|---|
| Product/governance documentation | 49 Markdown files, including PRD v3, architecture principles, roadmap, invariants, task cards, and known-issue ledgers. |
| Flutter/Dart | `lib/main.dart`, generated FRB bindings under `lib/src/rust/`, two Flutter tests, and the native-assets build hook. |
| Rust | Workspace root/bridge plus `household-core` and `kimatta-storage`; six hand-written/generated Rust source files and four Cargo manifests/toolchain files. |
| Android/Kotlin/Gradle | One `FlutterActivity`, Android manifest/resources, Kotlin DSL build configuration, and Gradle wrapper properties. |
| Operations | WSL/Windows emulator bridge scripts and an optional metadata-cleanup Git hook. |
| Platforms absent by design | No iOS, web, desktop, backend, cloud, or production deployment tree. Android is the MVP platform. |

The code surface is small: approximately 630 non-generated lines across the principal Dart, Rust, Kotlin, Gradle, and hook files, excluding operational scripts, documentation, lockfiles, and generated FRB code.

### Inspected

- All hand-written Dart and Rust product code, manifests, package/workspace definitions, Android/Kotlin/Gradle configuration, tests, build hook, Git hook, and `tools/emulator.sh`.
- Generated Dart API surface and generated-file inventory; generated implementation imports were traced.
- Architecture/governance documents: `README.md`, `docs/PRD_v3.md` architecture/testing sections, `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` privacy/implementation sections, `docs/V3_*`, `docs/ROADMAP.md`, MVP invariants, and the task cards for MVP-002, MVP-004, MVP-017, MVP-019, and MVP-022.
- Current `KNOWN_ISSUES.md` and relevant entries in `KNOWN_ISSUES-low.md` to distinguish known/deferred risks from newly observed risks.
- Current Android debug/release build behavior, APK Rust ABIs, and release signer.

### Not inspected

- Project-local `.claude` and global Claude infrastructure, per the immutable-input/runtime policy.
- Line-by-line internals of generated `frb_generated.*`, `health.freezed.dart`, binary launcher images, or dependency source code.
- Every historical PRD/task card in full; non-current cards were sampled where they define a reviewed architectural boundary.
- `tools/windows/setup-adb-bridge.ps1` in detail, live Windows firewall/emulator state, on-device runtime behavior, or iOS packaging.
- External cloud, auth, sync, sharing, analytics, signing, Play, or production systems; none are configured in this repository.

There is no Python code, so a Python import graph does not apply. The mixed Dart/Rust/Kotlin/Gradle entry points and dependency paths were accounted for directly.

## Current architecture trace

```text
Android launcher
  android/app/src/main/AndroidManifest.xml
    -> MainActivity : FlutterActivity
      -> lib/main.dart::main
        -> RustLib.init()                         [generated FRB runtime]
        -> getApplicationSupportDirectory()      [Dart platform adapter]
        -> HealthScreen                          [transient Dart view state]
          -> coreVersion()                       [coarse sync FFI call]
          -> healthCheck(dbPath)                 [coarse async FFI call]
            -> rust/src/api/health.rs
              -> kimatta_storage::open(path)
                -> SQLite migration v1
                -> app-private files/kimatta.db

Build path
  flutter command
    -> hook/build.dart
      -> flutter_rust_bridge native-assets builder
        -> kimatta_bridge cdylib/staticlib
          -> kimatta-storage
            -> household-core
```

### Dependency direction

- Dart presentation imports only generated bridge API/runtime and `path_provider`; there is no duplicated Dart domain model.
- `kimatta_bridge -> kimatta-storage -> household-core` is the only application dependency chain. `household-core` knows nothing about Flutter, SQLite, or food types.
- `kimatta-storage` owns SQLite and imports the identity types it persists. No SQLite handle crosses the bridge.
- Generated FRB files form an expected Dart cycle between the API types and loader; hand-written code does not form a package cycle.
- Kotlin contains only the Flutter host activity. Gradle invokes Flutter; Flutter's native-assets hook invokes Rust.

This matches `docs/V3_MIGRATION_PLAN.md:39-51` and `docs/PRD_v3.md:209-245`, subject to findings M-3 and M-4 below.

## Severity-ordered findings

### MEDIUM M-1 — The storage write API can attach a member to the wrong existing household

**Evidence:** `rust/crates/kimatta-storage/src/lib.rs:50-69`, especially the explicit contract at lines 50-52 and the insert at lines 63-67; `docs/task/MVP_INVARIANTS.md:5,21`; `docs/task/mvp/MVP-004_EMULATOR_AUTH_HOUSEHOLD_PERSISTENCE.md:35-48,58-74`.

`insert_household(household, members)` inserts the parent and each member in one transaction, but it never verifies `member.household_id == household.id`. The foreign key proves only that the referenced household exists. If household B already exists, a call that creates household A while passing a member scoped to B succeeds and commits that member under B.

The existing `orphan_member_rejected` test (`rust/crates/kimatta-storage/src/lib.rs:119-126`) cannot detect this because it references a nonexistent household. This is the exact negative case MVP-004 AC-2 requires with a second-household fixture.

**Impact:** The repository's primary authorization/data boundary can be violated by one malformed application-layer call. There is no live caller today, which keeps severity below HIGH, but this must block the first real household write path.

**Required disposition:** In MVP-004, either validate the pairing and return a typed domain/storage error, or make mis-parenting unrepresentable by deriving the stored household ID from the parent command. Add a two-existing-household negative test.

**Status:** Already tracked as open in `KNOWN_ISSUES.md:131`; verified against current source.

### MEDIUM M-2 — Android backup policy is implicit, so `kimatta.db` is eligible for automatic cloud backup

**Evidence:** `lib/main.dart:16-24` places `kimatta.db` under `getApplicationSupportDirectory()`; the MVP-002 evidence identifies this as Android `getFilesDir()` (`docs/task/mvp/MVP-002_ENGINEERING_FOUNDATION_DOMAIN_SPINE.md:88-93,108-111`). `android/app/src/main/AndroidManifest.xml:2-5` declares no `android:allowBackup`, `android:fullBackupContent`, or `android:dataExtractionRules` policy.

Android Auto Backup defaults `allowBackup` to true and includes files under `getFilesDir()` unless an application declares exclusions/rules. Official behavior: <https://developer.android.com/identity/data/autobackup>.

**Impact:** As soon as MVP-004 writes names/household identity, the supposedly local-first database can leave the device through an OS cloud-backup path that the repository has neither selected, documented, versioned, privacy-reviewed, nor restore-tested. On Android 12+, device-to-device behavior also needs explicit consideration rather than assuming `allowBackup=false` controls every transfer.

**Required disposition:** Before real household data is accepted, explicitly decide cloud-backup and device-transfer policy. If backup is allowed, define version-aware include/exclude rules and test restore/migration. If it is not allowed, declare the appropriate manifest/rule configuration and keep the user-controlled export/restore path in MVP-017. This is a policy decision, not merely an XML edit.

**Status:** Newly identified by this repository-wide review; not present in the current known-issue ledgers.

### MEDIUM M-3 — Storage connection ownership and concurrency behavior are undefined

**Evidence:** `rust/src/api/health.rs:30-41`; `rust/crates/kimatta-storage/src/lib.rs:34-48`; `lib/main.dart:21-24`; `docs/PRD_v3.md:235-245,673-699`.

Every valid `health_check` opens a new SQLite connection, toggles foreign keys around migrations, reads the version, and drops the connection. There is no process-level connection owner, serialization strategy, busy timeout, or lifecycle/reinitialization contract. FRB exposes non-sync calls as futures and dispatches work away from the Dart UI, so overlap becomes reachable as soon as a second storage operation is added.

Today, only one valid storage-touching call exists; the error probe rejects whitespace before opening storage. Therefore this is a latent design boundary, not a current race.

**Impact:** Copying the health-call pattern into MVP-004 can produce overlapping migration/open/write calls and immediate `database is locked` failures, while obscuring which layer owns transactions and connection lifecycle.

**Required disposition:** Decide the connection owner before adding the second storage-touching bridge operation. Keep any connection/handle entirely on the Rust side; define initialization, serialization, shutdown/reinitialization, and corruption/open-failure behavior. Do not return a SQLite handle to Dart.

**Status:** Already tracked as open in `KNOWN_ISSUES.md:128`; verified against current source.

### MEDIUM M-4 — The transitional bridge/storage contract is not usable for the first write path

**Evidence:** `rust/crates/kimatta-storage/src/lib.rs:6,53-57`; `rust/Cargo.toml:9-13`; `rust/crates/kimatta-storage/Cargo.toml:6-9`; `docs/V3_MIGRATION_PLAN.md:43-51`.

The only write API takes `household_core::Household` and `HouseholdMember`. `kimatta-storage` does not re-export those public types, and the bridge crate depends on `kimatta-storage` but not `household-core`. Consequently, the bridge cannot construct the parameters of the only write function through its currently declared direct dependency.

This is currently invisible because the function has no production caller. It becomes load-bearing in MVP-004, which must bootstrap identity, household, and member through a coarse command.

**Impact:** The documented temporary edge "bridge -> storage until application exists" is not a complete public contract. A rushed fix can either widen bridge dependencies arbitrarily or leak persistence-shaped inputs into the FFI API.

**Required disposition:** Settle the MVP-004 command/DTO boundary first, then make the Rust-side application seam able to construct domain objects and call storage. Options include a small application service that depends on core+storage, a deliberate bridge dependency on core, or storage-owned input records/re-exports. Prefer a service command that also enforces M-1 and owns the connection from M-3.

**Status:** Already tracked as open in `KNOWN_ISSUES.md:125`; verified against current source.

### MEDIUM M-5 — Native initialization and platform-directory failures occur before any recoverable UI exists

**Evidence:** `lib/main.dart:11-28`.

`main()` awaits `RustLib.init()` and `getApplicationSupportDirectory()` before `runApp()`. A missing/incompatible native library, hook packaging regression, platform-channel failure, or directory-resolution failure prevents Flutter from mounting any application widget. The later `FutureBuilder` handles a database-open error only as text and offers no retry/recovery path.

**Impact:** The most consequential platform/persistence failures present as an absent app shell or unrecoverable development error rather than truthful user-visible state. MVP-017 requires honest recovery states, but the failure boundary should be established before MVP-004 makes persistence part of first launch.

**Required disposition:** Introduce an app bootstrap state boundary that mounts UI before or around fallible initialization, categorizes initialization/storage failures, supports safe retry or recovery where possible, and avoids exposing raw internal details. Test failed initialization/path/database-open states independently from the native happy path.

**Status:** Newly identified by this review.

### MEDIUM M-6 — The generated/native bridge gate is documented but not enforced by repository automation

**Evidence:** No tracked `.github/`, other CI configuration, or general quality-gate script exists. The only Git hook is optional and deletes Windows metadata (`.githooks/pre-commit:1-25`). Binding generation is a manual instruction (`README.md:28-39`; `docs/ROADMAP.md:250-278`). The repository itself documents that body-only Rust changes can false-pass against a stale release library unless `cargo build --release` precedes `flutter test` (`docs/ROADMAP.md:265-278`; qualified by `KNOWN_ISSUES-low.md:72`).

**Impact:** The most load-bearing two-language contract depends on contributors remembering several commands in the correct order. A clean local test result is not durable evidence that generated bindings, the release library, and Android packaging all correspond to the reviewed source. There is also no automated clean-checkout build or APK ABI/signing inspection.

**Required disposition:** Before multiple bridge APIs or beta work, provide one repository-owned gate that runs formatting/analyzers, Rust clippy/tests, binding freshness validation, current native build, Flutter tests, and Android packaging. Run it in CI when CI becomes available; until then, use the same script as the single documented local command. Avoid relying on an optional destructive cleanup hook for unrelated enforcement.

**Status:** The absence of CI is documented historically in `docs/V3_MIGRATION_PLAN.md:17`; this review elevates the bridge-specific enforcement consequence.

### MEDIUM M-7 — The cross-cutting bridge error/init contract is owned by the health probe module

**Evidence:** `rust/src/api/health.rs:1-28,44-47`; generated publication in `lib/src/rust/api/health.dart:14-45`; `lib/main.dart:8,55-60`.

`KimattaError` is documented as the bridge-wide error surface, and `init_app` is the global FRB initializer, but both live in `api::health`. A second API module must import a global application concern from a probe module or publish a second unrelated error enum. In addition, `KimattaError::Storage { message }` flattens every SQLite/migration failure to an unstructured internal string, limiting safe retry/recovery behavior and potentially exposing paths or schema detail when rendered directly.

**Impact:** The first real API will either create an inverted dependency on `health` or force a breaking reorganization of generated Dart imports. Recovery code cannot reliably distinguish lock, corruption, migration, missing directory, or disk conditions.

**Required disposition:** Before adding a second API module, move app initialization and shared error types to an application-level module. Define stable, user-safe error categories and keep diagnostic detail separate from the public DTO/error contract.

**Status:** The module-ownership portion is already tracked as open in `KNOWN_ISSUES.md:122`; raw error categorization is an additional failure-boundary observation.

## Explicitly deferred gaps, not current defects

These are real readiness constraints but are already fenced by the roadmap and should not be confused with accidental architecture defects:

| Gap | Current evidence | Discharge point |
|---|---|---|
| Temporary brand/application ID | `README.md:42-48`, `android/app/build.gradle.kts:17-25` | DEC-004 before MVP-018/021/022 |
| Release APK uses Android debug signing | `android/app/build.gradle.kts:35-40`; verified signer `CN=Android Debug` | MVP-022 / separately authorized signing |
| No validated export/restore or corruption recovery | `docs/task/mvp/MVP-017_OFFLINE_CACHE_RECOVERY.md:35-75` | MVP-017 |
| No observability/telemetry | `docs/task/mvp/MVP-019_OBSERVABILITY_FOUNDATION.md:23-78` | MVP-019, privacy-gated |
| No production backend/auth/sharing | Roadmap register and delivery gates | DEC-003, MVP-018 through MVP-021 |
| No iOS tree or packaging evidence | `docs/V3_IMPLEMENTATION_STATUS.md:44-47` | PRE-003, post-launch |
| Current UI is a disposable bridge health screen | `lib/main.dart:13-15,30`; roadmap names MVP-003 next | MVP-003 |

## Optional redesign ideas

These are not defects and should not be implemented without the relevant card/decision:

1. Let the first real coarse command (`bootstrap_household`) establish a small Rust application-service seam that owns connection access, constructs domain entities, and enforces household pairing. This can resolve M-1, M-3, and M-4 without exposing storage structures to Dart.
2. Use separate public error codes and private diagnostic context. The public DTO can remain stable while Rust logs or local diagnostics retain the underlying SQLite error chain.
3. Treat Android OS backup/transfer as its own platform adapter and policy surface. Do not silently equate OS Auto Backup, user-exported backup, and future cloud sync; they have different consent, validation, and restore semantics.
4. Add a repository-owned architecture check only when there is a second enforceable rule. Today, `cargo tree`, Dart imports, and Cargo manifests are small enough for direct review; a large dependency framework would be premature.

## Strengths worth preserving

- **Clear ownership split:** `docs/PRD_v3.md:209-245,673-727` is reflected in code. Dart owns presentation/platform lookup; Rust owns identity and SQLite; no database handle or duplicated durable model crosses FFI.
- **Correct inward dependency direction:** `kimatta_bridge -> kimatta-storage -> household-core`; the kernel is free of food, Flutter, and persistence concerns.
- **Small coarse bridge:** The only public operations are version/health calls. There is no per-field FFI chatter or hand-written binding edit.
- **SQLite fundamentals:** Strict normalized tables, explicit migration v1, foreign keys, transactional multi-row insert, idempotent reopen, and migration validation are present and tested (`rust/crates/kimatta-storage/src/lib.rs:19-71,99-162`).
- **Safety restraint:** `household-core` and `kimatta-storage` forbid unsafe code; no unnecessary application async runtime, universal kernel abstractions, sync engine, or cloud dependency has been added.
- **Typed identity:** `HouseholdId` and `MemberId` reject empty/whitespace-only input and are distinct types (`rust/crates/household-core/src/lib.rs:6-49`).
- **Toolchain reproducibility:** Flutter/FRB/Rust versions are pinned; a clean archived checkout successfully rebuilt the debug APK. Flutter regenerated the intentionally untracked Gradle wrapper, so its absence from Git is not a clean-build defect.
- **Android packaging proof:** The current release APK builds and carries `libkimatta_bridge.so` for `arm64-v8a`, `armeabi-v7a`, and `x86_64`.
- **Honest roadmap boundaries:** Deployment, recovery, observability, cloud, production activation, and iOS are assigned to explicit future gates rather than implied complete.
- **Test seams:** The widget screen accepts fake data/functions, Rust tests run without Flutter, and native bridge tests exercise typed errors and a real temp-file migration.

## Verification evidence

Commands were run against the current working tree unless noted:

| Check | Result |
|---|---|
| Git repository/scope | PASS — root is the reviewed directory; `master` at `241f3d8`; only pre-existing untracked `KNOWN_ISSUES-audit.md` |
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 2 `household-core`, 7 `kimatta-storage`, 0 bridge tests |
| `cargo tree --workspace -i rusqlite` | PASS — one `rusqlite v0.40.2` |
| `flutter analyze` | PASS — no issues |
| `cargo build --release && flutter test` with documented Cargo `PATH` prerequisite | PASS — 5 Flutter tests |
| Same Flutter test without Cargo bin in `PATH` | Expected environment failure — native-assets hook could not invoke `rustup`; repository documents the prerequisite |
| Clean archived checkout `flutter build apk --debug` | PASS — Flutter regenerated ignored wrapper artifacts; debug APK built in 82.7 s |
| Current `flutter build apk --release` | PASS — 52.3 MB APK |
| Release APK Rust ABI inspection | PASS — arm64-v8a, armeabi-v7a, x86_64 |
| Release signer inspection | Expected pre-beta state — Android Debug certificate, matching Gradle configuration |
| On-device/emulator run | NOT RUN in this review; prior dated evidence is recorded in MVP-002 |
| iOS build | OUT OF SCOPE / unavailable by repository design |

## Bottom line

Preserve the current high-level architecture. It is appropriately small and points in the right direction. Do not add broad frameworks or cloud infrastructure to fix these findings.

Before MVP-004 writes real household data, resolve M-1 through M-5 together as one coherent bootstrap/persistence boundary and make an explicit Android backup/transfer decision. Before the bridge grows materially or beta evidence is trusted, resolve M-6 and M-7. The release/deployment gaps can remain deferred to their existing roadmap gates.
