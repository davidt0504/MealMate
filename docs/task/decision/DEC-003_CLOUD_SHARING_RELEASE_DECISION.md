# DEC-003 — Cloud and sharing release boundary

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Decision |
| Workstream | Cloud/security |
| Depends on | MVP-017 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | After local core loop is Done |
| Recommended workflow | `deep-options`; then `grill-me` for owner risk/cost choices |
| External actions | Research is read-only; creating projects/domains/credentials requires separate approval |

## Outcome and user value

Define a safe progression from emulators to a real non-production backend and public sharing, without accidentally creating production exposure.

## Authoritative sources

- `docs/PRD_v2.md` §§7.10–7.11, 14.2–14.8, 20–21
- `docs/ROADMAP.md` D-008, D-009, D-012, D-015
- `docs/task/MVP_INVARIANTS.md`

## Locked constraints

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
