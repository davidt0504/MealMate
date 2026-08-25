# Kimatta — Product Requirements Document v3
## Food Outcome Controller on the Household Control Kernel

**Status:** Authoritative working PRD for development pivot  
**Date:** 2026-08-24  
**Supersedes:** `MealMate_PRD_v2.md` where this document changes product or architecture direction  
**Consumer brand:** **Kimatta**

---

## 1. Executive decision

Kimatta remains a focused consumer food product. It does **not** become a general family organizer in v3.

The architectural change is that Kimatta is no longer treated as a self-contained meal-planning application. It becomes the first bounded domain controller and dedicated user interface on a deliberately small **Household Control Kernel**.

> **Kimatta maintains a useful desired food state for the household while minimizing the administrative human attention required to reach and maintain it.**

The familiar loop—recipes → meal plan → pantry-aware shopping list → shopping → repeat—remains important, but it is a mechanism. The deeper product is a feedback controller that observes known household food state, compares it with the desired state, proposes/chooses bounded actions under constraints, surfaces only material uncertainty, and learns from ordinary use.

### Architecture decision

- **Rust** owns the durable household-control kernel, food-domain model, planner/control logic, and authoritative local SQLite state.
- **Flutter/Dart** owns the polished Android/iOS consumer UI and platform-facing presentation.
- A generated Dart↔Rust bridge provides a deliberately **coarse-grained FFI boundary**.
- Cloud services are optional adapters. Core planning must not depend on cloud availability or a remote LLM.

This is not primarily a micro-performance optimization. Rust is chosen because the durable control layer should be portable, strongly typed, independently testable, reusable by future interfaces/controllers, and capable of growing into more demanding optimization workloads without being coupled to Flutter widget state.

---

## 2. Product mission and outcome model

### 2.1 Mission

> **Reduce the amount of remembering, deciding, coordinating, checking, and maintenance required to feed a household.**

Kimatta should return administrative attention rather than capture it.

The long-term capability ladder is:

1. **Remember** — retain household meals, preferences, policies, history, traditions, and useful state.
2. **Prepare** — proactively form plans and shopping requirements and surface only unresolved decisions.
3. **Act** — execute explicitly authorized, low-risk/reversible external actions where trustworthy and useful.

### 2.2 Desired food state

For the active planning horizon, Kimatta should attempt to make this statement true:

> **Enabled household meals are adequately covered, practically feasible under known constraints, acceptable to the people eating them, and represented by an actionable shopping state—with unresolved uncertainty surfaced only when it materially matters.**

Coverage states may include:

- `unresolved`
- `tentatively_covered`
- `covered`
- `locked_by_user`
- `intentionally_open`
- `needs_attention`

“Covered” is always relative to known information. It does **not** mean that Kimatta guarantees that a meal will be cooked, that the pantry is exact, or that an ingredient/product is medically proven safe.

### 2.3 Public positioning

Kimatta should remain understandable without exposing control-systems terminology.

Candidate messaging:

- **Kimatta — Meal planning. Decided.**
- **Get meal planning out of your head.**
- **The better Kimatta knows your household, the less you should need to manage it.**

---

## 3. Governing principles

The full cross-product constitution is `HOUSEHOLD_CONTROL_PRINCIPLES.md`. Kimatta MUST conform to it.

Key Kimatta rules:

1. Outcomes over engagement.
2. Administrative attention is a scarce control resource.
3. Automate administration, not meaning.
4. Humans retain authority at consequential boundaries.
5. Hard constraints are not tradable utility weights.
6. State quality outranks optimizer cleverness.
7. Uncertainty/provenance must be explicit where automation depends on them.
8. Learn from ordinary use rather than interrogating users.
9. Rules/algorithms first; ML where evidence helps; LLMs at messy boundaries.
10. Local/private by default.
11. Optimize robustness and plan stability, not merely nominal mathematical score.
12. Automated decisions must be reproducible/debuggable.
13. No ads, engagement feeds, streaks, or notifications designed to manufacture usage.
14. Do not prematurely generalize food-specific concepts into a universal ontology.

---

## 4. Target users and jobs

### Primary user

The adult or adults carrying recurring household food-planning cognitive labor: deciding meals, remembering preferences/restrictions, accounting for schedules, checking what is available, creating shopping requirements, handling leftovers, remembering traditions, and revising plans when life changes.

The household—not one abstract user—is the planning unit.

### Core jobs

Kimatta should reduce or eliminate the work involved in answering:

- What are we eating?
- Will the household accept it?
- Does it fit the time/circumstances of that day?
- What hard restrictions apply?
- Have we eaten it too recently?
- What can be reused across the cycle?
- What do we need to buy?
- What can safely remain intentionally unresolved?
- What changed enough that a human really needs to decide?

---

## 5. Architecture

```text
┌─────────────────────────────────────────────────────────┐
│                     FLUTTER SURFACE                     │
│ Kimatta UI · navigation · accessibility · platform UX   │
└───────────────────────┬─────────────────────────────────┘
                        │ generated Dart↔Rust bridge
                        ▼
┌─────────────────────────────────────────────────────────┐
│              HOUSEHOLD CONTROL KERNEL — RUST            │
│ household/member identity                               │
│ policy envelope                                         │
│ control-relevant evidence/provenance                    │
│ outcome assessments                                     │
│ action proposals                                        │
│ attention requests                                      │
│ audit/outcome ledger                                    │
│ minimal controller contract                             │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                   FOOD DOMAIN — RUST                    │
│ recipes · meals · pantry · preferences · constraints    │
│ FoodController · candidate generation · planner         │
│ shopping-list derivation · coverage assessment          │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                        ADAPTERS                          │
│ SQLite · notifications · calendar · cloud · LLM/OCR     │
│ sharing/web · future commerce                           │
└─────────────────────────────────────────────────────────┘
```

### Dependency rule

Domain controllers may depend on the kernel. The kernel must never depend on a domain.

Future controllers should not directly import one another when a normalized shared context/port can be used instead.

---

## 6. Technology stack and alternatives

### 6.1 Flutter/Dart for presentation

**Choice:** retain Flutter for Android/iOS UI.

Why:
- existing Kimatta work is already Flutter;
- mature cross-platform mobile UI/tooling;
- accessibility, navigation, animations, widgets, and platform plugin ecosystem;
- fast UI iteration/hot reload;
- avoids duplicating SwiftUI/Compose presentation work.

Alternatives:

**Compose Multiplatform** — credible and increasingly mature. It could be superior for a Kotlin-first team or Android-heavy product. It does not eliminate the need for a portable control core and would force a presentation rewrite now.

**Fully native SwiftUI + Jetpack Compose** — maximizes native integration but duplicates presentation effort and increases small-team maintenance.

**Rust UI frameworks (Dioxus/Slint/etc.)** — strategically interesting but not currently superior to Flutter for a polished mainstream consumer mobile surface. Use Rust where it provides durable architectural leverage rather than language purity.

### 6.2 Rust for durable kernel/domain

**Choice:** stable Rust pinned with `rust-toolchain.toml`.

Reasons:
- strong static types and explicit ownership;
- memory safety without a GC;
- portable native library across Android/iOS/desktop/server/CLI/simulation tooling;
- well suited to deterministic state machines, SQLite ownership, optimization/search, and high-integrity tests;
- keeps control/domain logic independent of Flutter lifecycle and view state;
- performance headroom exists if later scheduling/optimization becomes computationally heavier.

Rust cost/risk:
- learning curve;
- borrow/ownership concepts;
- two-language repo;
- native compilation/cross-target build setup;
- debugging a language boundary.

These are accepted only because the boundary is intentionally coarse and the durable kernel is expected to outlive the first UI.

### 6.3 FFI / Dart↔Rust bridge

**FFI = Foreign Function Interface.** It is simply the mechanism that lets code in one language/runtime call compiled functions from another. Here Flutter/Dart calls a compiled Rust library.

**Preferred tooling:** `flutter_rust_bridge` after a mandatory integration spike.

Rules:
- bridge service-level operations, not field-by-field UI access;
- generated bindings are not hand-edited;
- explicit DTOs cross the boundary;
- no SQLite handles cross the boundary;
- UI-only ephemeral state stays in Dart;
- durable control state stays in Rust.

Good examples:
- `cover_cycle(request) -> PlanningResult`
- `get_active_cycle_view(id) -> CycleViewDto`
- `apply_user_decision(decision) -> UpdatedAssessment`

Bad examples:
- one FFI call for every ingredient-row render;
- Dart manually stepping the Rust search algorithm;
- duplicated mutable authoritative models in both languages.

**Phase-0 requirement:** verify Android debug/release and iOS CI/release packaging before moving substantive business logic. If one FRB packaging backend is unstable, use another supported integration path rather than abandoning the architecture immediately.

### 6.4 Local persistence

**Choice:** SQLite owned by Rust via `rusqlite`.

Why:
- transactional and mature;
- reliable historical/query workloads;
- explicit schema migrations;
- portable across future surfaces;
- deterministic planner snapshots;
- local-first source of truth rather than treating a cloud cache as the operational database.

Alternatives:

**Firestore-first/offline cache (v2 design):** excellent collaboration velocity, but less attractive when complex local history/control queries become foundational and complete local state matters.

**Drift in Dart:** excellent SQLite tooling, but wrong ownership if Rust is the control/state kernel; it would split persistence responsibility or force chatty boundary calls.

**Object databases:** convenient for app CRUD but weaker fit for a durable, queryable, portable systems kernel.

**Pure event sourcing:** attractive for auditability but too complex as the sole storage model. Use relational current state plus append-oriented controller/audit records.

### 6.5 Cloud/backend

Core MVP planning has no cloud dependency.

Cloud may support:
- auth/sync later;
- share-preview hosting;
- product/crash analytics;
- server-only paid LLM operations;
- future integrations.

Do **not** build a generalized sync engine in v3 MVP.

### 6.6 Suggested Rust libraries

Use current compatible versions verified during implementation, not blindly frozen from this PRD.

Likely choices:
- `serde` / `serde_json` — DTO/export serialization;
- `uuid` — IDs;
- `thiserror` — typed error enums;
- `tracing` — structured diagnostics;
- `jiff` — civil dates/time zones;
- `rusqlite` — SQLite;
- explicit migration helper or small tested migration runner.

Avoid adding Tokio/async runtime merely because Rust supports it. Keep the core synchronous unless a demonstrated need exists; execute heavier Rust work off the Flutter UI thread through the bridge.

---

## 7. Minimal Household Control Kernel

Do not build a universal world model. The v1 kernel contains only primitives already justified by Kimatta and strongly likely to recur in another controller.

### 7.1 Household identity

Typed IDs and ownership/membership primitives:

```text
HouseholdId
MemberId
UserId
```

The kernel does not know what a recipe or meal is.

### 7.2 Control-relevant evidence/provenance

Do not wrap every field in probability.

Use coarse categories where automated decisions depend on truth quality:

```text
EvidenceSource:
  ExplicitUser
  VerifiedIntegration
  Derived
  InferredBehavior
  ImportedUntrusted

Confidence:
  Explicit
  High
  Low
  Unknown
```

Optional metadata includes observation time, expiry/staleness, source reference, and evidence count.

Use numerical probabilities only after calibration data exists.

### 7.3 Policy envelope

Kernel form:

```text
Policy {
  id,
  household_id,
  domain,
  policy_type,
  parameters,
  enabled,
  source
}
```

The kernel stores/routes. The food controller interprets `food.*` policy types.

No universal policy DSL in v3.

### 7.4 OutcomeAssessment

A controller can report whether its desired state is covered and why not.

```text
OutcomeAssessment {
  controller_id,
  horizon,
  status,
  unresolved_issues[],
  assumptions[],
  reason_codes[]
}
```

### 7.5 ActionProposal

A proposed action is not authorization.

```text
ActionProposal {
  id,
  controller_id,
  action_type,
  expected_benefit_band,
  confidence,
  reversibility,
  required_authority,
  deadline?,
  reason_codes[]
}
```

### 7.6 AttentionRequest

Controllers ask for human attention through a structured request rather than directly manufacturing engagement.

```text
AttentionRequest {
  id,
  controller_id,
  urgency,
  deadline?,
  decision_benefit_band,
  estimated_effort_band,
  options[],
  reason_codes[]
}
```

### 7.7 Outcome/audit ledger

Record meaningful state transitions and planner decisions:
- algorithm version;
- input snapshot hash/version;
- candidate/selection reason codes;
- selected action;
- human correction;
- resulting coverage state.

Do not make the entire product event-sourced.

### 7.8 Minimal controller contract

Conceptual form:

```rust
trait HouseholdController {
    fn assess(&self, ctx: &ControlContext) -> Result<OutcomeAssessment>;
    fn propose(&self, ctx: &ControlContext) -> Result<Vec<ActionProposal>>;
}
```

Keep this tiny. Food-specific commands need not be forced into a premature generic trait.

---

## 8. Food domain model

Preserve the good v2 decisions, implemented in Rust/SQLite:

- `Ingredient`
- `CustomIngredient`
- `IngredientLine`
- `Recipe`
- `RecipeProvenance`
- `MealStub`
- `PlannedMeal`
- `MealComponent`
- `PantryItem`
- `ShoppingListItem`
- `HouseholdMemberProfile`
- `FoodPreferenceEvidence`
- `MealHistoryEvidence`
- `FoodPolicy`
- `MealConstraint`
- `PlannerRun`
- future `MealTradition`

### Required semantics

- a planned meal is a meal occurrence, not permanently equivalent to one recipe;
- multiple components are supported even if MVP UI usually shows one main;
- non-recipe meals (leftovers, dining out/takeout, frozen/quick, freeform, intentionally open/away) are first-class in the schema;
- planned ≠ confirmed cooked;
- pantry is simple and optional, not a quantified warehouse inventory;
- member preferences are member-scoped;
- hard restrictions remain hard constraints;
- unknown safety data remains unknown.

### Civil dates

Conceptual meal dates use local/civil dates, not UTC instants. “Tuesday dinner” should not shift because of timezone conversion.

---

## 9. FoodController algorithm v1

The planner must be deterministic for the same input snapshot + algorithm version.

### 9.1 Why not an LLM planner

A recurring LLM planner is rejected because it is harder to reproduce, can invent unavailable actions, complicates hard-constraint verification, adds latency/cost/network dependence, and weakens offline behavior.

LLMs may later parse/generate messy candidate data. Accepted output becomes ordinary structured state and passes deterministic validation.

### 9.2 Why not CP-SAT in MVP

**CP-SAT** is Google OR-Tools' constraint-programming/SAT solver for discrete/integer optimization. It is excellent for complex scheduling, assignment, optional intervals, precedence, capacity, and Boolean constraints.

Do not use it in Kimatta MVP because:
- OR-Tools officially targets C++, Python, Java, and C#, not Rust;
- integrating its C++ library into a Rust mobile core adds another native packaging/binding boundary;
- Kimatta's first weekly meal problem is small enough for a transparent bounded search;
- state/candidate quality is the larger early bottleneck.

CP-SAT remains a serious future candidate for a ScheduleController or for substantially more complex discrete constraints.

### 9.3 Candidate generation

For each unresolved meal slot generate candidates from:
1. existing plan/lock state;
2. household recipes and meal stubs;
3. starter meals;
4. leftovers;
5. takeout/dining out if enabled;
6. frozen/quick fallback;
7. intentionally open.

An LLM may not silently invent a trusted action.

### 9.4 Hard filtering

Reject candidates with known:
- restriction conflicts;
- explicit vetoes when policy makes them hard;
- impossible prep window;
- locked/explicit-user-choice conflict.

Absence of detected allergen data is not proof of safety.

### 9.5 Candidate features/scoring

Within the feasible set calculate inspectable components such as:
- household acceptance;
- schedule fit and prep slack;
- recency/return interval;
- familiarity/novelty fit;
- ingredient overlap;
- pantry fit;
- leftover utility;
- cost only when reliable data exists;
- gentle balanced-eating tie-breaker.

Every score component should have reason codes suitable for debugging/explanation.

### 9.6 Whole-cycle search: beam search

**Beam search** keeps only the best `B` partial plans at each step instead of exhaustively exploring every combination.

MVP process:
1. order slots canonically or process most constrained first;
2. expand each partial plan using top feasible candidates;
3. score partial plans including sequence effects;
4. retain top `B` states;
5. continue until cycle is complete;
6. choose best final plan with deterministic tie-breaking.

Why beam search:
- easy to implement in pure Rust;
- bounded runtime/memory;
- sequence-aware;
- transparent/debuggable;
- replaceable later behind the same planner interface.

Beam width/candidate pruning must be benchmarked on fixtures rather than guessed as product truth.

### 9.7 Lexicographic optimization hierarchy

Do not collapse the controller into one universal weighted score.

**Tier 0 — hard/inviolable**
- known safety/restriction conflicts;
- explicit hard veto;
- locks;
- impossible feasibility;
- explicit human decisions.

**Tier 1 — coverage/serious failure avoidance**
- enabled slots covered;
- prep feasible;
- serving needs plausible.

**Tier 2 — household acceptance**
- predicted satisfaction;
- strong-dislike protection;
- sequence-level fairness.

**Tier 3 — robustness and stability**
- prefer slack;
- penalize fragile time fits;
- penalize unnecessary plan churn;
- preserve near-term accepted commitments.

**Tier 4 — administrative attention**
- minimize questions;
- minimize manual swaps/corrections;
- minimize foreground planning time.

**Tier 5 — secondary efficiency**
- variety/recency;
- ingredient reuse;
- pantry use;
- leftovers;
- cost;
- gentle health balance.

A lower tier never compensates for violating a higher tier.

### 9.8 Robustness/slack

Prefer a 20-minute meal in a 30-minute window over a 29-minute meal when other quality is similar.

`slack = available_minutes - expected_required_minutes`

Fragile plans receive a penalty as slack approaches zero.

### 9.9 Plan churn and commitment horizon

After plan acceptance:
- distant unlocked meals may change when materially beneficial;
- near-term meals require a larger benefit threshold;
- locked meals never move automatically.

Do not continuously “improve” the week for trivial score gains.

### 9.10 Household preference aggregation

MVP strategy:
1. hard restrictions;
2. explicit hard veto/strong dislike treatment;
3. estimate per-member candidate satisfaction;
4. enforce a minimum floor when appropriate;
5. optimize group aggregate;
6. penalize repeated week-level sacrifice of the same member.

Do not use pure averaging or pure least-misery alone.

### 9.11 Learning roadmap

Start with explicit evidence + simple household-local adaptation.

Future candidates, only when data supports them:
- Bayesian updating for preference confidence/return intervals;
- change-point detection for preference drift;
- contextual bandits for safe low-stakes exploration among already feasible meals;
- lightweight local models for acceptance or duration prediction.

Do not start with end-to-end reinforcement learning; rewards are sparse, delayed, confounded, and multi-person.

---

## 10. Model sufficiency

Before claiming a cycle/day is “covered,” evaluate whether the known state can support that claim.

Examples:
- no prep-time estimate → no strong schedule-fit claim;
- incomplete pantry → list can be ready but stock completeness is unknown;
- restrictions skipped → do not imply restriction verification;
- little preference history → label plan conservative/default rather than highly personalized.

Surface assumptions only when they materially affect trust or action.

Priority order:

> **state correctness > constraint completeness > candidate-action completeness > optimization sophistication**

---

## 11. Attention governor v1

Kimatta must not ask questions merely because more data would be useful.

Conceptual rule:

`attention_priority = expected_decision_benefit - interruption/effort_cost`

Use qualitative bands/hand-built thresholds in v3, not fake numerical VOI estimates.

High priority:
- plan infeasible;
- hard constraint unresolved;
- imminent uncertainty with material downside.

Low priority:
- curiosity;
- marginal preference refinement;
- analytics/data collection;
- distant trivial optimization.

Bundle low-urgency questions into an intentional planning moment. Routine/high-confidence work stays silent.

---

## 12. SQLite/schema discipline

### Source of truth

Rust-owned SQLite is authoritative for local operational state. Flutter does not independently mutate the same durable tables.

### Schema rules

- normalized relational tables for core entities;
- stable immutable IDs;
- foreign keys enabled;
- explicit migrations;
- transactions for multi-record transitions;
- append-oriented planner/audit records;
- JSON only for bounded/versioned metadata or policy parameters, not as a substitute for core schema design.

### Migration rules

Every schema change:
- has a migration;
- is tested from representative prior versions;
- has recovery/backup consideration;
- never silently destroys existing user data.

### Backup/export

Before mature public release, provide versioned export/backup and restore validation. A system that successfully becomes external memory has a higher duty to protect that memory.

---

## 13. Rust/Flutter ownership boundary

### Rust owns
- durable household/member domain identity;
- food data;
- policies/constraints;
- persistence;
- planner/control execution;
- shopping-list derivation;
- coverage assessments;
- audit/outcome ledger.

### Flutter owns
- route/navigation state;
- animations;
- ephemeral form state before submission;
- screen/view selection;
- accessibility/presentation;
- OS-specific surface composition.

### Platform adapters

A platform integration may remain in Dart/Swift/Kotlin when that ecosystem is superior. Rust is the trusted kernel, not a religion.

Future calendar integration should produce normalized household timing constraints consumed by the FoodController rather than embedding calendar-provider logic inside food optimization.

---

## 14. Security/privacy

1. Household data private by default.
2. No advertising profiles or sale of household behavior.
3. Cloud requests receive minimum necessary data.
4. External email/web/text is untrusted input; it cannot modify policy or authority.
5. LLM output is untrusted until structured/validated.
6. Sensitive telemetry remains local/coarse.
7. Auth/sync failure cannot break local core planning.
8. Future action authority is explicit, scoped, and revocable.
9. Consequential actions use prepare/stage/verify/commit patterns where possible.
10. Evaluate database-at-rest encryption only with an explicit threat/key-management design; do not invent custom crypto.

---

## 15. MVP user experience

### First run

No mandatory durable account.

Minimal setup:
- individual vs household framing;
- optional household name;
- clear/skippable restriction setup;
- optional shopping/planning rhythm;
- quick path to meal stubs/starter meals.

No long preference questionnaire.

### Defining workflow

1. User adds/selects familiar meals quickly.
2. User locks anything they care about.
3. User invokes **Cover My Week** / equivalent.
4. FoodController locally generates a coherent cycle.
5. UI shows covered state plus only important unresolved assumptions/decisions.
6. User accepts/swaps with minimal effort.
7. Shopping list is immediately derived.
8. Ordinary behavior becomes evidence for future cycles.

Manual planning always remains available.

### Mature design target

Default experience should trend toward:

> **This week is covered.**

or

> **2 things need you.**

rather than an empty grid requiring reconstruction.

---

## 16. MVP scope

### Required kernel
- household/member identity;
- policy envelope;
- control-relevant evidence/provenance;
- outcome assessment;
- action proposal;
- attention request;
- audit/outcome ledger;
- tiny controller contract.

### Required food domain
- structured recipe/ingredient model;
- meal stubs;
- meal occurrence/components;
- simple pantry;
- planning dates/cycle;
- shopping-list consolidation;
- restrictions;
- locked meals;
- basic food policies/constraints;
- deterministic whole-cycle planner;
- coverage/model-sufficiency assessment;
- planner version/reason codes;
- offline operation.

### Required UI
- recipe/meal library;
- planning-cycle view;
- Cover My Week;
- swap/lock/manual override;
- shopping list;
- pantry;
- minimal preferences/settings.

### Demote before cutting controller proof
- recipe photo support;
- very broad starter pack;
- URL-import stretch;
- outbound sharing/web-preview polish;
- social polish;
- breakfast/lunch presentation polish.

### Not MVP
- generalized household dashboard;
- second domain controller;
- calendar ingestion;
- CP-SAT/OR-Tools;
- universal knowledge graph;
- generic policy DSL;
- autonomous purchases;
- custom sync engine;
- cross-household behavioral ML;
- cloud LLM dependency;
- quantified pantry inventory;
- public social feed.

---

## 17. Migration from current Kimatta repository

Do not rewrite blindly.

### Phase 0 — repository inventory

Before destructive changes:
1. inspect Git branch/status/uncommitted work;
2. inventory current Flutter/Dart versions/dependencies;
3. identify screens/models/services/repositories/persistence/Firebase/tests;
4. determine whether real persisted user data exists;
5. map each significant component to KEEP / MIGRATE / ADAPT / DEFER / DELETE;
6. write `docs/V3_MIGRATION_PLAN.md`.

### Phase 1 — Rust bridge spike

- add Rust toolchain/workspace;
- integrate FRB;
- expose trivial health/version call;
- prove Android debug/release;
- prove iOS CI/release packaging;
- document clean-checkout generation/build commands;
- pin toolchains/bridge versions.

No major business-logic migration until this works.

### Phase 2 — kernel and SQLite

- migrations;
- minimal `household-core`;
- SQLite adapter;
- initial repository/application services;
- tests.

### Phase 3 — migrate food state incrementally

Suggested order:
- ingredient;
- recipe/meal stub;
- planned meal/component;
- pantry;
- shopping list.

Keep UI functional where practical.

### Phase 4 — FoodController

- candidate generation;
- hard filtering;
- scoring/reason codes;
- beam search;
- coverage assessment;
- planner ledger;
- Cover My Week UI.

### Phase 5 — hardening

- fixtures/property tests;
- migrations;
- offline testing;
- performance benchmarks;
- backup/export;
- privacy-safe telemetry.

---

## 18. Testing strategy

### Rust unit tests

Core planner tests run without Flutter.

Cover:
- restrictions;
- locks;
- recency;
- sequence variety;
- fairness;
- pantry assumptions;
- shopping consolidation;
- reproducibility;
- coverage/model sufficiency.

### Synthetic/golden household fixtures

Create version-controlled fixtures for:
- cold start;
- conservative household;
- high-variety household;
- multiple strong dislikes;
- busy week;
- many locked meals;
- sparse/empty pantry;
- restrictions set/skipped;
- leftovers/fallback-heavy week.

Do not assert one exact plan when multiple plans are equally acceptable. Assert invariants and quality properties.

### Property/invariant tests

Examples:
- known hard restriction never selected;
- locked meal never moves;
- same deterministic input/version yields same result;
- adding a hard constraint cannot make an infeasible candidate feasible;
- shopping quantities never become negative;
- historical planner records are not mutated by a later plan.

### Performance

Benchmark on representative low/mid-range Android hardware. Establish budgets from profiling rather than inventing arbitrary millisecond targets.

### Bridge tests

Test:
- DTO mapping;
- typed errors;
- large payloads;
- lifecycle/reinitialization;
- Android/iOS release packaging.

---

## 19. Metrics

Do not optimize DAU/session length upward.

Track two independent axes.

### A. Outcome quality
- planning cycles reaching accepted/ready coverage;
- auto-resolved unresolved slots;
- hard-constraint violation rate (known constraints: target zero);
- correction/rejection rate;
- plan churn after acceptance;
- model-sufficiency/false-covered failures.

### B. Administrative attention
- active seconds from planning intent to ready state;
- explicit decisions;
- manual swaps/corrections;
- questions;
- system-initiated interruptions.

Optimize the Pareto frontier: same/better outcomes for less attention.

Business activation, repeat cycles, retention, conversion, and LTV still matter commercially, but must not cause engagement-maximizing product design.

Never fabricate counterfactual “hours saved.”

---

## 20. Monetization

No ads.

Core deterministic intelligence should be useful without artificial scarcity.

Potential future ladder:

### Remember
- richer household memory/history;
- backup/sync;
- advanced policies.

### Prepare
- proactive plan refresh;
- schedule-derived constraints;
- advanced cost/waste optimization;
- cross-device household automation.

### Act
- staged grocery/cart integrations;
- authorized external actions.

Variable-cost LLM convenience may also be premium: messy recipe/photo/handwriting import, transformations, Fridge Rescue/creative generation.

Premium sells **delegated responsibility and expensive convenience**, not attention or arbitrary message quotas.

---

## 21. Roadmap

### MVP — Rust kernel + bounded FoodController
- architecture pivot;
- local SQLite;
- food model;
- plan/list/pantry;
- basic policies/restrictions;
- deterministic whole-cycle planner;
- coverage/attention/correction telemetry.

### Phase 1.5 — adaptive FoodController
- member feedback;
- inferred preference evidence;
- improved recency/variety;
- robustness/stability;
- confidence-aware questions;
- outcome ledger refinement.

### Phase 2 — shared household food controller
- durable auth;
- multi-user sync;
- member-specific preference collaboration;
- traditions;
- richer leftovers/non-recipe flows;
- fairness.

### Phase 2.5 — second-domain proof

Build the smallest useful second controller, likely **Schedule Feasibility**:
- consume calendar commitments;
- infer/represent usable household time windows;
- detect feasibility constraints;
- expose normalized timing constraints;
- do not build a family-calendar replacement.

Let FoodController consume those derived constraints.

Only then decide which additional abstractions truly belong in the shared kernel/supervisor.

### Phase 3 — delegated actions
- proactive preparation;
- low-risk authorized reshuffling;
- staged grocery integrations;
- richer OS/system surfaces.

---

## 22. Adversarial architecture review

### Risk: Rust slows shipping
**Why real:** learning curve, native build tooling, bridge debugging.  
**Mitigation:** Rust owns only durable core; Flutter owns UX; APIs stay coarse; platform spike first.  
**Kill criterion:** if a bounded spike cannot produce reliable Android+iOS release builds, preserve interfaces and temporarily implement the core in Dart rather than blocking the product indefinitely.

### Risk: FFI becomes a chatty distributed-monolith boundary
**Mitigation:** screen/use-case DTOs and service-level calls; no database handles/row getters across bridge.

### Risk: premature universal kernel
**Mitigation:** second-domain evidence required before new universal abstractions.

### Risk: optimizer solves the wrong model
**Mitigation:** model sufficiency, provenance, conservative behavior, candidate completeness work before optimizer sophistication.

### Risk: beam search becomes a dead end
**Mitigation:** hide implementation behind planner interface; maintain fixtures that allow later MILP/CP-SAT comparison.

### Risk: SQLite complicates collaboration
**Mitigation:** defer generalized sync; stable IDs/granular state; solve collaboration when it becomes a real requirement.

### Risk: two-language cognitive burden
**Mitigation:** simple Rust style, limited crates, no unsafe in domain/application, strong tests, generated bridge, explicit ownership docs.

### Risk: “outcome controller” becomes marketing vapor
**Mitigation:** `Cover My Week` is required MVP proof. CRUD-only does not satisfy v3.

### Risk: minimizing attention hides important uncertainty
**Mitigation:** severity/safety/reversibility override attention-saving.

### Risk: architecture ideology harms mobile integration
**Mitigation:** keep OS-specific adapters in Dart/Swift/Kotlin where better. Rust is the kernel, not a purity rule.

---

## 23. Decision log

| Decision | v3 choice | Rationale |
|---|---|---|
| Consumer scope | Food-focused Kimatta | Clear acquisition/JTBD |
| Long-term role | First Household Control controller | Prevent future silo/rewrite |
| Core language | Rust | Portable durable control kernel |
| UI | Flutter/Dart | Mature consumer-mobile UX; preserve work |
| Interop | Generated FRB after spike | Coarse Dart↔Rust boundary |
| Local source of truth | SQLite in Rust | Transactional/queryable/portable/auditable |
| Cloud | Optional/non-core in MVP | Offline independence |
| MVP planner | deterministic filter + scoring + beam search | Transparent, bounded, native Rust |
| CP-SAT | future candidate | Powerful but unnecessary/no official Rust API |
| Optimization | lexicographic tiers | Hard constraints never traded away |
| Uncertainty | coarse evidence/provenance first | Avoid fake probability |
| Attention | explicit scarce resource | Questions require justification |
| LLM | unstructured/generative boundary | Not trusted recurring controller |
| Universal abstractions | minimal kernel | Avoid framework-first overengineering |
| Second controller | Schedule Feasibility candidate | Direct Kimatta value + architecture proof |

---

## 24. Glossary

**FFI — Foreign Function Interface**  
A mechanism allowing one language/runtime to call functions compiled from another language. Here Dart/Flutter calls Rust. A generator such as `flutter_rust_bridge` creates most glue so application code does not manually work with C pointers.

**CP-SAT**  
Google OR-Tools' Constraint Programming/SAT solver for discrete/integer optimization. Particularly useful for scheduling, assignment, optional tasks, precedence and resource-capacity constraints. It is not needed for Kimatta MVP.

**Beam search**  
A bounded search that keeps only the best `B` partial solutions after each expansion step. It trades guaranteed global optimality for predictable speed and practical sequence-level planning.

**Lexicographic optimization**  
Objectives are ranked in priority tiers. A lower-priority gain cannot compensate for failure at a higher tier.

**Receding-horizon / Model Predictive Control concept**  
Estimate state, plan over a bounded future horizon, apply current actions, observe what changed, and replan rather than committing an uncertain entire future.

**Value of Information (VOI)**  
Expected improvement in decision quality from obtaining additional information. Used conceptually to decide whether asking a human is worth their attention.

---

## 25. Research/technical references

### Cognitive load / attention
- Aviv et al., *Cognitive household labor: gender disparities and consequences for maternal mental health and wellbeing*. https://doi.org/10.1007/s00737-024-01490-w
- Daminger, *The Cognitive Dimension of Household Labor*. https://doi.org/10.1177/0003122419859007
- Gilbert et al., *Outsourcing Memory to External Tools: A Review of Intention Offloading*. https://pubmed.ncbi.nlm.nih.gov/35789477/
- Jones et al., prospective-memory intervention systematic review/meta-analysis. https://pubmed.ncbi.nlm.nih.gov/33393806/
- Gilbert et al., *Optimal use of reminders: Metacognition, effort, and cognitive offloading*. https://pubmed.ncbi.nlm.nih.gov/31448938/
- Fitz et al., *Batching smartphone notifications can improve well-being*. https://doi.org/10.1016/j.chb.2019.07.016

### Rust / Flutter / interop
- Rust releases: https://blog.rust-lang.org/releases/
- Rust iOS targets: https://doc.rust-lang.org/stable/rustc/platform-support/apple-ios.html
- Rust Android targets: https://doc.rust-lang.org/rustc/platform-support/android.html
- Dart FFI: https://dart.dev/interop/c-interop
- Dart build hooks/code assets: https://dart.dev/tools/hooks
- flutter_rust_bridge: https://pub.dev/packages/flutter_rust_bridge
- Flutter platform support: https://docs.flutter.dev/reference/supported-platforms
- Kotlin Multiplatform alternatives: https://kotlinlang.org/docs/multiplatform/supported-platforms.html
- Dioxus mobile alternative: https://dioxuslabs.com/learn/0.7/guides/platforms/mobile/

### Persistence / optimization
- rusqlite: https://docs.rs/rusqlite/latest/rusqlite/
- Jiff: https://docs.rs/jiff/latest/jiff/
- Google OR-Tools: https://developers.google.com/optimization/
- CP-SAT: https://developers.google.com/optimization/cp/cp_solver
- OR-Tools install/languages: https://developers.google.com/optimization/install/
- good_lp: https://docs.rs/good_lp/latest/good_lp/
- HiGHS Rust bindings: https://docs.rs/highs/latest/highs/

---

## 26. MVP acceptance test

Kimatta v3 MVP is complete when:

1. Flutter reliably calls the Rust core in Android and iOS release pipelines.
2. Rust owns authoritative local food/control state in SQLite.
3. User can start without mandatory durable account.
4. Household/member structure exists.
5. User can quickly create/select meal stubs/recipes.
6. Restrictions and a small number of food policies are represented.
7. Manual/locked meals are respected.
8. **Cover My Week** locally generates a coherent plan without LLM/network dependency.
9. Known hard constraints are never knowingly violated.
10. Planner evaluates the cycle as a sequence, not independent meals only.
11. Important unresolved assumptions are surfaced and low-value questions remain silent.
12. Shopping list is derived immediately from accepted plan.
13. Core loop works offline.
14. Planner runs are versioned/reproducible enough to debug.
15. Ordinary behavior creates structured evidence for future cycles.
16. Analytics can measure coverage/corrections/attention without exporting sensitive household content.
17. The product demonstrates the central claim:

> **The household reaches a useful food state with materially less administrative human effort than reconstructing the plan manually.**
