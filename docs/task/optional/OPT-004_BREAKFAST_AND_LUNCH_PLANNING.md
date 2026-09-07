# OPT-004 — Breakfast and lunch planning

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Planner/experience + content |
| Depends on | MVP-013, MVP-023, MVP-024, MVP-032 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | None |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `OPT-004`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

A household that eats planned breakfasts or lunches can say so, and Cover My Week covers those slots with dishes that suit them. Today the planner covers dinner only — not because it cannot do more, but because nothing lets the household ask and no shipped recipe knows which meal it belongs to.

## What already exists — read this before estimating

Most of this is built. The card is smaller than its title suggests, and a plan that assumes otherwise will over-build.

- **The domain models all three slots.** `MealSlot` is `{ Breakfast, Lunch, Dinner }` (`rust/crates/food-domain/src/lib.rs:66-70`), `MealScope::new` accepts any non-empty combination (`:106`), and `MealScope::dinner_only()` is documented as a *default*, not a limit: "dinner defaults on, breakfast and lunch are opt-in" (`:101`).
- **The planner already plans any scope.** `candidates::generate` iterates `snapshot.scope.slots()`, and `FoodPolicies.slot_windows` is already keyed by `MealSlot`, so per-slot time budgets work. Tests exercise multi-slot scopes today — `rust/src/api/shopping.rs:440` and `rust/src/api/planned_meals.rs:260` both build `MealScope::new([Lunch, Dinner])`.
- **The bridge already accepts a scope.** `save_in` in `rust/src/api/planning.rs:99-112` takes `meal_slots: &[MealSlotDto]` and builds the scope from it.

So the planner, the storage layer and the bridge need **no new capability**. What is missing is a way to ask, and content that knows what it is.

## Load-bearing constraints

- **Dinner stays the default.** A household that never opens this setting must see exactly today's behaviour. `MealScope::dinner_only()` remains what a new cycle gets.
- **A dish must know which slots suit it.** Starter content has no course or slot field, so shipping breakfast content today would also make Banana Bread a candidate for Tuesday dinner, and chili a candidate for breakfast. The slot-suitability data is this card's real work.
- **Suitability is advisory, not a filter the user cannot override.** The household can always place any recipe in any slot by hand; MVP-012's manual override is not narrowed. Suitability governs what the planner *proposes*.
- **The shopping list and pantry are slot-agnostic and stay that way.** More planned meals means more lines; nothing about aggregation changes.
- **`prep_minutes` means what MVP-032 decided it means.** Breakfast slots will have short windows, so the field's existing ambiguity (prep-only vs prep+cook, `KNOWN_ISSUES.md`) bites harder here. Resolve it in this card or state why not.

## Scope

**Phase 1 — slot suitability on content**
- Add a slot-suitability field to the starter schema and to `Recipe`, defaulting to dinner so every existing entry keeps its behaviour with no migration of meaning. Persist it; regenerate the bridge.
- Populate it for the shipped roster.
- Have `candidates::generate` prefer suitable dishes for a slot. Decide, and record, whether an unsuitable dish is excluded outright or merely scored down — scoring down keeps the "Nothing fits" escape hatch that D-041 worries about.

**Phase 2 — letting the household ask**
- A scope control in the planning-cycle UI that sends `meal_slots` to the existing `save_in`. Dinner preselected.
- Cover My Week renders the extra slots per day.

**Phase 3 — the held-back starter recipes**
- Import the owner recipes held back by MVP-032 for exactly this reason, from `refs/recipes-pending.json`: three sides (honey glazed carrots, roasted green beans, roasted vegetables) and two breakfast/baking dishes (banana bread, apple cinnamon roll casserole). They are transcribed and ready; they need only a suitability value.
- Sides raise a question this card must answer or defer explicitly: a side is not a meal. Either suitability grows a "side" value that the planner may attach *alongside* a main, or sides stay out and the three carrots/beans/vegetables entries wait again. Do not ship a side as a standalone dinner.

## Non-goals

- Changing the shopping list, pantry, restrictions or rights model.
- Multi-course meal composition beyond whatever the side decision above settles.
- Snacks, or any slot outside `MealSlot`'s three.
- Nutrition or calorie targets per slot.

## Decision gates

- If the side-dish question cannot be settled cheaply, ship breakfast and lunch without sides and leave the three side entries pending. Say so in the handoff rather than forcing a composition model into this card.
- If slot suitability turns out to want more than one value per recipe (a soup that is lunch or dinner), that is expected — model it as a set, not a single value, before writing content.

## Acceptance criteria

- **AC-1:** A household with the default scope sees behaviour identical to today, pinned by a test asserting `dinner_only` remains the default for a new cycle.
- **AC-2:** A household can select breakfast and/or lunch in the UI, and the selection round-trips through `save_in` and back into `snapshot.scope`.
- **AC-3:** Cover My Week proposes a slot-suitable dish for each enabled slot, and does not propose a dinner-only dish into breakfast.
- **AC-4:** Every shipped starter entry carries a suitability value; existing entries default to dinner and their proposals are unchanged, pinned by a test.
- **AC-5:** The five held-back owner recipes are imported and suitable for their real slots, or the handoff records why they still wait.
- **AC-6:** `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `flutter analyze` and `flutter test` pass; the bridge is regenerated if any DTO changed.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Test output showing the default scope unchanged |
| AC-2 | Emulator run selecting a scope and re-reading it after relaunch |
| AC-3 | A Cover result for a multi-slot scope, with the per-slot sources shown |
| AC-4 | Content test over the shipped roster |
| AC-5 | The content diff, or the handoff paragraph explaining the deferral |
| AC-6 | Command output in the handoff |

## Stop/failure conditions

- Stop on an invariant conflict, a stale dependency, or a need to change the shopping, restriction or rights models to make this work — any of those means the card is mis-scoped.
- Stop if slot suitability cannot be added without a schema migration that rewrites existing rows' meaning.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, the side-dish decision, whether the five held-back recipes shipped, blockers, and **Next implementation task**.
