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
`docs/task/README.md` for `OPT-006`. **Resolved 2026-09-15** — all six gates below record an owner
answer from a `grill-me` session; `plan-task` may proceed. **`plan-task` has run** (2026-09-16):
`feature/opt-006` already carries an uncommitted, apparently complete implementation (storage
migration + bridge + domain derivation across `kimatta-storage`/`food-domain`/`rust/src/api`, and
`lib/features/pantry/` rewritten with the My Shelves screen, chip row controls, and the
categorization-remediation and custom-ingredient-creation flows) with `cargo test --workspace`
and `flutter test` green. **Outstanding before sign-off:** the Evidence plan's "owner review on
device of at least two visual candidates" (below) has not happened — only one candidate exists,
already built — so AC-1/AC-2 are implemented but not yet evidenced per this card's own
requirement.

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

## Decision gates — resolved 2026-09-15 (grill-me)

1. **Staple model: M2-lite.** Mark stays binary (have/none, per OPT-003/M1) — this is the sole
   truth state. A separate, independent one-tap "flag for restock" action emits the existing
   `r:<ref>` restock line immediately, decoupled from the have/none mark; one new boolean
   (`restock_requested`), no tri-state column. This *is* a gating-condition change from OPT-003's
   original derivation rule, not merely an added source: the restock line now fires on
   `restock_requested` alone, independent of any staple concept, whereas OPT-003's rule was
   staple-scoped. Rejected: full M2 tri-state (most code, UI risk on an
   already-flagged-confusing screen); M4 standing list (per-trip tap tax scales with
   trip-frequency × staple-count, and "always on the list" isn't a truthful stock statement —
   conflicts invariants 6 and 19).
2. **Default screen: My Shelves, single screen, inline add.** Default view shows only rows with
   `marked = true` (post-gate-1, "staple" is not a stored concept — see gate 1), grouped by
   `store_category`; a persistent search field expands the same screen to the full catalog in
   place (no FAB — no precedent for one in this app, and a plain search box does the job; no
   second route, no tab bar) and collapses back on select. Rejected: My
   Shelves with a separate search/browse route (extra nav cost, no truth-value gain); two-tab
   Have/Everything (persistent chrome + a which-tab decision on every open, against
   "never a chore").
3. **Row control: shelf-tag chip buttons.** Two always-visible, worded buttons per row ("Low",
   "Used up") styled as small shelf-sticker/tag chips rather than generic icon-buttons — same
   interaction and tap-count as plain labeled buttons, all added value is visual/tactile styling
   reinforcing the pantry metaphor. Satisfies AC-2 literally (words stay on the row). Rejected:
   3-dot overflow menu (generic, adds a tap to the common action); swipe actions (fails the
   no-tooltip requirement); row-tap-to-remove hybrid (accidental-tap risk, no literal word on the
   row for the action — revisit only if row density becomes a real problem later).
4. **"Used it up": unmark + auto-restock.** Tapping "Used up" clears the have-mark AND fires the
   same `r:<ref>` restock line as the "Low" chip — restores OPT-003/M1's safety net (zero-stock
   always reaches the list) while "Low" remains an optional earlier heads-up; no household
   discipline required to avoid silent drift. **The safety net must hold on every path to "not
   marked," not only the "Used up" chip** — the plain have-mark toggle also present on the row
   reaches the same "don't have it" state, so unmarking through *either* control routes through
   the same combined unmark+restock-flag transaction; "Used up" is not the sole gate to the
   safety net. The restock command must be idempotent so a prior "Low" tap followed by an unmark
   doesn't duplicate the line. Recipe-linked/planned-meal consumption was raised and explicitly
   **not** adopted — it collides with the closed decision banning inference from planned meals
   (line above); dropped rather than reopened. Revisit only as a separate future card if manual
   tapping proves to be a real weekly burden after real usage.
5. **Custom ingredients: store_category required at creation.** A dropdown reusing the catalog's
   existing `store_category` enum is a required field on the custom-ingredient creation form.
   Every custom item lands on a real shelf; one grouping code path, no "Other"/uncategorised
   fallback bucket anywhere on My Shelves. Pre-existing custom ingredients whose `store_category`
   is still `NULL` (created before this requirement) are handled by a one-time categorization
   remediation flow, scoped to have-marked rows only (an unmarked custom ingredient missing a
   category blocks nothing visible yet, so it waits until marked, at which point it enters the
   same list the categorize banner counts) — never a fallback bucket, never force-migrated to a
   default. The banner's count and the flow it opens must draw from the same scoped list, never
   two independently-computed sets.
6. **OPT-003 disposition: folded, superseded.** OPT-003's register row in `docs/ROADMAP.md`
   becomes *Superseded by OPT-006* in the same edit that lands this card's implementation — no
   OPT-003 scope remains outside what is decided above.

## Load-bearing constraints

- Invariant 6 (coarse, optional, never an audit) and 19 (a mark suppresses a purchase, never proves quantity) hold in whatever model is chosen.
- The list stays a pure derivation; any new state is an explicit user statement with a bridge command that returns what was stored (invariant 21).
- Planner scoring changes are out of scope; a truthful pantry is the lever.
- Owner reviews the built candidate on-device before sign-off, per the amended Evidence plan below (owner preference recorded in memory: see UI variants in-app).

## Scope (to be refined after the gates close)

- Pantry screen redesign in Flutter; any state addition in `kimatta-storage` with one migration; bridge DTO/command additions per the chosen model; `derive_shopping_list` rule rows per OPT-003's table extended for the chosen model, with `SHOPPING_ALGORITHM_VERSION` bump if derivation changes.

## Non-goals

- Everything under *Decisions closed*. Expiry, recipes-from-pantry, barcode or receipt scanning, shared/cloud pantry.

## Acceptance criteria (draft; finalised with the gates)

- **AC-1:** A household can see what it has and what it keeps stocked without scrolling past unmarked catalog rows.
- **AC-2:** Every pantry state is labeled in words on the row; no unlabeled toggle remains.
- **AC-3:** The `marked`/`restock_requested` restock-line derivation is covered by domain tests for each row of its state table, and the list stays order-independent and snapshot-deterministic.
- **AC-4:** No column, DTO field or command carries a count, target, package or timestamp.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1, AC-2 | Widget tests plus owner review on device of the one built candidate — **pending**: owner accepts it as-is or requests changes. A second full visual candidate is built only if that review rejects a core interaction choice (the chip row controls, the grouped-by-category layout, or the search-expand mechanism) — not for styling/polish requests, which are handled as changes to the existing candidate. (Amended 2026-09-16 from "at least two visual candidates" — the original two-candidate requirement was written during scoping, before an implementation existed; building a second full screen upfront would have been speculative duplication of a mostly-cosmetic difference against an already-built, fully-tested candidate.) |
| AC-3 | `food-domain` tests extended from OPT-003's fixture set |
| AC-4 | Bounded inspection at review |

## Stop/failure conditions

- Stop if any step requires a quantity or an inference to satisfy an AC, if the chosen model cannot reuse the MVP-016 overlay without a special case, or on any invariant conflict. Two cycles then defer.

## Handoff

Standard `docs/ROADMAP.md` handoff; if OPT-003 is folded, record its supersession in the same edit.
