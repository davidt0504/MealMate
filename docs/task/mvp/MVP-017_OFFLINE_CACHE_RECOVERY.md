# MVP-017 — Offline cache and recovery

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Reliability/offline |
| Depends on | MVP-008, MVP-009, MVP-013, MVP-014, MVP-016 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Emulator/network controls only |

> **v3 amendment (2026-08-24, D-028/D-029):** SQLite is local-first by construction; this card narrows to restart/recovery, backup/export, and cache behavior for optional cloud adapters. The "Firestore offline persistence is the MVP mechanism" constraint is superseded. Authoritative source adds `docs/PRD_v3.md` §12 "Backup/export", §6.5; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Make the complete recipe → plan → pantry-aware list → shopping loop dependable through restart and connectivity loss.

## Authoritative sources

- `docs/PRD_v2.md` §§7.1, 14.3–14.4, 19.2, 24.8–24.10; `docs/ROADMAP.md` D-011, D-012; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Deliberately prefetch/cache required household data; do not assume documents happened to be read.
- Firestore offline persistence is the MVP mechanism; no second database without measured evidence and decision.
- Pending, rejected, and reconciled writes are visible; destructive outcomes and sign-out risk are explicit.
- Scope excludes multi-user conflict resolution beyond granular records.

## Scope

- Define offline readiness, prefetch, connection/pending/error states, retry/recovery, and a deterministic failure test harness across all core features.

## Non-goals

- Full sync engine, collaborative merge UI, background guarantees the OS cannot provide, or uncached public shares.

## Decision gates

- If Firestore persistence cannot meet a measured acceptance case, stop for `deep-options`; do not quietly add local storage.

## Acceptance criteria

- **AC-1:** After deliberate prefetch and restart, recipes, active cycle/plan, pantry, and list remain usable offline.
- **AC-2:** Offline create/edit/check actions queue and reconcile correctly after reconnect.
- **AC-3:** Rejected reconnect writes, deletions, uncached records, and retry paths are truthful and recoverable.
- **AC-4:** Sign-out/identity transition with pending writes cannot silently lose data.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Android emulator restart/disconnect scenario matrix |
| AC-2 | Automated queued-write/reconnect tests |
| AC-3 | Fault-injection tests and user-visible-state inspection |
| AC-4 | Pending-write/sign-out tests plus fresh-context coverage review |

## Stop/failure conditions

- Stop on silent data loss, misleading offline claim, or need for a second database. Two cycles then re-plan.

## Handoff

Record scenario-by-scenario evidence and LOCAL-CORE-LOOP-READY status in `docs/ROADMAP.md`; promote DEC-003 when Done.
