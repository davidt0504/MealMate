# MVP-008 — Manual recipe CRUD

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Recipes |
| Depends on | MVP-003, MVP-007 |
| Complexity | Complex |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **v3 amendment (2026-08-24, D-028/D-029):** Recipe reads/writes go through coarse bridge commands (e.g. `save_recipe`), never per-field calls (invariant 21). Authoritative source adds `docs/PRD_v3.md` §6.3, §16; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Let a household create, browse, inspect, edit, and delete its own recipes without relying on import or starter content.

## Authoritative sources

- `docs/PRD_v2.md` §§7.5–7.6, 24.3; `docs/ROADMAP.md` D-016; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Preserve entered ingredient text and expose validation without silent normalization.
- Destructive deletion requires clear confirmation and predictable dependent-data behavior.
- Empty/loading/error states are honest and recoverable; controls are accessible.

## Scope

- Recipe list/detail/form, structured ingredient rows, validation, CRUD state, deletion behavior, and tests.

## Non-goals

- URL import, photos, public sharing, advanced search, meal planning, or automated parsing promises.

## Decision gates

- Resolve delete/reference behavior before allowing a referenced recipe to be removed.

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

Record evidence/status in `docs/ROADMAP.md`.
