# MVP-010 — Recipe photo support

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Recipes/media |
| Depends on | MVP-004, MVP-008 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Local device storage only; cloud object storage only if MVP-018 or DEC-003 explicitly authorizes it |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-010`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Allow a user to attach and replace a useful recipe image with bounded storage, privacy, and recovery behavior.

## Authoritative sources

- `docs/PRD_v3.md` §6.3 (coarse bridge), §13 (platform adapters may stay in Dart/Kotlin), §16 "Demote before cutting controller proof"
- `docs/ROADMAP.md` task-register cut rule, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 11, 12, 15, 17, 21
- Historical (D-028): `docs/PRD_v2.md` §§7.5, 24

## Load-bearing constraints

- Household media is private by default and separately projected if ever shared.
- Validate type/size, resize appropriately, clean up replacements/deletes, and show save failure honestly.
- The card may be explicitly cut if the PRD's photo deferral condition is accepted and recorded.
- A photo is a presentation/platform asset referenced from the Rust-owned recipe record through the coarse bridge; the image bytes do not cross the bridge per field (invariant 21, PRD §6.3). The picker and file handling may stay in Dart/Kotlin where that ecosystem is better (PRD §13).
- PRD §16 demotes photo support before cutting controller proof, so this card yields to MVP-023 and MVP-024 whenever they contend for the same session.

## Scope

- Image selection, bounded processing, local storage, per-household file scoping, recipe display, failure/retry, and cleanup tests.

## Non-goals

- Starter-content licensing, public share images, image search/generation, editing suite, or iOS picker behavior.

## Decision gates

- Before implementation, confirm Done-versus-explicit-cut and local/emulator media-test strategy.

## Acceptance criteria

- **AC-1:** A valid image can be attached, viewed, replaced, and removed.
- **AC-2:** Invalid/oversize inputs and failed saves preserve recipe integrity.
- **AC-3:** Stored images are scoped to the owning household and unreachable from another household's records; orphan cleanup behavior is tested.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Android integration flow |
| AC-2 | Failure tests |
| AC-3 | Household file-scoping and cleanup tests plus independent privacy review |

## Stop/failure conditions

- Stop before cloud Storage activation, copyrighted asset inclusion, or silent orphaning. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, delivery-gate progress, **Next implementation task**, and either `Done` or an explicit justified cut; never silently omit this card.
