# MVP-026 — Account deletion and data export

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Privacy and data rights |
| Depends on | MVP-018, MVP-020, DEC-006, DEC-007 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | Deleting records in the MVP-018 dev project and revoking MVP-020 projections requires explicit approval; never production |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-026`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

A household member can export everything the app holds about them and then delete it, and the deletion is honest about what it did and did not reach.

## Authoritative sources

- `docs/PRD_v3.md` §14 (security and privacy), §6.5 (cloud is an optional adapter)
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §20 (deletion and export as privacy-architecture principles)
- `docs/task/decision/DEC-007_LEGAL_COMPLIANCE_POSTURE_DECISION.md` — decides what deletion means, the shared-data case, and the retention window
- `docs/task/decision/DEC-006_MONETIZATION_MODEL_DECISION.md` — decides what happens to household entitlement when the purchasing member deletes their account
- `docs/task/decision/DEC-003_CLOUD_SHARING_RELEASE_DECISION.md` — archive-cascades-to-revoke with queued offline revocation and a pending indicator
- `docs/ROADMAP.md` D-037, D-040; `docs/task/MVP_INVARIANTS.md` 11, 12, 15

## Load-bearing constraints

- Google Play requires a user-facing account-deletion path. `MVP-022:38` covers deletion/export **posture** and AC-2 reviews it; no card builds the mechanism. This one does. The **web-reachable** deletion request URL Play also requires belongs to `MVP-030`, which owns hosting.
- **Export must exist and must be offered before deletion.** `allowBackup=false` and there is no cloud backup, so deletion is the only irreversible action in the product and the user must be able to take their data out first.
- Deletion must never claim to have reached something it has not. `DEC-003` made published-projection revocation eventually consistent with a pending indicator precisely because the cascade runs client-side and must work offline; the deletion UI inherits that honesty requirement rather than asserting a projection is already gone.
- Shared household data is not the deleting member's alone. Whatever `DEC-007` decides about the member-versus-household case, the implementation may not leave another member's recipes or plans unusable without their own action.
- The local loop must survive deletion of the optional cloud identity — `MVP-018:35` makes the cloud adapter optional by construction and `MVP-018` AC-6 already forbids silent household-data loss on identity transition.
- Deletion is destructive and irreversible: `docs/task/README.md:47` forbids `--auto` for destructive and data-loss work, and this card's `External actions` row reflects that its evidence needs dev-project records and projection revocation.

## Scope

- In-app export of the household's durable data in a documented, re-readable format; export offered as a precondition of the deletion flow.
- In-app account deletion with an unambiguous confirmation that states what will be removed, what will remain, and what is queued rather than done.
- The shared-household case `DEC-007` decides — member-only versus whole-household — including what happens to recipes authored by the departing member.
- Cascade to published projections through `MVP-020`'s revoke path, with the pending indicator when offline.
- The entitlement consequence `DEC-006` decides when the purchasing member deletes.
- Offline behaviour, and re-entry after a failed or interrupted deletion.
- Dev-project evidence that records are actually gone, and adversarial evidence that nothing survives that the UI claimed was removed.

## Non-goals

- The web-reachable deletion request URL (`MVP-030`), hosting anything, production deletion, bulk or admin deletion tooling, data-subject request workflow beyond the in-app path, and undo or a restore-from-deletion path.

## Decision gates

- Deletion semantics, the retention window, and the shared-data case are `DEC-007`'s to settle; this card does not choose them. If `DEC-007` is unresolved when planning starts, stop rather than inventing a semantic.
- The entitlement consequence is `DEC-006`'s. Same rule.

## Acceptance criteria

- **AC-1:** Export produces a complete, re-readable copy of the household's durable data, and the deletion flow cannot be completed without the user having been offered it.
- **AC-2:** Deletion removes exactly what `DEC-007` specifies, verified against the dev project, with no orphaned records under the deleted identity.
- **AC-3:** The confirmation surface states what is removed, what remains, and what is queued; no wording asserts a published projection is already gone when revocation is still pending.
- **AC-4:** The shared-household case behaves as `DEC-007` specifies, and no other member's data becomes unusable as a result.
- **AC-5:** Household entitlement behaves as `DEC-006` specifies when the purchasing member deletes.
- **AC-6:** Deletion attempted offline queues honestly and completes on reconnect; an interrupted deletion leaves no half-deleted state that the app cannot describe.
- **AC-7:** Adversarial: a second read after deletion — through the bridge and directly against the dev project — finds nothing the UI reported as removed.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Round-trip test: export, delete, re-import or inspect the export; widget test that the flow gates on the offer |
| AC-2 | Dev-project query before and after, recorded per collection |
| AC-3 | Widget test asserting the exact copy for each of the three states, plus a screenshot |
| AC-4 | Two-member fixture; assertions on the surviving member's data |
| AC-5 | Entitlement state asserted after the purchaser's deletion |
| AC-6 | Offline test with reconnect; interrupted-deletion test asserting a describable state |
| AC-7 | Adversarial read-back through both paths after deletion |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- Stop if `DEC-006` or `DEC-007` is unresolved on the point being implemented.
- Stop if the design would delete another member's data without their action, or would claim completion for a queued revocation.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, decisions, blockers, MONETIZATION-READY progress, and **Next implementation task**. Select a dependent next only when this card is `Done` and that dependent passes its applicable planning or decision workflow gate.
