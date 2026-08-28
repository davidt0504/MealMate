# MVP-008 — Manual recipe CRUD

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Recipes |
| Depends on | MVP-003, MVP-007 |
| Complexity | Complex |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-008`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Let a household create, browse, inspect, edit, and delete its own recipes without relying on import or starter content.

## Authoritative sources

- `docs/PRD_v3.md` §6.3 (coarse bridge rules), §13 (Rust/Flutter ownership boundary), §16 (required UI)
- `docs/ROADMAP.md` D-016, D-023, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 2, 17, 21
- Historical (D-028): `docs/PRD_v2.md` §§7.5–7.6, 24.3

## Load-bearing constraints

- Preserve entered ingredient text and expose validation without silent normalization.
- Destructive deletion requires clear confirmation and predictable dependent-data behavior.
- Empty/loading/error states are honest and recoverable; controls are accessible.
- Recipe reads and writes cross the bridge as service-level commands over explicit DTOs — one command per whole recipe, never a call per field, and no SQLite handle crosses the boundary (invariant 21, PRD §6.3).
- Ephemeral form state before submission stays in Dart; the saved recipe is Rust-owned (PRD §13, invariant 17).

## Scope

- Recipe list/detail/form, structured ingredient rows, validation, CRUD through coarse bridge commands, deletion behavior, and tests.

## Non-goals

- URL import, photos, public sharing, advanced search, meal planning, or automated parsing promises.

## Decision gates

- Resolve delete/reference behavior before allowing a referenced recipe to be removed. **Resolved (owner, 2026-08-28): archive, never hard-delete.** "Delete" sets an `archived_at` marker; the recipe leaves the library, pickers and planner candidates (`MVP-023`); every past and already-planned future occurrence keeps its reference and renders an "archived" marker; restore is available. Reason: PRD §12 (stable immutable IDs, append-oriented audit records) and §7.7 (reason codes name recipe IDs) require references to stay resolvable; blocking deletion would make every cooked recipe permanently undeletable; cascading would destroy history. `MVP-011` reuses the same marker to hide starter recipes; `MVP-012` records the occurrence side.

## Acceptance criteria

- **AC-1:** A user can create and later view/edit a multi-ingredient recipe without data loss.
- **AC-2:** Invalid inputs produce actionable errors and do not corrupt persisted state.
- **AC-3:** Delete confirmation and dependent-reference behavior match the resolved policy.
- **AC-4:** Recipe states and primary flows are keyboard/screen-reader/touch usable.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Widget + emulator persistence flow |
| AC-2 | Validation/unit/integration tests |
| AC-3 | Reference/deletion tests and manual confirmation check |
| AC-4 | Accessibility inspection/test record |

## Stop/failure conditions

- Stop if UI state bypasses repositories or delete semantics are ambiguous. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, and **Next implementation task**.
