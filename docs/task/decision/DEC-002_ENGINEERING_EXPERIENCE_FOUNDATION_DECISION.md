# DEC-002 — Engineering-experience foundation

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | Verify |
| Type | Decision |
| Workstream | Engineering foundation |
| Depends on | MVP-001 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | After MVP-001 is Done |
| Recommended workflow | `deep-options`; then `grill-me` only for unresolved owner tradeoffs |
| External actions | None |

## Outcome and user value

Choose the smallest maintainable Flutter/Dart foundation that makes behavior testable and future cards predictable without building speculative architecture.

## Authoritative sources

- `docs/PRD_v2.md` §§14, 19–21
- `docs/ROADMAP.md` D-002, D-005, D-012, D-017

## Locked constraints

- Keep domain rules independent from Flutter widgets and Firebase details.
- Use repository/service interfaces and dependency injection only where they create a real test seam.
- Static analysis, formatting, unit/widget tests, and Android build checks must be reproducible locally.
- Select maintained packages compatible with the resolved SDK; do not copy the legacy dependency set.

## Decisions to resolve

1. State management/navigation/dependency-injection choices, including the viable “minimal Flutter primitives” option.
2. Feature/module layout and generated-code policy.
3. Exact formatting, analysis, test, coverage, and CI command contract.
4. Lightweight architecture-dependency enforcement, if its value exceeds its maintenance cost.

## Options and tradeoffs

- Minimal SDK primitives: least machinery, but conventions may drift as features grow.
- One cohesive maintained state/routing stack: clearer scaling and tests, with package coupling.
- Heavier layered framework/code generation: consistency at scale, but disproportionate MVP complexity.
- Hybrid: thin domain/repository boundaries plus a small UI state/router choice; recommended unless repository evidence contradicts it.

## Completion criteria

- Decisions include versions/constraints, rationale, rejected alternatives, test implications, and reversal cost.
- Commands are Flutter/Dart-specific and runnable in the project environment.
- Record the contract in the roadmap; MVP-002 may then become Ready.

## Resolution (2026-08-23)

Resolved via `deep-options` against the current clean-room scaffold (Flutter 3.47.1 / Dart 3.13.1, one file in `lib/`, stock `flutter_lints`, no state/nav/DI/codegen chosen yet). Owner selected among presented options for each of the four decisions below.

### 1. State management, navigation, dependency injection

**Chosen:** Hybrid. Plain-Dart domain/repository interfaces, injected via constructor. `flutter_riverpod` (non-codegen variant, not `riverpod_generator`) for UI state; Riverpod provider overrides double as the DI/test seam. `go_router` for declarative, URI-based navigation.

- **Versions/constraints:** current stable majors of `flutter_riverpod` and `go_router` resolved against Dart SDK `^3.13.1` at MVP-002 implementation time; not pinned in this decision.
- **Rationale:** matches this card's own recommended hybrid; provider overrides satisfy the locked constraint "DI only where it creates a real test seam" without a separate service locator; `go_router`'s URI-based routing is concretely needed for MVP-020/021 Android App Links, where a framework contract for malformed/unknown routes beats handwritten `Navigator 1.0` logic on a security-relevant path.
- **Rejected alternatives:** minimal Flutter primitives (`setState`/`ChangeNotifier`/`InheritedWidget`/`Navigator 1.0`) — this card's own text flags convention drift as features grow, and deep-link parsing for MVP-020/021 becomes fully handwritten exactly where correctness matters; Bloc + `get_it` + `injectable` + `freezed` — codegen and a full service-locator layer exceed the locked DI constraint and this card's own "disproportionate MVP complexity" warning.
- **Test implications:** screens/widgets test via `ProviderScope` overrides; navigation tests via `go_router`'s test helpers; domain/repository tests stay framework-free.
- **Reversal cost:** Medium — confined to the presentation layer; domain/data are unaffected, but every screen consuming a provider or the router would need rewriting.

### 2. Feature/module layout and generated-code policy

**Chosen:** Feature-first with a thin shared core. `lib/domain/` (pure-Dart entities + repository interfaces, no Flutter/Firebase imports), `lib/data/` (Firebase-backed repository implementations), `lib/features/<feature>/` (presentation, one folder per feature area — e.g. `recipes`, `planner`, `pantry`, `shopping_list`, `sharing`). No codegen (`freezed`/`json_serializable`) — hand-written immutable classes with `copyWith`.

- **Versions/constraints:** none beyond decision 1's packages.
- **Rationale:** maps 1:1 onto PRD §9 entities and the existing MVP card boundaries (MVP-007 recipes, MVP-012 planner, MVP-014 pantry, MVP-016 shopping, ...), minimizing folder-to-card translation cost; MVP entities are small enough that hand-written `copyWith` is less machinery than a `build_runner` step.
- **Rejected alternatives:** layer-first (`lib/models`, `lib/repositories`, `lib/screens`, `lib/widgets`) — fine at today's single-file size, but mixes unrelated features together as MVP-007 through MVP-016 land, with nothing enforcing a feature boundary; full codegen stack — disproportionate boilerplate machinery for MVP-sized entities and conflicts with "do not copy the legacy dependency set."
- **Test implications:** domain tests mirror `lib/domain` under `test/domain`; feature widget tests live under `test/features/<feature>`.
- **Reversal cost:** Low — folder reorganization only, no behavior change; codegen can be adopted later per-entity without a repo-wide rewrite.

### 3. Formatting, analysis, test, coverage, and CI command contract

**Chosen commands** (extends the PRE-001 contract):

```bash
dart format --output=none --set-exit-if-changed .
flutter analyze
flutter test --coverage
flutter build apk --debug
```

**Coverage — tiered gate:** 100% line coverage required on `lib/domain/` and `lib/data/` (checked via `lcov`/`genhtml` filtered to those paths); `lib/features/` presentation coverage is generated and reported but not gated. `package:glados` (property-based testing; verified actively maintained on pub.dev) is added as a dev-dependency for the domain functions carrying the highest correctness risk per `MVP_INVARIANTS.md` #3/#4/#10 — unit conversion, shopping aggregation, restriction matching, planning-cycle date math — to generate adversarial/boundary inputs rather than relying only on hand-picked cases.

- **Versions/constraints:** `package:glados`, current stable version compatible with Dart `^3.13.1`, dev-dependency only.
- **Rationale:** a blanket 100% gate on `build()` methods and generated boilerplate buys little and produces low-signal tests; scoping the hard gate to domain/data logic — exactly where the MVP invariants live — keeps the number meaningful. Property-based testing closes the gap line coverage cannot: a covered line does not guarantee a meaningful assertion.
- **Rejected alternatives:** blanket 100% repo-wide coverage (the original ask, taken literally) — high authoring tax across upcoming MVP cards for a number that still permits a test that hits every line and asserts nothing; `package:mutation_test` now (verified maintained, last published Feb 2026) — reruns the full suite per mutant, a real CI/local time cost not justified before the domain layer has substantial logic; noted as the upgrade path once MVP-007/012/014/015 land.
- **Test implications:** every domain/data PR must maintain 100% coverage on those directories; `glados` tests are added incrementally as each risk-bearing domain function is implemented, starting with MVP-002's identity/date/pantry/entitlement logic.
- **Reversal cost:** Low — the coverage gate is a CI check, not a code dependency, and can be retargeted without touching application code; `glados` tests can be deleted without affecting production code.

### 4. Lightweight architecture-dependency enforcement

**Chosen:** a small script (a few lines), run alongside `flutter analyze` in the command contract, that fails if any file under `lib/domain/` imports `package:flutter` or `package:cloud_firestore`/`package:firebase_*`.

- **Versions/constraints:** none — no new package dependency.
- **Rationale:** checks the one rule the locked constraints explicitly name (domain independent of Flutter/Firebase) at negligible cost, satisfying this card's own "value must exceed maintenance cost" test — currently the only layering rule that exists.
- **Rejected alternatives:** `clean_code_lints` and `import_lint` (both verified real pub.dev packages purpose-built for this — AST-precise, catch transitive leakage the script's string match cannot) — not justified yet for a single rule; `clean_code_lints` also rides on the newer `analysis_server_plugin` system. No enforcement — under-protects a constraint this card itself calls load-bearing.
- **Test implications:** the script runs alongside `flutter analyze`; no test-code changes.
- **Reversal cost:** Low — a standalone script, trivially removed or replaced by `clean_code_lints`/`import_lint` the moment a second layering rule is needed.
