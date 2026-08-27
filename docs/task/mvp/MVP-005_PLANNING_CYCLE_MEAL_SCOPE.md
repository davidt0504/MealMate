# MVP-005 — Planning cycle and meal scope

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Planning domain |
| Depends on | MVP-002, MVP-004 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-005`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Represent and persist a household planning rhythm using real dates, dinner-first defaults, and optional breakfast/lunch scope.

## Authoritative sources

- `docs/PRD_v3.md` §8 "Civil dates" and required semantics, §12 (schema/migration discipline), §16 (required food domain)
- `docs/ROADMAP.md` D-005, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 4, 17, 20
- Historical (D-028): `docs/PRD_v2.md` §§7.3–7.4, 10.1–10.2, 24

## Load-bearing constraints

- A cycle is anchored to household shopping/planning rhythm, never weekday assumptions.
- Dinner defaults on; breakfast/lunch are opt-in and independently representable.
- The planning-cycle entity is Rust-owned in SQLite; Flutter never mutates it directly (invariant 17).
- Conceptual meal dates are civil dates, never UTC instants — "Tuesday dinner" must not shift under timezone conversion (PRD §8). DST and locale boundaries are explicit.

## Scope

- Implement cycle/scope persistence, civil-date calculations, defaults, validation, and focused Rust tests, exposed to Flutter through coarse bridge DTOs.
- Per `docs/V3_MIGRATION_PLAN.md` §3 the `food-domain` crate arrives with the first food entity (MVP-005/007); whichever card executes first creates it. This card does not assume it already exists.

## Non-goals

- Planner UI, recommendations, recurring traditions, or notifications.

## Decision gates

- Choose the civil-date representation in Rust. PRD §6.6 lists `jiff` as a *likely* choice adopted at its first call site, not a decision this card inherits; confirm or replace it with the version resolved at implementation.

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

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, EMULATOR-PERSISTENCE-READY progress, and **Next implementation task**.
