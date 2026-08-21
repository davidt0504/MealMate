# <ID> — <Outcome>

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation / Decision / Readiness |
| Workstream | <name> |
| Depends on | <IDs or —> |
| Complexity | Focused / Complex / Mechanical |
| Assurance | Standard / Elevated |
| Sequential batching | Allowed only after prior card is Done / No |
| Recommended workflow | `<workflow>` |
| External actions | None / explicit list and authority boundary |

## Outcome and user value

<One bounded outcome and why it matters.>

## Authoritative sources

- `docs/PRD_v2.md` §<sections>
- `docs/ROADMAP.md` decisions <IDs>
- `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- <Three to seven task-specific constraints.>

## Scope

- <Required work.>

## Non-goals

- <Explicit exclusions and future work.>

## Decision gates

- <Choice requiring resolution before implementation, or “None.”>

## Acceptance criteria

- **AC-1:** <Observable outcome.>

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | <Automated test, emulator/device result, build artifact, or bounded manual inspection> |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

Record PASS/FAIL/NOT VERIFIED evidence, decisions, blockers, and the resulting status in `docs/ROADMAP.md`. Promote dependents only when this card is Done.
