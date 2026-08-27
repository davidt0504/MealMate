# MVP-023 — FoodController planner core

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Planning/domain (Rust) |
| Depends on | MVP-002, MVP-009, MVP-012, MVP-014 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | None |

> **Body derived 2026-08-26 (D-029, D-034)** after DEC-005 (D-030).

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-023`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Implement the deterministic local Cover My Week controller in Rust: candidate generation for every unresolved enabled slot, Tier-0 hard filtering, inspectable feature scoring with reason codes, whole-cycle beam search under the lexicographic tiers, coverage and model-sufficiency assessment, attention requests, and an append-oriented planner ledger. The kernel primitives it needs (`Policy`, `OutcomeAssessment`, `ActionProposal`, `AttentionRequest`) land here, not earlier (principles §25).

This is the card that makes the v3 claim real. PRD §22 names Cover My Week the required MVP proof; CRUD alone does not satisfy v3.

## Authoritative sources

- `docs/PRD_v3.md` §7 (kernel primitives), §9 (algorithm v1), §10 (model sufficiency), §11 (attention governor), §2.2 (coverage states), §6.3
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Planner implementation", "Attention/model-sufficiency behavior"
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §4–§9, §13–§15, §25, §27
- `docs/V3_MIGRATION_PLAN.md` §3 (crate boundaries and arrival points)
- `docs/ROADMAP.md` D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 9, 18, 19, 20, 21

## Load-bearing constraints

- **Deterministic** for the same input snapshot and algorithm version, including across process restarts. Every run records the algorithm version, the input snapshot identity, and reason codes (invariant 20, PRD §9, §7.7).
- **No LLM and no network** anywhere in the planning path (invariant 9, PRD §9.1).
- **No CP-SAT or OR-Tools** (PRD §9.2, §16 "Not MVP").
- Lexicographic tiers 0–5. Hard restrictions, explicit hard vetoes and locks are **Tier 0, never score weights**, and no lower tier may compensate for a higher-tier failure (invariant 18, PRD §9.7).
- Absence of detected allergen data is not proof of safety; coverage is always relative to known information (invariant 19, PRD §9.4, §2.2).
- **Model sufficiency is evaluated before any "covered" claim** (PRD §10): no prep-time estimates means no strong schedule-fit claim; an incomplete pantry means no stock-completeness claim; skipped restrictions means no restriction-verification claim; sparse preference history means a plan is labelled conservative, not personalised.
- Attention requests are **produced, not acted on**. The controller never decides to notify (principles §9, PRD §11).
- Kernel primitives land here because this is their first call site; nothing generic is added without one (principles §25).
- `household-core` never imports a food type (PRD §5 dependency rule).
- Planner and audit records are append-oriented and never mutated by a later run (invariant 20, PRD §7.7, §12).
- Beam width and candidate limits are benchmarked on fixtures rather than asserted as product truth (PRD §9.6). The benchmark itself is `MVP-025`.

## Scope

- Kernel primitives with their first real call site: `Policy`, `OutcomeAssessment`, `ActionProposal`, `AttentionRequest`, the coarse evidence/provenance categories, and the append-oriented outcome/audit ledger (PRD §7.2–7.7).
- The tiny `HouseholdController` contract — `assess` and `propose` (PRD §7.8). Food-specific commands are not forced into it.
- Candidate generation in the `food-domain` crate for every unresolved enabled slot, from all seven PRD §9.3 sources: existing plan/lock state, household recipes and meal stubs, starter meals, leftovers, takeout/dining out where enabled, frozen/quick fallback, intentionally open. The starter-meal candidate source is implemented against the shared entity shape; it does not require `MVP-011`'s shipped content to exist, so this card does not depend on `MVP-011`.
- Tier-0 hard filtering (PRD §9.4).
- Inspectable feature scoring with reason codes (PRD §9.5).
- Deterministic whole-cycle beam search: canonical or most-constrained slot ordering, partial-plan expansion, sequence-aware scoring, top-`B` retention, deterministic tie-breaking (PRD §9.6).
- Coverage and model-sufficiency assessment producing the PRD §2.2 states.
- Plan churn and commitment-horizon behavior (PRD §9.9).
- Household preference aggregation per PRD §9.10 — neither pure averaging nor pure least-misery.
- The `kimatta-application` crate, whose arrival `docs/V3_MIGRATION_PLAN.md` §3 places at exactly this card. `food-domain` already exists from MVP-005/007; this card extends it rather than creating it.
- One coarse bridge command returning a planning-result DTO (PRD §6.3, invariant 21).

## Non-goals

- The Cover My Week UI (`MVP-024`); fixtures, property tests and the beam-width benchmark (`MVP-025`).
- ML, Bayesian updating, contextual bandits, reinforcement learning (PRD §9.11 — future candidates only when data supports them).
- CP-SAT, a generic policy DSL, a universal ontology, a second domain controller, a global attention governor.
- Cost optimization where no reliable cost data exists (PRD §9.5).

## Decision gates

- Slot ordering (canonical versus most-constrained-first) and the initial beam width are this card's choices, recorded with the reason. `MVP-025`'s benchmark may revise them; this card must not assert either as settled product truth.
- If household preference aggregation cannot satisfy PRD §9.10 without an owner tradeoff on fairness, stop for `deep-options` rather than picking a fairness rule unilaterally.

## Acceptance criteria

- **AC-1:** A candidate with a known restriction conflict, an explicit hard veto, an impossible prep window, or a lock conflict is never selected — at any beam width.
- **AC-2:** The same input snapshot and algorithm version produce identical plans across repeated runs and across process restarts, and each run records algorithm version, snapshot identity and reason codes.
- **AC-3:** The planner evaluates the cycle as a sequence: a fixture where the per-slot-optimal choice is not the cycle-optimal one selects the cycle-optimal plan.
- **AC-4:** Coverage assessment reports model sufficiency honestly across the four PRD §10 cases and never claims safety, exact stock, or personalization the state cannot support.
- **AC-5:** Attention requests carry urgency, decision-benefit band, estimated-effort band, options and reason codes; the planner performs no notification and no external action itself.
- **AC-6:** Historical planner and audit records are byte-unchanged after a later planning run.
- **AC-7:** `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean; no `unsafe` in the new crates; `household-core` imports no food type.
- **AC-8:** One coarse bridge command carries the whole operation; no per-field call and no SQLite handle crosses the boundary.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | `cargo test --workspace` Tier-0 suite, run across at least two beam widths |
| AC-2 | Repeat-run and restart determinism tests; recorded ledger row showing version, snapshot identity, reason codes |
| AC-3 | Named sequence fixture with its expected-property assertion (not one exact plan) |
| AC-4 | Four model-sufficiency tests, one per PRD §10 case, plus independent copy review of any "covered" wording |
| AC-5 | Attention-request unit tests; grep confirming no notification or IO call in the planner crate |
| AC-6 | Ledger-immutability test replanning over existing records |
| AC-7 | Command output; `grep -rn unsafe` empty; dependency-direction grep |
| AC-8 | Bridge signature review and generated-binding diff |

## Stop/failure conditions

- Stop on any kernel primitive added without a call site; on a Tier-0 constraint expressed as a weight; on an LLM or network dependency; on non-determinism; on a hand-edited generated binding; or on a planner that mutates a historical record.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record PASS/FAIL/NOT VERIFIED evidence, the chosen slot ordering and beam width with their rationale, the resulting status, LOCAL-CORE-LOOP-READY progress, and **Next implementation task**. Update the kernel-primitive and planner checklist rows this card discharges in `docs/V3_IMPLEMENTATION_STATUS.md`, without duplicating live current/next status there.
