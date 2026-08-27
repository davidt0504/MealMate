# MVP-007 — Ingredient and recipe data foundation

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Recipes/domain |
| Depends on | MVP-002, MVP-004 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-007`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Store household recipes and ingredients in a structured, evolvable form that retains what the user entered and supports safe later filtering and shopping math.

## Authoritative sources

- `docs/PRD_v3.md` §8 (food domain model and required semantics), §12 (SQLite/schema discipline), §16 (required food domain)
- `docs/ROADMAP.md` D-005, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 2, 10, 17, 19, 21
- Historical (D-028): `docs/PRD_v2.md` §§7.5–7.6, 10.3–10.4, 14.4

## Load-bearing constraints

- Keep original ingredient text alongside parsed identity, amount, unit, preparation, and optionality.
- Unknown/ambiguous values remain representable; parsing never fabricates certainty.
- Recipes are household-owned with provenance/publication metadata sufficient for later rights boundaries.
- `Ingredient`, `CustomIngredient`, `IngredientLine`, `Recipe` and `RecipeProvenance` are Rust-owned SQLite entities in normalized tables with stable immutable IDs and foreign keys enabled; JSON is used only for bounded, versioned metadata, never as a substitute for relational design (PRD §8, §12, invariant 17).
- No generalized sync engine exists in the MVP (PRD §6.5), so record granularity is driven by query and migration needs, not by conflict-resolution speculation.

## Scope

- Complete recipe/ingredient domain types in Rust, the migration and persistence mapping, validation, the repository layer, coarse bridge DTOs, and round-trip tests.
- Per `docs/V3_MIGRATION_PLAN.md` §3 the `food-domain` crate arrives with the first food entity (MVP-005/007); whichever card executes first creates it. This card does not assume it already exists.

## Non-goals

- Full parser, URL import, public publishing, photos, recommendation engine, or shopping aggregation.

## Decision gates

- Use `deep-options` during planning if ingredient identity/unit representation or recipe component shape remains consequentially ambiguous.

## Acceptance criteria

- **AC-1:** Valid recipes with structured and original ingredients round-trip without loss.
- **AC-2:** Unknown, optional, fractional, range, and preparation cases remain honest and recoverable.
- **AC-3:** Household-scoped reads and writes succeed for the owning household and never return or mutate another household's rows.
- **AC-4:** Schema supports MVP-009, MVP-012, and MVP-015 without parallel-array coupling.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Property/round-trip tests |
| AC-2 | Independent edge-case fixture tests |
| AC-3 | Rust household-scoping tests with a second-household fixture, including negative cases |
| AC-4 | Fresh-context schema review against downstream cards |

## Stop/failure conditions

- Stop if original data would be discarded, uncertainty hidden, or a vendor type enters the domain. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, and **Next implementation task**.
