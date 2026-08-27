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
