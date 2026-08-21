# MVP-007 — Ingredient and recipe data foundation

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Recipes/domain |
| Depends on | MVP-002, MVP-004 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

## Outcome and user value

Store household recipes and ingredients in a structured, evolvable form that retains what the user entered and supports safe later filtering and shopping math.

## Authoritative sources

- `docs/PRD_v2.md` §§7.5–7.6, 10.3–10.4, 14.4; `docs/ROADMAP.md` D-005; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Keep original ingredient text alongside parsed identity, amount, unit, preparation, and optionality.
- Unknown/ambiguous values remain representable; parsing never fabricates certainty.
- Recipes are household-owned with provenance/publication metadata sufficient for later rights boundaries.
- Keep Firestore records granular enough for future shared-state conflict safety.

## Scope

- Complete recipe/ingredient domain types, persistence mapping, validation, repository implementation, indexes/rules, and round-trip tests.

## Non-goals

- Full parser, URL import, public publishing, photos, recommendation engine, or shopping aggregation.

## Decision gates

- Use `deep-options` during planning if ingredient identity/unit representation or recipe component shape remains consequentially ambiguous.

## Acceptance criteria

- **AC-1:** Valid recipes with structured and original ingredients round-trip without loss.
- **AC-2:** Unknown, optional, fractional, range, and preparation cases remain honest and recoverable.
- **AC-3:** Household rules reject unauthorized access.
- **AC-4:** Schema supports MVP-009, MVP-012, and MVP-015 without parallel-array coupling.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Property/round-trip tests |
| AC-2 | Independent edge-case fixture tests |
| AC-3 | Rules tests |
| AC-4 | Fresh-context schema review against downstream cards |

## Stop/failure conditions

- Stop if original data would be discarded, uncertainty hidden, or a vendor type enters the domain. Two cycles then re-plan.

## Handoff

Record evidence/status in `docs/ROADMAP.md`.
