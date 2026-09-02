# Planner fixtures (MVP-025)

Version-controlled synthetic households for the planner's invariant tests and the
beam-width benchmark (PRD v3 §18, §9.6). All data is synthetic: no real household data,
names, or restrictions. Every fixture anchors at 2026-08-29 over a seven-day cycle
(dinner-only unless the shape says otherwise) and is built by `mod.rs` beside this file —
deterministically, so `snapshot_hash()` is pinned per fixture in
`../invariant_tests.rs::fixture_hashes_are_pinned`. Any fixture edit changes its pinned
hash: update the pinned literal deliberately, never delete the pin.

## Fixture table (AC-1)

| name | stressor | source | shape | reuse |
|---|---|---|---|---|
| cold_start | empty library, no preferences, restrictions unconfigured, starter present | PRD §18 | 0 recipes, 3 starter entries, 1 member, nothing configured | tests, bench, §22 |
| conservative_household | small repeat-tolerant library, narrow likes, repetitive history | PRD §18 | 6 recipes, 3 likes, 7 history dinners alternating 2 recipes | tests, bench, §22 |
| high_variety_household | large library and second slot stress the K cut and beam width | PRD §18 | 48 recipes, lunch+dinner scope, 2 members, 12 preferences | tests, bench, §22 |
| multiple_strong_dislikes | most recipes disliked by ≥1 of the members | PRD §18 | 8 recipes, 3 members whose dislikes hit 7 of 8 | tests, bench, §22 |
| busy_week | tight slot window vs preps over/at it, unknown preps, dining out enabled | PRD §18 | 30-minute dinner window; preps 55/45/30/15 and two `None`; `dining_out_enabled: true` — the corpus's only `CandidateSource::DiningOut` fixture | tests, bench, §22 |
| many_locked_meals | 5 of 7 dinners locked, one lock conflicting with a restriction | PRD §18 | 5 locked existing dinners; peanut restriction vs locked peanut recipe (`LOCK_HELD_OVER` path) | tests, bench, §22 |
| sparse_pantry | heavily shared ingredients with quantities to aggregate, empty pantry | PRD §18 | 3 recipes sharing rice/beans/tomato; Cup/Gram units, a range, an unknown quantity, an unresolved optional line; `pantry_marked` empty | tests, bench, §22 |
| restrictions_set_and_skipped | one Known + one Other restriction, with a skipped twin | PRD §18 | Peanuts + `other:cilantro`; `restrictions_skipped_variant()` returns the same household unconfigured — shape-asserted only, no invariant or benchmark plans the variant | tests, bench, §22 (skipped variant: shape assertion only) |
| leftovers_fallback_heavy | big batches, sourced history, no dining out | PRD §18 | two servings-8 recipes, history feeding the first cycle day, `dining_out_enabled: false` | tests, bench, §22 |
| repetitive_meal_library | near-duplicate recipes — repetition/tie-break stress | pivot prompt | five chilis with identical lines and prep, titles one word apart, plus one distinct dish | tests, bench, §22 |
| intentionally_open_nights | two explicitly `Open` slots amid an empty week | pivot prompt | 2 `Open` existing occurrences, 5 empty slots, 2 recipes | tests, bench, §22 |

"Reuse" — **tests**: `../invariant_tests.rs` quantifies the six MVP-025 invariants over
`all()`; **bench**: `examples/beam_width.rs` (feature `fixtures`) sweeps B×K over the same
snapshots; **§22**: fixtures carry full `Recipe`s (projected by `candidate_info_of`, which
mirrors the storage loader), so a later MILP/CP-SAT comparison consumes the identical
inputs (PRD §22 "beam search becomes a dead end" mitigation).

## Red demonstrations (AC-2)

Each invariant test has been shown to fail against a deliberately broken planner, then the
sabotage was reverted and the test re-ran green. Recorded per invariant:

| Inv | Test | Sabotage (file:line) | Observed failing assertion |
|---|---|---|---|
| 1 | `invariant_1_a_known_hard_restriction_is_never_selected_on_any_fixture` | `tier0.rs:112` — `.or(Some(restriction_conflict_code()))` routes every restriction conflict into assumptions instead of rejection | `many_locked_meals: proposed Recipe { recipe_id: RecipeId("r-lk-1") … } on 2026-09-04 dinner conflicts with a configured restriction` |
| 2 | `invariant_2_a_locked_meal_never_moves_on_any_fixture` | `tier0.rs:84` — `if false && slot.resolved …` deletes the resolved-slot `LOCK_CONFLICT` rejection | `many_locked_meals: locked meal moved at 2026-08-30 dinner — left: [Leftovers { note: None }], right: [Recipe { recipe_id: RecipeId("r-lk-3") }]` |
| 3 | `invariant_3_same_input_and_version_yield_identical_results_on_every_fixture` | `beam.rs:103` — tie-break routed through a process-global `AtomicUsize` incremented per search, reversing `plan_text` order on every second call (deterministically run 1 ≠ run 2) | `cold_start: two runs diverged` — run 1 chose `starter:fx-lentil-stew` on 2026-08-29, run 2 chose `starter:fx-veg-curry` (tied starters flipped) |
| 4 | `invariant_4_adding_a_hard_constraint_never_makes_an_infeasible_candidate_feasible` | `tier0.rs:111` — restriction check gated with `&& snapshot.policies.hard_vetoes.is_empty()` (constraint-interaction bug) | `many_locked_meals/veto:beans: 2026-09-03 dinner household_recipe recipe:r-lk-1:1/1 became feasible under a tighter constraint` |
| 5 | `invariant_5_shopping_quantities_are_well_formed_and_pantry_never_subtracts` | `shopping.rs:632` — before returning, every `OmittedPantryMarked` line's `Exact` quantity is multiplied by 1/2 (naive "reduce by what's at home") | `conservative_household: marking the pantry changed more than the status` — cheese `Exact(4/1)` vs `Exact(2/1)` |
| 1b | `invariant_1_…` (name resolution, added by the MVP-025 fix pass) | `candidates.rs:366` — the applying arm writes `format!("{STARTER_NOTE_PREFIX}{}-drift", s.slug)`: the prefix stays, the *slug* drifts, so the note is still recognised as a starter stub but no longer resolves | `cold_start: proposal names unknown starter slug "fx-lentil-stew-drift"` — and with `line_names_of`'s pre-fix `unwrap_or_default()` restored under the *same* sabotage the test goes **green**, which is the silent-pass path the fix closes |
| 5b | `invariant_5_…` (pantry half, added by the MVP-025 fix pass) | `shopping.rs:508` — `marked: Default::default()` makes the derivation ignore `pantry_marked` entirely, the no-op regression the quantity-halving sabotage cannot reach | `marking every identity must flip at least one line to OmittedPantryMarked` — pre-fix this sabotage passed, since `marked` was then bit-identical to `unmarked` and every equality held |
| 6 | `invariant_6_a_second_apply_never_mutates_the_first_ledger_rows` (kimatta-storage `controller.rs`) | `kimatta-storage/src/lib.rs:466` — `controller_ledger_no_update` trigger neutered with `WHEN 0` in the migration that creates it (the last `M::up` entry in `MIGRATION_ARRAY`, located by its `CREATE TRIGGER controller_ledger_no_update` string anchor; fresh in-memory DBs re-run migrations, so the edit is visible) | `called Result::unwrap_err() on an Ok value: 2` — the direct `UPDATE controller_ledger` succeeded |

Procedure per row: apply the sabotage, run just that test (observed output above, captured
verbatim in the MVP-025 orchestration handoff), `git checkout -- <file>`, re-run green.
Green re-runs for rows 2–6 were batched into one full-suite run after the last revert; row 1
was re-run individually. No sabotage was ever committed (`git status` clean afterward).
