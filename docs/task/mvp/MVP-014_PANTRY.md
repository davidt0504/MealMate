# MVP-014 — Pantry

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Pantry |
| Depends on | MVP-003, MVP-004, MVP-007 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below. Pantry semantics are unchanged by v3; only ownership and the shopping contract are restated.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-014`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Let users optionally mark ingredients as present so shopping lists omit known pantry items without demanding inventory maintenance.

## Authoritative sources

- `docs/PRD_v3.md` §8 (food domain model; pantry is simple and optional), §9.5 (pantry fit as a score component), §16
- `docs/ROADMAP.md` D-011, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 6, 17, 19
- Historical (D-028): `docs/PRD_v2.md` §§7.8–7.9, 10.6, 24.4

## Load-bearing constraints

- State is binary have/don't-have; no quantities, expiry, audits, or completeness claim.
- Missing pantry data means unknown, not absent.
- Identity matching is explicit and users can override state easily.
- `PantryItem` is a Rust-owned SQLite entity (invariant 17).
- Pantry presence may suppress a purchase but never proves sufficient quantity, and a pantry-aware list never claims stock completeness (invariant 19, PRD §10). MVP-015 and MVP-023 consume this contract.

## Scope

- Household pantry records in Rust, browse/search/toggle UI over coarse bridge commands, recipe-context toggles where useful, and tests.

## Non-goals

- Inventory counts, barcode scan, depletion prediction, receipt import, or mandatory onboarding.

## Decision gates

- None unless ingredient identity cannot support predictable toggles.

## Acceptance criteria

- **AC-1:** Users can mark/unmark ingredient identity and state persists by household.
- **AC-2:** An empty/incomplete pantry never blocks recipe, planning, or shopping flows.
- **AC-3:** Household-scoped reads and writes never return or mutate another household's rows, and accessible controls expose current state.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Repository/widget/integration tests |
| AC-2 | Empty/partial-state tests |
| AC-3 | Rust household-scoping tests with a second-household fixture, plus accessibility check |

## Stop/failure conditions

- Stop if scope becomes quantified inventory or unknown is treated as don't-have. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, and **Next implementation task**.
