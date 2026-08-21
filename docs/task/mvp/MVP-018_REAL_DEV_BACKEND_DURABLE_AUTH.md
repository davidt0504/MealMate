# MVP-018 — Real dev backend and durable auth

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Cloud/auth |
| Depends on | MVP-017, DEC-003, DEC-004 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Creating/configuring a non-production Firebase project and auth credentials requires explicit approval; never production |

## Outcome and user value

Prove the local core loop on a separate real development backend and let anonymous users upgrade identity without losing household data.

## Authoritative sources

- `docs/PRD_v2.md` §§14.2, 14.6, 19.2; `docs/ROADMAP.md` D-008, DEC-003 result; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Development configuration is visibly separate and fails closed; production remains nonexistent/unconfigured.
- Rules/indexes deploy from versioned source and are tested before use.
- Anonymous credential linking preserves UID/household data where possible; collision/recovery behavior is explicit.
- App Check uses development/debug posture only and does not create a false security claim.

## Scope

- Configure the approved dev project, secrets/config boundary, rules/indexes/storage, durable auth provider, upgrade/restart/recovery flows, and Android-device/emulator evidence.

## Non-goals

- Production project, release signing, invitations, iOS providers, billing/subscriptions, or sharing publication.

## Decision gates

- External project/provider creation pauses until separately authorized; credential collision policy must be resolved before implementation.

## Acceptance criteria

- **AC-1:** Dev builds cannot silently target production or run with missing environment identity.
- **AC-2:** Anonymous household data survives durable-account upgrade and restart.
- **AC-3:** Collision/cancel/network/retry cases preserve recoverability and honest messaging.
- **AC-4:** Dev rules, indexes, Storage, and App Check posture pass independent review on Android evidence.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Configuration negative tests and project-ID evidence |
| AC-2 | Real-dev integration scenario |
| AC-3 | Fault/collision matrix |
| AC-4 | Deployment diff, rules tests, Android run, fresh-context security review |

## Stop/failure conditions

- Stop on production target, committed secret, data-loss path, paid-resource surprise, or missing external approval. Two cycles then re-plan.

## Handoff

Record evidence/status and REAL-DEV-BACKEND-READY progress in `docs/ROADMAP.md`.
