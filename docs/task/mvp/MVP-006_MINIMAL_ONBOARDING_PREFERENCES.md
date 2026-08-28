# MVP-006 — Minimal onboarding and preferences

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Onboarding |
| Depends on | MVP-003, MVP-004, MVP-005 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-006`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Let users begin immediately and optionally set household planning rhythm, meal scope, and restrictions without a sign-in wall or long questionnaire.

## Authoritative sources

- `docs/PRD_v3.md` §15 (First run; no long preference questionnaire), §11 (attention governor), §8 (member-scoped preferences), §2.1
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §10 (progressive profiling), §11 (normal use should train the system)
- `docs/ROADMAP.md` D-015, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 8, 10, 17
- Historical (D-028): `docs/PRD_v2.md` §§7.2–7.4, 10.1, 24

## Load-bearing constraints

- Skip is always available; defaults produce a usable household.
- Restrictions are preferences/warnings, not medical assurance.
- Settings remain editable after onboarding. Cycle and meal scope persist by household; member taste and dislike preferences persist per member (PRD §8). `MVP-023` aggregates them at planning time per PRD §9.10; this card must not pre-average them into a household value.
- No long preference questionnaire. Progressive profiling only: ask when the immediate payoff is apparent, and prefer one question that becomes a durable policy over many that do not (principles §10, PRD §15).
- Durable settings are Rust-owned; ephemeral form state before submission stays in Dart (PRD §13, invariant 17).

## Scope

- Implement minimal first-run state, optional controls, skip path, settings editing, persistence, accessibility, and tests.

## Non-goals

- Invitations, extensive taste calibration, nutrition goals, paywalls, or recommendation UI.

## Decision gates

- Beyond the restriction-scope question below, none; defer new preference fields unless required by MVP-009 or MVP-013.
- **PRD v3 does not decide whether restrictions are household-scoped or member-scoped.** §8 scopes member preferences and says hard restrictions remain hard constraints without assigning their scope. This card records the choice with its reason; `MVP-009`'s filtering and `MVP-023`'s Tier-0 hard filtering both consume it. **Resolved (owner, 2026-08-28): household-scoped.** Reason: PRD §8/§9.4/§9.5 put the member dimension in preference *scoring*, not in Tier-0 hard filtering; MVP-009 AC-3 already says "household restrictions"; and no MVP concept records who attends an occurrence, so a member-scoped set would always be unioned to the household set at planning time. Attribution ("Sam: peanuts") is a plan-level UX detail (an optional note), not a scope change. Revisit when attendance or per-member meals exist; that is a migration step, not a redesign.

## Acceptance criteria

- **AC-1:** A user can skip onboarding and reach a usable dinner-first app.
- **AC-2:** Cycle, meal scope, and restrictions can be saved and later edited.
- **AC-3:** No account creation or long intake is required.
- **AC-4:** Member taste and dislike preferences are stored and read keyed by member, not by household.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Widget/integration skip-path test |
| AC-2 | Persistence and settings tests |
| AC-3 | Fresh-install emulator walkthrough |
| AC-4 | Storage-layer test writing preferences for two `member_id` values against MVP-002's household-member table and reading both back distinctly |

## Stop/failure conditions

- Stop if onboarding becomes mandatory, claims safety, or expands into recommendation calibration. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, and **Next implementation task**.
