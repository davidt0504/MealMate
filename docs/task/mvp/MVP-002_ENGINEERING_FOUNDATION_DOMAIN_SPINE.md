# MVP-002 — Rust household kernel and SQLite foundation

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

> Retitled 2026-08-24 under D-028; formerly the pure-Dart domain spine. The ID and filename are kept because the register and five other cards cite `MVP-002`. Library/crate choices below are confirmed or revised at DEC-005. DEC-002 is not a dependency: it governs the Dart side (MVP-003 onward); this card only edits its command-contract block (AC-4).

| Field | Value |
|---|---|
| Status | Ready |
| Type | Implementation |
| Workstream | Domain foundation (Rust) |
| Depends on | PRE-002, DEC-005 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | None beyond the toolchain PRE-002 installed |

## Outcome and user value

Land the smallest Rust-owned, SQLite-backed foundation that later food-domain cards build on: typed household/member identity, a tested migration runner, and the bridge health call reading real schema state. Household ownership is the core data boundary (invariant 1); this card makes Rust its owner (invariant 17).

## Authoritative sources

- `docs/PRD_v3.md` §7.1 (household identity), §7.7 (audit ledger, deferred here), §12 (SQLite/schema discipline), §17 Phase 2
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §25 (kernel minimalism), §26 (Rust principles)
- `docs/ROADMAP.md` D-028, D-029, the DEC-005 resolution, the PRE-002 command contract
- `docs/task/MVP_INVARIANTS.md` 1, 17, 20, 21

## Load-bearing constraints

- The kernel crate never imports food-specific types (PRD §5 dependency rule).
- `#![forbid(unsafe_code)]` in the kernel and storage crates; unsafe stays isolated to generated bridge code.
- Foreign keys enabled on every connection; multi-record writes are transactions; explicit, tested migrations (PRD §12).
- No JSON blobs for domain relationships.
- Only primitives a call site needs: identity now; `Policy`, `OutcomeAssessment`, `ActionProposal`, `AttentionRequest`, and the planner ledger are deferred to MVP-023 (principles §25: second-domain evidence before universal abstractions).
- No user data exists yet (see `docs/V3_MIGRATION_PLAN.md`), so no compatibility migration is required; the migration runner is still tested from an empty database.

## Scope

- Turn `rust/` into a workspace holding `household-core`, `kimatta-storage`, and the PRE-002 bridge crate, without moving generated paths. Allowed edges: `kimatta-storage` → `household-core`; bridge → storage directly until `kimatta-application` exists.
- `household-core`: `HouseholdId` and `MemberId` newtypes that reject empty/whitespace input; `Household { id, name: Option<String> }`; `HouseholdMember { id, household_id, display_name }`.
- `kimatta-storage`: `open(path)` that enables foreign keys and applies migrations; migration v1 creating `household` and `household_member` (FK with cascade); `schema_version(&conn)`; a transactional `insert_household(household, members)`. Libraries as confirmed by DEC-005 (sketch: `rusqlite` with the `bundled` feature so Android builds do not need a system SQLite, `rusqlite_migration`).
- Bridge: `health_check` opens the database at the Dart-supplied path, applies migrations, and returns the schema version; storage errors map to the typed bridge error.
- Add the Rust gate to the DEC-002 command contract (the `lib/domain` lines were removed under D-028) (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`).

## Non-goals

- Food entities (MVP-005/007/012/014/015), Firebase, sync, planner, kernel primitives beyond identity, backup/export.

## Decision gates

- DEC-005 must be Done: it confirms the backend, the crate layout, and the storage libraries this card uses.

## Acceptance criteria

- **AC-1:** `cargo test --workspace` passes and covers: id validation (empty/whitespace rejected, valid round-trips, no silent trimming), migration from an empty database reaches v1, `MIGRATIONS.validate()`, foreign-key enforcement (orphan member rejected), transaction rollback when a later row fails, and idempotent reopen of an already-migrated file database (version unchanged, data kept). `cargo tree -i rusqlite` resolves exactly one version.
- **AC-2:** `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean; no `unsafe` in `household-core` or `kimatta-storage`.
- **AC-3:** The release APK with SQLite runs on the emulator and the health screen shows schema version 1. **D-027 block:** the emulator and adb bridge are owner-run on the Windows host (D-022); if unavailable after the bounded wait, `NOT VERIFIED` with discharge at MVP-004's persistence evidence, and Done requires the owner's `owner-accepted YYYY-MM-DD` clause.
- **AC-4:** The DEC-002 command contract in `docs/ROADMAP.md` carries the Rust gate, and the new gate has been exercised.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | `cargo test` output and `cargo tree -i rusqlite` output |
| AC-2 | Command output; `grep -rn unsafe rust/crates/household-core rust/crates/kimatta-storage` empty |
| AC-3 | Emulator screenshot via the D-022 bridge, or the D-027 block with owner acceptance |
| AC-4 | ROADMAP diff and gate output |

## Stop/failure conditions

- Stop on speculative abstraction (any kernel primitive without a call site), an invariant conflict, or a cross-compile failure that would require `unsafe` or hand-edited generated code.
- Two failed cycles return the card to planning.

## Handoff

Record evidence, the resolved library versions, and DOMAIN-READY progress in `docs/ROADMAP.md`; update `docs/V3_IMPLEMENTATION_STATUS.md`.
