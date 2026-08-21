# MVP-014 — Pantry

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Pantry |
| Depends on | MVP-003, MVP-004, MVP-007 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

## Outcome and user value

Let users optionally mark ingredients as present so shopping lists omit known pantry items without demanding inventory maintenance.

## Authoritative sources

- `docs/PRD_v2.md` §§7.8–7.9, 10.6, 24.4; `docs/ROADMAP.md` D-011; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- State is binary have/don't-have; no quantities, expiry, audits, or completeness claim.
- Missing pantry data means unknown, not absent.
- Identity matching is explicit and users can override state easily.

## Scope

- Household pantry records, browse/search/toggle UI, recipe-context toggles where useful, rules, and tests.

## Non-goals

- Inventory counts, barcode scan, depletion prediction, receipt import, or mandatory onboarding.

## Decision gates

- None unless ingredient identity cannot support predictable toggles.

## Acceptance criteria

- **AC-1:** Users can mark/unmark ingredient identity and state persists by household.
- **AC-2:** An empty/incomplete pantry never blocks recipe, planning, or shopping flows.
- **AC-3:** Unauthorized access fails and accessible controls expose current state.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Repository/widget/integration tests |
| AC-2 | Empty/partial-state tests |
| AC-3 | Rules tests and accessibility check |

## Stop/failure conditions

- Stop if scope becomes quantified inventory or unknown is treated as don't-have. Two cycles then re-plan.

## Handoff

Record evidence/status in `docs/ROADMAP.md`.
