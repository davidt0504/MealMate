# MVP-022 — Android beta readiness

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Verification/readiness |
| Workstream | Release |
| Depends on | DEC-004, MVP-001–MVP-009, MVP-011–MVP-021, and MVP-010 unless explicitly cut; all delivery gates |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | Final MVP card |
| Recommended workflow | `plan-task` |
| External actions | Readiness only; production project, paid services, signing, Play submission, DNS, commit/push require separate approval |

## Outcome and user value

Produce evidence that Meal Mate is safe and reliable enough for an Android beta decision without silently performing the launch.

## Authoritative sources

- `docs/PRD_v2.md` §§19–21, 24; `docs/ROADMAP.md` gates/decisions; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

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

## Acceptance criteria

- **AC-1:** PRD §24 items 1–11 and every invariant have traceable PASS evidence or an explicitly approved permitted cut.
- **AC-2:** Security/rules/privacy/abuse/report/deletion/export and public-sharing threat controls pass independent review.
- **AC-3:** Android clean-install core loop, offline/reconnect, durable auth, links, accessibility, and supported-device performance pass.
- **AC-4:** Monitoring, rollback, budget/alert, incident owner, Play declarations, privacy/support, domain/signing/store prerequisites are ready or clearly identify an external blocker.
- **AC-5:** A dated go/no-go report distinguishes readiness from actions still requiring authorization.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Automated traceability validator plus evidence audit |
| AC-2 | Fresh-context security/privacy review and rules suite |
| AC-3 | Android emulator/device release-candidate matrix |
| AC-4 | Operational/store checklist with owners and artifacts |
| AC-5 | Signed-off go/no-go document; no deployment action |

## Stop/failure conditions

- Stop on missing evidence, critical vulnerability, silent data loss, unsupported policy claim, unclear incident ownership, or request to activate externally without approval. Two remediation cycles then re-plan the failing boundary.

## Handoff

Record the PRODUCTION-BETA-READY result, evidence links, accepted risks, blockers, and the separately authorized next action in `docs/ROADMAP.md`.
