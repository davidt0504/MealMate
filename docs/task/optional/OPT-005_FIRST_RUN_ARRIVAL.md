# OPT-005 — Branded first-run arrival and optional orientation

> Optional planning input, not an MVP dependency or approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Optional post-MVP implementation |
| Workstream | First-run experience / brand arrival |
| Depends on | MVP-033, MVP-034 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | Only when it cannot delay a required MVP or launch card |
| Recommended workflow | `deep-options` on the arrival decision gate, then `plan-task` |
| External actions | Owner-device usability observation only |

## Workflow gate

Before planning or execution, read `docs/ROADMAP.md` and apply the mandatory planning or execution gate in `docs/task/README.md` for `OPT-005`. Because this card is optional, the owner must explicitly select it without displacing required MVP or launch work.

## Outcome and user value

A new household may meet Kimatta with one quiet, intentional arrival screen before its first covered-week proposal. The screen offers an immediate route to the answer and an optional, skippable orientation for someone who wants context. It creates a sense of arrival without turning the first use into a questionnaire, a setup flow, or a required tutorial.

The primary route remains the existing promise: a populated **Cover My Week** proposal. The optional route explains only the three things a person can do from that proposal — accept it, change a dish, or tell Kimatta something it could not check — then returns them to the same result. It asks for no household facts and writes no preferences.

## Authoritative sources

- `.impeccable.md` — settled, unhurried, warm personality; *kanso*, *ma*, *shibui*; answer-first, warmth-by-voice, accessible motion; especially the warning that a screen needing a tutorial is wrong
- `docs/PRD_v3.md` §2.3 (no control-systems vocabulary), §11 (attention governor), §15 (mature target: this week is covered), principle 8 (learn from ordinary use rather than interrogation)
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §7 (candidate-action completeness), §21 (no artificial scarcity), and the rejected giant-onboarding pattern
- `docs/ROADMAP.md` D-039 (only the reversible `Kimatta (dev)` display identity is approved; wordmark lockups remain unused), D-041 (MVP-033's answer-first supersession), and the task-status contract
- `docs/task/mvp/MVP-033_FIRST_RUN_COVERED_WEEK.md` — the first-run path this card may deliberately revise only after MVP-033 is Done
- `docs/task/MVP_INVARIANTS.md` 17 and 19

## Load-bearing constraints

- This card does **not** reopen or block MVP-033. Until this optional card is selected, the shipped first screen remains Cover My Week exactly as MVP-033 specifies.
- The primary action has no precondition, field, chip, account, or choice beyond tapping it, and it reaches the same filled Cover proposal after the bounded starter install.
- Orientation is optional and escapable at every point. It collects no data, records no policy, and never makes the person finish an educational sequence to reach their week.
- “Guided” means a short explanation of the existing result and corrections, not a product tour, coach-mark overlay, or configuration wizard. If it needs more than three calm beats, return to the decision gate rather than expanding it.
- MVP-034 supplies the chosen tokens. Until `DEC-004` clears a permanent identity, use only the already approved reversible `Kimatta (dev)` text and app icon; do not introduce the unused wordmark lockups or a new trademark claim.
- Rust remains authoritative for durable first-run state. Do not overload `onboarded` with an arrival-screen dismissal; decide and name any new state explicitly at planning.
- Motion is limited to route/state transitions, honors reduced motion, and never celebrates completion. All paths work at system text scale 200% and with keyboard/screen-reader navigation.

## Scope

- One airy first-run arrival screen, using MVP-034's approved typography, color, spacing, and icon treatment.
- A clear primary action such as **Cover my week**, which routes to the existing first Cover proposal.
- An optional **How Kimatta works** route of at most three concise screens or states, each with an always-visible route to Cover.
- Durable, Rust-owned dismissal semantics only if the decision gate selects showing the arrival once per household; loading, error, restoration, and back-navigation behavior; widget and owner-device evidence.

## Non-goals

- Any required questionnaire, profile setup, dietary/restriction intake, account/sign-in, forced tutorial, coach-mark overlay, celebratory animation, change to Cover's planning logic, wordmark/logo redesign, or permanent naming decision.
- Replacing the current first-run path before this card is selected and passes its evidence.

## Decision gates

1. **Does an arrival screen earn its one extra decision?** Compare the current direct-to-Cover path with a prototype of the arrival screen on the owner's phone. The arrival proceeds only if it preserves a clear, fast route to a covered week and the owner judges the brand/context benefit worth the extra tap. Otherwise close the card without implementation.
2. **What ends the arrival state?** Options: primary route chosen; any route away; orientation completed/skipped; first render. The selected behavior must never re-show an unavoidable arrival after a person has already reached Cover, and must state whether an interrupted orientation resumes or abandons safely.
3. **What is the optional orientation's smallest useful content?** Default: three result-oriented beats — Kimatta proposes dinner, the household can change a dish, and uncertainties are stated rather than hidden. Reject any content that asks for information or teaches controls absent from Cover.

## Acceptance criteria

- **AC-1:** On a fresh household, the arrival screen is calm, accessible, and contains a plainly primary direct route to a populated Cover proposal; it shows no input, account, restriction, or configuration control.
- **AC-2:** The primary route reaches Cover with the same starter-install, timeout, failure, and in-shell recovery guarantees as MVP-033; it does not expose an empty Plan screen.
- **AC-3:** The optional orientation can be skipped from every state, takes no more than three beats, writes no household data, and always ends at the same Cover proposal.
- **AC-4:** Once the selected completion condition has occurred, cold launch and restoration do not trap or repeatedly force the arrival/orientation; durable state is Rust-owned and household-scoped.
- **AC-5:** Visual implementation uses MVP-034 tokens, honors reduced motion and system text scale 200%, and passes the app's accessibility guidelines at representative small-phone and large-text viewports.
- **AC-6:** An owner-device comparison records the current direct path and the arrival path, tap count and elapsed time to a visible shopping list, and the owner’s decision on whether the added screen earns its cost.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1–AC-4 | Widget tests with delayed/failing starter install, restored routes, and all orientation exits |
| AC-5 | Widget accessibility/text-scale checks plus screenshots at representative phone viewports |
| AC-6 | Owner phone, release APK, documented tap/time comparison and decision-gate result |

## Stop/failure conditions

- Stop if the screen becomes an intake, an unavoidable tutorial, a second brand decision, or a prerequisite for a covered-week answer.
- Stop if the prototype makes the direct route slower or less understandable without a recorded owner decision that the trade-off is worthwhile.
- Stop if tokens or identity treatment are not resolved by MVP-034, or if the work would delay required MVP or launch work.
- After two failed remediation cycles, return to the decision gate rather than expanding the experience.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the decision-gate result, resulting status, evidence, residual risk, and whether the optional screen was implemented or closed without implementation.
