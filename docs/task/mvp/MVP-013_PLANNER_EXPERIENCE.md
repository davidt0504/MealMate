# MVP-013 — Planner experience

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Planning/UI |
| Depends on | MVP-003, MVP-009, MVP-012 |
| Complexity | Complex |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **v3 amendment (2026-08-24, D-028/D-029):** Manual planning is retained; the "Cover My Week" intelligent action moves to MVP-024 on top of the MVP-023 planner. Authoritative source adds `docs/PRD_v3.md` §15; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Let households place, inspect, move, scale, and lock meal components across their real planning cycle with restriction warnings in context.

## Authoritative sources

- `docs/PRD_v2.md` §§7.7–7.8, 10.1–10.2, 24.5–24.6; `docs/ROADMAP.md` D-005; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Render the configured cycle and enabled meal slots, not a fixed week.
- Preserve locks and multi-component meals; warnings remain visible and explainable.
- Make common touch actions accessible without drag-only dependence.

## Scope

- Cycle navigation, add/move/remove/scale/lock actions, recipe picker, states, restriction warnings, and UI tests.

## Non-goals

- Automated plan generation, traditions, nutrition dashboards, collaboration cursors, or notifications.

## Decision gates

- None; use the simplest interaction that passes accessibility and component requirements.

## Acceptance criteria

- **AC-1:** Users can manage components across every configured date/slot without losing state.
- **AC-2:** Locked meals and restriction warnings are visually and semantically clear.
- **AC-3:** Empty/loading/error/retry and narrow-screen/text-scale states remain usable.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Widget + emulator end-to-end flow |
| AC-2 | State tests and semantics inspection |
| AC-3 | Golden/bounded manual matrix where stable, plus widget tests |

## Stop/failure conditions

- Stop if UI flattens meal components or hard-codes weekdays. Two cycles then re-plan.

## Handoff

Record evidence/status in `docs/ROADMAP.md`.
