# V3 Migration Plan — Kimatta (pending DEC-004) on the Household Control Kernel

**Status:** Phase-0 deliverable of `docs/CODEX_CLAUDE_PIVOT_PROMPT.md`, written 2026-08-24 before any destructive change.
**Authoritative inputs:** `docs/PRD_v3.md`, `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md`. `docs/PRD_v2.md` is historical context (D-028).
**Governance:** `docs/ROADMAP.md` D-028 (adopt v3) and D-029 (how the task system is reconciled).

## 1. Phase-0 inventory (read-only, 2026-08-24)

| Item | Finding |
|---|---|
| Branch / status | `master`; index holds the owner's uncommitted DEC-004 remediation (`KNOWN_ISSUES.md`, `KNOWN_ISSUES-low.md`, `docs/task/decision/DEC-004_*.md`); `docs/PRD_v3.md`, `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md`, `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` untracked. All preserved byte-identical by this plan. |
| Flutter / Dart | Flutter 3.47.1 stable, Dart 3.13.1 (`pubspec.yaml` `sdk: ^3.13.1`); PRE-001 command contract pins JDK 17, Gradle 9.3.1, Android platform-36, NDK 28.2.13676358. |
| Dependencies | `cupertino_icons ^1.0.8`; dev: `flutter_lints ^6.0.0`. Nothing else. |
| Application code | `lib/main.dart` — the stock Flutter counter template. `test/widget_test.dart` — the counter smoke test. That is the whole codebase (MVP-001 clean-room scaffold, 2026-08-22). |
| Models / screens / services / repositories | None. DEC-002 (2026-08-23) *decided* a pure-Dart domain spine; MVP-002 was `Ready` to build it but no code exists. |
| Persistence / Firebase | None configured. MVP-004/017/018 cards assumed Firestore + emulator. |
| Tests / CI | One widget test; no CI. `flutter analyze` and `flutter test` green at inventory time. |
| Real persisted user data / schema | **None.** No database, no schema, no user data. **No schema-compatibility migration plan is needed for v3.** |
| Platforms | Android only (`dev.mealmate.temp`, minSdk 24, D-019/D-020/D-021). iOS is post-launch (D-015). |
| Host toolchain | No Rust toolchain installed; NDK present; no macOS/CI for iOS. |
| Task system | 22 MVP + 4 DEC + 1 PRE + 1 OPT cards, all citing `PRD_v2`; `docs/ROADMAP.md` is the operational source of truth. |

## 2. Component map — KEEP / MIGRATE / ADAPT / DEFER / DELETE

| Component | Disposition | When | Rationale |
|---|---|---|---|
| `lib/main.dart` (counter template) | DELETE (replace) | PRE-002 | Template code; nothing of the product is in it. |
| `test/widget_test.dart` (counter smoke test) | DELETE (replace) | PRE-002 | Tests template behavior only. |
| `android/` platform dir, identity `dev.mealmate.temp`, minSdk 24 | KEEP | — | Flutter owns platform presentation; DEC-004 replaces the identity later. |
| `docs/`, `KNOWN_ISSUES*.md`, task cards, `LICENSE`, `.githooks` | KEEP (+ amend cards) | this session | Governance survives; cards get dated v3 banners (D-029). |
| DEC-001, PRE-001, MVP-001 (Done) | KEEP | — | Historical; unaffected. |
| DEC-002 D-023 (Riverpod non-codegen + go_router) | KEEP | — | Presentation-layer decision; v3 leaves UI stack to Flutter. |
| DEC-002 D-024–D-026 (Dart `lib/domain` layout, coverage gate, independence grep) | ADAPT | MVP-002 | Durable domain moves to Rust; the Dart gate keeps `lib/features` reporting; MVP-002 installs the cargo gate. |
| MVP-002 (pure-Dart domain spine) | DEFER → rewritten | this session | Superseded by the Rust kernel/SQLite foundation; same ID kept. |
| Firestore-as-durable-store assumptions (MVP-004, MVP-017, MVP-018, DEC-003) | DEFER | after DEC-005 | PRD v3 §6.4–6.5: SQLite in Rust is authoritative; cloud is an optional adapter. |
| DEC-004 naming clearance | KEEP untouched | owner-paced | Kimatta is the v3 brand name in the PRD; clearance is DEC-004's job. |
| Nothing | MIGRATE | — | There is no domain code to migrate. The "migration" is a build-out under new ownership rules. |

## 3. Ownership boundary and dependency direction

Flutter/Dart owns screens, navigation, accessibility, animations, transient form/view state, and platform adapters where the ecosystem is better there. Rust owns household/member identity used by control logic, durable food state, policies/constraints, control-relevant confidence/provenance, planner/scoring/search, shopping-list derivation, coverage/model-sufficiency assessment, the authoritative SQLite store, and planner/action/outcome audit records (PRD v3 §13, invariants 17 and 21).

```text
Flutter UI  ──generated FRB bridge (coarse DTO calls)──▶  Rust workspace  ──▶  SQLite (Rust-owned)

Crate dependency direction (kernel never imports food types):
  household-core  <-  food-domain  <-  kimatta-application  <-  kimatta-bridge
  household-core  <-  kimatta-storage  <-  kimatta-application (or kimatta-bridge until it exists)
```

`kimatta-storage` may import `food-domain` once it exists; never the reverse. Start with three crates — `household-core`, `kimatta-storage`, `kimatta-bridge` (MVP-002). `food-domain` appears with the first food entity (MVP-005/007); `kimatta-application` appears when a use case orchestrates more than one domain service (MVP-023). Empty crates are not created ahead of need.

## 4. Phase → card map (PRD v3 §17)

| PRD phase | Cards | Notes |
|---|---|---|
| 0 — inventory | this document | Done 2026-08-24. |
| 1 — Rust bridge spike | `PRE-002`, then `DEC-005` | Android debug + release proof; backend and core-language commitment. iOS packaging → `PRE-003` (post-launch, D-015). |
| 2 — kernel + SQLite | `MVP-002` | Identity, migrations, transactions, cargo gate. Other kernel primitives wait for a call site. |
| 3 — food state, incrementally | `MVP-005`, `MVP-007`, `MVP-012`, `MVP-014`, `MVP-015` (as amended) | Ingredient → recipe/stub → planned meal/component → pantry → shopping list. UI cards (`MVP-003/006/008/009/010/011/013/016`) consume bridge DTOs. |
| 4 — FoodController | `MVP-023`, `MVP-024` | Candidate generation, Tier-0 filtering, scoring, beam search, coverage assessment, ledger; Cover My Week UI. |
| 5 — hardening | `MVP-025`, `MVP-017`, `MVP-019` | Fixtures/property tests, benchmarks, recovery/backup, privacy-safe telemetry. |
| Cloud / sharing / release | `DEC-003`, `MVP-018`, `MVP-020`, `MVP-021`, `MVP-022` | Cloud is an optional adapter; no sync engine in MVP. |

Cards below PRE-002/DEC-005 carried a dated amendment banner from 2026-08-24. Twenty-one of them were re-derived in a single pass on 2026-08-26 (D-034), which D-029 permitted; `MVP-003` is re-derived by its own approved plan.

## 5. Kill criterion

PRD v3 §22: if a bounded spike cannot produce reliable release builds, preserve the interfaces and temporarily implement the core in Dart. `PRE-002` produces the evidence; `DEC-005` makes the call. The commitment is made on Android evidence only (D-015; declared deviation recorded in `docs/V3_IMPLEMENTATION_STATUS.md`); `PRE-003` re-opens it for iOS.

## 6. What NOT to build (carried from the prompt "What NOT to build" and "Minimal Household Control Kernel" lists, and PRD v3 §3 item 13, §9.1, §16)

Family Seasons; doctor/school/sports management; generalized household dashboard; global attention broker; calendar controller; generic agent framework; generic policy language/DSL; universal ontology or knowledge graph; POMDP framework; CP-SAT/OR-Tools; cloud sync engine; cross-household recommendation training; a second domain controller; recurring LLM planning; quantified pantry inventory; feeds, streaks, or engagement notifications. Preserve extension seams only.

## 7. Preservation rules for every later card

- Never delete functioning code merely because v3 assigns different ownership.
- Before changing a persisted schema, re-establish whether real user data exists and write a compatibility plan (none is needed today).
- Generated bridge files are never hand-edited.
- Owner work in progress is hashed before and after any step that could touch it. Guarded owner work means the decision cards, the PRD, and the v3 docs. `KNOWN_ISSUES.md` and `KNOWN_ISSUES-low.md` are excluded: they are review-workflow artifacts that `/redteam-code`, `/fix-findings`, `/ki-maintain` and `/tidy` write by design, so hashing them makes a card's evidence false as soon as a review pass runs (D-032).
