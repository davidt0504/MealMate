# MVP-017 — Offline durability and recovery

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

> Retitled 2026-08-26 under D-034; formerly "Offline cache and recovery". The ID and filename are kept because the register and two other cards cite `MVP-017`. There is no cache to manage once SQLite is the local source of truth; what remains is durability, recovery, and backup.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Reliability/offline |
| Depends on | MVP-008, MVP-009, MVP-013, MVP-014, MVP-016 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Android emulator and network controls only |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-017`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Make the complete recipe → plan → pantry-aware list → shopping loop dependable through restart, process death, and a missing network — and give the household a way to get its data back out.

## Authoritative sources

- `docs/PRD_v3.md` §12 (SQLite discipline; "Backup/export"), §6.4–6.5 (local-first; cloud is optional), §14 item 7 (auth/sync failure cannot break local core planning), §16
- `docs/ROADMAP.md` D-011, D-012, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 7, 8, 17
- Historical (D-028): `docs/PRD_v2.md` §§7.1, 14.3–14.4, 19.2, 24.8–24.10

## Load-bearing constraints

- SQLite owned by Rust is local-first **by construction**: the core loop is offline by default, not by a caching strategy layered over a remote store, and a second durable store stays out of scope (PRD §6.4, invariant 17).
- Multi-record state transitions are transactions; an interrupted write leaves the database consistent, never half-applied (PRD §12).
- Versioned export and a validated restore are in scope: a system that succeeds at being external memory has a higher duty to protect that memory (PRD §12 "Backup/export").
- Destructive outcomes and any data-loss risk are stated honestly to the user (invariant 8).
- Cloud adapters are optional and out of the core loop; an adapter failure may degrade an optional feature but can never break local planning (PRD §6.5, §14 item 7).
- Scope excludes multi-user conflict resolution.

## Scope

- Define and verify restart and recovery behavior across the core loop: recipes, active cycle and plan, pantry, and shopping list.
- Database integrity after interrupted writes, a corrupt database file, and a missing database file.
- Versioned backup/export and a restore that is validated, not merely produced.
- Honest error, retry, and recovery states in the UI.
- A deterministic fault-injection harness so each of the above is reproducible.

## Non-goals

- Sync engines, collaborative merge UI, background guarantees the OS cannot provide, quantified conflict resolution.
- **Transferred out:** the sign-out / identity-transition data-safety obligation this card previously carried as AC-4 moves to `MVP-018` (recorded there as its new AC-6). No durable account exists at this card, so the obligation cannot be tested here; it is not dropped.

## Decision gates

- Choose the export format and its versioning. If backup or restore appears to need encryption or key management, stop for `deep-options` — PRD §14 item 10 forbids inventing a crypto scheme, and no archive format is decided here.

## Acceptance criteria

- **AC-1:** After restart with no network available, recipes, the active cycle and plan, pantry, and the shopping list remain usable.
- **AC-2:** Create, edit, and check-off actions commit locally and survive process death mid-transaction with no partial write.
- **AC-3:** Fault injection over an interrupted write, a corrupt database file, and a missing database file produces truthful, recoverable user-visible state.
- **AC-4:** A versioned export and its restore round-trip validate — the restored database is equivalent to the source and the export records its schema version.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Android emulator restart matrix with the network disabled |
| AC-2 | Automated process-death tests asserting transactional integrity |
| AC-3 | Fault-injection tests plus user-visible-state inspection |
| AC-4 | Export/restore round-trip test with a schema-version assertion, plus fresh-context coverage review |

## Stop/failure conditions

- Stop on silent data loss, a misleading offline claim, an invented archive or crypto scheme, or any proposal to add another durable store alongside SQLite. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record scenario-by-scenario evidence, the resulting status, LOCAL-CORE-LOOP-READY progress, and **Next implementation task**; select `DEC-003` next only if this card is `Done` and `DEC-003` passes its decision workflow gate.
