# Known Issues — LOW

- **Accept does not visibly settle after a covered week is accepted** (`lib/features/planning/cover_screen.dart`, `docs/bugs/MVP-033_ACCEPT_FEEDBACK.md`) — on Samsung Galaxy S20 FE / Android 13, release `v1.0.1+2`, the first-run proposal accepted in one tap and exposed the shopping-list route, but the enabled-looking **Accept** button stayed in place with no explicit success acknowledgement. The result is ambiguous feedback, not a failed write or blocked navigation. Fix: after a successful `_notifier.accept()`, replace or disable the primary action and retain the calm shopping-list route; cover success, failure, and repeat-tap behavior in widget tests. **Status:** OPEN.

## orch/4 -- 2026-08-28

Full review: (lost) `wt/4/.orch/redteam-app-dart-2026-08-28T1845.md` was removed with the step-4 worktree before the orchestrator relayed it; the entries below are the only surviving record. Follow-up review: /home/davidlinux/.claude/reviews/redteam-mvp003-integration-verify-2026-08-28T1904-03fd.md

### LOW

- **`dispose()` can re-run `buildRouter` when the first build threw** (`lib/app/app.dart:21-23`, `:36-39`) -- `_router` is `late final`, initialized on first read inside `build`. Dart re-runs a `late final` initializer if a prior attempt threw, and `dispose()` reads `_router` unconditionally, so a throwing `buildRouter` produces a second construction and a second exception during unmount, obscuring the original. Not reachable today — `buildRouter` cannot throw as `lib/app/router.dart` currently stands — but it becomes reachable as the router grows: `go_router-18.0.0/lib/src/route.dart:1085-1090` asserts a `restorationScopeId` on the `StatefulShellRoute` whenever any branch sets one, which is exactly this router's shape. Fix: make the field nullable and dispose with `_router?.dispose()`, or gate disposal on a `bool` set once `build` completes. Deferred: debug-time diagnosability only, and the guard costs the `late final` idiom the class is built around; revisit if `buildRouter` gains a throwing path.
  **Status:** OPEN

---

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-redteam-pass2-2026-08-24T1320-5bbe.md

### LOW

- **The new entry is filed under a section header attributing it to a different review** (`KNOWN_ISSUES.md`, the section whose `Full review:` names `redteam-dec-004-name-shortlist-2026-08-24T0837-e592.md`) -- that section's header declares `Full review: .../redteam-dec-004-name-shortlist-2026-08-24T0837-e592.md` (pass 1), but its trailing entry comes from pass 2 and carries its own trailing `Full review: ...T1303-7845.md` to override the header. A reader or a later `/ki-maintain` run that trusts the section header follows the pass-1 link and does not find the entry. The placement itself is correct — the alternative section's `Source:` is `docs/task/README.md`, the wrong review chain, and the entry reads naturally beneath the pass-1 entry it supersedes. Fix: promote the pass-2 review link into the section header as a second `Full review:` line covering the chain, or accept the inline override and note it. Deferred: the inline `Full review:` override already resolves it in practice.
  **Status:** OPEN

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-fix-findings-2026-08-24T1442-970e.md

### LOW

- **`/redteam-code` regenerates a dead `Source:` header on every run** (`~/.claude/commands/redteam-code.md:121`; symptom was the section headers of this file and `KNOWN_ISSUES.md`) -- the command spec requires a new known-issues section's `Source:` header to be its `**Reviewed:**` target, which for a `/redteam-code` run is an impl-handoff doc the command disposes at the end of that same run. Six such dead `Source:` lines accumulated across both trackers and were deleted 2026-08-24; the next run adds a seventh. **Convention adopted 2026-08-24:** drop `Source:` when its target is a disposed handoff; keep it when it names a durable in-repo file, as the `docs/task/README.md` section of `KNOWN_ISSUES.md` does. The adjacent `Full review:` line always resolves and is sufficient alone. Fix: encode that rule in the `/redteam-code` spec. Deferred: the skill file is outside this repo and was ruled out of scope for the fix pass that found this; tracked here so the rule survives the pass.
  **Status:** OPEN

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-fix-findings-2026-08-24T1537-7fba.md

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-2026-08-24t2107-8195-2026-08-24T2111-0bd4.md

### LOW

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

- **DEC-005's single-`rusqlite`-version guard has no carrier in the Rust gate MVP-002 added** (the `cargo fmt --all --check && cargo clippy` gate line in `docs/ROADMAP.md`'s DEC-002 block; the `rusqlite` and `rusqlite_migration` lines in `rust/crates/kimatta-storage/Cargo.toml`) -- DEC-005 decision 4 makes `rusqlite_migration` conditional ("dropped only if `cargo tree -i rusqlite` shows more than one version"), but the dependencies are caret ranges (`rusqlite = { version = "0.40", ... }`, `rusqlite_migration = "2"`), so a future `cargo update` can pull a `rusqlite_migration` minor that bumps its `rusqlite` requirement and yield two `libsqlite3-sys` copies under the `bundled` feature. AC-1 verified one version once, by hand; the new gate line runs `fmt`, `clippy`, and `test` only, so the recorded rationale for keeping the crate would silently stop being validated. Fix: append `cargo tree --workspace -i rusqlite` to the Rust gate line, or pin both dependencies to exact versions so `cargo update` becomes a deliberate act. Deferred: failure mode is a loud link-time duplicate-symbol error, and `Cargo.lock` is committed.
  **Status:** OPEN

## master -- 2026-08-25

Raised during the fix pass for /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp002-2026-08-24T2213-6556.md, not by the review itself.

### LOW

- **`insert_household` accepts a member belonging to a different existing household** (`rust/crates/kimatta-storage/src/lib.rs`, `insert_household`) -- moved to `KNOWN_ISSUES.md` on 2026-08-25 and re-rated MEDIUM by the arch review of `rust/src`; that entry carries the full text plus the two additions the review made. Tracked there, not here.
  **Status:** MOVED 2026-08-25 -- see `KNOWN_ISSUES.md`, same title.

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

## master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-mvp003-redteam-pass1-2026-08-26T0847-3c5a.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-redteam-pass1-2026-08-26T0928-3cb9.md

## master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-mvp003-fixfindings-rev6-2026-08-26T1010-5279.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-fixfindings-rev6-2026-08-26T1343-37fc.md

## master -- 2026-08-26

Source: /home/davidlinux/.claude/reviews/impl-handoff-v3-card-rederivation-2026-08-26T1337-0211.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-v3-card-rederivation-2026-08-26T1626-ad3a.md

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

## master -- 2026-08-27

Source: /home/davidlinux/.claude/plans/goofy-juggling-stroustrup.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp003-plan-rev9-2026-08-27T1602-03c3.md

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

- **`healthReportProvider` and `health_provider.dart` no longer describe what they do** (`lib/features/settings/health_provider.dart:12`) -- `health_check` became `open_database` and now installs the process-wide connection as its primary effect, so this provider is the app's connection-lifetime owner, not a diagnostic; `lib/app/app.dart:43` needs prose to explain that. It also sits under `features/settings` while being a startup concern. Fix: rename to `databaseProvider` outside `features/settings` — four call sites.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-6-2026-08-28T2327-6b40.md
  **Status:** DEFERRED 2026-08-28 -- fix-both-or-defer-both. Renaming only the Dart provider leaves `HealthReport`, `lib/src/rust/api/health.dart` and `rust/src/api/health.rs` carrying the identical stale vocabulary, so `databaseProvider` would return a `HealthReport` from `health.dart` -- the same naming defect, half-fixed. Fixing both means renaming the Rust module and regenerating the FRB bridge (`frb_generated.rs`, `frb_generated.dart`, `.io.dart`, `.web.dart`), out of proportion for a card at `Verify`. Fold into the next card that touches these providers, renaming both sides in one pass.

---

## orch/8 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-8-2026-08-29T0109-5f49.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md

Note: the review exists as two copies, and the `~/.claude/reviews/` one every `Full review:` line below points at
is the **pre-fix snapshot** -- writes outside the worktree were blocked in the 2026-08-29 fix-pass session, so
only the worktree copy, `~/.claude/reviews/redteam-impl-handoff-orch-8-2026-08-29T0120-ca0e.md`, carries the closed statuses
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

## orch/10 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-10-2026-08-29T0221-5a60.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-10-2026-08-29T0230-130e.md

## orch/12 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-12-2026-08-29T0423-be39.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-12-2026-08-29T0439-99c7.md

### LOW

---

- **`_pending` discards the stored first-seen restriction order** (`lib/features/restrictions/restrictions_screen.dart:75`) -- emits checked known kinds in vocabulary order then free text, destroying the order `HouseholdRestrictions` documents preserving at `rust/crates/food-domain/src/restriction.rs:109`; the Settings subtitle re-renders reordered after any save. Fix: build from one ordered pending list, or soften the domain comment.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-12-2026-08-29T0439-99c7.md
  **Status:** OPEN

---

- **`load_restrictions` accepts an absent household where `save_restrictions` rejects it** (`rust/crates/kimatta-storage/src/lib.rs:568`) -- no `require_household`, so a bogus id returns an empty set that `lib/features/settings/settings_screen.dart:60` renders as the affirmative `None set — nothing is filtered out.`; the sibling `load_member_preferences` (:692) does check. Unreachable until MVP-009 holds an id from elsewhere. Fix: add `require_household`, or record the asymmetry where MVP-009 reads it.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-12-2026-08-29T0439-99c7.md
  **Status:** OPEN

---

- **Corrupt preference rows lose the coordinates corrupt restriction rows keep** (`rust/crates/kimatta-storage/src/lib.rs:702`) -- `restriction_from_row` reports `CorruptRestriction { household, position, kind, text }` (:611), but the preference loader maps straight into transparent `StorageError::Preference`, so a bad row surfaces as `unknown sentiment "liek"` with no member and no position. Fix: a `CorruptPreference { member, position, sentiment, subject }` variant mirroring `CorruptRestriction`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-12-2026-08-29T0439-99c7.md
  **Status:** OPEN

---

## orch/14 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-14-2026-08-29T0605-a1a4.md
Full review: /home/davidlinux/.claude/reviews/redteam-recipe-crud-handoff-2026-08-29T0615-043d.md

### LOW

- **A malformed archive date is reported as a `Planning` error** (`rust/src/api/recipe.rs:381`) -- `archive_recipe_in` uses `parse_civil_date`, whose error maps to `KimattaError::Planning`, so a recipe-archive rejection reaches the user as "Recipes unavailable: <planning message>"; `archive_rejects_a_non_civil_date_as_planning_error` pins the mislabelling. Unreachable while the only caller passes `todayCivilDate()`. Fix: map to `KimattaError::Recipe`, or document the deliberate reuse in the test's doc comment.
  Full review: /home/davidlinux/.claude/reviews/redteam-recipe-crud-handoff-2026-08-29T0615-043d.md
  **Status:** OPEN

---

- **A row touched only via `Optional` or `Unit` is silently dropped** (`lib/features/recipes/recipe_form_screen.dart:154`) -- the blank-row skip tests only the five text controllers and ignores `unitKey`/`optional`, so a row where the user picked `cup` and flipped `Optional` is `continue`d: not validated, not saved, not reported, and the form navigates away. Self-declared as an open low risk in the handoff. Fix: add `unitKey != unitNoneKey || optional` to the emptiness test so a touched row is validated.
  Full review: /home/davidlinux/.claude/reviews/redteam-recipe-crud-handoff-2026-08-29T0615-043d.md
  **Status:** OPEN

## orch/14 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-14-2026-08-29T0658-3755.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0713-5c00.md

### LOW

- **`recipeDetailProvider` is a non-`autoDispose` family** (`lib/features/recipes/recipes_provider.dart:100`) -- the only `.family` in `lib/`, so Riverpod retains one instance per recipe id opened for the `ProviderScope`'s life, each holding a full `RecipeDto` and a watch on `recipeLibraryProvider`; a long browse of a large library grows unboundedly and nothing frees it short of a restart. Fix: `FutureProvider.autoDispose.family` -- both screens watch it while mounted, so nothing needs the cache longer.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0713-5c00.md
  **Status:** OPEN -- considered and deliberately not fixed in the orch/14 fix pass 2026-08-29, per the reviewer's own `defer:` disposition. `autoDispose` is a one-token change, but it trades the memory bound for a user-visible full-body `CircularProgressIndicator` on every re-open of a recipe (`recipe_detail_screen.dart:86-97` routes `AsyncLoading` to the spinner arm), which is a UX call with no correctness impact. Fold into the next card that touches these providers.

---

---

- **Restore navigates away from the detail screen but not from the archived list** (`lib/features/recipes/recipe_detail_screen.dart:58`) -- `_run` is shared by archive and restore and always ends `context.go('/recipes')`, so restoring from a recipe you are reading ejects you to the library, while `ArchivedRecipesScreen._restore` (`recipe_list_screen.dart:68`) stays put. Same action, two outcomes. Fix: give restore its own continuation that stays on the detail screen and lets `recipeDetailProvider` re-render it.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0713-5c00.md
  **Status:** OPEN -- considered and deliberately not fixed in the orch/14 fix pass 2026-08-29, per the reviewer's own `defer:` disposition ("needs a UX call on the intended landing spot, not just a code change"). It is the only user-visible behaviour change the pass would have made, and this codebase records calls of that kind as `(owner decision <date>)` (`recipes_provider.dart:53`, `recipe_detail_screen.dart:22`); the call was not made unattended. Note for whoever takes it: staying put is not a quiet re-render -- a recomputed `recipeDetailProvider` emits `AsyncLoading`, which `recipe_detail_screen.dart:86-97` routes to a full-body spinner, so the user sees content → spinner → content.

## orch/14 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-14-2026-08-29T0750-b94e.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0804-a280.md

## orch/14 -- 2026-08-29

Source: test/app_test.dart
Full review: /home/davidlinux/.claude/reviews/redteam-app-test-2026-08-29T0823-c1d2.md

### LOW

- **`/recipes/archived`'s accessibility check measures the empty state** (`test/app_test.dart:2739`) -- the a11y loop passes `recipes:` but not `archived:`, so that iteration measures `Center(Text('Nothing archived.'))` and never checks the screen's only control, the `Restore` `TextButton` in `ListTile.trailing` (`lib/features/recipes/recipe_list_screen.dart:99`). Fix: add `archived: () => const [okSummary]` to the loop's harness call -- inert on the other three locations.
  Full review: /home/davidlinux/.claude/reviews/redteam-app-test-2026-08-29T0823-c1d2.md
  **Status:** OPEN

---

- **`ArchivedRecipesScreen._restore`'s failure path is untested** (`lib/features/recipes/recipe_list_screen.dart:68`) -- its `catch`, `finally` and `_busy` gate are a second copy of the detail screen's `_run` shape and only the detail copy is pinned (`test/app_test.dart:1895`), so dropping the `finally` leaves every `Restore` button permanently disabled after one failed restore. Fix: one test with `restoreRecipe:` throwing, asserting the snackbar and that `onPressed` is non-null after.
  Full review: /home/davidlinux/.claude/reviews/redteam-app-test-2026-08-29T0823-c1d2.md
  **Status:** OPEN

---

- **The recipe form's tab-traversal bound is an undocumented `40`** (`test/app_test.dart:2762`) -- its sibling at `:672` uses `maxTabPresses`, a constant given a rationale because `KNOWN_ISSUES-low.md:319` flagged the same pattern; when MVP-009 adds row controls the failure will read as "Add ingredient lost focusability" rather than "the bound is stale". Fix: hoist to `maxFormTabPresses` beside `maxTabPresses`, with one line on what it is sized against.
  Full review: /home/davidlinux/.claude/reviews/redteam-app-test-2026-08-29T0823-c1d2.md
  **Status:** OPEN

## orch/14 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-14-2026-08-29T0900-dcd3.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0911-2a9d.md

### LOW

---

- **`ArchivedRecipesScreen._restore` returns silently on a null household id** (`lib/features/recipes/recipe_list_screen.dart:70`) -- a bare early return with the `Restore` button still enabled, so the tap reads as a broken button; `RecipeFormScreen` disables Save on the same condition (`recipe_form_screen.dart:360`) and `RecipeDetailScreen` takes the id off the DTO. Near-unreachable, since `archivedRecipesProvider.build` awaits the household first. Fix: gate `onPressed` on the id, or take it from the DTO as the detail screen does.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-14-2026-08-29T0911-2a9d.md
  **Status:** OPEN -- deferred at the orch/14 pass-1 fix round, deliberately. Two of the review's three suggested directions are unavailable or unpinnable: taking the id off the DTO is impossible because `RecipeSummaryDto` carries only `id` and `title` (`lib/src/rust/api/recipe.dart:260`-`264`), unlike the full `RecipeDto` the detail screen holds; and both gating `onPressed` and adding the `describeFailure` snackbar are behaviour-preserving in every state the app can reach, because `ArchivedRecipesNotifier.build` awaits `householdProvider.selectAsync(...)` before the list renders. Pinning either would need a fake that omits that await and so contradicts production. Gating `onPressed` is the only shape available without widening the bridge DTO; revisit when a caller can reach the archived list without a resolved household.

## orch/16 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-16-2026-08-29T1022-c507.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-16-2026-08-29T1040-68c3.md

### LOW

- **Exception windows are located in the unmasked `hay`, so overlapping exceptions would over-mask** (`rust/crates/food-domain/src/restriction.rs:371`) -- unreachable on the current table (all nine exceptions are two tokens; neither `milk` nor `butter` is any phrase's first token), but adding `"butter milk"` or `"milk chocolate"` would silently clear the shared token and suppress a warning -- the false-negative direction invariant 10 forbids. The safety property is an accident of the data, not a guard. Fix: a synthetic overlapping-pair test, or locate windows against `masked`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-16-2026-08-29T1040-68c3.md
  **Status:** OPEN

---

- **`first_term` re-tokenises every static term on every call** (`rust/crates/food-domain/src/restriction.rs:387`) -- `contains_phrase(&masked, &tokens(term))` allocates a `Vec<String>` plus a `String` per token, per term, per line, per restriction: ~225 tokenisations per ingredient line with all 11 kinds set, against the 18 `hay` clones the handoff names as its performance risk (Known Risk 9 points at the cheaper one). No correctness impact; comfortable at MVP library sizes. Fix: all but six terms are single tokens -- compare against the token slice directly and keep the phrase path for the rest.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-16-2026-08-29T1040-68c3.md
  **Status:** OPEN

---

- **`restrictionCopySamples` is a test fixture living in `lib/`, and the copy module imports a screen** (`lib/features/recipes/restriction_warnings.dart:37`) -- read only by `test/restriction_warnings_test.dart:16` yet shipped in the release binary; it is also the only production reference to the DTO's `linePosition` and `ruleVersion`. The same file imports `features/restrictions/restrictions_screen.dart` for `describeRestriction` alone, making a pure copy module depend on a screen. Fix: build the sample list in the test from the exported constants; lift `describeRestriction` into a non-screen module.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-16-2026-08-29T1040-68c3.md
  **Status:** OPEN

## orch/21 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-21-2026-08-29T1325-fc64.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-21-2026-08-29T1336-81e8.md

## orch/23 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-23-2026-08-29T1500-9be1.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-23-2026-08-29T1512-79d5.md

### LOW

- **`CorruptComponent` omits `note`, the field that makes a recipe row corrupt** (`rust/crates/kimatta-storage/src/lib.rs:1840`) -- the error carries kind/recipe_id/scale but not `note`, so the "recipe row carrying a note" shape renders a message describing a valid component and names no anomaly. Its own test at line 5014 is forced down to `..` for the same reason. Fix: add `note: Option<String>` to the variant and its format string; `ComponentRow` already holds it.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-23-2026-08-29T1512-79d5.md
  **Status:** OPEN

---

- **Automation lock refusal asserts a lock state it never read** (`rust/crates/kimatta-storage/src/lib.rs:1753`) -- `set_planned_meal_lock` returns `LockedPlannedMeal` for every `Automation` call before opening a transaction, so the message "planned meal X is locked against automation" claims the row exists and is locked when neither was checked. The refusal is correct; only the wording is. Fix: a distinct `AutomationMayNotLock(String)` variant, leaving `LockedPlannedMeal` to lines 1686 and 1796 which do read `locked` first.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-23-2026-08-29T1512-79d5.md
  **Status:** OPEN

---

## orch/25 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-25-2026-08-29T1621-1d84.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-25-2026-08-29T1633-ef03.md

### LOW

- **The absent-household pantry read contract is untested** (`rust/crates/kimatta-storage/src/lib.rs:1033`) -- `list_pantry_entries`' doc states an absent household reads as the catalog with nothing marked, but nothing pins it, while the opposite write-path behaviour is pinned; a later "symmetry" edit adding `require_household` would break the contract `MVP-015`/`MVP-023` are told to rely on, with a green suite. Fix: one test asserting `list_pantry_entries(conn, &hid("ghost"))` returns the catalog with `marked == false` throughout.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-25-2026-08-29T1633-ef03.md
  **Status:** OPEN

---

- **Unreachable `householdId == null` guard disables every pantry row silently** (`lib/features/pantry/pantry_screen.dart:126`) -- `_body` runs only in `pantryProvider`'s `AsyncData` arm and `PantryNotifier.build` awaits the household id first, so the null arm cannot fire; if the provider graph ever changed it would make every switch inert with no message. Fix: drop the null arm, or render an explicit message instead of silently disabling the controls.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-25-2026-08-29T1633-ef03.md
  **Status:** OPEN

---

## orch/27 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-27-2026-08-29T1752-c55a.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-27-2026-08-29T1804-63bd.md

### LOW

- **`CycleOverflow` names two different anchors for the same failure** (`rust/crates/food-domain/src/lib.rs:228`) -- `window_containing`'s `overflow` closure reports the *stored* anchor for its three early guards while the terminal `Self::new` reports the *computed* window anchor, so the error's `anchor` field means different things depending on which guard fired. The test matches only the variant. Fix: report the requested window's anchor consistently, or carry the offset in the variant.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-27-2026-08-29T1804-63bd.md
  **Status:** OPEN

---

- **The `maxTabPresses` rationale still describes the `/plan` placeholder** (`test/app_test.dart:781`) -- "two presses today, since Plan contributes only `Cover My Week`" predates `PlannerScreen`, whose app bar alone adds two `IconButton`s ahead of it, plus per-cell Add buttons and lock switches. The bound of 12 still holds, so the comment misleads without failing. Fix: restate the count against the planner's leading focusables, or drop the number and keep the bounded-traversal rationale.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-27-2026-08-29T1804-63bd.md
  **Status:** OPEN

---

- **Planner strings outside `planner_copy.dart` escape the safety-honesty pin** (`lib/features/planning/planner_screen.dart:451`) -- `'No recipes in your library yet.'`, the confirm-dialog body (:161) and the sheet headings live in the widget file, so `plannerCopySamples` -- documented as "Every string the planner renders" -- never feeds them to the invariant-10 assurance regex. Today's strings all pass; nothing stops a future one. Fix: move them into `planner_copy.dart` and sample them.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-27-2026-08-29T1804-63bd.md
  **Status:** OPEN

---

## orch/27 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-27-2026-08-29T1841-520a.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-27-2026-08-29T1900-494b.md

### LOW

- **Per-cell action controls carry no cell context for a screen reader** (`lib/features/planning/planner_screen.dart:298`) -- every cell renders an identically-labelled `Add`, and the component menu's `Move…`/`Scale…`/`Remove` repeat per component; the `Card` has a `ValueKey` but no `Semantics` container, so explore-by-touch announces "Add" seven times with no day or slot. `labeledTapTargetGuideline` passes, so no test sees it. Fix: wrap each cell in `Semantics(container: true, label: '$date · ${slotLabel(slot)}')` and sample the label from `planner_copy.dart`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-27-2026-08-29T1900-494b.md
  **Status:** OPEN

---

## orch/29 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-29-2026-08-29T2040-d2cb.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-29-2026-08-29T2047-558f.md

### LOW

- **Duplicate ids make `derive_shopping_list` order-dependent** (`rust/crates/food-domain/src/shopping.rs:363`) -- `ShoppingInput`'s doc claims "Order of every `Vec` is irrelevant to the output", but duplicate recipe ids resolve first-wins (`or_insert`) while duplicate identity refs resolve last-wins (`collect` into `BTreeMap`). Unreachable via `load_shopping_input`, which dedups both; `derive_shopping_list` is public and pure, so the invariant is overclaimed. Fix: weaken the doc to require unique ids/refs, or make both collections first-wins.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-29-2026-08-29T2047-558f.md
  **Status:** OPEN

---

## orch/29 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-29-2026-08-29T2117-8f5f.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-29-2026-08-29T2126-026f.md

### LOW

- **`derive_shopping_list` never filters `meals` by `from..=to`** (`rust/crates/food-domain/src/shopping.rs:416`) -- the doc claims `contribution_count` counts pairs "in range" and the list echoes `from`/`to`, but `collect_raws` iterates every meal; only `load_shopping_input`'s SQL `BETWEEN` bounds it. A hand-built `ShoppingInput` derives a list mislabelled with a range it does not respect. Fix: `retain` on the range in `collect_raws`, or state the precondition.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-29-2026-08-29T2126-026f.md
  **Status:** OPEN

---

- **Overflow fallback hand-rebuilds `Raw` field by field** (`rust/crates/food-domain/src/shopping.rs:494`) -- a ten-line struct literal copies all seven fields unchanged because `Raw` has no `Clone`, so a future field must be added in two places. Readability only; the compiler catches the omission. Fix: `#[derive(Clone)]` on `Raw`, then `separate_line(raw.clone(), ...)`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-29-2026-08-29T2126-026f.md
  **Status:** OPEN

---

- **Re-export rationale comment cites a name no item produces** (`rust/crates/food-domain/src/lib.rs:20`) -- the comment says a glob would expose `food_domain::derive`, but `shopping.rs` defines no `derive`; the function is `derive_shopping_list`, which a glob would re-export under that name. The stated reason for the explicit list describes nothing that could happen. Fix: state the real reason (a reviewable crate-root surface) or drop the comment.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-29-2026-08-29T2126-026f.md
  **Status:** OPEN

---

## orch/31 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-31-2026-08-29T2229-6d2a.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2237-2591.md

## orch/31 -- 2026-08-29

Source: rust/src/api/shopping.rs
Full review: /home/davidlinux/.claude/reviews/redteam-api-shopping-2026-08-29T2300-7c38.md

### LOW

- **`set_shopping_line_state` accepts two input fields it silently discards** (`rust/src/api/shopping.rs:70`) -- the command reuses its output DTO, so callers must supply `changed` and `checked_against`; neither is read (line 192 overwrites the token, `changed` is never referenced), so a client passing a stale token gets no error. Fix: take the four used fields as parameters, or document output-only on the type as well as the command.
  Full review: /home/davidlinux/.claude/reviews/redteam-api-shopping-2026-08-29T2300-7c38.md
  **Status:** OPEN

---

---

- **`delete_item_in` skips the household existence check its four siblings make** (`rust/src/api/shopping.rs:227`) -- `kimatta_storage::delete_shopping_manual_item` (`kimatta-storage/src/lib.rs:2600`) goes straight to the DELETE, so a nonexistent household id yields "no shopping item mi-1 in this household" — naming the item when the household is the wrong thing. Diagnostics only. Fix: add `require_household` before the DELETE.
  Full review: /home/davidlinux/.claude/reviews/redteam-api-shopping-2026-08-29T2300-7c38.md
  **Status:** OPEN

---

- **`hidden` and `restored` can both be true, and neither field is defined** (`rust/src/api/shopping.rs:171`) -- `clearing` tests only "no flag set" and the table CHECK is `checked + hidden + restored > 0`, so a row carrying both opposites stores fine; the DTO doc documents `changed`/`checked_against` but never says what `hidden` or `restored` mean, leaving rendering undefined. Fix: reject the pair with a typed `ShoppingError` at `set_state_in`, or document precedence on the DTO.
  Full review: /home/davidlinux/.claude/reviews/redteam-api-shopping-2026-08-29T2300-7c38.md
  **Status:** OPEN

---

- **Every non-clearing line-state write re-derives the whole window** (`rust/src/api/shopping.rs:175`) -- `set_state_in` runs a full `load_shopping_list` to read one token, so checking off N lines costs N derivations plus N transactions, with no batch form (unlike `set_pantry_marks`); the derive and write are separate transactions, so a concurrent planner edit binds a stale token. Fix: measure first; if it matters, batch the write or wrap derive+write in one transaction.
  Full review: /home/davidlinux/.claude/reviews/redteam-api-shopping-2026-08-29T2300-7c38.md
  **Status:** OPEN

## orch/31 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-31-2026-08-29T2323-144d.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2331-d2ca.md

### LOW

- **`describeCheckedAgainst` breaks its own verbatim-fallback contract** (`lib/features/shopping/shopping_copy.dart:82`) -- `_fraction` returns its input unchanged on an unsplittable body, so the `null` sentinel never fires for an `exact:`/`range:` prefix and garbage is relabelled with a unit ("was 1.5 cup") instead of falling back verbatim as the docstring at `:52` promises. Fix: make `_fraction` return `String?` and propagate `null`; add an `exact:1.5|known:cup` copy test.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2331-d2ca.md
  **Status:** OPEN

---

- **"Start over" confirmation under-counts what it clears** (`lib/features/shopping/shopping_screen.dart:197`) -- `resetBody` counts the filtered `line_states` (orphans dropped at `rust/src/api/shopping.rs:124`), but `reset_shopping_list` deletes every row for the window (`rust/crates/kimatta-storage/src/lib.rs:2628`), so the dialog states a smaller number than the action removes. Fix: add `orphanedLineStateCount` to the first argument, or reword to describe what the user can see.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2331-d2ca.md
  **Status:** OPEN

---

- **"Add anyway" silently discards an existing check** (`lib/features/shopping/shopping_screen.dart:387`) -- the Already-have row sends `checked: false` unconditionally, and a line reaches that section still carrying `checked: true` (the key is status-independent), so restoring a line the user had checked and pushed to the pantry drops the check and its token invisibly. Fix: pass `view.stateFor(line.key)?.checked ?? false` as the sibling "Put back" already does for `restored`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2331-d2ca.md
  **Status:** OPEN

---

- **`_writing` survives a cycle change and a write can outlive its notifier** (`lib/features/shopping/shopping_screen.dart:106`) -- the set is never cleared when `_offset` changes and line keys carry no window bound, so the same key renders disabled in the next cycle; `shoppingProvider` is `autoDispose.family`, so the in-flight write's `_republish` then throws on a disposed notifier and reports a failure for a write that succeeded. Fix: clear `_writing` on offset change and no-op `_republish` when unmounted.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2331-d2ca.md
  **Status:** OPEN

---

- **New-item `_write` key is dead and "Add item" is never disabled** (`lib/features/shopping/shopping_screen.dart:235`) -- a save keyed `existing?.id ?? ''` is never consulted (`_manualRow` guards on `item.id`, the Add button has no guard), and Rust mints a fresh id per blank-id save, so a double submit creates two identical items. Fix: guard the Add button on `_writing.contains('')`, or drop the empty key and disable the button for the save.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2331-d2ca.md
  **Status:** OPEN

## orch/33 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-33-2026-09-02T1146-243b.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md

### LOW

- **`PLAN_INFEASIBLE` copy attributes a scoring outcome to "the hard checks"** (`rust/crates/food-domain/src/planner/coverage.rs:54`) -- the sentence says no leftovers option "passed the hard checks", but `only_fallbacks` is `!any(source.is_meal())` (`mod.rs:157`) and `is_meal()` excludes `Leftovers` (`candidates.rs:50`). A sourceless leftovers candidate passes Tier 0 and is still reported as having failed it, with an empty `rejections` list to explain. Fix: reword to the actual condition, or split sourceless leftovers into its own code and sentence.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **Split doc comment across two bridge tests** (`test/bridge_native_test.dart:539`) -- the MVP-016 shopping test's comment head sits above `test('a cover cycle outcome crosses the bridge')` at :543 while its tail sits at :599 above the MVP-016 test at :601. Comments only; analyze, format and all 287 tests pass. Self-disclosed in the MVP-023 handoff (Reviewer Note 1); left unfixed because the session lost edit permission. Fix: delete the two orphaned lines above :543 and restore the full MVP-016 comment above :601.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **`decode_parameters` silently discards malformed lines while its siblings report corruption** (`rust/crates/kimatta-storage/src/controller.rs:56`) -- `filter_map(|line| line.split_once('\t'))` drops any tab-less stored line, though `list_policies`' rustdoc (:100) promises a row that could not be a policy is reported and the same function raises `CorruptLedgerEntry` for a corrupt `source` token (:124). Round-trip is lossless for well-formed data. A corrupted parameters blob surfaces as a generic `UNKNOWN_POLICY_TYPE` instead of naming the row. Fix: return `Result` and raise `CorruptLedgerEntry { column: "parameters" }` on a tab-less non-empty line.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **`FoodPolicies::from_policies` non-`food` branch is unreachable and mislabels if reached** (`rust/crates/food-domain/src/planner/snapshot.rs:58`) -- the only production caller passes `list_policies(&tx, household, "food")` (`controller.rs:357`), whose SQL filters on domain, so the branch is defensive bloat. If a future caller passes an unfiltered list, every foreign-domain policy adds a spurious `UNKNOWN_POLICY_TYPE` issue to the Cover My Week assessment. Fix: drop the branch, or filter by domain inside the function and ignore foreign domains silently.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **`PREP_FEASIBLE_UNKNOWN` duplicates `NO_PREP_TIME_ESTIMATES`** (`rust/crates/food-domain/src/planner/score.rs:16`) -- score.rs:142 raises it on exactly the negation of `coverage.rs:120`'s `schedule_fit_known`, which raises `NO_PREP_TIME_ESTIMATES`. Only the coverage code has an `ISSUE_TEXT` entry and claim-blocking wiring; the score-side code lands in `assessment.assumptions` and renders as `""`. A later card adding copy for one name produces duplicate or missing text. Fix: emit the single coverage code from both sites, or document at :16 that this code is score-local and never user-facing.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **`PlanningSnapshot` has public fields and no validating constructor; `length_days: 0` panics** (`rust/crates/food-domain/src/planner/mod.rs:213`) -- `let from = snapshot.dates()[0];` and `horizon()` at :133 index an empty `Vec` when `length_days == 0`; `snapshot.rs:148` also carries `expect("loader proved the cycle fits")`. Both rest on a loader invariant the public type does not enforce, and the type is re-exported through `kimatta-storage`. MVP-025 is about to hand-build fixtures. Fix: add a checked constructor validating `MIN_CYCLE_DAYS..=MAX_CYCLE_DAYS`, or make the fields `pub(crate)` with accessors.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **`load_planning_snapshot` issues N+1 queries and loads full recipe records for seven fields** (`rust/crates/kimatta-storage/src/controller.rs:359`) -- `list_recipes(Active)` then `load_recipe` per recipe, each materialising instructions, provenance and every line only to build a `RecipeCandidateInfo`. A 200-recipe household runs 201 queries per `cover_cycle` call. No wrong output; it adds fixed cost to an already-expensive operation. Fix: one projecting join, or a `list_recipe_candidate_info` in the storage crate; best paired with the scoring hot-loop fix.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **`KimattaError::Planner` is a dead variant already frozen into the FFI contract** (`rust/src/api/error.rs:23`) -- nothing constructs it: grepping `rust/src/` and `lib/features/` returns the definition plus three machine-generated `frb_generated.rs` arms (:1764, :3382, :4460) and nothing else, and the `From<ApplicationError>` impl at :30-36 is exhaustive over a one-variant enum with no catch-all. Its own doc concedes it is "reserved for planner-specific failures the application layer may grow". It generated ~60 lines of freezed Dart and an unreachable arm in `describeFailure` (`lib/features/household/household_screen.dart:43-45`). Fix: delete the variant and its Dart arm and regenerate -- cheap now, a breaking change to a Dart surface once MVP-024 ships against it.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** RESOLVED 2026-09-02 (MVP-024) -- already fixed before this card: `rust/src/api/error.rs` carries no `Planner` variant (verified by grep; variants are InvalidPath/NotOpen/Storage/Planning/Recipe/Restriction/PlannedMeal/Shopping) and `describeFailure` has no such arm.

---

- **`slot_is_resolved` is dead, and the resolution guard is absent from `apply_plan_and_record`** (`rust/crates/kimatta-storage/src/controller.rs:422`) -- never called (the only other grep hit is an unrelated test *name* at `food-domain/src/planner/tests.rs:467`), and it duplicates `candidates::is_resolved` (`candidates.rs:131`), which is used. So the applier enforces only the lock half: an unlocked slot the household set to `Open` is deleted and rewritten as `Automation` with no guard. Unreachable today because `tier0::filter` keeps only the `ExistingPlan` candidate for a resolved slot (`tier0.rs:68-70`), so the `continue` at `controller.rs:300` fires -- but that protection lives in `food-domain` while `apply_plan_and_record` is `pub`. Fix: call it in the apply loop beside the lock check, or delete it and note that resolution is enforced upstream.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **A corrupt `policy` row is reported as `CorruptLedgerEntry`, naming the wrong table** (`rust/crates/kimatta-storage/src/controller.rs:124`) -- `list_policies` raises that variant for a `policy` row with an unparseable `source`, and its message is "ledger entry {entry} has {column} {value:?}, which is not a stored token" (`lib.rs:158-163`). An operator reads `ledger entry p1 has source "bogus"` for a row in `policy`, inspects `controller_ledger`, and finds nothing wrong. The corruption is correctly refused rather than coerced; only the message misleads. Fix: a shared `CorruptRow { table, id, column, value }` variant, or a second variant for policies.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **`days()`'s doc says the history window "simply shortens"; the caller drops it entirely** (`rust/crates/kimatta-storage/src/controller.rs:29`) -- the match at :397-400 takes `(Some(from), Some(to))` or `Vec::new()`, so a `None` bound yields an *empty* history rather than a shorter one. Reachable only within 14 days of jiff's minimum civil date, so the practical impact is nil and the defect is that the comment states the opposite of the behaviour. (Also: `days()`'s `n.abs()` would panic on `i64::MIN`, but it is only ever called with the literals `-14` and `-1`.) Fix: clamp the lower bound to the calendar minimum so the window genuinely shortens, or correct the comment.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

- **A short `ids` slice is reported as a data error rather than a programmer error** (`rust/crates/kimatta-storage/src/controller.rs:305`) -- passing fewer `ids` than `proposed` slots returns `NoSuchPlannedMeal { meal: "proposed slot {index}" }`, a synthetic string in an id field, so the caller cannot tell "I passed a short slice" from "a stored occurrence vanished". Rollback is correct. Defensive only: `kimatta-application/src/lib.rs:100-105` mints exactly one id per proposed slot. Fix: an `IdCountMismatch { expected, got }` variant, or a debug assertion at the top of the function.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

---

## orch/33 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-33-2026-09-02T1304-e11f.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1317-9799.md

### LOW

- **`list_recipe_candidate_info`'s ref arm cites the wrong table's CHECK** (`rust/crates/kimatta-storage/src/controller.rs:386`) -- the rustdoc points at `lib.rs:379`, which is `pantry_item`'s exactly-one CHECK; `recipe_ingredient_line`'s own is `lib.rs:264` and is at-most-one, permitting both-NULL. Match arms are correct; the comment invites deleting the `(None, None)` arm the real constraint needs. Fix: re-point the citation at `lib.rs:264` and say at-most-one.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1317-9799.md
  **Status:** OPEN

---

- **`PlanningSnapshot.existing`/`.history` documented "by date then slot"; loader gives date then id** (`rust/crates/food-domain/src/planner/snapshot.rs:138`) -- `list_planned_meals` is `ORDER BY date, id` (`kimatta-storage/src/lib.rs:2387`), so within a date the order is planned-meal-id (UUID for automation rows), not slot. Hash stays deterministic because ids are in `meal_text`; the risk is a later change trusting the documented order and dropping the id, which would make `StalePlan` fire spuriously. Fix: correct the field docs, or add `slot` to the ORDER BY.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1317-9799.md
  **Status:** OPEN

---

- **`commitment_horizon_days` is unclamped under a comment saying all three knobs are clamped** (`rust/src/api/planner.rs:354`) -- `beam_width` and `candidates_per_slot` get `.clamp(1, 16)`; the third is a bare `unwrap_or`. No overflow (it only feeds an `i64` compare in `score.rs:103`), but `u32::MAX` from Dart makes every date near-term, applying `NEAR_TERM_CHURN` (-3) uniformly and freezing the existing plan. Untested. Fix: clamp to the 31-day cycle maximum, or scope the comment to the two fields it covers.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1317-9799.md
  **Status:** OPEN

---

- **`SearchTrace.states_scored` omits the per-slot ranking pass** (`rust/crates/food-domain/src/planner/beam.rs:62`) -- `search` calls `score_plan` once per feasible candidate to rank before truncating to K, and counts only the extension scorings. On a 7-locked-dinner / 50-recipe fixture that is 370 scorings reported as zero. It is the only cost signal in the ledger payload and MVP-025's benchmark baseline. Fix: fold `Σ|feasible|` in, or add a separate `candidates_ranked` counter.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1317-9799.md
  **Status:** OPEN

## orch/33 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-33-2026-09-02T1354-52de.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1401-4b38.md

### LOW

- **Bridge keeps `LOCK_CONFLICT` in `reason_codes` while dropping its rejections** (`rust/src/api/planner.rs:321`) -- the filter strips every `LOCK_CONFLICT` row from `PlanningResultDto.rejections`, but `planner/mod.rs:246` still chains the code into `assessment.reason_codes` and nothing filters it at the bridge. A Dart consumer indexing `rejections` by code finds zero rows, and `issue_text("LOCK_CONFLICT")` is `""` so there is no copy either. MVP-024 is the first UI that could trip on it. Fix: filter the code out of the DTO's `reason_codes`, or document on `PlanningResultDto.rejections` that `reason_codes` is a superset.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1401-4b38.md
  **Status:** OPEN -- re-deferred 2026-09-02 (MVP-024), not mooted: the Rust-layer contract oddity remains (the field's contents are unchanged and any other consumer still sees it), but MVP-024's UI never reads `result.reason_codes` (a documented provider rule in `lib/features/planning/cover_provider.dart`), so no current consumer misbehaves. A Rust-side filter belongs with whichever later card first renders reason codes; re-scoped to that card.

## orch/33 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-33-2026-09-02T1446-32d3.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1502-ac00.md

### LOW

- **`assess` runs the leftovers-source scan on every slot** (`rust/crates/food-domain/src/planner/mod.rs:364`) -- `leftovers_sourced_from_stored` is called unconditionally per enabled slot, but `slot_coverage` reads the flag only when the candidate `is_leftovers()` (`coverage.rs:215`). Each call re-scans `existing ∪ history` and rebuilds a `RecipeView` per component, linear-scanning `snapshot.recipes` and deep-cloning title, line names and refs -- ~735 discarded view constructions on a 21-slot / 50-recipe cycle. `score.rs:65-70` hoists exactly this cost for exactly this reason. No wrong answer. Fix: gate the call on the stored occurrence holding a `Leftovers` component, or memoise per date.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1502-ac00.md
  **Status:** OPEN

## orch/33 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-33-2026-09-02T1536-838b.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1548-e7cc.md

### LOW

- **Facts-free starter view's `title` is a slug the veto and preference matchers read as a dish name** (`rust/crates/food-domain/src/planner/candidates.rs:225`) -- the unknown-slug arm sets `title: slug`, and `tokens` splits on non-alphanumerics, so `chicken-curry` matches a `Like: "curry"` preference and scores `MEMBER_LIKE +1` at the tier that sets the per-member floor. The archived-recipe branch it mirrors uses an opaque recipe id. Fix: leave `title` empty in that arm, or split display from matched text on `RecipeView`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1548-e7cc.md
  **Status:** OPEN

---

- **`leftovers_sourced_from_stored` reads `history` with no `previous < anchor` guard** (`rust/crates/food-domain/src/planner/candidates.rs:258`) -- its counterpart `score::leftover_source` guards the same lookup at `score.rs:127`. The loader keeps `existing` and `history` date-disjoint (`controller.rs:525-529`), but `PlanningSnapshot`'s fields are public and every planner test builds one by hand, so a history row on or after the anchor silently re-opens the assess/search divergence this card closed. Fix: mirror the guard.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1548-e7cc.md
  **Status:** OPEN

## orch/35 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-35-2026-09-02T1722-7e0f.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-35-2026-09-02T1736-f8f4.md

### LOW

- **Dead statements written to satisfy the compiler, not to assert** (`rust/crates/food-domain/src/planner/invariant_tests.rs:166`) -- `history_ids.clear()` exists only to justify the `mut` binding; `let _ = lines;` (line 648) discards a counter `assert_well_formed` computes and never asserts; `fixtures/mod.rs:798` re-assigns `pantry_marked = vec![]` that `base()` already set at `mod.rs:291`. Reads as configuration, is tautology. Fix: drop the `mut`+`clear()`, assert a floor on `lines` or delete it, delete the redundant assignment.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-35-2026-09-02T1736-f8f4.md
  **Status:** RESOLVED 2026-09-02 -- all three deleted; the `lines` counter went rather than gaining a floor (the caller's `lines_seen > 0` already covers it), and `sparse_pantry`'s pinned hash was unchanged by the assignment's removal, confirming it was a tautology.

---

- **Exploratory B×K grid skips `multiple_strong_dislikes`, the only fixture with a strictly-better cell** (`rust/crates/food-domain/examples/beam_width.rs:19`) -- `SEARCH_HEAVY` is an undocumented four-name list that includes 3-recipe `leftovers_fallback_heavy` but excludes the 8-recipe fixture that produced the recorded headroom cell (`docs/ROADMAP.md:322`), so no intermediate-B data exists where the evidence says shipped params leave quality on the table. Non-blocking: `beam.rs:85-108` is monotone in B and K, so `(64, cap)` upper-bounds every cell. Fix: derive the list from `feasible_cap`, or add the fixture and record the monotonicity argument.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-35-2026-09-02T1736-f8f4.md
  **Status:** RESOLVED 2026-09-02 -- fixture added (`SEARCH_HEAVY` now 5), selection criterion recorded on the const and the monotonicity argument in the module doc; benchmark re-run resolved the open question — the headroom is a beam-width effect reached at B>=16 for every swept K (and at B=8/K=2), verdict still SURVIVES.

---

- **ROADMAP evidence row overstates invariant 4's parameter coverage** (`docs/ROADMAP.md:322`) -- the AC-2 cell claims invariants "1-4 ... quantified over all 11 fixtures x (B,K) in {(1,1),(8,12),(64,64)}", but `invariant_tests.rs:570-577` runs invariant 4 over `[(1,1), default]` only, as its own doc comment states. The row is what a verifier reads to grant PASS. Materially small: the subset check is parameter-independent, only the corollary is affected. Fix: narrow the parenthetical to invariants 1-3, or annotate invariant 4's two cells.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-35-2026-09-02T1736-f8f4.md
  **Status:** RESOLVED 2026-09-02 -- the AC-2 cell now reads "invariants 1-3 ... x (B,K)", with invariant 4 recorded separately as parameter-independent subset checks plus a corollary at (1,1) and (8,12); appended rather than narrowed, so invariant 4 keeps its place in the evidence sentence.

---

- **`by_name` rebuilds every fixture per call; the skipped-restrictions variant is never planned** (`rust/crates/food-domain/src/planner/fixtures/mod.rs:64`) -- `by_name` calls `all()` (line 65), so `fixture_shapes_hold`'s eleven lookups build 121 fixtures including 48-recipe `high_variety_household`. Separately `restrictions_skipped_variant` (line 96) has one caller (`invariant_tests.rs:286`) that only asserts its own shape; no invariant and not `beam_width.rs` ever plans it, so `fixtures/README.md:22`'s "tests, bench, §22" reuse cell overstates it. Fix: match-then-build or `OnceLock` in `by_name`; run one invariant over the variant or narrow the README cell.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-35-2026-09-02T1736-f8f4.md
  **Status:** RESOLVED 2026-09-02 -- `by_name` builds once via `OnceLock` (chosen over match-then-build, which would duplicate the eleven-name list; `all()` stays uncached so `fixtures_are_deterministic` still compares two independent builds), and the README's reuse and shape cells for that row now say the skipped variant is shape-asserted only.

## orch/37 -- 2026-09-02

Source: MVP-024 implementation (execute-plan step 12 dispositions).

### LOW

- **The declined-apply distinction never surfaces in any UI** -- `ACTION_DECLINED` is written to the ledger (`kimatta-application/src/lib.rs`, pinned by `a_declined_apply_is_distinguishable_from_a_preview`), and MVP-024's cover screen re-renders honestly from the returned `applied: false` outcome, but no screen tells the user "your apply was refused" in words distinct from an ordinary needs-attention preview. Residual of the resolved `KNOWN_ISSUES.md` `selected_action` entry, recorded as new rather than re-deferred. Fix: distinct copy on the `applied: false` path of an accept, with whichever card next touches cover-screen copy.
  **Status:** OPEN

- **Cover My Week plans the whole window at offset 0, past-dated empty slots included** (decision note, not a defect) -- MVP-024 kept MVP-023's tested whole-window behavior (`changed_slots == 7` in the native cover test; matches the MVP-013 grid, which renders the full window). Observed consequence: invoking Cover mid-cycle fills empty slots dated before `today` in the current window. Recorded as the closure of MVP-013's open note on the question; revisit only if a real household complains about backfilled past days.
  **Status:** OPEN

## orch/37 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-37-2026-09-02T1936-1e59.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T1949-115e.md

### LOW

- **`acceptedCopy` is defined and sampled but never rendered** (`lib/features/planning/cover_copy.dart:56`) -- `'Plan written.'` exists only at its definition and in `test/cover_copy_test.dart:25`'s sample list; `cover_screen.dart`'s accepted path (line 81) renders the shopping link alone. Being in the sample list makes the dead constant look exercised. Fix: render it beside the shopping link -- the accepted state currently has no textual confirmation -- or delete it and its sample.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T1949-115e.md
  **Status:** OPEN

---

- **Unguarded `setState` after an await boundary in the cover screen** (`lib/features/planning/cover_screen.dart:302`) -- `_accept` (:302) and `_decide` (:314) open with an unguarded `setState(() => _busy = true)` while every other `setState` in the file is `mounted`-guarded (:305, :309, :320). Both are reached after an await that can outlive the route (`_openSwapPicker` :399, `_confirmVeto` :425), so a decision returning into a disposed state throws. Fix: `if (!mounted) return;` before the opening `setState` in both.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T1949-115e.md
  **Status:** OPEN

---

- **The cover screen's error branch has no widget test** (`lib/features/planning/cover_screen.dart:44`) -- the `AsyncError` arm renders `describeFailure` plus a "Try again" that invalidates the provider; none of the eight cover widget tests (`test/app_test.dart:1183-1391`) put the provider in an error state. The planner's equivalent branch does have one (`test/app_test.dart:5597`), so the gap is against the file's own convention. Fix: mirror that test -- a throwing `cover` seam, assert the `describeFailure` text, tap "Try again" and assert the seam is re-invoked.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T1949-115e.md
  **Status:** OPEN

## orch/37 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-37-2026-09-02T2033-1a2c.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T2041-c699.md

### LOW

- **`?offset=` outside i32 truncates silently past the ±520 bound** (`lib/app/router.dart:78`) -- `int.tryParse` yields a 64-bit int that reaches `sse_encode_i_32` -> `putInt32`, which keeps the low 32 bits without throwing, so `?offset=4294967297` narrows to `1`, passes `MAX_OFFSET_CYCLES`, and previews a window the URL did not ask for. Fix: clamp the parsed offset to `[-520, 520]` in the route builder.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T2041-c699.md
  **Status:** RESOLVED 2026-09-02 -- `coverOffset` in `lib/app/router.dart` saturates the parsed value to the i32 range before it crosses, so an out-of-range deep link can no longer narrow into an unrelated in-range window. Deliberately *not* the clamp to `[-520, 520]` this entry proposed: clamping to the bound would silently move the user to week 520, whereas saturating hands the intent to the bound, which now refuses in prose ("that week is too far away to plan"). Pinned by the `coverOffset` group in `test/app_test.dart`, including a case asserting `'1000'` is *not* clamped.

---

- **`RESTRICTIONS_NOT_CONFIGURED` is rendered twice on the same screen** (`lib/features/planning/cover_screen.dart:110`) -- the reviewed-restrictions card and `assumptionLines` (:73) both fire on that code, so the household reads the same fact as an actionable card and again inside "What we couldn't check". Fix: add the code to `assumptionCopy`'s deliberately-unmapped set now that the reviewed row owns the message, or filter it from `assumptionLines` while the row shows.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T2041-c699.md
  **Status:** OPEN

---

- **`coverCopySamples` has no completeness guard** (`test/cover_copy_test.dart:7`) -- the list is hand-maintained and the two invariant-19 regexes iterate only it; `isNotEmpty` is the sole structural assertion, so a constant added to `cover_copy.dart` and not to the list escapes both the safety and the reversal check silently. Fix: assert a count with a comment naming why, or derive the sample list from one exported map the widgets also read.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T2041-c699.md
  **Status:** OPEN 2026-09-02, narrowed -- the count guard is now asserted in both suites (`coverCopySamples.length == 31`, `plannerCopySamples.length == 36`), which pins each list against erosion. It does **not** close the stated failure scenario: a new constant added to `cover_copy.dart` and never sampled leaves the count unchanged, and Dart has no reflection over a library's top-level constants. Only the second direction -- deriving the samples from one exported map the widgets also read -- catches that, and it is a production-copy restructure across `cover_copy.dart`, `cover_screen.dart`, `planner_copy.dart` and `planner_screen.dart`. That restructure is what remains open here.

## orch/37 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-37-2026-09-02T2117-c7db.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T2127-eb06.md

### LOW

- **The Cover screen's decision-failure snackbar is untested** (`lib/features/planning/cover_screen.dart:334`) -- no test makes the `decide:` hook throw, so `_decide`/`_accept`'s `catch -> _report` and the `_busy` release in `finally` are never exercised, on a screen whose three refusals (`BlankVetoSubject`, `DecisionOutsideWindow`, `LockedPlannedMeal`) are all user-visible. Fix: one widget test with a throwing `decide:` hook asserting `find.byType(SnackBar)` carries `describeFailure`'s prose, as the pantry and household screens already do.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T2127-eb06.md
  **Status:** RESOLVED 2026-09-02 -- `a refused decision surfaces as a snackbar and frees the screen` (`test/app_test.dart`) throws from the `decide:` hook, asserts the snackbar carries `describeFailure`'s prose, and asserts the tile's Swap is re-enabled afterwards (the `finally` half). Two of the three refusals this entry names have since changed identity in the same pass: `BlankVetoSubject` is now `UnmatchableVetoSubject`, and `LockedPlannedMeal` no longer reaches this screen -- the locked swap is refused as `ApplicationError::SwapOntoLockedSlot` and mapped to prose.

---

- **One user-facing sentence lives in the widget, outside the sampled copy surface** (`lib/features/planning/cover_screen.dart:381`) -- `'Your recipe library could not be read.'` is prose, not a control label, yet it is neither in `cover_copy.dart` nor in `coverCopySamples`, so none of the three invariant-19 whole-surface regexes ever sees it and the count guard cannot detect a string that was never a constant. Distinct from the `coverCopySamples` entry above, which is scoped to unsampled constants *in* `cover_copy.dart`. Fix: move it to `cover_copy.dart` as a named constant and add it to `coverCopySamples`.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T2127-eb06.md
  **Status:** RESOLVED 2026-09-02 -- the sentence is now `swapLibraryUnavailableCopy` in `cover_copy.dart` and sampled, so all three whole-surface regexes see it. `questionLockedCopy` was added in the same pass and sampled with it; the count guard moved 29 -> 31.

## orch/39 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-39-2026-09-02T2310-dff6.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-02T2320-5b75.md

### LOW

- **`commit_swap` destroys the previous `.pre-restore` before anything replaces it** (`rust/src/api/health.rs:118`) -- the one-generation cleanup runs before `rename(db_path, pre)` at line 125, so a restore that fails anywhere up to line 140 deletes the earlier backup generation without creating a new one; on Unix the rename would have replaced it atomically anyway. Fix: drop `remove_file(pre)` and let the rename replace it (keep the explicit `pre_journal` removal), or move both removals after the rename succeeds.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-02T2320-5b75.md
  **Status:** RESOLVED 2026-09-03 -- `remove_file(pre)` is gone and the rename replaces `pre` atomically; the `pre_journal` removal and the journal move now sit inside the `db_path`-exists branch, with an `else` arm that removes a journal orphaned by an absent database rather than mispairing it with the kept generation.

---

- **Export failures bypass corrupt-error typing** (`rust/crates/kimatta-storage/src/lib.rs:610`) -- `VACUUM INTO` (line 610) and `schema_version` (line 581) use a bare `?` rather than `typed_sqlite`, so page-level damage found during an export surfaces as `KimattaError::Storage` with the raw SQLite string instead of the honest `Corrupt` copy `household_screen.dart` supplies. Fix: `.map_err(typed_sqlite)` on both, matching `open` and `validate_export` in the same file.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-02T2320-5b75.md
  **Status:** RESOLVED 2026-09-03 -- both sites now route through `typed_sqlite`, pinned by `an_export_of_a_damaged_database_is_typed_as_corrupt`.

---

- **The live database path is constructed independently in two places** (`lib/features/settings/backup_provider.dart:29`) -- `BackupActions._dbPath()` and `healthReportProvider` (`lib/features/settings/health_provider.dart:13`) each build `'${dir.path}${Platform.pathSeparator}kimatta.db'`; if one is ever changed, restore/start-fresh act on a file the app never opens and report success while nothing visible changes. Fix: one exported `localDatabasePath()` called by both.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-02T2320-5b75.md
  **Status:** RESOLVED 2026-09-03 -- `localDatabasePath()` in `health_provider.dart` is the single spelling; `healthReportProvider`, `restore` and `startFresh` all call it and `BackupActions._dbPath()` is gone.

## orch/39 -- 2026-09-03

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-39-2026-09-03T0005-e036.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-03T0012-89d3.md

### LOW

- **Post-verify staging cleanup is swallowed and covers only `-journal`** (`rust/src/api/health.rs:99`) -- `let _ = remove_file("{staging}-journal")` is the mirror of the strict three-suffix loop at line 79, but swallows failure and omits `-wal`/`-shm`; `commit_swap`'s assertion checks `{db_path}-wal`, not `{staging}-wal`, so nothing downstream notices. Fix: reuse the `ignore_not_found` + `?` loop shape here.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-03T0012-89d3.md
  **Status:** RESOLVED 2026-09-03 -- both staging clears are the same strict loop over one `const SIDECARS` list, which every clear/move/put-back site in `health.rs` now drives off; pinned by `a_restore_leaves_no_staging_artefact_behind`.

---

- **Failed `pre_journal` removal mispairs the kept generation** (`rust/src/api/health.rs:136`) -- the `exists` branch renames `db_path` onto `pre` before removing `{pre}-journal`; a non-`NotFound` failure there leaves generation N's database beside generation N-1's journal, which `recover_original`'s unconditional put-back (line 183) then moves next to the restored database (R0). Fix: remove `{pre}-journal` before the rename, so a failure aborts with nothing moved.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-03T0012-89d3.md
  **Status:** RESOLVED 2026-09-03 -- not by that reorder, which plan review showed destroys the kept generation's journal when the rename then fails. Instead each `{pre}` sidecar is replaced-or-removed *after* the rename (its database is gone by then), and `recover_original` puts back only the sidecars this restore moved, so a foreign journal cannot reach the restored original; pinned by `a_foreign_pre_journal_is_not_carried_back_to_the_original`.

## integration -- 2026-09-03

Full review: /home/davidlinux/.claude/reviews/redteam-integration-ratification-2026-09-03T1119-b77c.md

### LOW

- **Bridge functions with no production caller** (`rust/src/api/recipe.rs:185`, `rust/src/api/planned_meals.rs:50`) -- `add_custom_ingredient`, `list_custom_ingredients`, and `load_planned_meal` are called only by `test/bridge_native_test.dart`; no `lib/` UI wired them. Fix: wire them or annotate as intentionally test-only like `derive_shopping_list`'s documented precedent.
  Full review: /home/davidlinux/.claude/reviews/redteam-integration-ratification-2026-09-03T1119-b77c.md
  **Status:** OPEN

- **`#[frb(sync)]` applied inconsistently to constant functions** (`rust/src/api/restrictions.rs:20`) -- `core_version` is sync but the constant-vocabulary `known_restriction_kinds`/`known_unit_kinds`/`known_meal_component_kinds` are async Futures. Harmless; callers await them fine. Fix: annotate the `known_*` trio sync at the next codegen regeneration.
  Full review: /home/davidlinux/.claude/reviews/redteam-integration-ratification-2026-09-03T1119-b77c.md
  **Status:** OPEN

- **Fresh-vs-migrated schema parity never asserted directly** (`rust/crates/kimatta-storage/src/lib.rs:3038`) -- `assert_db_equivalent` is called only by the export tests; parity holds by construction (single migration path) but nothing locks it against a future hand-written fast path. Fix: one test comparing a fresh `:memory:` DB vs a v1-origin migrated DB via the helper's schema half.
  Full review: /home/davidlinux/.claude/reviews/redteam-integration-ratification-2026-09-03T1119-b77c.md
  **Status:** OPEN

- **No `foreign_key_check` backstop around migration** (`rust/crates/kimatta-storage/src/lib.rs:541`) -- `open()` disables FKs, migrates, re-enables, never runs `PRAGMA foreign_key_check`; safe while no migration rebuilds tables. Fix: per-migration `.foreign_key_check()` (rusqlite_migration supports it) or one check after a version advance.
  Full review: /home/davidlinux/.claude/reviews/redteam-integration-ratification-2026-09-03T1119-b77c.md
  **Status:** OPEN

- **`shopping_line_state.from_date`/`to_date` lack the civil-date GLOB CHECK used everywhere else** (`rust/crates/kimatta-storage/src/lib.rs:423`) -- plain `TEXT NOT NULL` while `planning_cycle.anchor_date` and `planned_meal.date` carry the GLOB; a malformed window written raw silently orphans overlay rows, and CHECKs are immutable once shipped. Fix: add the GLOB CHECK in the next storage migration.
  Full review: /home/davidlinux/.claude/reviews/redteam-integration-ratification-2026-09-03T1119-b77c.md
  **Status:** OPEN

---

## integration -- 2026-09-03

Source: lib/features/recipes/recipe_form_screen.dart
Full review: /home/davidlinux/.claude/reviews/redteam-recipe-form-screen-2026-09-03T1504-04b9.md

### LOW

- **Servings and prep-time rejection copy shows the user `4294967295`** (`lib/features/recipes/recipe_form_screen.dart:149`) -- `maxQuantity` (`0xFFFFFFFF`, a bridge-encoder artifact) is interpolated into inline error text, so typing `0` in Servings reads "must be a whole number from 1 to 4294967295"; `app_test.dart:3950` and `:4118` pin the literal string. Fix: drop the upper bound from the copy, keeping the `maxQuantity` check as a silent guard, and update the two assertions.
  Full review: /home/davidlinux/.claude/reviews/redteam-recipe-form-screen-2026-09-03T1504-04b9.md
  **Status:** OPEN

---

## integration -- 2026-09-03

Source: lib/features/recipes/recipe_form_screen.dart
Full review: /home/davidlinux/.claude/reviews/redteam-recipe-form-screen-2026-09-03T1540-7688.md

### LOW

- **Household-mismatch refusal is a dead end with Save left enabled** (`lib/features/recipes/recipe_form_screen.dart:297`) -- the guard's own docstring says "there is no field the user could correct to resolve it", yet the refusal is a transient `SnackBar` and every control stays enabled, so retrying reads as worth doing; stale inline errors also survive the early `return`. Sibling unrecoverable states render in place (`:365`, `:354`). Fix: latch the mismatch into state and render it where 'Recipe not found.' renders, Save disabled.
  Full review: /home/davidlinux/.claude/reviews/redteam-recipe-form-screen-2026-09-03T1540-7688.md
  **Status:** OPEN

---

## integration -- 2026-09-03

Source: lib/features/recipes/recipe_form_screen.dart
Full review: /home/davidlinux/.local/state/claude-orch/5f7efeea1579/0276579f2864/wt/integration/.orch/redteam-recipe-form-screen-2026-09-03T1540-7688.md

### LOW

- **Renaming an ingredient line drops its catalog identity for good, with nothing to re-link it** (`lib/features/recipes/recipe_form_screen.dart:86`) -- `emittedIngredient` clears the seeded `IngredientRefDto` whenever the row's `name` no longer matches what it was loaded with (owner decision 2026-09-03: keeping it is the unsafe direction, since `Identities::status` at `shopping.rs:411` matches on the ref alone, so a renamed line would inherit a pantry mark and vanish from the shopping list). The cleared line is correct but coarse: it goes back to `Unresolved`, so it lists separately, uncategorised and always `Needed` -- over-listing, never silent omission. Nothing re-matches it afterwards; the ref is gone until the user re-picks. MVP-009/011 own explicit ingredient mapping and may re-link renamed lines under that scope, at which point this degrades from permanent to transient. Fix: none until then -- deliberate conservative behaviour, tracked so the coarseness is not mistaken for a bug.
  Full review: /home/davidlinux/.local/state/claude-orch/5f7efeea1579/0276579f2864/wt/integration/.orch/redteam-recipe-form-screen-2026-09-03T1540-7688.md
  **Status:** OPEN

---
## master -- 2026-09-06

Source: /home/davidlinux/.claude/reviews/impl-handoff-mvp032-owner-recipes-2026-09-06T1644-18f1.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp032-owner-recipes-2026-09-06T1653-4415.md

### LOW

- **Four orphan catalog ids left by the two dropped federal recipes still seed every device** (`docs/research/MVP-032_STARTER_PROVENANCE_MANIFEST.json`) -- `ing-turkey`, `ing-marjoram`, `ing-tarragon` and `ing-red-beans` are referenced by no authored entry; they trace to Homemade Turkey Soup and New Orleans Red Beans, dropped at the 2026-09-06 roster review. The catalog is never filtered, so all four install and sit as unusable rows on the eager Pantry list. `every_catalog_reference_resolves` only checks the forward direction. Fix: delete the four entries and add the reverse assertion beside it, so the next content drop cannot leave the same residue.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp032-owner-recipes-2026-09-06T1653-4415.md
  **Status:** RESOLVED 2026-09-06 -- the four entries are deleted (catalog 164 -> 160) and the
  reverse assertion `every_catalog_id_is_referenced_by_some_entry` is in `starter.rs` beside
  `every_catalog_reference_resolves`. It runs over `all_starter_content()` on purpose: against the
  shipped subset it would fail on the eight ids the ten unshipped Kimatta entries legitimately
  hold, which are noted under the `MVP-011` AC-3 entry in `KNOWN_ISSUES.md` instead.

- **`okStarterReport`'s values and its docstring describe three different rosters** (`test/app_test.dart:47`) -- the fixture says `installed: 24, catalogInstalled: 131`, its docstring claims "the 131-entry catalog seeds and the 24 federal entries install", and the new library-invalidation test at `:5376` says it "models 26"; the shipped roster is 49 recipes and 164 catalog entries. Only `pendingCookReview: 10` is current. The widget logic reads the fields as `> 0`, so nothing fails and the drift repeats at `MVP-033`. Fix: set 49/164/49/10 and correct both docstrings, or drop the counts from the prose.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp032-owner-recipes-2026-09-06T1653-4415.md
  **Status:** RESOLVED 2026-09-06 -- fixture set to 49 / 160 / 49 / 10 and both docstrings rewritten
  to say what the numbers are, including the `models 26` claim at `test/app_test.dart:5378`.
  **Revised 2026-09-09 (D-043):** rights quarantine leaves a 31 / 125 / 31 / 10 first-install
  fixture (six cleared federal + 25 owner recipes); the test fixture and its prose were updated.

---
## master -- 2026-09-07

Source: /home/davidlinux/.claude/reviews/impl-handoff-mvp-032-generalized-2026-09-07T0940-9011.md
Full review: /home/davidlinux/.claude/reviews/redteam-mvp-032-generalized-2026-09-07T0957-70a3.md

### LOW

- **`without_starters` exists twice and the two copies have already diverged once** (`rust/crates/kimatta-application/src/lib.rs:474`, `rust/src/api/planner.rs:530`) -- near-identical ~30-line test helpers in different crates; the application copy minted `dismissed-<slug>` without the household segment, panicking `archive_recipe` on a second household. Fixed 2026-09-06, but only one copy has the regression test (`two_households_can_each_dismiss_the_starter_roster`) and the next change must be made twice. Fix: if it recurs, a `#[cfg(test)]` helper on `food-domain`, which both already depend on.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp-032-generalized-2026-09-07T0957-70a3.md
  **Status:** OPEN

- **`cooked_on` and `corrections` carry values their own field docs contradict on all 25 owner entries** (`docs/research/MVP-032_STARTER_PROVENANCE_MANIFEST.json`, `rust/crates/food-domain/src/starter.rs:53`) -- every owner entry records `cooked_on: 2026-09-06`, a date on which nothing was cooked, and puts the attestation caveat in `corrections`, documented as the log of "actionable corrections resolved". A real correction would have to be appended to a caveat sentence, and a later freshness check keyed on `cooked_on` reads a false date. Not user-visible: `cook_review` crosses no bridge surface. Fix: one line in `policy.cook_review` recording the convention, or an `attested_on` field if `cook_review` ever gains a consumer.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp-032-generalized-2026-09-07T0957-70a3.md
  **Status:** OPEN

- **`starter-candidates.md` contradicts itself on who made the five authorship rejections** (`docs/research/starter-candidates.md:30`, `:269`) -- line 30 says the five were "rejected here, not passed to the owner"; lines 269-277 say the owner marked the rows and "the rejections above stand"; `docs/ROADMAP.md:208` summarises AC-1 as "owner marks dated 2026-09-05 (26 `select`, 5 `reject`)". AC-1's whole content is the audit trail, so a document stating both positions weakens it. Fix: reword line 30's parenthetical to say the research pass rejected under the card's authorship rule and the owner ratified at the stop.
  Full review: /home/davidlinux/.claude/reviews/redteam-mvp-032-generalized-2026-09-07T0957-70a3.md
  **Status:** OPEN

---

## master -- 2026-09-07

Source: rust/crates/food-domain/src/starter.rs
Full review: /home/davidlinux/.claude/reviews/redteam-starter-allergen-federal-tests-2026-09-07T1823-7363.md

### LOW

- **The allergen sweep is case-sensitive while `assess` is not** (`rust/crates/food-domain/src/starter.rs:1026`) -- `name.contains(word)` matches raw case; `assess` lowercases tokens. All 530 line names are lowercase today by accident of authoring, with no assertion pinning it, so a future line named `"Ziti"` or `"Ranch dressing"` is skipped entirely and ships the defect the sweep exists to catch. Fix: lowercase once per line before the kind loop, or assert every line name equals its own lowercase.
  Full review: /home/davidlinux/.claude/reviews/redteam-starter-allergen-federal-tests-2026-09-07T1823-7363.md
  **Status:** OPEN

- **`ravioli-bake`'s rule-version-1 miss is pinned in two tests** (`rust/crates/food-domain/src/starter.rs:1004`, `:1124`) -- the `RECORDED_RULE_V1_MISSES` row and `ravioli_bake_declares_gluten_only_through_its_sauce_line` both assert the same negative about the same line, so the `RULE_VERSION` 1 -> 2 bump has two sites to update for one miss and can update one and miss the other. Fix: keep the sweep row (line-scoped, coupled to `policy.expected_conflicts_note`) and fold the sauce-line nuance into its doc comment, or cross-reference the two.
  Full review: /home/davidlinux/.claude/reviews/redteam-starter-allergen-federal-tests-2026-09-07T1823-7363.md
  **Status:** OPEN

- **`unwrap_or_default()` guards a state the same test already proved unreachable** (`rust/crates/food-domain/src/starter.rs:835`) -- `:813` unconditionally asserts `source_url().is_some_and(|u| !u.is_empty())` for the same recipe, so the `None` arm cannot be reached; the guard also re-reads the option instead of binding it once. Fix: hoist one `let url` above the first assertion, or use `.expect()` so a future reordering fails loudly rather than degrading to `""`.
  Full review: /home/davidlinux/.claude/reviews/redteam-starter-allergen-federal-tests-2026-09-07T1823-7363.md
  **Status:** OPEN

---

## master -- 2026-09-14

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-2026-09-14T1024-13b8.md
Full review: /home/davidlinux/.claude/reviews/redteam-apk-auto-publish-2026-09-14T1032-9a79.md

### LOW

- **A pending tag run is silently replaced by a later master push in the shared concurrency group** (`.github/workflows/release-apk.yml:28`) -- `group: release-apk` spans master, `v*` tag and dispatch runs; a queued tag run is cancelled when a master push arrives mid-run, so that tag's release is never created. Fix: note it in the README tag sentence, or key the group on `github.ref`.
  Full review: /home/davidlinux/.claude/reviews/redteam-apk-auto-publish-2026-09-14T1032-9a79.md
  **Status:** OPEN

---

## master -- 2026-09-15

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-2026-09-15T0923-9f58.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-2026-09-15t0923-9f58-2026-09-15T0937-445b.md

### LOW

- **The concurrency comment's rationale for `cancel-in-progress: false` does not match `gh` behaviour** (`.github/workflows/release-apk.yml:25`) -- the comment claims a cancelled publish leaves an asset-less release and a 404 link; `gh release create` makes a draft first, so the real leftover is an orphan draft `build-N` a re-run may collide with. Fix: reword the comment to name the orphan draft.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-2026-09-15t0923-9f58-2026-09-15T0937-445b.md
  **Status:** OPEN

---

## master -- 2026-09-15

Source: /home/davidlinux/.claude/reviews/impl-handoff-fix-001-beta-feedback-fixes-2026-09-15T0935-d846.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-fix-001-beta-feedback-fixes-2026-09-15t0935-d846-2026-09-15T0946-49c0.md

### LOW

- **Row menu actions ignore the per-key busy guard that swipe and checkbox honour** (`lib/features/shopping/shopping_screen.dart:582`) -- `onSelected` runs while the row's write is in flight; a Skip during a pending check writes the pre-check state and can overwrite the check. Fix: `enabled: !busy` on the `PopupMenuButton`.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-fix-001-beta-feedback-fixes-2026-09-15t0935-d846-2026-09-15T0946-49c0.md
  **Status:** RESOLVED 2026-09-15 -- the row `PopupMenuButton` now takes `enabled: !busy`, so the explain sheet it opens is gated too; pinned by `the line menu is disabled while that row's write is in flight` (`test/app_test.dart`).

- **No test covers Accept returning after a later decision** (`lib/features/planning/cover_screen.dart:87`) -- `showAccept` keys on `outcome.applied`; no test pins true→false after a swap/veto, so a regression hiding Accept on a changed unwritten plan would pass. Fix: widget test accept → decide → Accept visible, confirmation gone.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-fix-001-beta-feedback-fixes-2026-09-15t0935-d846-2026-09-15T0946-49c0.md
  **Status:** RESOLVED 2026-09-15 -- pinned by `a decision after accept brings Accept back and drops the confirmation` (`test/app_test.dart`), which passed on first run: no production change was needed.

- **A shopping write in flight across a cycle change is lost from view on return** (`lib/features/shopping/shopping_provider.dart:153`) -- paging away and back while a skip or check is in flight rebuilds the autoDispose notifier, which can fetch before the write lands; the landed write republishes into the disposed notifier, so the row shows its pre-write state until a refresh. Predates FIX-001; found by the fix pass's plan redteam. Fix: invalidate `shoppingProvider(startedAt)` in `_write`'s finally when the cycle changed.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-fix-001-beta-feedback-fixes-2026-09-15t0935-d846-2026-09-15T0946-49c0.md
  **Status:** OPEN

---

## feature/opt-006 -- 2026-09-17

Source: /home/davidlinux/.claude/reviews/impl-handoff-feature-opt-006-2026-09-17T1037-5e24.md
Full review: /home/davidlinux/.claude/reviews/redteam-opt-009-import-2026-09-17T1100-43b9.md

### LOW

- **Household-table heuristic not tied to migrations; overrides newer-schema gate** (`rust/crates/kimatta-storage/src/lib.rs:776`) -- a future migration renaming `household` makes older apps call newer exports "not a Kimatta export"; a foreign DB with a `household` table passes validation. Fix: invariant comment at `MIGRATION_ARRAY`; optionally require a second migration-1 table.
  Full review: /home/davidlinux/.claude/reviews/redteam-opt-009-import-2026-09-17T1100-43b9.md
  **Status:** OPEN

- **Picker platform errors surface as raw PlatformException text** (`lib/features/settings/settings_screen.dart:159`) -- a throwing `FilePicker.pickFile()` reaches `describeFailure`'s `_` arm, showing `Import unavailable: PlatformException(...)`. Fix: catch around `pickImportFile` with fixed copy plus one widget test.
  Full review: /home/davidlinux/.claude/reviews/redteam-opt-009-import-2026-09-17T1100-43b9.md
  **Status:** OPEN

---
