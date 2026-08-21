# MVP-001 — Clean-room scaffold

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation |
| Workstream | Foundation |
| Depends on | DEC-001, PRE-001 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | Destructive replacement requires explicit plan approval; no commit/push |

## Outcome and user value

Replace the disposable legacy app with a current, minimal Android Flutter scaffold whose identity is stable and whose baseline is trustworthy.

## Authoritative sources

- `docs/PRD_v2.md` §§14.1, 19; `docs/ROADMAP.md` D-004, D-007, D-015; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Scaffold in a temporary directory first; use an explicit replacement allowlist.
- Preserve `.git`, `docs`, `LICENSE`, and only DEC-001-approved product metadata.
- Preserve no legacy code, schema, tests, generated platform tree, or migration path.
- Generate durable identifiers from DEC-001; do not scaffold web/desktop/iOS unless DEC-001 explicitly requires it.

## Scope

- Capture rollback evidence, generate the clean scaffold, replace only approved paths, configure baseline analysis, and add a minimal smoke test.

## Non-goals

- Domain architecture, Firebase, features, visual design, CI, signing, or migration.

## Decision gates

- Stop if DEC-001 identifiers/platforms or PRE-001 command contract are unresolved.

## Acceptance criteria

- **AC-1:** Only the approved preserve set and new scaffold remain.
- **AC-2:** Product/package identifiers match DEC-001: `applicationId dev.mealmate.temp`, Dart package `meal_mate`, `minSdk 24`; no `com.example` placeholder remains, and the dev identity is documented as temporary pending DEC-004.
- **AC-3:** Format, analyze, tests, and Android debug build pass.
- **AC-4:** A fresh-context review confirms no legacy implementation or unauthorized platform surface survived.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Before/after manifest and Git diff |
| AC-2 | Targeted identifier search and Android manifest/build inspection |
| AC-3 | Command outputs and artifact path |
| AC-4 | Independent read-only review report |

## Stop/failure conditions

- Stop on an untracked-file conflict, unclear deletion target, failed rollback preparation, or unauthorized external action. After two remediation cycles, return to planning.

## Handoff

Record evidence/status in `docs/ROADMAP.md`; then promote DEC-002.
