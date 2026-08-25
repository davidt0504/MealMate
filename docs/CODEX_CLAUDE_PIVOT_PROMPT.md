# Coding-Agent Pivot Prompt — Kimatta v2 → v3 Rust Household-Control Architecture

You are working inside an existing Kimatta Flutter repository. Development has already begun. **Do not assume the repository is disposable, do not rewrite it wholesale, and do not start by coding speculative architecture.**

Authoritative documents in the repository:

- `PRD_v3.md`
- `HOUSEHOLD_CONTROL_PRINCIPLES.md`

If an older PRD conflicts with v3, v3 wins. Older documents are historical context.

## Mission

Pivot Kimatta from a Flutter-centric meal-planning application into:

> **a focused Kimatta Flutter UI backed by a Rust-first Food Outcome Controller and a deliberately minimal Household Control Kernel.**

Kimatta remains a consumer food product. Do **not** build a generalized family dashboard.

The primary MVP proof is:

> **The household can reach a useful food-covered state with less administrative attention than manual planning.**

The defining intelligent action is a local deterministic **Cover My Week / Cover My Cycle** controller.

---

## Non-negotiable ownership boundaries

### Flutter/Dart owns
- screens/widgets;
- navigation;
- accessibility;
- animations;
- transient form/view state;
- platform presentation and platform adapters when ecosystem support is better there.

### Rust owns
- household/member identity used by control logic;
- food-domain durable state;
- policies/constraints;
- control-relevant confidence/provenance;
- planner/scoring/search logic;
- shopping-list derivation;
- coverage/model-sufficiency assessments;
- SQLite authoritative local source of truth;
- planner/action/outcome audit records.

### Bridge
Use `flutter_rust_bridge` (FRB) after proving it works in this repository.

Keep the bridge **coarse-grained**.

Good APIs:
- `cover_cycle(request)`
- `get_active_cycle_view(id)`
- `save_recipe(command)`
- `apply_planning_decision(command)`

Bad APIs:
- one FFI call per rendered property;
- exposing SQLite handles;
- Dart manually orchestrating the Rust search loop;
- duplicate authoritative models on both sides.

Generated bridge files must not be hand-edited.

### Dependency direction

`household-core <- food-domain <- kimatta-application <- bridge`

The kernel must never import food-specific types.

Do not create a second controller in this task.

---

# FIRST ACTION: INSPECT, DO NOT MODIFY

Before changing code:

1. inspect `git status`, current branch, untracked/uncommitted work;
2. inventory repository structure;
3. identify Flutter/Dart versions and all dependencies;
4. identify existing models, screens, services, repositories, persistence, Firebase, tests, CI;
5. determine whether real persisted user data/schema exists;
6. map every significant component to:
   - KEEP in Flutter presentation;
   - MIGRATE to Rust;
   - ADAPT behind interface;
   - DEFER;
   - DELETE only if demonstrably obsolete;
7. write `docs/V3_MIGRATION_PLAN.md` before destructive refactoring.

Preserve user work. Never delete functioning code merely because v3 uses a different ownership boundary.

---

# Phase 0 — prove Rust integration first

The first engineering milestone is a platform/build spike.

## Toolchains

- verify current stable Flutter/Dart compatible with repo;
- pin concrete Rust stable in `rust-toolchain.toml`;
- pin FRB/runtime/codegen versions deliberately;
- stable Rust only unless a documented blocker exists.

The PRD was written when Rust 1.98.0 and FRB 2.13.0 were current; **verify current compatibility rather than blindly copying versions**.

## Spike requirements

1. integrate Rust workspace/library;
2. expose `core_version()` or `health_check()` through FRB;
3. call it from Flutter;
4. prove Android debug;
5. prove Android release;
6. prove iOS build/signing in CI/Mac environment;
7. verify typed errors cross bridge correctly;
8. document binding-regeneration/build commands from clean checkout;
9. verify development cycle is tolerable.

Evaluate FRB's current Native Assets/build-hook backend first if compatible, but treat packaging as a spike. If that backend is unstable, use another supported FRB integration method. Do not abandon the Rust architecture solely because one packaging backend is immature.

Stop and document blockers before migrating large amounts of code if release packaging is not reliable.

---

# Proposed repository shape

Adapt to the actual repo rather than force names, but preserve dependency direction.

```text
/
  lib/                         # existing Flutter application
    presentation/
    platform/
    bridge/

  rust/
    Cargo.toml                 # workspace
    rust-toolchain.toml
    crates/
      household-core/
      food-domain/
      storage-sqlite/
      kimatta-application/
      kimatta-bridge/
```

A flatter 3–5-crate workspace is acceptable. Do **not** create dozens of microcrates.

---

# Minimal Household Control Kernel

Implement only primitives justified by Kimatta:

- typed household/member IDs and ownership;
- `Policy` envelope (`domain`, `type`, parameters, enabled/source);
- coarse `Confidence` / `EvidenceSource` only where control decisions need it;
- `OutcomeAssessment`;
- `ActionProposal`;
- `AttentionRequest`;
- planner/outcome audit records;
- a tiny controller contract.

Do NOT implement:
- universal knowledge graph/triples;
- generic Entity/Predicate ontology;
- universal policy expression language;
- generic life optimizer;
- POMDP framework;
- generalized family UI;
- global AI-agent framework;
- second domain controller.

Use typed Rust structures.

---

# Persistence

Make Rust-owned SQLite authoritative.

Preferred access:
- `rusqlite`;
- explicit migrations (`rusqlite_migration` or a small tested migration runner);
- foreign keys enabled;
- transactions for multi-record changes.

Rules:
- Flutter does not separately mutate durable core tables;
- do not use giant untyped JSON blobs for domain relationships;
- JSON is acceptable for bounded/versioned policy params or export metadata;
- migration tests are mandatory;
- audit/planner records may be append-oriented, but do not event-source the entire app.

Before changing an existing persisted schema, establish whether real user data exists and create a migration/compatibility plan.

---

# Rust coding standards

The repo owner is new to Rust. Optimize for readability and correctness, not cleverness.

- stable Rust;
- `cargo fmt`;
- `cargo clippy -- -D warnings` after initial integration stabilizes;
- no `unsafe` in application/domain crates;
- isolate generated/necessary unsafe bridge/native code;
- `thiserror` for typed errors;
- `serde` for DTO/export structures;
- `tracing` for structured diagnostics;
- civil date type such as `jiff::civil::Date` for conceptual meal dates;
- avoid unnecessary async/Tokio in core;
- no mutable global planner state;
- planner consumes an explicit snapshot and returns a result;
- deterministic tie-breaking.

---

# Food-domain migration

Preserve v3/v2 semantics:

- Ingredient
- CustomIngredient
- IngredientLine
- Recipe + provenance
- MealStub
- PlannedMeal
- MealComponent
- PantryItem
- ShoppingListItem
- HouseholdMemberProfile
- FoodPreferenceEvidence
- MealHistoryEvidence
- MealConstraint
- FoodPolicy
- PlannerRun

Required behavior:
- planned meal can contain multiple components;
- non-recipe meal types are first-class;
- planned != confirmed cooked;
- pantry is simple/optional, not quantified inventory;
- hard restrictions never become ordinary score weights;
- unknown safety data remains unknown;
- preferences are household-member scoped.

Migrate incrementally and keep Flutter functional where practical.

---

# Planner implementation

Do not use an LLM for recurring planning.

Do not use CP-SAT/OR-Tools in MVP.

## Candidate generation

For each unresolved enabled meal slot include candidates from:
- existing planned/locked state;
- household recipes/stubs;
- starter meals;
- leftovers;
- takeout/dining out if enabled;
- quick/frozen fallback;
- intentionally open.

## Hard filtering

Reject known:
- restriction conflicts;
- explicit hard vetoes;
- impossible prep-time candidates;
- lock/explicit-choice violations.

Unknown allergy/manufacturing information remains unknown; never claim safety merely because no conflict is detected.

## Feature scoring

Return inspectable score components/reason codes:
- household acceptance;
- schedule fit/slack;
- recency;
- variety/familiarity;
- ingredient overlap;
- pantry fit;
- leftovers;
- cost only when reliable;
- gentle health tie-breaker only.

## Whole-cycle search

Implement deterministic **beam search**:
1. canonical or most-constrained slot order;
2. expand partial plans;
3. score sequence effects;
4. retain top `B` partial plans;
5. deterministic tie-breaking;
6. benchmark beam width/candidate limits with fixtures.

## Lexicographic objective

Priority tiers:
0. hard constraints;
1. coverage/feasibility;
2. household acceptance/fairness;
3. robustness/stability;
4. administrative attention;
5. secondary variety/cost/health/ingredient-efficiency.

Lower tiers never compensate for higher-tier failure.

## Stability

- locks are absolute;
- penalize churn;
- use a commitment/freeze horizon;
- do not change an accepted near-term meal for trivial gains.

## Output

Return:
- proposed/selected plan;
- coverage assessment;
- unresolved issues;
- assumptions;
- reason codes;
- algorithm version;
- attention/action proposals where relevant.

---

# Attention/model-sufficiency behavior

Every question must justify itself.

Use simple qualitative priority:

`expected decision benefit - interruption/effort cost`

High priority:
- current plan infeasible;
- hard constraint unresolved;
- imminent material uncertainty.

Low priority:
- collect preference data for its own sake;
- analytics curiosity;
- marginal distant optimization.

Before UI says “covered,” verify model sufficiency:
- missing prep time -> no strong schedule-fit claim;
- incomplete pantry -> no complete-stock claim;
- skipped restrictions -> no allergy/restriction verification claim;
- cold-start preferences -> conservative/default, not “personalized.”

Do not invent exact probabilities without calibration.

---

# Shopping-list behavior

Deterministic rules:
- scale compatible numeric ingredient quantities by planned servings;
- merge by canonical ingredient identity only when unit/identity compatibility is trustworthy;
- uncertain/incompatible lines remain separate;
- pantry presence may suppress purchase but does not prove sufficient quantity;
- user can restore an omitted line;
- group by store category.

Test heavily.

---

# UI pivot

Do not remove manual planning.

Add the defining intelligent action:

> **Cover My Week**

Flow:
1. user locks/chooses anything important;
2. controller fills/repairs remainder;
3. UI shows only material unresolved decisions;
4. user accepts/swaps;
5. shopping list is already ready.

Long-term default state:
- “This week is covered.”
- or “2 things need you.”

No feed, streak, notification growth hack, or artificial daily engagement surface.

---

# Tests

Create Rust unit/domain tests and reusable planning fixtures.

Fixtures:
- cold start;
- busy week;
- many locked meals;
- multiple dislikes;
- restrictions;
- sparse pantry;
- repetitive meal library;
- leftovers/fallbacks;
- intentionally open nights.

Invariants/property tests:
- known hard restriction never selected;
- locked meal never moved;
- same input+algorithm version => same deterministic output;
- adding a hard constraint cannot make a previously infeasible candidate feasible;
- no negative shopping quantity;
- historical planner records not mutated by later planning.

Do not overfit to one exact plan when multiple answers are equally valid. Assert invariants/quality thresholds.

---

# Privacy-safe metrics

Provide hooks/events for:
- planner run/version;
- cycle covered;
- auto-resolved slot count;
- explicit decision count;
- correction/swap count;
- planner duration bucket;
- coarse reason-code frequency.

Never export raw:
- names;
- restrictions/allergies;
- recipe text;
- pantry contents;
- private notes;
- exact private schedules.

Time-in-app is an anti-metric.

---

# What NOT to build

Reject scope creep:
- Family Seasons;
- doctor/school/sports management;
- generalized household dashboard;
- global attention broker;
- calendar controller;
- generic agent framework;
- generic policy language;
- universal ontology;
- CP-SAT integration;
- cloud sync engine;
- cross-household recommendation training.

Preserve extension seams only.

---

# Deliverables from this coding session

At minimum produce:

1. `docs/V3_MIGRATION_PLAN.md`
2. working Rust/Flutter bridge spike
3. pinned/documented toolchain/build commands
4. initial Rust workspace/module boundaries
5. SQLite migration foundation
6. tests for first migrated domain primitive
7. README architecture update
8. `docs/V3_IMPLEMENTATION_STATUS.md` recording:
   - complete;
   - in progress;
   - deferred;
   - blockers;
   - deviations from PRD + rationale.

Do not claim the pivot complete until Android+iOS packaging is verified and Rust owns real domain state.

---

# Decision rule when uncertain

- clever universal abstraction vs simplest correct Kimatta abstraction with an extension seam → choose the latter;
- faster UI-coupled business logic vs clean Rust domain boundary → choose the clean boundary when it concerns durable control logic;
- sophisticated optimizer vs better state/constraint correctness → improve state/constraints first;
- ask user for more data vs safely proceed without it → safely proceed without it;
- missing information materially changes correctness → ask at the moment the value is obvious;
- platform adapter easier/safer in Dart/Swift/Kotlin → use that adapter and normalize into Rust.

## Glossary

**FFI:** Foreign Function Interface; Dart calling a compiled Rust library through generated bridge code.

**CP-SAT:** Google OR-Tools discrete constraint solver. Powerful for complex schedules/assignments; intentionally not MVP.

**Beam search:** bounded sequence search retaining the best partial plans.

**Lexicographic optimization:** higher-priority requirements cannot be traded away for lower-priority improvements.

Proceed by inspecting the repository first. Do not ask the user to restate anything present in `PRD_v3.md` or `HOUSEHOLD_CONTROL_PRINCIPLES.md`.
