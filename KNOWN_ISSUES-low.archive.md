# Known Issues Archive

## Archived 2026-09-13

### master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-redteam-pass2-2026-08-24T1320-5bbe.md

#### LOW

- **The intra-file line-number-anchors entry's title said "Four" while the entry listed six** (`KNOWN_ISSUES.md`, the intra-file line-number-anchors entry) -- the bold title reads "Four new intra-file line-number anchors in a card guaranteed to be re-edited" while the parenthetical lists six (`:55`, `:86`, `:90`, `:301`, plus `:126` and `:150`), and the body still says "the card now self-references by line number in four places" and "All four resolve correctly today". Retitling was avoided deliberately to protect the grep-by-title dedup the next redteam pass relies on, but that grep keys on the distinctive phrase "intra-file line-number anchors in a card guaranteed to be re-edited", which survives dropping the numeral. Fix: drop the count from the title and update the two "four" occurrences in the body to match the six-item list, or state the count once in the body only. Deferred: cosmetic; the body's list is authoritative and correct.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The title drops the numeral and the body now reads "six places" / "All six resolve". The six-item set this entry quotes was itself wrong: `:126` is not an anchor site, and the real sixth is the line-wrapped "48 is discharged for the chosen name" at `:157`. Both corrected in the same edit; the citation is now the six quoted phrases, not line numbers.

### master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-fix-findings-2026-08-24T1442-970e.md

#### LOW

- **`:315`'s "passed every check that could be performed read-only on both stores" still overreaches for the Play package-ID check** (`docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md:315-316`, cf. `:132`, `:167-168`) -- the rewritten conclusion says `Kimatta` "passed every check that could be performed read-only on both stores", but a Play check *was* performed read-only and did not pass: `:167-168` records the `app.kimatta` listing-URL 404 plus a null index search and concludes "per the rule above this is `NOT VERIFIED`, not `PASS`", and the table at `:132` reads "NOT VERIFIED (unprovable read-only)". The qualifier only holds if "could be performed" means "could be *conclusively* performed", a distinction carried solely by the table gloss two hundred lines earlier. The section's "Open items for resolution" (`:326-328`) also omits the package-ID item, though the D-027 residual-risk discharge at `:86-91` turns on it. Fix: narrow to "every store check that read-only methods can settle", or add the package-ID item to the open-items list. Deferred: the table and note 2 both state it correctly; prose precision only.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The conclusion now reads "passed every store check that read-only methods can settle" and states the `app.kimatta` package-ID check separately as `NOT VERIFIED` per note 2; that item is also added to the "Open items for resolution" list, so the section is self-contained.

### master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-fix-findings-2026-08-24T1537-7fba.md

#### LOW

- **The re-anchor substitution at `KNOWN_ISSUES.md:31` left a lowercase sentence start** (`KNOWN_ISSUES.md`, the "Done evidence lives entirely outside version control" entry) -- the entry reads "...would block the logs from being committed as-is. the Status contract in `docs/ROADMAP.md` makes recorded evidence the condition for Done...". The replaced token was `` `docs/ROADMAP.md:13` ``, sentence-initial and rendered as a code span, so capitalization was carried by the backticks; substituting the lowercase phrase left the sentence starting lowercase. Introduced by the 2026-08-24 re-anchor pass -- `git show HEAD:KNOWN_ISSUES.md` starts that sentence with the code span. Fix: capitalize to "The Status contract in `docs/ROADMAP.md` makes...". Deferred: cosmetic, one character.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The sentence now reads "...as-is. The Status contract in `docs/ROADMAP.md` makes...".

- **The line-anchors entry's "Fix:" clause still proposes replacements for only 3 of the 6 sites it now lists** (`KNOWN_ISSUES.md`, the intra-file line-number-anchors entry) -- the 2026-08-24 pass edited the body from four sites to six ("six places", "All six resolve correctly today", six quoted phrases) but left the trailing clause naming three string anchors ("the locked constraint forbidding Play Console identifier testing", "the `External actions` field", "the trademark clearance-checklist item"), covering lines 32, 15 and 49. Nothing is proposed for line 33 (the rename-churn bullet, `DEC-004:55`) or for either line-48 site (the Web/search clearance criterion, `DEC-004:150` and `:156-157`). The clause is now internally inconsistent with the body it sits under rather than merely incomplete. Fix: add the two missing anchors ("the locked constraint on repo-wide rename churn", "the Web/search clearance criterion"), or reword to "replace all six with string anchors, e.g. ...". Deferred: the Fix clause is advisory, not a count claim; the citation and body are correct.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The clause now reads "replace all six with string anchors, e.g. ...", so the three named anchors are explicitly a sample rather than the full set.

- **"(wrapped across a line break, which is why grep sweeps miss it)" is attached to the one phrase that does not wrap** (`KNOWN_ISSUES.md`, the intra-file line-number-anchors entry) -- the citation lists `"48 is discharged for the chosen name" (wrapped across a line break, which is why grep sweeps miss it)`, but that phrase greps cleanly: `grep -Fc "48 is discharged for the chosen name"` against `DEC-004` returns 1. What wraps is the reference it belongs to -- `DEC-004:156` ends "... their Play and Apple rows suggest. Line" and `:157` opens "48 is discharged for the chosen name at resolution". The anchor was chosen precisely because it is the non-wrapping form, so the parenthetical tells a future maintainer that the one greppable anchor in the set is ungreppable. Fix: move the parenthetical onto the reference it describes rather than the quoted anchor. Deferred: misleading annotation only; the anchor itself is correct and resolves to one hit.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The note moved off the quoted anchor and onto the reference it describes, which is what actually wraps; it is written without line numbers, since the entry is OPEN and the same change set removed this file's sibling numeric anchors into `DEC-004`.

- **"was performed read-only and did not pass" frames a categorically unprovable item as a failed check** (`docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md`, the "Separately, the `app.kimatta` package-ID check" sentence in "Where this leaves DEC-004") -- note 2 is titled "**A Play package-ID cell can never read `PASS`.**", so no read-only check could have passed; "did not pass" describes a foregone conclusion in the grammar of an adverse result. The card's own observations point the other way -- "`app.kimatta`: listing URL HTTP 404; Play index search for the string returned 50 tiles, none matching" is weak evidence toward availability. The sentence sits three lines after the same paragraph says the preferred scheme "is intact and needs no fallback", so a resolution-step reader may read "did not pass" as a reason to abandon `app.kimatta`. Fix: "cannot be settled read-only at all and therefore stands at `NOT VERIFIED` per note 2, notwithstanding a 404 listing URL and a null index search". Deferred: the same sentence's "per note 2" clause corrects it for an attentive reader; prose precision.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The sentence now reads "cannot be settled read-only at all and therefore stands at `NOT VERIFIED` per note 2, notwithstanding a 404 listing URL and a null index search".

- **The two-versus-three-segment application-ID shape is not in "Open items for resolution"** (`docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md`, the "Open items for resolution" list in "Where this leaves DEC-004") -- the stated rationale for adding the package-ID item to that list was self-containment, but by the same standard the segment-shape choice belongs there: the preceding paragraph says "three segments is the more common convention, and that remains a judgment call for the resolution step", and the table preamble records the three-segment string as unscreened and names it the Completion-criteria fallback shape. It is part of decision 1 (name plus production application ID), so the list's closing "and decisions 2–4 of this card" does not sweep it in. The list names three of the four things the resolution step must settle. Fix: add "the two- versus three-segment shape for the application ID, the three-segment form being unscreened", or drop the self-containment rationale. Deferred: the preceding paragraph states it two lines earlier; completeness only.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. Added to the list. The same edit also added decision 1 itself, which was likewise absent, and kept the trademark search as the leading independently-enumerated item because it is the one item that blocks Done.

### master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-2026-08-24t2107-8195-2026-08-24T2111-0bd4.md

#### LOW

- **`_probe` only catches `KimattaError`; any other failure is an unhandled async error** (`lib/main.dart`, `_HealthScreenState._probe`) -- a non-`KimattaError` bridge failure (panic, codec mismatch) escapes the `on KimattaError` clause and the screen keeps showing `probe: not probed`, so a bridge fault reads as "button does nothing". Fix: add a bare `catch (e)` branch rendering `'unexpected: $e'`. Deferred: spike-only screen, removed by MVP-003.
  **Status:** RESOLVED 2026-08-28 (MVP-003 removed HealthScreen and the spike main())

- **Widget test skips the `hasError` branch of the health `FutureBuilder`** (`test/widget_test.dart`; `lib/main.dart` `HealthScreen.build`) -- the test feeds a completed `report` only; the `'health error: ...'` rendering and the `KimattaError_Storage` variant are touched by no test. Fix: one extra `testWidgets` with `report: Future.error(const KimattaError.storage(message: 'x'))`. Deferred: card scope only requires a fake-fed widget test; screen goes with MVP-003.
  **Status:** RESOLVED 2026-08-28 (MVP-003 removed HealthScreen and the spike main())

- **Spike-only `ensureSemantics()` left in production `main()`** (`lib/main.dart`, `main()`) -- `SemanticsBinding.instance.ensureSemantics()` was added so `uiautomator dump` could locate the probe button for the AC-5 screenshot; tagged `ponytail:` with a removal note. Fix: delete with `HealthScreen`. Deferred: harmless, removal is bundled with the screen.
  **Status:** RESOLVED 2026-08-28 (MVP-003 removed HealthScreen and the spike main())

### master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp002-2026-08-24T2213-6556.md

#### LOW

- **`reopen_is_idempotent` writes a pid-named file to the shared temp dir and only cleans up on success** (`rust/crates/kimatta-storage/src/lib.rs:131-144`) -- the path is `std::env::temp_dir().join(format!("kimatta-{}.db", std::process::id()))` and `std::fs::remove_file` is the last statement, so any panic or failed assertion earlier in the test leaves the database behind. A later run in a process reusing that pid (Linux pids wrap at 32768 by default) reopens a database already containing household `h`, and `insert_household(...).unwrap()` panics on the primary-key conflict -- a failure unrelated to the code under test. The `tempfile` dev-dependency was deliberately skipped as speculative, but the hand-rolled substitute lacks the drop-guard that makes it safe. Fix: make the filename unique per run (nanosecond timestamp or atomic counter), or hold the file in a small guard struct whose `Drop` removes it. Deferred: self-inflicted flake only, needs a stale file plus a pid collision.
  **Status:** RESOLVED 2026-08-25 -- fixed rather than deferred. The test now takes a `tempfile::tempdir()`; `TempDir::drop` removes the directory even on panic, so neither the collision nor the litter is possible. `tempfile` added as a dev-dependency of `kimatta-storage`.

- **Dart test asserts two unrelated behaviors under a name describing only one** (`test/bridge_native_test.dart:15-26`) -- `test('typed Rust error is matchable in Dart')` now also creates a temp directory and asserts `report.schemaVersion == 1`, the assertion carrying MVP-002 AC-4's "bridge test now asserting schema version 1 on a real file" evidence. Its sibling at `:11` (`core_version crosses the bridge`) follows one-behavior-per-test, so this departs from the file's own convention rather than following a house style; a schema-version regression reports under a name about typed errors, and the two assertions have independent failure causes. Fix: split into a second `test('health_check migrates a real database to schema v1')`, alongside the `KimattaError_Storage` test the pass-1 MEDIUM asks for. Deferred: cosmetic; fold in with the `Storage` test.
  **Status:** RESOLVED 2026-08-25 -- fixed rather than deferred. Split into `health_check migrates a real database to schema v1`, alongside the new `storage failure surfaces as KimattaError_Storage`; `flutter test` is now 5/5.

### master -- 2026-08-25


#### LOW

- **The production database path has no automated test carrier** (`lib/features/settings/health_provider.dart`, `healthReportProvider`) -- `getApplicationSupportDirectory()` is called once inside the provider, which every test overrides, so nothing catches a regression to the path source. A `PathProviderPlatform` mock would assert the mock rather than the device, so no useful unit test exists. The pre-fix value (`Directory.systemTemp` → `/data/local/tmp` on Android) was unwritable by an app uid, so the regression is silent-at-build-time and fatal at runtime. MVP-002 AC-3's on-device evidence is the only verification; it passed once on 2026-08-25 (`emulator-5554`), but that is a point-in-time check, not a regression guard. Fix: covered indirectly once MVP-004's persistence evidence exercises a real write; MVP-003's card carries a load-bearing constraint to keep the path app-private when it replaces `main()`. Deferred: `HealthScreen` and the spike `main()` were deleted at MVP-003; the path source moved to `healthReportProvider`, which every test overrides, so the gap survives the move. Re-verified on-device 2026-08-28 (MVP-003 Settings screenshot + run-as listing); still no regression guard — tests override the provider.
  **Status:** RESOLVED 2026-08-28 -- MVP-004 AC-3 PASS: a real household write landed at `/data/user/0/dev.mealmate.temp/files/kimatta.db` on a post-`pm clear` install and survived a force-stop relaunch (ids identical). Still a point-in-time on-device check, not a unit-test guard.

### master -- 2026-08-25

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-mvp003-fixfindings-2026-08-25T2013-264a.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-fixfindings-2026-08-25T2234-cce8.md

#### LOW

- **Step 7's `test/app_test.dart` harness carries `retry:` with no Riverpod-2 fallback** (`~/.claude/plans/goofy-juggling-stroustrup.md:459`, vs the fallback instruction at `:57`) -- the MVP-003 plan's Risks row reads "Confirm at step 3: `ProviderScope(retry:)` exists (Riverpod >= 3.0) -- if the resolved major is 2.x, drop the `retry` argument". Singular, and checked at step 3, when the only occurrence is the one step 6 writes into `lib/main.dart` (`:442`). `test/app_test.dart` does not exist until step 7, and its harness at `:459` repeats `retry: (_, _) => null`, so on a 2.x resolution the executor drops it from `main.dart` and copies it into the test verbatim. Fix: have `:57` name both call sites (`lib/main.dart` and the step-7 harness), or note it inline at `:459`. Deferred: fails loudly at compile time, costs one cycle, and only on a 2.x resolution -- which `flutter pub add` at step 3 is expected to avoid.
  **Status:** RESOLVED 2026-08-25 (fixed in the MVP-003 plan, rev 5; both widened by a sibling sweep — see redteam-impl-handoff-master-mvp003-fixfindings-2026-08-25T2234-cce8.md)

- **`NavigationBar` clamps label text scale to 1.3, so the plan's overflow contingency cannot fire** (`~/.claude/plans/goofy-juggling-stroustrup.md:63` Risks row, `:481` test 7, `:124` the AC-2 evidence wording) -- the row reads "`NavigationBar` labels at text scale 2.0 may overflow ... If it overflows, `labelBehavior: onlyShowSelected` -- recorded as a theme decision." The framework forecloses it: labels are wrapped in `MediaQuery.withClampedTextScaling(maxScaleFactor: _kMaxLabelTextScaleFactor)` = 1.3 (`packages/flutter/lib/src/material/navigation_bar.dart:31`, `:506-512`) and the bar's height "does not adjust with ... [MediaQueryData.textScaler]" (`:210-213`). At `textScaleFactorTestValue = 2.0` the bar renders at 1.3 and cannot overflow, so the `labelBehavior` fallback is dead prose and the AC-2 evidence line at `:124` ("text scale 2.0") overstates what test 7 covers for the shell chrome; the placeholder bodies and `AppBar` titles it also exercises do reach a genuine 2.0, and those are the parts that can overflow. Fix: replace the Risks row with the clamp citation (a resolved non-risk worth keeping so a later card does not re-derive it) and word AC-2's evidence as "destination screens at text scale 2.0; `NavigationBar` labels clamp at 1.3 by framework design". Deferred: no behavioural consequence -- dead contingency prose plus a slightly overstated evidence line.
  **Status:** RESOLVED 2026-08-25 (fixed in the MVP-003 plan, rev 5; both widened by a sibling sweep — see redteam-impl-handoff-master-mvp003-fixfindings-2026-08-25T2234-cce8.md)

### master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-mvp003-redteam-pass1-2026-08-26T0847-3c5a.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-redteam-pass1-2026-08-26T0928-3cb9.md

#### LOW

- **The `font_scale` set/reset window has no trap and no acknowledgement anywhere in the plan** (`~/.claude/plans/goofy-juggling-stroustrup.md`, the `settings put system font_scale 2.0` / `1.0` pair in step 9, and the On-device bullet of the Verification summary) -- MVP-003 plan step 9 sets `settings put system font_scale 2.0`, captures `settings_scale2.png`, and resets to `1.0`, with no trap between them. Any interruption in that window leaves the emulator at font scale 2.0 for every later session -- a host-state side effect that outlives this card and is inherited by MVP-004 onward. The MVP-003 fix pass recorded this as deferred out-of-scope and stated it was "recorded in the plan's Verification section as 'noted, not fixed'", but it is not: the Verification summary's On-device bullet mentions only "font_scale 2.0 screenshot". (The pass-1 wording also cited a `grep -n 'noted'` hit whose referent the rev-7 pass rewrote; that clause is struck rather than re-anchored.) The acknowledgement lived solely in the handoff doc, which is disposed after review -- the same missing-carrier failure the `KNOWN_ISSUES.md` MEDIUM titled "`docs/ROADMAP.md` contradicts itself on MVP-002 AC-3, and the deferred prose sweep has no carrier" exists to prevent, which is why this entry is the carrier. Fix: wrap the pair (`trap '... settings put system font_scale 1.0' EXIT` before the `font_scale 2.0` line, cleared after the reset), or add one sentence to the plan's Verification summary recording it as a known accepted window; if step 9 has already run, verify the emulator is back at 1.0 before the next evidence card. Deferred: evidence-integrity risk is nil (the four commands are adjacent and all `exit 1` gates in the block sit after the reset), but the host-state risk is durable and needed a home outside the disposed handoff.
  **Status:** RESOLVED 2026-08-28 (MVP-003 step 9 wraps the 2.0/1.0 pair in a `trap … EXIT` and asserts `settings get system font_scale` reads 1.0 before the fragment ends)

### master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-mvp003-fixfindings-rev6-2026-08-26T1010-5279.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-fixfindings-rev6-2026-08-26T1343-37fc.md

#### LOW

- **The new `font_scale` deferral has no closure trigger in the MVP-003 plan's step-10 KI-low reconciliation** (`~/.claude/plans/goofy-juggling-stroustrup.md:559`, against the entry this file carries at `:175`) -- step 10 names four `KNOWN_ISSUES-low.md` entries by title and states exactly what happens to each at card close: three flip to `RESOLVED`, one stays `OPEN` with a re-anchor and an appended re-verification note. The `font_scale` entry written by redteam pass 1 is not among them (`sed -n '559p' goofy-juggling-stroustrup.md | grep -c font_scale` -> 0), so once MVP-003 executes it stays `OPEN` indefinitely with a Fix ("wrap the pair ... before `:517`") that is only actionable *before* execution. The half that stays actionable is its own trailing clause -- verify the emulator is back at font scale 1.0 -- and step 10 is where that check belongs. Correction to the pass-1 premise: `~/.claude/plans/` retains executed plans back to 2026-08-11, so the entry's `:517-521` / `:570-574` anchors do not dangle after execution; the line-neutrality constraint bought more than was claimed. Fix: add one clause to `:559` in the same shape as the "production database path" entry it already handles -- the `font_scale` entry stays `OPEN` and gains `Emulator confirmed back at font_scale 1.0 after the MVP-003 step-9 run (adb shell settings get system font_scale)`, or flips to `RESOLVED` if step 9 ends up carrying the `trap`. Deferred: ledger hygiene only, no behavioural consequence, and the entry's own trailing clause keeps it actionable in the meantime.
  **Status:** RESOLVED 2026-08-26 (fixed in the MVP-003 plan / pass-1 review doc)

- **The pass-1 review doc's Disposition pointer was computed before the Disposition itself shifted the lines it cites** (`/home/davidlinux/.claude/reviews/redteam-mvp003-redteam-pass1-2026-08-26T0928-3cb9.md:28`) -- the Disposition paragraph warns "Note for anyone re-running the fence suite at `:42-45`". In the current file `:42-45` is the "Cut markers" bullet (`android:configChanges` / `fontScale`); the fence suite is at `:56-59`. The cause is mechanical and self-inflicted: the Disposition occupies `:18-30` plus a blank line = 14 inserted lines, and 56 - 14 = 42, so the anchor was measured against the pre-insertion file. The same fix pass adopted line-count neutrality as a hard design constraint for the plan it was editing and did not apply the same reasoning to the document it was inserting into. The warning is the durable artifact for the next reviewer of this chain and it points 14 lines off. Fix: change `:42-45` to `:56-59`, or replace the number with the bullet's bold title (`**Verification suite, scoped to the fence**`), which cannot shift. Deferred: cosmetic misdirection in a review doc that `/tidy` eventually archives; costs a future reader one grep.
  **Status:** RESOLVED 2026-08-26 (fixed in the MVP-003 plan / pass-1 review doc)

- **MVP-003 step 9's `timeout: 600000` headroom was computed for `up` alone, not for the four commands sharing its tool call** (`~/.claude/plans/goofy-juggling-stroustrup.md:501-509`) -- the timeout was sized against `cmd_up`'s worst case: `LOCK_WAIT` 300s (`tools/emulator.sh:26`, `flock -w` at `:102`) + `BOOT_TIMEOUT` 180s (`:22`, polled at `:141-146`) + ~10s port polling at `:111` ~= 490s, against the 600000ms Bash-tool ceiling. But fragment 1 does not end at the gate: `:504`-`:509` then run `adb install -r` of a Flutter debug APK (`docs/ROADMAP.md:150` records the PRE-001 artifact at 150MB) over the WSL->Windows TCP bridge, `am start`, a 3s device-side sleep, `screencap` and `wm size` -- plausibly 30-90s more. 490 + 90 ~= 580s leaves ~20s of margin against a ceiling that cannot be raised, and blowing it kills the fragment *after* a successful bring-up, losing `plan.png` and the `wm size` output CUT 1 needs. Fix: split the gate into its own tool call so `up` gets the full 600s and the install/launch/capture sequence gets a fresh one; reword the existing `:497` "three tool calls" comment rather than extending it, to preserve line-count neutrality. Deferred: the worst case still fits, the failure mode is an idempotent rerun (`status` short-circuits, `install -r` re-installs) rather than wrong evidence, and the fix restructures the fragment split.
  **Status:** RESOLVED 2026-08-26 (fixed in the MVP-003 plan / pass-1 review doc)

### master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-v3-card-rederivation-2026-08-26T1337-0211.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-v3-card-rederivation-2026-08-26T1626-ad3a.md

#### LOW

- **MVP-025 hands a beam-width verdict to a card that will already be Done, with no re-open path** (`docs/task/mvp/MVP-025_PLANNER_FIXTURES_AND_INVARIANT_TESTS.md:66`, restated at `:60`) -- MVP-025's decision gate says "If the benchmark shows the shipped beam width or candidate limit is wrong, the change belongs to `MVP-023`", and MVP-023's own gate (`docs/task/mvp/MVP-023_FOOD_CONTROLLER_PLANNER_CORE.md:74`) says "`MVP-025`'s benchmark may revise them". But `docs/task/SEQUENCE.txt` steps 32-35 run MVP-023 to completion before MVP-025 starts, and LOCAL-CORE-LOOP-READY (`docs/ROADMAP.md:23`) requires both Done. Neither card names the mechanism for re-opening a Done card. D-031's Status contract covers changes to "sources, decisions, product scope, architecture, or declared dependencies" -- a benchmark verdict overturning a recorded choice arguably qualifies, but no card says so, so the handover is an implied transition rather than a defined one. Fix: one sentence in MVP-025's gate stating that an adverse verdict returns MVP-023 to Draft under D-031 with the benchmark row as the trigger. Deferred: SEQUENCE ordering makes this a process nicety with no live sequencing hazard.
  **Status:** RESOLVED 2026-08-26 -- MVP-025's gate now states the D-031 Draft return with the AC-4 benchmark row as trigger (D-036). The MVP-024 AC-8 escalation was deliberately placed in MVP-024's own Stop/failure conditions rather than here, because MVP-025 is Done before AC-8 can fire.

- **MVP-023 requires starter meals as a candidate source but does not depend on MVP-011** (`docs/task/mvp/MVP-023_FOOD_CONTROLLER_PLANNER_CORE.md:10`) -- MVP-023's scope names all seven PRD v3 §9.3 candidate sources including "starter meals", and the 2026-08-26 re-derivation added the reciprocal coupling to MVP-011 (`docs/task/mvp/MVP-011_STARTER_CONTENT_READINESS.md:35`: "every shipped recipe must satisfy the same structured shape MVP-023 hard-filters and scores"). MVP-023's `Depends on` is `MVP-002, MVP-009, MVP-012, MVP-014`; MVP-011 is not reachable transitively (MVP-009 -> 006/007/008; MVP-012 -> 004/005/007; MVP-014 -> 003/004/007). A bidirectional content coupling was created without either card's `Depends on` or register row moving -- the same D-029 obligation the pass honored for MVP-004's added MVP-003 dependency. Fix: add MVP-011 to MVP-023's `Depends on` (card cell + `docs/ROADMAP.md` register row together), or state in MVP-023's scope that the starter-meal candidate source is implemented against the entity shape and does not require shipped starter content to exist. Deferred: `docs/task/SEQUENCE.txt` already orders MVP-011 (steps 20-21) before MVP-023 (steps 32-33), so this is declaration hygiene with no sequencing hazard.
  **Status:** RESOLVED 2026-08-26 -- took the decoupling option (D-036): MVP-023's scope now states the starter-meal candidate source is implemented against the shared entity shape and does not require MVP-011's shipped content, so no `Depends on` edit was needed.

- **Stray blank lines split two Load-bearing constraint lists into separate markdown lists** (`docs/task/mvp/MVP-011_STARTER_CONTENT_READINESS.md:37` and `docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md:36`) -- in both cards the constraints prepended by the 2026-08-26 re-derivation are separated from the pre-existing bullets by a blank line, which renders as two `<ul>` blocks with extra vertical space rather than one list. Every other card the same pass touched appended constraints without the gap, so the two are inconsistent with the pass's own output. Fix: delete the blank line in each. Deferred: cosmetic rendering only.
  **Status:** RESOLVED 2026-08-26 -- both blank lines deleted (D-036).

### master -- 2026-08-27

Source: /home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-plan-rev8-2026-08-27T1321-869f.md

#### LOW

- **Step 1b implements one of the three obligations `docs/task/README.md:66` places on the Elevated fresh-context verifier** (the MVP-003 plan's step 1b, "Fresh-context verifier (README Elevated tier)", against `docs/task/README.md:66`) -- the plan cites the README Elevated tier as 1b's authority and calls it "a **gate**, not a formality". README:66 reads: "Elevated: tests should be derived from acceptance criteria before or independently from implementation when practical; a fresh-context verifier reruns them, investigates failures, and checks material coverage gaps." Step 1b instructs the subagent to rerun `cargo test --workspace` and `(cd rust && cargo build --release) && flutter test`, confirm one AC-3 line in `docs/V3_IMPLEMENTATION_STATUS.md`, and report pass counts. Failure investigation is covered implicitly by "Any failure -> stop", but the material-coverage-gap check is absent -- and that is the half a rerun cannot substitute for, on the card whose promotion to `Done` the whole of step 1c hangs on. Fix: add one clause to the subagent brief -- check MVP-002's four ACs against the tests that claim them and report any criterion with no test carrier -- or state in the plan why a coverage check is not re-run at promotion time (the card already recorded 4/4 PASS at Verify), so the divergence from README:66 is deliberate rather than dropped. Deferred: the rerun half is the load-bearing half at promotion time, and the owner already exercised the coverage judgement when the card reached Verify.
  **Status:** RESOLVED 2026-08-28 (fixed in the MVP-003 plan, rev 9–11) -- moot rather than implemented: step 1b was removed entirely, and the divergence from `docs/task/README.md:66` is now recorded in the plan's Context section as a deliberate, visible acceptance. No coverage-gap check was added.

- **The MVP-003 plan's step-7 `setUp` line is uncompilable as written; only the `tearDown` half carries the correction** (the plan's step 7, the line beginning "`setUp`: `tester.view.physicalSize`") -- the line reads "`setUp`: `tester.view.physicalSize = const Size(1080, 2400); tester.view.devicePixelRatio = 2.75;` -- `tearDown`: `...` (done via `addTearDown` inside each `testWidgets`, since `tester` is per-test)." The parenthetical corrects only the tearDown half, but `tester` is equally out of scope in a `setUp` callback: both halves must live inside each `testWidgets` body. An executor transcribing the literal text writes a `setUp` that does not compile. Fix: restate as "at the top of each `testWidgets` body: set `physicalSize`/`devicePixelRatio`, then `addTearDown(...)` for the four resets" -- one sentence, no `setUp`/`tearDown` framing. Deferred: `flutter analyze` catches it within one step of being written.
  **Status:** RESOLVED 2026-08-28 (fixed in the MVP-003 plan, rev 9–11) -- step 7 now reads "at the top of each `testWidgets` body", with neither `setUp` nor `tearDown` framing.

- **The MVP-003 plan's step 10 marks both `SEQUENCE.txt` MVP-003 steps `Done`, but step 3 is a conditional that will not have run** (the plan's step 10, the `docs/task/SEQUENCE.txt` bullet anchored on `MVP-003_ANDROID_APP_SHELL_NAVIGATION.md`, against `docs/task/SEQUENCE.txt` steps 3 and 4) -- the two hits for the card filename are step 3, `[ONLY if no accessible current approved MVP-003 plan is supplied] /plan-task ...`, and step 4, `/execute-plan`. This plan is the supplied approved plan, so step 3's guard is false and the step never runs; marking it `# Done <TODAY> -- see the docs/ROADMAP.md evidence log.` records a `/plan-task` invocation that did not happen. The plan holds itself to a higher bar elsewhere: step 1c item 9 justifies its `SEQUENCE.txt` step-0 mark with "Step 0 named exactly four targets ... so this mark is honest." Fix: mark step 4 `Done <TODAY>`; mark step 3 with its actual disposition (`not run -- a current approved plan was supplied`), or state in the plan why "Done" is the right mark for a skipped conditional. Deferred: ledger honesty only; both marks retire the same file section and no downstream reader acts on the distinction.
  **Status:** RESOLVED 2026-08-28 (fixed in the MVP-003 plan, rev 9–11) -- though not as this entry proposed: the guard *did* fire. The earlier plan was not resolvable from the repository, so `/plan-task` ran on 2026-08-28 and produced the rev-11 plan that step 4 then executed. Both marks record steps that actually happened.

- **The MVP-003 plan's PRE-002 toolchain re-anchor covers the parenthetical but not the second reference to the deleted test in the same bullet** (the plan's step 10, the bullet anchored on `every Dart test -- the fake-fed widget test included`, against `docs/ROADMAP.md:275`) -- the roadmap bullet names the deleted file twice: "...so every Dart test -- **the fake-fed widget test included** -- needs rustup + the pinned toolchain on the host; **the widget test** avoids *loading* the library, not *building* it." Step 10 instructs only "re-anchor the parenthetical to `test/app_test.dart`", leaving the second clause's bare "the widget test" pointing at a file step 6 deletes. Both references are true of `test/app_test.dart` (it is fake-fed via `ProviderScope` overrides and never calls `RustLib.init()`), so the fix is mechanical -- the instruction just does not reach the second site. Fix: name both sites in the bullet, or state the edit as "replace both occurrences of the deleted test's referent in this bullet with `test/app_test.dart`". Deferred: a one-word staleness in a prose bullet whose substance stays correct; mechanical to fix later.
  **Status:** RESOLVED 2026-08-28 (fixed in the MVP-003 plan, rev 9–11) -- step 10 now names both referents, and both were replaced with `test/app_test.dart`.

### master -- 2026-08-27

Source: /home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-plan-rev9-2026-08-27T1602-03c3.md

#### LOW

- **The MVP-003 plan's font_scale EXIT trap is cleared before the reset is verified** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:623`, against `:625`) -- `trap - EXIT` runs one line before the `settings get system font_scale` assertion that checks whether the trap was needed. That read is `|| true`-guarded, so a transient bridge failure yields an empty string, fails the `= "1.0"` comparison, and exits 1 with the safety net already disarmed and the device's real scale unknown -- durable host state inherited by MVP-004 onward. Fix: move `trap - EXIT` to after the assertion.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-plan-rev9-2026-08-27T1602-03c3.md
  **Status:** RESOLVED 2026-08-27 (fixed in the MVP-003 plan, rev 10: `trap - EXIT` moved below the `settings get system font_scale` assertion, so a failed assertion exits with the trap still armed and the 1.0 reset re-runs)

- **The MVP-003 plan's step-9 gate fragment contradicts the block's own `set -euo pipefail` rule** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:560`, against `:594`) -- the header states "EVERY fragment starts with `set -euo pipefail`" and that the preamble "is REPEATED VERBATIM at the top of every fragment", but the first of four fragments (through CUT 0) has none; the first occurrence is after the CUT 0 marker. Harmless as written -- the single command carries its own `|| { exit 1; }` -- but any line added there inherits no `-e`, the rev-8 condition the comment exists to prevent. Fix: hoist `set -euo pipefail` above the `status || up` gate, or scope the claim to fragments after CUT 0 and say why the gate is exempt.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-plan-rev9-2026-08-27T1602-03c3.md
  **Status:** RESOLVED 2026-08-27 (fixed in the MVP-003 plan, rev 10: `set -euo pipefail` hoisted above the `status || up` gate, and the header's preamble claim rewritten to what is true per fragment -- measured 1/4/3/3 lines)

### orch/6 -- 2026-08-28

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-6-2026-08-28T2309-b6ae.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md

#### LOW

- **The household name field is never resynced to the stored, trimmed value** (`lib/features/household/household_screen.dart:77`) -- `_seedOnce` fills the controller once and never again, but `rename_household` trims, so after saving `"  Casa  "` the header reads `Casa` while the field still reads `  Casa  `; saving whitespace clears the name to NULL and the field still shows the spaces. Fix: reseed from the DTO `renameHousehold` returns, on successful save only.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md
  **Status:** RESOLVED 2026-08-28 -- fix pass: `_save` now resyncs the field from the DTO `HouseholdNotifier.rename` returns, through `_resync`, which also places the caret at the end so a keyboard-driven save does not jump it. Pinned by `the name field resyncs to the trimmed, stored value`; confirmed red pre-fix (field held `  Casa  `).

- **The rename refresh discards the returned DTO and re-enters the write path** (`lib/features/household/household_screen.dart:49`) -- `_save` throws away `renameHousehold`'s `HouseholdDto` and invalidates `householdProvider`, re-running `bootstrapHousehold()`, which mints two throwaway UUIDs and opens an IMMEDIATE (write-lock) transaction just to read. One UPDATE costs two extra reads plus a spinner flash mid-edit. Fix: a read-only `load_household` command, or hold the returned DTO in a Notifier.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md
  **Status:** RESOLVED 2026-08-28 -- fix pass: `householdProvider` is now an `AsyncNotifierProvider`, and `HouseholdNotifier.rename` publishes the DTO the bridge already returned instead of invalidating. The UPDATE no longer costs two extra reads, and the AsyncLoading spinner flash mid-edit is gone. Same change resolves the MEDIUM disposal race in the full review.

- **`ensure_household`'s comment claims cross-caller safety the code does not provide** (`rust/crates/kimatta-storage/src/lib.rs:143`) -- "concurrent callers cannot both create" is bought by the single `Mutex`-owned connection, not by IMMEDIATE; across connections `open` sets no `busy_timeout`, so a second one gets `SQLITE_BUSY` with no retry. A reader who trusts the comment may add a second connection believing the race is handled. Fix: name the single-connection invariant and state the BUSY behaviour.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md
  **Status:** RETRACTED 2026-08-28 -- the premise is false. `rusqlite` 0.40.2 calls `ffi::sqlite3_busy_timeout(db, 5000)` unconditionally in `InnerConnection::open_with_flags` (`inner_connection.rs:118`), the path `Connection::open` takes, so every connection carries a 5-second busy timeout by default. A second connection contending with the IMMEDIATE write lock therefore waits and serialises rather than failing immediately, which is exactly what the comment claims -- so the comment is accurate and `lib.rs:143` is left unchanged. Caught by `/redteam-plan` on the fix plan, which would otherwise have written this same false claim into the source permanently.

- **`rename_household`'s post-rename re-read ignores the id and its error arm is unreachable** (`rust/src/api/household.rs:44`) -- `load_household` returns the oldest row by rowid regardless of which id was renamed, correct only because a second household cannot exist; the `"household vanished after rename"` arm is dead, since zero rows changed already returns `NoSuchHousehold`. Matters when MVP-018 linking arrives. Fix: `load_household_by_id`, or a comment/`debug_assert` naming the invariant.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md
  **Status:** RESOLVED 2026-08-28 -- fix pass: `kimatta_storage::load_household_by_id` added (sharing a `with_members` helper with `load_household`, whose behaviour is unchanged), and `api/household.rs` reads back by id. Escalated from "accepted" because the same pass made the returned DTO load-bearing -- the notifier now stores and displays it. `rename_household` is split over a `rename_in(&mut Connection, ..)` seam so the defect is pinned at its own layer by `rename_reads_back_the_household_it_named`, confirmed red pre-fix (returned `h1` when renaming `h2`). The unreachable arm is kept as a non-panicking guard with a comment stating why it is unreachable and that no transaction spans the UPDATE and the re-read.

- **`tools/emulator.sh seed` is cwd-dependent, ignores its arguments, and cannot detect a failed launch** (`tools/emulator.sh:219`) -- the APK path at `:222` is cwd-relative while every other command is not; `"$@"` is passed to `cmd_seed`/`cmd_reset` at `:232`-`:233` but never read; `am start` at `:224` exits 0 on `Error: Activity not started`, so a launch that never happened reports success. Fix: derive the root from `${BASH_SOURCE[0]}`, drop the unused `"$@"`, use `am start -W` and check its output.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md
  **Status:** RESOLVED 2026-08-28 -- fix pass: APK path derived from `${BASH_SOURCE[0]}` with an explicit missing-APK message; `am start -W` with the output checked for both `^Error` and a missing `Status: ok`, since a resolved-but-dead launch prints `Status: timeout` and no Error line. Dropping `"$@"` alone would have changed nothing observable, so both dispatch arms instead reject extra arguments with exit 2. Verified live against the emulator from `/tmp`: `reset --wipe` exits 2 with a message, `seed` located the APK, and one run genuinely caught `Status: timeout` (old code would have reported success) while the next reported `Status: ok`.

- **Gate-bearing MVP-004 on-device evidence lives outside the repo and cannot be re-verified** (`docs/ROADMAP.md:174`) -- AC-3/AC-4/AC-5 PASS rests on `~/mvp004_evidence/` artifacts; that evidence flipped `KNOWN_ISSUES.md:12` to RESOLVED and carries the `EMULATOR-PERSISTENCE-READY` gate at `docs/ROADMAP.md:22`. Only transcribed ids and mtimes remain in-repo. Fix: paste the raw `run-as ls -l`, airplane-mode and sqlite3 output verbatim into the card's verification record. (Correction: this entry originally said the artifacts were "unreadable from the worktree". They are readable -- the claim came from the handoff and was not checked.)
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md
  **Status:** RESOLVED 2026-08-28 -- fix pass, partially: the AC-3 claim is now in-repo as evidence rather than summary. `docs/task/mvp/MVP-004_...md` carries a "Raw evidence" subsection with the two verbatim `sqlite3` dumps of `kimatta_first.db` / `kimatta_relaunch.db` (schema v1, one household + one member, identical ids across the force-stop), the artifact listing, and the screenshot capture commands. The `run-as ls -l` mtime and `settings get global airplane_mode_on` were live device reads that no pulled artifact carries; they are recorded in a separate block explicitly labelled as transcription, since pasting them as raw output would reproduce the very defect this entry names.

### orch/8 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-8-2026-08-29T0109-5f49.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md

#### LOW

- **`save_planning_cycle` is the only read-then-write transaction that is not IMMEDIATE** (`rust/crates/kimatta-storage/src/lib.rs:302`) -- it reads through `require_household` then writes under a DEFERRED transaction, while `ensure_household` (`:201`) and `ensure_planning_cycle` (`:282`) both use IMMEDIATE. Unreachable behind the single connection at `rust/src/db.rs:9`; a second connection turns it into a 5s busy stall. Fix: `TransactionBehavior::Immediate`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md
  **Status:** RESOLVED 2026-08-29 -- fix pass: `save_planning_cycle` now opens
  `transaction_with_behavior(TransactionBehavior::Immediate)`, matching its two read-then-write siblings, and its
  doc comment states the lock ordering. Pinned by `save_takes_the_write_lock_before_it_reads`, which holds RESERVED
  on a second connection with the busy timeout at zero and asserts the `BEGIN` is refused before
  `require_household` runs; confirmed red pre-fix (`got NoSuchHousehold("absent")`). `insert_household` (`:94`) is
  left DEFERRED deliberately -- it is write-only, so its first statement takes the write lock with no upgrade.

### orch/10 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-10-2026-08-29T0221-5a60.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-10-2026-08-29T0230-130e.md

#### LOW

- **`EMULATOR-PERSISTENCE-READY` gate cell asserts both closed and not closed** (`docs/ROADMAP.md:22`) -- the cell reads `**Not closed:** ... MVP-005 reached `Done` 2026-08-29 -- closed 2026-08-29`, and line 7 still names the closed gate as the current milestone while line 23 records progress on the next one. The next card's selection reads a source of truth that gives two answers. Fix: replace `**Not closed:**` with `**Closed 2026-08-29:**` and advance line 7 to `LOCAL-CORE-LOOP-READY`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-10-2026-08-29T0230-130e.md
  **Status:** RESOLVED -- fixed in the orch/10 fix pass 2026-08-29; both edits applied.

- **No index on `recipe_ingredient_line`'s two ingredient foreign keys** (`rust/crates/kimatta-storage/src/lib.rs:129`) -- `ingredient_id` and `custom_ingredient_id` are deliberately `NO ACTION`, so every `ingredient`/`custom_ingredient` delete full-scans the line table to prove no child references it; they are also MVP-009/MVP-015's join keys. Migration 3 indexes `custom_ingredient(household_id)` and `recipe(household_id)`, so the omission is asymmetric with its own siblings. Fix: add both indexes to migration 3 now, while no shipped database has reached v3; afterwards it costs a fourth migration.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-10-2026-08-29T0230-130e.md
  **Status:** RESOLVED -- fixed in the orch/10 fix pass 2026-08-29; both indexes added to migration 3, pinned by `the_line_ingredient_foreign_keys_are_indexed`.

### orch/12 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-12-2026-08-29T0423-be39.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-12-2026-08-29T0439-99c7.md

#### LOW

- **Restrictions editor edit affordances stay live during an in-flight save** (`lib/features/restrictions/restrictions_screen.dart:88`) -- `_saving` gates only the Save button (:180), so a checkbox, `Add` or chip-delete tapped during the await is silently discarded when `_save` re-seeds `_known`/`_other` from the returned set. Fix: disable the three affordances while `_saving`, or drop the post-save re-seed.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-12-2026-08-29T0439-99c7.md
  **Status:** RESOLVED -- fixed in the orch/12 fix pass 2026-08-29; the checkbox list, `Add` and chip delete are disabled while `_saving`, keeping the post-save re-seed so the form still shows what storage actually kept. Pinned by `the edit affordances are disabled while a save is in flight`, confirmed red pre-fix.

- **Free text duplicating a known restriction token is stored twice** (`lib/features/restrictions/restrictions_screen.dart:190`) -- `_add` dedups only within `_other`, and `is_same` (`rust/crates/food-domain/src/restriction.rs:101`) never collapses a `Known`/`Other` pair, so Peanuts checked plus typed `peanuts` yields `Peanuts, peanuts` in Settings and a double match for MVP-009. Fix: reject free text matching a token or label in `kinds` and tick that checkbox instead.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-12-2026-08-29T0439-99c7.md
  **Status:** RESOLVED -- fixed in the orch/12 fix pass 2026-08-29; `_add` now takes `kinds` and matches the trimmed lowercase entry against each token and its label, ticking that checkbox instead of adding a chip. Pinned by `free text naming a known kind ticks its box instead of adding a chip`, which also covers the already-ticked case, confirmed red pre-fix.

### orch/14 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-14-2026-08-29T0658-3755.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0713-5c00.md

#### LOW

- **Redundant `as bridge` import used once** (`lib/features/recipes/recipes_provider.dart:7`) -- the bridge library is imported twice, plain and prefixed, but only `listArchivedRecipes` (`:90`) uses the prefix while the other six calls do not. Copied from `household_provider.dart:4-7`, where the prefix resolves a real `completeOnboarding` shadowing; there is no collision here. No runtime effect, `flutter analyze` clean. Fix: drop the aliased import, or prefix all six and say why.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0713-5c00.md
  **Status:** RESOLVED -- fixed in the orch/14 fix pass 2026-08-29; the aliased import dropped and `:90` calls `listArchivedRecipes` unprefixed like the file's other six bridge calls. Took the first option, not "prefix all six": `household_provider.dart`'s prefix resolves a real shadowing, and copying it where nothing shadows would spread a pattern that means nothing. No test -- import-only, no reachable behaviour difference; `flutter analyze` rc 0 and the suite unchanged. This entry was the one LOW of the three the reviewer marked `defer:` that was overridden, because deferring it avoids no risk.

### orch/14 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-14-2026-08-29T0750-b94e.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0804-a280.md

#### LOW

- **`_firstErrorKey`'s Servings branch is unexercised** (`lib/features/recipes/recipe_form_screen.dart:217`) -- of the three dispatch arms added by the scroll fix, only title and rows are tested. Nothing asserts `key: _servingsKey` is still attached to the Servings `TextField` (`:333`), so dropping it would make `_revealFirstError` silently no-op on a servings-only rejection with all 152 tests green. Fix: one `usePixel5` test entering `0` in Servings and asserting the error is visible.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0804-a280.md
  **Status:** RESOLVED -- fixed in the orch/14 pass-1 fix 2026-08-29 by `a rejected servings count scrolls its error into view`. Not the one-row test the finding suggested: on the default form the servings error renders inside the fold whether or not the scroll runs, so that version passes with `key: _servingsKey` deleted and pins nothing. The test adds two empty rows -- skipped by `_validate`, so they add height without a competing row error -- which puts Servings above the viewport once `tapVisible` scrolls to Save. Verified non-vacuous by mutation: with the key removed the error sits at **-899.3** against a viewport top of **56.0** and the test fails `error is clipped above the scroll viewport`.

### orch/14 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-14-2026-08-29T0900-dcd3.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0911-2a9d.md

#### LOW

- **A row's inline error survives `_removeLine` and misnumbers the surviving row** (`lib/features/recipes/recipe_form_screen.dart:275`) -- `_LineDraft.error` is cleared only by `_validate`, and the message embeds the index at validation time while `_row` re-derives its header live, so after a rejected save removing row 1 leaves a card headed "Ingredient 1" showing "Ingredient 2: ...". Self-corrects on the next Save. Fix: clear `error` on the remaining drafts inside `_removeLine`, or drop the index from the message.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0911-2a9d.md
  **Status:** RESOLVED (orch/14 pass 1) -- fixed by neither listed option: `_LineDraft.error` now stores the message *without* the `Ingredient N:` prefix (`:58`) and `_row` applies the number from the live index at paint time (`:449`), so a removal renumbers every surviving row automatically. Clearing the survivors' errors was rejected because it would drop rejection signals that are still true for the rows above the removal -- the silent-rejection shape this card already fixed once. Pinned by `Remove ingredient renumbers the errors of the rows below it` (`test/app_test.dart:2450`), which fails pre-fix.

### orch/21 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-21-2026-08-29T1325-fc64.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-21-2026-08-29T1336-81e8.md

#### LOW

- **Catalog-correction comment overclaims propagation** (`rust/crates/kimatta-storage/src/lib.rs:1162`) -- the inline comment says the whole-catalog upsert stops a corrected name being stranded, but the short-circuit at line 1154 tests ids only, so a correction with no new id and no pending recipe never lands. The fn doc at line 1113 states the true behaviour; a maintainer reads the optimistic one at the point of change. Fix: scope the inline claim to "when this branch is entered" and point at the install-once limitation.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-21-2026-08-29T1336-81e8.md
  **Status:** RESOLVED (orch/21 pass 1) -- fixed as the entry records: the inline comment now says the stranding is avoided "on a device that reaches this branch", states that the branch is reached only when a new id or slug is pending, and points at the fn's own "Install-once by design" note for the case that never gets there. The behaviour is unchanged and the install-once limitation remains a recorded deferral; widening `missing_catalog` into a content-hash comparison was rejected as a behaviour change on every device, which is a later card's call.

- **The embedded starter manifest is parsed twice per install call** (`rust/src/api/starter.rs:32`) -- `shipped_starter_content()` already calls `all_starter_content()`, and the next line calls it again purely to count `pending_cook_review`, deserialising the 240-line JSON twice on every app launch. No wrong output. Fix: parse once, derive `pending` from that value, and partition its recipes for the shipped view.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-21-2026-08-29T1336-81e8.md
  **Status:** RESOLVED (orch/21 pass 1) -- fixed as the entry records, with the split extracted into `split_authored` (`rust/src/api/starter.rs:57`) rather than inlined, so the pin can exercise the shipped function instead of re-implementing the partition beside it. `install_starter_content` now makes one `all_starter_content()` call; `shipped_starter_content` is untouched as the public shipped view and moved to the test module's imports. Pinned by `the_one_parse_split_matches_the_two_view_functions`, whose `all(cook_review.is_some())` predicate is what catches an inverted partition -- equality alone would not, because today's roster is entirely `cook_review: null` and both halves compare trivially.

- **`policy.excluded` is parsed, `#[allow(dead_code)]`, and enforces nothing** (`rust/crates/food-domain/src/starter.rs:119`) -- nothing checks that `allowed_rights_bases` and `excluded` are disjoint, so a file listing `cc_by` in both parses cleanly; the contradiction is caught only by `RightsBasis` having no `CcBy` variant. A future variant for an excluded basis would override the manifest's own policy silently. Fix: either move the field to the un-schema'd prose half of `policy`, or add a disjointness check and drop the `#[allow]`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-21-2026-08-29T1336-81e8.md
  **Status:** RESOLVED (orch/21 pass 1) -- the second of the two listed options: `parse_starter_content` now rejects a basis appearing in both halves (`rust/crates/food-domain/src/starter.rs:211`) and the `#[allow(dead_code)]` is gone, replaced by a doc comment stating what the field enforces. Keeping the field and making it load-bearing was preferred over demoting it to prose because AC-1 reads `excluded` as the rights record. Scope recorded honestly in the code comment: `convert_recipe` already rejects any basis outside `allowed_rights_bases`, so `excluded` cannot tighten acceptance -- self-contradiction is the whole of what it can be wrong about. Pinned by `a_basis_both_allowed_and_excluded_is_a_content_error`, confirmed failing against the pre-fix tree before the check was added.

- **Two displaced doc comments in `test/app_test.dart`** (`test/app_test.dart:28`) -- inserting `okStarterReport` and `okRecipeNoPrep` orphaned the comments above them: the "Two lines, one with every structured field" comment now sits above `okStarterReport` but describes `okRecipe`, and the "`okRecipe` with two conflicts" comment (line 113) now sits above `okRecipeNoPrep` but describes `okRecipeWithConflicts`. Fix: move each new declaration below the comment that belongs to its neighbour.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-21-2026-08-29T1336-81e8.md
  **Status:** RESOLVED (orch/21 pass 1) -- fixed by neither listed option: the *comments* moved, not the declarations. Both orphaned blocks were cut from above the interlopers and re-anchored above the declarations they describe (`okRecipe` and `okRecipeWithConflicts`); `okStarterReport` and `okRecipeNoPrep` keep the comments they already had, so no declaration moved and no fixture gained or lost a comment. Moving the declarations was rejected because `okRecipeNoPrep` is defined in terms of `okRecipe`, so relocating it drags an initialisation-order constraint that relocating a comment does not -- and reordering top-level fixtures churns a file every widget test reads.

- **The roster conflict test depends on two unstated orderings** (`rust/crates/food-domain/src/starter.rs:645`) -- `every_recipe_matches_its_declared_conflicts` uses `Vec::dedup` on unsorted data and compares order-sensitively, so it is correct only because `assess` nests the restriction loop outside the line loop and because every `expected_conflicts` array is authored in `RestrictionKind::ALL` order. An `assess` refactor to line-outer iteration would fail all affected entries with a message blaming the content file. Fix: sort before `dedup` or compare as sets, and note that declaration order is not significant.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-21-2026-08-29T1336-81e8.md
  **Status:** RESOLVED (orch/21 pass 1) -- the sort arm, extracted into a `canonical` helper both sides of the assertion now call (`rust/crates/food-domain/src/starter.rs:395`). Sorting is by index in `RestrictionKind::ALL`, not `sort_unstable`, because the enum derives no `Ord` and deriving one on a domain type purely so a test can sort would be a production change for test convenience. Set comparison was rejected: `HashSet`'s `Debug` is unordered, which would have made the existing "assess found X, the file declares Y" failure message nondeterministic. This also brings the site in line with its two siblings (`every_slug_is_unique`, `every_catalog_id_is_unique`), which already sorted before `dedup`. `policy.expected_conflicts_order` in the content file now records that declaration order is not significant. Pinned by `canonical_collapses_interleaved_duplicates_and_ignores_authoring_order`.

### orch/31 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-31-2026-08-29T2229-6d2a.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2237-2591.md

#### LOW

- **`checked ⇔ checked_against` normalised in only one direction** (`rust/crates/kimatta-storage/src/lib.rs:2449`) -- the doc claims the pair "can never drift", but only `checked = false → checked_against = None` is forced; `checked = true` with no token reaches the INSERT and fails the table CHECK (`lib.rs:383`) as an untyped `Sqlite("CHECK constraint failed")`, unlike the typed blank-key case three lines above. Unreachable from the bridge today. Fix: reject the shape with a typed `ShoppingError` before the transaction, or narrow the doc comment.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2237-2591.md
  **Status:** RESOLVED 2026-08-29 -- `set_shopping_line_state` now rejects `checked = true` with no token as `Shopping(CheckWithoutQuantity)` before the transaction, and the doc comment names both directions; pinned by `a_check_without_its_quantity_is_a_typed_error_not_a_check_violation`.

- **No retention or purge for past windows' shopping line states** (`rust/crates/kimatta-storage/src/lib.rs:2615`) -- rows are keyed `(household, from, to, line_key)` and the only deletes are this window's rows and the household cascade, so every past cycle window persists forever. Volume is negligible (~1,600 rows/year); the edge is that re-deriving an old window resurrects last month's checks unmarked as stale. Fix: document the unbounded retention in the card, or purge windows older than N cycles.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2237-2591.md
  **Status:** RESOLVED 2026-08-29 -- documentation branch taken: the MVP-016 card's resolved-decisions line now states that line states are retained per window indefinitely and that re-deriving a past window resurrects that window's own checks by design. No purge was added; deleting user data needs a retention policy no PRD or decision record sets.

### orch/31 -- 2026-08-29

Source: rust/src/api/shopping.rs
Full review: /home/davidlinux/.claude/reviews/redteam-api-shopping-2026-08-29T2300-7c38.md

#### LOW

- **Dead `pub(crate)` widening with a false "Shared with `shopping.rs`" doc** (`rust/src/api/recipe.rs:210`) -- `quantity_to_domain` and `unit_to_domain` (also `recipe.rs:244`) were widened and documented as shared, but grep shows their only call sites are `recipe.rs:292`/`:293`; `shopping.rs` imports only the `_from_domain` pair and computes the token from the domain line. Rust does not warn on unused visibility. Fix: revert both to private and drop the two doc lines.
  Full review: /home/davidlinux/.claude/reviews/redteam-api-shopping-2026-08-29T2300-7c38.md
  **Status:** RESOLVED 2026-08-29 -- both reverted to private and the two doc lines dropped (MVP-016 Flutter round).
