# OPT-006 — Pantry that feels like a pantry, not a list

> Scoping card (D-042). The design is not resolved here; a dedicated `grill-me` session resolves the decision gates below and folds its answers into this card before `plan-task`.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Optional post-MVP implementation |
| Workstream | Pantry and shopping |
| Depends on | MVP-014, MVP-015, MVP-016, MVP-034 |
| Complexity | Complex |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `grill-me` (fold answers into this card) → `plan-task` |
| External actions | None |

## Outcome and user value

Opening Pantry should feel like looking at the shelves at home: what is there, what is a staple,
what is running out, and nothing else. Today it is one flat `SwitchListTile` per catalog and
custom ingredient (160 rows and growing) with an unlabeled toggle. Owner note 2026-09-08:
"Pantry UI is terrible … design the pantry to feel more like navigating a pantry at home and not
like a raw list." The governing constraint, stated by the owner: **the pantry must never become a
chore.**

This card absorbs `OPT-003` (staples). OPT-003's storage, bridge and derivation design stays the
reference implementation for the staple *flag*; its list-interaction model is reopened below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in
`docs/task/README.md` for `OPT-006`. **Planning may not start until every decision gate below
records an owner answer** from a `grill-me` session; a plan against an unresolved gate is stale by
definition.

## What already exists — read before designing

- Mark = row presence in `pantry_item`; no row = no record, never "does not have" (MVP-014).
- `Ingredient.store_category` exists on every catalog identity (bakery, pantry, produce …) and already groups the shopping list; the pantry ignores it.
- Shopping list → pantry: purchased→pantry (explicit, reversible) and, after FIX-001, "Already have it" per line. The list is the pantry's main input; the pantry screen is for review and correction.
- Planner `PANTRY_FIT` (tier 5) scores recipes by marked ingredients, so a truthful pantry improves Cover My Week without any planner change.
- OPT-003 design: `pantry_staple` table, `set_pantry_staple`, restock line `r:<ref>` through the existing overlay.

## Decisions closed (do not reopen without a new decision)

- **No counts, targets, package sizes or consumption tracking.** Owner-confirmed 2026-09-08 after deep-options; PRD.md §5.2 ("quantified inventory is the category's abandonment trap"), invariant 6, invariant 19, PRD v3 §16. "Set how many of a staple to keep" is answered at the shelf, not in the app.
- **No free-text "keep 3" note on a staple.** OPT-003 rejection stands.
- **No inference of stock from checks, time or planned meals.** OPT-003 rejections stand.

## Decision gates (the grill-me agenda)

1. **What is a staple, and how does it reach the list?** Candidates, in the order the owner leaned on 2026-09-08 (undecided):
   - *M2 running low:* mark becomes have / running low / none. "Low" on a staple emits the restock line while stock remains. No numbers. Invariant 6 reworded from "binary" to "coarse". Risk: a third state on a screen whose one toggle already confuses.
   - *M4 standing list:* a staple appears on every trip's list regardless of mark; check or skip each time. Simplest derivation; right for eggs and bread, noisy for oil and rice. Implies "staple" means "things I check every trip".
   - *M1 as recorded in OPT-003:* unmark triggers restock. Fallback. Weakness: restock appears only after the last box is gone and only if someone remembers to unmark.
   The answer must be tested against "never a chore": how many taps per week does each model cost a household of four?
2. **What does the screen show by default?** Candidates: only marked and staple items grouped by `store_category` ("my shelves"), with search/browse to add; or a two-tab Have / Everything; or the current flat list with better labels. Lean: my-shelves default, because 160 unmarked rows are the complaint.
3. **What replaces the toggle?** A labeled state chip (Have · Low · Staple) vs a swipe vs a per-row menu. Must be understood without a tooltip.
4. **"Used it up".** PRD §5.2 names a one-tap remove. Where does it live and does it differ from unmark?
5. **Custom ingredients.** Do they get store categories at creation so they land on a shelf, or stay uncategorised?
6. **OPT-003 disposition.** Fold entirely into this card (recommended: its register row becomes *Superseded by OPT-006*) or keep as a separate card sequenced first.

## Load-bearing constraints

- Invariant 6 (coarse, optional, never an audit) and 19 (a mark suppresses a purchase, never proves quantity) hold in whatever model is chosen.
- The list stays a pure derivation; any new state is an explicit user statement with a bridge command that returns what was stored (invariant 21).
- Planner scoring changes are out of scope; a truthful pantry is the lever.
- Owner reviews visual candidates on both phones before the pick (owner preference recorded in memory: see UI variants in-app).

## Scope (to be refined after the gates close)

- Pantry screen redesign in Flutter; any state addition in `kimatta-storage` with one migration; bridge DTO/command additions per the chosen model; `derive_shopping_list` rule rows per OPT-003's table extended for the chosen model, with `SHOPPING_ALGORITHM_VERSION` bump if derivation changes.

## Non-goals

- Everything under *Decisions closed*. Expiry, recipes-from-pantry, barcode or receipt scanning, shared/cloud pantry.

## Acceptance criteria (draft; finalised with the gates)

- **AC-1:** A household can see what it has and what it keeps stocked without scrolling past unmarked catalog rows.
- **AC-2:** Every pantry state is labeled in words on the row; no unlabeled toggle remains.
- **AC-3:** The chosen staple model's list behaviour is covered by domain tests for each row of its state table, and the list stays order-independent and snapshot-deterministic.
- **AC-4:** No column, DTO field or command carries a count, target, package or timestamp.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1, AC-2 | Widget tests plus owner review on device of at least two visual candidates |
| AC-3 | `food-domain` tests extended from OPT-003's fixture set |
| AC-4 | Bounded inspection at review |

## Stop/failure conditions

- Stop if any step requires a quantity or an inference to satisfy an AC, if the chosen model cannot reuse the MVP-016 overlay without a special case, or on any invariant conflict. Two cycles then defer.

## Handoff

Standard `docs/ROADMAP.md` handoff; if OPT-003 is folded, record its supersession in the same edit.
