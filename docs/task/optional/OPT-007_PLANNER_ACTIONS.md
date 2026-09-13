# OPT-007 — Cover My Week actions: say what they do, ask for another, reshuffle the week

> Scoping card (D-042). The design is not resolved here; a dedicated `grill-me` session resolves the decision gates below and folds its answers into this card before `plan-task`.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Optional post-MVP implementation |
| Workstream | Planner experience |
| Depends on | MVP-023, MVP-024, MVP-025 |
| Complexity | Complex |
| Assurance | Elevated (planner determinism and veto irreversibility) |
| Sequential batching | No |
| Recommended workflow | `grill-me` (fold answers into this card) → `plan-task` (no `--auto`) |
| External actions | None |

## Outcome and user value

On a proposed slot the household can say "not this, show me something else" and get another
proposal from the app; can reshuffle the whole week when nothing appeals; and can tell what each
button will do before tapping it. Owner note 2026-09-08: "swap needs to say something else more
intuitive … a button alongside that and Never suggest for another suggestion from the app.
Probably also a button for a completely reshuffle week."

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in
`docs/task/README.md` for `OPT-007`. **Planning may not start until every decision gate below
records an owner answer** from a `grill-me` session.

## What already exists — read before designing

- **Swap** (`lib/features/planning/cover_screen.dart:222-228, 411-424`) opens a picker over the recipe library plus fallback kinds (leftovers, dining out, frozen/quick, open) and records `PlanDecisionDto.swap`, which locks the slot. It is a manual choice, not a re-proposal.
- **Never suggest** records `PlanDecisionDto.veto` → `food.hard_veto` by whole-token phrase against recipe titles *and* ingredient line names; irreversible, no undo, no list of vetoes in the UI. Invariant: locks and hard vetoes cannot be bought through by softer preferences.
- **No re-propose API.** `coverCycle(apply:)` runs the whole deterministic bounded beam search over one immutable snapshot. There is no entry point for "propose again for this slot excluding X" or "propose a different week".
- The planner is deterministic by design (MVP-023/025 invariant tests): the same snapshot yields the same plan. "Another suggestion" therefore needs a snapshot *input* that changes — an exclusion set or a declared variation key — not randomness.
- Every preview, decline, apply and correction is recorded in the controller ledger (MVP-023); new actions must be too.

## Decision gates (the grill-me agenda)

1. **Vocabulary.** What the three actions are called on the card. Candidates: "Choose a different meal" (manual picker) · "Show me another" (app re-proposes) · "Never suggest this". Reshuffle: "Try a different week" vs "Reshuffle". Test on the phone with the beta tester.
2. **Re-propose semantics.** For "Show me another": run the search with the current recipe for that slot added to a per-run exclusion set (deterministic, no seed), holding every other slot fixed? Or re-plan the unlocked remainder of the week around the change? Lean: single-slot with the rest held, because the household is reacting to one meal.
3. **Reshuffle semantics.** Exclude every currently proposed recipe and re-run over the unlocked slots; locked and swapped slots never move. Does reshuffle count as a decline in the ledger? How many reshuffles before the planner runs out of candidates, and what does it say then (attention request, not a spinner)?
4. **Never suggest placement and reversibility.** Putting an irreversible veto next to a frequent action invites mistakes. Options: confirmation kept but veto list with remove added under Preferences; or move veto into the "another" flow as a secondary choice; or make veto reversible by design. Lean: reversible via a visible veto list before it is placed alongside.
5. **Where fallback kinds live.** Leftovers / dining out / frozen / open are picker entries today. Do they stay in the manual picker only, or can "Show me another" propose them? (Interacts with OPT-008.)
6. **Ledger and evidence.** Which reason codes the new actions record; whether a reshuffle is one event or N.

## Load-bearing constraints

- Determinism: identical snapshot plus identical exclusion set yields the identical proposal (extend MVP-025's invariant tests).
- Locks and vetoes are never overridden by a re-propose or reshuffle.
- Coarse bridge (invariant 21): one command per action, returning the new proposal, no handle leakage.
- Attention requests, not silent fallback, when candidates are exhausted.
- No change to accept semantics (MVP-024 one tap; MVP-033 no trap).

## Scope (to be refined after the gates close)

- `food-domain` planner: exclusion input on the search snapshot; `kimatta-application`: re-propose for slot / reshuffle unlocked slots, ledger entries; bridge commands; Cover screen actions, copy and tests; veto list surface if gate 4 says so.

## Non-goals

- Leftover-aware planning policy and any leftovers toggle (OPT-008). Breakfast/lunch scope (OPT-004). Randomness of any kind.

## Acceptance criteria (draft)

- **AC-1:** "Show me another" on a slot yields a different recipe for that slot with all other slots unchanged, and the same request on the same snapshot yields the same answer.
- **AC-2:** Reshuffle changes every unlocked, unswapped slot's recipe where candidates allow and never moves a locked or swapped slot; exhaustion produces an attention request.
- **AC-3:** Every action's label states its effect in words the beta tester reads correctly without explanation (owner-device check).
- **AC-4:** A veto placed from the card is visible somewhere the household can remove it, if gate 4 resolves to reversible.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1, AC-2 | `food-domain`/`kimatta-application` tests extending MVP-025 fixtures; determinism permutation test |
| AC-3 | Owner and beta-tester device check, recorded in the handoff |
| AC-4 | Widget and bridge tests for veto list and removal |

## Stop/failure conditions

- Stop on any path that would let a soft action override a lock or veto, on any nondeterminism, or on an invariant conflict. Two cycles then defer.

## Handoff

Standard `docs/ROADMAP.md` handoff.
