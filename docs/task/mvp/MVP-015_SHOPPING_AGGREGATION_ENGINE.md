# MVP-015 — Shopping aggregation engine

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Shopping/domain |
| Depends on | MVP-012, MVP-014 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

## Outcome and user value

Generate a trustworthy pantry-aware shopping projection from planned meal components without inventing unsafe unit conversions.

## Authoritative sources

- `docs/PRD_v2.md` §§7.9, 10.7, 24.7; `docs/ROADMAP.md` D-012; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Apply per-component serving scale before aggregation.
- Subtract only explicit pantry presence and preserve user/manual lines.
- Combine only identical ingredients with explicitly compatible unit families; incompatible/uncertain lines remain separate with original text.
- Output is deterministic and explainable.

## Scope

- Write acceptance-derived tests first, implement pure-Dart projection/aggregation, category assignment seams, provenance/explanation, and performance bounds.

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

Record evidence/status and supported conversion families in `docs/ROADMAP.md`.
