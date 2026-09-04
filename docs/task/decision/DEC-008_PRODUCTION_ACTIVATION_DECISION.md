# DEC-008 — Production activation, key custody and release channel

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Decision |
| Workstream | Release |
| Depends on | DEC-003, DEC-004 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No implementation until Done |
| Recommended workflow | `deep-options`; then `grill-me` for owner risk and cost choices |
| External actions | Research is read-only; enrolling a developer account, enabling a billing plan, generating a signing key, or reserving a package identifier requires separate owner authorization |

## Workflow gate

Before resolving `DEC-008`, read `docs/ROADMAP.md`, confirm every declared dependency is Done with evidence, and confirm the roadmap identifies this card as the current required decision or explicit owner-paced work. This decision does not need to be the Next implementation task and does not occupy the implementation lane. Its result and Done transition require explicit owner approval.

## Outcome and user value

Decide how the product reaches production without any of it happening by accident — what infrastructure exists, who holds the signing key, which release channel is used, and what must be in place before the first irreversible upload.

## Authoritative sources

- `docs/task/decision/DEC-003_CLOUD_SHARING_RELEASE_DECISION.md` and `docs/ROADMAP.md` D-037 — the dev/production topology, the name-free dev project ID, the no-billing-account rule, App Check debug-only with enforcement deferred, and `:76`'s irreversibility warning
- `docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md` — the production `applicationId`, and `:59`'s requirement to re-check its availability immediately before the first upload that burns it
- `docs/ROADMAP.md` D-019, D-030, D-040
- `docs/task/MVP_INVARIANTS.md` 15 ("No task implicitly activates production, paid services, credentials, DNS, signing, or store submission")
- `docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md` `:39` — readiness is separated from production activation, release signing, DNS and store submission

## Locked constraints

- **What this card re-opens from `DEC-003`, and only this:** the production-side Cloud Functions / Blaze / Real-Time Developer Notifications topology needed for server-side subscription validation. D-037's exclusions were scoped to the **dev** project ("no Cloud Storage, no Cloud Functions", no payment method) and that dev boundary stands unchanged. Every other `DEC-003` clause — projection-copy sharing, the 64 KB rules clause, archive-cascades-to-revoke, the provenance-keyed rights rule, the name-free dev project ID — stands. Stating this precisely matters: under D-031 a material change to a Done decision would return `MVP-018`, `MVP-020` and `MVP-021` to Draft.
- **An Android `applicationId` cannot change after publication.** `DEC-003:76`: a post-launch challenge becomes "a **relaunch rather than a rename**". Every irreversible commitment stays gated on a clean `DEC-004` result.
- The custom domain remains `DEC-004`'s decision (`DEC-003:99,130`); this card does not claim it.
- Invariant 15 means nothing here activates anything. This card decides; `MVP-031` prepares; the owner acts.
- App Check enforcement is deferred to `MVP-022` by D-037. This card does not move it.

## Decisions to resolve

1. **Production backend topology.** Whether production runs Cloud Functions on Blaze for server-side purchase validation and RTDN intake, and if so what the cost ceiling and alerting are. Without it, `MVP-029` can only verify purchases client-side — which on a money path means entitlement can be forged by a modified client.
2. **RTDN handling.** Whether renewal, cancellation, refund, grace-period and account-hold notifications are consumed, and by what, given `MVP-027` holds the state machine those events drive.
3. **Signing key custody.** Play App Signing (Google holds the key, upload key is replaceable) versus self-managed (full control, unrecoverable if lost). The current build is debug-signed with a per-machine keystore, which is not a launch option either way.
4. **Developer account type.** Personal versus organisation. This is not cosmetic: personal accounts created recently are subject to a **closed-testing requirement** — a minimum tester count sustained for a minimum period before production access is granted — which is a multi-week calendar blocker, not an engineering task.
5. **Release-track ladder.** Internal → closed → open → production, and which rungs are actually used. Interacts with decision 4: if the tester requirement applies, closed testing is mandatory and its duration must be in the launch schedule.
6. **Rollback strategy.** Staged rollout percentage, halt criteria, and what rollback means when an `applicationId` cannot change and a bad build has already been installed. Play does not support un-publishing a version users already have.
7. **Production project provisioning order** — when the production Firebase project is created relative to the `DEC-004` name being final, given project IDs are permanent and globally unique.

## Options and tradeoffs

- **Play App Signing** — the upload key can be reset if lost, which removes the single worst failure mode for a solo developer. Cost: Google holds the app signing key, which some find unacceptable. Given the current key is a per-machine debug keystore whose loss would already strand every installed copy, the recoverability argument is strong here.
- **Self-managed signing** — full custody, no dependency. Losing the keystore means the app can never be updated by anyone, ever; the only remedy is a new listing under a new `applicationId`, which `DEC-003:76` describes as a relaunch.
- **Server-side validation on Blaze** — the only way entitlement resists a modified client, and the only way RTDN can be consumed. Costs a billing account on production, which D-037 deliberately avoided on dev, plus a cost ceiling and alerting.
- **Client-only validation** — no infrastructure, no cost, ships fastest. On a money path it means the entitlement check is advisory: a modified client grants itself the paid tier. Whether that matters is a business judgement about expected revenue versus expected abuse, and it should be made explicitly rather than by default.
- **Organisation account** — may avoid the closed-testing requirement and looks more credible on a listing, but requires a verifiable legal entity and D-U-N-S registration, which is its own multi-week path.

## Completion criteria

- Decisions 1–7 resolved with rationale, or explicitly deferred with the consequence for `MVP-029` and `MVP-031` named.
- The `DEC-003` re-opening is stated as a precise clause list, with an explicit statement that all other clauses stand — so no reader concludes `MVP-018`/`MVP-020`/`MVP-021` returned to Draft.
- If client-only validation is chosen, the residual risk is recorded as an accepted risk with its abuse model, not left implicit.
- The closed-testing requirement is either confirmed applicable with its duration entered into the launch schedule, or confirmed inapplicable with the reason.
- Key custody decided, with the recovery story for the losing case written down.
- A production cost ceiling and alert threshold recorded if Blaze is adopted.

## Stop/failure conditions

- Stop if resolution requires enrolling a developer account, enabling billing, generating a production key, or reserving a package identifier — all are owner actions.
- Stop if a decision would re-open a `DEC-003` clause beyond the production Functions/Blaze/RTDN topology named above; that is a D-031 event affecting three Done cards and needs its own decision.
- Stop if production provisioning is proposed before `DEC-004` is Done — project IDs and package identifiers are permanent.
- After two failed remediation cycles, return to planning rather than weakening the criteria.

## Resolution

_Unresolved._
