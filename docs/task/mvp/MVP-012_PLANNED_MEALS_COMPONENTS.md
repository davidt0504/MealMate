# MVP-012 — Planned meals and components

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Planning/domain |
| Depends on | MVP-004, MVP-005, MVP-007 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

## Outcome and user value

Persist a meal occurrence on a real date with one or more recipe components, optional serving scales, and a lock that later automation must respect.

## Authoritative sources

- `docs/PRD_v2.md` §§7.7, 10.1–10.2, 14.4; `docs/ROADMAP.md` D-005; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Occurrence, date/slot, components, and component scaling are distinct concepts.
- A locked occurrence cannot be moved/replaced by later automation.
- Records are granular enough to reduce last-write-wins conflict damage.

## Scope

- Implement domain behavior, persistence/rules, add/remove/reorder components, serving scale, lock state, and contract tests.

## Non-goals

- Planner screen, automated suggestions, traditions, non-recipe meals, or real-time collaboration conflict resolution.

## Decision gates

- Resolve component ordering/identity and recipe-deletion reference behavior before persistence is finalized.

## Acceptance criteria

- **AC-1:** Multi-component occurrences round-trip by real date and enabled slot.
- **AC-2:** Component scales and ordering remain stable through edits.
- **AC-3:** Lock behavior is explicit and tested for future automation contracts.
- **AC-4:** Unauthorized household access fails.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Domain/repository integration tests |
| AC-2 | Edge-case/property tests |
| AC-3 | Contract tests written independently of UI |
| AC-4 | Rules tests and fresh-context schema review |

## Stop/failure conditions

- Stop if the schema assumes one recipe per meal or one document per whole plan. Two cycles then re-plan.

## Handoff

Record evidence/status in `docs/ROADMAP.md`.
