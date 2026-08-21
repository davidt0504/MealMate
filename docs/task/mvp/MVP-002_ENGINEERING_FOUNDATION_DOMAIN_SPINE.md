# MVP-002 — Engineering foundation and domain spine

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Domain foundation |
| Depends on | MVP-001, DEC-002 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | None |

## Outcome and user value

Create a small, pure-Dart domain spine and test seams so later features share correct concepts without premature infrastructure.

## Authoritative sources

- `docs/PRD_v2.md` §§7, 10, 14; `docs/ROADMAP.md` D-005, D-012; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Domain types cannot depend on widgets or Firebase.
- Model household ownership, stable IDs, structured/original ingredients, real-date cycles, planned meal components, pantry presence, and entitlement capability without speculative fields.
- Repository interfaces and in-memory fakes exist only for near-term workflows; no production database.
- Entitlement defaults must not accidentally grant paid or unsafe capability.

## Scope

- Implement core value objects/entities, validation, repository contracts, fakes, serialization boundaries where needed, and contract/unit tests.

## Non-goals

- UI, Firebase schema, sync, recommendations, full billing model, or feature-complete repositories.

## Decision gates

- Resolve any domain ambiguity that would change three or more downstream cards before coding it.

## Acceptance criteria

- **AC-1:** Core invariants are represented and invalid states are rejected or explicit.
- **AC-2:** Repository contracts support the next emulator-backed workflows without vendor leakage.
- **AC-3:** Pure-Dart tests cover identities, dates, ingredient preservation, meal components, pantry state, and entitlement defaults.
- **AC-4:** Architecture remains thin and dependency rules pass.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Domain test matrix |
| AC-2 | Contract tests using fakes |
| AC-3 | Test output and coverage-gap review |
| AC-4 | Analyze/dependency inspection by fresh-context verifier |

## Stop/failure conditions

- Stop on speculative abstraction, invariant conflict, or need to select production persistence. Two failed cycles return to planning.

## Handoff

Record evidence/status and DOMAIN-READY progress in `docs/ROADMAP.md`.
