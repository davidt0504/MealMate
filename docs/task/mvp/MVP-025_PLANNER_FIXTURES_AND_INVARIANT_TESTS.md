# MVP-025 — Planner fixtures and invariant tests

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Planning/quality |
| Depends on | MVP-023 |
| Complexity | TBD |
| Assurance | TBD |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None (derive after DEC-005) |

## Outcome and user value

Create version-controlled synthetic household fixtures (cold start, busy week, many locked meals, multiple dislikes, restrictions set/skipped, sparse pantry, repetitive library, leftovers/fallbacks, intentionally open nights), property/invariant tests (hard restriction never selected, locked meal never moves, determinism, monotonic constraints, no negative shopping quantities, historical records never mutated), and a beam-width/candidate-limit benchmark. Assert invariants and quality thresholds, not one exact plan.

## Authoritative sources

- `docs/PRD_v3.md` §18 (testing strategy), §9.6 (beam width must be benchmarked)
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Tests"
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §28
- `docs/task/MVP_INVARIANTS.md` 18, 20

Body to be derived after DEC-005 Done (D-029).
