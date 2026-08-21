# Meal Mate Roadmap

This is the durable operational source of truth after the MVP as well as during it. The PRD defines product intent; task cards define bounded work; this file records sequence, status, gates, decisions, blockers, and evidence.

## Current milestone

**Milestone:** Foundation readiness  
**Next action:** Complete `PRE-001`. `DEC-001` is Done (2026-08-21); its naming decision is deferred to `DEC-004`, now `Ready` and owner-paced. `DEC-004` directly blocks `MVP-018`, `MVP-021`, and `MVP-022`, and therefore transitively gates `MVP-019` and `MVP-020` — nothing from `MVP-018` onward can start until it is Done. Do not implement the app until `PRE-001` is Done.  
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
| PRE-001 | Ready | Android toolchain readiness | — |
| MVP-001 | Draft | Clean-room Flutter scaffold | DEC-001, PRE-001 |
| DEC-002 | Draft | Engineering-experience foundation | MVP-001 |
| MVP-002 | Draft | Domain spine and repository seams | MVP-001, DEC-002 |
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
| DEC-004 | Ready | Naming clearance and production identity | DEC-001 |
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

## Evidence log

| Date | Card | Result | Evidence |
|---|---|---|---|
| 2026-08-21 | DEC-001 | Done | Resolution recorded in the card; decisions 2–3 final, decision 1 descoped to DEC-004. Clearance for `MealMate`: Play package ID — FAIL (`com.mealmate.app` search-index entry; listing URL now 404, i.e. indexed then unpublished, and Play never releases a published ID). Play title — FAIL (≥5 apps titled MealMate). Web/search — FAIL (mealmateco.com, same concept). Trademark — NOT VERIFIED (web-sourced USPTO 97352318, appliances class; no authoritative TESS search). Apple App Store title — NOT VERIFIED (not performed). |

## Calibration and post-launch

After the first three implementation cards (`MVP-001` through `MVP-003`), review actual duration, context load, test yield, and remediation count; resize later cards only if evidence warrants it.

Post-launch begins with iOS feasibility/tooling and the iOS client, then follows PRD phases 1.5, 2, 2.5, and 3. Other candidates include improved recommendation calibration, richer collaboration, dedicated local storage only if measured need emerges, deterministic imports beyond OPT-001, paid delegation/LLM conveniences, and desktop experiments. Promote a candidate only by adding a bounded card and dependencies here.
