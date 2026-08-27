# Known Issues

Additional LOW findings are tracked in `KNOWN_ISSUES-low.md`.

## master -- 2026-08-26

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-v3-card-rederivation-2026-08-26T1626-ad3a.md

### MEDIUM

- **MVP-003 contradicts its own v3 amendment banner** (`docs/task/mvp/MVP-003_ANDROID_APP_SHELL_NAVIGATION.md:17`, `:29`, `:52`) -- the banner at `:17` instructs that "Authoritative source adds `docs/PRD_v3.md` §15–16" and adds a sixth "Cover My Week" destination placeholder, but `:29` still cites `docs/PRD_v2.md` alone and AC-1 at `:52` still reads "All five primary destinations". The banner also says "Re-derive this card after DEC-005 is Done" -- DEC-005 is Done (`docs/ROADMAP.md:36`), so the re-derivation is unblocked and overdue rather than merely deferred. Found while fixing the D-034 seam defects (D-036); MVP-003 was deliberately not edited because D-034 fenced it off and `docs/task/SEQUENCE.txt` requires a current approved plan to re-derive it. `MVP-024`'s decision gate was changed to a stop condition so it no longer defers to a re-derived card that does not exist. Fix: fold the banner into the card during its own approved re-derivation plan -- add the PRD v3 sources and correct the destination count together. Deferred: the card is fenced pending that plan; no current card depends on the contradiction being resolved first.
  **Status:** OPEN

- **MVP-022 AC-1 still restates the §26 traceability table it declares as its authority** (`docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md:57`, cf. `docs/ROADMAP.md:95`, `:99`) -- the table's preamble states "This table is the single authority for §26 traceability; cards do not repeat it, and `MVP-022` AC-1 reads from it", but AC-1 restates row 1's split verbatim ("Item 1's iOS half is carried by `PRE-003` post-launch under D-015 and invariant 14 and is **not** an MVP criterion") and hard-codes the "items **2–17**" range, which silently breaks if the table gains or loses a row. Found by the D-036 review after that pass removed the same defect from `MVP-024`'s two sites. Left in place deliberately: D-034 tiers `MVP-018`–`MVP-022` as content preserved verbatim pending `DEC-003`/`DEC-004`, and this is release-gate content. Fix: narrow AC-1 to "every §26 item the table assigns to a card, per the table" and drop the row-1 restatement, when `DEC-003` releases that band. Deferred: the table and AC-1 agree today, so the risk is drift, not a live contradiction.
  **Status:** OPEN

---

## master -- 2026-08-21

Full review: /home/davidlinux/.claude/reviews/redteam-dec-001-naming-identity-2026-08-21T1001-911d.md

### MEDIUM

- **Gating MVP-018 on DEC-004 may be an unnecessary critical-path coupling** (`docs/ROADMAP.md:51`, `docs/task/mvp/MVP-018_REAL_DEV_BACKEND_DURABLE_AUTH.md:10`) -- MVP-018 provisions a non-production Firebase project, and Firebase registers Android apps by package name additively at no cost. Making DEC-004 a hard prerequisite puts an owner-paced naming task on the critical path for the last five cards to avoid one re-registration in a project that gets discarded. The alternative is to let MVP-018 bind `dev.mealmate.temp` and gate only MVP-021 (App Links / `assetlinks.json`) and MVP-022 (Play, where the ID becomes permanent) on DEC-004. Deferred: needs an owner design decision, not a fix.
  **Status:** RESOLVED 2026-08-21 -- coupling kept deliberately. `DEC-004` promoted to `Ready` instead, so the owner-paced research starts now with ~17 cards of runway rather than being routed around. Firebase Auth users and Firestore rules are project-level and would survive an ID change, so the technical cost of decoupling was low -- but so was the schedule benefit, and the coupling is what guarantees no temporary identifier ever reaches a real project.

---

### LOW

- **`dev.mealmate.temp` reverses into an unowned registrable domain** (`docs/task/decision/DEC-001_PRODUCT_PLATFORM_IDENTITY_DECISION.md:58`) -- reverse-DNS reading is `temp.mealmate.dev`; `.dev` is a live gTLD, so `mealmate.dev` is a real domain the owner does not hold. This is the same objection used on line 60 to reject `com.<owner-domain>.mealmate`. Fix if wanted: use a prefix that cannot be a real domain (`local.mealmate.dev` reverses to `dev.mealmate.local`, reserved by RFC 6762), or record that the collision was considered and accepted. Deferred: bounded risk, the ID never ships.
  **Status:** RESOLVED 2026-08-21 -- identifier kept; acceptance recorded in DEC-001 §1 Reversal cost. With DEC-004 still gating MVP-018, the temp ID lives only across MVP-001--MVP-017, which are emulator-only per D-008, so it never reaches an external system at all.

- **Documentation staleness in README:9 and register ordering** (`docs/task/README.md:9`, `docs/ROADMAP.md:56`) -- README:9 still reads "Only `DEC-001` and `PRE-001` start Ready" while the register shows DEC-001 as Done; and the new DEC-004 row sits at line 56, below the MVP-018/021/022 rows that depend on it. Fix: reword README:9 to name the currently-Ready set, and move the DEC-004 row above MVP-018. Deferred: cosmetic.
  **Status:** RESOLVED 2026-08-21 -- `docs/task/README.md:9` rewritten to name the current Done/Ready set and the transitive gating; DEC-004's register row moved above MVP-018.

- **Evidence log is a new ROADMAP section no contract describes** (`docs/ROADMAP.md:103-107`) -- the Status contract and `docs/task/README.md:12` say evidence is recorded in ROADMAP but do not describe the Date/Card/Evidence table shape, so the next card reaching Done has no format to follow; the single existing row mixes status, four collision claims, and a routing note in one cell. Fix: one sentence in the Status contract naming the Evidence log, plus a Date | Card | Result | Evidence column convention. Deferred: cosmetic; can settle when the next card reaches Done.
  **Status:** RESOLVED 2026-08-21 -- Status contract now names the Evidence log and requires PASS/FAIL/NOT VERIFIED per item; table widened to `Date | Card | Result | Evidence` and the DEC-001 row relabelled. Settled before PRE-001 files the next entry rather than after.

## master -- 2026-08-22

Full review: /home/davidlinux/.claude/reviews/redteam-pre001-android-toolchain-2026-08-22T1549-0545.md

### MEDIUM

- **Done evidence lives entirely outside version control** (`docs/ROADMAP.md`, the PRE-001 Evidence-log row) -- the PRE-001 Evidence log cell cites `C:\pre001_evidence\pre001_evidence.png`, emulator logs, and a WSL copy at `~/pre001_evidence/`. All verified present and genuine, but none of it is in the repository, one copy sits on a Windows drive root, and `.gitignore:3` (`*.log`) would block the logs from being committed as-is. The Status contract in `docs/ROADMAP.md` makes recorded evidence the condition for Done, so the first host cleanup makes an already-Done card unverifiable. Fix: either commit a small evidence directory (the PNG is 58KB; logs need a `.gitignore` exception or a `.txt` rename) or state in the Status contract that evidence is transient and the ROADMAP row is the durable record. Deferred: needs an owner retention policy, not a mechanical fix.
  **Status:** OPEN

---

### LOW

- **AC-5 evidence wording is contradicted by the row that states it** (`docs/ROADMAP.md`, the AC-5 clause of the PRE-001 Evidence-log row) -- the AC-5 cell reads "before/after `git status` identical; only pre-existing untracked `.claude/`", but writing that row modified `docs/ROADMAP.md`, so `git status` shows ` M docs/ROADMAP.md`. The claim is true for the moment the check ran; as written it is falsifiable by inspection. Fix: reword to "no Meal Mate application file changed; the only in-repo change is this documentation row". Deferred: cosmetic wording.
  **Status:** OPEN

- **Firewall prerequisites are unverifiable without elevation, and failure is silent** (`docs/ROADMAP.md`, items 1–2 of "Prerequisites discovered during PRE-001") -- prerequisites 1 and 2 are stated as established fact, but `Get-NetFirewallRule` from an unelevated WSL shell returns "Access is denied", so a later session cannot confirm the rules survived a Windows update or policy refresh. The documented failure mode is a silent timeout, indistinguishable from an adb server that simply is not running. Fix: record a one-line unelevated triage step (check for a listener on 5037 before blaming the firewall). Deferred: mitigation only, no current breakage.
  **Status:** OPEN

- **Gateway derivation assumes WSL2 NAT networking** (`docs/ROADMAP.md`, the `ADB_SERVER_SOCKET` export in the PRE-001 emulator/adb bridge contract) -- `ADB_SERVER_SOCKET=tcp:$(ip route | awk '/default/ {print $3}' | head -1):5037` re-derives at use time and so handles reboot drift, but under WSL2 mirrored networking the default route no longer points at the Windows host and the derived socket silently addresses the wrong target. Currently `172.21.80.1`, so nothing is broken today. Fix: one sentence noting the contract assumes NAT mode and that mirrored mode requires `localhost`. Deferred: speculative until the networking mode changes.
  **Status:** OPEN

## master -- 2026-08-23

Full review: /home/davidlinux/.claude/reviews/redteam-dec-002-engineering-experience-foundation-2026-08-23T1409-bf16.md

### MEDIUM

- **100% `lib/data` coverage gate has no Firebase-testability strategy** (`docs/task/decision/DEC-002_ENGINEERING_EXPERIENCE_FOUNDATION_DECISION.md:88-94`) -- Decision 2 defines `lib/data/` as "Firebase-backed repository implementations"; Decision 3 then requires 100% line coverage on `lib/data/` with no discussion of how Firebase SDK error/edge paths (network failures, permission-denied, emulator-vs-production divergence) will be exercised to 100% without a heavy mocking layer or emulator-backed integration tests, neither of which this decision selects. Doesn't block DEC-002 or MVP-002 (which excludes Firebase-backed implementations as a non-goal), but is a foreseeable landmine for MVP-004, the first card that populates `lib/data/` under this gate. Fix: commit to an interface-fake testing strategy for Firebase-backed repositories before MVP-004 starts, or narrow the 100% gate to exclude thin Firebase SDK wrapper methods. Deferred: surfaces at MVP-004 planning, not now.
  **Status:** OPEN

## master -- 2026-08-24

Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-name-shortlist-2026-08-24T0837-e592.md

### LOW

- **Package-ID column screened `app.<name>`, a form the resolution may not choose** (`docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md:119`, `:290`) -- the summary table's column header is `Package ID `app.<name>``, but line 290 leaves the shape open: "`app.kimatta` is a valid two-segment application ID; three segments is the more common convention, and that remains a judgment call for the resolution step." If the resolution picks three segments, the column's only remaining value -- its ability to surface a `FAIL` -- screened a string that will not be used. Fix: note in the table preamble that the column is scoped to the two-segment form and a three-segment choice is unscreened. Deferred: cosmetic; the column cannot `PASS` by construction, so the practical loss is small.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The table preamble now scopes the column to the two-segment form and records the three-segment shape as unscreened; that shape also became the bind-time fallback named in the Completion criteria, so the scoping note is now load-bearing rather than cosmetic.

- **"the three whose `.app` domain is still free" is false register-wide** (`docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md:293-295`) -- the closing sentence reads "`Ichiju`, `Shitaku`, and `Osusume` are the three whose `.app` domain is still free", but `kondate.app` is also free (table row `:122` reads `FREE`; per-name evidence `:167` reads "`kondate.app` unregistered"). The claim holds only under a silent restriction to non-rejected names. The table reinforces the confusion: `:122` renders Kondate's `FREE` unbolded while `:121`/`:123`/`:124`/`:126` bold theirs, an undocumented distinction. Fix: scope the sentence ("the three surviving candidates whose...") and either bold Kondate's `FREE` or state what the bolding signifies.
  **Status:** RESOLVED 2026-08-24 -- fixed rather than deferred. The sentence is scoped to surviving candidates and names `kondate.app` explicitly; the bolding convention is documented in the table preamble instead of editing twelve rows, which makes Kondate's unbolded `FREE` correct as it stands.

- **The scoped "three *surviving* candidates whose `.app` domain is still free" is still off by one** (`docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md`, the "three *surviving* candidates whose `.app` domain is still free" sentence; the summary-table preamble; the Kimatta row) -- the pass-1 fix above narrowed the sentence to survivors and carved out `kondate.app`, but the new boundary still excludes the lead: `Kimatta`'s `.app` cell in that row reads **FREE**, its disposition is **Lead**, and the summary-table preamble defines the bolding as marking "an unregistered domain for a name still in play" -- so by the table's own convention four surviving candidates have a free `.app`, not three. Fix: say "the three *backups* whose `.app` domain is still free", which is what the clause means and matches the preceding "`Osusume` is the only backup with no recorded encumbrance". Deferred: prose accuracy; the immediately preceding sentence already states **`kimatta.app` is unregistered** in bold, so an attentive reader reconciles it. Full review: /home/davidlinux/.claude/reviews/redteam-dec-004-name-shortlist-2026-08-24T1303-7845.md
  **Status:** OPEN

## master -- 2026-08-24

Source: docs/task/README.md
Full review: /home/davidlinux/.claude/reviews/redteam-done-gate-exception-2026-08-24T1055-a300.md

### MEDIUM

- **Conjunct 1 can be satisfied by a constraint the card wrote for itself** (`docs/task/README.md:51`, `docs/ROADMAP.md`, the D-027 row, `docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md`, the package-ID residual-risk paragraph) -- conjunct 1 of the Done-gate exception accepts "the constraint or stop condition that puts it outside scope" with no requirement that the constraint be externally imposed, owner-ratified, or predate the item. `DEC-004` discharges its package-ID item by citing its own text ("Locked constraint line 32 puts the authoritative check outside this card's own work"), a bullet `DEC-004` authored. Substantively correct in this instance, but as a general rule any future card can immunise an inconvenient required item by adding a `Locked constraints` bullet forbidding the check, then citing it. Only conjunct 3 (owner acceptance) is a genuine external safeguard, which the "all three hold" framing understates; `D-027`'s rationale does not note this. Fix: require the constraint or stop condition be one the owner set or ratified rather than one the card introduced to discharge the item, or state in `D-027` that conjunct 3 is the operative safeguard and 1--2 are documentation requirements. Deferred: needs a design decision on the intended safeguard model; no current card exploits it.
  **Status:** OPEN

---

### LOW

- **New intra-file line-number anchors in a card guaranteed to be re-edited** (`docs/task/decision/DEC-004_NAMING_CLEARANCE_DECISION.md` — the phrases "locked constraint line 33", "Locked constraint line 32", "Line 15's authorization", "Card line 48 carries two conditions", "48 is discharged for the chosen name" (the reference it belongs to is itself split across a line break — the sentence ends "... their Play and Apple rows suggest. Line" and continues "48 is discharged for the chosen name at resolution" — so a sweep for "line 48" misses it), and "clearance checklist line 49") -- the card now self-references by line number in six places. All six resolve correctly today (32 = the Play Console constraint, 33 = the rename-churn bullet, 15 = `External actions`, 48 = the Web/search clearance criterion, 49 = the trademark checklist item), but `DEC-004` is open and will gain a Resolution section at the resolution step, and every anchor sits above the likely insertion point. The prior pass deliberately chose a string anchor over a line number when fixing the roadmap's Google Play clause; these six went the other way in a more volatile file. Fix: replace all six with string anchors, e.g. "the locked constraint forbidding Play Console identifier testing", "the `External actions` field", "the trademark clearance-checklist item". Deferred: drift is a future risk, not a present defect.
  **Status:** OPEN

- **"cards already Done were recorded under the prior practice" asserts a conformity DEC-001 lacked** (`docs/task/README.md:51`, `docs/ROADMAP.md:113`) -- the prior practice, as still written in the same sentence, is "Only all required `PASS` permits Done". `DEC-001` was recorded Done on 2026-08-21 with two clearance items at `NOT VERIFIED` (trademark, Apple App Store title), so it did not conform to that practice. The non-retroactivity clause correctly blocks precedent-reasoning from `DEC-001`, which was the prior pass's actual concern, but the sentence additionally characterises those records as conforming when the `DEC-001` row shows they were not. Fix: "cards already Done are not reopened" states the operative rule without asserting conformity. Deferred: prose accuracy; the operative non-retroactivity rule is correct as written.
  **Status:** RESOLVED 2026-08-24 -- fixed in the same session. The clause now reads "cards already Done are unaffected, whether or not they would have met it", which states the effect without characterising the prior records as conforming. `DEC-001`'s own evidence row separately records that it predates D-027.

## master -- 2026-08-25

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-master-mvp002-emulator-2026-08-25T1249-0fee.md

### MEDIUM

- **`docs/ROADMAP.md` contradicts itself on MVP-002 AC-3, and the deferred prose sweep has no carrier** (`docs/ROADMAP.md:8` vs `:36` and `:133`; `docs/V3_IMPLEMENTATION_STATUS.md:24`; `docs/task/SEQUENCE.txt:10` and `:12-17`; `docs/ROADMAP.md:161` and `:241`) -- `ROADMAP:133` records `AC-3 PASS ... verified on-device 2026-08-25` and "The D-027 block is discharged", while `ROADMAP:8`, the "Next action" line of the file that declares itself the durable operational source of truth, still reads "AC-3 emulator evidence awaits the owner-run D-022 bridge (D-027 block, discharge MVP-004)". `SEQUENCE.txt:10`/`:12-17` and `V3_IMPLEMENTATION_STATUS.md:24` repeat the stale version, and `SEQUENCE.txt` step 0 is marked `[OWNER]`, so the next session's first read sends it to ask the owner for work already done. Separately `ROADMAP:161`/`:241` still name `taskkill /IM emulator.exe /F`, superseded by the measured process-name rule in `tools/emulator.sh:10-12` (`qemu-system-x86_64` works in both launch cases). The sweep was knowingly deferred behind the MVP-002 commit (both files are staged), but the deferral lived only in the handoff doc, which is disposed after review; `tools/emulator.sh` is also still untracked and unreferenced anywhere in the repo. Fix: after the MVP-002 commit, one pass over the five cited lines, plus the ROADMAP pointer to `tools/emulator.sh` and register row D-033; stage the script with the rest of the change set so it survives independently. Deferred: blocked on the commit; this entry is the tracking carrier.
  **Status:** OPEN

## master -- 2026-08-25

Full review: /home/davidlinux/.claude/reviews/arch-dir-src-2026-08-25T0900-9639.md

### MEDIUM

- **The Dart error contract and FRB's app initializer live inside the `health` api module** (`rust/src/api/health.rs`, the `KimattaError` enum, its `From<StorageError>` impl, and `init_app`) -- `KimattaError` is documented as "Bridge error surface. Variants are the Dart-matchable contract", a whole-crate concern, and `init_app` is FRB's app-global initializer; both sit beside the health probe. FRB mirrors module structure into Dart, so the contract is published at `lib/src/rust/api/health.dart` and `lib/main.dart` imports the error type from a health module. A second api module must then either `use crate::api::health::KimattaError` -- making a persistence module read as a dependent of a health probe -- or declare its own enum, splitting the Dart error contract into two unrelated sealed classes. Fix: move `KimattaError` and its `From` impl to `api/error.rs`; `init_app` should move too, though its case is weaker since Dart never names it (only `RustLib.init()` calls it) and moving it rewires `frb_generated.*`. Deferred: gated on MVP-002's AC-3 artifact -- a codegen cycle rebuilds the release APK whose on-device evidence was captured 2026-08-25 and not yet closed. Trigger is a second `api` module existing, which no card has yet created; the cost then is a breaking change to a Dart import path app code depends on, so it belongs inside that card rather than after it.
  **Status:** OPEN

- **`insert_household`'s parameters cannot be constructed by the bridge crate** (`rust/crates/kimatta-storage/src/lib.rs`, `insert_household`; `rust/Cargo.toml`) -- the signature takes `&Household` and `&[HouseholdMember]` from `household_core`, but `kimatta-storage` re-exports nothing and the bridge crate does not depend on `household-core`, so the storage crate's only write API is uncallable from its only intended consumer. Invisible today because `insert_household` has no caller outside its own test module. Fix: **change** the existing `use household_core::{Household, HouseholdMember};` to `pub use household_core::{Household, HouseholdId, HouseholdMember, IdError, MemberId};` -- adding a second import of those names alongside the existing one is `E0252`, so this is a conversion of that line, not an addition. That keeps the sanctioned edge list (bridge -> storage) intact; adding `household-core` as a second direct bridge dependency also works but widens the bridge's dependency surface. Deferred: gated on MVP-002's AC-3 artifact, and it is a one-line edit at the moment MVP-004's write path needs it -- a re-export with no consumer is plumbing ahead of demand, the same ground on which this function's mis-parent fix is deferred below.
  **Status:** OPEN

- **`health_check` establishes an open-and-migrate-per-call pattern with no connection ownership** (`rust/src/api/health.rs`, `health_check`; `rust/crates/kimatta-storage/src/lib.rs`, `open`) -- every call opens a fresh `Connection`, sets `foreign_keys` OFF, runs `to_latest`, sets it back ON, reads `user_version`, then drops the connection. Nothing holds or shares a connection and no `busy_timeout` is set, so SQLite's default no-busy-handler applies and overlapping calls would contend; FRB dispatches non-`sync` functions on a worker pool, so overlap is reachable in principle. **Not reachable today:** `lib/main.dart` makes exactly one storage-touching call, the initial report -- `probeError` is a closure passed to the widget and invoked only on button press, and it passes a whitespace path, which returns `InvalidPath` before storage is touched. So no two calls can contend on the file. This is the shape MVP-004's real persistence calls will copy. Fix: decide connection ownership before a second storage-touching bridge function exists -- a process-wide `OnceLock<Mutex<Connection>>` in the bridge with migrations run once at `init_app`, or an open-once handle returned to Dart; a single owned connection also makes `busy_timeout` moot. Deferred: gated on MVP-002's AC-3 artifact; a design decision for MVP-004, and settling an ownership model before its first real caller fixes that caller's shape prematurely.
  **Status:** OPEN

- **`insert_household` accepts a member belonging to a different existing household** (`rust/crates/kimatta-storage/src/lib.rs`, `insert_household`) -- **moved here from `KNOWN_ISSUES-low.md` on 2026-08-25 and re-rated MEDIUM**; originally raised 2026-08-25 by the MVP-002 redteam fix pass, re-rated by the arch review above, which classes it MEDIUM as the only data-integrity finding in its set. The foreign key only tests existence, so a member whose `household_id` names another *existing* household is inserted successfully even though the function's name and position imply the members are that household's. `orphan_member_rejected` cannot detect it: it uses a household id that does not exist at all, so it proves foreign-key enforcement rather than the pairing invariant. The 2213 review's MEDIUM was closed on its doc half only -- the doc comment now states that callers own the parent/child pairing. The arch review adds two things: (a) a characterization test for the mis-parented case needs no `StorageError` variant and no caller, so the "wait for the caller" reasoning does not by itself reach the missing test; (b) it offers a second fix option alongside the guard -- verbatim, *"drop `household_id` from the member parameter entirely and derive it from `household.id` at insert time, which makes the class of bug unrepresentable"*. Fix: either enforce `m.household_id == household.id`, or take the parameter-shape option, plus the two-household test either way. Deferred: both options change `insert_household`'s contract -- the guard by adding a `StorageError` variant, the parameter-shape option by changing what a caller passes -- and MVP-004's write path is what determines the right shape, so choosing now settles that caller's contract prematurely. Resolve with MVP-004's write path; add the test in the same pass.
  **Status:** OPEN
