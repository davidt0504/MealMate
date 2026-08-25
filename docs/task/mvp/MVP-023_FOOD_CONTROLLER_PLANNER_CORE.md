# MVP-023 — FoodController planner core

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Planning/domain (Rust) |
| Depends on | MVP-002, MVP-009, MVP-012, MVP-014 |
| Complexity | TBD |
| Assurance | TBD |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None (derive after DEC-005) |

## Outcome and user value

Implement the deterministic local Cover My Week controller in Rust: candidate generation for every unresolved enabled slot, Tier-0 hard filtering, inspectable feature scoring with reason codes, whole-cycle beam search under the lexicographic tiers, coverage/model-sufficiency assessment, attention requests, and an append-oriented planner ledger. The kernel primitives it needs (`Policy`, `OutcomeAssessment`, `ActionProposal`, `AttentionRequest`) land here, not earlier (principles §25).

## Authoritative sources

- `docs/PRD_v3.md` §7 (kernel primitives), §9 (algorithm v1), §10 (model sufficiency), §11 (attention governor)
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Planner implementation", "Attention/model-sufficiency behavior"
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §4–§9, §13–§15
- `docs/task/MVP_INVARIANTS.md` 9, 18, 19, 20

Body to be derived after DEC-005 Done (D-029).
