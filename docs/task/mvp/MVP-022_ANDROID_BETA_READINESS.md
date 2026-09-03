# MVP-022 — Android beta readiness

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Verification/readiness |
| Workstream | Release |
| Depends on | DEC-004, MVP-001–MVP-009, MVP-011–MVP-021, MVP-023, MVP-024, MVP-025; all delivery gates through SHARING-SECURITY-READY |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | Final MVP card |
| Recommended workflow | `plan-task` |
| External actions | Readiness only; production project, paid services, signing, Play submission, DNS, commit/push require separate approval |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below. `MVP-023`, `MVP-024` and `MVP-025` were already on the `Depends on` line and in the register. Per D-034 this card's security, operational, store-readiness and signing content is **preserved as written** — `DEC-003` and `DEC-004` decide it.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-022`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Produce evidence that Meal Mate is safe and reliable enough for an Android beta decision without silently performing the launch.

## Authoritative sources

- `docs/PRD_v3.md` §26 (MVP acceptance test), §22 ("Cover My Week is required MVP proof"), §16, §14
- `docs/ROADMAP.md` gates/decisions, D-015, D-028, D-030, D-034, and the PRD v3 §26 traceability table; `docs/task/MVP_INVARIANTS.md` 1–21
- Historical (D-028): `docs/PRD_v2.md` §§19–21, 24

## Load-bearing constraints

- PRD v3 §22 makes **Cover My Week the required MVP proof** — a CRUD-only build does not satisfy v3. A go recommendation therefore requires `MVP-024`'s acceptance evidence, not merely a green core loop.
- Reverify the full core loop, offline behavior, rules, auth recovery, public projection, privacy, and accessibility from clean state.
- Complete threat model, sharing-security gate, abuse/report ownership, cost budgets/alerts plan, and monitoring/runbook.
- Prepare current Google Play health/privacy/data-safety declarations, deletion/export posture, privacy policy, and support surfaces.
- Separate evidence/readiness from production activation, release signing, DNS, and store submission.
- iOS is not an MVP criterion.

## Scope

- Dependency/evidence audit, release-candidate build checks, clean-install/upgrade-not-required scenarios, performance/reliability/accessibility/security/privacy reviews, operational and store-readiness checklist, known-risk ledger, and go/no-go recommendation.

## Non-goals

- Actual public/beta release, production provisioning, marketing launch, iOS, or post-MVP features.

## Decision gates

- Any FAIL/NOT VERIFIED in a required launch control blocks a go recommendation; owner explicitly accepts only risks that are genuinely non-blocking.
- **What ratio demonstrates the central claim is not decided by PRD v3.** §22 requires Cover My Week as proof and §19 sets no numeric bar. The go/no-go records the measured ratio and the owner's verdict. A fixed pass bar requires an owner decision and its own D-row. Working reference, not binding: Cover My Week active seconds ≤50% of the manual arm's, on the median of the recorded scenarios.

## Acceptance criteria

- **AC-1:** PRD v3 §26 items **2–17**, plus item 1's **Android half**, and every invariant in `docs/task/MVP_INVARIANTS.md` (1–21), have traceable PASS evidence or an explicitly approved permitted cut, per the §26 traceability table in `docs/ROADMAP.md`. Item 1's iOS half is carried by `PRE-003` post-launch under D-015 and invariant 14 and is **not** an MVP criterion.
- **AC-2:** Security/rules/privacy/abuse/report/deletion/export and public-sharing threat controls pass independent review.
- **AC-3:** Android clean-install core loop, offline/reconnect, durable auth, links, accessibility, and supported-device performance pass.
- **AC-4:** Monitoring, rollback, budget/alert, incident owner, Play declarations, privacy/support, domain/signing/store prerequisites are ready or clearly identify an external blocker, and no temporary `.temp` application identifier reaches the store submission.
- **AC-5:** A dated go/no-go report distinguishes readiness from actions still requiring authorization.
- **AC-6:** The go/no-go report records the median ratio from `MVP-024`'s AC-8 measurement and the owner's verdict on whether it demonstrates the central claim.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Automated traceability validator plus evidence audit |
| AC-2 | Fresh-context security/privacy review and rules suite |
| AC-3 | Android emulator/device release-candidate matrix |
| AC-4 | Operational/store checklist with owners and artifacts |
| AC-5 | Signed-off go/no-go document; no deployment action |
| AC-6 | Verdict section in the dated go/no-go document citing the AC-8 record |

## Stop/failure conditions

- Stop on missing evidence, critical vulnerability, silent data loss, unsupported policy claim, unclear incident ownership, or request to activate externally without approval. Two remediation cycles then re-plan the failing boundary.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the PRODUCTION-BETA-READY result, evidence links, accepted risks, blockers, resulting status, delivery-gate progress, and **Next implementation task**; any next action that requires separate authorization remains unauthorized until granted.
