# OPT-007 — Cover My Week actions: say what they do, ask for another, reshuffle the week

> Scoping card (D-042). Decision gates resolved 2026-09-28; owner answers are folded in below the gate list.

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
records an owner answer** from a `grill-me` session. **Resolved 2026-09-28** — all six gates
record the owner's answers from the 2026-09-28 planning session (external Codex session, reviewed
by `redteam-plan`); the approved plan is
`~/.codex/personal-workflows/pilot/plans/meal-mate-opt-007-meal-alternatives.md` (baseline
`c56bbb4`), and its §1 is the authoritative list of settled behavior.

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

### Owner answers (2026-09-28)

1. **Vocabulary:** per-slot **Another**; manual picker **Choose**; **Lock in** / **Locked in** confirms a meal for its date (not a favorite); **Never suggest this meal**. The week action's label is configurable: default **New mix**, alternate **Another plan**, compared in a local two-variant tester experiment after comprehension testing. AC-3's device check still applies.
2. **Another:** single slot, every other occurrence held. The whole occurrence (every component) is excluded for that slot, deterministically; no seed.
3. **Week action:** changes unlocked future/today suggestions; locked, explicitly open and manually chosen fallback slots never move. Displaced recipes stay excluded until Accept, Discard or explicit **Reconsider**. It is not recorded as a decline, and no dislike is inferred. Exhaustion keeps the previous proposal and says `No more alternatives for <day>` with Choose / Reconsider; no retry quota, no silent recycling.
4. **Never suggest:** reversible. It creates a confirmed recipe-identity rule and stays in the card's secondary menu. Settings > Meal exclusions lists identity rules and legacy phrase rules with Remove. Legacy phrase rules keep their exact title+ingredient meaning.
5. **Fallback kinds:** only in the manual picker. Another never proposes a fallback; a fallback-only slot offers Choose.
6. **Ledger:** one event per user action (a week request is one event with per-slot details). There are distinct codes for initial proposal, another, week alternatives, lock/unlock, choose, undo, reconsider, discard, review, accept and permanent-policy changes.

**Scope added by the plan:** Cover edits become a persisted draft that **Accept** writes exactly as reviewed, instead of re-planning. Accept stays one tap and does not require locks, which refines the "no change to accept semantics" constraint below. The plan also adds a Tier-5 similarity-to-commitment term (v1 rule table) and local-only wording-experiment instrumentation.

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
