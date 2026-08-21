# MVP-021 — Android App Links recipient flow

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Sharing/Android |
| Depends on | MVP-020, DEC-004 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Real domain association, DNS, and signing require explicit approval; no production/store action |

## Outcome and user value

Route standard HTTPS recipe links predictably into the installed Android app while preserving a useful browser preview for everyone else.

## Authoritative sources

- `docs/PRD_v2.md` §§7.10, 14.7, 24.9; `docs/ROADMAP.md` D-009, D-015; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Android only; iOS Universal Links are post-launch.
- No Firebase Dynamic Links or cross-install deferred-deep-link guarantee.
- Validate link host/path/opaque ID and handle malformed, missing, revoked, offline, and unauthorized-save states safely.
- Viewing needs no account; saving/planning uses normal anonymous-first household behavior.

## Scope

- Android intent filters/router, cold/warm start, installed/uninstalled browser behavior, recipient preview/save/plan path, offline/error states, and association evidence where authorized.

## Non-goals

- iOS, deferred attribution, custom link provider, production domain activation, or store campaign analytics.

## Decision gates

- Real association proof pauses for explicit domain/DNS/signing authorization; local test hosts do not count as production readiness.

## Acceptance criteria

- **AC-1:** Valid links open the correct recipe on cold and warm installed-app starts.
- **AC-2:** Uninstalled/unsupported clients receive the web preview without an account wall.
- **AC-3:** Unauthenticated, offline, malformed, missing, and revoked cases are safe and recoverable.
- **AC-4:** An anonymous recipient can save/plan an eligible recipe without gaining source-household access.
- **AC-5:** Real Android domain association is PASS or explicitly NOT VERIFIED pending separately authorized external setup.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Android integration tests on cold/warm starts |
| AC-2 | Browser/device matrix |
| AC-3 | Negative/offline scenario tests |
| AC-4 | Auth/rules integration test |
| AC-5 | Digital Asset Links verification and device result, or bounded NOT VERIFIED record |

## Stop/failure conditions

- Stop on privilege leakage, unsafe fallback, iOS scope, or unauthorized DNS/signing. Two cycles then re-plan.

## Handoff

Record evidence/status and SHARING-SECURITY-READY result in `docs/ROADMAP.md`.
