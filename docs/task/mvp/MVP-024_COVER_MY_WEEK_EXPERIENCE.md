# MVP-024 — Cover My Week experience

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Planning/UI |
| Depends on | MVP-013, MVP-015, MVP-023 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | None |

> **Body derived 2026-08-26 (D-029, D-034)** after DEC-005 (D-030).

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-024`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Add the defining intelligent action on top of manual planning: the user locks what matters, the controller fills or repairs the rest, the UI shows only material unresolved decisions, the user accepts or swaps, and the shopping list is already derived. Default states trend toward "This week is covered" or "2 things need you" rather than an empty grid demanding reconstruction.

This card carries PRD §22's required MVP proof. `MVP-022` cannot recommend a go without its evidence.

## Authoritative sources

- `docs/PRD_v3.md` §15 (defining workflow; mature design target), §11 (attention governor), §10 (model sufficiency), §22
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "UI pivot"
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §23 ("Nothing needs you" is a valid UI state), §30 (anti-patterns), §8, §11
- `docs/ROADMAP.md` D-023, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 17, 18, 19, 20, 21

## Load-bearing constraints

- **Manual planning is never removed or degraded** to push users toward the automated action (PRD §15, `MVP-013`).
- The surface is an **exception console, not a feed**. No infinite feed, streak, badge, gamification, daily summary, or engagement notification (principles §23, §30; PRD §3 item 13).
- Only decisions the assessment marks **material** are surfaced; low-value and curiosity questions stay silent (PRD §11).
- The UI never claims safety, exact stock, or personalization the model cannot support, and never presents a "covered" state the assessment does not support (invariant 19, PRD §10).
- **Severity, safety and reversibility override attention-saving** (PRD §22 "minimizing attention hides important uncertainty"). Quietness is never bought with a suppressed hard-constraint problem.
- Locked meals are Tier-0 and visibly preserved; nothing locked moves (invariant 18).
- The screen holds no durable state and issues coarse bridge commands; the plan, the assessment and the list all come from Rust (invariants 17, 21).
- Ordinary use is the training signal: accepts, swaps and vetoes become evidence without a separate feedback chore (principles §11).

## Scope

- The Cover My Week action and its result presentation.
- Coverage state and the assumptions PRD §10 marks material, in the user's language rather than control-systems terminology (PRD §2.3).
- Accept, swap and lock with minimal effort from the result view.
- The derived shopping list surfaced as already ready — no separate generate step (PRD §15 step 7).
- Default states trending toward "This week is covered" / "N things need you".
- Recording accept, swap and veto as ordinary structured evidence for future cycles.
- Empty, loading, error and retry states; accessibility to the standard `MVP-003` and `MVP-013` set.

## Non-goals

- The planner itself (`MVP-023`); fixtures and invariant tests (`MVP-025`).
- Notifications of any kind, a household dashboard, a second controller, proactive background replanning (PRD §21 Phase 3).
- Explaining the tier system or the search algorithm to the user.

## Decision gates

- **Where Cover My Week lives in the shell is settled by `MVP-003`.** If that placement is absent when this card is planned, stop and route the question to `MVP-003` rather than deciding it here.
- If the assessment cannot distinguish material from immaterial assumptions well enough to keep the default surface quiet, stop — that is a `MVP-023` assessment gap, not a UI copy problem.

## Acceptance criteria

- **AC-1:** From a cycle with unresolved slots, one action produces a coherent plan with no network available.
- **AC-2:** Locked meals are visually and semantically marked, and nothing locked moves.
- **AC-3:** The screen surfaces exactly the assumptions the assessment marks material and no others; a fixture whose only uncertainty is low-value produces no question.
- **AC-4:** Accept, swap and veto are reachable in minimal steps, and all three are recorded as evidence.
- **AC-5:** The derived shopping list is available immediately after acceptance, with no separate generate step.
- **AC-6:** No engagement surface exists in the build — no streak, feed, badge, or scheduled notification.
- **AC-7:** Text scaling, semantic labels, focus and tap targets, and light/dark contrast pass the bounded checks.
- **AC-8:** A dated attention measurement is recorded for at least four household scenarios constructible through the app UI available at this card's point in `docs/task/SEQUENCE.txt` (`MVP-006` preferences, `MVP-008` recipes, `MVP-012`/`MVP-013` slots). For each: active seconds and explicit decisions to reach an accepted, ready plan via Cover My Week and via the manual `MVP-013` path; filled enabled slots counted from the grid and restriction warnings counted from `MVP-009`'s surface, for both arms; arm order counterbalanced. The record states the median ratio of active seconds. The manual arm is observed, not gated.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Widget test plus an emulator run with the network disabled |
| AC-2 | State tests and semantics inspection over a many-locked-meals fixture |
| AC-3 | Paired fixtures — one material, one low-value — asserting question count |
| AC-4 | Interaction tests plus a ledger assertion that the evidence row was written |
| AC-5 | Integration test from accept to list, asserting no intermediate action |
| AC-6 | Repository grep and an independent inspection record |
| AC-7 | Accessibility test/inspection record and a text-scale screenshot |
| AC-8 | Dated measurement record: per-scenario active seconds and decision counts for both arms, operator(s) with one row per arm, arm order, filled-slot and warning counts, and the median ratio |

## Stop/failure conditions

- Stop if the UI asserts coverage the assessment does not support, if manual planning is degraded, if an engagement surface is introduced, or if a safety-relevant uncertainty is suppressed to keep the screen quiet.
- If the AC-8 measurement shows no material attention reduction, stop: that is a planner or experience gap, not a measurement artifact. An adverse result is a material change to `MVP-023`'s recorded decisions and returns that card to Draft under D-031, with the AC-8 record as the trigger — the same route `MVP-025`'s benchmark verdict takes.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record PASS/FAIL/NOT VERIFIED evidence, the resulting status, LOCAL-CORE-LOOP-READY progress, and **Next implementation task**, preserving this evidence for `MVP-022`'s PRD §22 proof. Update the corresponding checklist rows in `docs/V3_IMPLEMENTATION_STATUS.md` without duplicating live current/next status there.
