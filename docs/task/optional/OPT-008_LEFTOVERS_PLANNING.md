# OPT-008 — Leftovers: plan cook-once-eat-twice, or turn leftovers off

> Scoping card (D-042). The design is not resolved here; a dedicated `grill-me` session resolves the decision gates below and folds its answers into this card before `plan-task`.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Optional post-MVP implementation |
| Workstream | Planner policy |
| Depends on | MVP-023, MVP-024, MVP-025 |
| Recommended before | OPT-007 (its reshuffle should exist before leftover nights change what a slot holds); not a hard dependency |
| Complexity | Complex |
| Assurance | Elevated (planner policy) |
| Sequential batching | No |
| Recommended workflow | `grill-me` (fold answers into this card) → `plan-task` (no `--auto`) |
| External actions | None |

## Outcome and user value

A household that cooks big and eats twice gets a week that says so: a cook night followed by a
leftovers night, with the shopping list scaled accordingly. A household that never eats leftovers
can say so once and never see them proposed. Owner note 2026-09-08: "anticipate leftovers better
(maybe have a way to turn off leftovers) instead of a whole meal everyday."

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in
`docs/task/README.md` for `OPT-008`. **Planning may not start until every decision gate below
records an owner answer** from a `grill-me` session.

## What already exists — read before designing

- `candidates.rs`: a recipe *can feed leftovers* when `servings × scale ≥ members + 1`; unknown servings never qualify. A `leftovers` candidate is available for a slot when a prior slot the same day holds such a dish (`leftovers_sourced_from_stored`, `leftovers_possible`). With dinner-only scope there is no "prior slot the same day", so the path is effectively dormant.
- `leftovers` is also a manual fallback kind in the Swap picker (`cover_screen.dart:433`).
- Tier 5 scoring already has a leftovers term next to variety, reuse and pantry.
- Household has members but no meal-size, appetite or leftovers preference; recipes carry `servings`.
- Planned is not cooked (invariant 19): a leftovers night is a proposal that the cook night happened, and coverage grading must say so.

## Decision gates (the grill-me agenda)

1. **Off switch first?** A household-level "propose leftovers: yes/no" (default?) is the smallest deliverable and may be all the beta needs. Lean: ship the switch in this card regardless of the rest; the planner already has the term to gate.
2. **Cross-day leftovers.** Today leftovers only chain within a day. The note wants "cook Monday, eat Tuesday". Rule candidates: a dish that feeds ≥ 2 × members may fill the *next* dinner as leftovers; or a household-set cadence ("we eat leftovers N nights a week"); or only when the recipe is flagged "keeps well". Lean: servings-based rule with a household cap N per week.
3. **What is the household size for the arithmetic?** `members` count vs a declared "usually cook for" number (guests, small children). Lean: declared number, defaulting to members.
4. **Shopping list effect.** A leftovers night adds no ingredients; the cook night's scale must be enough for both. Does the planner raise `scale` on the cook night, or only pick recipes whose base servings already cover it? Scale changes the list (invariant 17 fixtures).
5. **Coverage and attention wording.** A leftovers night is covered *conditionally* on the cook night; how the card says that without overstating certainty (PRD v3 §2.2).
6. **Interaction with OPT-007.** "Show me another" on a leftovers night: re-propose the cook night, or replace the leftovers night with a fresh meal? Reshuffle respects leftover pairs as units?

## Load-bearing constraints

- Deterministic search; policy is data in `FoodPolicies`, not branches in the beam.
- Planned ≠ cooked; coverage never claims a leftover exists.
- Locks and vetoes are unaffected; a locked cook night can still source a leftovers night.
- Scope control (breakfast/lunch, OPT-004) is a separate card; this card assumes dinner-only or works for any scope without depending on OPT-004.

## Scope (to be refined after the gates close)

- Household preference storage and bridge (one migration); `FoodPolicies` leftovers policy; candidate generation across days; scoring adjustments; coverage wording; Cover card rendering of a leftovers night; MVP-025 fixtures extended.

## Non-goals

- Tracking whether leftovers were actually eaten; expiry; portion arithmetic beyond servings × scale; freezer inventory.

## Acceptance criteria (draft)

- **AC-1:** With leftovers off, no proposal contains a leftovers component, and the fixture plans that previously did now cover those slots otherwise or raise attention.
- **AC-2:** With leftovers on and a qualifying recipe, the planner proposes cook-night → leftovers-night pairs under the chosen rule, deterministically.
- **AC-3:** The shopping list for a week with a leftovers night contains the cook night's scaled quantities and nothing for the leftovers night.
- **AC-4:** The Cover card states the conditional nature of a leftovers night in words the beta tester reads correctly.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1, AC-2 | `food-domain` planner tests extending MVP-025 fixtures; determinism test |
| AC-3 | `derive_shopping_list` fixture with a paired week |
| AC-4 | Owner and beta-tester device check |

## Stop/failure conditions

- Stop on any nondeterminism, on a coverage grade that overstates certainty, or on an invariant conflict. Two cycles then defer.

## Handoff

Standard `docs/ROADMAP.md` handoff.
