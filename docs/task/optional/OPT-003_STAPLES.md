# OPT-003 — Staples: keep-stocked items that reach the list without a planned meal

> Optional planning input, not an MVP dependency or approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Optional post-MVP implementation |
| Workstream | Pantry and shopping |
| Depends on | MVP-014, MVP-015, MVP-016; sequenced after MVP-034 (D-041 release band) |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | Only when it cannot delay a required MVP or launch card |
| Recommended workflow | `plan-task` |
| External actions | None |

## Workflow gate

Before planning or execution, read `docs/ROADMAP.md` and apply the mandatory planning or execution gate in `docs/task/README.md` for `OPT-003`. Because this card is optional, the owner must explicitly select it without displacing required MVP or launch work.

## Outcome and user value

A household marks pasta, rice, olive oil as **staples** — things it wants on the shelf whether or not this week's plan uses them. When a staple is not marked as had, it appears on the shopping list as a restock line even if no planned meal needs it. When it is marked, it is silent. The household decides how much to buy; Kimatta never counts.

This is the "we keep three boxes of pasta" need, met without a quantified inventory. The owner explored counted stock, hidden estimated stock and inference-from-checks on 2026-09-05 and rejected all three (see *Rejected designs*); the quantity-free model below is what fits the existing pantry, list and evidence posture with no invariant or PRD change.

## Authoritative sources

- `docs/task/MVP_INVARIANTS.md` 6 (pantry is optional binary have/don't-have, never a quantified inventory audit), 19 (a mark suppresses a purchase but never proves quantity), 17 (same snapshot derives the same list; bump `SHOPPING_ALGORITHM_VERSION`), 21 (coarse bridge)
- `docs/PRD_v3.md` §8 (pantry is simple and optional, not a quantified warehouse inventory), §10 (incomplete pantry → list can be ready but stock completeness is unknown), §16 (quantified pantry inventory is Not MVP); `docs/PRD.md` §5.2 (quantified inventory is the category's abandonment trap)
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §6 (represent uncertainty; pantry presence can be stale; explicit user statement is the evidence class used here)
- `docs/ROADMAP.md` MVP-014 handoff row 2026-08-29 (row-presence model: a `pantry_item` row means marked; no row means no record, never "does not have"), MVP-015 and MVP-016 rows (pure derivation; per-window overlay; purchased→pantry explicit, idempotent, reversible)
- `rust/crates/food-domain/src/shopping.rs` (`derive_shopping_list`, `ShoppingLine`, `LineStatus`, `Quantity::Unknown`, `contribution_count` invariant), `rust/src/api/pantry.rs`, `lib/features/pantry/pantry_screen.dart` (`SwitchListTile` row)

## Design (recorded 2026-09-05, owner-resolved)

**Model.** A staple is a flag on an ingredient identity (`IngredientRef`, catalog or custom), household-scoped. It carries no quantity, target, package size, note or timestamp. The only pantry state remains the existing mark.

| Staple | Marked | List effect |
|---|---|---|
| no | — | unchanged from MVP-015 |
| yes | yes | silent (existing `OmittedPantryMarked` if a meal needs it) |
| yes | no, meal needs it | the existing `Needed` line, flagged `staple` |
| yes | no, no meal needs it | one **restock line**: `Needed`, `Quantity::Unknown`, no contributions |

**Storage.** A new table `pantry_staple(household_id, ingredient_id, custom_ingredient_id)` with the same shape, CHECK, unique partial indexes and FK indexes as `pantry_item`. It cannot be a column on `pantry_item`: that table's row presence *is* the mark, and an unmarked staple needs a row. Row presence = staple; no row = not a staple. One migration, next schema version.

**Derivation.** `ShoppingInput` gains `staples: Vec<IngredientRef>`; `ShoppingLine` gains `staple: bool`. After grouping, every staple identity that is not marked and has no line is emitted as a restock line in its store-category group (uncategorised if the identity has none) with key `r:<escaped ref>` under the existing key-escaping rule, so the MVP-016 overlay (check, hide, restore, `checked_against`) works on it unchanged. `contribution_count` stays the sum of contributions (a restock line contributes zero). Still pure and clock-free; `SHOPPING_ALGORITHM_VERSION` becomes 2.

**Bridge.** `PantryEntryDto` gains `staple: bool`; one new coarse command `set_pantry_staple(household_id, ingredient, staple) -> PantryEntryDto` mirroring `set_pantry_mark`'s return-what-was-stored contract (invariant 21). `ShoppingLineDto` gains `staple`.

**Flutter.** Pantry row: the existing switch stays the mark; a trailing star toggles staple. Shopping list: a `staple` badge on flagged lines; a restock line renders its name, the badge and "restock" in place of a quantity, using the existing unknown-quantity copy path in `shopping_copy.dart`.

**Loop closure — explicit only.** Checking a restock line hides it for this window through the existing overlay. Marking it as had is the existing purchased→pantry action at trip end (MVP-016 AC-3). Running low or out is the existing unmark. Kimatta never infers a purchase from a check, a consumption from a planned meal, or a trip from a window rollover — the same posture as planned meals, which carry no "cooked" field (`planned_meal.rs` header; invariant 19). A forgotten trip-end tap makes the staple reappear next window, which is recovered by the same tap; a forgotten unmark keeps it silent, exactly as any stale mark does today. No new failure class is introduced.

## Rejected designs (do not reopen without a new decision)

- **Counted stock (on-hand + target, with or without reservations from planned meals).** Needs package sizes the catalog lacks, recipe-unit→package-unit conversion ("½ box" vs "225 g"), consumption on *planned* rather than cooked meals, and amends invariants 6 and 19 plus PRD §16. The abandonment-trap rationale stands.
- **Hidden estimated stock ("the app knows the numbers").** Estimates from checks and planned meals decay with every unrecorded use and cannot be corrected because they are hidden; a wrong estimate becomes a silent omission, which invariant 19 forbids. `shopping.rs`: "Nothing here guesses."
- **Check on a staple line auto-marks pantry.** Splits the checkbox's meaning by line kind (MVP-016: a check "records that the user ticked this list, not an inventory claim"), makes uncheck an inventory write, and lands next to an open restore/mark issue (`KNOWN_ISSUES.md`). Bounded upgrade if beta shows the trip-end tap is skipped: one key prefix, one bridge command.
- **Implicit purchased→pantry at window rollover.** Marks checked-but-not-bought lines as had; changes AC-3 from explicit to implicit for every line.
- **Usage-triggered "still stocked?" question** (staple used by a planned meal and unbought or unre-marked for N windows, surfaced in Attention). Repo-consistent (ask, don't assume) but speculative; needs a `marked_at` column and a magic N. Deferred until beta shows forgotten-unmark is a real problem. Recorded here so a later card starts from this shape.
- **Free-text note on a staple ("keep 3 boxes").** Dropped: the line names the item and the household decides the amount.

## Load-bearing constraints

- No quantity, target, package, count or timestamp is stored for a staple or a mark. Invariant 6 holds verbatim.
- A staple never asserts absence: an unmarked staple is "no record", and the restock line is a purchase suggestion, not an inventory claim (MVP-014 row-presence rule; PRD §10).
- The list stays a pure, clock-free derivation; nothing infers a purchase, a consumption or a trip from any action.
- The restock line is an ordinary `ShoppingLine` so every MVP-016 overlay behaviour and Undo path applies without a special case.
- Planner scoring (`score.rs` pantry fit) is unchanged.

## Scope

- Migration adding `pantry_staple`; storage read/write mirroring `pantry_item`'s; `load_shopping_input` reads staples.
- `derive_shopping_list`: `staples` input, `staple` line field, restock lines, `SHOPPING_ALGORITHM_VERSION` 2; fixture and permutation tests extended.
- Bridge: `PantryEntryDto.staple`, `set_pantry_staple`, `ShoppingLineDto.staple`; regenerated bindings.
- Flutter: star toggle on the pantry row; badge and restock rendering on the list; copy tests.

## Non-goals

- Everything under *Rejected designs*.
- Staples as a manual-item variant, recurring reminders, expiry, "used it up" on the list itself, or any change to purchased→pantry.

## Decision gates

- None. The owner resolved quantities (none), loop closure (existing explicit action) and evidence posture (explicit only) on 2026-09-05.

## Acceptance criteria

- **AC-1:** A staple that is not marked and not needed by any planned meal in the window appears once as a restock line in its store-category group; the same staple marked appears nowhere; a staple needed by a meal appears only as the existing line, flagged.
- **AC-2:** Restock lines check, hide, restore and reset through the existing overlay; purchased→pantry marks a checked restock line's identity and the line is absent on the next derivation; Undo unmarks it.
- **AC-3:** `derive_shopping_list` remains order-independent with staples present, `contribution_count` still equals the sum of contributions, and the same snapshot derives the same list.
- **AC-4:** No column, DTO field or bridge command carries a quantity, count or timestamp for staples or marks.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Domain tests on `derive_shopping_list` for the four rows of the model table |
| AC-2 | Bridge tests over `set_shopping_line_state`, purchased→pantry and its Undo with a restock line key |
| AC-3 | Existing permutation and fixture tests extended with staples |
| AC-4 | Bounded inspection of the migration, DTOs and generated bindings at review |

## Stop/failure conditions

- Stop if this delays MVP or launch work, if any step would require a quantity or an inference to satisfy an AC, or if the restock line cannot reuse the overlay without a special case. Two cycles then defer.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, evidence, blockers, and **Next implementation task**.
