# MVP-031 — Production activation and submission readiness

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Verification/readiness |
| Workstream | Release |
| Depends on | MVP-022, MVP-026, MVP-027, MVP-028, MVP-029, MVP-030, DEC-004, DEC-006, DEC-007, DEC-008 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | Final launch card |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | Readiness only; production provisioning, release signing, store listing entry, submission, and the launch itself require separate owner authorization |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-031`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Produce everything a public paid launch requires, assembled and verified, so the owner can perform the launch as one deliberate authorized act rather than a sequence of discoveries.

## Authoritative sources

- `docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md` — the **beta** go/no-go and its readiness packet; this card is the **launch** equivalent and inherits its acceptance shape (`:61`)
- `docs/task/decision/DEC-008_PRODUCTION_ACTIVATION_DECISION.md` — topology, key custody, release-track ladder, closed-testing requirement, rollback
- `docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md` — `:59` requires re-checking production `applicationId` availability immediately before the first upload that burns it
- `docs/task/decision/DEC-007_LEGAL_COMPLIANCE_POSTURE_DECISION.md` and `docs/task/mvp/MVP-030_LEGAL_DOCUMENTS_PUBLISHED.md` — the URLs and declarations this listing needs
- Historical, subject to `DEC-006` decision 8: `docs/PRD_v2.md:753` — re-check current Google authentication, UGC/sharing, **subscription** and health-related rules before the Android public release
- `docs/ROADMAP.md` D-019, D-037, D-040; `docs/task/MVP_INVARIANTS.md` 14, 15

## Load-bearing constraints

- **This card never performs the launch.** Invariant 15 forbids a task activating production, paid services, credentials, DNS, signing or store submission. Following `MVP-022:61`, acceptance is a readiness packet plus a record of owner-performed actions; **"submitted" is never an acceptance criterion**.
- **`DEC-004`'s pre-upload re-check is carried here.** Production `applicationId` availability must be re-verified immediately before the first upload that burns it, because the identifier is permanent and `DEC-003:76` makes a later change "a relaunch rather than a rename".
- **No `.temp` identifier reaches submission** (`MVP-022` AC-4). The current build ships `dev.mealmate.temp`; `MVP-018` owns the rename execution and this card verifies it landed.
- **`PRD_v2.md:753`'s subscription-policy re-check is this card's**, not `MVP-022`'s: `:753` says "before the Android **public** release", `MVP-022` is beta readiness that runs before the billing cards exist, and there would be nothing to check there. `MVP-022` carries only a deferral line.
- **Play "App access" reviewer instructions are mandatory** because functionality sits behind a paywall — a direct consequence of shipping the paywall live at launch (D-040). A reviewer who cannot reach the paid experience rejects the submission.
- The closed-testing requirement `DEC-008` establishes is a **calendar** dependency, not an engineering one, and belongs in the readiness packet as a schedule item with a date.
- Rollback is constrained by the platform: a version already installed cannot be recalled. The packet states what rollback actually means here rather than assuming it means revert.

## Scope

- Production provisioning readiness per `DEC-008`, including the Blaze cost ceiling and alerting if adopted.
- Release-signing readiness per `DEC-008`'s custody decision, with the recovery story recorded.
- Store listing assets and copy; the **IARC age/content rating and target-audience declaration**; the Play data-safety form answers, sourced from `MVP-019`'s taxonomy and `DEC-007`'s posture.
- Play "App access" reviewer instructions covering the paywalled paths, with test credentials handled as an owner action.
- The `PRD_v2.md:753` re-check of current Google authentication, UGC/sharing, subscription and health-related policy.
- The `DEC-004` pre-upload `applicationId` availability re-check.
- Rollback, staged-rollout and halt criteria; incident owner; monitoring thresholds for the paid path.
- A dated **launch** go/no-go recording the readiness verdict, the accepted risks, and every action still requiring authorization.

## Non-goals

- Performing the launch, production provisioning, signing, listing entry or submission; marketing; iOS; **store-listing localisation** — a single-locale launch is the deliberate choice, recorded rather than left silent; and re-doing `MVP-022`'s beta readiness, which stands on its own evidence.

## Decision gates

- Every substantive choice here belongs to `DEC-004`, `DEC-006`, `DEC-007` or `DEC-008`. If any is unresolved on a point the packet must state, this card records it as an external blocker rather than choosing.
- Whether the measured launch criteria pass is the owner's verdict, as `MVP-022` AC-6 established for the beta.

## Acceptance criteria

- **AC-1:** A readiness packet exists covering provisioning, signing custody and recovery, listing assets, IARC rating, data-safety answers, App access instructions, and the legal URLs from `MVP-030` — each item ready or naming an explicit external blocker with an owner.
- **AC-2:** The `PRD_v2.md:753` policy re-check is recorded with its date and findings, covering authentication, UGC/sharing, subscription and health rules, and any finding that changes an earlier card is raised rather than absorbed.
- **AC-3:** Production `applicationId` availability re-checked and recorded, and no `.temp` identifier appears anywhere in the submission artefacts.
- **AC-4:** Rollback, staged-rollout and halt criteria are written, including what rollback cannot undo, with a named incident owner and monitoring thresholds for the paid path.
- **AC-5:** The closed-testing requirement is recorded as satisfied, in progress with a date, or inapplicable with the reason.
- **AC-6:** A dated launch go/no-go distinguishes readiness from actions still requiring authorization, and records the owner's verdict.
- **AC-7:** Every owner-performed external action is recorded as performed-by-owner with its date; no acceptance criterion is satisfied by this card performing one.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | The packet itself, with an owner and artefact per line |
| AC-2 | Dated re-check record with sources and findings |
| AC-3 | Availability check record; grep of submission artefacts for `.temp` |
| AC-4 | Written rollback and incident plan with thresholds |
| AC-5 | Schedule entry with dates, or a recorded inapplicability |
| AC-6 | Signed-off go/no-go document; no deployment action |
| AC-7 | Owner-action log, dated |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- **Stop before performing any production, signing, listing or submission action** — recording that it is ready is this card's whole job.
- Stop if a required launch control is FAIL or NOT VERIFIED; the owner accepts only risks that are genuinely non-blocking, as `MVP-022` established.
- Stop if the `:753` re-check finds a policy change that invalidates an earlier card's design, and raise it rather than working around it.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the PUBLIC-LAUNCH-READY result, evidence links, accepted risks, blockers, resulting status, and **Next implementation task**; any next action requiring separate authorization remains unauthorized until granted.
