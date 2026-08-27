# MVP-009 — Restriction warnings and filtering

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Safety/preferences |
| Depends on | MVP-006, MVP-007, MVP-008 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None; no medical claims |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-009`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Warn and filter based on known structured ingredients while making uncertainty visible, so users can avoid obvious conflicts without receiving false safety assurance.

## Authoritative sources

- `docs/PRD_v3.md` §9.4 (hard filtering), §9.7 (lexicographic tiers), §10 (model sufficiency), §16
- `docs/ROADMAP.md` D-012, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 10, 18, 19
- Historical (D-028): `docs/PRD_v2.md` §§7.4, 10.5, 11.4

## Load-bearing constraints

- Never label a recipe “safe” or infer absence from missing/ambiguous data.
- Match only explicit, explainable mappings; retain an Unknown/Needs review state.
- Warnings identify the known ingredient/reason and do not replace user judgment.
- A restriction conflict is a Tier-0 hard constraint, never a score weight, and no lower tier may compensate for it (invariant 18, PRD §9.4, §9.7).
- Absence of detected allergen data is not proof of safety (PRD §9.4). If restriction setup was skipped, the product may not imply restriction verification anywhere (PRD §10, invariant 19).

## Scope

- Define restriction vocabulary/mappings, deterministic evaluation, warning/filter UI, uncertainty copy, and adversarial tests.

## Non-goals

- Medical advice, cross-contamination claims, exhaustive allergen ontology, LLM classification, or nutrition scoring.

## Decision gates

- Use `deep-options` if restriction taxonomy/source or hard-filter versus warn behavior cannot meet the PRD without owner tradeoff.

## Acceptance criteria

- **AC-1:** Known conflicts generate an explainable warning and are excluded where a hard filter is requested. The conflict set this card computes is the same set MVP-023 rejects at Tier 0 — one vocabulary, two consumers.
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

In one `docs/ROADMAP.md` handoff edit, record the evidence, known limitations, resulting status, delivery-gate progress, and **Next implementation task**.
