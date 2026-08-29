# Known Issues — LOW

## orch/4 -- 2026-08-28

Full review: (lost) `wt/4/.orch/redteam-app-dart-2026-08-28T1845.md` was removed with the step-4 worktree before the orchestrator relayed it; the entries below are the only surviving record. Follow-up review: /home/davidlinux/.claude/reviews/redteam-mvp003-integration-verify-2026-08-28T1904-03fd.md

### LOW

- **`dispose()` can re-run `buildRouter` when the first build threw** (`lib/app/app.dart:21-23`, `:36-39`) -- `_router` is `late final`, initialized on first read inside `build`. Dart re-runs a `late final` initializer if a prior attempt threw, and `dispose()` reads `_router` unconditionally, so a throwing `buildRouter` produces a second construction and a second exception during unmount, obscuring the original. Not reachable today — `buildRouter` cannot throw as `lib/app/router.dart` currently stands — but it becomes reachable as the router grows: `go_router-18.0.0/lib/src/route.dart:1085-1090` asserts a `restorationScopeId` on the `StatefulShellRoute` whenever any branch sets one, which is exactly this router's shape. Fix: make the field nullable and dispose with `_router?.dispose()`, or gate disposal on a `bool` set once `build` completes. Deferred: debug-time diagnosability only, and the guard costs the `late final` idiom the class is built around; revisit if `buildRouter` gains a throwing path.
  **Status:** OPEN

---

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-redteam-pass2-2026-08-24T1320-5bbe.md

### LOW

- **The intra-file line-number-anchors entry's title said "Four" while the entry listed six** (`KNOWN_ISSUES.md`, the intra-file line-number-anchors entry) -- the bold title reads "Four new intra-file line-number anchors in a card guaranteed to be re-edited" while the parenthetical lists six (`:55`, `:86`, `:90`, `:301`, plus `:126` and `:150`), and the body still says "the card now self-references by line number in four places" and "All four resolve correctly today". Retitling was avoided deliberately to protect the grep-by-title dedup the next redteam pass relies on, but that grep keys on the distinctive phrase "intra-file line-number anchors in a card guaranteed to be re-edited", which survives dropping the numeral. Fix: drop the count from the title and update the two "four" occurrences in the body to match the six-item list, or state the count once in the body only. Deferred: cosmetic; the body's list is authoritative and correct.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The title drops the numeral and the body now reads "six places" / "All six resolve". The six-item set this entry quotes was itself wrong: `:126` is not an anchor site, and the real sixth is the line-wrapped "48 is discharged for the chosen name" at `:157`. Both corrected in the same edit; the citation is now the six quoted phrases, not line numbers.

- **The new entry is filed under a section header attributing it to a different review** (`KNOWN_ISSUES.md`, the section whose `Full review:` names `redteam-dec-004-name-shortlist-2026-08-24T0837-e592.md`) -- that section's header declares `Full review: .../redteam-dec-004-name-shortlist-2026-08-24T0837-e592.md` (pass 1), but its trailing entry comes from pass 2 and carries its own trailing `Full review: ...T1303-7845.md` to override the header. A reader or a later `/ki-maintain` run that trusts the section header follows the pass-1 link and does not find the entry. The placement itself is correct — the alternative section's `Source:` is `docs/task/README.md`, the wrong review chain, and the entry reads naturally beneath the pass-1 entry it supersedes. Fix: promote the pass-2 review link into the section header as a second `Full review:` line covering the chain, or accept the inline override and note it. Deferred: the inline `Full review:` override already resolves it in practice.
  **Status:** OPEN

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-fix-findings-2026-08-24T1442-970e.md

### LOW

- **`:315`'s "passed every check that could be performed read-only on both stores" still overreaches for the Play package-ID check** (`docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md:315-316`, cf. `:132`, `:167-168`) -- the rewritten conclusion says `Kimatta` "passed every check that could be performed read-only on both stores", but a Play check *was* performed read-only and did not pass: `:167-168` records the `app.kimatta` listing-URL 404 plus a null index search and concludes "per the rule above this is `NOT VERIFIED`, not `PASS`", and the table at `:132` reads "NOT VERIFIED (unprovable read-only)". The qualifier only holds if "could be performed" means "could be *conclusively* performed", a distinction carried solely by the table gloss two hundred lines earlier. The section's "Open items for resolution" (`:326-328`) also omits the package-ID item, though the D-027 residual-risk discharge at `:86-91` turns on it. Fix: narrow to "every store check that read-only methods can settle", or add the package-ID item to the open-items list. Deferred: the table and note 2 both state it correctly; prose precision only.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The conclusion now reads "passed every store check that read-only methods can settle" and states the `app.kimatta` package-ID check separately as `NOT VERIFIED` per note 2; that item is also added to the "Open items for resolution" list, so the section is self-contained.

- **`/redteam-code` regenerates a dead `Source:` header on every run** (`~/.claude/commands/redteam-code.md:121`; symptom was the section headers of this file and `KNOWN_ISSUES.md`) -- the command spec requires a new known-issues section's `Source:` header to be its `**Reviewed:**` target, which for a `/redteam-code` run is an impl-handoff doc the command disposes at the end of that same run. Six such dead `Source:` lines accumulated across both trackers and were deleted 2026-08-24; the next run adds a seventh. **Convention adopted 2026-08-24:** drop `Source:` when its target is a disposed handoff; keep it when it names a durable in-repo file, as the `docs/task/README.md` section of `KNOWN_ISSUES.md` does. The adjacent `Full review:` line always resolves and is sufficient alone. Fix: encode that rule in the `/redteam-code` spec. Deferred: the skill file is outside this repo and was ruled out of scope for the fix pass that found this; tracked here so the rule survives the pass.
  **Status:** OPEN

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-fix-findings-2026-08-24T1537-7fba.md

### LOW

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

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-2026-08-24t2107-8195-2026-08-24T2111-0bd4.md

### LOW

- **`_probe` only catches `KimattaError`; any other failure is an unhandled async error** (`lib/main.dart`, `_HealthScreenState._probe`) -- a non-`KimattaError` bridge failure (panic, codec mismatch) escapes the `on KimattaError` clause and the screen keeps showing `probe: not probed`, so a bridge fault reads as "button does nothing". Fix: add a bare `catch (e)` branch rendering `'unexpected: $e'`. Deferred: spike-only screen, removed by MVP-003.
  **Status:** RESOLVED 2026-08-28 (MVP-003 removed HealthScreen and the spike main())

- **Widget test skips the `hasError` branch of the health `FutureBuilder`** (`test/widget_test.dart`; `lib/main.dart` `HealthScreen.build`) -- the test feeds a completed `report` only; the `'health error: ...'` rendering and the `KimattaError_Storage` variant are touched by no test. Fix: one extra `testWidgets` with `report: Future.error(const KimattaError.storage(message: 'x'))`. Deferred: card scope only requires a fake-fed widget test; screen goes with MVP-003.
  **Status:** RESOLVED 2026-08-28 (MVP-003 removed HealthScreen and the spike main())

- **Spike-only `ensureSemantics()` left in production `main()`** (`lib/main.dart`, `main()`) -- `SemanticsBinding.instance.ensureSemantics()` was added so `uiautomator dump` could locate the probe button for the AC-5 screenshot; tagged `ponytail:` with a removal note. Fix: delete with `HealthScreen`. Deferred: harmless, removal is bundled with the screen.
  **Status:** RESOLVED 2026-08-28 (MVP-003 removed HealthScreen and the spike main())

- **PRE-002 command contract duplicated in README with a different step order** (`README.md` "Architecture direction" block; `docs/ROADMAP.md` "PRE-002 command contract") -- README puts `export PATH` before the rustup install and omits the `dart format`/`flutter analyze` gate lines; both sequences work, but future contract edits (MVP-002's `cargo` gate) must land twice with no check. Fix: keep the short clean-checkout block in README and link to the ROADMAP contract for the gate. Deferred: doc hygiene; fold into the next card that edits the contract.
  **Status:** OPEN

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-stale-lib-2026-08-24T2152-ba7d.md

### LOW

- **"a stale `.so` gives a FALSE PASS, not a load failure" is stated unconditionally; FRB hash-checks on init** (the PRE-002 block's `a stale .so gives a FALSE` comment in `docs/ROADMAP.md`; `docs/task/decision/DEC-005_BRIDGE_BACKEND_AND_CORE_COMMITMENT.md:61`; and, added 2026-08-24 when the DEC-002 contract was fixed, the trailing `# stale .so = false pass; never run alone` comments on the `(cd rust && cargo build --release) && flutter test` lines in `docs/ROADMAP.md`'s DEC-002 block and in `docs/task/decision/DEC-002_ENGINEERING_EXPERIENCE_FOUNDATION_DECISION.md`) -- all four state the false-pass mode absolutely, but FRB validates the codegen content hash at init: `_sanityCheckContentHash` (`flutter_rust_bridge-2.13.0/lib/src/main_components/entrypoint.dart:136`) throws `StateError` when the Dart-side `rustContentHash` (`lib/src/rust/frb_generated.dart:72`) differs from `FLUTTER_RUST_BRIDGE_CODEGEN_CONTENT_HASH` (`rust/src/frb_generated.rs:42`). An API-surface change followed by `flutter_rust_bridge_codegen generate` therefore fails loudly against a stale `.so`; only a body-only edit leaves the hash unchanged and produces the silent false pass. Fix: narrow the claim in all four places to name the body-only condition. Deferred: the prescribed command is correct and safe either way; only the stated mechanism is over-broad.
  **Status:** OPEN

- **Prose is the only enforcement of the `cargo build --release` + `flutter test` chain, and the residual risk is tracked nowhere** (the `(cd rust && cargo build --release) && flutter test` line, in `docs/ROADMAP.md`'s PRE-002 and DEC-002 blocks, in `README.md`, and in `docs/task/decision/DEC-002_ENGINEERING_EXPERIENCE_FOUNDATION_DECISION.md`) -- the pass-1 MEDIUM was a silent-false-pass failure mode now closed by a sentence telling a human not to trigger it. No enforcing artefact exists: `.github/` is absent, `tools/` holds only `windows/setup-adb-bridge.ps1`, and `.githooks/pre-commit` handles `*:Zone.Identifier` files only. The sturdier routes named in the fix plan (`tool/test.sh`, a Makefile target, an `externalLibrary:` override in `test/bridge_native_test.dart`) were deferred but never recorded, so with the MEDIUM marked ADDRESSED the residual risk has no carrier into MVP-002 -- exactly when Rust edit volume rises. Fix: a `tool/test.sh` wrapping `(cd rust && cargo build --release) && flutter test`, folded into MVP-002's Rust-gate AC where the contract is being edited anyway; that also moots the contract-duplication problem. Deferred: deliberate spike-scope trade; this entry is the tracking carrier.
  **Status:** OPEN

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp002-2026-08-24T2213-6556.md

### LOW

- **`reopen_is_idempotent` writes a pid-named file to the shared temp dir and only cleans up on success** (`rust/crates/kimatta-storage/src/lib.rs:131-144`) -- the path is `std::env::temp_dir().join(format!("kimatta-{}.db", std::process::id()))` and `std::fs::remove_file` is the last statement, so any panic or failed assertion earlier in the test leaves the database behind. A later run in a process reusing that pid (Linux pids wrap at 32768 by default) reopens a database already containing household `h`, and `insert_household(...).unwrap()` panics on the primary-key conflict -- a failure unrelated to the code under test. The `tempfile` dev-dependency was deliberately skipped as speculative, but the hand-rolled substitute lacks the drop-guard that makes it safe. Fix: make the filename unique per run (nanosecond timestamp or atomic counter), or hold the file in a small guard struct whose `Drop` removes it. Deferred: self-inflicted flake only, needs a stale file plus a pid collision.
  **Status:** RESOLVED 2026-08-25 -- fixed rather than deferred. The test now takes a `tempfile::tempdir()`; `TempDir::drop` removes the directory even on panic, so neither the collision nor the litter is possible. `tempfile` added as a dev-dependency of `kimatta-storage`.

- **Dart test asserts two unrelated behaviors under a name describing only one** (`test/bridge_native_test.dart:15-26`) -- `test('typed Rust error is matchable in Dart')` now also creates a temp directory and asserts `report.schemaVersion == 1`, the assertion carrying MVP-002 AC-4's "bridge test now asserting schema version 1 on a real file" evidence. Its sibling at `:11` (`core_version crosses the bridge`) follows one-behavior-per-test, so this departs from the file's own convention rather than following a house style; a schema-version regression reports under a name about typed errors, and the two assertions have independent failure causes. Fix: split into a second `test('health_check migrates a real database to schema v1')`, alongside the `KimattaError_Storage` test the pass-1 MEDIUM asks for. Deferred: cosmetic; fold in with the `Storage` test.
  **Status:** RESOLVED 2026-08-25 -- fixed rather than deferred. Split into `health_check migrates a real database to schema v1`, alongside the new `storage failure surfaces as KimattaError_Storage`; `flutter test` is now 5/5.

- **DEC-005's single-`rusqlite`-version guard has no carrier in the Rust gate MVP-002 added** (the `cargo fmt --all --check && cargo clippy` gate line in `docs/ROADMAP.md`'s DEC-002 block; the `rusqlite` and `rusqlite_migration` lines in `rust/crates/kimatta-storage/Cargo.toml`) -- DEC-005 decision 4 makes `rusqlite_migration` conditional ("dropped only if `cargo tree -i rusqlite` shows more than one version"), but the dependencies are caret ranges (`rusqlite = { version = "0.40", ... }`, `rusqlite_migration = "2"`), so a future `cargo update` can pull a `rusqlite_migration` minor that bumps its `rusqlite` requirement and yield two `libsqlite3-sys` copies under the `bundled` feature. AC-1 verified one version once, by hand; the new gate line runs `fmt`, `clippy`, and `test` only, so the recorded rationale for keeping the crate would silently stop being validated. Fix: append `cargo tree --workspace -i rusqlite` to the Rust gate line, or pin both dependencies to exact versions so `cargo update` becomes a deliberate act. Deferred: failure mode is a loud link-time duplicate-symbol error, and `Cargo.lock` is committed.
  **Status:** OPEN

## master -- 2026-08-25

Raised during the fix pass for /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp002-2026-08-24T2213-6556.md, not by the review itself.

### LOW

- **`insert_household` accepts a member belonging to a different existing household** (`rust/crates/kimatta-storage/src/lib.rs`, `insert_household`) -- moved to `KNOWN_ISSUES.md` on 2026-08-25 and re-rated MEDIUM by the arch review of `rust/src`; that entry carries the full text plus the two additions the review made. Tracked there, not here.
  **Status:** MOVED 2026-08-25 -- see `KNOWN_ISSUES.md`, same title.

- **The production database path has no automated test carrier** (`lib/features/settings/health_provider.dart`, `healthReportProvider`) -- `getApplicationSupportDirectory()` is called once inside the provider, which every test overrides, so nothing catches a regression to the path source. A `PathProviderPlatform` mock would assert the mock rather than the device, so no useful unit test exists. The pre-fix value (`Directory.systemTemp` → `/data/local/tmp` on Android) was unwritable by an app uid, so the regression is silent-at-build-time and fatal at runtime. MVP-002 AC-3's on-device evidence is the only verification; it passed once on 2026-08-25 (`emulator-5554`), but that is a point-in-time check, not a regression guard. Fix: covered indirectly once MVP-004's persistence evidence exercises a real write; MVP-003's card carries a load-bearing constraint to keep the path app-private when it replaces `main()`. Deferred: `HealthScreen` and the spike `main()` were deleted at MVP-003; the path source moved to `healthReportProvider`, which every test overrides, so the gap survives the move. Re-verified on-device 2026-08-28 (MVP-003 Settings screenshot + run-as listing); still no regression guard — tests override the provider.
  **Status:** RESOLVED 2026-08-28 -- MVP-004 AC-3 PASS: a real household write landed at `/data/user/0/dev.mealmate.temp/files/kimatta.db` on a post-`pm clear` install and survived a force-stop relaunch (ids identical). Still a point-in-time on-device check, not a unit-test guard.

- **The DEC-002 decision card's own command block has no Rust gate and is a fourth copy of the contract** (`docs/task/decision/DEC-002_ENGINEERING_EXPERIENCE_FOUNDATION_DECISION.md`, the `bash` block under "**Chosen commands**") -- MVP-002 added the Rust gate to three places (both `docs/ROADMAP.md` blocks and `README.md`) but not to this one, which still lists `dart format` / `flutter analyze` / `cargo build --release && flutter test` / `flutter build apk --debug`. It is not covered by the existing README-duplication entry above, which is titled for the *PRE-002* contract; this block is a copy of the *DEC-002* contract. The block's own preamble says "the operative contract is the DEC-002 and PRE-002 command-contract blocks in `docs/ROADMAP.md`", so nothing is currently wrong -- but a reader who trusts the card runs a gate-less sequence. Fix: replace the block with a link to the ROADMAP contract, or add the gate line; either way fold it into the consolidation the README entry above already tracks. Deferred: self-declared non-operative, and consolidating all four copies is a card of its own.
  **Status:** OPEN

## master -- 2026-08-25

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-dec002-contract-2026-08-24T2239-7082.md
Full review: /home/davidlinux/.claude/reviews/redteam-dec002-contract-supersession-2026-08-25T1201-8dfa.md

### LOW

- **The new §3 note duplicates, and mildly contradicts, the card's existing v3 pointer banner** (`docs/task/decision/DEC-002_ENGINEERING_EXPERIENCE_FOUNDATION_DECISION.md:81` against `:17`) -- the card already carries a supersession banner at `:17` ("decisions 1 and 3's Dart commands **stand** for presentation code; the `lib/domain` coverage gate, the domain-independence script, the `--coverage` flag, and the `glados` targets named below are superseded"). The note added at `:81` restates the coverage-gate supersession that `:17` already covers, and sets a different disposition against it for the commands: `:17` says §3's Dart commands *stand*, `:81` says "the operative contract is ... in `docs/ROADMAP.md`" -- two answers to "is this block live?". The same tension shows in the block itself: `:17` scopes it to "Dart commands ... for presentation code", yet `:86` now carries a Rust `cargo build --release`. Fix: fold the operative-contract pointer into the `:17` banner, where a reader of a Done card looks first, and cut `:81` back to what `:17` does not already say. Deferred: prose hygiene on a Done card; no operational risk.
  **Status:** OPEN

## master -- 2026-08-25

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp002-emulator-2026-08-25T1249-0fee.md

### LOW

- **`tools/emulator.sh up` reports success for any booted device, never checking it is `$AVD`** (`tools/emulator.sh:61-64`, used at `:97`, `:104`, `:122`, `:143`) -- `ready()` is satisfied by any line matching `[[:space:]]device$` in `adb devices`; `$AVD` (`EMU_AVD`, default `pre001_avd`) is used only to launch the emulator and to probe the lock file. If a different AVD is already up, the fast path at `:97` emits the socket and exits 0 while the requested AVD is never started; if a different AVD is booting, `device_present` at `:122` takes the "waiting rather than launching another" branch for the full `EMU_BOOT_TIMEOUT`. `qemu_count`'s `emulator|qemu` match (`:70`) cannot separate them either. Fix: resolve the serial to its AVD (`adb -s <serial> emu avd name`, or `getprop ro.boot.qemu.avd_name`) and compare, or state the single-emulator assumption in the header beside the WSL2-NAT one. Deferred: one AVD on this host; the header note is the cheap half.
  **Status:** OPEN

- **`EMU_EXE` is the only host path with no env override, and a missing SDK is reported as an acceleration failure** (`tools/emulator.sh:23`, `:88-91`, `:136`) -- `AVD`, `BOOT_TIMEOUT` and the host `adb.exe` are all overridable (`EMU_AVD`, `EMU_BOOT_TIMEOUT`, `EMU_ADB_EXE`), but `EMU_EXE` is a hardcoded `C:\Android\sdk\emulator\emulator.exe`. If the SDK moves, PowerShell's `& '<missing path>'` throws CommandNotFoundException, `accel_ok` returns non-zero, and `:136` reports "emulator.exe -accel-check failed; no hardware acceleration" -- a wrong diagnosis of exactly the class this script exists to eliminate. Fix: `EMU_EXE=${EMU_EMULATOR_EXE:-'C:\Android\sdk\emulator\emulator.exe'}` plus a `Test-Path` existence probe (as `avd_locked` already does), or widen the failure message to name both causes. Deferred: correct for this host today; only misleads if the SDK moves.
  **Status:** OPEN

- **Header comment claims `socket()` is the only stdout-writing helper; two others also are** (`tools/emulator.sh:42`, vs `:71` and `:85`) -- `socket()` carries `# value function: the ONLY helper that writes stdout`, but `qemu_count` (`:71`, `printf '%s' "$n"`) and `host_adb` (`:85`, `printf '%s' "$pick"`) are value functions too. The comment states a load-bearing invariant (stdout purity, enforced by `exec 9>&1 1>&2` at `:30`), so a future editor either mistrusts it or "fixes" a helper that is correct as written. Fix: "value function -- writes its result to stdout, as do `qemu_count` and `host_adb`; every other helper writes stderr." Deferred: cosmetic.
  **Status:** OPEN

## master -- 2026-08-25

Full review: /home/davidlinux/.claude/reviews/arch-dir-src-2026-08-25T0900-9639.md

### LOW

- **`core_version()` reports the bridge package's version, not the core's** (`rust/src/api/health.rs`, `core_version`) -- `env!("CARGO_PKG_VERSION")` expands in whichever crate compiles it, which is `kimatta_bridge`, so the name promises `household-core` and delivers the bridge package version. Both read `0.1.0` today, so the bridge test's `expect(coreVersion(), '0.1.0')` passes under either reading and nothing detects the divergence when the crates version apart. Fix: rename to `bridge_version()`, true under either reading; pointing it at a `household_core::version()` was rejected -- the bridge has no `household-core` dependency, so that needs a new dependency edge to settle a cosmetic finding. Reach of the rename: `lib/main.dart` (call site and the `core_version:` display string) and `test/widget_test.dart` (the `find.text('core_version: 9.9.9')` assertion) both die when MVP-003 replaces `HealthScreen`, but `test/bridge_native_test.dart`'s `coreVersion()` assertion and the Rust definition survive that card; `PRE-002_RUST_FRB_BRIDGE_SPIKE.md` and `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` both name the function in API-specification lines that would go stale, while `docs/ROADMAP.md`'s PRE-002 AC-5 row names it inside quoted on-device screenshot evidence -- historical, do not rewrite. Deferred: gated on MVP-002's AC-3 artifact; cosmetic until the two crates version apart, which is the real trigger.
  **Status:** OPEN

- **Three naming conventions across a three-crate workspace** (`rust/Cargo.toml`, `rust/crates/kimatta-storage/Cargo.toml`, `rust/crates/household-core/Cargo.toml`) -- `kimatta_bridge` (underscore, prefixed, doubling as the workspace root package), `kimatta-storage` (hyphen, prefixed), `household-core` (hyphen, unprefixed). MVP-002 names `kimatta-application` as the next crate, which is consistent with one of the two existing prefix rules and not the other, so the fourth crate has no rule to follow. Largely pre-empted: `PRE-002_RUST_FRB_BRIDGE_SPIKE.md` records the `kimatta-*` names as "provisional pending DEC-004; renaming crates is mechanical", and `docs/task/SEQUENCE.txt`'s DEC-004 owner step already says to "add Rust crate names to the rename scope" -- though DEC-004's own rename-scope completion criterion currently lists `applicationId`, manifest label, pubspec name and docs, not crate names. Fix: settle the convention as part of DEC-004's rename scope rather than separately; this entry exists so the convention question (prefix all or none) is not lost behind the name question. Deferred: no operational risk; owner-paced and blocked on DEC-004.
  **Status:** OPEN

## master -- 2026-08-25

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-arch-dir-src-deferrals-2026-08-25T1325-7abd.md
Full review: /home/davidlinux/.claude/reviews/redteam-arch-dir-src-deferrals-2026-08-25T1337-5462.md

### LOW

- **The `MOVED` stub is permanent, and the recorded risk against it is inverted** (`KNOWN_ISSUES-low.md:99-100`) -- the fix pass that moved the `insert_household` mis-parent entry to `KNOWN_ISSUES.md` recorded the risk as "the cross-file move could dangle if `/ki-maintain` runs before MVP-004", on the reasoning that `MOVED` is "a `**Status:**` value the maintenance tooling has not seen before and may not model". `~/.claude/commands/ki-maintain.md` §3 does model it, by falling through: "`**Status:**` field with value starting with `OPEN`, `DEFERRED`, or any other value → **open**". So `MOVED` classifies as open and is never archived; the live entry at `KNOWN_ISSUES.md:117` is `**Status:** OPEN` and is never archived either, and nothing can dangle. The real cost is the opposite one: this file now carries a permanently-open LOW entry that is not a finding, duplicating a bold title that also lives in the sibling ledger, so every future open-LOW sweep counts it. Its stated purpose -- "the grep handle other passes use" -- is already served by the live entry, because `/redteam-code` step 7b greps *both* files for a bold title. Fix: give the stub a status the aging pass resolves (`RESOLVED 2026-08-25 -- moved to KNOWN_ISSUES.md`), or delete it; dedup is unaffected either way. Deferred: cosmetic ledger hygiene; the stub is harmless, just never removable and redundant with the live entry.
  **Status:** OPEN

## master -- 2026-08-25

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-mvp003-fixfindings-2026-08-25T2013-264a.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-fixfindings-2026-08-25T2234-cce8.md

### LOW

- **Step 7's `test/app_test.dart` harness carries `retry:` with no Riverpod-2 fallback** (`~/.claude/plans/goofy-juggling-stroustrup.md:459`, vs the fallback instruction at `:57`) -- the MVP-003 plan's Risks row reads "Confirm at step 3: `ProviderScope(retry:)` exists (Riverpod >= 3.0) -- if the resolved major is 2.x, drop the `retry` argument". Singular, and checked at step 3, when the only occurrence is the one step 6 writes into `lib/main.dart` (`:442`). `test/app_test.dart` does not exist until step 7, and its harness at `:459` repeats `retry: (_, _) => null`, so on a 2.x resolution the executor drops it from `main.dart` and copies it into the test verbatim. Fix: have `:57` name both call sites (`lib/main.dart` and the step-7 harness), or note it inline at `:459`. Deferred: fails loudly at compile time, costs one cycle, and only on a 2.x resolution -- which `flutter pub add` at step 3 is expected to avoid.
  **Status:** RESOLVED 2026-08-25 (fixed in the MVP-003 plan, rev 5; both widened by a sibling sweep — see redteam-impl-handoff-master-mvp003-fixfindings-2026-08-25T2234-cce8.md)

- **`NavigationBar` clamps label text scale to 1.3, so the plan's overflow contingency cannot fire** (`~/.claude/plans/goofy-juggling-stroustrup.md:63` Risks row, `:481` test 7, `:124` the AC-2 evidence wording) -- the row reads "`NavigationBar` labels at text scale 2.0 may overflow ... If it overflows, `labelBehavior: onlyShowSelected` -- recorded as a theme decision." The framework forecloses it: labels are wrapped in `MediaQuery.withClampedTextScaling(maxScaleFactor: _kMaxLabelTextScaleFactor)` = 1.3 (`packages/flutter/lib/src/material/navigation_bar.dart:31`, `:506-512`) and the bar's height "does not adjust with ... [MediaQueryData.textScaler]" (`:210-213`). At `textScaleFactorTestValue = 2.0` the bar renders at 1.3 and cannot overflow, so the `labelBehavior` fallback is dead prose and the AC-2 evidence line at `:124` ("text scale 2.0") overstates what test 7 covers for the shell chrome; the placeholder bodies and `AppBar` titles it also exercises do reach a genuine 2.0, and those are the parts that can overflow. Fix: replace the Risks row with the clamp citation (a resolved non-risk worth keeping so a later card does not re-derive it) and word AC-2's evidence as "destination screens at text scale 2.0; `NavigationBar` labels clamp at 1.3 by framework design". Deferred: no behavioural consequence -- dead contingency prose plus a slightly overstated evidence line.
  **Status:** RESOLVED 2026-08-25 (fixed in the MVP-003 plan, rev 5; both widened by a sibling sweep — see redteam-impl-handoff-master-mvp003-fixfindings-2026-08-25T2234-cce8.md)

## master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-mvp003-redteam-pass1-2026-08-26T0847-3c5a.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-redteam-pass1-2026-08-26T0928-3cb9.md

### LOW

- **The `font_scale` set/reset window has no trap and no acknowledgement anywhere in the plan** (`~/.claude/plans/goofy-juggling-stroustrup.md`, the `settings put system font_scale 2.0` / `1.0` pair in step 9, and the On-device bullet of the Verification summary) -- MVP-003 plan step 9 sets `settings put system font_scale 2.0`, captures `settings_scale2.png`, and resets to `1.0`, with no trap between them. Any interruption in that window leaves the emulator at font scale 2.0 for every later session -- a host-state side effect that outlives this card and is inherited by MVP-004 onward. The MVP-003 fix pass recorded this as deferred out-of-scope and stated it was "recorded in the plan's Verification section as 'noted, not fixed'", but it is not: the Verification summary's On-device bullet mentions only "font_scale 2.0 screenshot". (The pass-1 wording also cited a `grep -n 'noted'` hit whose referent the rev-7 pass rewrote; that clause is struck rather than re-anchored.) The acknowledgement lived solely in the handoff doc, which is disposed after review -- the same missing-carrier failure the `KNOWN_ISSUES.md` MEDIUM titled "`docs/ROADMAP.md` contradicts itself on MVP-002 AC-3, and the deferred prose sweep has no carrier" exists to prevent, which is why this entry is the carrier. Fix: wrap the pair (`trap '... settings put system font_scale 1.0' EXIT` before the `font_scale 2.0` line, cleared after the reset), or add one sentence to the plan's Verification summary recording it as a known accepted window; if step 9 has already run, verify the emulator is back at 1.0 before the next evidence card. Deferred: evidence-integrity risk is nil (the four commands are adjacent and all `exit 1` gates in the block sit after the reset), but the host-state risk is durable and needed a home outside the disposed handoff.
  **Status:** RESOLVED 2026-08-28 (MVP-003 step 9 wraps the 2.0/1.0 pair in a `trap … EXIT` and asserts `settings get system font_scale` reads 1.0 before the fragment ends)

## master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-mvp003-fixfindings-rev6-2026-08-26T1010-5279.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-fixfindings-rev6-2026-08-26T1343-37fc.md

### LOW

- **The new `font_scale` deferral has no closure trigger in the MVP-003 plan's step-10 KI-low reconciliation** (`~/.claude/plans/goofy-juggling-stroustrup.md:559`, against the entry this file carries at `:175`) -- step 10 names four `KNOWN_ISSUES-low.md` entries by title and states exactly what happens to each at card close: three flip to `RESOLVED`, one stays `OPEN` with a re-anchor and an appended re-verification note. The `font_scale` entry written by redteam pass 1 is not among them (`sed -n '559p' goofy-juggling-stroustrup.md | grep -c font_scale` -> 0), so once MVP-003 executes it stays `OPEN` indefinitely with a Fix ("wrap the pair ... before `:517`") that is only actionable *before* execution. The half that stays actionable is its own trailing clause -- verify the emulator is back at font scale 1.0 -- and step 10 is where that check belongs. Correction to the pass-1 premise: `~/.claude/plans/` retains executed plans back to 2026-08-11, so the entry's `:517-521` / `:570-574` anchors do not dangle after execution; the line-neutrality constraint bought more than was claimed. Fix: add one clause to `:559` in the same shape as the "production database path" entry it already handles -- the `font_scale` entry stays `OPEN` and gains `Emulator confirmed back at font_scale 1.0 after the MVP-003 step-9 run (adb shell settings get system font_scale)`, or flips to `RESOLVED` if step 9 ends up carrying the `trap`. Deferred: ledger hygiene only, no behavioural consequence, and the entry's own trailing clause keeps it actionable in the meantime.
  **Status:** RESOLVED 2026-08-26 (fixed in the MVP-003 plan / pass-1 review doc)

- **The pass-1 review doc's Disposition pointer was computed before the Disposition itself shifted the lines it cites** (`/home/davidlinux/.claude/reviews/redteam-mvp003-redteam-pass1-2026-08-26T0928-3cb9.md:28`) -- the Disposition paragraph warns "Note for anyone re-running the fence suite at `:42-45`". In the current file `:42-45` is the "Cut markers" bullet (`android:configChanges` / `fontScale`); the fence suite is at `:56-59`. The cause is mechanical and self-inflicted: the Disposition occupies `:18-30` plus a blank line = 14 inserted lines, and 56 - 14 = 42, so the anchor was measured against the pre-insertion file. The same fix pass adopted line-count neutrality as a hard design constraint for the plan it was editing and did not apply the same reasoning to the document it was inserting into. The warning is the durable artifact for the next reviewer of this chain and it points 14 lines off. Fix: change `:42-45` to `:56-59`, or replace the number with the bullet's bold title (`**Verification suite, scoped to the fence**`), which cannot shift. Deferred: cosmetic misdirection in a review doc that `/tidy` eventually archives; costs a future reader one grep.
  **Status:** RESOLVED 2026-08-26 (fixed in the MVP-003 plan / pass-1 review doc)

- **MVP-003 step 9's `timeout: 600000` headroom was computed for `up` alone, not for the four commands sharing its tool call** (`~/.claude/plans/goofy-juggling-stroustrup.md:501-509`) -- the timeout was sized against `cmd_up`'s worst case: `LOCK_WAIT` 300s (`tools/emulator.sh:26`, `flock -w` at `:102`) + `BOOT_TIMEOUT` 180s (`:22`, polled at `:141-146`) + ~10s port polling at `:111` ~= 490s, against the 600000ms Bash-tool ceiling. But fragment 1 does not end at the gate: `:504`-`:509` then run `adb install -r` of a Flutter debug APK (`docs/ROADMAP.md:150` records the PRE-001 artifact at 150MB) over the WSL->Windows TCP bridge, `am start`, a 3s device-side sleep, `screencap` and `wm size` -- plausibly 30-90s more. 490 + 90 ~= 580s leaves ~20s of margin against a ceiling that cannot be raised, and blowing it kills the fragment *after* a successful bring-up, losing `plan.png` and the `wm size` output CUT 1 needs. Fix: split the gate into its own tool call so `up` gets the full 600s and the install/launch/capture sequence gets a fresh one; reword the existing `:497` "three tool calls" comment rather than extending it, to preserve line-count neutrality. Deferred: the worst case still fits, the failure mode is an idempotent rerun (`status` short-circuits, `install -r` re-installs) rather than wrong evidence, and the fix restructures the fragment split.
  **Status:** RESOLVED 2026-08-26 (fixed in the MVP-003 plan / pass-1 review doc)

## master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-v3-card-rederivation-2026-08-26T1337-0211.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-v3-card-rederivation-2026-08-26T1626-ad3a.md

### LOW

- **MVP-025 hands a beam-width verdict to a card that will already be Done, with no re-open path** (`docs/task/mvp/MVP-025_PLANNER_FIXTURES_AND_INVARIANT_TESTS.md:66`, restated at `:60`) -- MVP-025's decision gate says "If the benchmark shows the shipped beam width or candidate limit is wrong, the change belongs to `MVP-023`", and MVP-023's own gate (`docs/task/mvp/MVP-023_FOOD_CONTROLLER_PLANNER_CORE.md:74`) says "`MVP-025`'s benchmark may revise them". But `docs/task/SEQUENCE.txt` steps 32-35 run MVP-023 to completion before MVP-025 starts, and LOCAL-CORE-LOOP-READY (`docs/ROADMAP.md:23`) requires both Done. Neither card names the mechanism for re-opening a Done card. D-031's Status contract covers changes to "sources, decisions, product scope, architecture, or declared dependencies" -- a benchmark verdict overturning a recorded choice arguably qualifies, but no card says so, so the handover is an implied transition rather than a defined one. Fix: one sentence in MVP-025's gate stating that an adverse verdict returns MVP-023 to Draft under D-031 with the benchmark row as the trigger. Deferred: SEQUENCE ordering makes this a process nicety with no live sequencing hazard.
  **Status:** RESOLVED 2026-08-26 -- MVP-025's gate now states the D-031 Draft return with the AC-4 benchmark row as trigger (D-036). The MVP-024 AC-8 escalation was deliberately placed in MVP-024's own Stop/failure conditions rather than here, because MVP-025 is Done before AC-8 can fire.

- **MVP-023 requires starter meals as a candidate source but does not depend on MVP-011** (`docs/task/mvp/MVP-023_FOOD_CONTROLLER_PLANNER_CORE.md:10`) -- MVP-023's scope names all seven PRD v3 §9.3 candidate sources including "starter meals", and the 2026-08-26 re-derivation added the reciprocal coupling to MVP-011 (`docs/task/mvp/MVP-011_STARTER_CONTENT_READINESS.md:35`: "every shipped recipe must satisfy the same structured shape MVP-023 hard-filters and scores"). MVP-023's `Depends on` is `MVP-002, MVP-009, MVP-012, MVP-014`; MVP-011 is not reachable transitively (MVP-009 -> 006/007/008; MVP-012 -> 004/005/007; MVP-014 -> 003/004/007). A bidirectional content coupling was created without either card's `Depends on` or register row moving -- the same D-029 obligation the pass honored for MVP-004's added MVP-003 dependency. Fix: add MVP-011 to MVP-023's `Depends on` (card cell + `docs/ROADMAP.md` register row together), or state in MVP-023's scope that the starter-meal candidate source is implemented against the entity shape and does not require shipped starter content to exist. Deferred: `docs/task/SEQUENCE.txt` already orders MVP-011 (steps 20-21) before MVP-023 (steps 32-33), so this is declaration hygiene with no sequencing hazard.
  **Status:** RESOLVED 2026-08-26 -- took the decoupling option (D-036): MVP-023's scope now states the starter-meal candidate source is implemented against the shared entity shape and does not require MVP-011's shipped content, so no `Depends on` edit was needed.

- **Stray blank lines split two Load-bearing constraint lists into separate markdown lists** (`docs/task/mvp/MVP-011_STARTER_CONTENT_READINESS.md:37` and `docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md:36`) -- in both cards the constraints prepended by the 2026-08-26 re-derivation are separated from the pre-existing bullets by a blank line, which renders as two `<ul>` blocks with extra vertical space rather than one list. Every other card the same pass touched appended constraints without the gap, so the two are inconsistent with the pass's own output. Fix: delete the blank line in each. Deferred: cosmetic rendering only.
  **Status:** RESOLVED 2026-08-26 -- both blank lines deleted (D-036).

## master -- 2026-08-26

Full review: /home/davidlinux/.claude/reviews/redteam-d036-fix-pass-2026-08-26T1820-55a7.md

### LOW

- **D-036's "extends D-034's 'Three AC-level edits' enumeration to five" counts a constraint edit as an AC-level edit** (`docs/ROADMAP.md:157`, against `:155`) -- D-034 enumerates "Three AC-level edits ... `MVP-018`'s new AC-6, `MVP-019`'s PRD §19 metric list, `MVP-022`'s AC-1". The second member is not an AC-level edit: the §19 metric list landed in `MVP-019`'s Load-bearing constraints (`docs/task/mvp/MVP-019_OBSERVABILITY_FOUNDATION.md:40`), which is precisely the constraint-without-a-criterion defect D-036 was convened to fix and did fix by adding AC-5. D-036 then extends 3 + 2 = 5 while inheriting the mislabel, so "five" counts one non-AC; the accurate count of AC-level edits inside D-034's fenced second group (MVP-018--MVP-022, DEC-003) is four -- `MVP-018` AC-6, `MVP-022` AC-1, `MVP-019` AC-5, `MVP-022` AC-6. The row is also unscoped: D-034's enumeration applies only to that second group, but D-036's sentence does not restate the scope, so a reader applying it repo-wide will miss `MVP-006` AC-4 and `MVP-024` AC-8 / extended AC-4 (which need no enumeration, being in the fully-re-derived first group, and which the row does mention separately in prose). Fix: reword to "extends D-034's second-group AC-level enumeration to four ACs -- `MVP-018` AC-6, `MVP-022` AC-1, and now `MVP-019` AC-5 and `MVP-022` AC-6 -- and corrects D-034's count, which listed `MVP-019`'s §19 metric list as an AC-level edit when it was a Load-bearing constraint." Deferred: prose accuracy in a decision row; the substantive reasoning behind both additions is correct (PRD v3 §19 decides the metric set, §22 decides the proof vehicle, neither decides a numeric bar, so the bar correctly became a decision gate).
  **Status:** OPEN

- **"median ratio" and "median of the recorded scenarios" are different statistics and can flip the go/no-go verdict** (`docs/task/mvp/MVP-024_COVER_MY_WEEK_EXPERIENCE.md:77` and `:90` vs `docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md:53`) -- AC-8 says "The record states the median ratio of active seconds" (median of the per-scenario ratios); MVP-022's working reference says "Cover My Week active seconds ≤50% of the manual arm's, on the median of the recorded scenarios", which reads as the ratio of the medians. Worked on four plausible (Cover My Week seconds, manual seconds) scenarios -- (60, 100), (30, 120), (200, 300), (50, 60): median of ratios = median(0.60, 0.25, 0.667, 0.833) = (0.60 + 0.667)/2 = 0.633, which fails a 50% bar; ratio of medians = 55/110 = 0.50, which passes it. The same dataset flips the verdict. Fix: make `MVP-022:53` read "on the median of the per-scenario ratios recorded under `MVP-024` AC-8", matching AC-8's wording exactly. Deferred: MVP-022:53's reference is explicitly "not binding" and AC-6 takes its number from AC-8's definition, so the operative statistic is unambiguous; only the two definitions disagree.
  **Status:** OPEN

- **The MVP-011 <-> MVP-023 coupling was decoupled in one direction only** (`docs/task/mvp/MVP-011_STARTER_CONTENT_READINESS.md:35`, against `KNOWN_ISSUES-low.md`'s "MVP-023 requires starter meals as a candidate source but does not depend on MVP-011" entry marked RESOLVED 2026-08-26) -- D-036 took the decoupling option on the MVP-023 side: its scope now states the starter-meal candidate source is built against the shared entity shape and does not require MVP-011's shipped content, and the prior entry is marked RESOLVED as if the coupling is fully handled. The reciprocal half is untouched: `MVP-011:35` still reads "every shipped recipe must satisfy the same structured shape MVP-023 hard-filters and scores", and `docs/task/SEQUENCE.txt` runs MVP-011 at steps 20-21, twelve steps before MVP-023 at 32-33. At MVP-011 planning time that shape exists nowhere -- MVP-023 has not been built, and MVP-011's Authoritative sources cite PRD §9.3, §8 and §16 but not §9.4, where hard filtering is actually specified. Fix: change `MVP-011:35` to cite PRD v3 §9.4's hard-filter inputs directly instead of `MVP-023`, add §9.4 to MVP-011's Authoritative sources, and optionally append one clause to the prior RESOLVED note recording that the MVP-011 side was closed by re-anchoring rather than by a dependency edit. Deferred: naming drift, not an unmeetable requirement -- the constraint names restrictions and prep time, both derivable from PRD §8/§9.4 today.
  **Status:** OPEN

## master -- 2026-08-27

Source: /home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-plan-rev8-2026-08-27T1321-869f.md

### LOW

- **Step 1b implements one of the three obligations `docs/task/README.md:66` places on the Elevated fresh-context verifier** (the MVP-003 plan's step 1b, "Fresh-context verifier (README Elevated tier)", against `docs/task/README.md:66`) -- the plan cites the README Elevated tier as 1b's authority and calls it "a **gate**, not a formality". README:66 reads: "Elevated: tests should be derived from acceptance criteria before or independently from implementation when practical; a fresh-context verifier reruns them, investigates failures, and checks material coverage gaps." Step 1b instructs the subagent to rerun `cargo test --workspace` and `(cd rust && cargo build --release) && flutter test`, confirm one AC-3 line in `docs/V3_IMPLEMENTATION_STATUS.md`, and report pass counts. Failure investigation is covered implicitly by "Any failure -> stop", but the material-coverage-gap check is absent -- and that is the half a rerun cannot substitute for, on the card whose promotion to `Done` the whole of step 1c hangs on. Fix: add one clause to the subagent brief -- check MVP-002's four ACs against the tests that claim them and report any criterion with no test carrier -- or state in the plan why a coverage check is not re-run at promotion time (the card already recorded 4/4 PASS at Verify), so the divergence from README:66 is deliberate rather than dropped. Deferred: the rerun half is the load-bearing half at promotion time, and the owner already exercised the coverage judgement when the card reached Verify.
  **Status:** RESOLVED 2026-08-28 (fixed in the MVP-003 plan, rev 9–11) -- moot rather than implemented: step 1b was removed entirely, and the divergence from `docs/task/README.md:66` is now recorded in the plan's Context section as a deliberate, visible acceptance. No coverage-gap check was added.

- **The MVP-003 plan's step-7 `setUp` line is uncompilable as written; only the `tearDown` half carries the correction** (the plan's step 7, the line beginning "`setUp`: `tester.view.physicalSize`") -- the line reads "`setUp`: `tester.view.physicalSize = const Size(1080, 2400); tester.view.devicePixelRatio = 2.75;` -- `tearDown`: `...` (done via `addTearDown` inside each `testWidgets`, since `tester` is per-test)." The parenthetical corrects only the tearDown half, but `tester` is equally out of scope in a `setUp` callback: both halves must live inside each `testWidgets` body. An executor transcribing the literal text writes a `setUp` that does not compile. Fix: restate as "at the top of each `testWidgets` body: set `physicalSize`/`devicePixelRatio`, then `addTearDown(...)` for the four resets" -- one sentence, no `setUp`/`tearDown` framing. Deferred: `flutter analyze` catches it within one step of being written.
  **Status:** RESOLVED 2026-08-28 (fixed in the MVP-003 plan, rev 9–11) -- step 7 now reads "at the top of each `testWidgets` body", with neither `setUp` nor `tearDown` framing.

- **The MVP-003 plan's step 10 marks both `SEQUENCE.txt` MVP-003 steps `Done`, but step 3 is a conditional that will not have run** (the plan's step 10, the `docs/task/SEQUENCE.txt` bullet anchored on `MVP-003_ANDROID_APP_SHELL_NAVIGATION.md`, against `docs/task/SEQUENCE.txt` steps 3 and 4) -- the two hits for the card filename are step 3, `[ONLY if no accessible current approved MVP-003 plan is supplied] /plan-task ...`, and step 4, `/execute-plan`. This plan is the supplied approved plan, so step 3's guard is false and the step never runs; marking it `# Done <TODAY> -- see the docs/ROADMAP.md evidence log.` records a `/plan-task` invocation that did not happen. The plan holds itself to a higher bar elsewhere: step 1c item 9 justifies its `SEQUENCE.txt` step-0 mark with "Step 0 named exactly four targets ... so this mark is honest." Fix: mark step 4 `Done <TODAY>`; mark step 3 with its actual disposition (`not run -- a current approved plan was supplied`), or state in the plan why "Done" is the right mark for a skipped conditional. Deferred: ledger honesty only; both marks retire the same file section and no downstream reader acts on the distinction.
  **Status:** RESOLVED 2026-08-28 (fixed in the MVP-003 plan, rev 9–11) -- though not as this entry proposed: the guard *did* fire. The earlier plan was not resolvable from the repository, so `/plan-task` ran on 2026-08-28 and produced the rev-11 plan that step 4 then executed. Both marks record steps that actually happened.

- **The MVP-003 plan's PRE-002 toolchain re-anchor covers the parenthetical but not the second reference to the deleted test in the same bullet** (the plan's step 10, the bullet anchored on `every Dart test -- the fake-fed widget test included`, against `docs/ROADMAP.md:275`) -- the roadmap bullet names the deleted file twice: "...so every Dart test -- **the fake-fed widget test included** -- needs rustup + the pinned toolchain on the host; **the widget test** avoids *loading* the library, not *building* it." Step 10 instructs only "re-anchor the parenthetical to `test/app_test.dart`", leaving the second clause's bare "the widget test" pointing at a file step 6 deletes. Both references are true of `test/app_test.dart` (it is fake-fed via `ProviderScope` overrides and never calls `RustLib.init()`), so the fix is mechanical -- the instruction just does not reach the second site. Fix: name both sites in the bullet, or state the edit as "replace both occurrences of the deleted test's referent in this bullet with `test/app_test.dart`". Deferred: a one-word staleness in a prose bullet whose substance stays correct; mechanical to fix later.
  **Status:** RESOLVED 2026-08-28 (fixed in the MVP-003 plan, rev 9–11) -- step 10 now names both referents, and both were replaced with `test/app_test.dart`.

## master -- 2026-08-27

Source: /home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-plan-rev9-2026-08-27T1602-03c3.md

### LOW

- **The MVP-003 plan's font_scale EXIT trap is cleared before the reset is verified** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:623`, against `:625`) -- `trap - EXIT` runs one line before the `settings get system font_scale` assertion that checks whether the trap was needed. That read is `|| true`-guarded, so a transient bridge failure yields an empty string, fails the `= "1.0"` comparison, and exits 1 with the safety net already disarmed and the device's real scale unknown -- durable host state inherited by MVP-004 onward. Fix: move `trap - EXIT` to after the assertion.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-plan-rev9-2026-08-27T1602-03c3.md
  **Status:** RESOLVED 2026-08-27 (fixed in the MVP-003 plan, rev 10: `trap - EXIT` moved below the `settings get system font_scale` assertion, so a failed assertion exits with the trap still armed and the 1.0 reset re-runs)

- **The MVP-003 plan's step-9 gate fragment contradicts the block's own `set -euo pipefail` rule** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:560`, against `:594`) -- the header states "EVERY fragment starts with `set -euo pipefail`" and that the preamble "is REPEATED VERBATIM at the top of every fragment", but the first of four fragments (through CUT 0) has none; the first occurrence is after the CUT 0 marker. Harmless as written -- the single command carries its own `|| { exit 1; }` -- but any line added there inherits no `-e`, the rev-8 condition the comment exists to prevent. Fix: hoist `set -euo pipefail` above the `status || up` gate, or scope the claim to fragments after CUT 0 and say why the gate is exempt.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-plan-rev9-2026-08-27T1602-03c3.md
  **Status:** RESOLVED 2026-08-27 (fixed in the MVP-003 plan, rev 10: `set -euo pipefail` hoisted above the `status || up` gate, and the header's preamble claim rewritten to what is true per fragment -- measured 1/4/3/3 lines)

## master -- 2026-08-27

Source: /home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-plan-rev10-2026-08-27T2030-6780.md

### LOW

- **The MVP-003 plan's precedence rule describes the step-9 and step-10 decision points with pre-§0 arms** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:71`) -- it exempts "step 9's `NOT VERIFIED` path, step 10's `Done`-or-`Verify` choice", but step 9 now has three outcome states and step 10's register cell three arms. Both stay correctly exempt, so nothing writes a wrong value; the plan's own branch inventory just under-counts them. Fix: point both at step 9 §0.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-plan-rev10-2026-08-27T2030-6780.md
  **Status:** OPEN

- **The MVP-003 plan's step-9 third fragment is a bash syntax error as written** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:613`) -- `adb shell input tap <x> <y>`: bash reads the angle brackets as redirections, so `bash -n` over the fragment fails at parse time and the error points at line 4 rather than at the value the executor forgot to substitute. Fails safe (nothing runs). Fix: `"$TAPX" "$TAPY"`, so `set -u` names the missing value.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-plan-rev10-2026-08-27T2030-6780.md
  **Status:** OPEN

- **The MVP-003 plan's CUT-0 marker omits the timeout the header calls mandatory for the fragment it opens** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:585`, against `:575`) -- CUT 1 and CUT 2 carry `timeout: 180000` inline; CUT 0 carries none, and the install fragment is the one the header says "must NOT inherit the 120s default". A 120s kill mid-install drops the `^Success` line, so the gate classifies a harness timeout as an in-scope card failure. Fix: state `timeout: 300000` in the CUT-0 marker.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-plan-rev10-2026-08-27T2030-6780.md
  **Status:** OPEN

## master -- 2026-08-27

Source: /home/davidlinux/.claude/reviews/redteam-mvp003-plan-rev10-2026-08-27T2030-6780.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-review-audit-2026-08-27T2129-abf3.md

### LOW

- **The rev-10 review's `font_scale` finding cites a contract scoped to a block that gate is not in** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:646`, against `:680`) -- the quoted contract actually reads "**The pid block** exits non-zero on any failed gate and prints a line containing `FAIL`", and the pid block starts at `:648`, after the `font_scale` assertion. The gap is real on other grounds (7 of 8 `exit 1` sites carry the token) but the cited authority does not say it. Fix: rebase the finding on the measured token convention, or widen `:680` past the pid block.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-review-audit-2026-08-27T2129-abf3.md
  **Status:** OPEN

- **The MVP-003 plan's step-9 CUT-0 sites restate the `S-BRIDGE` arm that §0 claims exclusivity over** (`/home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md:584` and `:585`, against `:686`) -- both say "go to step 10 with status Verify" in prose instead of selecting on §0, while `:686` reads "Nothing else re-enumerates them". They agree with `:688` today, so nothing writes a wrong value; the false exclusivity claim is what lets the next revision leave a site behind. Fix: point both at §0's `S-BRIDGE` row, or narrow `:686` to downstream consumers.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-review-audit-2026-08-27T2129-abf3.md
  **Status:** OPEN

## orch/4 -- 2026-08-28

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-4-2026-08-28T1752-5375.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-android-shell-2026-08-28T1805-b91e.md

### LOW

- **`docs/ROADMAP.md`'s Next-implementation-task line omits the blocked-lane qualification** (`docs/ROADMAP.md:9`) -- it names `MVP-004` with no status, but `docs/task/README.md:20` requires the next card be `Ready` (MVP-004 is `Draft`, register `:40`) and `:22` requires no card sit at `Verify` (MVP-003 does, `:39`), so a reader of the compact Current-milestone block can start planning a blocked card. Fix: restore a qualifier -- "`MVP-004` -- `Draft`; blocked until MVP-003 reaches `Done`".
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-android-shell-2026-08-28T1805-b91e.md
  **Status:** OPEN

- **`App.initialLocation` is silently discarded on rebuild** (`lib/app/app.dart:19`) -- `late final _router = buildRouter(...)` runs once and there is no `didUpdateWidget`, so a second `pumpWidget` of a same-shaped tree reuses `_AppState` and drops a changed `initialLocation` with no error or analyzer warning; `test/app_test.dart:142` already pumps twice and depends on that reuse. A later test varying `initial` across two pumps in one body asserts against the old location. Fix: document the field as construction-time-only, or assert equality in `didUpdateWidget`.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-android-shell-2026-08-28T1805-b91e.md
  **Status:** OPEN

- **The tab-traversal test's press count is an undocumented magic number** (`test/app_test.dart:175`) -- exactly five tab events, tuned to `/plan`'s current focusables (the Cover My Week button at `lib/app/router.dart:40` precedes the bar's five destinations), and the `expect(focused, isNotNull)` at `:180` is near-vacuous because `primaryFocus` is always set in a pumped app. When MVP-013 changes the Plan tree the test fails pointing at bar focusability rather than the real cause. Fix: loop tab presses, bounded, until the focused node's ancestor is a `NavigationBar`.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-android-shell-2026-08-28T1805-b91e.md
  **Status:** OPEN

---

## integration -- 2026-08-28

Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-integration-verify-2026-08-28T1904-03fd.md

### LOW

- **`App`'s default `initialLocation` is never exercised** (`lib/app/app.dart:10`, `test/app_test.dart:17-24`) -- the harness always passes `initial` (default `/plan`), so the production default `homeLocation` that `lib/main.dart:13` relies on is never constructed under test; both agree today, so a divergence would pass silently. Fix: let the harness omit the argument when `initial` is null, or add one test pumping `const App()` and asserting the Plan title.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-integration-verify-2026-08-28T1904-03fd.md
  **Status:** OPEN

- **Re-tapping the active tab resets its stack with no test** (`lib/app/shell.dart:17`) -- the only shell behaviour `test/app_test.dart` does not pin; a regression to "no-op on re-tap" would pass. Fix: push `/plan/cover`, re-tap Plan, assert the Plan title.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-integration-verify-2026-08-28T1904-03fd.md
  **Status:** OPEN

- **MVP-003 evidence row ends "Not committed or pushed" after the merge to `integration`** (`docs/ROADMAP.md:172`) -- the row is commit `c28232c`, merged as `844d0f4` on `integration`; only `master` lacks it. Earlier rows share the convention, so this needs a handoff-contract decision. Fix: drop the sentence at the orchestrator's merge step, or make it precise ("on `integration` as `<sha>`; not on `master`").
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-integration-verify-2026-08-28T1904-03fd.md
  **Status:** OPEN

---

## orch/6 -- 2026-08-28

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-6-2026-08-28T2309-b6ae.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md

### LOW

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

- **`healthReportProvider` and `health_provider.dart` no longer describe what they do** (`lib/features/settings/health_provider.dart:12`) -- `health_check` became `open_database` and now installs the process-wide connection as its primary effect, so this provider is the app's connection-lifetime owner, not a diagnostic; `lib/app/app.dart:43` needs prose to explain that. It also sits under `features/settings` while being a startup concern. Fix: rename to `databaseProvider` outside `features/settings` — four call sites.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md
  **Status:** DEFERRED 2026-08-28 -- fix-both-or-defer-both. Renaming only the Dart provider leaves `HealthReport`, `lib/src/rust/api/health.dart` and `rust/src/api/health.rs` carrying the identical stale vocabulary, so `databaseProvider` would return a `HealthReport` from `health.dart` -- the same naming defect, half-fixed. Fixing both means renaming the Rust module and regenerating the FRB bridge (`frb_generated.rs`, `frb_generated.dart`, `.io.dart`, `.web.dart`), out of proportion for a card at `Verify`. Fold into the next card that touches these providers, renaming both sides in one pass.

---

## orch/8 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-8-2026-08-29T0109-5f49.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md

Note: the review exists as two copies, and the `~/.claude/reviews/` one every `Full review:` line below points at
is the **pre-fix snapshot** -- writes outside the worktree were blocked in the 2026-08-29 fix-pass session, so
only the worktree copy, `.orch/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md`, carries the closed statuses
(`RESOLVED -- 5 fixed, 2 deferred, 0 retracted`) and the per-finding reasoning the entries below summarise. The
two are otherwise byte-identical. Reconcile the copies when this branch merges.

### LOW

- **`planningCycleProvider`'s anti-rebuild `selectAsync` choice has no test** (`lib/features/planning/planning_provider.dart:13`) -- the comment claims watching the whole future would cost a bridge write and a "Loading…" flicker on every rename; nothing proves it, and `_FakePlanningCycleNotifier` (`test/app_test.dart:84`) re-implements the same line, so a switch to `.future` stays green. Fix: a rename test on the real notifier asserting one `ensurePlanningCycle` call and no loading string.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md
  **Status:** OPEN -- deferred in the 2026-08-29 fix pass under batch criterion (b): the regression costs a
  "Loading…" flicker plus one extra `ensurePlanningCycle`, which is idempotent by construction
  (`kimatta-storage/src/lib.rs:285`-`:291` returns the stored row and ignores `default_cycle`), so no stored or
  displayed value is wrong. UX polish with no correctness impact. Fold into the next card that touches this provider.

- **The unknown-slot read path is untested and `slot` is the only unconstrained column** (`rust/crates/kimatta-storage/src/lib.rs:339`) -- the two sibling read-side guards have tests (`:572`, `:631`) but `MealSlot::parse`'s failure has none, and `planning_meal_slot.slot` (`:63`) carries no CHECK while both columns beside it in the same migration do. A `'supper'` row surfaces as an untested `KimattaError::Storage`. Fix: a storage test mirroring `:631`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md
  **Status:** OPEN -- half fixed 2026-08-29. The read path is now pinned by
  `an_unknown_slot_row_is_rejected_on_read`, which inserts a `'supper'` row directly and asserts
  `StorageError::Planning(UnknownMealSlot("supper"))`. The `CHECK (slot IN (...))` half is deliberately not
  applied: migration 2 is immutable on shipped databases, so pinning the slot vocabulary there costs a third
  migration the first time a slot is added. That half stays open.

- **`save_planning_cycle` is the only read-then-write transaction that is not IMMEDIATE** (`rust/crates/kimatta-storage/src/lib.rs:302`) -- it reads through `require_household` then writes under a DEFERRED transaction, while `ensure_household` (`:201`) and `ensure_planning_cycle` (`:282`) both use IMMEDIATE. Unreachable behind the single connection at `rust/src/db.rs:9`; a second connection turns it into a 5s busy stall. Fix: `TransactionBehavior::Immediate`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md
  **Status:** RESOLVED 2026-08-29 -- fix pass: `save_planning_cycle` now opens
  `transaction_with_behavior(TransactionBehavior::Immediate)`, matching its two read-then-write siblings, and its
  doc comment states the lock ordering. Pinned by `save_takes_the_write_lock_before_it_reads`, which holds RESERVED
  on a second connection with the busy timeout at zero and asserts the `BEGIN` is refused before
  `require_household` runs; confirmed red pre-fix (`got NoSuchHousehold("absent")`). `insert_household` (`:94`) is
  left DEFERRED deliberately -- it is write-only, so its first statement takes the write lock with no upgrade.

- **Four `kimatta-storage` re-exports have no consumer** (`rust/crates/kimatta-storage/src/lib.rs:6`) -- `CivilDate`, `DEFAULT_CYCLE_DAYS`, `MIN_CYCLE_DAYS` and `MAX_CYCLE_DAYS` appear outside `food-domain` only on the re-export line, so the crate's public API overstates what is load-bearing and `cargo` warns on none of it. Fix: drop them, or let the schema-bound test consume the two `*_CYCLE_DAYS` constants.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md
  **Status:** OPEN -- deferred in the 2026-08-29 fix pass under batch criterion (d), fix-both-or-defer-both, and the
  second half of the suggested fix is withdrawn as unsound: `the_length_check_constraint_matches_the_domain_bounds`
  does now read `MIN_CYCLE_DAYS`/`MAX_CYCLE_DAYS`, but it reaches them through `use super::*` from the crate's own
  `mod tests`, a path that resolves whether or not line 6 says `pub`. An in-crate test cannot justify a `pub use`.
  On the other half: `CivilDate` cannot be dropped, because it is the return type of the re-exported
  `parse_civil_date`, the parameter type of `format_civil_date` and the return type of `PlanningCycle::anchor()` --
  all re-exported here and all used by `rust/src/api/planning.rs`, which would otherwise need its own `food-domain`
  dependency to name the type, reintroducing the coupling `food-domain/src/lib.rs:8` re-exports jiff's `Date` to
  avoid. One member of the set cannot be fixed, so the set is deferred. Revisit when MVP-006's editor gives the
  three constants a real out-of-crate caller.
