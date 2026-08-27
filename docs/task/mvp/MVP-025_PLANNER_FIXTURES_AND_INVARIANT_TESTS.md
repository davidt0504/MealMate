# MVP-025 — Planner fixtures and invariant tests

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Planning/quality |
| Depends on | MVP-023 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | None |

> **Body derived 2026-08-26 (D-029, D-034)** after DEC-005 (D-030).

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-025`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Create version-controlled synthetic household fixtures, property and invariant tests, and the beam-width benchmark that turns `MVP-023`'s search parameters from a guess into a measured choice. Assert invariants and quality properties, never one exact plan when several are equally acceptable.

## Authoritative sources

- `docs/PRD_v3.md` §18 (testing strategy; the fixture list), §9.6 ("Beam width/candidate pruning must be benchmarked on fixtures rather than guessed as product truth")
- `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Tests"
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §28 (engineering/research discipline)
- `docs/ROADMAP.md` D-012, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 18, 20

## Load-bearing constraints

- **Assert invariants and quality thresholds, never one exact plan** where the fixture admits several equally acceptable answers (PRD §18).
- Fixtures are version-controlled and synthetic. No real household data, no real names, no real restrictions.
- The benchmark's **numbers, not intuition, justify** the beam width and candidate limits `MVP-023` shipped (PRD §9.6, principles §28).
- Benchmarks run on representative low/mid-range Android hardware; budgets come from profiling, not from an invented millisecond target (PRD §18 "Performance").
- The suite runs without Flutter — core planner tests are pure Rust (PRD §18).
- A test that cannot fail proves nothing: each invariant test must be demonstrated against a deliberately broken planner before it counts.

## Scope

**Eleven fixtures** — the union of two source lists, sourced per item. PRD §18's nine: cold start; conservative household; high-variety household; multiple strong dislikes; busy week; many locked meals; sparse/empty pantry; restrictions set and skipped; leftovers/fallback-heavy week. Plus the two the pivot prompt names that §18 omits: repetitive meal library; intentionally open nights.

**Six property/invariant tests** (PRD §18, pivot prompt):

1. A known hard restriction is never selected.
2. A locked meal never moves.
3. The same deterministic input and algorithm version yield the same result.
4. Adding a hard constraint cannot make a previously infeasible candidate feasible.
5. Shopping quantities never become negative.
6. Historical planner records are not mutated by a later plan.

**The beam-width and candidate-limit benchmark**, with its recorded verdict: hardware, method, numbers, and whether `MVP-023`'s shipped values survive.

## Non-goals

- Changing planner behavior — a benchmark finding that the shipped beam width is wrong produces a `MVP-023` change, recorded here as the evidence for it.
- ML, statistical calibration, or a CP-SAT comparison harness beyond keeping the fixtures reusable for one later (PRD §22 "beam search becomes a dead end" mitigation).
- UI tests (`MVP-024`), device performance budgets for anything but the planner.

## Decision gates

- If the benchmark shows the shipped beam width or candidate limit is wrong, the change belongs to `MVP-023`. This card records the measurement and hands it over; it does not tune the planner itself. An adverse verdict is a material change to `MVP-023`'s recorded decisions and returns that card to Draft under D-031, with the AC-4 benchmark row as the trigger.

## Acceptance criteria

- **AC-1:** Every fixture is version-controlled, deterministic, and documented with what it is meant to stress and which source list it comes from.
- **AC-2:** Each of the six invariants has a test that has been **shown to fail** against a deliberately broken planner, with that demonstration recorded.
- **AC-3:** No test asserts a single exact plan where its fixture admits several equally acceptable ones.
- **AC-4:** The beam-width/candidate-limit benchmark is recorded with hardware, method, numbers and verdict.
- **AC-5:** `cargo test --workspace` runs the whole suite without Flutter.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Fixture files in the repository plus their README table mapping each to its stressor and source |
| AC-2 | Six recorded red-then-green demonstrations, one per invariant |
| AC-3 | Fresh-context review of every assertion for exact-plan coupling |
| AC-4 | Benchmark output with the device model, method and verdict recorded in `docs/ROADMAP.md` |
| AC-5 | `cargo test --workspace` output, run with no Flutter toolchain invoked |

## Stop/failure conditions

- Stop if a test is weakened to make it pass, if a fixture encodes real household data, or if a benchmark run only on the development host is reported as a device budget.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record PASS/FAIL/NOT VERIFIED evidence, the benchmark verdict, resulting status, LOCAL-CORE-LOOP-READY progress, and **Next implementation task**. Update the corresponding checklist rows in `docs/V3_IMPLEMENTATION_STATUS.md` without duplicating live current/next status there.
