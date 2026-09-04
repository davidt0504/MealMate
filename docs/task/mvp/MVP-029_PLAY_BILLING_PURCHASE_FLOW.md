# MVP-029 — Play Billing purchase flow

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Monetization |
| Depends on | MVP-027, MVP-028, DEC-006, DEC-008 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | Play Console configuration, a merchant account, and license-test purchase accounts are owner actions requiring separate authorization; never production |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-029`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

A household can buy the paid tier, keep it across reinstalls and devices, and lose it correctly when they stop paying — with entitlement that a modified client cannot forge beyond what `DEC-008` accepted.

## Authoritative sources

- `docs/task/decision/DEC-008_PRODUCTION_ACTIVATION_DECISION.md` — server-side validation and RTDN topology, and the accepted residual risk if client-only validation was chosen
- `docs/task/decision/DEC-006_MONETIZATION_MODEL_DECISION.md` — packaging, price and what a purchase grants
- `docs/task/mvp/MVP-027_HOUSEHOLD_ENTITLEMENT_MODEL.md` — the state set this card produces transitions for
- `docs/PRD_v3.md` §19 (metrics), §20
- Historical, subject to `DEC-006` decision 8: `docs/PRD_v2.md:753` (re-check current Google subscription rules before the public release)
- `docs/ROADMAP.md` D-037, D-040; `docs/task/MVP_INVARIANTS.md` 15, 16, 21

## Load-bearing constraints

- **The core loop must remain independent of billing.** A failed, pending or entirely absent purchase path degrades only the paid surface; planning, restrictions and the shopping list are untouched. This inherits `MVP-018:35`'s optional-adapter rule and `MVP-027`'s core-loop independence.
- **Entitlement is reconciled, not asserted.** This card produces transitions into `MVP-027`'s state machine; it does not own the state, and it may not add a state `MVP-027` does not already represent — that would be a D-031 event on a Done card.
- Whether entitlement resists a modified client is `DEC-008`'s decision, not this card's. If `DEC-008` chose client-only validation, this card implements it and the accepted residual risk is cited, not silently re-litigated.
- **Purchase and entitlement telemetry belongs here.** `MVP-019:51` non-goals billing telemetry and nothing else reclaims it, so without this the paywall ships unmeasurable. D-040 records the supersession. The same privacy rules apply: no sensitive household or recipe content in any event.
- `PRD_v2.md:753` requires re-checking current Google **subscription** policy before the public release. `MVP-022` carries a deferral line; the substantive re-check is `MVP-031`'s, and this card's implementation must be consistent with whatever that finds.
- Money path: acceptance evidence includes the adversarial tier, and `docs/task/README.md:47` forbids `--auto` here.

## Scope

- Play Billing integration: product configuration read, purchase, acknowledgement, and the mandatory acknowledgement deadline.
- Restore on reinstall and on a second device, reconciled against the household entitlement rather than the device.
- Cancel, upgrade, downgrade and proration as `DEC-006`'s packaging requires.
- Server-side purchase validation and Real-Time Developer Notification intake per `DEC-008`, or the documented client-only path if that is what `DEC-008` chose.
- Transitions into every `MVP-027` lifecycle state, including grace period and account hold.
- Purchase and entitlement telemetry, privacy-bounded.
- Evidence gathered with license-test accounts against a non-production configuration.

## Non-goals

- The entitlement model (`MVP-027`), the paywall surface (`MVP-028`), price or packaging policy (`DEC-006`), production provisioning or store submission (`MVP-031`), refund processing (Play is merchant of record), and any non-Play payment path.

## Decision gates

- Server-side versus client-only validation is `DEC-008`'s. If unresolved, stop — the two produce materially different cards and the difference is the whole security posture of the money path.
- Packaging shape drives whether upgrade/downgrade/proration is in scope at all; `DEC-006` decides.

## Acceptance criteria

- **AC-1:** A purchase completes, is acknowledged within the deadline, and moves `MVP-027`'s entitlement to active at the household level — visible to a second member on a second device.
- **AC-2:** Restore after reinstall returns the household to its correct state without a second charge, and without depending on the device it was bought on.
- **AC-3:** Cancellation, expiry, grace period and account hold each drive the corresponding `MVP-027` transition; no state is produced that `MVP-027` cannot represent.
- **AC-4:** Validation behaves as `DEC-008` specifies; if server-side, a purchase token failing validation never grants entitlement, and RTDN events reconcile state without user action.
- **AC-5:** The core loop is unaffected by every billing failure mode — offline, cancelled mid-flow, Play unavailable, validation failing.
- **AC-6:** Telemetry records purchase and entitlement outcomes with no sensitive household or recipe content, and the event contract is versioned as `MVP-019` requires.
- **AC-7:** Adversarial: an interrupted purchase leaves no phantom entitlement; a refunded purchase revokes; a replayed or tampered purchase token is rejected under the `DEC-008` posture; entitlement and receipt disagreeing resolves to the documented rule rather than to whichever was read last.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | License-test purchase against the non-production configuration; second-device read |
| AC-2 | Reinstall and second-device restore, recorded |
| AC-3 | State-transition tests over the full `MVP-027` set, including a test that an unrepresentable state is a compile or contract failure |
| AC-4 | Validation tests per the `DEC-008` posture; RTDN fixture replay if adopted |
| AC-5 | Core-loop suite re-run under each billing failure mode |
| AC-6 | Event-contract test plus a redaction inspection |
| AC-7 | Adversarial suite: interrupted, refunded, replayed, tampered, and disagreement fixtures |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- Stop if `DEC-008` has not settled the validation posture.
- Stop if a purchase path would make the core loop depend on billing, or would require a new entitlement state — the first is an architecture violation, the second reopens a Done card.
- Stop before any production configuration, real charge, or store submission.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, decisions, blockers, MONETIZATION-READY progress, and **Next implementation task**. Select a dependent next only when this card is `Done` and that dependent passes its applicable planning or decision workflow gate.
