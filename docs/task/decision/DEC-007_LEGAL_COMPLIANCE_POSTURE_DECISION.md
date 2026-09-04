# DEC-007 — Legal, privacy and compliance posture

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Decision |
| Workstream | Legal/compliance |
| Depends on | DEC-003 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No implementation until Done |
| Recommended workflow | `deep-options`; then `grill-me` for owner risk and cost choices |
| External actions | Research is read-only; retaining counsel, registering a DMCA agent, or buying hosting requires separate owner authorization |

## Workflow gate

Before resolving `DEC-007`, read `docs/ROADMAP.md`, confirm every declared dependency is Done with evidence, and confirm the roadmap identifies this card as the current required decision or explicit owner-paced work. This decision does not need to be the Next implementation task and does not occupy the implementation lane. Its result and Done transition require explicit owner approval.

## Outcome and user value

Decide the legal and compliance posture a public launch requires — which documents exist, who authors them, which jurisdictions are in scope, and what user-facing rights must be reachable — so that `MVP-026` and `MVP-030` build to a settled standard rather than guessing.

## Authoritative sources

- `docs/PRD_v3.md` §14 (security and privacy), §16 "Not MVP"
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §20 (deletion and export as privacy-architecture principles)
- `docs/ROADMAP.md` D-037 (DMCA designated-agent registration assigned to `MVP-022`; the owner is the abuse contact; takedown at beta scale is console deletion), D-040
- `docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md` — `:38` prepares Play health/privacy/data-safety declarations, deletion/export **posture**, a privacy policy and support surfaces; AC-2 puts them through independent review. This card decides the posture that card prepares against.
- `docs/task/decision/DEC-003_CLOUD_SHARING_RELEASE_DECISION.md` — projection-copy sharing, archive-cascades-to-revoke, the provenance-keyed rights rule, and the report path to an owner-controlled address
- `docs/task/MVP_INVARIANTS.md` 11, 12, 15

## Locked constraints

- The repo currently has **zero** occurrences of "terms of service", GDPR, CCPA or COPPA. Nothing may be assumed to already exist; each is a decision this card makes explicitly, including the decision not to address one.
- Google Play requires a user-facing account-deletion path and a **web-reachable** deletion request URL. `MVP-022:38` covers only the posture; `MVP-026` builds the in-app path and `MVP-030` hosts the web one. This card decides what deletion must actually do.
- Deletion is destructive and irreversible for the household's local data — `allowBackup=false` and there is no cloud backup, so an export path must exist before deletion is offered.
- Sharing is projection-copy, not a read of household documents (`DEC-003`), so a takedown deletes a projection and does not reach the source recipe. Any published policy must describe that truthfully.
- Nothing here authorizes retaining counsel, paying a registration fee, or purchasing hosting.

## Decisions to resolve

1. **Terms of Service — authored how?** Template, generator, or counsel. The repo has no ToS and no card writes one until `MVP-030`.
2. **Privacy policy — authored how, and hosted where?** `MVP-022:38` prepares one; nothing hosts it. Play requires a reachable URL before submission, and `DEC-003` routes the custom domain to `DEC-004`.
3. **Jurisdictions in scope.** Whether GDPR, CCPA/CPRA or COPPA obligations are accepted at launch, or whether distribution is restricted to avoid them. This drives the Play data-safety answers and the age-rating questionnaire in `MVP-031`.
4. **DMCA designated agent.** D-037 records this as an `MVP-022` criterion. Decide whether to register, who is named, and what the takedown workflow is beyond `DEC-003`'s console deletion.
5. **Who signs off the Play data-safety declaration**, and against what evidence — a false declaration is a store-policy violation, and `MVP-019`'s event taxonomy is what determines whether the answers are true.
6. **What deletion means.** Whether it removes the member only or the whole household when they are the last member; what happens to shared recipes other members rely on; whether published projections are revoked synchronously or eventually (`DEC-003` chose queued offline revocation with a pending indicator); and what the retention window is, if any.
7. **Whether account deletion must be reachable from the web** as well as in-app, and at what URL — Play's requirement, but the shape is a choice.
8. **Refund and cancellation posture.** Play is merchant of record for Play-billed subscriptions and its refund rules govern, but the published policy must say something. Decide what.

## Options and tradeoffs

- **Template ToS/privacy policy** — free and immediate, and adequate for a small beta. Risk: templates routinely misdescribe the architecture, and a policy that claims cloud storage this app does not use, or omits the projection-copy sharing model, is worse than none.
- **Counsel-drafted** — accurate and defensible, and the only option that survives a real complaint. Cost is real money and calendar time, and `DEC-004`'s stop condition already refuses a paid legal opinion for naming.
- **Restrict distribution to avoid GDPR/CCPA** — Play allows country targeting, and a US-only launch materially narrows obligations. Risk: it forecloses the EU without a decision to reopen it, and `MVP-031`'s listing then encodes that choice.
- **Accept GDPR at launch** — future-proof and aligned with the privacy-architecture principles the product already follows. Cost: data-subject request handling, a lawful-basis statement, and a retention policy the deletion flow must honour.
- **Household deletion versus member deletion** — deleting the last member's household is simple and complete; deleting a member from a shared household leaves recipes whose author is gone. The second is more correct and materially harder, and `MVP-026` must implement whichever is chosen.

## Completion criteria

- Decisions 1–8 resolved with rationale, or explicitly deferred with the consequence for `MVP-026`, `MVP-030` and `MVP-031` named.
- The deletion semantics are stated precisely enough that `MVP-026` can write acceptance criteria without further interpretation, including the shared-data and published-projection cases.
- Jurisdiction scope recorded, with its consequence for the Play data-safety answers and any country targeting in `MVP-031`.
- The DMCA position recorded, and `MVP-022`'s criterion either discharged or explicitly rerouted.
- Hosting location for both documents decided, or the dependency on `DEC-004`'s domain stated explicitly.

## Stop/failure conditions

- Stop if resolution requires retaining counsel, paying a registration fee, or purchasing a domain or hosting — all are owner actions outside this card.
- Stop if a proposed policy would describe the architecture inaccurately; an incorrect privacy policy is a policy violation, not a documentation defect.
- Stop if a deletion semantic would leave another member's data unusable without their action.
- After two failed remediation cycles, return to planning rather than weakening the criteria.

## Resolution

_Unresolved._
