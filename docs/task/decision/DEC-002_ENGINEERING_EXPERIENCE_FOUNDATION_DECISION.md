# DEC-002 — Engineering-experience foundation

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | Draft |
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
