# MVP Invariants

Every MVP card must preserve these constraints. A conflict stops the task and returns it to planning.

1. Household ownership is the core data boundary; solo use is a one-person household.
2. Ingredients remain structured while original quantity/unit text is retained for display and recovery.
3. Unit conversion and shopping aggregation are conservative: uncertain or incompatible lines stay separate.
4. Planning uses real dates and a household cycle, not a hard-coded week.
5. A planned meal occurrence supports one or more recipe components and optional serving scale per component.
6. Pantry is optional binary have/don't-have state, never a required quantified inventory audit.
7. The core recipe → plan → pantry-aware list → shopping loop remains usable offline with deliberate cache/prefetch and recovery behavior.
8. First launch is anonymous-first. Any upgrade preserves data, and data-loss risk is stated honestly.
9. Core planning has no recurring LLM dependency.
10. Restriction handling warns from known structured data, exposes uncertainty, and never claims a recipe is “safe.”
11. Public sharing uses a deliberately projected public representation with opaque identifiers, revocation, and no household/private leakage.
12. Imported/private content and public content have distinct rights and publication rules.
13. Analytics and diagnostics exclude sensitive recipe, restriction, household, and free-text content by default.
14. Android is the MVP and initial-launch platform. iOS is post-launch and must not block MVP acceptance.
15. No task implicitly activates production, paid services, credentials, DNS, signing, or store submission.
16. Future-phase architecture must not leak into the MVP without measured need and an explicit roadmap decision.
