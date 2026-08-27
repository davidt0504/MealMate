# MVP-019 — Observability foundation

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Observability/privacy |
| Depends on | MVP-018 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Dev-only analytics/crash configuration; production activation requires separate approval |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below. Per D-034 the consent posture and vendor choice are **preserved as written** — `DEC-003` and owner/legal review decide those.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-019`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Provide privacy-safe diagnostics and a stable event contract that can reveal failures without collecting sensitive household or recipe content.

## Authoritative sources

- `docs/PRD_v3.md` §19 (metrics: the two axes), §11 (attention governor), §14 item 6 (sensitive telemetry stays local/coarse)
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Privacy-safe metrics"; `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §22, §30
- `docs/ROADMAP.md` D-010, D-012, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 13, 20
- Historical (D-028): `docs/PRD_v2.md` §§14.2, 15, 17, 21

## Load-bearing constraints

- Adapters are vendor-light and disabled/testable; product features own their later event emissions.
- Never send recipe/ingredient text, restrictions, names, emails, household content, pantry contents, private notes, exact schedules, share tokens, or free text.
- Use a small versioned taxonomy focused on completion, failure, latency, and recovery—not engagement maximization.
- Value/financial metrics remain labeled and do not become fabricated client events.
- **Two independent axes, never collapsed into one score** (PRD §19). Axis A, outcome quality: planning cycles reaching accepted/ready coverage; auto-resolved unresolved slots; hard-constraint violation rate for known constraints, target zero; correction/rejection rate; plan churn after acceptance; model-sufficiency and false-covered failures. Axis B, administrative attention: active seconds from planning intent to ready state; explicit decisions; manual swaps/corrections; questions; system-initiated interruptions. The goal is the Pareto frontier — same or better outcomes for less attention.
- **Time-in-app is an anti-metric.** DAU, MAU and session length are never product objectives (PRD §19, principles §22).
- Never fabricate counterfactual "hours saved" (PRD §19).

## Scope

- Define the event/error taxonomy, consent/config posture, analytics/crash adapters, redaction, dev validation, and privacy tests/documentation.
- Emission hooks for the planner, all seven from the pivot prompt: planner run and algorithm version; cycle covered; auto-resolved slot count; explicit decision count; correction/swap count; planner duration bucket; coarse reason-code frequency. These are the *shape* of emission; the metric set above is what they must be able to answer.

## Non-goals

- Full dashboards, marketing attribution, notification growth loops, billing, production enablement, or every feature event.

## Decision gates

- Stop for owner/legal review if the minimum data contract cannot be stated without sensitive fields.
- **How a §19 metric owned by an already-Done card is discharged is not decided by PRD v3 or a recorded D-number.** This card runs at `docs/task/SEQUENCE.txt` steps 44–45, after the planner and UI cards at 22–37, and the constraint above puts later event emissions with the product features — so most Axis A and Axis B metrics map to cards that are Done by then. Whether such a metric is carried as a D-027 `NOT VERIFIED` item, returns its owning card to Draft under D-031, or is assigned to a follow-on card is this card's choice, recorded with its reason; stop for the owner if AC-5's mapping cannot be completed under any of the three.

## Acceptance criteria

- **AC-1:** Allowed event names/properties and prohibited data are versioned and documented.
- **AC-2:** Automated tests prove redaction/drop behavior for sensitive examples.
- **AC-3:** Dev analytics/crash events can be observed and globally disabled.
- **AC-4:** Independent privacy review finds no sensitive payload or misleading value metric.
- **AC-5:** A mapping table names, for each Axis A and Axis B metric in PRD v3 §19, either the taxonomy event that answers it or the card that must emit it, and no entry requires a field on the prohibited list.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Contract/schema review |
| AC-2 | Privacy unit tests |
| AC-3 | Dev console/debug evidence and disable test |
| AC-4 | Fresh-context payload inspection |
| AC-5 | Metric-to-event mapping table, one row per §19 metric, reviewed against the prohibited-data list |

## Stop/failure conditions

- Stop on sensitive payload, opaque auto-collection, production activation, or policy ambiguity. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resulting status, REAL-DEV-BACKEND-READY completion, and **Next implementation task**.
