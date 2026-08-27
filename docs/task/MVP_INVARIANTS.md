# MVP Invariants

Every MVP card must preserve these constraints. A conflict stops the task and returns it to planning. Items 17–21 apply from PRD v3 (D-028) onward. Item 7 was restated from PRD v3 under D-036.

1. Household ownership is the core data boundary; solo use is a one-person household.
2. Ingredients remain structured while original quantity/unit text is retained for display and recovery.
3. Unit conversion and shopping aggregation are conservative: uncertain or incompatible lines stay separate.
4. Planning uses real dates and a household cycle, not a hard-coded week.
5. A planned meal occurrence supports one or more recipe components and optional serving scale per component.
6. Pantry is optional binary have/don't-have state, never a required quantified inventory audit.
7. The core recipe → plan → pantry-aware list → shopping loop remains usable offline because Rust-owned SQLite is the local source of truth, with durability, recovery and backup behavior rather than a cache layered over a remote store (PRD v3 §6.4–6.5, §12; restated under D-036).
8. First launch is anonymous-first. Any upgrade preserves data, and data-loss risk is stated honestly.
9. Core planning has no recurring LLM dependency.
10. Restriction handling warns from known structured data, exposes uncertainty, and never claims a recipe is “safe.”
11. Public sharing uses a deliberately projected public representation with opaque identifiers, revocation, and no household/private leakage.
12. Imported/private content and public content have distinct rights and publication rules.
13. Analytics and diagnostics exclude sensitive recipe, restriction, household, and free-text content by default.
14. Android is the MVP and initial-launch platform. iOS is post-launch and must not block MVP acceptance.
15. No task implicitly activates production, paid services, credentials, DNS, signing, or store submission.
16. Future-phase architecture must not leak into the MVP without measured need and an explicit roadmap decision.
17. Rust-owned SQLite is the authoritative local source of truth for durable household/food state; Flutter never separately mutates those tables (PRD v3 §12–13).
18. Hard restrictions, explicit hard vetoes, and locks are Tier-0 constraints, never score weights; a lower tier never compensates for a higher-tier failure (PRD v3 §9.7).
19. Planned is not cooked; coverage is relative to known information and never claims safety, exact stock, or personalization the model cannot support (PRD v3 §2.2, §10).
20. The planner is deterministic for the same input snapshot and algorithm version, records reason codes and version, and never mutates historical planner records (PRD v3 §7.7, §9.6, §18).
21. The Dart↔Rust bridge is coarse-grained: service-level DTO calls; generated bindings are never hand-edited; no SQLite handles or per-field calls cross it (PRD v3 §6.3).
