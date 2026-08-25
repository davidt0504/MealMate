# MVP-019 — Observability foundation

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Observability/privacy |
| Depends on | MVP-018 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Dev-only analytics/crash configuration; production activation requires separate approval |

> **v3 amendment (2026-08-24, D-028/D-029):** Add privacy-safe planner metric hooks (run/version, cycle covered, auto-resolved slots, decisions, swaps, duration bucket, coarse reason codes); never export names, restrictions, recipe text, pantry, notes, or exact schedules; time-in-app is an anti-metric. Authoritative source adds `docs/PRD_v3.md` §19; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Provide privacy-safe diagnostics and a stable event contract that can reveal failures without collecting sensitive household or recipe content.

## Authoritative sources

- `docs/PRD_v2.md` §§14.2, 15, 17, 21; `docs/ROADMAP.md` D-010, D-012; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Adapters are vendor-light and disabled/testable; product features own their later event emissions.
- Never send recipe/ingredient text, restrictions, names, emails, household content, share tokens, or free text.
- Use a small versioned taxonomy focused on completion, failure, latency, and recovery—not engagement maximization.
- Value/financial metrics remain labeled and do not become fabricated client events.

## Scope

- Define event/error taxonomy, consent/config posture, analytics/crash adapters, redaction, dev validation, and privacy tests/documentation.

## Non-goals

- Full dashboards, marketing attribution, notification growth loops, billing, production enablement, or every feature event.

## Decision gates

- Stop for owner/legal review if the minimum data contract cannot be stated without sensitive fields.

## Acceptance criteria

- **AC-1:** Allowed event names/properties and prohibited data are versioned and documented.
- **AC-2:** Automated tests prove redaction/drop behavior for sensitive examples.
- **AC-3:** Dev analytics/crash events can be observed and globally disabled.
- **AC-4:** Independent privacy review finds no sensitive payload or misleading value metric.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Contract/schema review |
| AC-2 | Privacy unit tests |
| AC-3 | Dev console/debug evidence and disable test |
| AC-4 | Fresh-context payload inspection |

## Stop/failure conditions

- Stop on sensitive payload, opaque auto-collection, production activation, or policy ambiguity. Two cycles then re-plan.

## Handoff

Record evidence/status and REAL-DEV-BACKEND-READY completion in `docs/ROADMAP.md`.
