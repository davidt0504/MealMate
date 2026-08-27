# MVP-013 — Planner experience

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Planning/UI |
| Depends on | MVP-003, MVP-009, MVP-012 |
| Complexity | Complex |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-013`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Let households place, inspect, move, scale, and lock meal components across their real planning cycle with restriction warnings in context.

## Authoritative sources

- `docs/PRD_v3.md` §15 ("Manual planning always remains available"), §13 (Flutter owns route/view state), §6.3, §16
- `docs/ROADMAP.md` D-005, D-023, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 5, 17, 18, 21
- Historical (D-028): `docs/PRD_v2.md` §§7.7–7.8, 10.1–10.2, 24.5–24.6

## Load-bearing constraints

- Render the configured cycle and enabled meal slots, not a fixed week.
- Preserve locks and multi-component meals; warnings remain visible and explainable.
- Make common touch actions accessible without drag-only dependence.
- Manual planning is retained and always available; it is never degraded to push users toward the automated action (PRD §15).
- The screen consumes a coarse cycle-view DTO and issues bridge commands; it holds no durable state (invariants 17, 21).

## Scope

- Cycle navigation, add/move/remove/scale/lock actions, recipe picker, states, restriction warnings, and UI tests.

## Non-goals

- Automated plan generation — the deterministic planner is MVP-023 and the "Cover My Week" action that invokes it is MVP-024, built on top of this screen. Also excluded: traditions, nutrition dashboards, collaboration cursors, notifications.

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

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, and **Next implementation task**.
