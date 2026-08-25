# MVP-006 — Minimal onboarding and preferences

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Onboarding |
| Depends on | MVP-003, MVP-004, MVP-005 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **v3 amendment (2026-08-24, D-028/D-029):** No preference questionnaire; progressive profiling only — ask when the immediate payoff is apparent (principles §10). Authoritative source adds `docs/PRD_v3.md` §15; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Let users begin immediately and optionally set household planning rhythm, meal scope, and restrictions without a sign-in wall or long questionnaire.

## Authoritative sources

- `docs/PRD_v2.md` §§7.2–7.4, 10.1, 24; `docs/ROADMAP.md` D-015; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Skip is always available; defaults produce a usable household.
- Restrictions are preferences/warnings, not medical assurance.
- Settings remain editable after onboarding and persist by household.

## Scope

- Implement minimal first-run state, optional controls, skip path, settings editing, persistence, accessibility, and tests.

## Non-goals

- Invitations, extensive taste calibration, nutrition goals, paywalls, or recommendation UI.

## Decision gates

- None; defer new preference fields unless required by MVP-009 or MVP-013.

## Acceptance criteria

- **AC-1:** A user can skip onboarding and reach a usable dinner-first app.
- **AC-2:** Cycle, meal scope, and restrictions can be saved and later edited.
- **AC-3:** No account creation or long intake is required.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Widget/integration skip-path test |
| AC-2 | Persistence and settings tests |
| AC-3 | Fresh-install emulator walkthrough |

## Stop/failure conditions

- Stop if onboarding becomes mandatory, claims safety, or expands into recommendation calibration. Two cycles then re-plan.

## Handoff

Record evidence/status in `docs/ROADMAP.md`.
