# Meal Mate Roadmap

This is the durable operational source of truth after the MVP as well as during it. The PRD defines product intent; task cards define bounded work; this file records sequence, status, gates, decisions, blockers, and evidence.

## Current milestone

**Milestone:** Foundation readiness  
**Next action:** `DEC-002` (engineering-experience foundation) is in `Verify` (2026-08-23) — decision fields and command contract recorded (PASS), but the command contract's coverage-gate tooling (`lcov`/`genhtml`) is not installed or exercised on this host (NOT VERIFIED); `MVP-002` stays `Ready` because its specific DEC-002 dependency — the command contract being recorded in the roadmap — is satisfied; install and exercise `lcov`/`genhtml` before DEC-002 flips to `Done`. `MVP-001` is Done (2026-08-22). `PRE-001` is Done (2026-08-22). `DEC-001` is Done (2026-08-21); its naming decision is deferred to `DEC-004`, now `In Progress` (shortlist prescreen recorded 2026-08-24) and owner-paced. `DEC-004` directly blocks `MVP-018`, `MVP-021`, and `MVP-022`, and therefore transitively gates `MVP-019` and `MVP-020` — nothing from `MVP-018` onward can start until it is Done.
**Platform:** Android MVP and initial launch. iOS is the first post-launch platform priority.

## Status contract

`Draft → Ready → In Progress → Verify → Done`. A material source or decision change returns an affected card to Draft. Done requires the card's required evidence to be recorded; approval of one artifact is not approval of another. Done evidence is recorded in the Evidence log below — one row per card, with each required item labelled `PASS`, `FAIL`, or `NOT VERIFIED` per `docs/task/README.md`.

## Delivery gates

| Gate | Exit condition |
|---|---|
| DOMAIN-READY | DEC-001, PRE-001, MVP-001, DEC-002, and MVP-002 Done |
| EMULATOR-PERSISTENCE-READY | MVP-003 through MVP-005 Done with emulator persistence evidence |
| LOCAL-CORE-LOOP-READY | MVP-006 through MVP-017 Done, including offline recovery |
| REAL-DEV-BACKEND-READY | DEC-003, MVP-018, and MVP-019 Done in a non-production project |
| SHARING-SECURITY-READY | MVP-020 and MVP-021 security, projection, and routing evidence passes |
| PRODUCTION-BETA-READY | DEC-004 Done and MVP-022 passes; production activation remains a separately authorized action |

## Task register

| ID | Status | Outcome | Depends on |
|---|---|---|---|
| DEC-001 | Done | Product/platform identity (name deferred to DEC-004) | — |
| PRE-001 | Done | Android toolchain readiness | — |
| MVP-001 | Done | Clean-room Flutter scaffold | DEC-001, PRE-001 |
| DEC-002 | Verify | Engineering-experience foundation | MVP-001 |
| MVP-002 | Ready | Domain spine and repository seams | MVP-001, DEC-002 |
| MVP-003 | Draft | Android shell and navigation | MVP-002, DEC-002 |
| MVP-004 | Draft | Emulator auth, household, persistence | MVP-002 |
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
| MVP-017 | Draft | Offline cache and recovery | MVP-008, MVP-009, MVP-013, MVP-014, MVP-016 |
| DEC-003 | Draft | Cloud/sharing release boundary | MVP-017 |
| DEC-004 | In Progress | Naming clearance and production identity | DEC-001 |
| MVP-018 | Draft | Real dev backend and durable auth | MVP-017, DEC-003, DEC-004 |
| MVP-019 | Draft | Privacy-safe observability | MVP-018 |
| MVP-020 | Draft | Share publication and web preview | MVP-009, MVP-018, DEC-003 |
| MVP-021 | Draft | Android App Links recipient flow | MVP-020, DEC-004 |
| MVP-022 | Draft | Android beta readiness | DEC-004, MVP-001–MVP-009, MVP-011–MVP-021, and MVP-010 unless explicitly cut; all delivery gates through SHARING-SECURITY-READY |
| OPT-001 | Draft | Structured URL recipe import | MVP-008; optional, never blocks MVP |

MVP-010 may be explicitly cut under the PRD's photo deferral rule, but must not silently disappear. OPT-001 has no dependency path into MVP completion.

## MVP acceptance traceability

| PRD §24 item | Owning cards |
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

## Evidence log

| Date | Card | Result | Evidence |
|---|---|---|---|
| 2026-08-21 | DEC-001 | Done | Resolution recorded in the card; decisions 2–3 final, decision 1 descoped to DEC-004. Clearance for `MealMate`: Play package ID — FAIL (`com.mealmate.app` search-index entry; listing URL now 404, i.e. indexed then unpublished, and Play never releases a published ID). Play title — FAIL (≥5 apps titled MealMate). Web/search — FAIL (mealmateco.com, same concept). Trademark — NOT VERIFIED (web-sourced USPTO 97352318, appliances class; no authoritative TESS search). Apple App Store title — NOT VERIFIED (not performed). Recorded under the pre-2026-08-24 practice; predates D-027, whose exception does not apply retroactively. |
| 2026-08-22 | PRE-001 | Done | AC-1 PASS (Flutter 3.47.1 / Dart 3.13.1 at `~/development/flutter`; WSL JDK 17.0.19 with JRE-8 retained as system default; Windows JDK 17.0.20.1; Gradle 9.3.1; Android platform-36, build-tools 36.1.0, platform-tools 37.0.1, cmdline-tools 15859902, emulator 37.1.11, `system-images;android-36;google_apis;x86_64` rev 7; template defaults minSdk 24 / targetSdk 36 / compileSdk 36 — minSdk matches the D-020 floor exactly, no flag needed). AC-2 PASS (`dart format` clean, `flutter analyze` no issues, 1/1 test). AC-3 PASS (`app-debug.apk`, 150MB). AC-4 PASS (Pixel 5 AVD `pre001_avd`, Android 16/API 36, WHPX usable per `-accel-check`; app installed, launched and screenshotted from WSL over the D-022 adb TCP bridge — evidence `C:\pre001_evidence\pre001_evidence.png` + emulator logs, WSL copy `~/pre001_evidence/`). AC-5 PASS (before/after `git status` identical; only pre-existing untracked `.claude/`; no Meal Mate application file changed). See PRE-001 command contract below. |
| 2026-08-23 | DEC-002 | Verify | Decision fields present — PASS (rationale/versions/rejected-alternatives/test-implications/reversal-cost recorded for all four sub-decisions: (1) Riverpod non-codegen + go_router over plain-Dart domain/repository interfaces; (2) feature-first layout, no codegen; (3) tiered 100% coverage gate on `lib/domain`+`lib/data` with `glados` property-based tests on the highest-risk domain functions; (4) grep-based script enforcing domain independence from Flutter/Firebase). Commands runnable in the project environment — NOT VERIFIED (`lcov`/`genhtml` not installed on this host; contract not yet exercised — see the DEC-002 command contract below, now carrying a real enforcing assertion; install and rerun before MVP-002 first executes it, then flip this row's Result to `Done` and this criterion to PASS/FAIL). Contract recorded in roadmap — PASS (see D-023–D-026 and the DEC-002 command contract below). |
| 2026-08-22 | MVP-001 | Done | AC-1 PASS (`git diff --stat HEAD` = 124 files, 257 insertions, 4124 deletions; `ios/ macos/ windows/ linux/ web/` deleted in full, plus `lib/models/recipe.dart`, `lib/screens/recipe_management/recipe_form_screen.dart`, `test/models/recipe_test.dart`, `android/app/src/main/kotlin/com/example/meal_mate/MainActivity.kt` and all three Groovy `*.gradle` files; independent depth-2 sweep found only the approved surviving set plus stock scaffold output). AC-2 PASS (`applicationId`/`namespace`/`package` = `dev.mealmate.temp`, Dart package `meal_mate`, `android:label="MealMate (dev)"`, explicit `minSdk = 24` with `targetSdk`/`compileSdk` still inheriting `flutter.*` per D-020; zero `com.example` on the code surface — the only two repo-wide occurrences are governance prose in this card and DEC-001; temporary-pending-DEC-004 documented in `README.md`, `pubspec.yaml`, and `build.gradle.kts`). AC-3 PASS (`dart format` clean, `flutter analyze` no issues, `flutter test` 1/1 — all reran independently by the verifier; `flutter build apk --debug` → `app-debug.apk` 150,419,440 bytes. The APK path was permission-blocked in both agent sessions and was confirmed directly by the owner on 2026-08-22). AC-4 PASS (fresh-context verifier: no legacy Dart/schema/test, no non-Android platform tree, `.metadata` `migration.platforms` = root + android only, no residual Groovy, no empty orphan package dir; every scaffold config byte-diffs to stock Flutter 3.47.1 apart from the three intended identity edits). Surviving set = 9 entries (`.git .claude .idea .githooks .gitattributes docs LICENSE KNOWN_ISSUES.md README.md`), owner-approved 2026-08-22 — this deliberately exceeds the card's literal four-entry list. Rollback `~/mvp001_rollback_2026-08-22/` (tarball rehearsed byte-faithful before the destructive step). Not committed or pushed. |
| 2026-08-24 | DEC-004 | In Progress | Shortlist prescreen only; no name chosen, so no clearance item is cleared for a final name. Google Play — NOT VERIFIED (title triage across 12 candidates by two independent methods; lead `Kimatta` shows no titled match, but Play's own search under-reported at least one real listing — `com.orangefox22.Kondate`, a meal-planning app — so a null is "not found by two methods", not "absent"; package-ID availability for `app.<name>` is unprovable read-only, so every package-ID cell for a screened name reads `NOT VERIFIED`. The listing-URL 404 plus no-index-hit observation is recorded for the lead `app.kimatta` alone — see the card's **Kimatta** evidence block; no per-name package-ID observation is recorded for the other ten screened names. Play Console was not used). Apple App Store — NOT VERIFIED (iTunes Search API, the 11 screened names; `Kimatta` no match; exact out-of-category titles found for Shitaku/Provi/Norra; in-category titles found for Kondate). Web/search — NOT VERIFIED (`Kimatta` first page is dictionary entries only; `kimatta.app` is unregistered per RDAP, `kimatta.com` registered 2004-06-01; no domain purchased). Trademark — NOT VERIFIED (`Kimatta` only; tmsearch.uspto.gov returned HTTP 200 / 125,660 bytes of SPA shell with zero query-term occurrences, API paths 405 and 404 — receipt in the card; secondary web search found no record, indicative only; authoritative search needs an interactive session or a paid search and is routed to the owner). Rejected at Pass 1: Kondate, Savora (both in-category Play title collisions). Register, dispositions and per-name evidence in the card. This row is edited in place when the card resolves — flip Result to `Done` and each criterion to `PASS`, `FAIL`, or `NOT VERIFIED`; do not append a second DEC-004 row. Only the package-ID criterion is eligible to stay at `NOT VERIFIED` at Done, and only with the owner's dated, attributed acceptance of the residual risk recorded in this same cell as `owner-accepted YYYY-MM-DD`, per D-027 and `docs/task/README.md`. The trademark criterion is outstanding owner work, not an exception case, and blocks Done until performed. |

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
flutter test --coverage

# Domain/data coverage gate — 100% required, lib/features is reported only, not gated
lcov --extract coverage/lcov.info 'lib/domain/*' 'lib/data/*' -o coverage/domain_data.info
lcov --summary coverage/domain_data.info --fail-under-lines 100
# ^ exits non-zero on <100% coverage; the runner must treat that as a build failure, not merely a log line
# (confirm the flag is accepted by the installed lcov version on first run; if rejected, fall back to:
#  lcov --summary coverage/domain_data.info | grep -q '100.0%' || exit 1)
genhtml coverage/domain_data.info -o coverage/html

# Domain independence — fails if lib/domain imports Flutter or Firebase
! grep -rE "^import 'package:(flutter|cloud_firestore|firebase_[a-z_]+)/" lib/domain

flutter build apk --debug
```

`package:glados` (dev-dependency) provides property-based/adversarial tests for the domain functions named in the DEC-002 resolution's decision 3 (unit conversion, shopping aggregation, restriction matching, planning-cycle date math). The domain-independence grep and the coverage gate are shell steps, not packages — see DEC-002 resolution decision 4 for the upgrade path (`clean_code_lints`/`import_lint`) if a second layering rule is ever needed.

## Calibration and post-launch

After the first three implementation cards (`MVP-001` through `MVP-003`), review actual duration, context load, test yield, and remediation count; resize later cards only if evidence warrants it.

Post-launch begins with iOS feasibility/tooling and the iOS client, then follows PRD phases 1.5, 2, 2.5, and 3. Other candidates include improved recommendation calibration, richer collaboration, dedicated local storage only if measured need emerges, deterministic imports beyond OPT-001, paid delegation/LLM conveniences, and desktop experiments. Promote a candidate only by adding a bounded card and dependencies here.
