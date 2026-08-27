# MVP-003 — Android app shell and navigation

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | App shell |
| Depends on | MVP-002, DEC-002, DEC-005 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No; calibrate task sizing after completion |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | None |

> **v3 amendment (2026-08-24, D-028/D-029):** D-023 Riverpod/go_router stands; add a "Cover My Week" destination placeholder (MVP-024). Depends on: +DEC-005. Authoritative source adds `docs/PRD_v3.md` §15–16; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; the roadmap keeps it blocked until MVP-002 is explicitly promoted to Done, after which a current approved plan must fold this amendment into the card before implementation.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-003`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Provide a coherent Android shell that exposes the future recipes, planner, pantry, shopping, and settings destinations without implementing them prematurely.

## Authoritative sources

- `docs/PRD_v2.md` §§6–7, 13.1, 14.1; `docs/ROADMAP.md` D-002, D-015; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Android phone layout and accessibility first.
- Navigation state is testable; placeholders make unavailable functionality honest.
- No iOS, desktop, full web app, Firebase, or feature logic.
- Replacing `lib/main.dart`'s `HealthScreen` must keep the database path app-private — `getApplicationSupportDirectory()` (`path_provider`), never `Directory.systemTemp`, which resolves to the unwritable `/data/local/tmp` on Android. MVP-002 fixed this and it has no automated test; see that card's AC-3.

## Scope

- Implement theme tokens, shell/navigation, route error handling, placeholder destinations, and widget/navigation tests.

## Non-goals

- Polished visual system, onboarding flow, deep links, domain CRUD, or backend state.

## Decision gates

- None beyond DEC-002.

## Acceptance criteria

- **AC-1:** All five primary destinations are reachable and route restoration/error behavior is deterministic.
- **AC-2:** Text scaling, semantic labels, focus/tap targets, and light/dark contrast receive bounded checks.
- **AC-3:** Analyze, widget tests, and Android smoke launch pass.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Navigation widget tests |
| AC-2 | Accessibility test/inspection record |
| AC-3 | Commands plus emulator screenshot/log |

## Stop/failure conditions

- Stop if shell work starts owning feature/domain state or requires an unapproved platform. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, first-three-card sizing calibration, and **Next implementation task**.
