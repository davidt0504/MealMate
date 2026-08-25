# MVP-020 — Share publication and web preview

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Sharing/security |
| Depends on | MVP-009, MVP-018, DEC-003 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Dev hosting/domain only with explicit approval; no production deploy/DNS |

> **v3 amendment (2026-08-24, D-028/D-029):** Cloud sharing is an optional adapter over a projected public representation; the projection is derived from Rust-owned state. Authoritative source adds `docs/PRD_v3.md` §6.5, §14; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Let an authorized household member publish a privacy-bounded recipe projection, open it on the web without auth, report it, and revoke it.

## Authoritative sources

- `docs/PRD_v2.md` §§7.10–7.11, 14.7, 20–21, 24.9; `docs/ROADMAP.md` D-009, DEC-003 result; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Public records are immutable/versioned projections with opaque unguessable IDs, never reads of household documents.
- Projection allowlist excludes household IDs, private metadata, restrictions/preferences, analytics IDs, and storage paths/tokens.
- Revoked/missing content fails closed; inbound preview needs no account.
- Imported content follows rights/attribution policy and may link to source instead of republishing.
- Publication may require durable auth; report intake has an owner and abuse boundary.

## Scope

- Projection/publish/revoke service, rules/functions tests, preview states and metadata, reporting intake, expiry/retention/cost controls from DEC-003, and dev evidence.

## Non-goals

- Android routing, deferred deep links, social feed, comments, search indexing strategy, production deployment, or iOS links.

## Decision gates

- Do not publish until DEC-003 security, abuse, rights, retention, and hosting choices are complete and external dev deployment is authorized.

## Acceptance criteria

- **AC-1:** Only an authorized durable household user can publish/revoke an eligible recipe.
- **AC-2:** Anonymous recipients see only allowlisted projection fields and correct missing/revoked states.
- **AC-3:** Imported/private rights cases follow policy and attribution survives the preview.
- **AC-4:** Report, abuse, enumeration, rate/cost, and privacy-leakage tests pass.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Rules/function integration and negative tests |
| AC-2 | Snapshot/schema allowlist tests and unauthenticated browser check |
| AC-3 | Rights/provenance fixture review |
| AC-4 | Threat-derived tests plus independent security/privacy review |

## Stop/failure conditions

- Stop on direct household reads, enumerable IDs, unowned abuse workflow, ambiguous rights, or unauthorized deployment. Two cycles then re-plan.

## Handoff

Record evidence/status and SHARING-SECURITY-READY progress in `docs/ROADMAP.md`.
