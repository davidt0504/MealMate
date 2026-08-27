# MVP-012 — Planned meals and components

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Planning/domain |
| Depends on | MVP-004, MVP-005, MVP-007 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-012`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Persist a meal occurrence on a real date with one or more recipe components, optional serving scales, and a lock that later automation must respect.

## Authoritative sources

- `docs/PRD_v3.md` §8 (food domain model, required semantics), §9.4 (Tier-0 hard filtering), §12 (schema discipline), §16
- `docs/ROADMAP.md` D-005, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 5, 17, 18, 19, 20
- Historical (D-028): `docs/PRD_v2.md` §§7.7, 10.1–10.2, 14.4

## Load-bearing constraints

- Occurrence, date/slot, components, and component scaling are distinct concepts.
- `PlannedMeal`, `MealComponent` and `MealStub` are Rust-owned SQLite entities (invariant 17).
- Non-recipe meal types — leftovers, dining out or takeout, frozen/quick, freeform, and intentionally open or away — are first-class in the schema, not a later special case (PRD §8).
- Planned is not confirmed cooked; the schema represents them separately (invariant 19).
- A lock is a Tier-0 constraint that later automation may never trade away, never a score weight (invariant 18, PRD §9.7). MVP-023 consumes this contract.

## Scope

- Implement domain behavior, the migration and persistence layer, add/remove/reorder components, serving scale, lock state, the non-recipe meal types above at schema level, and contract tests.

## Non-goals

- Planner screen, automated suggestions, traditions, non-recipe-meal **UI**, or real-time collaboration conflict resolution.

## Decision gates

- Resolve component ordering/identity and recipe-deletion reference behavior before persistence is finalized.

## Acceptance criteria

- **AC-1:** Multi-component occurrences round-trip by real date and enabled slot.
- **AC-2:** Component scales and ordering remain stable through edits.
- **AC-3:** Lock behavior is explicit and tested for future automation contracts.
- **AC-4:** Household-scoped reads and writes succeed for the owning household and never return or mutate another household's rows.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Domain/repository integration tests |
| AC-2 | Edge-case/property tests |
| AC-3 | Contract tests written independently of UI |
| AC-4 | Rust household-scoping tests with a second-household fixture, plus fresh-context schema review |

## Stop/failure conditions

- Stop if the schema assumes one recipe per meal, collapses a whole plan into a single row, or represents a non-recipe meal as an absent recipe. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, and **Next implementation task**.
