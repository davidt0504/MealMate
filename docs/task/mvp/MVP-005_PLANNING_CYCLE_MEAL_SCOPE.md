# MVP-005 — Planning cycle and meal scope

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Planning domain |
| Depends on | MVP-002, MVP-004 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

## Outcome and user value

Represent and persist a household planning rhythm using real dates, dinner-first defaults, and optional breakfast/lunch scope.

## Authoritative sources

- `docs/PRD_v2.md` §§7.3–7.4, 10.1–10.2, 24; `docs/ROADMAP.md` D-005; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- A cycle is anchored to household shopping/planning rhythm, never weekday assumptions.
- Dinner defaults on; breakfast/lunch are opt-in and independently representable.
- Date/time-zone behavior must be explicit around DST and locale boundaries.

## Scope

- Implement cycle/scope persistence, date calculations, defaults, validation, and focused tests.

## Non-goals

- Planner UI, recommendations, recurring traditions, or notifications.

## Decision gates

- Explore date/time representation if current Dart APIs cannot preserve the invariant simply.

## Acceptance criteria

- **AC-1:** A household can save/load its cycle anchor and enabled meal slots.
- **AC-2:** Generated cycle dates are correct across month/year, locale, and DST boundaries.
- **AC-3:** Defaults are dinner-only without forcing setup.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Repository/integration tests |
| AC-2 | Independent boundary-case tests written from the criteria |
| AC-3 | Unit test and bounded UI/state inspection |

## Stop/failure conditions

- Stop if implementation hard-codes a week or silently converts calendar dates through UTC. Two cycles then re-plan.

## Handoff

Record evidence/status and EMULATOR-PERSISTENCE-READY progress in `docs/ROADMAP.md`.
