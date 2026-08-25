# MVP-009 — Restriction warnings and filtering

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Safety/preferences |
| Depends on | MVP-006, MVP-007, MVP-008 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None; no medical claims |

> **v3 amendment (2026-08-24, D-028/D-029):** Restriction conflicts are Tier-0 hard filtering in the Rust planner, never score weights (invariant 18); unknown safety data stays unknown (invariant 19). Authoritative source adds `docs/PRD_v3.md` §9.4; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Warn and filter based on known structured ingredients while making uncertainty visible, so users can avoid obvious conflicts without receiving false safety assurance.

## Authoritative sources

- `docs/PRD_v2.md` §§7.4, 10.5, 11.4; `docs/ROADMAP.md` D-012; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Never label a recipe “safe” or infer absence from missing/ambiguous data.
- Match only explicit, explainable mappings; retain an Unknown/Needs review state.
- Warnings identify the known ingredient/reason and do not replace user judgment.

## Scope

- Define restriction vocabulary/mappings, deterministic evaluation, warning/filter UI, uncertainty copy, and adversarial tests.

## Non-goals

- Medical advice, cross-contamination claims, exhaustive allergen ontology, LLM classification, or nutrition scoring.

## Decision gates

- Use `deep-options` if restriction taxonomy/source or hard-filter versus warn behavior cannot meet the PRD without owner tradeoff.

## Acceptance criteria

- **AC-1:** Known conflicts generate an explainable warning and are excluded where a hard filter is requested.
- **AC-2:** Unknown/ambiguous ingredients remain visible as uncertainty, never negative assurance.
- **AC-3:** Changing household restrictions deterministically updates results.
- **AC-4:** Safety language contains no unsupported “safe/free-from” claim.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Tests-first conflict matrix |
| AC-2 | Adversarial/unknown fixtures |
| AC-3 | State/integration test |
| AC-4 | Independent copy and gap review |

## Stop/failure conditions

- Stop on false-negative concealment, medical positioning, or untraceable classification. Two cycles then re-plan.

## Handoff

Record evidence/status and known limitations in `docs/ROADMAP.md`.
