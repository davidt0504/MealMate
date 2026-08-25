# Household Control System — Governing Principles and Research Foundation

**Status:** Foundational product/engineering constitution  
**Date:** 2026-08-24  
**Applies to:** Kimatta and future applications/controllers built on the Household Control System

This document preserves the ideas that should survive individual apps, UI redesigns, team changes, and future conversations.

---

## 1. Foundational thesis: from attention economy to outcome economy

Most consumer software has been economically and technically optimized around acquiring attention: sessions, clicks, notifications, watch time, engagement, and conversion.

This project deliberately optimizes a different resource:

> **Useful desired outcomes achieved or maintained per unit of administrative human attention.**

Conceptually:

\[
\max \; U(\text{useful outcomes}) + U(\text{meaningful human agency})
\]

subject to minimizing:

\[
C_{attention}+C_{risk}+C_{privacy}+C_{avoidable\ effort}
\]

The system is **not** trying to minimize every kind of human attention. Attention spent on relationships, creativity, enjoyment, choosing goals, and meaningful participation is often intrinsically valuable.

The target is administrative attention:
- remembering;
- tracking;
- reconciling;
- checking;
- coordinating;
- repeatedly making low-value decisions;
- reconstructing state software could have retained.

> **Automate administration, not meaning.**

---

## 2. Research basis

Household labor includes a cognitive dimension beyond physical execution: anticipating needs, identifying options, deciding, assigning, and monitoring. Research including Daminger's cognitive-labor framework and later empirical work on household cognitive labor supports treating this hidden planning burden as a distinct design target.

Research on **intention offloading** shows that people use external aids such as reminders/calendars to reduce prospective-memory demands. The design implication is not merely “send reminders”; it is that software can become a trustworthy external memory, which raises the importance of reliability and calibrated confidence.

Research on notification/interruption management indicates that interruption timing and batching can affect perceived attention, control, stress, mood, and productivity. Therefore notification count/timing is not neutral infrastructure—it is part of the cognitive cost function.

These studies do not prove that a generalized household controller will work. They support the underlying claims that cognitive administration is real, externalized memory can help, and interruptions have measurable cost.

---

## 3. The control-system model

A bounded household domain can be treated as a feedback-control problem:

```text
Desired state
     │
     ▼
Outcome monitor ←── believed/current state
     │
     ▼
Candidate actions
     │
     ▼
Domain controller / optimizer
     │
     ▼
Act / prepare / ask / wait
     │
     ▼
Observed outcome/evidence
     └──────────────→ state update
```

The controller repeatedly:
1. estimates current state;
2. compares it with desired state;
3. identifies unresolved coverage/error;
4. generates feasible candidate actions;
5. applies hard constraints;
6. optimizes among remaining actions;
7. decides whether human attention is justified;
8. stages/executes permitted work;
9. observes/verifies the result;
10. replans when reality changes.

This borrows from Model Predictive/Receding-Horizon Control. Household life is not a deterministic physical plant: state is partial, humans change goals, social constraints matter, and many actions are uncertain. The analogy is architectural, not literal.

---

## 4. Never optimize “life” as one scalar

Do not build a universal life score.

The system MUST distinguish hard constraints from preferences/objectives.

### Hard constraints
Examples:
- known safety restrictions;
- explicit non-negotiable commitments;
- authorization/consent boundaries;
- legal requirements;
- explicit “never” rules.

### Primary objectives
Examples:
- outcome coverage;
- feasibility;
- household acceptance.

### Lower objectives
Examples:
- convenience;
- cost;
- variety;
- reduced travel;
- reduced attention.

A lower-level benefit can never compensate for violating a hard constraint.

Use **lexicographic/tiered optimization** before a weighted utility function. Weighted scores are acceptable inside a tier when normalized, inspectable, and empirically evaluated.

---

## 5. State quality before optimizer quality

A solver can perfectly optimize the wrong model.

Development priority:

\[
\boxed{
\text{State correctness}
>
\text{Constraint completeness}
>
\text{Candidate-action completeness}
>
\text{Optimization sophistication}
}
\]

Do not celebrate a marginally better objective score if:
- an event is missing;
- a restriction is stale;
- a fallback action is absent;
- the system cannot verify whether the previous action succeeded.

Every controller needs a **model-sufficiency test** before declaring an outcome covered.

---

## 6. Represent uncertainty; do not manufacture certainty

Household state is partially observed.

Examples:
- planned meal may not have been cooked;
- pantry presence can be stale;
- a child's size changes;
- event dates may be extracted from untrusted text;
- preferences can be inferred weakly from behavior.

Initial systems should use legible evidence classes such as:
- explicit user statement;
- verified integration;
- high-confidence inference;
- low-confidence inference;
- unknown/contradicted.

Every important inference should retain provenance/staleness where relevant.

Numerical probability is useful only when calibrated. Do not turn arbitrary scores into fake probabilities.

---

## 7. Candidate-action completeness is first-class

Optimization only chooses among actions represented in the model.

A transport controller that knows only “Parent A drives / Parent B drives” cannot discover Grandma, carpool, skipping an event, or rescheduling unless those actions are modeled.

Each domain owns **candidate-action generation** and must distinguish:

- possible action;
- available action;
- authorized action;
- proposed action;
- selected action;
- executed action;
- verified outcome.

LLMs may suggest candidate possibilities from messy context but cannot grant authority or make an unverified action real.

---

## 8. Human attention is an expensive control resource

When uncertain, software can broadly:
1. observe more;
2. wait;
3. act within authority;
4. ask a human.

Asking has cost:
- interruption;
- foreground seconds;
- decision complexity;
- context switching;
- cognitive/emotional burden.

Conceptual rule:

\[
\text{Ask if Value of Information} > \text{Attention Cost}
\]

Early systems should use reasoned ordinal bands/thresholds rather than pretending exact VOI is known.

---

## 9. Controllers request attention; they do not own attention

A domain controller produces an `AttentionRequest` with:
- urgency;
- deadline;
- expected benefit;
- estimated effort;
- options;
- reason codes.

The controller should not independently decide “send a push notification now.”

A future Attention Governor can suppress, defer, batch, surface peripherally, or interrupt.

Kimatta may initially implement a local simplified version, but the architectural distinction must remain.

---

## 10. Optimize information gain per interaction

The goal is not zero interaction.

A five-second answer that removes dozens of future decisions is excellent.

Prefer questions with high reusable information value.

Bad:
> Complete a 25-question onboarding survey.

Better:
> Wednesday became busy. Should Wednesday dinners usually stay under 25 minutes?

One timely answer becomes a durable policy.

**Progressive profiling rule:**

> Never ask for data merely because it might someday be useful. Ask when the immediate payoff is apparent.

---

## 11. Normal use should train the system

Do not create maintenance whose purpose is feeding the algorithm.

Evidence should emerge from:
- accepted suggestion;
- manual choice;
- swap;
- veto;
- repeated use;
- completed action;
- correction;
- user-created fallback.

Explicit tuning can exist but should be voluntary and short.

---

## 12. Algorithm hierarchy

Use the least complex reliable method:

1. deterministic rules/state machines;
2. heuristic/weighted scoring;
3. bounded combinatorial search;
4. lightweight/local statistical learning;
5. specialized exact solvers when justified;
6. local generative models where useful;
7. cloud LLMs only at messy/unstructured/generative boundaries.

LLMs are useful for:
- parsing messy text;
- OCR cleanup/semantic extraction;
- natural-language → candidate structured state;
- creative generation.

LLMs must not:
- silently override hard constraints;
- become the recurring trusted controller by default;
- fabricate authoritative state;
- grant themselves authority;
- generate post-hoc explanations disconnected from the actual decision path.

After LLM ingestion/generation, validate and convert output into structured state.

---

## 13. Algorithm toolbox

### Rules/state machines
Use for hard constraints, authorization, status transitions, expiry/staleness, deterministic derived state.

Advantages: predictable, testable, auditable.

### Weighted scoring
Use inside one priority tier for preference, recency, convenience, variety, etc.

Requirements:
- normalized components;
- reason codes;
- never hide hard constraints in weights.

### Beam search
Good for small sequence decisions such as early Kimatta weekly planning.

Advantages:
- bounded runtime;
- easy Rust implementation;
- sequence-aware;
- deterministic;
- replaceable.

### MILP — Mixed-Integer Linear Programming
Potentially useful when relationships are naturally linear/discrete: assignments, resource allocation, some scheduling formulations.

Rust ecosystem options include `good_lp` with solver backends and HiGHS bindings.

Costs: modeling/linearization complexity and native solver packaging.

### CP-SAT / constraint programming
Google OR-Tools CP-SAT is powerful for integer/discrete constraint problems such as scheduling, optional tasks, precedence, no-overlap, and capacity.

Do not introduce it merely because it sounds sophisticated. OR-Tools has no official Rust API, so a Rust/mobile system adds C++/binding/package complexity.

Potentially excellent for a future ScheduleController if problem complexity justifies it.

### Bayesian updating
Potential future use for uncertain preference strength, return intervals, staleness likelihood, success probability.

Calibration is mandatory before presenting values as probabilities.

### Contextual bandits
Potentially useful for **safe, low-stakes exploration among already acceptable actions**.

Never explore across safety/authorization boundaries.

### Receding-horizon planning / MPC concept
Plan a bounded future horizon, execute only what is appropriate now, observe, and replan.

Add stability/churn penalties so the controller does not repeatedly disturb accepted human plans.

### Value of Information
Use conceptually to decide whether missing information is worth human attention.

### Robust/stochastic planning
When durations or availability are uncertain, prefer slack and robustness over brittle nominal optima. Future controllers may use learned distributions/percentiles or formal temporal-uncertainty methods.

---

## 14. Stability is an objective

Repeated reoptimization can be worse than a slightly lower-scoring stable plan.

Use:
- locks;
- commitment horizons;
- churn penalties;
- minimum-improvement thresholds.

Near-term accepted plans require stronger justification to change than distant plans.

Humans need predictability.

---

## 15. Robustness beats tight optimization

Do not optimize resources to 100% nominal utilization.

A plan that only works if traffic is perfect, practice ends exactly on time, and dinner takes precisely its nominal duration is not good household planning.

Model slack/fragility and prefer solutions that tolerate ordinary variation.

---

## 16. Progressive autonomy

Automation is a ladder, not an on/off switch:

0. **Observe** — update state.
1. **Advise** — recommend.
2. **Prepare** — draft/assemble.
3. **Stage** — create reversible/tentative action.
4. **Execute** — perform pre-authorized reversible action.
5. **Commit** — consequential/irreversible action.

Authority is scoped per domain/action/household.

High-consequence domains may never permit full autonomous commitment without explicit consent.

The system earns authority through reliability.

---

## 17. External actions require transactional thinking

Real-world services do not share one database transaction.

Important actions should model:
- preconditions;
- execution;
- verification;
- rollback/compensation where possible.

Prefer:

> prepare → verify → commit

Example: do not cancel an old appointment before a replacement is actually secured unless the human explicitly accepts that risk.

---

## 18. Outcome verification matters

“Planned” is not “done.”

Represent stages such as:
- desired;
- planned;
- initiated;
- completed;
- verified;
- failed/corrected.

Without verification, the world model accumulates fictional successes.

Evidence strength should influence later planning.

---

## 19. Multi-person households are multi-principal systems

Do not optimize as if one person represents the household.

Model:
- member-specific preferences;
- roles/responsibilities;
- authority;
- availability;
- burden/fairness over time.

Do not route every unresolved question to the person who historically responds fastest; that can preserve or amplify invisible cognitive-load inequality.

Fairness is contextual and not equivalent to 50/50 task counts.

---

## 20. Privacy is architecture, not marketing

A unified household controller can become one of the most sensitive datasets a family owns.

Principles:
- local-first domain/world state;
- minimum necessary cloud disclosure;
- explicit user-controlled sync;
- no behavioral advertising;
- no household-data sale;
- encrypted transport;
- secure key/credential storage;
- deletion/export;
- provenance;
- external content treated as untrusted.

The household should remain the customer, not the inventory.

---

## 21. Business incentives must align with attention reduction

Reject revenue models that economically reward more user attention.

No ads.

Subscription or fixed pricing is compatible with an outcome economy because value need not be proportional to minutes in the app.

A customer who barely opens the product because delegated work is handled reliably can be a **high-value successful customer**.

Potential monetization axes:
- responsibility/capability transferred to software;
- household reliability/sync;
- variable-cost external/generative services.

Not:
- number of notifications;
- amount of feed content;
- artificially metered deterministic intelligence.

---

## 22. Product metrics

Do not optimize DAU/MAU or session length as product objectives.

Track two independent dimensions.

### Outcome quality
- coverage;
- success;
- hard-constraint/error rate;
- corrections;
- regret where measurable;
- robustness;
- stability.

### Administrative attention
- active seconds;
- decisions;
- interruptions;
- context switches where measurable;
- questions;
- manual maintenance.

Goal: improve the **Pareto frontier**—same/better outcomes for less attention, or materially better outcomes without disproportionate attention.

Business retention/revenue still matter, but must not reverse the product objective.

---

## 23. “Nothing needs you” is a valid UI state

Do not populate emptiness merely to create engagement.

A mature household surface should be able to say:

> **Nothing needs you right now.**

When intervention is necessary:

> **2 decisions need you — ~20 seconds.**

The default interface is an exception console, not a feed.

---

## 24. Domain controllers, not one giant brain

Use hierarchical modularity.

Examples:
- `FoodController`
- future `ScheduleController`
- future `ReadinessController`

Each controller owns:
- domain model;
- candidate-action generation;
- domain constraints;
- local optimization;
- outcome assessment;
- action proposals;
- attention requests.

A future supervisor coordinates cross-domain conflicts.

Do not create one universal optimizer.

---

## 25. Kernel minimalism

Shared kernel may contain only genuinely cross-domain primitives:
- household/member identity;
- policy envelope;
- control-relevant evidence/provenance;
- outcome assessment;
- action proposal;
- attention request;
- authorization primitives when actually needed;
- outcome/audit ledger.

Avoid:
- universal semantic graph;
- universal policy DSL;
- generic “life entity” ontology;
- POMDP engine;
- generic optimizer;
- family-super-app dashboard before multiple real domains exist.

> **Second-domain evidence is required before extracting new universal abstractions.**

---

## 26. Rust/implementation principles

Current architectural direction:
- Rust core/domain/persistence;
- Flutter consumer UI;
- generated Dart↔Rust FFI boundary;
- SQLite local source of truth.

Rust is selected for safety, portability, deterministic systems logic, optimization headroom, and a long-lived reusable core—not for cleverness or benchmark vanity.

Avoid:
- unnecessary generics;
- macro-heavy internal DSLs;
- unnecessary async runtimes;
- unsafe application/domain logic.

If an OS/platform feature is easier/safer in Dart, Swift, or Kotlin, implement the adapter there and normalize its data into the kernel.

---

## 27. Explainability and reproducibility

Automated decisions should be explainable from real decision inputs.

Retain:
- algorithm version;
- input snapshot/version/hash;
- reason codes;
- selected candidate/action;
- corrections.

Examples:
- “quick fit for Wednesday”;
- “family favorite”;
- “not eaten recently”;
- “uses ingredients already planned.”

Never generate fictional post-hoc explanations.

---

## 28. Engineering/research discipline

Before adding sophistication:
1. identify concrete failure mode;
2. collect fixtures/data;
3. define metric;
4. benchmark current method;
5. compare alternative;
6. adopt only if improvement justifies complexity.

Do not add ML because it sounds advanced.  
Do not add a solver because the problem contains optimization.  
Do not move every adapter into Rust because Rust is the kernel language.  
Do not add an LLM because input is inconvenient.

---

## 29. Adversarial checklist for each new controller

Before implementation ask:

1. What exact desired state is maintained?
2. What is hard vs soft?
3. How is current state observed?
4. Which facts can be wrong/stale?
5. How is uncertainty represented?
6. What candidate actions exist?
7. What plausible actions are missing?
8. What authority is required?
9. How reversible is each action?
10. How is success verified?
11. How can the optimizer exploit a bad metric?
12. How can attention minimization hide important information?
13. How can the system unfairly dump work on one household member?
14. What happens offline?
15. What happens when cloud/AI providers fail?
16. Can the decision be reproduced/debugged?
17. What data leaves the device?
18. Is correction/override easy?
19. Does this feature create recurring maintenance?
20. Would the feature still be valuable if engagement metrics disappeared?

---

## 30. Anti-patterns

Reject by default:

- infinite feeds;
- streaks;
- engagement notifications;
- unnecessary daily summaries;
- gamification of household administration;
- “AI handled it” without structured verification;
- invisible safety assumptions;
- universal weighted life scores;
- fake probabilities;
- cloud-required core workflows;
- pooled behavioral training by default;
- hidden affiliate influence;
- silent irreversible actions;
- giant onboarding questionnaires;
- duplicated durable state across apps;
- framework-first abstractions without a second real domain.

---

## 31. North-star statement

> **Most software asks humans to operate it. We build software that operates on behalf of humans within explicit goals, constraints, and authority. It should observe enough of the user's world to maintain useful outcomes, perform or prepare routine work safely, and ask for human attention only when judgment, consent, uncertainty, or meaningful participation genuinely requires it.**

Guardrail:

> **Automate administration, not meaning.**

---

## 32. Bibliography

### Cognitive labor / attention / offloading
1. Aviv et al., *Cognitive household labor: gender disparities and consequences for maternal mental health and wellbeing*. https://doi.org/10.1007/s00737-024-01490-w
2. Daminger, *The Cognitive Dimension of Household Labor*. https://doi.org/10.1177/0003122419859007
3. Gilbert et al., *Outsourcing Memory to External Tools: A Review of “Intention Offloading”*. https://pubmed.ncbi.nlm.nih.gov/35789477/
4. Jones et al., prospective-memory interventions systematic review/meta-analysis. https://pubmed.ncbi.nlm.nih.gov/33393806/
5. Gilbert et al., *Optimal use of reminders: Metacognition, effort, and cognitive offloading*. https://pubmed.ncbi.nlm.nih.gov/31448938/
6. Fitz et al., *Batching smartphone notifications can improve well-being*. https://doi.org/10.1016/j.chb.2019.07.016

### Technical references
7. Rust release announcements: https://blog.rust-lang.org/releases/
8. Rust iOS targets: https://doc.rust-lang.org/stable/rustc/platform-support/apple-ios.html
9. Rust Android targets: https://doc.rust-lang.org/rustc/platform-support/android.html
10. Dart FFI: https://dart.dev/interop/c-interop
11. Dart build hooks/code assets: https://dart.dev/tools/hooks
12. flutter_rust_bridge: https://pub.dev/packages/flutter_rust_bridge
13. Flutter supported platforms: https://docs.flutter.dev/reference/supported-platforms
14. rusqlite: https://docs.rs/rusqlite/latest/rusqlite/
15. Jiff: https://docs.rs/jiff/latest/jiff/
16. Google OR-Tools: https://developers.google.com/optimization/
17. CP-SAT: https://developers.google.com/optimization/cp/cp_solver
18. OR-Tools installation/language support: https://developers.google.com/optimization/install/
19. good_lp: https://docs.rs/good_lp/latest/good_lp/
20. HiGHS Rust bindings: https://docs.rs/highs/latest/highs/

---

## 33. Interpretation rule

When this document conflicts with a local implementation convenience:

- safety beats convenience;
- correctness beats cleverness;
- state quality beats optimizer sophistication;
- user attention beats engagement;
- local reliability beats unnecessary cloud dependence;
- domain clarity beats premature universality;
- reversible delegated action beats flashy autonomy;
- observed value beats marketing claims.

If a future decision genuinely invalidates one of these principles, update this document explicitly with rationale rather than silently drifting away from it.
