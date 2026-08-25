# MVP-010 — Recipe photo support

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Recipes/media |
| Depends on | MVP-004, MVP-008 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Emulator storage only unless a later card explicitly authorizes dev cloud |

> **v3 amendment (2026-08-24, D-028/D-029):** Photos are presentation/platform assets referenced from Rust-owned recipe records through the coarse bridge; PRD v3 demotes photo support before cutting controller proof. Authoritative source adds `docs/PRD_v3.md` §6.3, §16; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Allow a user to attach and replace a useful recipe image with bounded storage, privacy, and recovery behavior.

## Authoritative sources

- `docs/PRD_v2.md` §§7.5, 24; `docs/ROADMAP.md` task-register cut rule; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Household media is private by default and separately projected if ever shared.
- Validate type/size, resize appropriately, clean up replacements/deletes, and show upload failure honestly.
- The card may be explicitly cut if the PRD's photo deferral condition is accepted and recorded.

## Scope

- Image selection, bounded processing, storage repository, access rules, recipe display, failure/retry, and cleanup tests.

## Non-goals

- Starter-content licensing, public share images, image search/generation, editing suite, or iOS picker behavior.

## Decision gates

- Before implementation, confirm Done-versus-explicit-cut and local/emulator media-test strategy.

## Acceptance criteria

- **AC-1:** A valid image can be attached, viewed, replaced, and removed.
- **AC-2:** Invalid/oversize inputs and failed uploads preserve recipe integrity.
- **AC-3:** Unauthorized access is rejected and orphan cleanup behavior is tested.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Android integration flow |
| AC-2 | Failure tests |
| AC-3 | Rules/cleanup tests plus independent privacy review |

## Stop/failure conditions

- Stop before cloud Storage activation, copyrighted asset inclusion, or silent orphaning. Two cycles then re-plan.

## Handoff

Record Done or an explicit justified cut in `docs/ROADMAP.md`; never silently omit the card.
