# MVP-027 — Household entitlement model

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Monetization |
| Depends on | MVP-018, DEC-006 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | Creating entitlement documents and rules in the MVP-018 dev project requires explicit approval; never production |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-027`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

The app knows, offline and deterministically, what this household is entitled to — without the core loop ever depending on that answer.

## Authoritative sources

- `docs/task/decision/DEC-006_MONETIZATION_MODEL_DECISION.md` — the free/paid boundary, entitlement scope, `foundingUser`, `complimentary` grants, and the member-departure rule
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §21 — permitted monetization axes; "artificially metered deterministic intelligence" is excluded
- `docs/PRD_v3.md` §20, `:1004` ("without artificial scarcity"), §12 (durable state)
- Historical, subject to `DEC-006` decision 8: `docs/PRD_v2.md` lines 390-395 (household entitlement; billing ownership stored separately from effective entitlement), `:862` (`foundingUser`, `complimentary`)
- `docs/ROADMAP.md` D-028, D-030, D-040; `docs/task/MVP_INVARIANTS.md` 2, 15, 16, 21

## Load-bearing constraints

- **The core loop must never depend on entitlement.** Planning, Cover My Week, restrictions and the shopping list must produce identical results whether entitlement is present, absent, stale or unreadable. This is the architectural expression of §21's ban on metering deterministic intelligence, and it is what makes an entitlement outage a non-event.
- **Entitlement state is where metering would first become representable.** §21 forbids it, so the model must have no field capable of expressing a quota, run-count or throttle against deterministic planning. Absence of the capability is the enforcement.
- Entitlement resolves at the **household**, with billing ownership stored separately from effective entitlement, so a household is not asked to buy per member and losing the purchaser does not silently revoke everyone.
- **Rust holds a cached projection, not the authority.** The household entitlement document lives in the `MVP-018` dev project so it can be shared across members' devices; the kernel caches it for offline reads. The card names that backend surface and states the reconciliation contract — last-writer resolution, offline read-through, and how stale a cached value may be before it is treated as unknown — so `MVP-029` only adds a producer and does not redesign the model.
- **Unknown is not the same as unentitled.** A missing or stale projection means the app does not know, and the honest response is the free experience without an assertion that the household has not paid.
- The lifecycle states must be enumerated **now**. Adding a state after this card is `Done` is a material change to its declared scope and returns it to Draft under D-031, so a card completed without grace-period or account-hold representation would have to be reopened by `MVP-029`.
- Bridge calls stay coarse and service-level (invariant 21); no per-field access and no SQLite handles cross the boundary.

## Scope

- The entitlement document shape in the `MVP-018` dev project, its rules, and the household-scoping that prevents one household reading another's.
- The Rust-side cached projection, its schema migration, and its offline read path.
- **The full lifecycle state set**: active, in grace period, on account hold, paused, upgraded, downgraded, expired, and unknown. Representation only — `MVP-029` produces transitions, `MVP-028` renders them.
- Billing ownership recorded separately from effective entitlement.
- `foundingUser` tagging and persistence, and the `complimentary` household grant, as `DEC-006` defines them.
- The member-departure rule `DEC-006` decides, including the purchaser-deletes case `MVP-026` invokes.
- The reconciliation contract, written down as a contract and pinned by tests.

## Non-goals

- Any store or billing SDK dependency, purchase or restore flows, receipt or server-side validation, RTDN intake (all `MVP-029`); any paywall or gating surface (`MVP-028`); price, packaging or policy (`DEC-006`); production provisioning.

## Decision gates

- The free/paid boundary, entitlement scope, `foundingUser` definition and member-departure rule are `DEC-006`'s. If it is unresolved on a point this card must represent, stop rather than choosing.
- Whether `PRD_v2.md` §16 may be cited as authority is `DEC-006` decision 8.

## Acceptance criteria

- **AC-1:** Entitlement resolves at the household; a second member on a second device sees the same effective entitlement, and billing ownership is readable independently of it.
- **AC-2:** The core loop is bit-identical with entitlement present, absent, stale and unreadable — asserted over the `MVP-025` fixtures, not argued.
- **AC-3:** All eight lifecycle states are representable and round-trip through the bridge; a state added later would be a schema change, and the test set names each one.
- **AC-4:** No field in the model can express a quota, run-count or throttle against deterministic planning; a repository grep plus an inspection record demonstrates the absence.
- **AC-5:** A missing or stale projection reads as **unknown**, and no surface asserts the household has not paid.
- **AC-6:** `foundingUser` and `complimentary` persist as `DEC-006` defines, and survive reopen.
- **AC-7:** The member-departure rule behaves as specified, including the purchaser-deletes case.
- **AC-8:** Adversarial: a second household cannot read or write this household's entitlement; a hand-edited stale projection is detected and treated as unknown rather than trusted.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Two-member, two-device dev-project test; independent read of billing ownership |
| AC-2 | Planner determinism suite re-run under four entitlement conditions, byte-compared |
| AC-3 | Round-trip test naming each of the eight states |
| AC-4 | `grep` record over the model and bridge, plus a written inspection note |
| AC-5 | Unit and widget tests for missing and stale projections |
| AC-6 | Persistence-across-reopen test |
| AC-7 | Departure and purchaser-deletion fixtures |
| AC-8 | Adversarial cross-household rules test; tampered-projection test |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- **Stop if any design would make the core loop read entitlement**, or would introduce a field capable of metering deterministic planning — both are §21 violations, not tradeoffs.
- Stop if `DEC-006` has not settled a point this card must represent.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, decisions, blockers, MONETIZATION-READY progress, and **Next implementation task**. Select a dependent next only when this card is `Done` and that dependent passes its applicable planning or decision workflow gate.
