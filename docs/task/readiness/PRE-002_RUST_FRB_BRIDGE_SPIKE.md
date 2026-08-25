# PRE-002 — Rust/FRB toolchain and bridge spike

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | Done |
| Type | Readiness |
| Workstream | Developer environment / architecture pivot |
| Depends on | MVP-001 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | Complete before DEC-005 and MVP-002 |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | `rustup`/`cargo`/`flutter_rust_bridge_codegen` installs write to `~/.cargo` and `~/.rustup` (user-level, network access); no credentials, signing, cloud resources, or system-wide package changes |

This card does not depend on `DEC-002` (Done 2026-08-24): the spike adds no Dart domain code — only the format/analyze/test contract already in force from PRE-001 applies. `MVP-001` is Done, so this card is `Ready`.

## Outcome and user value

Prove, on this WSL2 host, that a Rust library can be built, bridged with `flutter_rust_bridge`, called from the Flutter app with a typed error, and packaged into Android debug and release APKs — before any durable domain logic is written in Rust. PRD v3 makes this spike a hard prerequisite: "No major business-logic migration until this works."

## Authoritative sources

- `docs/PRD_v3.md` §6.2–6.3 (Rust, FFI/bridge rules, Phase-0 requirement), §17 Phase 1, §22 (Rust risk and kill criterion)
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Phase 0 — prove Rust integration first"
- `docs/ROADMAP.md` D-015, D-022, D-028
- `docs/task/MVP_INVARIANTS.md` 15, 21

## Load-bearing constraints

- Stable Rust only, pinned in `rust-toolchain.toml`; the `flutter_rust_bridge` Dart package version equals the codegen version.
- The bridge surface is coarse (invariant 21): a version call and a health call, nothing per-field.
- Generated bindings (`lib/src/rust/**`, `frb_generated.*`) are never hand-edited. Integration build glue that `integrate` writes (build hook, Gradle plugin/config) may be edited only inside the bounded fix budget below, and every such edit is recorded.
- No SQLite in the spike — the bridge and the native C cross-compile are separate failure surfaces; SQLite lands in MVP-002.
- Do not use a bare `git checkout -- .` or `git clean` at any point: the owner's uncommitted work (`KNOWN_ISSUES.md`, `KNOWN_ISSUES-low.md`, `DEC-004`, the v3 docs) is hashed before the spike and must be byte-identical after it. Any revert uses the delta of `git status --short` against that baseline, guarded by the hash check.
- iOS tooling is out of scope on this host (D-015, invariant 14); see Non-goals.

## Scope

Outcome-level; the spike decides mechanism and records what it found.

- Install and pin a stable Rust toolchain plus `flutter_rust_bridge_codegen`. Versions current when this card was written (2026-08-24): Rust 1.98.0, FRB 2.13.0 (pub and crates.io), `cargo-ndk` 4.1.2; Android NDK 28.2.13676358 is already installed under `~/Android/sdk/ndk`. **Re-verify at execution; do not copy blindly.**
- Integrate FRB into this repository. Evaluate the build-hook/native-assets backend first if compatible with the pinned Flutter (the prompt's instruction), otherwise the default integration; record which backend built and why.
- Expose a version call and a health call whose error type is a typed Rust enum that the Dart side can catch and match on.
- Replace the template counter screen with a minimal screen that renders both calls; keep a widget test that does not load the native library.
- Build Android debug and release APKs and verify the Rust library is present for every shipped ABI. The ABI set is a spike output, recorded in the command contract.
- Install and run on the D-022 emulator per the PRE-001 command contract; capture screenshots of the version/health screen and of the typed-error path.
- Time one incremental Rust change → regenerated bindings → debug APK, and record the seconds.
- Record the clean-checkout command contract (install, generate, build) in `docs/ROADMAP.md`. Mirror the same commands into the `README.md` "Architecture direction" section, replacing its "will be recorded by `PRE-002`" sentence.

### Suggested starting point (the spike may deviate; record deviations)

- Bridge crate at `rust/crates/kimatta-bridge` (workspace root `rust/`), so MVP-002 can add `household-core` and `kimatta-storage` beside it without moving generated paths. `kimatta-*` crate and type names are provisional pending DEC-004; renaming crates is mechanical.
- API: `core_version() -> String`; `health_check(db_path: String) -> Result<HealthReport, KimattaError>` where `KimattaError` is a `thiserror` enum declared in the scanned `api/` module (e.g. `InvalidPath`, `Storage`), and the stub rejects an empty/whitespace path so the error path is exercisable before SQLite exists.
- Flutter: `HealthScreen(Future<HealthReport> report, Future<HealthReport> Function() probeError)`; the widget test passes completed futures.
- Fix budget: at most two diagnosable fixes per backend before falling back to the other backend; at most one fallback cycle, then stop and report to DEC-005.
- Every cargo/FRB command runs with `export PATH="$HOME/.cargo/bin:$PATH"` because agent shells do not source `~/.cargo/env`.

## Non-goals

- SQLite, `rusqlite`, migrations, or any domain type (MVP-002).
- iOS build or signing on this host — see AC-8.
- Release signing, Play, CI, credentials (invariant 15).
- Any Firebase work.

## Decision gates

- Backend choice and the PRD §22 core-language commitment are not decided here. This card produces the evidence; `DEC-005` decides. If both backends fail the fix budget, stop and hand the evidence to DEC-005 with the PRD §22 fallback (temporary Dart core behind the same interfaces) on the table.

## Acceptance criteria

- **AC-1:** Toolchain versions are pinned (`rust-toolchain.toml`, exact FRB versions in `pubspec.yaml` and `Cargo.toml`) and documented with install locations and the chosen backend.
- **AC-2:** A typed Rust error crosses the bridge and is observed in Dart as a matchable exception type, proven by a test that loads the built native library (preferred whenever the backend permits; the spike records the resulting test↔toolchain coupling as input to DEC-005 decision 3) or by on-device capture. The fake-fed widget test in Scope does **not** discharge this item. If on-device was the only possible route and AC-5 ends NOT VERIFIED, AC-2 inherits AC-5's D-027 block.
- **AC-3:** `flutter build apk --debug` succeeds.
- **AC-4:** `flutter build apk --release` succeeds and the release APK contains the Rust library for every shipped ABI.
- **AC-5:** The app launches on the D-022 emulator and both the health screen and the error path are captured. **D-027 block:** the emulator, adb server, and firewall rule run on the Windows host under owner control (PRE-001 prerequisites 1–2); if the bridge is unavailable after the contract's bounded wait, the item is `NOT VERIFIED` with discharge at the next card that produces runtime evidence (MVP-002 AC-3), and Done requires the owner's `owner-accepted YYYY-MM-DD` clause.
- **AC-6:** Owner-work hashes recorded before the spike are unchanged after it.
- **AC-7:** The incremental dev-cycle time (Rust edit → APK) is recorded in seconds.
- **AC-8:** iOS release packaging — `NOT VERIFIED` by construction on this host. **D-027 block:** no macOS host or CI exists and iOS is post-launch by D-015; discharge point `PRE-003`; Done requires the owner's `owner-accepted YYYY-MM-DD` clause in the Evidence-log row.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | `rustc --version`, `flutter_rust_bridge_codegen --version`, pinned files, backend recorded in the ROADMAP command contract |
| AC-2 | Output of a native-loading test, or the AC-5 emulator screenshot of the error path; state which route was used |
| AC-3 | Build output and artifact path |
| AC-4 | Build output plus `unzip -l` listing of `lib/<abi>/*.so` for each ABI |
| AC-5 | Windows-host emulator screenshots captured via the D-022 adb bridge, or the D-027 block with owner acceptance |
| AC-6 | `sha256sum -c` output against the pre-spike hash file |
| AC-7 | Timed command output |
| AC-8 | D-027 block with owner acceptance recorded in the ROADMAP evidence row |

## Stop/failure conditions

- Stop before any system-wide package replacement, credentials, signing, or unauthorized external resource.
- Stop if a fix would require hand-editing generated bindings.
- Two failed backend cycles return the card to planning and route the evidence to DEC-005.

## Handoff

Record PASS/FAIL/NOT VERIFIED evidence, the PRE-002 command contract (install, generate, build, ABI set, dev-cycle seconds), and the backend outcome in `docs/ROADMAP.md`; update `docs/V3_IMPLEMENTATION_STATUS.md` and the README build section. Promote DEC-005 only when this card is Done.

## Evidence (2026-08-24)

Status `Done`: every criterion is PASS except AC-8, which is `NOT VERIFIED` by construction under D-027 with `owner-accepted 2026-08-24` recorded in the ROADMAP Evidence-log row (discharge point PRE-003). Full per-criterion evidence, the command contract, ABI set, and dev-cycle seconds are in `docs/ROADMAP.md`; raw artefacts (logs, screenshots, hash file, fix ledger) in `~/pre002_evidence/`.

- Backend: native-assets built and shipped; cargokit not attempted. Fix ledger: 1 of 2 — `flutter test` needs `cargo build --release` in `rust/` because the generated loader reads `rust/target/release/` (no glue edited).
- AC-2 route: native-loading test (`test/bridge_native_test.dart`), the preferred route; on-device capture also exists (AC-5).
- Deviations from the suggested starting point: crate is `rust/` (`kimatta_bridge`) rather than `rust/crates/kimatta-bridge` — cargo rejects an empty `crates/*` workspace glob, so MVP-002 adds the workspace with its first sibling crate; `--no-integration-test` passed to `integrate`; `freezed`/`build_runner`/`freezed_annotation` added because data-carrying error enums generate `freezed` sealed classes (bridge tooling, invariant 21).
- Widget test isolation is at load time only: under this backend `flutter test` still runs the Rust build hook (input to DEC-005 decision 3).
