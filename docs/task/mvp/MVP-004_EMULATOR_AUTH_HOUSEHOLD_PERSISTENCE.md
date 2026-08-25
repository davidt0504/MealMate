# MVP-004 — Emulator auth, household, and persistence

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Data/auth |
| Depends on | MVP-002 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Local Firebase Emulator Suite only; no cloud project/credentials |

> **v3 amendment (2026-08-24, D-028/D-029):** Durable persistence moves to Rust-owned SQLite (MVP-002); the Firebase emulator is not the durable store. This card narrows to anonymous-first entry and household identity UI over the Rust kernel. Authoritative source adds `docs/PRD_v3.md` §6.4–6.5, §15; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Let a new user enter anonymously and persist household-owned data against a safe local emulator boundary.

## Authoritative sources

- `docs/PRD_v2.md` §§7.2, 14.2, 14.4, 14.6; `docs/ROADMAP.md` D-008; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- First write must be protected by deny-by-default, household-scoped rules.
- Emulator configuration fails closed when absent; never fall through to a real project.
- Solo user is a one-person household; IDs are stable and ownership explicit.
- Durable-account upgrade is deferred to MVP-018.

## Scope

- Wire anonymous emulator auth, household bootstrap, a minimal repository implementation, rules/indexes, seed/reset tooling, and emulator/rules tests.

## Non-goals

- Real Firebase project, invitations, production auth providers, rich schema, or offline claim verification.

## Decision gates

- Resolve any rule/data-shape conflict with MVP-002 before migration-like code appears.

## Acceptance criteria

- **AC-1:** First launch creates/reuses an anonymous identity and one owned household.
- **AC-2:** Authorized household reads/writes pass and cross-household/unauthenticated access fails.
- **AC-3:** Restart retains expected emulator-backed state without duplicate bootstrap records.
- **AC-4:** Missing emulator configuration cannot contact a cloud backend.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Integration test and emulator data inspection |
| AC-2 | Automated rules tests, including negative cases |
| AC-3 | Restart integration test |
| AC-4 | Fail-closed test/network log; independent security review |

## Stop/failure conditions

- Stop on any real-project connection, permissive rule workaround, credential request, or ambiguous ownership. Two cycles then re-plan.

## Handoff

Record PASS/FAIL/NOT VERIFIED evidence and status in `docs/ROADMAP.md`.
