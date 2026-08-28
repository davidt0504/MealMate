# V3 Pivot Checklist

**Updated:** 2026-08-28 (MVP-003 card re-derivation, completing D-034). Update this file only when a pivot deliverable or deferral below changes; it is not a live task-status ledger.
**Authoritative inputs:** `docs/PRD_v3.md`, `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md`, `docs/CODEX_CLAUDE_PIVOT_PROMPT.md`.

Live task status, current work, and next work are authoritative only in `docs/ROADMAP.md`.

> **The pivot is not complete.** The Rust kernel/storage foundation landed (MVP-002, 2026-08-24); iOS packaging of the Rust-bridged app is unverified. This file records exactly what is and is not done (prompt "Deliverables").

## Complete

- Phase-0 repository inventory and component map — `docs/V3_MIGRATION_PLAN.md` (prompt deliverable 1).
- MVP invariants 17–21 added for Rust ownership, Tier-0 constraints, planned ≠ cooked, deterministic planner, coarse bridge — `docs/task/MVP_INVARIANTS.md`.
- Cards: `PRE-002` (Rust/FRB toolchain and bridge spike, `Ready`), `DEC-005` (bridge backend and core-language commitment), `MVP-002` rewritten as the Rust kernel/SQLite foundation; outline cards `MVP-023`, `MVP-024`, `MVP-025`, `PRE-003`; dated amendment banners on every other affected card (D-029). *(Historical: that banner stage ran 2026-08-24 and was discharged 2026-08-26 by D-034 — see the next bullet.)*
- Card re-derivation, 2026-08-26 (D-034): 21 cards re-derived from PRD v3 (`MVP-004`–`MVP-022`, `DEC-003`, `OPT-001`) and `MVP-023`, `MVP-024`, `MVP-025`, `PRE-003` given full bodies. `MVP-004` gained an `MVP-003` dependency and was retitled; `MVP-017` was retitled and its sign-out obligation transferred to `MVP-018` as a new AC-6; the PRD v3 §26 traceability table landed in `docs/ROADMAP.md`. `MVP-003` was excluded and was re-derived 2026-08-28 by its own approved plan (step 2), completing D-034.
- Roadmap reconciliation — D-028, D-029, register rows, gates, next action, DEC-002 contract note, evidence-row note, traceability note — `docs/ROADMAP.md`.
- README architecture direction section.
- DEC-002 flipped to Done on recorded evidence; moved to the EMULATOR-PERSISTENCE-READY gate (D-028).
- Working Rust/Flutter bridge spike (prompt deliverable 2) — `PRE-002`, 2026-08-24: native-assets backend, typed error crosses the bridge, debug + release APKs carry `libkimatta_bridge.so` for three ABIs, emulator evidence captured. Done 2026-08-24; AC-8 (iOS) NOT VERIFIED under D-027, owner-accepted 2026-08-24, discharge PRE-003.
- Pinned toolchain and clean-checkout build commands (deliverable 3) — PRE-002 command contract in `docs/ROADMAP.md`.
- README architecture section carries the real build commands (deliverable 7).
- DEC-005 Done (2026-08-24, D-030): Rust core committed on the native-assets backend; MVP-002 promoted to Ready.
- Rust workspace, SQLite migration foundation, and household-identity tests (prompt deliverables 4–6) — `MVP-002` Done 2026-08-27: `rust/` workspace with `crates/household-core` and `crates/kimatta-storage` (`rusqlite` 0.40.2 bundled, `rusqlite_migration` 2.6.0, migration v1, transactional insert, 9 Rust tests); bridge `health_check` opens the real database and reports schema version 1; Rust gate in the command contract. AC-3 verified on-device 2026-08-25 (`emulator-5554`, schema v1 at `/data/user/0/dev.mealmate.temp/files/kimatta.db`); the D-027 block is discharged, all four ACs PASS, and the owner approved closeout on 2026-08-27.

## Deferred (with the card that discharges each)

| Prompt deliverable / item | Card |
|---|---|
| 2. Working Rust/Flutter bridge spike | Complete (PRE-002, Done 2026-08-24) |
| 3. Pinned/documented toolchain and clean-checkout build commands | Complete (PRE-002 command contract in ROADMAP) |
| 4. Initial Rust workspace/module boundaries | Complete (MVP-002, 2026-08-24) |
| 5. SQLite migration foundation | Complete (MVP-002, 2026-08-24) |
| 6. Tests for the first migrated domain primitive (household identity) | Complete (MVP-002, 2026-08-24) |
| 7. README architecture update with real build commands | Complete (PRE-002) |
| Kernel primitives beyond identity (`Policy`, `OutcomeAssessment`, `ActionProposal`, `AttentionRequest`, ledger) | MVP-023 |
| Planner, coverage assessment, attention requests | MVP-023 |
| Cover My Week UI | MVP-024 |
| Fixtures, property tests, beam-width benchmark | MVP-025 |
| Privacy-safe metric hooks | MVP-019 |
| iOS build/signing proof | PRE-003 (post-launch, D-015) |
| PRD v3 §26 per-item traceability table | Complete 2026-08-26 — `docs/ROADMAP.md` "MVP acceptance traceability" |
| Full re-derivation of bannered cards | 21 done 2026-08-26 in one pass (D-034); `MVP-003` discharged 2026-08-28 by its approved plan (step 2), completing D-034 |

## Blockers

- **iOS packaging cannot be verified on this host** — no macOS or CI. Discharge: `PRE-003`. The prompt's own rule stands: the pivot is not "complete" until Android + iOS packaging is verified.
- **Future emulator runtime evidence may depend on owner action** — the D-022 Windows-host emulator, adb server, and firewall rule (PRE-001 prerequisites 1–2) are not always agent-operable. PRE-002 AC-5 and MVP-002 AC-3 were captured after the owner started the bridge; PRE-002 AC-8 (iOS) remains discharged at PRE-003 under D-027.

## Deviations from the prompt / PRD, with rationale

| Deviation | Rationale |
|---|---|
| No code or toolchain work in this session; the eight deliverables are split across PRE-002 → DEC-005 → MVP-002 | Owner decision via `/deep-options` (2026-08-24). The prompt's Phase-0 rule requires the migration plan before refactoring, and `docs/task/README.md` forbids `--auto` for the first three implementation cards and requires a red-teamed plan for Elevated cards; a one-shot pivot would bypass both. |
| Rust-core commitment (DEC-005, resolved 2026-08-24 as D-030) made on **Android evidence only**; iOS deferred to PRE-003 | PRD v3 §17 Phase 1 / §22 ask for Android + iOS. D-015 and invariant 14 make iOS post-launch and no macOS/CI exists. iOS re-opens the kill criterion at PRE-003. |
| Three crates to start (`household-core`, `kimatta-storage`, `kimatta-bridge`) instead of the five sketched | Empty crates are scaffolding; `food-domain` and `kimatta-application` appear with their first real type/use case. Dependency direction is preserved. |
| Kernel primitives beyond identity deferred to MVP-023 | Principles §25: only primitives with a call site; the planner is the first call site. |
| `tracing`, `jiff`, `serde` adopted at first call site rather than up front | PRD §6.6 lists them as "likely choices"; adding unused dependencies adds nothing. |
| Task cards reconciled by banners + outline cards, not a full rewrite | D-029: a full rewrite before the spike would be redone if DEC-005 invokes the §22 fallback. **Superseded 2026-08-26 (D-034)** — the spike settled the question the deferral was hedging, so the full re-derivation ran. |

## Evidence pointers

- Baseline at inventory: `flutter analyze` clean, `flutter test` 1/1 (2026-08-24).
- Owner work untouched: sha256 of `KNOWN_ISSUES.md`, `KNOWN_ISSUES-low.md`, `DEC-004`, and the three v3 docs recorded before and verified after this session's edits.
