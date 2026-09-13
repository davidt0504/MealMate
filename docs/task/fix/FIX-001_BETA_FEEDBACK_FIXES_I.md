# FIX-001 — Beta feedback fixes I: shopping-list naming, removal, ingredient entry, Accept feedback

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Fix batch (D-042) |
| Workstream | Shopping, recipes, planner surface |
| Depends on | MVP-016, MVP-024, MVP-032, MVP-033 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

## Outcome and user value

Four confirmed defects from the 2026-09-08 owner beta notes (`docs/BACKLOG.md`) are fixed in one
session: the shopping list names the product the recipe actually asks for; removing a line is
discoverable and says what it means; adding an ingredient is typed once; Accept acknowledges
itself. Every item is root-caused; none has an open design question.

## Workflow gate

Before planning or execution, read `docs/ROADMAP.md` and apply the mandatory planning or
execution gate in `docs/task/README.md` for `FIX-001`. A fix batch may hold several bounded
items; any item that turns out to need a decision is ejected to `docs/BACKLOG.md` rather than
resolved inside the plan.

## Authoritative sources

- `docs/task/MVP_INVARIANTS.md` 2 (original text retained), 6 and 19 (pantry mark semantics), 17 (same snapshot derives the same list)
- `docs/bugs/MVP-033_ACCEPT_FEEDBACK.md` (item 4 dossier)
- `docs/task/optional/OPT-003_STAPLES.md` *Rejected designs* — a check never implies a pantry mark; an explicit "Already have it" action is an explicit user statement and is not that rejection
- `KNOWN_ISSUES.md` orch/31 MEDIUM (restore then re-mark does not re-hide) — fixed here because item 2 lands on the same code

## Items

### 1. "Crushed tomatoes" on the list for a diced-tomato recipe

**Cause (verified 2026-09-08).** `rust/crates/food-domain/content/starter_recipes.json` catalog
entry `ing-canned-tomatoes` has canonical name "crushed tomatoes"; the lines "diced tomatoes" in
David's Chili, Taco Soup, Taco Pasta, Stuffed Pepper Casserole and Black Bean and Lentil Soup
point at it. `derive_shopping_list` (`rust/crates/food-domain/src/shopping.rs:595-599`) names a
merged line by the catalog canonical name. Six catalog ids have line names outside canonical or
aliases; only this one changes the product bought (the others: lemon juice → lemon, lime juice →
lime, orzo pasta → orzo, elbow macaroni pasta → small pasta, chicken → skinless chicken pieces).

**Fix.** Add catalog `ing-diced-tomatoes` ("diced tomatoes", store category pantry); remap the
five lines. One schema-version migration that (a) inserts the new catalog row itself, so it does
not depend on `install_starter_content` ordering, and (b) runs
`UPDATE recipe_ingredient_line SET ingredient_id = 'ing-diced-tomatoes' WHERE ingredient_id = 'ing-canned-tomatoes' AND lower(name) LIKE '%diced%'`
(`recipe_ingredient_line`, `kimatta-storage/src/lib.rs:265`), because
`install_starter_content` skips recipes whose slug already exists, so a JSON-only fix reaches
fresh installs only. Add the five benign names as aliases. Add one Rust content test: every
starter line name equals its catalog canonical name or one of its aliases.

### 2. Removing a line from the shopping list

**Cause.** "Remove from list" exists only inside the "Why is this here?" bottom sheet
(`lib/features/shopping/shopping_screen.dart:544`). Removal is a per-window `hidden` overlay;
it never touches recipes, planned meals, pantry or planner scoring, and it does not persist to
the next cycle. The commonest reason to remove a line — "I already have this" — is therefore
mis-served: the line returns every week and the pantry never learns.

**Fix.** Two explicit intents, no new storage or bridge command:
- **Skip this time** — existing `hidden` write. Exposed by swipe (`Dismissible`) with an Undo
  snackbar whose copy says recipes are unchanged, and kept in the sheet.
- **Already have it** — existing `set_pantry_mark`. Line moves to Already have; persists across
  cycles; Undo unmarks exactly the returned set (same contract as purchased→pantry).
- Fix the open orch/31 edge (restore → unmark → re-mark leaves the line restored) with the clamp
  the finding describes, since the new action exercises that path.

### 3. "As written" and "Name" both required on a new ingredient row

**Cause.** `original_text` is required non-blank in `food-domain` and is what recipe detail and
the shopping explain sheet render; `name` is what matching, restriction scanning, merging and
pantry identity key on. Both are needed in the domain; only the form makes the user type both.

**Fix (Flutter only).** New rows show no "As written" field; at save the form composes
`original_text` from amount, unit, name and preparation. Loaded rows whose stored text differs
from the composed form (starter, imported) show it as a read-only "As written" caption. The
verbatim-send rule of MVP-008 still applies to what the user typed. Copy test on the composer.

### 4. Accept stays actionable after success

Per the dossier: `CoverScreen` keys the button on `result.status`, not `_accepted`. After a
successful accept, replace the primary control with the calm confirmation and the shopping-list
action already rendered; regression coverage for failure and repeated taps. No celebration, no
modal, no change to MVP-024 one-tap acceptance or MVP-033 no-trap completion.

## Load-bearing constraints

- The list stays a pure, clock-free derivation; item 1 changes content and one identity, not the naming rule, so `SHOPPING_ALGORITHM_VERSION` is unchanged unless the plan finds otherwise.
- A pantry mark is written only by an explicit user action (item 2); a swipe never marks.
- `original_text` is never derived in Rust; the form composes it as the user's entry (item 3).
- Generated bindings are not hand-edited; item 1 adds no bridge command.

## Non-goals

- Pantry screen changes, staples, counts (OPT-006). Planner action vocabulary, reshuffle (OPT-007).
- Renaming or merging the other five mismatched catalog ids beyond adding aliases.
- Any parse of free text into structured fields (OPT-002).

## Decision gates

- None. All four items were resolved in the 2026-09-08 owner interview.

## Acceptance criteria

- **AC-1:** A week containing David's Chili lists "diced tomatoes", not "crushed tomatoes", on a device that installed content before this change; a fresh install agrees. The content test fails on any starter line whose name is outside its catalog canonical/aliases.
- **AC-2:** Swiping a needed line removes it to the Removed section with Undo; "Already have it" marks the ingredient, moves the line to Already have, survives a new cycle, and Undo unmarks it; the orch/31 sequence re-hides the line.
- **AC-3:** A new ingredient row saves with a non-blank composed `original_text` without the user typing it; a starter row shows its source line read-only and unchanged after save.
- **AC-4:** After a successful Accept the primary Accept control is gone and the shopping-list action is present; a failed accept leaves Accept in place; a second tap is not possible.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | `kimatta-storage` migration test on a pre-change fixture DB; `food-domain` content test; owner-device check on an existing install |
| AC-2 | Widget tests for swipe/Undo and Already-have/Undo; bridge test for the orch/31 sequence |
| AC-3 | Widget test asserting the composed `original_text` field by field; copy test |
| AC-4 | Widget tests for success, failure and repeated tap |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- Stop and eject the item to `docs/BACKLOG.md` if any item needs a decision not recorded here.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence per item, decisions, blockers, and **Next implementation task**. Close the dossier `docs/bugs/MVP-033_ACCEPT_FEEDBACK.md` and the orch/31 KNOWN_ISSUES entry in the same handoff. Never promote a status without its required evidence and explicit owner approval.
