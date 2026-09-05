# MVP-033 — First run lands on a covered week

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Planner experience / onboarding |
| Depends on | MVP-032, MVP-024, MVP-006 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None (emulator and owner-device evidence only) |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-033`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

The first thing a new household sees is a proposed covered week with an Accept button, not a welcome screen and not an empty grid. Two taps from install to an accepted plan and a derived shopping list, with no question asked. The persona is a tired parent who installed the app to hand off "what's for dinner"; every screen between them and that answer is a cost. Corrections happen by reacting on the result (Swap, "Never suggest…", the restrictions prompt), which MVP-024 already ships.

## Authoritative sources

- `docs/PRD_v3.md` §15 (mature target: "This week is covered", never an empty grid requiring reconstruction; skip always available), §11 (attention governor: no question merely because data would be useful), §2.3 (no control vocabulary), principle 8 (learn from ordinary use rather than interrogating users)
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` — exception console, not a feed; onboarding questionnaires rejected by default
- `docs/task/mvp/MVP-006_MINIMAL_ONBOARDING_PREFERENCES.md` AC-1/AC-3 and `docs/task/mvp/MVP-024_COVER_MY_WEEK_EXPERIENCE.md` — the surfaces this card rewires
- `docs/ROADMAP.md` D-041 (promotion; MVP-006 AC-1 supersession); `.impeccable.md` design principles 1–4
- `lib/app/app.dart` (first-run gate and starter install), `lib/app/router.dart`, `lib/features/onboarding/welcome_screen.dart`, `lib/features/planning/cover_screen.dart`, `lib/features/planning/cover_copy.dart`, `test/cover_copy_test.dart`
- `docs/task/MVP_INVARIANTS.md` 8, 17, 19

## Load-bearing constraints

- **Zero input before the result.** No screen with a field, a chip or a choice precedes the first Cover proposal. Assumptions (household of one, dinner-only seven-day cycle from today, no restrictions) are already stated on the result as "What we couldn't check"; they are not restated on a screen of their own.
- **Starter install completes before the first Cover runs**, otherwise the first impression is "Nothing we know of fits this slot". The await applies only on the `onboarded == false` path; later launches keep the non-blocking install MVP-006 pinned ("a failed completion still lets the user in"), and a failed install still routes on.
- **One first-run sentence, then silence.** Under the banner, only until the first-run state ends: a sentence that says what Kimatta does for them and that anything can be changed. It lives in `cover_copy.dart` so `test/cover_copy_test.dart` samples it: no safety claim (invariant 19), no control-systems word (PRD §2.3). If a second sentence seems necessary, the screen has a design problem to fix instead.
- **The Cover screen stays inside the shell**, so the error arms (install failed, cover failed, household not loaded) always land on a screen with the navigation bar and an explanation, never a dead end. Welcome existed outside the shell and needed its own skip path for this reason; deleting it removes that hazard rather than moving it.
- **Tonight first.** The cycle anchors on the household's creation date (`PlanningCycle::default_for`), so on first run the first slot is today by construction. Verify at plan time; no ordering code is expected.
- **Rust owns `onboarded`.** The flag flips through the existing `complete_onboarding` bridge call, never a Dart-side store (invariant 17).
- MVP-006 AC-1 ("a user can skip onboarding") is superseded by D-041: there is no onboarding to skip. AC-3 (no account, no long intake) is strengthened, not changed.

## Scope

- First-launch routing: when the household DTO resolves with `onboarded == false`, await the starter install (bounded; a failure or timeout still proceeds), then route to `/plan/cover`.
- Flip `onboarded` at the point the decision gate below settles; the first-run sentence renders only while the first-run state holds.
- Delete `welcome_screen.dart`, the `/welcome` route, `welcomeLocation`, and their tests; update `app_test.dart` for the new first-launch path.
- Add the first-run sentence to `cover_copy.dart`; render it under the banner in `cover_screen.dart` on the first-run path only.
- Emulator and owner-device evidence; the stopwatch measurement in AC-6.

## Non-goals

- Any new planner mode (tonight-only proposals), restriction or allergy pre-questions, coach marks or stepped tours, auto-accepting the proposal, changes to Cover's sections or decisions, theming (`MVP-034`), and any change to what "covered" means.

## Decision gates

- **When first-run ends.** Options: on first Accept; on first navigation away from the first Cover; on first render. Constraint: it ends no later than the user's first navigation away, so a household that prefers manual planning is never routed to Cover twice, and the sentence never shows after it ends. Planning picks the simplest mechanism that meets the constraint and records the choice.
- Whether `RESTRICTIONS_NOT_CONFIGURED` should render in "Needs you" rather than "What we couldn't check" on the first run only. Default: no change; the CTA already exists on the same screen. Record the choice.

## Acceptance criteria

- **AC-1:** After `pm clear` and launch, the first screen is the Cover proposal with at least one filled slot and the first-run sentence; no field, chip or choice was shown before it.
- **AC-2:** After Accept, the shopping list is non-empty. A second cold launch lands on `/plan` with no sentence and no Cover routing.
- **AC-3:** The starter install is awaited on the first-run path and not awaited on later launches; a failing install on first run still routes to Cover and reports the failure (widget test with the install provider overridden to throw / to delay).
- **AC-4:** A Cover failure or an unloaded household on first run lands on an in-shell screen with the navigation bar and an explanation; no path strands the user.
- **AC-5:** No `/welcome` route, no `welcome_screen.dart`, no dead `welcomeLocation` reference; `flutter analyze` and `flutter test` pass; `cover_copy_test.dart` samples the new sentence.
- **AC-6:** On the owner's phone, release APK via `tools/device.sh`: install → accepted plan with the shopping list visible in at most 3 taps and under 60 seconds, recorded with the tap count and time.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | `tools/emulator.sh reset` then launch in airplane mode; screenshot of the first screen |
| AC-2 | Same run continued: shopping screenshot; second launch screenshot |
| AC-3 | Widget tests in `app_test.dart` with overridden `starterInstallProvider` |
| AC-4 | Widget tests with the cover and household providers overridden to error |
| AC-5 | `grep` for the removed symbols in the handoff; command output |
| AC-6 | Owner-recorded stopwatch and tap count in the handoff |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- Stop if `MVP-032` is not Done: with no shipped starters the first Cover is empty and this card's outcome is unreachable.
- Stop if the design requires any input before the first result; that is a scope change for the owner.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, the decision-gate choices, blockers, and **Next implementation task** (`MVP-034` when this card is Done).
