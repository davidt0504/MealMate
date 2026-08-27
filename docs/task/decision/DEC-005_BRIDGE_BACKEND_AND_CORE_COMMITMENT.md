# DEC-005 — Bridge backend and core-language commitment

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Decision |
| Workstream | Architecture pivot |
| Depends on | PRE-002 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | After PRE-002 is Done |
| Recommended workflow | `deep-options`; then `grill-me` only for unresolved owner tradeoffs |
| External actions | None |

## Workflow gate

Before resolving `DEC-005`, read `docs/ROADMAP.md`, confirm every declared dependency is Done with evidence, and confirm the roadmap identifies this card as the current required decision or explicit owner-paced work. This decision does not need to be the Next implementation task and does not occupy the implementation lane. Its result and Done transition require explicit owner approval.

## Outcome and user value

Turn PRE-002's evidence into a recorded commitment: which bridge integration the repository uses, and whether Kimatta proceeds with the Rust core or invokes the PRD v3 §22 fallback. Every downstream domain card is re-derived from this decision (D-029).

## Authoritative sources

- `docs/PRD_v3.md` §6.2–6.4, §6.6 (suggested libraries), §17 Phases 1–2, §22 "Risk: Rust slows shipping" (kill criterion)
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Phase 0 — prove Rust integration first", "Decision rule when uncertain"
- `docs/ROADMAP.md` D-015, D-028, D-029; the PRE-002 evidence row and command contract
- `docs/task/MVP_INVARIANTS.md` 14, 17, 21

## Locked constraints

- Do not abandon the Rust architecture because one packaging backend is immature (prompt); do not proceed with Rust if release packaging is unreliable after the bounded spike (PRD §22).
- The bridge stays coarse-grained whichever backend is chosen.
- Any fallback preserves the interfaces (`cover_cycle`, `save_recipe`, …) so the core can move back to Rust later.

## Decisions to resolve

1. **Bridge backend** — build-hook/native-assets or the default FRB integration, from PRE-002's build/release evidence.
2. **Core-language commitment** — commit to the Rust core, or invoke the §22 fallback (temporary Dart core behind the same interfaces). **This commitment is made on Android evidence only — a declared deviation from PRD §17 Phase 1 / §22, which ask for Android+iOS.** Justification: D-015 and invariant 14 make iOS post-launch and this host has no macOS/CI. iOS re-opens the kill criterion at `PRE-003`.
3. **Test coupling** — whether it is acceptable that `flutter test` requires the Rust toolchain (true under the build-hook backend), and how the command contract states it.
4. **MVP-002 layout and libraries** — confirm or revise the crate layout (`household-core`, `kimatta-storage`, `kimatta-bridge`) and the storage libraries sketched in MVP-002 (`rusqlite` with `bundled`, `rusqlite_migration`). PRD §6.6 lists these as suggestions, not decisions.
5. **iOS discharge** — confirm `PRE-003` as the card that carries the iOS `NOT VERIFIED` items from PRE-002.

## Options and tradeoffs

- **Build-hook/native-assets backend** — Flutter's own hook pipeline compiles and bundles the Rust library; no Gradle plugin. Failure modes: maturity of hooks on the pinned Flutter stable, release-build behavior, `flutter test` triggering a host cargo build, debugging inside the Dart hooks pipeline rather than Gradle.
- **Default FRB integration (Gradle plugin)** — battle-tested; cargo runs per ABI from Gradle. Failure modes: NDK discovery, Gradle-version compatibility, a second build system to understand.
- **Hand-rolled `dart:ffi` + `cargo-ndk`** — full control, no generator. Rejected by the prompt: typed errors and DTOs would be written twice, violating the no-duplicate-authoritative-models rule.
- **§22 fallback: temporary Dart core** — keeps the product moving if packaging is unreliable; costs a later migration and loses the portable kernel until then. Only when both backends fail the PRE-002 fix budget.

## Completion criteria

- Decisions 1–5 recorded with rationale, rejected alternatives, test implications, and reversal cost.
- The decision is recorded as the next free D-number in `docs/ROADMAP.md`; the Android-only scope of decision 2 is stated there and in `docs/V3_IMPLEMENTATION_STATUS.md` Deviations.
- MVP-002 may then become Ready; downstream cards are re-derived per D-029.

## Resolution (2026-08-24, D-030)

Decided from the PRE-002 evidence (`docs/ROADMAP.md` evidence row 2026-08-24 and the PRE-002 command contract). Android evidence only; iOS re-opens decision 2 at PRE-003.

1. **Bridge backend — native-assets** (`flutter_rust_bridge_codegen integrate --integration-backend native-assets`; `hook/build.dart` → `flutter_rust_bridge_hooks` 2.13.0 → `native_toolchain_rust`). Rationale: passed every PRE-002 gate with 1 of 2 fixes — debug + release APKs, `libkimatta_bridge.so` for `arm64-v8a`/`armeabi-v7a`/`x86_64`, emulator run, ~10 s edit→APK; one build system, no Gradle plugin. Rejected: cargokit (never needed; archived upstream, Groovy template pinned to AGP 7.3 against this repo's AGP 9.1 — untested risk), hand-rolled `dart:ffi` (duplicate typed models, prompt rule). Reversal cost: one `integrate --integration-backend cargokit` run; Rust and Dart APIs unchanged (FRB migration doc is symmetric). Versions: Flutter 3.47.1, Rust 1.98.0, FRB 2.13.0 across codegen/pub/hooks.
2. **Core language — commit to Rust.** PRD §22 kill criterion ("release packaging unreliable after the bounded spike") was not met: release packaging succeeded first try. The §22 fallback (temporary Dart core) is rejected as contra-evidence. Reversal trigger: PRE-003 failing on both backends.
3. **Test coupling — accepted and stated.** Under native-assets every `flutter test` runs the Rust build hook (host, cargo release profile) and the generated loader opens `rust/target/release/libkimatta_bridge.so`, so the contract reads `(cd rust && cargo build --release) && flutter test` as one line: the hook does not refresh that path, and a stale `.so` yields a false pass rather than a load failure. Every dev/CI host needs rustup — unavoidable once the domain is Rust. Widget tests isolate from *loading*, not *building*. Optional later lanes, not decided here: `FLUTTER_NATIVE_ASSETS=false` for widget-only runs (unverified), mocking `RustLibApi` (FRB seam) from MVP-003.
4. **Layout and libraries.** `rust/Cargo.toml` stays the bridge package (`kimatta_bridge`, FRB default path) and becomes the workspace root when MVP-002 adds `crates/household-core` and `crates/kimatta-storage` (cargo rejects an empty `crates/*` glob, so the `[workspace]` block lands with the first sibling). Moving the bridge to `crates/kimatta-bridge` is rejected: it touches `flutter_rust_bridge.yaml` and `hook/build.dart` for no functional gain. Libraries: `rusqlite` with `bundled` (SQLite C compiled through the same NDK clang the hook proved; system SQLite headers are not reliably exposed by the NDK) and `rusqlite_migration` (small, tested; dropped only if `cargo tree -i rusqlite` shows more than one version). Versions confirmed at MVP-002 AC-1. `freezed`/`build_runner`/`freezed_annotation` remain as generated-binding tooling (invariant 21); D-024's no-codegen rule still governs hand-written Dart.
5. **iOS discharge — PRE-003** carries PRE-002 AC-8 (NOT VERIFIED, owner-accepted 2026-08-24). A macOS CI runner now would breach invariant 15 and D-015.

Test implications: `cargo test --workspace` joins the gate at MVP-002 (AC-4 there); `test/bridge_native_test.dart` is the standing in-process bridge regression net. Ranked failure modes: (1) `rusqlite bundled` cross-compile under the hook — MVP-002 stop condition, cargokit fallback intact; (2) Flutter/FRB hooks API drift — both pinned, reversal = one integrate run; (3) cold `flutter test` cost on fresh hosts — documented, not a defect.
