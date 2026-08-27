# MVP-020 — Share publication and web preview

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Sharing/security |
| Depends on | MVP-009, MVP-018, DEC-003 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Dev hosting/domain only with explicit approval; no production deploy/DNS |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below. Per D-034 this card's security, abuse, rights, retention and hosting content is **preserved as written** — `DEC-003` decides it.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-020`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Let an authorized household member publish a privacy-bounded recipe projection, open it on the web without auth, report it, and revoke it.

## Authoritative sources

- `docs/PRD_v3.md` §6.5 (cloud is an optional adapter), §14 (security/privacy), §13 (ownership boundary), §16
- `docs/ROADMAP.md` D-009, D-028, D-030, D-034, DEC-003 result; `docs/task/MVP_INVARIANTS.md` 11, 12, 15, 17
- Historical (D-028): `docs/PRD_v2.md` §§7.10–7.11, 14.7, 20–21, 24.9

## Load-bearing constraints

- The public projection is **derived from Rust-owned state through the coarse bridge** and published as an immutable, versioned record. Publication never exposes a read path into the household's own tables (invariants 11, 17).
- Sharing is an optional cloud adapter; its absence or failure must not affect the local core loop (PRD §6.5).
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

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, SHARING-SECURITY-READY progress, and **Next implementation task**.
