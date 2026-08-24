# MVP-001 — Clean-room scaffold

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Done |
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
- Generate the temporary development identifiers DEC-001 §1 specifies; production identity is DEC-004's. Do not scaffold web/desktop/iOS unless DEC-001 explicitly requires it.

## Scope

- Capture rollback evidence, generate the clean scaffold, replace only approved paths, configure baseline analysis, and add a minimal smoke test.

## Non-goals

- Domain architecture, Firebase, features, visual design, CI, signing, or migration.

## Decision gates

- Stop if DEC-001 identifiers/platforms or PRE-001 command contract are unresolved.

## Acceptance criteria

- **AC-1:** Only the approved preserve set and new scaffold remain.
- **AC-2:** Product/package identifiers match DEC-001: `applicationId dev.mealmate.temp`, Dart package `meal_mate`, Android manifest label `MealMate (dev)`, `minSdk 24`; no `com.example` placeholder remains, and the dev identity is documented as temporary pending DEC-004.
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

## Verification record — 2026-08-22

All four acceptance criteria `PASS`. AC-3 was initially held at `NOT VERIFIED` because a
permission hook denied every read of the APK path in both the implementer and verifier
sessions; the owner confirmed the artifact directly on 2026-08-22, closing it. The
condensed row is in `docs/ROADMAP.md`'s Evidence log; this is the detailed record.

| Criterion | Result | Evidence |
|---|---|---|
| AC-1 | **PASS** | `git diff --stat HEAD` = 124 files changed, 257 insertions, 4124 deletions. `ios/ macos/ windows/ linux/ web/` deleted in full; `lib/models/recipe.dart`, `lib/screens/recipe_management/recipe_form_screen.dart`, `test/models/recipe_test.dart`, `android/app/src/main/kotlin/com/example/meal_mate/MainActivity.kt` and the three Groovy `*.gradle` files deleted. Verifier's independent depth-2 sweep found only the 9-entry surviving set plus stock scaffold output and pre-approved regenerated residue; no empty orphan package directory. |
| AC-2 | **PASS** | `applicationId = "dev.mealmate.temp"`, `namespace = "dev.mealmate.temp"`, `package dev.mealmate.temp`, `android:label="MealMate (dev)"`, `name: meal_mate`, explicit `minSdk = 24` with `targetSdk`/`compileSdk` still inheriting `flutter.*` per D-020. Zero `com.example` on the code surface; the only two occurrences repo-wide are governance prose in this card (AC-2 text) and `DEC-001` (rejected alternatives). Temporary-pending-`DEC-004` documented in `README.md`, `pubspec.yaml`, and the `build.gradle.kts` comment. |
| AC-3 | **PASS** | `dart format` clean, `flutter analyze` no issues, `flutter test` 1/1 — all passed in the implementer session **and** reran clean in the verifier's independent session. `flutter build apk --debug` reported `✓ Built build/app/outputs/flutter-apk/app-debug.apk`, but every direct read of that path (`ls`, `stat`, `find`) was denied by a permission hook in **both** sessions. Neither party worked around the denial, so AC-3 was first recorded `NOT VERIFIED`. Owner confirmed the artifact directly on 2026-08-22: `-rw-r--r-- 1 davidlinux davidlinux 150419440 Aug 22 22:40 build/app/outputs/flutter-apk/app-debug.apk` (150 MB, matching PRE-001's debug APK size). AC-3 closed **PASS**. |
| AC-4 | **PASS** | Independent fresh-context verifier: no legacy Dart, schema, or test survives; no non-Android platform tree exists; `.metadata` `migration.platforms` lists only `root` and `android`; no Groovy Gradle file remains; every scaffold config byte-diffs to stock Flutter 3.47.1 apart from the three intended identity edits. |

**AC-3 closed 2026-08-22** by owner confirmation of the APK artifact. Card moved
`Verify → Done`; Evidence-log row written once; `DEC-002` promoted to `Ready`.

**Carried forward:** the permission hook that blocks reads of `build/app/outputs/flutter-apk/`
will block the same evidence step on any later card that builds an APK (`MVP-003`, `MVP-022`).
Worth granting once rather than re-deriving the workaround each time.

**Rollback available:** `/home/davidlinux/mvp001_rollback_2026-08-22/` — `worktree.tar.gz`
(835K, 532 `.git` entries), `HEAD.sha` (`9eb5c91`), `uncommitted.diff`, `manifest-before.txt`.
Restore was rehearsed before the destructive step and verified byte-faithful. Nothing has been
committed or pushed.
