# V3 Implementation Status

**Updated:** 2026-08-24 (session 1 of the pivot). Edit this file at every v3 card handoff.
**Authoritative inputs:** `docs/PRD_v3.md`, `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md`, `docs/CODEX_CLAUDE_PIVOT_PROMPT.md`.

> **The pivot is not complete.** No Rust code exists yet; Android and iOS packaging of a Rust-bridged app are unverified. This file records exactly what is and is not done (prompt "Deliverables").

## Complete

- Phase-0 repository inventory and component map — `docs/V3_MIGRATION_PLAN.md` (prompt deliverable 1).
- MVP invariants 17–21 added for Rust ownership, Tier-0 constraints, planned ≠ cooked, deterministic planner, coarse bridge — `docs/task/MVP_INVARIANTS.md`.
- Cards: `PRE-002` (Rust/FRB toolchain and bridge spike, `Ready`), `DEC-005` (bridge backend and core-language commitment), `MVP-002` rewritten as the Rust kernel/SQLite foundation; outline cards `MVP-023`, `MVP-024`, `MVP-025`, `PRE-003`; dated v3 amendment banners on every other affected card (D-029).
- Roadmap reconciliation — D-028, D-029, register rows, gates, next action, DEC-002 contract note, evidence-row note, traceability note — `docs/ROADMAP.md`.
- README architecture direction section.
- DEC-002 flipped to Done on recorded evidence; moved to the EMULATOR-PERSISTENCE-READY gate (D-028).
- Working Rust/Flutter bridge spike (prompt deliverable 2) — `PRE-002`, 2026-08-24: native-assets backend, typed error crosses the bridge, debug + release APKs carry `libkimatta_bridge.so` for three ABIs, emulator evidence captured. Done 2026-08-24; AC-8 (iOS) NOT VERIFIED under D-027, owner-accepted 2026-08-24, discharge PRE-003.
- Pinned toolchain and clean-checkout build commands (deliverable 3) — PRE-002 command contract in `docs/ROADMAP.md`.
- README architecture section carries the real build commands (deliverable 7).
- DEC-005 Done (2026-08-24, D-030): Rust core committed on the native-assets backend; MVP-002 promoted to Ready.

## In progress

- None. Next action: `/plan-task docs/task/mvp/MVP-002_ENGINEERING_FOUNDATION_DOMAIN_SPINE.md` (no `--auto`).

## Deferred (with the card that discharges each)

| Prompt deliverable / item | Card |
|---|---|
| 2. Working Rust/Flutter bridge spike | Complete (PRE-002, Done 2026-08-24) |
| 3. Pinned/documented toolchain and clean-checkout build commands | Complete (PRE-002 command contract in ROADMAP) |
| 4. Initial Rust workspace/module boundaries | MVP-002 |
| 5. SQLite migration foundation | MVP-002 |
| 6. Tests for the first migrated domain primitive (household identity) | MVP-002 |
| 7. README architecture update with real build commands | Complete (PRE-002) |
| Kernel primitives beyond identity (`Policy`, `OutcomeAssessment`, `ActionProposal`, `AttentionRequest`, ledger) | MVP-023 |
| Planner, coverage assessment, attention requests | MVP-023 |
| Cover My Week UI | MVP-024 |
| Fixtures, property tests, beam-width benchmark | MVP-025 |
| Privacy-safe metric hooks | MVP-019 |
| iOS build/signing proof | PRE-003 (post-launch, D-015) |
| PRD v3 §26 per-item traceability table | re-derived with the cards (D-029) |
| Full re-derivation of bannered cards | one at a time after DEC-005 (D-029) |

## Blockers

- **iOS packaging cannot be verified on this host** — no macOS or CI. Discharge: `PRE-003`. The prompt's own rule stands: the pivot is not "complete" until Android + iOS packaging is verified.
- **Emulator runtime evidence depends on owner action** — the D-022 Windows-host emulator, adb server, and firewall rule (PRE-001 prerequisites 1–2) are not agent-operable. PRE-002 AC-5 was captured on 2026-08-24 once the owner started the bridge; PRE-002 AC-8 (iOS) and MVP-002 AC-3 carry D-027 blocks.

## Deviations from the prompt / PRD, with rationale

| Deviation | Rationale |
|---|---|
| No code or toolchain work in this session; the eight deliverables are split across PRE-002 → DEC-005 → MVP-002 | Owner decision via `/deep-options` (2026-08-24). The prompt's Phase-0 rule requires the migration plan before refactoring, and `docs/task/README.md` forbids `--auto` for the first three implementation cards and requires a red-teamed plan for Elevated cards; a one-shot pivot would bypass both. |
| Rust-core commitment (DEC-005, resolved 2026-08-24 as D-030) made on **Android evidence only**; iOS deferred to PRE-003 | PRD v3 §17 Phase 1 / §22 ask for Android + iOS. D-015 and invariant 14 make iOS post-launch and no macOS/CI exists. iOS re-opens the kill criterion at PRE-003. |
| Three crates to start (`household-core`, `kimatta-storage`, `kimatta-bridge`) instead of the five sketched | Empty crates are scaffolding; `food-domain` and `kimatta-application` appear with their first real type/use case. Dependency direction is preserved. |
| Kernel primitives beyond identity deferred to MVP-023 | Principles §25: only primitives with a call site; the planner is the first call site. |
| `tracing`, `jiff`, `serde` adopted at first call site rather than up front | PRD §6.6 lists them as "likely choices"; adding unused dependencies adds nothing. |
| Task cards reconciled by banners + outline cards, not a full rewrite | D-029: a full rewrite before the spike would be redone if DEC-005 invokes the §22 fallback. |

## Evidence pointers

- Baseline at inventory: `flutter analyze` clean, `flutter test` 1/1 (2026-08-24).
- Owner work untouched: sha256 of `KNOWN_ISSUES.md`, `KNOWN_ISSUES-low.md`, `DEC-004`, and the three v3 docs recorded before and verified after this session's edits.
