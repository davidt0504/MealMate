# MVP-015 — Shopping aggregation engine

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Shopping/domain |
| Depends on | MVP-012, MVP-014 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-015`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Generate a trustworthy pantry-aware shopping projection from planned meal components without inventing unsafe unit conversions.

## Authoritative sources

- `docs/PRD_v3.md` §13 (Rust owns shopping-list derivation), §9.5 (pantry fit), §16
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Shopping-list behavior"
- `docs/ROADMAP.md` D-012, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 3, 6, 17, 20
- Historical (D-028): `docs/PRD_v2.md` §§7.9, 10.7, 24.7

## Load-bearing constraints

- Apply per-component serving scale before aggregation.
- Subtract only explicit pantry presence and preserve user/manual lines. Pantry presence may suppress a purchase but never proves sufficient quantity (invariant 6, pivot prompt).
- Combine only identical ingredients with explicitly compatible unit families; incompatible/uncertain lines remain separate with original text.
- Derivation runs in Rust and is deterministic and reproducible for the same input snapshot and algorithm version (invariants 17, 20).
- A user can restore a line the derivation omitted; omission is never silent and never final (pivot prompt).
- Grouping by store category is part of the derivation contract, not a presentation afterthought.
- A derived quantity is never negative.

## Scope

- Write acceptance-derived tests first, implement the projection and aggregation in Rust, category-assignment seams, provenance/explanation, coarse bridge DTOs, and performance bounds.

## Non-goals

- Shopping UI, prices, store inventory, nutrition math, fuzzy/LLM conversion, or quantified pantry subtraction.

## Decision gates

- Use `deep-options` if supported unit families/conversion precision cannot be bounded simply.

## Acceptance criteria

- **AC-1:** Planned component scaling yields correct ingredient demand.
- **AC-2:** Pantry-present identities are omitted without assuming completeness.
- **AC-3:** Compatible units combine correctly; incompatible/ambiguous inputs remain separate and recoverable.
- **AC-4:** Reordering inputs does not change semantic output and provenance explains each line.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Independent tests-first fixture matrix |
| AC-2 | Pantry partial/empty fixtures |
| AC-3 | Property/boundary tests for all supported units |
| AC-4 | Determinism/provenance tests plus fresh-context gap review |

## Stop/failure conditions

- Stop on guessed conversions, discarded originals, float precision ambiguity, or hidden input dependence. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, supported conversion families, resulting status, delivery-gate progress, and **Next implementation task**.
