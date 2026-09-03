# DEC-003 — Cloud and sharing release boundary

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Decision |
| Workstream | Cloud/security |
| Depends on | MVP-017 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | After local core loop is Done |
| Recommended workflow | `deep-options`; then `grill-me` for owner risk/cost choices |
| External actions | Research is read-only; creating projects/domains/credentials requires separate approval |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below. Only Authoritative sources and Locked constraints changed: a re-derivation may not resolve a decision, so **Decisions to resolve and Options and tradeoffs are byte-unchanged**. v3 does not re-open the Firebase vendor choice — D-008 stands.

## Workflow gate

Before resolving `DEC-003`, read `docs/ROADMAP.md`, confirm every declared dependency is Done with evidence, and confirm the roadmap identifies this card as the current required decision or explicit owner-paced work. This decision does not need to be the Next implementation task and does not occupy the implementation lane. Its result and Done transition require explicit owner approval.

## Outcome and user value

Define a safe progression from emulators to a real non-production backend and public sharing, without accidentally creating production exposure.

## Authoritative sources

- `docs/PRD_v3.md` §6.5 (cloud is an optional adapter; no generalized sync engine in v3 MVP), §14 (security/privacy), §16 "Not MVP"
- `docs/ROADMAP.md` D-008, D-009, D-012, D-015, D-028, D-030, D-034
- `docs/task/MVP_INVARIANTS.md` 11, 12, 15, 17
- Historical (D-028): `docs/PRD_v2.md` §§7.10–7.11, 14.2–14.8, 20–21

## Locked constraints

- Cloud is an **optional adapter**. Core planning must not depend on it, and no decision resolved here may make the local core loop require a network (PRD §6.5, invariant 17).
- No custom sync engine in the MVP (PRD §16 "Not MVP").
- Emulator, real development, and production environments are separate.
- Public shares are projected records, never direct access to household documents.
- Standard HTTPS + Android App Links; no Firebase Dynamic Links or deferred-install promise.
- Inbound preview is unauthenticated; publication may require durable auth.
- Production, paid services, DNS, signing, and store changes require explicit separate authorization.

## Decisions to resolve

1. Firebase project/environment topology and configuration separation.
2. Public-share projection, revocation, abuse/report ownership, retention, and rate/cost controls.
3. Hosting/domain strategy for development evidence versus production activation.
4. App Check rollout and failure posture.

## Options and tradeoffs

- Single project: cheapest setup, unacceptable environment coupling.
- Dev + production: simple and adequate for MVP if configuration is fail-closed.
- Dev + staging + production: stronger rehearsal, more operational burden.
- Recommended default: dev now, production later; add staging only when release rehearsal demonstrates need.

## Completion criteria

- A threat-informed architecture, environment matrix, cost boundary, and rollback/revocation plan are recorded.
- The decision clearly separates authorized local/non-production work from later external activation.
- MVP-018 and MVP-020 can be planned without inventing security policy.

## Resolution

Resolved 2026-09-03 in an owner `grill-me` interview against the step-40 `deep-options` package. The owner selected every choice below; the step-40 package is adopted with six substantive changes, each recorded with the evidence that forced it. Registered as D-037.

### Decision 1 — Firebase project/environment topology

**Dev + production, production created later.** Staging is not created; it is added only if `MVP-022` release rehearsal demonstrates need. Single-project is rejected outright — it breaks the locked constraint that environments stay separate.

**The dev project ID is name-free** (shape `mm-dev-<random>`), not derived from `DEC-004`'s candidate name. Firebase project IDs are permanent, globally unique, unrenameable, and published as `<project>.web.app`, so a name-bearing dev ID is a permanent public assertion of a mark whose clearance is still `NOT VERIFIED`. This decouples decision 3 from `DEC-004` entirely: project and hosting creation no longer wait on naming. It does **not** unblock `MVP-018` AC-5, which still requires `DEC-004`'s `applicationId` rename before any application identifier binds.

Configuration separation is fail-closed: Android flavors `dev`/`prod` with per-flavor `google-services.json`; production config is never committed before activation; missing config makes the cloud adapter unavailable and leaves the local core loop untouched (PRD §6.5, §14.7, invariant 17). The dev application ID is `DEC-004`'s production ID plus a `.dev` suffix, so only the production ID is ever registered in the production project and a dev build aimed at production fails registration rather than authenticating.

**Naming risk posture (informs `DEC-004`, does not resolve it):** the exposure is trademark, not copyright. The owner runs the interactive USPTO Trademark Search plus EUIPO/WIPO himself — free, and `DEC-004` records the result as authoritative in place of its current `NOT VERIFIED`. Paid clearance is escalated to only on a Class 9/42 hit. Every irreversible commitment — Play package ID, domain purchase, production project, store listing — stays gated on a clean result, because an Android `applicationId` cannot be changed after publication, making a post-launch challenge a relaunch rather than a rename.

### Decision 2 — Projection, revocation, abuse, retention, rate/cost

**Projection-copy.** Publication, which requires durable auth (`MVP-018`), writes a whitelisted record into a public namespace under an opaque 128-bit random ID. Only display fields and rights/attribution data cross; never household, member, restriction, pantry, or free-text content (invariants 11–13). Live rules-filtered reads of household documents are excluded by locked constraint; signed/expiring URLs are deferred as unneeded machinery and remain addable later without a schema break, since republish already issues a new ID.

**Cost boundary — no billing account.** No payment method is attached to the dev project. This is stronger than step-40's "Spark plan" framing, which no longer holds as written: Cloud Storage requires the Blaze plan as of 2026-02-03, and a Spark project has no buckets at all (calls return 402/403). The dev environment therefore provisions **no Cloud Storage and no Cloud Functions**. Nothing in the MVP needs either — `MVP-010` was cut 2026-08-29 so no image surface exists, and `MVP-020`'s allowlist already excludes storage paths. Publication is a client Firestore write validated by a security-rules field allowlist, which is also more testable under the emulator than server code would be. Invariant 15 then holds literally: with no payment method, no task can activate a paid service even by mistake.

This overrules two clauses in downstream cards that predate the platform change, and those cards read this section at planning: `MVP-018`'s scope line "rules/indexes/storage" and its AC-4 "Dev rules, indexes, Storage, and App Check posture" resolve to **Storage not provisioned in dev**, with that absence as the reviewed posture.

**Quotas, with the arithmetic.** Firestore free tier is 50,000 document reads and 20,000 writes per day; a publish is one write and a preview view is one read. Firebase Hosting free tier is 10 GB/month **and 360 MB/day**, and the daily figure binds first: at roughly 50 KB per preview page that is about **7,200 views/day**, not the ~200,000/month step-40 quoted from the monthly figure alone. Exceeding the ceiling disables the site after a short grace period rather than billing, so the failure is closed and the local core loop is unaffected throughout.

**No per-household share count cap.** Step-40's 200-share cap is dropped. Its stated purpose was a cost ceiling, which no longer exists; it is per-household while nothing caps households, so it constrains honest users more than a determined one; and security rules cannot count documents, so enforcing it would require a counter document maintained by batched writes, rules validating every increment and decrement, and drift repair. It is replaced by the control rules *can* express in one clause: **reject any share document over 64 KB**. A wordy real recipe runs 5–10 KB, giving 6–12× headroom while blocking a payload approaching Firestore's own 1 MiB document ceiling. A count cap may be added later if abuse is actually observed.

**Revocation.** Step-40's trigger — "source-recipe delete revokes its shares" — names an operation that does not exist: `MVP-008` resolved (owner, 2026-08-28) that recipes are **archived, never hard-deleted**, and `MVP-011` reuses the same marker to hide starter recipes. The resolved rule is that **setting `archived_at` marks every share of that recipe for revocation**, and restore does not republish — the user republishes and receives a fresh ID, so a URL that has 404'd stays dead.

Because there are no Cloud Functions, that cascade runs client-side, and archiving is a core-loop action that must work offline (PRD §6.5, invariant 17). Revocation is therefore **eventually consistent**: when online it applies immediately; when offline it queues, surfaces as pending, and retries on connectivity, and the UI states that the public link will be removed when the device is next online rather than claiming it is already gone. Silently implying a public copy is gone while it still serves is the leakage class invariant 11 exists to prevent. This reuses the pending-unsynced-state machinery `MVP-018` AC-6 already requires. `MVP-008` AC-3's existing delete confirmation gains the count of public links the action will remove.

**Rights and publication rules (discharges invariant 12; `MVP-020`'s decision gate routes this here, and `Decisions to resolve` above did not name it).** The rule is keyed on provenance origin:

- **Starter content** publishes freely with no attribution machinery. `MVP-011` shipped `original`, `us_federal_public_domain` and `cc0` only, having excluded CC BY at planning precisely because the attributed surface was `MVP-020`'s unbuilt work. None of the three surviving bases carries an attribution obligation. Re-admitting CC BY later is possible once the preview carries attribution, but `MVP-011` is `Done` and this decision does not reopen it.
- **User-authored content** publishes in full, instructions included, and the publish action stores a rights-attestation flag on the projection. That flag is what makes `MVP-020` AC-3 assertable in a fixture instead of unverifiable prose. A technical restriction to structured fields only was rejected: ingredient lists alone give a recipient nothing to cook from, hollowing out the value `MVP-020` and `MVP-021` exist to prove, and creative selection and arrangement can attract protection anyway, so it is not the clean shield it appears to be.

**Abuse, report and takedown.** The owner is the abuse contact; the preview page carries a report path to an owner-controlled address, finalized with the `DEC-004` domain. Takedown at beta scale is **deletion through the Firebase console** — no administrative code path is built, and none should be inferred, because a takedown means the owner removing another household's projection while revoke is publisher-initiated. DMCA designated-agent registration (US Copyright Office, $6, renewable every three years) becomes an **`MVP-022` acceptance criterion**, not a note here: safe harbour only matters once third parties can publish, which first happens when the Android beta opens, and a criterion blocks that gate whereas a footnote does not. Registering earlier would expose a home address or cost roughly $50–150/yr for a box or agent service during a window in which the owner is the only publisher.

**Retention.** Projections persist until revoked. No expiry or scheduled cleanup in the MVP (invariant 16). Account- and household-level deletion is `MVP-022`'s existing deletion/export posture; orphaned projections with no remaining owner are removable through the console by the same route as takedown.

**Stale projections — recorded here because no card owns it.** Projections are immutable and versioned, so editing a recipe after publication leaves the public page on the old version indefinitely. `MVP-020` carries the obligation to make that visible rather than silent: the preview shows a `published on <date>` line, and propagating an edit requires a republish.

### Decision 3 — Hosting and domain

**The default `<project>.web.app` origin serves all development evidence**, with the name-free project ID from decision 1. It is real HTTPS and serves `/.well-known/assetlinks.json`, so full App Links verification is rehearsable against the dev flavor's application ID and debug signing certificate for `MVP-020` and `MVP-021` evidence, with no external action beyond the authorized project creation. Buying a domain now is rejected: it couples to unresolved `DEC-004` naming and carries purchase and DNS actions this card does not authorize. Emulator-only hosting is rejected: App Links verification needs a real HTTPS origin. Standard HTTPS only; no Firebase Dynamic Links and no deferred-install promise. A custom domain is `DEC-004` identity work at production activation, where App Links re-verify against the release certificate at `MVP-022`.

### Decision 4 — App Check rollout and failure posture

**Adopt at `MVP-018` with the debug provider; enforcement stays off in dev; enforcement plus the Play Integrity provider becomes an `MVP-022` acceptance criterion.**

Step-40's "monitor, then enforce in dev" is not viable as written. App Check requires the `PLAY_RECOGNIZED` app-recognition label by default, and an application not published on Google Play cannot receive it, so enforcing in dev would reject the sideloaded dev build's own cloud writes. The only two exits from that state are registering the dev build's debug token — which is this resolution with extra steps — or relaxing the recognition requirement in the console, which weakens the check for every environment including the one that matters. Enforcing in dev would also override `MVP-018`'s load-bearing constraint that App Check uses a development/debug posture only and creates no false security claim. Whether the Play internal-testing track clears `PLAY_RECOGNIZED` is confirmed at `MVP-022`, not assumed here.

The flip at `MVP-022` is evidenced by the console's verified-versus-unverified request metric on real traffic, which is the baseline step-40 wanted and could not have obtained from self-issued dev tokens.

Two rules are recorded regardless:

- **Debug tokens are per-device secrets and never enter version control** — a local Gradle property or environment variable only. A committed debug token is worse than no App Check, because it bypasses the check while appearing to protect. `MVP-018` AC-4 evidence includes a repository grep asserting this.
- **Enforcement scopes to Firestore and Auth only.** The public preview is deliberately unauthenticated by locked constraint; App Check must never be placed in front of Hosting, and a later "gap closure" that does so is a regression.

**Failure posture.** Under enforcement, cloud writes fail closed. The local core loop has no cloud dependency and cannot be degraded by it (invariant 17, PRD §14.7), and the unauthenticated preview stays reachable.

### Environment matrix

| Env | Project | Billing | Config | Storage / Functions | App Check | Hosting | Authorization |
|---|---|---|---|---|---|---|---|
| Emulator | none | — | local only | none | debug, unenforced | emulator | already authorized (D-008) |
| Dev | real non-production, name-free ID, created at `MVP-018` | **no payment method** | committed dev flavor | **not provisioned** | debug, never enforced | `<project>.web.app` | `MVP-018` scope |
| Production | later | at activation | never committed before activation | decided at activation | enforced, Play Integrity, re-confirmed `MVP-022` | custom domain (`DEC-004`) | separate owner authorization |

### Threats and mitigations

| Threat | Mitigation |
|---|---|
| Share-ID enumeration | Opaque 128-bit random IDs |
| Household leakage through the public surface | Projection-copy with a field allowlist; no rules path from public records to household documents |
| Oversized or junk payloads | 64 KB per-document rules clause; Firestore's own 1 MiB ceiling behind it |
| Cost abuse | No payment method; free-tier ceilings fail closed; durable auth required to publish |
| Availability abuse | Hosting disables at ~7,200 views/day; the local core loop is unaffected |
| Accidental production exposure | Fail-closed flavors, distinct application IDs, production config never committed (invariant 15) |
| Abusive or infringing content | Report path, owner takedown via console, rights attestation, DMCA agent at `MVP-022` |
| Stale public copy after an edit | `published on <date>` on the preview; republish to propagate |
| Public copy surviving an offline archive | Queued revocation with a pending indicator and honest wording |
| Bypassed App Check | Debug tokens never committed; repository grep in `MVP-018` AC-4 |
| Premature naming commitment | Name-free dev project ID; irreversible identifiers gated on `DEC-004` clearance |

### Rollback

Per share: delete the projection, and the URL 404s. Per environment: disable dev Hosting or remove the dev configuration, and the app degrades to local-only with the core loop intact. Nothing in this resolution activates production, paid services, DNS, signing, or store submission.

### Completion criteria

- **Threat-informed architecture, environment matrix, cost boundary, rollback/revocation plan recorded — PASS.** All four are above.
- **Authorized local/non-production work separated from later external activation — PASS.** The matrix's authorization column, plus the naming gate on irreversible identifiers.
- **`MVP-018` and `MVP-020` plannable without inventing security policy — PASS.** Both cards' open content is decided here, including the rights rule `MVP-020`'s decision gate routes to this card and the Storage clause the platform change invalidated.

### Downstream obligations created by this resolution

| Card | Obligation |
|---|---|
| `MVP-018` | Storage not provisioned; App Check debug-only and unenforced; debug-token grep in AC-4 evidence; pending-revocation state shares AC-6's machinery |
| `MVP-020` | 64 KB rules clause; rights-attestation flag asserted in AC-3; `published on <date>` on the preview; queued revocation with honest offline wording; no administrative takedown path |
| `MVP-008` | Delete confirmation names the count of public links the archive will remove |
| `MVP-022` | DMCA designated-agent registration; App Check enforcement flip with the Play Integrity provider on console metrics; staging created only if rehearsal demonstrates need |
| `DEC-004` | Authoritative USPTO/EUIPO/WIPO search replaces the current `NOT VERIFIED`; irreversible identifiers stay gated on a clean result |
