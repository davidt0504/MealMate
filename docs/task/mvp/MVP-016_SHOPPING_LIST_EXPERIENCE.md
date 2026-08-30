# MVP-016 — Shopping-list experience

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Shopping/UI |
| Depends on | MVP-003, MVP-004, MVP-015 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-016`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Turn the shopping projection into a fast, editable, household-owned list grouped for real store use.

## Authoritative sources

- `docs/PRD_v3.md` §6.3 (coarse bridge), §13 (Rust owns derivation, Flutter owns presentation), §15
- `docs/ROADMAP.md` D-011, D-012, D-023, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 3, 17, 21
- Historical (D-028): `docs/PRD_v2.md` §§7.9, 14.4, 24.7

## Load-bearing constraints

- Generated and manual items remain distinguishable and explainable.
- Users can add, remove, edit, and check off items; purchased items may explicitly flow to pantry.
- The screen consumes bridge DTOs from the MVP-015 derivation and issues bridge commands for edits, check-off, and restoring an omitted line; it holds no durable state (invariants 17, 21).
- Regeneration has explicit merge/overwrite semantics and cannot silently discard edits.

## Scope

- List generation action, persistence, grouped UI, manual edits/checkoff, regeneration behavior, optional purchased-to-pantry action, and tests.

## Non-goals

- Store routing, prices, barcode/receipt scanning, multi-user conflict resolution, or notifications.

## Decision gates

- Resolve regeneration/manual-edit precedence before implementing destructive replacement.
- Resolved 2026-08-29 (plan): live derivation + per-line overlay; merge by default, overwrite only by explicit reset; checks carry the quantity they were made against and are shown as changed rather than kept when it differs; a cycle-settings edit orphans window-keyed line states (accepted limitation, pinned by test); manual items are household-scoped and carry across cycles; line states are retained per window indefinitely with no purge — re-deriving a past window resurrects that window's own checks, which is by design, and the volume (~1,600 rows/year) does not justify a retention policy yet.

## Acceptance criteria

- **AC-1:** A plan produces the expected grouped list and explanations.
- **AC-2:** Manual edits and checked state persist and obey regeneration policy.
- **AC-3:** Purchased-to-pantry is explicit, idempotent, and reversible.
- **AC-4:** Item-level household scoping and accessibility behavior pass independent review.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Rust integration test against the MVP-015 bridge-test seeding helpers (`rust/src/api/shopping.rs` `open_seeded`/`save_recipe`/`plan`), **plus** a widget test asserting group headings and the per-line omission/`separate_reason` explanations |
| AC-2 | Restart/regeneration tests |
| AC-3 | State-transition tests |
| AC-4 | Rust household-scoping tests, semantics check, fresh-context review |

## Stop/failure conditions

- Stop if regeneration loses manual work or state is stored as one mutable household array. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, delivery-gate progress, and **Next implementation task**.
