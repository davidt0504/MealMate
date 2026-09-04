# MVP-030 — Legal documents published and linked

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Legal/compliance |
| Depends on | DEC-007, MVP-018 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | Hosting the documents and the deletion-request endpoint is an owner action requiring separate authorization; never production without it |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-030`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

The terms and the privacy policy exist, describe this app accurately, and are reachable from inside the app and from the open web — including the deletion request path Play requires.

## Authoritative sources

- `docs/task/decision/DEC-007_LEGAL_COMPLIANCE_POSTURE_DECISION.md` — authorship route, hosting, jurisdictions, DMCA position, deletion semantics, refund posture
- `docs/task/decision/DEC-003_CLOUD_SHARING_RELEASE_DECISION.md` — projection-copy sharing, the report path to an owner-controlled address, the `<project>.web.app` dev evidence host
- `docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md` — the domain the published URLs sit under
- `docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md:38` — prepares the privacy policy and support surfaces; this card publishes and wires them
- `docs/PRD_v3.md` §14; `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §20; `docs/task/MVP_INVARIANTS.md` 15, 19

## Load-bearing constraints

- **The documents must describe this architecture, not a generic one.** The app is local-first with an optional cloud adapter, sharing is a projection copy rather than a read of household documents, and there is no ad network and no third-party analytics beyond what `MVP-019` configures. A template that claims otherwise is a false statement in a published policy, which is worse than having none.
- **Play requires a web-reachable account-deletion request URL** in addition to the in-app path `MVP-026` builds. This card owns the hosted endpoint because it owns hosting; the two must describe the same semantics.
- Whatever `DEC-007` decided about deletion semantics, retention and jurisdictions is what gets published — this card does not re-decide, and a discrepancy between the policy text and `MVP-026`'s behaviour is a defect in this card.
- Documents are **versioned**, and the version a user accepted is recorded where `DEC-007` requires, so a later revision does not silently rewrite what they agreed to.
- The in-app links must work offline-degraded: a policy link that dead-ends with no explanation when the device is offline is a failure state, not an acceptable default.
- Hosting is an owner action. This card prepares, wires and verifies against a non-production host; publishing to the real domain is separately authorized.

## Scope

- Terms of Service and privacy policy authored per `DEC-007`'s route, describing the actual architecture.
- Versioning scheme and the acceptance record `DEC-007` specifies.
- Hosting configuration and the published URLs, evidenced on a non-production host.
- The web-reachable account-deletion request endpoint, consistent with `MVP-026`'s in-app semantics.
- In-app links from Settings and from the paywall's pre-purchase disclosure, with offline behaviour.
- The store-listing URLs `MVP-031` will need, handed over rather than entered.

## Non-goals

- The in-app deletion mechanism (`MVP-026`), deciding any policy content (`DEC-007`), production DNS or domain purchase, store listing entry (`MVP-031`), cookie or web analytics on the hosted pages, and localisation of the documents.

## Decision gates

- Authorship route, hosting location, jurisdictions and refund posture are all `DEC-007`'s. If unresolved, stop — publishing a policy is not a place to improvise.
- The domain is `DEC-004`'s; if it is not final, this card evidences against the dev host and records the production URL as pending.

## Acceptance criteria

- **AC-1:** Both documents exist, are versioned, and describe the architecture accurately — local-first, optional cloud adapter, projection-copy sharing, no ads — verified line by line against the implementation by an independent reader.
- **AC-2:** Both are reachable at stable URLs on the evidence host, and the URLs are recorded for `MVP-031`.
- **AC-3:** The web deletion-request endpoint is reachable without an account and describes the same semantics `MVP-026` implements.
- **AC-4:** In-app links from Settings and from the pre-purchase disclosure open the correct documents, and degrade honestly when offline.
- **AC-5:** The accepted version is recorded as `DEC-007` requires, and a revision does not retroactively alter the recorded acceptance.
- **AC-6:** No claim in either document contradicts `MVP-019`'s event taxonomy or `DEC-003`'s sharing model; cross-checked and recorded.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Independent review record, claim by claim against the implementation |
| AC-2 | Fetched URLs with response codes recorded |
| AC-3 | Unauthenticated fetch of the deletion endpoint; semantics diffed against `MVP-026` |
| AC-4 | Widget tests for both entry points plus an offline case |
| AC-5 | Acceptance-record test across a version bump |
| AC-6 | Cross-check table: policy claim → implementing card and evidence |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- **Stop if any claim in either document cannot be verified against the implementation** — publishing an inaccurate policy is a policy violation, not a wording problem.
- Stop if `DEC-007` has not settled the content being published, or if publishing would require production DNS before it is authorized.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, decisions, blockers, PUBLIC-LAUNCH-READY progress, and **Next implementation task**. Select a dependent next only when this card is `Done` and that dependent passes its applicable planning or decision workflow gate.
