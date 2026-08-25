# MVP-016 — Shopping-list experience

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Shopping/UI |
| Depends on | MVP-003, MVP-004, MVP-015 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None |

> **v3 amendment (2026-08-24, D-028/D-029):** The list screen consumes bridge DTOs from the Rust derivation (MVP-015); user restores of omitted lines are bridge commands. Authoritative source adds `docs/PRD_v3.md` §6.3, §15; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Turn the shopping projection into a fast, editable, household-owned list grouped for real store use.

## Authoritative sources

- `docs/PRD_v2.md` §§7.9, 14.4, 24.7; `docs/ROADMAP.md` D-011, D-012; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Generated and manual items remain distinguishable and explainable.
- Users can add, remove, edit, and check off items; purchased items may explicitly flow to pantry.
- Store each frequently edited item separately to limit sync-conflict damage.
- Regeneration has explicit merge/overwrite semantics and cannot silently discard edits.

## Scope

- List generation action, persistence, grouped UI, manual edits/checkoff, regeneration behavior, optional purchased-to-pantry action, and tests.

## Non-goals

- Store routing, prices, barcode/receipt scanning, multi-user conflict resolution, or notifications.

## Decision gates

- Resolve regeneration/manual-edit precedence before implementing destructive replacement.

## Acceptance criteria

- **AC-1:** A plan produces the expected grouped list and explanations.
- **AC-2:** Manual edits and checked state persist and obey regeneration policy.
- **AC-3:** Purchased-to-pantry is explicit, idempotent, and reversible.
- **AC-4:** Item-level data/rules and accessibility behavior pass independent review.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Integration test against MVP-015 fixtures |
| AC-2 | Restart/regeneration tests |
| AC-3 | State-transition tests |
| AC-4 | Rules tests, semantics check, fresh-context review |

## Stop/failure conditions

- Stop if regeneration loses manual work or state is stored as one mutable household array. Two cycles then re-plan.

## Handoff

Record evidence/status in `docs/ROADMAP.md`.
