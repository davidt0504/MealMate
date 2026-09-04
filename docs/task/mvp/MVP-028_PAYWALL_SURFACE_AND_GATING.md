# MVP-028 — Paywall surface and feature gating

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Monetization |
| Depends on | MVP-027, DEC-006 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-028`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

A household can see plainly what is paid, what is not, and what it would cost — stated honestly, with the deterministic planner never behind the gate.

## Authoritative sources

- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §21 — permitted monetization axes; "artificially metered deterministic intelligence" is excluded
- `docs/PRD_v3.md` §20 ("No ads"), `:1004` ("core deterministic intelligence should be useful without artificial scarcity")
- `docs/task/decision/DEC-006_MONETIZATION_MODEL_DECISION.md` — the boundary, packaging and price this surface renders
- `docs/task/mvp/MVP-027_HOUSEHOLD_ENTITLEMENT_MODEL.md` — the state set this surface reads
- Historical, subject to `DEC-006` decision 8: `docs/PRD_v2.md:788-793` (fairness constraints on packaging experiments)
- `docs/ROADMAP.md` D-023, D-040; `docs/task/MVP_INVARIANTS.md` 16, 19

## Load-bearing constraints

- **Explicit non-gate list, and it is the card's most important constraint.** The deterministic planner, Cover My Week, restriction warnings, the shopping-list derivation, and every other piece of core local intelligence may **not** sit behind this gate — `HOUSEHOLD_CONTROL_PRINCIPLES.md` §21 excludes metering deterministic work and `PRD_v3.md:1004` requires it to be useful without artificial scarcity. The gating seam must make gating them impossible to express, not merely absent.
- **Play's pre-purchase disclosure is mandatory.** Price, billing period, auto-renewal, and how to cancel must all be visible before a purchase can be initiated. "No dark patterns" is a posture; this is the testable requirement, and its absence is a store-policy violation.
- Price is never personalised on inferred wealth, vulnerability or willingness to pay (`PRD_v2.md:788-793`). Packaging and presentation may vary by cohort; the number may not be individually derived.
- No ads, in any state of this surface.
- The gate reads `MVP-027`'s entitlement and nothing else. **Unknown renders as the free experience** without asserting the household has not paid.
- Accessibility parity with the rest of the app: text scaling, semantic labels, focus order, tap targets, and contrast in both themes — the paywall is not exempt because it sells something.
- Grace-period and account-hold states need user-facing messaging, since a household in those states is neither plainly paid nor plainly free and silence would be misleading.

## Scope

- The paywall surface: what is offered, at what price, on what terms, with the mandatory pre-purchase disclosure.
- The gating seam: a single, testable point that reads entitlement and decides availability, with the non-gate list enforced structurally.
- Copy for every entitlement state `MVP-027` enumerates, including grace period, account hold, paused and unknown.
- Empty, error and offline states for the surface.
- Accessibility and text-scale coverage on the new screens.
- Faked-entitlement test harness, so the whole card is verifiable with no billing integration behind it.

## Non-goals

- Any purchase, restore or cancel flow, any billing SDK, receipt or server-side validation, RTDN (all `MVP-029`); the entitlement model itself (`MVP-027`); price or packaging policy (`DEC-006`); purchase telemetry (`MVP-029`); cohort experimentation infrastructure.

## Decision gates

- What is offered and at what price is `DEC-006`'s. If unresolved, this card may build the seam and the state rendering against a placeholder, but may not invent a boundary — and must say so in its evidence rather than shipping a guess.

## Acceptance criteria

- **AC-1:** The gating seam cannot express gating of the planner, Cover My Week, restriction warnings or the shopping-list derivation; a repository grep plus an inspection record demonstrates the structural impossibility, not merely the current absence.
- **AC-2:** The pre-purchase disclosure shows price, billing period, auto-renewal and how to cancel, and the purchase affordance is unreachable until it has been shown.
- **AC-3:** Every entitlement state from `MVP-027` renders distinct, accurate copy; grace period and account hold are named to the user rather than shown as plain free or plain paid.
- **AC-4:** Unknown entitlement renders the free experience and no surface asserts the household has not paid.
- **AC-5:** The surface meets the accessibility guidelines in light and dark at text scale 2.0 — tap targets, labels and contrast — matching the bar the existing screens hold.
- **AC-6:** No ad surface, and no price value derived from any user attribute; asserted by inspection and by grep.
- **AC-7:** The whole card is verifiable against a faked entitlement value with no billing integration present.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Type-level or API-level demonstration plus a grep record over the seam's call sites |
| AC-2 | Widget test asserting the disclosure precedes the affordance, plus a screenshot |
| AC-3 | Widget tests over each state with exact expected copy |
| AC-4 | Widget test for the unknown state asserting the absent claim |
| AC-5 | Accessibility guideline tests in both themes plus a text-scale screenshot |
| AC-6 | Repository grep and an independent inspection record |
| AC-7 | The full suite running green with the faked-entitlement harness and no billing dependency |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- **Stop if any proposed gating would place deterministic planning behind the paywall** — a §21 violation is not a tradeoff to be weighed against packaging.
- Stop if the surface would require a billing dependency to be testable; that is a sign the seam is in the wrong place.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, decisions, blockers, MONETIZATION-READY progress, and **Next implementation task**. Select a dependent next only when this card is `Done` and that dependent passes its applicable planning or decision workflow gate.
