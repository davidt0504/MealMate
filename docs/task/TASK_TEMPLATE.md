# <ID> — <Outcome>

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
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

## Workflow gate

- **Implementation / Readiness / Optional:** Before planning or execution, read `docs/ROADMAP.md` and apply the mandatory planning or execution gate in `docs/task/README.md` for `<ID>`. Do not begin implementation unless it passes. Any implementation plan must repeat the execution gate as its first execution step.
- **Decision:** Before resolving the decision, confirm every declared dependency is Done with evidence and the roadmap identifies `<ID>` as the current required decision or explicit owner-paced work. It does not need to be the Next implementation task and does not occupy the implementation lane.

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

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, decisions, blockers, delivery-gate progress, and **Next implementation task**. Select a dependent next only when this card is `Done` and that dependent passes its applicable planning or decision workflow gate; never promote a status without its required evidence and explicit owner approval.
