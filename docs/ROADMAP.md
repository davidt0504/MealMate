# Meal Mate Roadmap

This is the durable operational source of truth after the MVP as well as during it. The PRD defines product intent; task cards define bounded work; this file alone records live task status, sequence, gates, decisions, blockers, and evidence. Status cells in task cards point here rather than duplicating this register.

## Current milestone

- **Milestone:** EMULATOR-PERSISTENCE-READY; DOMAIN-READY achieved 2026-08-27.
- **Current implementation task:** `MVP-003` — `Verify`; evidence contract satisfied, awaiting explicit owner approval.
- **Next implementation task:** `MVP-004` — `Draft`; blocked until MVP-003 reaches `Done` and the implementation lane clears.
- **Concurrent owner-paced work:** `DEC-004` — `In Progress`; it does not occupy the implementation lane, but it still directly gates `MVP-018`, `MVP-021`, and `MVP-022` and transitively gates `MVP-019` and `MVP-020`.
- **Platform:** Android MVP and initial launch. iOS is the first post-launch platform priority.

## Status contract

`Draft → Ready → In Progress → Verify → Done`. A material change to a card's sources, decisions, product scope, architecture, or declared dependencies (its `Depends on` cell) returns an affected card to Draft. A package added inside a card's stated scope is recorded in the card's evidence and is not a Draft event. This sentence is the single Draft-trigger rule (D-031); `docs/task/README.md` points here. `Ready` means only that the card is eligible to plan: it is the selected next implementation task and every direct dependency is Done. Execution additionally requires a current approved plan against current sources. `In Progress` occupies the single implementation lane. `Verify` means implementation is complete but final evidence or owner status approval is unresolved. Move to `Done` only when all required evidence is recorded as `PASS` (or the D-027 owner-acceptance exception is fully satisfied) and the owner explicitly approves that card's status transition. Approvals remain nontransitive. The mandatory planning, execution, and handoff checks are defined once in `docs/task/README.md`.

## Delivery gates

| Gate | Exit condition |
|---|---|
| DOMAIN-READY | DEC-001, PRE-001, MVP-001, PRE-002, DEC-005, and MVP-002 Done — met 2026-08-27 |
| EMULATOR-PERSISTENCE-READY | DEC-002 Done and MVP-003 through MVP-005 Done with on-device SQLite persistence evidence (re-derived after DEC-005, D-029) |
| LOCAL-CORE-LOOP-READY | MVP-006 through MVP-017 and MVP-023 through MVP-025 Done, including offline recovery |
| REAL-DEV-BACKEND-READY | DEC-003, MVP-018, and MVP-019 Done in a non-production project |
| SHARING-SECURITY-READY | MVP-020 and MVP-021 security, projection, and routing evidence passes |
| PRODUCTION-BETA-READY | DEC-004 Done and MVP-022 passes; production activation remains a separately authorized action |

## Task register

| ID | Status | Outcome | Depends on |
|---|---|---|---|
| DEC-001 | Done | Product/platform identity (name deferred to DEC-004) | — |
| PRE-001 | Done | Android toolchain readiness | — |
| MVP-001 | Done | Clean-room Flutter scaffold | DEC-001, PRE-001 |
| PRE-002 | Done | Rust/FRB toolchain and bridge spike | MVP-001 |
| DEC-005 | Done | Bridge backend and core-language commitment | PRE-002 |
| DEC-002 | Done | Engineering-experience foundation | MVP-001 |
| MVP-002 | Done | Rust household kernel and SQLite foundation | PRE-002, DEC-005 |
| MVP-003 | Verify | Android shell and navigation | MVP-002, DEC-002, DEC-005 |
| MVP-004 | Draft | Anonymous-first entry and household identity | MVP-002, MVP-003 |
| MVP-005 | Draft | Planning cycle and meal scope | MVP-002, MVP-004 |
| MVP-006 | Draft | Minimal onboarding/preferences | MVP-003, MVP-004, MVP-005 |
| MVP-007 | Draft | Ingredient and recipe foundation | MVP-002, MVP-004 |
| MVP-008 | Draft | Manual recipe CRUD | MVP-003, MVP-007 |
| MVP-009 | Draft | Restriction warnings/filtering | MVP-006, MVP-007, MVP-008 |
| MVP-010 | Draft | Recipe photos | MVP-004, MVP-008 |
| MVP-011 | Draft | Curated starter content | MVP-007, MVP-008, MVP-009 |
| MVP-012 | Draft | Planned meals and components | MVP-004, MVP-005, MVP-007 |
| MVP-013 | Draft | Planner experience | MVP-003, MVP-009, MVP-012 |
| MVP-014 | Draft | Binary, optional pantry | MVP-003, MVP-004, MVP-007 |
| MVP-015 | Draft | Conservative shopping aggregation | MVP-012, MVP-014 |
| MVP-016 | Draft | Shopping-list experience | MVP-003, MVP-004, MVP-015 |
| MVP-017 | Draft | Offline durability and recovery | MVP-008, MVP-009, MVP-013, MVP-014, MVP-016 |
| DEC-003 | Draft | Cloud/sharing release boundary | MVP-017 |
| DEC-004 | In Progress | Naming clearance and production identity | DEC-001 |
| MVP-018 | Draft | Real dev backend and durable auth | MVP-017, DEC-003, DEC-004 |
| MVP-019 | Draft | Privacy-safe observability | MVP-018 |
| MVP-020 | Draft | Share publication and web preview | MVP-009, MVP-018, DEC-003 |
| MVP-021 | Draft | Android App Links recipient flow | MVP-020, DEC-004 |
| MVP-022 | Draft | Android beta readiness | DEC-004, MVP-001–MVP-009, MVP-011–MVP-021, MVP-023, MVP-024, MVP-025, and MVP-010 unless explicitly cut; all delivery gates through SHARING-SECURITY-READY |
| MVP-023 | Draft | FoodController planner core | MVP-002, MVP-009, MVP-012, MVP-014 |
| MVP-024 | Draft | Cover My Week experience | MVP-013, MVP-015, MVP-023 |
| MVP-025 | Draft | Planner fixtures and invariant tests | MVP-023 |
| PRE-003 | Draft | iOS toolchain readiness (post-launch) | PRE-002, DEC-005 |
| OPT-001 | Draft | Structured URL recipe import | MVP-008 |

MVP-010 may be explicitly cut under the PRD's photo deferral rule, but must not silently disappear. OPT-001 has no dependency path into MVP completion.

### D-035 migration baseline for existing Done cards

The recursive dependency gate applies prospectively to transitions made under D-035. On 2026-08-26, the six cards already Done were audited once to establish the starting closure:

- `DEC-001`: completion criteria PASS — decisions 2–3 were resolved and decision 1 was explicitly descoped to `DEC-004`. Its `FAIL` clearance observations are the evidence supporting that decision, not failed decision-card completion criteria; its two historical `NOT VERIFIED` observations remain recorded under the pre-D-027 practice.
- `PRE-001`, `MVP-001`, `DEC-002`, and `DEC-005`: their required completion items are recorded as PASS in the evidence log or decision resolution.
- `PRE-002`: AC-1–AC-7 PASS; AC-8 is `NOT VERIFIED` under the fully recorded D-027 exception with `owner-accepted 2026-08-24` and discharge at `PRE-003`.

These six statuses are the grandfathered starting state, not a relaxation for future work. After D-035, a direct dependency counts as successfully Done only when its evidence row satisfies the current PASS/D-027 contract and its owner-approved Done transition is recorded. Because each future Done transition applies that same rule, checking direct dependencies proves the later transitive closure without re-reading every ancestor.

## MVP acceptance traceability

| PRD_v2 §24 item (historical) | Owning cards |
|---|---|
| 1 | MVP-004, MVP-006 |
| 2 | MVP-005, MVP-006, MVP-009 |
| 3 | MVP-007, MVP-008, MVP-011 |
| 4 | MVP-014 |
| 5 | MVP-005, MVP-012, MVP-013 |
| 6 | MVP-005, MVP-006, MVP-013 |
| 7 | MVP-015, MVP-016 |
| 8 | MVP-017 |
| 9 | MVP-020, MVP-021 |
| 10 | MVP-017 and `MVP_INVARIANTS.md` |
| 11 | MVP-019 |

The table above is historical (D-028). The authoritative acceptance mapping is PRD v3 §26, derived 2026-08-26 with the cards (D-034). This table is the single authority for §26 traceability; cards do not repeat it, and `MVP-022` AC-1 reads from it.

| PRD v3 §26 item | Owning cards |
|---|---|
| 1 — Flutter calls the Rust core in Android **and iOS** release pipelines | **Split.** Android half: PRE-002 (Done 2026-08-24). iOS half: PRE-003, post-launch under D-015 and invariant 14 — **not an MVP criterion** |
| 2 — Rust owns authoritative local food/control state in SQLite | MVP-002, MVP-005, MVP-007, MVP-012, MVP-014, MVP-015 |
| 3 — User can start without a mandatory durable account | MVP-004, MVP-006 |
| 4 — Household/member structure exists | MVP-002, MVP-004 |
| 5 — User can quickly create/select meal stubs and recipes | MVP-007, MVP-008, MVP-011, MVP-012 |
| 6 — Restrictions and a small number of food policies are represented | MVP-006, MVP-009, MVP-023 |
| 7 — Manual and locked meals are respected | MVP-012, MVP-013, MVP-023 |
| 8 — **Cover My Week** generates a coherent plan locally, without LLM or network | MVP-023, MVP-024 |
| 9 — Known hard constraints are never knowingly violated | MVP-009, MVP-023, MVP-025 |
| 10 — Planner evaluates the cycle as a sequence | MVP-023, MVP-025 |
| 11 — Important unresolved assumptions surface; low-value questions stay silent | MVP-023, MVP-024 |
| 12 — Shopping list is derived immediately from the accepted plan | MVP-015, MVP-016, MVP-024 |
| 13 — Core loop works offline | MVP-017 |
| 14 — Planner runs are versioned and reproducible enough to debug | MVP-023, MVP-025 |
| 15 — Ordinary behavior creates structured evidence for future cycles | MVP-012, MVP-023, MVP-024 |
| 16 — Analytics measure coverage, corrections and attention without exporting sensitive content | MVP-019 |
| 17 — The product demonstrates the central claim | MVP-024 (attention measurement, AC-8), MVP-022 (go/no-go verdict, AC-6) |

`MVP-003` discharges no §26 item directly; it is the UI carrier for items 5, 7, 8 and 12.

## Decision record

| ID | Decision | Consequence |
|---|---|---|
| D-001 | This roadmap is the operational source of truth | Update it at every task handoff |
| D-002 | Use bounded, outcome-oriented cards | Split work when verification cannot stay coherent |
| D-003 | Resolve classified decision gates before dependent work | Do not bury consequential choices in implementation |
| D-004 | Rebuild from a clean-room scaffold | Preserve no legacy implementation |
| D-005 | Establish a thin domain spine before vertical UI work | Avoid both screen-first coupling and premature architecture |
| D-006 | Do not use Sinter or a custom orchestrator | Use explicit human-reviewed workflow invocations |
| D-007 | Nothing in the old Meal Mate app must be preserved | Only `.git`, docs, license, and selected product metadata survive reset |
| D-008 | Introduce Firebase in phases | Emulator first, real non-production project later, production separately |
| D-009 | Split sharing into publication/web and Android routing | Security boundary is independently verifiable |
| D-010 | Keep observability a separate foundation card | Product work owns its own events afterward |
| D-011 | Bound offline promises to the MVP core loop | Do not add a second database without evidence |
| D-012 | Use risk-tiered verification | Independence and evidence rise with failure cost |
| D-013 | Restate only load-bearing PRD constraints in cards | Keep cards usable without copying the PRD |
| D-014 | Use a curated narrow-rights starter-content hybrid | Original + verified federal/public-domain/CC0; attributed CC BY only when durable |
| D-015 | Android-only MVP/initial launch; iOS post-launch priority | No iOS hardware or CI blocks MVP |
| D-016 | URL import is an isolated MVP stretch goal | Core manual recipe flow remains sufficient |
| D-017 | Classify tasks by complexity and assurance independently | Model capability and verification rigor are separate decisions |
| D-018 | Use guarded workflow routing without automatic chaining | Explicit invocation and review remain control points |
| D-019 | Retire `MealMate` as the public brand; defer name and production ID to DEC-004 | `com.mealmate.app` is consumed on Play and the name is crowded. Scaffold with temporary `dev.mealmate.temp` / Dart `meal_mate`; replace before MVP-018, MVP-021, MVP-022 |
| D-020 | Pin Android `minSdk 24` explicitly | `targetSdk`/`compileSdk` inherit Flutter; PRE-001 verifies the default does not exceed 24 |
| D-021 | Scaffold Android only | Web added at MVP-020 only after deciding Flutter Web vs static Hosting for previews |
| D-022 | Run the Android emulator on the Windows host; WSL uses `adb` as a TCP client | No `/dev/kvm` on this Win10 Pro 22H2 host and nested virtualization is Win11-only. Applies to PRE-001 and the MVP-003/MVP-004/MVP-005 emulator evidence |
| D-023 | Riverpod (non-codegen) + go_router; plain-Dart domain/repository interfaces with Riverpod-override DI | Provider overrides are the test seam; go_router's URI routing serves MVP-020/021 App Links. See DEC-002 resolution |
| D-024 | Feature-first layout (`lib/domain`, `lib/data`, `lib/features/<name>`); no codegen | Maps to PRD §9 entities and MVP card boundaries; avoids build_runner for MVP-sized models. See DEC-002 resolution |
| D-025 | Tiered 100% coverage gate on `lib/domain`/`lib/data` only, plus `glados` property-based tests on the highest-risk domain functions | Blanket 100% on presentation code is low-signal; property tests catch what line coverage can't. `mutation_test` deferred until domain logic is substantial. See DEC-002 resolution |
| D-026 | Domain/Flutter-Firebase independence enforced by a small grep-based script in the command contract | Checks the one named layering rule at negligible cost; upgrade to `clean_code_lints`/`import_lint` only if a second rule emerges. See DEC-002 resolution |
| D-027 | Allow Done with a `NOT VERIFIED` item when the resolving action is not the card's own work to do and the owner accepts the residual risk | Three conjuncts, all required: the card names the constraint or stop condition putting the action outside its scope; the card names the downstream discharge point or the permanent residual risk; the owner records dated, attributed acceptance in the Evidence log. An item that is merely unfinished work inside the card's own scope still blocks Done. Applies to cards reaching Done on or after 2026-08-24. Introduced for DEC-004's Play package-ID item; DEC-004's trademark item is outstanding owner work and is not covered. See `docs/task/README.md` verification rules |
| D-028 | Adopt PRD v3: Rust household kernel + food domain + SQLite behind a coarse FRB bridge; Flutter owns presentation | D-024–D-026 stand for Dart presentation code but no longer govern durable domain state; `PRD_v2` is historical context; MVP-002 retitled to the Rust kernel/SQLite foundation; PRE-002/DEC-005 gate all domain work. Recorded 2026-08-24. DEC-002 moves from the DOMAIN-READY gate to EMULATOR-PERSISTENCE-READY (it governs Dart presentation) and reaches Done on already-recorded evidence: its only open item was the superseded `lib/domain` coverage gate |
| D-029 | Reconcile the task system by dated amendment banners plus outline cards, not a full rewrite | Full re-derivation of affected cards happens once per card after DEC-005 is Done; banners never delete `PRD_v2` citations; a dependency change edits the card's Depends-on cell, the banner, and this register together |
| D-030 | Commit to the Rust core on the FRB native-assets backend; `flutter test` requires the pinned Rust toolchain (`cargo build --release` in `rust/` first); bridge crate stays `rust/` and becomes the workspace root at MVP-002; `rusqlite` (`bundled`) + `rusqlite_migration`; iOS re-opens the commitment at PRE-003 | DEC-005 resolution on PRE-002 evidence (2026-08-24, Android only — declared deviation from PRD §17/§22). Reversal: one `integrate --integration-backend cargokit` run, APIs unchanged; §22 fallback rejected as contra-evidence |
| D-031 | The Status contract above carries the only Draft-trigger rule; `docs/task/README.md` points at it, and a package added inside a card's stated scope is card evidence, not a Draft event | `docs/ROADMAP.md` and `docs/task/README.md` previously enumerated different, non-overlapping triggers ("source or decision" vs "product, architecture, or dependency") with no stated precedence, so every new package was a judgment call. The merged sentence is a superset of both; "dependency" resolves to the card's `Depends on` cell, which is what D-029 already meant by a dependency change. Recorded 2026-08-25 after MVP-002 added `path_provider` and `tempfile`; that disposition (evidence record, no return to `Draft`) is what this rule now states generally. A package that moves an architectural boundary — a backend SDK, an auth provider, a second state-management library — is an architecture change and still returns the card to Draft |
| D-032 | The owner-work hash guard covers the decision cards, the PRD, and the v3 docs; `KNOWN_ISSUES.md` and `KNOWN_ISSUES-low.md` are excluded as review-workflow artifacts | `/redteam-code`, `/fix-findings`, `/ki-maintain`, `/tidy`, `/next-task` and `/import` all write the `KNOWN_ISSUES` files by design, so any card followed by a review pass broke the guard and made a card's own recorded evidence false — MVP-002's did. Hashing cannot distinguish a workflow append from an agent rewriting owner prose; `git diff` at commit time surfaces the latter. Recorded 2026-08-25. PRE-002 AC-6 stays honestly passed; future AC-6-style criteria are checkable again. Rejected: an append-only `git diff` assertion replacing `sha256sum -c` — the repo has no enforcement artifacts beyond the `Zone.Identifier` pre-commit hook, and `KNOWN_ISSUES-low.md` already tracks enforcement routes named but never built |
| D-033 | Emulator bring-up and adb access go through `tools/emulator.sh` (`up`/`adb`/`status`/`down`); agents use `tools/emulator.sh adb <args>` because shell env does not persist across tool calls | Codifies the D-022 bridge shape with the measured host quirks (process names, snapshot revert, socket derivation). Recorded 2026-08-25 |
| D-034 | The v3 amendment banners are discharged in one pass rather than one card at a time. Re-derivation replaces a superseded `PRD_v2` assumption only with a v3 fact already decided in `docs/PRD_v3.md`, `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md`, `docs/CODEX_CLAUDE_PIVOT_PROMPT.md`, `docs/V3_MIGRATION_PLAN.md`, `docs/task/MVP_INVARIANTS.md`, or a recorded D-number; where none of those decides the question the card names a decision gate instead of choosing | D-029 permitted either shape and `docs/task/SEQUENCE.txt` step 1 chose the single pass. Recorded 2026-08-26: 21 cards re-derived (`MVP-004`–`MVP-022`, `DEC-003`, `OPT-001`) and `MVP-023`, `MVP-024`, `MVP-025`, `PRE-003` given bodies. Cards are tiered by who decides their content, not by edit depth: for `MVP-004`–`MVP-017` and `OPT-001` PRD v3 decides, so they are fully re-derived; for `MVP-018`–`MVP-022` and `DEC-003` the cloud, security, environment-topology, App-Check, sharing-rights and release-control content is preserved verbatim because `DEC-003` and `DEC-004` have not decided it yet, and rewriting it here would be the premature specification principles §25 forbids. Three AC-level edits are still made in that second group where v3 itself decides the answer — `MVP-018`'s new AC-6, `MVP-019`'s PRD §19 metric list, `MVP-022`'s AC-1. `MVP-003` is excluded: a current approved plan must re-derive the card before implementation; if no accessible current plan is supplied, `docs/task/SEQUENCE.txt` requires creating and approving one first. `MVP-003`'s banner was the last outstanding remainder; step 2 of its approved plan folded it in on 2026-08-28, completing this pass. The later workflow/status reconciliation changed only operational metadata, not that card scope. D-033 is reserved for `tools/emulator.sh`, so the gap between D-032 and this row is expected and is not a lost decision. `docs/task/SEQUENCE.txt` steps 1–2 are discharged by this row. The sequence and compact current/next fields were reconciled on 2026-08-26 when the mandatory workflow gate was centralized, without changing the intended MVP-003 implementation scope. `DEC-003` is a D-032-guarded decision card edited by design here: only its Authoritative sources and Locked constraints change, and its Decisions-to-resolve and Options-and-tradeoffs sections are byte-unchanged |
| D-035 | Keep `docs/ROADMAP.md` as the sole live task-status source; task-card status cells point to its register, and every plan and execution repeats one shared planning/execution/handoff contract | A second `MVP_STATUS.md`/`TASK_STATUS.md` would split status from the roadmap's evidence and decisions, while a Markdown parser would add a new process failure mode. The compact Current milestone block names the single implementation lane and selected next task. The one-time migration audit above establishes the six pre-D-035 Done cards; later direct dependencies are sufficient because every new Done transition recursively requires valid evidence and explicit owner approval. Execution rechecks current state and plan freshness, then status, evidence, delivery-gate progress, and next task change together at handoff. Owner-paced decisions may proceed concurrently without occupying the implementation lane. Recorded 2026-08-26 after owner selection of the lowest-friction option; corrected by the same-session adversarial review before use |
| D-036 | Fix pass on the D-034 re-derivation: invariant 7 re-derived to the v3 local-first model; §26 item 17 split into `MVP-024` AC-8 measurement and `MVP-022` AC-6 verdict, with the pass bar left to an owner decision; the §26 table made the sole §26 citation site | Invariant 7 still mandated the cache/prefetch model `MVP-017` was re-derived to remove, and `docs/task/MVP_INVARIANTS.md:3` makes a conflict a stop condition; the new wording is anchored to PRD v3 §6.4–6.5 and §12 and the header notes the restatement. `MVP-001` and `DEC-001` cite the invariants file wholesale and are Done, but a scaffold card and a naming decision are immaterial to offline persistence, so neither returns to Draft under D-031. Item 17 previously resolved to `MVP-022`, whose AC-1 discharged it by reading the table that named `MVP-022`; production now sits with PRD §22's named proof vehicle and judgement with the go/no-go, neither repeating the table. This row extends D-034's "Three AC-level edits" enumeration to five: `MVP-019`'s AC-5 and `MVP-022`'s AC-6 are added because PRD v3 §19 decides the metric set and §22 decides that Cover My Week is the required proof — but neither decides a numeric pass bar, so that is recorded as a `MVP-022` decision gate rather than chosen here. `MVP-024` repeated the §26 item→card mapping in its sources and scope, at two sites, one claiming items the table assigns elsewhere; both were removed and its AC-4 extended to veto so the item-15 assignment is fully discharged. `MVP-022` AC-1 still restates table row 1's iOS split and hard-codes the "items 2–17" range — the same defect class surviving at the release gate, left in place because the D-034 tier keeps `MVP-018`–`MVP-022` content verbatim pending `DEC-003`/`DEC-004`, and tracked in `KNOWN_ISSUES.md`. `MVP-006` gained a criterion for member-scoped preference storage and a decision gate for restriction scope, which PRD v3 leaves open. All cards whose content changed were already `Draft`, so no D-031 transition was required. Recorded 2026-08-26 after two adversarial plan reviews |

## Evidence log

| Date | Card | Result | Evidence |
|---|---|---|---|
| 2026-08-21 | DEC-001 | Done | Resolution recorded in the card; decisions 2–3 final, decision 1 descoped to DEC-004. Clearance for `MealMate`: Play package ID — FAIL (`com.mealmate.app` search-index entry; listing URL now 404, i.e. indexed then unpublished, and Play never releases a published ID). Play title — FAIL (≥5 apps titled MealMate). Web/search — FAIL (mealmateco.com, same concept). Trademark — NOT VERIFIED (web-sourced USPTO 97352318, appliances class; no authoritative TESS search). Apple App Store title — NOT VERIFIED (not performed). Recorded under the pre-2026-08-24 practice; predates D-027, whose exception does not apply retroactively. |
| 2026-08-22 | PRE-001 | Done | AC-1 PASS (Flutter 3.47.1 / Dart 3.13.1 at `~/development/flutter`; WSL JDK 17.0.19 with JRE-8 retained as system default; Windows JDK 17.0.20.1; Gradle 9.3.1; Android platform-36, build-tools 36.1.0, platform-tools 37.0.1, cmdline-tools 15859902, emulator 37.1.11, `system-images;android-36;google_apis;x86_64` rev 7; template defaults minSdk 24 / targetSdk 36 / compileSdk 36 — minSdk matches the D-020 floor exactly, no flag needed). AC-2 PASS (`dart format` clean, `flutter analyze` no issues, 1/1 test). AC-3 PASS (`app-debug.apk`, 150MB). AC-4 PASS (Pixel 5 AVD `pre001_avd`, Android 16/API 36, WHPX usable per `-accel-check`; app installed, launched and screenshotted from WSL over the D-022 adb TCP bridge — evidence `C:\pre001_evidence\pre001_evidence.png` + emulator logs, WSL copy `~/pre001_evidence/`). AC-5 PASS (before/after `git status` identical; only pre-existing untracked `.claude/`; no Meal Mate application file changed). See PRE-001 command contract below. |
| 2026-08-24 | DEC-002 | Done | Decision fields present — PASS (rationale/versions/rejected-alternatives/test-implications/reversal-cost recorded for all four sub-decisions: (1) Riverpod non-codegen + go_router over plain-Dart domain/repository interfaces; (2) feature-first layout, no codegen; (3) tiered 100% coverage gate on `lib/domain`+`lib/data` with `glados` property-based tests on the highest-risk domain functions; (4) grep-based script enforcing domain independence from Flutter/Firebase). Commands runnable in the project environment — PASS (`dart format`, `flutter analyze`, `flutter test` exercised at PRE-001 AC-2 and MVP-001 AC-3; `flutter build apk --debug` at PRE-001 AC-3 and MVP-001 AC-3. The `lib/domain` 100% gate and its `lcov`/`genhtml` tooling are superseded by D-028 and were never installed; the `--coverage` flag had no other consumer and is dropped from the contract — `lib/features` coverage reporting is optional and non-gating until a card asks for it. Flipped to Done 2026-08-24 under D-028). Contract recorded in roadmap — PASS (see D-023–D-026 and the DEC-002 command contract below). |
| 2026-08-22 | MVP-001 | Done | AC-1 PASS (`git diff --stat HEAD` = 124 files, 257 insertions, 4124 deletions; `ios/ macos/ windows/ linux/ web/` deleted in full, plus `lib/models/recipe.dart`, `lib/screens/recipe_management/recipe_form_screen.dart`, `test/models/recipe_test.dart`, `android/app/src/main/kotlin/com/example/meal_mate/MainActivity.kt` and all three Groovy `*.gradle` files; independent depth-2 sweep found only the approved surviving set plus stock scaffold output). AC-2 PASS (`applicationId`/`namespace`/`package` = `dev.mealmate.temp`, Dart package `meal_mate`, `android:label="MealMate (dev)"`, explicit `minSdk = 24` with `targetSdk`/`compileSdk` still inheriting `flutter.*` per D-020; zero `com.example` on the code surface — the only two repo-wide occurrences are governance prose in this card and DEC-001; temporary-pending-DEC-004 documented in `README.md`, `pubspec.yaml`, and `build.gradle.kts`). AC-3 PASS (`dart format` clean, `flutter analyze` no issues, `flutter test` 1/1 — all reran independently by the verifier; `flutter build apk --debug` → `app-debug.apk` 150,419,440 bytes. The APK path was permission-blocked in both agent sessions and was confirmed directly by the owner on 2026-08-22). AC-4 PASS (fresh-context verifier: no legacy Dart/schema/test, no non-Android platform tree, `.metadata` `migration.platforms` = root + android only, no residual Groovy, no empty orphan package dir; every scaffold config byte-diffs to stock Flutter 3.47.1 apart from the three intended identity edits). Surviving set = 9 entries (`.git .claude .idea .githooks .gitattributes docs LICENSE KNOWN_ISSUES.md README.md`), owner-approved 2026-08-22 — this deliberately exceeds the card's literal four-entry list. Rollback `~/mvp001_rollback_2026-08-22/` (tarball rehearsed byte-faithful before the destructive step). Not committed or pushed. |
| 2026-08-24 | DEC-004 | In Progress | Shortlist prescreen only; no name chosen, so no clearance item is cleared for a final name. Google Play — NOT VERIFIED (title triage across 12 candidates by two independent methods; lead `Kimatta` shows no titled match, but Play's own search under-reported at least one real listing — `com.orangefox22.Kondate`, a meal-planning app — so a null is "not found by two methods", not "absent"; package-ID availability for `app.<name>` is unprovable read-only, so every package-ID cell for a screened name reads `NOT VERIFIED`. The listing-URL 404 plus no-index-hit observation is recorded for the lead `app.kimatta` alone — see the card's **Kimatta** evidence block; no per-name package-ID observation is recorded for the other ten screened names. Play Console was not used). Apple App Store — NOT VERIFIED (iTunes Search API, the 11 screened names; `Kimatta` no match; exact out-of-category titles found for Shitaku/Provi/Norra; in-category titles found for Kondate). Web/search — NOT VERIFIED (`Kimatta` first page is dictionary entries only; `kimatta.app` is unregistered per RDAP, `kimatta.com` registered 2004-06-01; no domain purchased). Trademark — NOT VERIFIED (`Kimatta` only; tmsearch.uspto.gov returned HTTP 200 / 125,660 bytes of SPA shell with zero query-term occurrences, API paths 405 and 404 — receipt in the card; secondary web search found no record, indicative only; authoritative search needs an interactive session or a paid search and is routed to the owner). Rejected at Pass 1: Kondate, Savora (both in-category Play title collisions). Register, dispositions and per-name evidence in the card. This row is edited in place when the card resolves — flip Result to `Done` and each criterion to `PASS`, `FAIL`, or `NOT VERIFIED`; do not append a second DEC-004 row. Only the package-ID criterion is eligible to stay at `NOT VERIFIED` at Done, and only with the owner's dated, attributed acceptance of the residual risk recorded in this same cell as `owner-accepted YYYY-MM-DD`, per D-027 and `docs/task/README.md`. The trademark criterion is outstanding owner work, not an exception case, and blocks Done until performed. |
| 2026-08-24 | PRE-002 | Done | AC-1 PASS (`rustc 1.98.0`, `cargo 1.98.0`, `flutter_rust_bridge_codegen 2.13.0`; pins in `rust/rust-toolchain.toml`, `pubspec.yaml`, `rust/Cargo.toml`; backend native-assets — see the PRE-002 command contract). AC-2 PASS via the native-loading test route: `test/bridge_native_test.dart` 2/2 under `flutter test`, `throwsA(isA<KimattaError_InvalidPath>())` on a whitespace path; coupling recorded in the contract for DEC-005 decision 3. AC-3 PASS (`app-debug.apk`). AC-4 PASS (`app-release.apk` 46.0 MB; `libkimatta_bridge.so` in `arm64-v8a` 676,480 B, `armeabi-v7a` 429,212 B, `x86_64` 704,128 B; debug carries the same set). AC-5 PASS (release APK on `emulator-5554` over the D-022 bridge; `~/pre002_evidence/health_screen.png` shows `core_version: 0.1.0` and `health: schema v0 at …/kimatta.db`, `typed_error.png` shows `probe: KimattaError.InvalidPath` after tapping the button located by its semantics `content-desc`). AC-6 PASS (`sha256sum -c` 38/38 OK over every staged owner path; index blobs of the four docs this card edits were byte-identical before staging). AC-7 PASS (11.05 / 10.32 / 9.33 s). AC-8 NOT VERIFIED — D-027 block: no macOS host or CI exists and iOS is post-launch (D-015, invariant 14); discharge point `PRE-003`; owner-accepted 2026-08-24 (recorded by the agent at the owner's explicit instruction). Fix ledger: 1 of 2 native-assets fixes used, cargokit not attempted. Card promoted to Done on that clause. Not committed or pushed. |
| 2026-08-24 | DEC-005 | Done | Decisions 1–5 recorded in the card with rationale, rejected alternatives, test implications, and reversal cost — PASS. Registered as D-030 — PASS. Android-only scope of decision 2 stated in D-030 and in `docs/V3_IMPLEMENTATION_STATUS.md` Deviations — PASS. MVP-002 promoted Draft → Ready. Not committed or pushed. |
| 2026-08-24 | MVP-002 | Done | AC-1 PASS (`cargo test --workspace`: household-core 2/2 — empty/whitespace rejected, no trimming; kimatta-storage 7/7 — empty DB → v1, `MIGRATIONS.validate()`, foreign keys on after `open()`, orphan member rejected via FK, rollback on duplicate later row, round trip, idempotent file reopen keeps version 1 and data; `cargo tree --workspace -i rusqlite` = one version). Resolved: `rusqlite` 0.40.2 (`bundled`), `rusqlite_migration` 2.6.0, `libsqlite3-sys` 0.38.2. AC-2 PASS (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` clean; `grep -rn unsafe rust/crates/household-core rust/crates/kimatta-storage` empty; both crates `#![forbid(unsafe_code)]`). AC-3 PASS (verified on-device 2026-08-25 over the D-022 bridge, `emulator-5554`, AVD `pre001_avd`, API 36 x86_64: release APK installed and launched, health screen reads `health: schema v1 at /data/user/0/dev.mealmate.temp/files/kimatta.db`, probe reads `probe: KimattaError.InvalidPath`, no `AndroidRuntime` exception; `ac3_health.png` / `ac3_probe.png` in `~/mvp002_evidence/`). **The D-027 block is discharged — no `owner-accepted` clause is needed.** The database path is `getApplicationSupportDirectory()` → `context.getFilesDir()` rather than `Directory.systemTemp`; verified upstream that `Directory::SystemTemp()` falls back to the literal `/data/local/tmp` on Android when `TMPDIR`/`TMP` are unset and that the Flutter Android embedding sets neither, so the pre-fix build would have failed this AC outright with `KimattaError.Storage`. AC-4 PASS (Rust gate added to the DEC-002 contract and mirrored in the PRE-002 block and README, and ordered ahead of the Flutter lines to match PRE-002 so a Rust failure precedes the APK build; exercised clean, plus `flutter test` 5/5 — the bridge tests now assert schema version 1 on a real file and `KimattaError_Storage` on an unopenable path). Cross-compile: `rusqlite bundled` built under the native-assets hook first try (debug APK 70.7 s cold). All four required items are PASS and no D-027 exception is needed; owner-approved 2026-08-27 in the instruction to handle closeout. `Verify → Done`; DOMAIN-READY achieved; `MVP-003` promoted `Draft → Ready`. Not committed or pushed. |
| 2026-08-28 | MVP-003 | Verify | AC-1 PASS (`test/app_test.dart` 14/14 within `flutter test` 18/18: five destinations, `/plan/cover` reachable and returnable, unknown route renders not-found with a working return action, restoration with a fresh router — the restoration test is an expected-to-pass regression pin with no falsifiability control, because `MaterialApp.router` restores the location through Flutter's own `Router` under the `'app'` scope rather than through go_router's ids; it pins this card's composition, not those ids). AC-2 PASS (three guidelines — `androidTapTargetGuideline`, `labeledTapTargetGuideline`, `textContrastGuideline` — × light/dark × five destinations; destination screens at text scale 2.0, `NavigationBar` labels clamped at 1.3 by framework design; focus automated via tab traversal). All AC-2 items come from `flutter test` and are bridge-independent. AC-3 PASS (gate exit codes all 0: `dart format`, `flutter analyze`, `cargo fmt`/`clippy`/`test --workspace` 9/9, `cargo build --release` + `flutter test` 18/18, `flutter build apk --debug`. On-device 2026-08-28 over the D-022 bridge, AVD `pre001_avd`, API 36 x86_64, 1080×2340 gesture navigation; evidence `~/mvp003_evidence_wt4/`: `plan.png`, `settings.png`, `db_listing.txt`, `settings_scale2.png`, `relaunch_after_kill.png`; pids before=3437 relaunched=3904. Screenshot readings: `plan.png` shows the Plan destination selected; `settings.png` shows `Local database: schema v1 at /data/user/0/dev.mealmate.temp/files/kimatta.db`; `settings_scale2.png` shows visibly scaled body text with no overflow while the bar labels stay clamped; `relaunch_after_kill.png` shows Settings still selected after `am kill`. `run-as dev.mealmate.temp ls -l files/kimatta.db` → `-rw------- 1 u0_a218 u0_a218 20480 2026-08-25 09:33 files/kimatta.db`, a real app-private file entry — but one that predates this run: the 2026-08-25 mtime is MVP-002's on-device session, so what this run exercised is reopen of an already-v1 database, not create-and-migrate. The migration is idempotent and left the schema at v1. No on-device first create has been observed on any card; tracked as an OPEN entry in `KNOWN_ISSUES.md`). Packages: go_router 18.0.0, flutter_riverpod 2.6.1 (D-031); the resolved Riverpod major is 2.x, which has no provider-retry behaviour to disable. Measured: `StatefulShellRoute.indexedStack` builds branches **lazily** on go_router 18.0.0 — established by the "the health provider is created once per launch" control, which read `creations == 0` with `App`'s `ref.listen` removed. Evidence contract satisfied; awaiting explicit owner approval. Not committed or pushed. |

### PRE-001 command contract

Pinned versions: Flutter 3.47.1 · Dart 3.13.1 · JDK 17 (WSL 17.0.19, Windows 17.0.20.1) · Gradle 9.3.1 · Android platform-36 · build-tools 36.1.0 · platform-tools 37.0.1 · cmdline-tools 15859902 · emulator 37.1.11 · `system-images;android-36;google_apis;x86_64` rev 7 · AVD `pre001_avd` (device `pixel_5`, Android 16 / API 36).

WSL locations: Flutter `~/development/flutter`, Android SDK `~/Android/sdk`, `JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64`. Windows: Android SDK `C:\Android\sdk`, JDK `C:\Program Files\Microsoft\jdk-17.0.20.101-hotspot`.

```bash
flutter create --org dev.mealmate --project-name meal_mate <project>
dart format --output=none --set-exit-if-changed .
flutter analyze
flutter test
flutter build apk --debug
```

`--org dev.mealmate --project-name meal_mate` sets the Dart package to `meal_mate` (matching DEC-001) but also assembles the applicationId as `dev.mealmate.meal_mate` — Flutter has no flag to decouple the applicationId's final segment from the project name (verified against `create_base.dart`'s `createAndroidIdentifier`, which returns `$organization.$name`). MVP-001 must explicitly set `applicationId "dev.mealmate.temp"` in `android/app/build.gradle` (and the corresponding package path) after scaffolding to reach DEC-001's actual identity — this command only gets the Dart package name and org prefix right. The explicit `minSdk 24` pin (D-020) is also Gradle work for MVP-001, not this command — PRE-001 only verified the template default already equals 24. Identifier flags were not exercised by PRE-001; MVP-001 AC-2 remains the point of verification, not this contract.

Emulator + adb bridge (D-022 shape — emulator on the Windows host, WSL as adb TCP client):

```bash
# 1. Launch the emulator (Windows host, non-blocking)
powershell.exe -NoProfile -Command "Start-Process 'C:\Android\sdk\emulator\emulator.exe' -ArgumentList '-avd','pre001_avd'"

# 2. Verify hardware acceleration before waiting on boot -- assert exit code, not stdout text
powershell.exe -NoProfile -Command "& 'C:\Android\sdk\emulator\emulator.exe' -accel-check; exit $LASTEXITCODE"
# If -accel-check fails (non-zero exit), the emulator from step 1 is already running with no
# adb bridge yet -- kill it directly instead of relying on the adb-based teardown in step 7:
#   taskkill /IM emulator.exe /F
#   taskkill /IM qemu-system-x86_64.exe /F   # the front-end always exists here; the qemu kill
#                                            # is a no-op if no child was spawned

# 3. Windows-side adb server bound to all interfaces
powershell.exe -NoProfile -Command "Start-Process -WindowStyle Hidden 'C:\Users\David\AppData\Local\Microsoft\WinGet\Packages\Google.PlatformTools_Microsoft.Winget.Source_8wekyb3d8bbwe\platform-tools\adb.exe' -ArgumentList '-a','-P','5037','nodaemon','server'"
# NOTE: this is the winget-installed adb.exe -- distinct from C:\Android\sdk\platform-tools\adb.exe,
# which the emulator's SDK-root validation requires to exist (prerequisite 3) but which this bridge
# does not itself run. The "Users\David" segment is host-specific; re-derive with `where.exe adb.exe`.

# 4. WSL client points at it via the default-route gateway
export ADB_SERVER_SOCKET=tcp:$(ip route | awk '/default/ {print $3}' | head -1):5037

# 5. Wait for boot to complete before installing (bounded -- an emulator that never boots
#    otherwise hangs the contract indefinitely; on timeout, treat as accel/boot failure and
#    fall back to the taskkill teardown noted above)
timeout 120 adb wait-for-device
for i in $(seq 1 60); do
  [[ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]] && break
  sleep 2
done

# 6. Install and launch
adb devices && adb install -r <apk> && adb shell monkey -p <applicationId> -c android.intent.category.LAUNCHER 1

# 7. Teardown when the session ends
adb emu kill
adb kill-server
```

Operational entry point since 2026-08-25: `tools/emulator.sh` (D-033).

**Prerequisites discovered during PRE-001, required for MVP-003/004/005:**

1. Windows Firewall had two inbound **Block** rules named `adb.exe` (program-scoped, `ports=Any`). Block overrides Allow, so no port-based Allow rule can work while they exist. They must be removed (elevated) or the WSL→emulator bridge silently times out.
2. An inbound Allow rule for TCP 5037 is required (elevated): `New-NetFirewallRule -DisplayName 'WSL adb server' -Direction Inbound -Protocol TCP -LocalPort 5037 -Action Allow -Profile Any -RemoteAddress 172.21.80.0/20`. Close it when emulator work is done (elevated): `Remove-NetFirewallRule -DisplayName 'WSL adb server'`.
3. Windows `platform-tools` must be installed under `C:\Android\sdk` even though the bridge uses a different `adb.exe` — the emulator validates the SDK root and aborts with `Broken AVD system path` without it.
4. `systeminfo` is not a usable WHPX signal on a WSL2 host (it reports only "A hypervisor has been detected"). Use `emulator.exe -accel-check` and assert on its **exit code**; its stdout puts `accel:` and the code on separate lines.

### DEC-002 command contract

Extends the PRE-001 contract above. Required for MVP-002 onward.

```bash
dart format --output=none --set-exit-if-changed .
flutter analyze
(cd rust && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace)
(cd rust && cargo build --release) && flutter test   # stale .so = false pass; never run alone
flutter build apk --debug
```

The `lib/domain` coverage gate (`lcov`/`genhtml`) and the domain-independence grep that stood here were removed under D-028 — durable domain code now lives in Rust, and MVP-002 added the Rust gate above (2026-08-24). `lib/features` coverage reporting is optional and non-gating; re-add `--coverage` when a card asks for it. The `glados` property-test targets named below (unit conversion, shopping aggregation, restriction matching, planning-cycle date math) are now Rust functions — property tests for them belong to MVP-025 in Rust; `glados` remains available for any Dart-side logic that warrants it.

### PRE-002 command contract

Extends the DEC-002 contract above. Required for MVP-002 onward. Backend: **native-assets** (`flutter_rust_bridge_codegen integrate --integration-backend native-assets`) — Flutter's own build-hook pipeline (`hook/build.dart` → `flutter_rust_bridge_hooks` → `native_toolchain_rust`) compiles and bundles the Rust crate; no Gradle plugin, no `cargo-ndk`. Chosen because the card asked for it first and it passed within budget (1 of 2 fixes used; cargokit not attempted). Pinned: Rust 1.98.0 (`rust/rust-toolchain.toml`, channel + targets), `flutter_rust_bridge` 2.13.0 = `flutter_rust_bridge_hooks` 2.13.0 = codegen 2.13.0 (exact pins in `pubspec.yaml`, `=2.13.0` in `rust/Cargo.toml`). Installs live in `~/.cargo` and `~/.rustup` (user-level; agent shells must `export PATH="$HOME/.cargo/bin:$PATH"`). NDK 28.2.13676358 is discovered by Flutter (`ndkVersion` default); nothing to configure.

```bash
# install (once per host)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain 1.98.0 --profile minimal
export PATH="$HOME/.cargo/bin:$PATH"
rustup target add --toolchain 1.98.0 aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
rustup component add --toolchain 1.98.0 rustfmt clippy
cargo install flutter_rust_bridge_codegen --version 2.13.0 --locked

# generate bindings after any change under rust/src/api (writes lib/src/rust/** and rust/src/frb_generated.rs — never hand-edit)
flutter_rust_bridge_codegen generate

# gate + build. The generated loader reads rust/target/release/; the native-assets hook
# builds under .dart_tool/hooks_runner and never refreshes it — a stale .so gives a FALSE
# PASS, not a load failure, so never run `flutter test` alone after a Rust edit.
dart format --output=none --set-exit-if-changed .
flutter analyze
(cd rust && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace)
(cd rust && cargo build --release) && flutter test
flutter build apk --debug
flutter build apk --release
```

- **ABI set:** `arm64-v8a`, `armeabi-v7a`, `x86_64` — `libkimatta_bridge.so` present in each, in both debug and release APKs.
- **Dev cycle (AC-7):** Rust edit → `generate` → `flutter build apk --debug` = 11.05 s, 10.32 s, 9.33 s (three runs, warm Gradle). The hook always builds the cargo `release` profile, debug APKs included.
- **Test↔toolchain coupling (DEC-005 decision 3):** `flutter test` runs the build hook, so every Dart test — the fake-fed `test/app_test.dart` included — needs rustup + the pinned toolchain on the host; `test/app_test.dart` avoids *loading* the library, not *building* it. Additionally the generated loader looks in `rust/target/release/` under `flutter test`, hence the `&&`-chained `cargo build --release` command above (recorded as fix #1 in the PRE-002 ledger; no generated or integrate-written file was edited).
- **Layout deviation from the card:** the bridge crate is `rust/` itself (`kimatta_bridge`, FRB's default), not `rust/crates/kimatta-bridge`; cargo rejects an empty `crates/*` workspace glob, so MVP-002 adds the `[workspace]` block when the first sibling crate exists. `--no-integration-test` was passed to `integrate`.
- **Generation tooling:** data-carrying error enums become `freezed` sealed classes (`KimattaError implements FrbException`; variants `KimattaError_InvalidPath`, `KimattaError_Storage`), so `freezed` 4.0.0 / `build_runner` 2.16.0 (dev) and `freezed_annotation` 3.1.0 were added. They serve generated bindings only (invariant 21); D-024's no-codegen rule still governs hand-written Dart.
- Emulator install/launch/teardown: PRE-001 contract steps 3–7. Note from this run: `adb emu kill` cannot reach the emulator console from WSL; close the emulator on the host (`taskkill /IM qemu-system-x86_64.exe /F` (works whether the AVD was launched by emulator.exe or Android Studio; see tools/emulator.sh)).

## Calibration and post-launch

After the first three implementation cards (`MVP-001` through `MVP-003`), review actual duration, context load, test yield, and remediation count; resize later cards only if evidence warrants it.

**First-three-card sizing calibration (recorded 2026-08-28, at `MVP-003`'s handoff).** Measured per card: `MVP-001` (Focused/Standard) — one working session, 1 test carried over from the scaffold, 0 remediation cycles, though it needed a fresh-context verifier pass and an owner confirmation for the permission-blocked APK path. `MVP-002` (Focused/Elevated) — spanned 2026-08-24 to the 2026-08-27 closeout with on-device evidence captured 2026-08-25, 9 Rust tests plus 4 bridge tests, 0 remediation cycles, but it reached `Done` without the fresh-context verifier its Elevated tier names, and its closeout left prose residue that `MVP-003` step 1c had to sweep. `MVP-003` (Focused/Standard) — one working session, 10 new widget tests (14 total with the bridge suite), 0 remediation cycles, 2 new packages, one emulator evidence cycle that passed first try. **Verdict: do not resize.** All three fit a single session's implementation once their plan existed, and the two cards that ran long did so in *planning and doc reconciliation*, not implementation — `MVP-003`'s plan reached rev 11 across several red-team passes while its code is six small files. The lever that would actually shorten these cards is fewer concurrent writers on the governance docs and a shorter plan-revision loop, not smaller cards. Two process findings carry forward: an Elevated card must not reach `Done` without its verifier, and a card's closeout should discharge its own prose residue rather than leaving it for the next card's step 1.

Post-launch begins with iOS feasibility/tooling and the iOS client, then follows PRD phases 1.5, 2, 2.5, and 3. Other candidates include improved recommendation calibration, richer collaboration, dedicated local storage only if measured need emerges, deterministic imports beyond OPT-001, paid delegation/LLM conveniences, and desktop experiments. Promote a candidate only by adding a bounded card and dependencies here.
