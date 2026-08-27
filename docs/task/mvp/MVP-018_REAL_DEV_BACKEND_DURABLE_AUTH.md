# MVP-018 — Real dev backend and durable auth

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Cloud/auth |
| Depends on | MVP-017, DEC-003, DEC-004 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Creating/configuring a non-production Firebase project and auth credentials requires explicit approval; never production |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below. Per D-034 this card's environment-topology, rules-deployment and App Check content is **preserved as written** — `DEC-003` decides it, and re-deriving it here would pre-empt that decision.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-018`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Prove the local core loop on a separate real development backend and let anonymous users upgrade identity without losing household data.

## Authoritative sources

- `docs/PRD_v3.md` §6.5 (cloud is an optional adapter; no generalized sync engine in v3 MVP), §14 item 7 (auth/sync failure cannot break local core planning), §16 "Not MVP"
- `docs/ROADMAP.md` D-008, D-028, D-030, D-034, DEC-003 result; `docs/task/MVP_INVARIANTS.md` 8, 15, 17
- Historical (D-028): `docs/PRD_v2.md` §§14.2, 14.6, 19.2

## Load-bearing constraints

- Cloud auth and sync are an **optional adapter**. Core planning never depends on them, and an auth or sync failure degrades only the optional feature — it can never break the local core loop (PRD §6.5, §14 item 7).
- No generalized sync engine in the MVP (PRD §16 "Not MVP"). Rust-owned SQLite remains the authoritative local store throughout (invariant 17).
- Development configuration is visibly separate and fails closed; production remains nonexistent/unconfigured.
- Rules/indexes deploy from versioned source and are tested before use.
- Anonymous credential linking preserves UID/household data where possible; collision/recovery behavior is explicit.
- App Check uses development/debug posture only and does not create a false security claim.

## Scope

- Configure the approved dev project, secrets/config boundary, rules/indexes/storage, durable auth provider, upgrade/restart/recovery flows, and Android-device/emulator evidence.
- Apply DEC-004's rename before any identifier binds to the dev project: `applicationId`, Android manifest label, `pubspec.yaml` name if DEC-004 renamed it, Firebase config, `README.md`, and `docs/`.

## Non-goals

- Production project, release signing, invitations, iOS providers, billing/subscriptions, or sharing publication.

## Decision gates

- External project/provider creation pauses until separately authorized; credential collision policy must be resolved before implementation.

## Acceptance criteria

- **AC-1:** Dev builds cannot silently target production or run with missing environment identity.
- **AC-2:** Anonymous household data survives durable-account upgrade and restart.
- **AC-3:** Collision/cancel/network/retry cases preserve recoverability and honest messaging.
- **AC-4:** Dev rules, indexes, Storage, and App Check posture pass independent review on Android evidence.
- **AC-5:** No `dev.mealmate.temp` and no `.temp` application identifier remains in `android/`, `pubspec.yaml`, Firebase config, `README.md`, `docs/`, or the Rust crate and package names under `rust/` (DEC-004's rename scope includes the crate names — `docs/task/SEQUENCE.txt` step 41).
- **AC-6:** A sign-out or identity transition with unsynced local state cannot silently lose household data. *(Transferred from `MVP-017` under D-034: no durable account exists at that card, so the obligation could not be tested there.)*

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Configuration negative tests and project-ID evidence |
| AC-2 | Real-dev integration scenario |
| AC-3 | Fault/collision matrix |
| AC-4 | Deployment diff, rules tests, Android run, fresh-context security review |
| AC-5 | Targeted identifier grep across the repository, including `rust/**/Cargo.toml` and crate names |
| AC-6 | Pending-state/sign-out tests plus fresh-context coverage review |

## Stop/failure conditions

- Stop on production target, committed secret, data-loss path, paid-resource surprise, or missing external approval. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, REAL-DEV-BACKEND-READY progress, and **Next implementation task**.
