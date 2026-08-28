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

> **Re-derived for PRD v3 on 2026-08-28 (D-029).** The 2026-08-24 amendment (D-023 stands; Cover My Week placeholder; +DEC-005) is folded into the body. Cover My Week is placed under Plan (`/plan/cover`), not a sixth tab: `NavigationBar` caps at five destinations and PRD v3 §15 treats Cover My Week as an action on the plan. Promoted `Draft → Ready` at the 2026-08-27 MVP-002 closeout and `Ready → In Progress` at this card's execution (2026-08-28).

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-003`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Provide a coherent Android shell that exposes the future recipes, planner, pantry, shopping, and settings destinations without implementing them prematurely.

## Authoritative sources

- `docs/PRD_v3.md` §6.3 (coarse bridge), §13 (Flutter owns route/navigation state), §15–16 (defining workflow; Required UI); `docs/ROADMAP.md` D-002, D-015, D-022, D-023, D-024, D-029, D-031, D-033; `docs/task/MVP_INVARIANTS.md` 14, 16, 17, 21. Historical: `docs/PRD_v2.md` §§6–7, 13.1, 14.1.

## Load-bearing constraints

- Android phone layout and accessibility first.
- Navigation state is testable; placeholders make unavailable functionality honest.
- No iOS, desktop, full web app, Firebase, or feature logic.
- Replacing `lib/main.dart`'s `HealthScreen` must keep the database path app-private — `getApplicationSupportDirectory()` (`path_provider`), never `Directory.systemTemp`, which resolves to the unwritable `/data/local/tmp` on Android. MVP-002 fixed this and it has no automated test; see that card's AC-3.
- Invariants 17/21: the shell holds no durable state and makes one coarse bridge call (`healthCheck`) at startup; no SQLite handles and no per-field calls cross the bridge.

## Scope

- Implement theme tokens (one seed, light/dark), the five-destination shell (Recipes, Plan, Pantry, Shopping, Settings) on `go_router` + `NavigationBar`, a `Cover My Week` placeholder route under Plan (MVP-024 lands the real one), unknown-route and restoration handling, honest placeholder destinations, and widget/navigation/accessibility tests. Plan-introduced diagnostic (not a PRD surface): Settings shows the local database schema version and path so MVP-002's AC-3 evidence keeps an in-app carrier.

## Non-goals

- Polished visual system, onboarding flow, deep links, domain CRUD, or backend state; no bridge calls beyond `healthCheck`.

## Decision gates

- None beyond DEC-002.

## Acceptance criteria

- **AC-1:** All five primary destinations and `/plan/cover` are reachable; an unknown route renders the not-found screen with a working return action; the selected destination survives state restoration.
- **AC-2:** Text scaling, semantic labels, focus/tap targets, and light/dark contrast receive bounded checks.
- **AC-3:** Analyze, widget tests, and Android smoke launch pass.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Navigation widget tests (five destinations, /plan/cover, unknown route, restoration with a fresh router) |
| AC-2 | Guideline matchers (tap target, labelled tap target, text contrast) light+dark per destination; destination screens at text scale 2.0 (`NavigationBar` labels clamp at 1.3 by framework design); tab-traversal test or inspection-only record |
| AC-3 | Command-contract exit codes plus emulator screenshots, `run-as` DB listing, and process-kill relaunch |

## Stop/failure conditions

- Stop if shell work starts owning feature/domain state or requires an unapproved platform. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, first-three-card sizing calibration, and **Next implementation task**.

## PRD v3 §26 traceability

No §26 item is discharged by this card; it is the UI carrier for items 5, 7, 8 and 12 (owned by MVP-008, MVP-013, MVP-024 and MVP-016 respectively — see the roadmap's §26 table, which stays the sole authority).

## Dependencies added by this card

- `go_router 18.0.0` — named by D-023; first call site is the shell (`lib/app/router.dart`).
- `flutter_riverpod 2.6.1` — named by D-023; first call site is the startup health provider (`lib/features/settings/health_provider.dart`).

D-031: in-scope, so card evidence, not a Draft event. The resolved Riverpod major is 2.x, which has no provider-retry behaviour to disable — `ProviderScope` gains its `retry` argument only in 3.0 — so no retry is configured at any call site. A failed database open is shown on Settings and is not re-run with backoff.
