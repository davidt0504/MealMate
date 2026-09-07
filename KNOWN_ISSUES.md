# Known Issues

Additional LOW findings are tracked in `KNOWN_ISSUES-low.md`.

## orch/31 -- 2026-08-29

Full review: `~/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2331-d2ca.md`

### MEDIUM

- **A pantry mark re-applied after being removed does not re-hide a line the user previously restored** (`lib/features/shopping/shopping_screen.dart:161`, cf. `lib/features/shopping/shopping_copy.dart:175`) -- residual of the `restored`-flag finding fixed in the same pass. `_setLine` now clamps `restored` to `false` unless the line's status is `omittedPantryMarked`, so an inert flag is dropped by the next write to that line; but the sequence *Add anyway → unmark the ingredient in Pantry → re-mark it* performs no line-state write between the unmark and the re-mark, so the clamp never fires and the stored `restored: true` still routes the line to **To buy** via `sectionFor`. The user sees a pantry mark that appears to have had no effect; the explain sheet's "Back to pantry" does render in that state and clears it. Closing it properly needs the pantry write path to clear `restored` for the ingredient's `m:` line keys — a cross-feature write into `shopping_line_state` coupled to the MVP-015 key format, on a card already approved Done — or a mark-generation stamp on the overlay row. It is also a defensible product reading that "Add anyway" is a durable statement of intent that a later mark should not silently overturn; that question should be settled before either implementation. Full review: `~/.claude/reviews/redteam-impl-handoff-orch-31-2026-08-29T2331-d2ca.md`
  **Status:** OPEN

---

## integration -- 2026-08-29

Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-integration-verify-2026-08-28T1904-03fd.md

### MEDIUM

- **MVP-024 AC-8 attention measurement is owed by the owner before MVP-022 (bound relaxed from before-DEC-003 by owner decision 2026-09-03)** (`docs/measurements/MVP-024_AC8_ATTENTION.md`, `docs/ROADMAP.md` MVP-024 Done row) -- approved Done 2026-09-02 under D-027 with the record unfilled. Fix: run the four-scenario stopwatch protocol (both arms, median of per-scenario ratios) before MVP-022; an adverse result returns MVP-023 to Draft under D-031.
  **Status:** OPEN

- **MVP-025 AC-4 device benchmark is discharged only by a real-hardware run** (`rust/crates/food-domain/examples/beam_width.rs`, `docs/ROADMAP.md` MVP-025 Done row) -- approved Done 2026-09-02 under D-027 with the host-run numbers verifier-reproduced. Fix: before `MVP-022` beta readiness, run the beam_width example on representative low/mid-range Android hardware and record the numbers; re-tune (B, K) only if the recorded device median wall time (5 runs, 1 warmup, cargo-ndk path per the ROADMAP clause) is unacceptable for interactive use — Score is hardware-independent and is not the trigger.
  **Status:** OPEN

- **MVP-011 AC-3 (human cook log) is discharged only as cook reviews are recorded** (`rust/crates/food-domain/content/starter_recipes.json`, `docs/ROADMAP.md` MVP-011 Done row) -- approved Done 2026-08-29 under D-027 with 0/10 recipes reviewed. **Partly resolved 2026-09-05 by MVP-032, and further 2026-09-06:** D-041's two-armed rule ships an entry with a recorded `cook_review` *or* with `rights.basis == us_federal_public_domain` and a `source_url`. **24 NHLBI entries** ship on the rights arm and **25 owner household entries** ship on the cook arm, so a fresh install now holds 49 recipes and is no longer empty. **Read the remaining ten carefully: `basis: original` no longer means "not shipped".** The owner's 25 are also `original` and do ship, on a recorded `cook_review`. What is still pending is the **ten Kimatta-authored entries** — the ones whose `provenance.source_author` is `Kimatta` rather than `David` — which carry `cook_review: null` and ship by neither arm. Fix, for those ten only: cook each and add `"cook_review": {"cooked_on": "YYYY-MM-DD", "by": "...", "corrections": null}`. Only one source collection (*Heart Healthy Home Cooking African American Style*, 3 of the 24 federal entries) states its recipes were tested; the rest ship on publisher standing, which is the residual risk D-041 accepted.
  Side effect worth knowing while those ten wait: the catalog is installed unfiltered, so the eight ids only they reference -- `ing-coconut-milk`, `ing-corn-tortillas`, `ing-curry-powder`, `ing-flour-tortillas`, `ing-peanut-butter`, `ing-red-lentils`, `ing-rice-noodles`, `ing-shrimp` -- install on every device and sit as unusable Pantry rows until these entries ship. `every_catalog_id_is_referenced_by_some_entry` deliberately runs over `all_starter_content()`, so it passes on them and catches only ids no authored entry references at all.
  **Status:** OPEN (for the ten Kimatta-authored entries only)

- **The domain carries one time field where it needs two** (`rust/crates/food-domain/src/recipe.rs`, `prep_minutes`; `rust/crates/food-domain/src/planner/tier0.rs:129-140`) -- a recipe has `prep_minutes` and nothing else, but two different quantities matter: the time the cook must be present, and the elapsed time to the table. Tier 0 compares `prep_minutes` against the household's SLOT_WINDOW and **rejects** a dish that exceeds it, silently -- there is no user-visible explanation for a dish that never appears. With one field the two populations conflict: an attended 60-minute stovetop simmer must record its full duration or it gets proposed into a 30-minute weeknight, while an unattended 6-hour slow-cooker dish must record only its 15 minutes of active work or it is rejected from every realistic slot despite being ideal weeknight food. Worked around 2026-09-06 by owner decision: `prep_minutes` means *time you must be available*, with the test 'could you leave the house while it cooks?' -- recorded in the content file's `policy.prep_minutes`. That is documentation holding a type-shaped gap shut, and a recipe authored by someone who has not read it will silently reintroduce the bug. Fix: add a second field distinguishing active from elapsed time and have Tier 0 test the active one; this would also let the UI show '10 min prep, 6 hrs in the slow cooker'. `OPT-004` already lists this field's meaning as a load-bearing constraint and is where the fix belongs. Revisit before `MVP-022` beta readiness. **Data check, 2026-09-06:** the split needs no new research — both numbers already exist. `docs/research/starter-candidates.md` §1 carries publisher-stated `prep min` and `cook min` columns for every federal candidate, and `refs/owner-recipe-transcriptions.md` carries a cook time per owner card while stating that no card records a prep time; owner prep therefore falls out by subtracting card cook from the shipped `prep_minutes`, cleanly for 22 of the 25 (the derived table is recorded in that transcription file). Three need an owner answer: `davids-chili` and `broccoli-soup`, whose cards record no time at all, and `ravioli-bake`, where 70 = 40 bake + 20 uncovered + 10 stand exactly, so subtraction gives prep 0 and the stand is probably not attended time. **Scope of the 2026-09-06 rule, also recorded in `policy.prep_minutes`:** it is applied to all 25 owner entries and to 4 federal ones (`barbecued-chicken`, `minestrone-soup`, `pink-beans-with-plantain`, `vegetable-stew`); `cold-fusilli-with-summer-vegetables` and `chickpeas-with-tomatoes-and-oregano` are prep-only and correct anyway; `pita-pizzas` follows neither convention; and 17 still carry the 2026-09-05 prep-only convention. Not user-visible today for the reason the entry gives above — no code path writes a `food.slot_window` policy, so Tier 0's window comparison never runs — which is why this stays routed rather than fixed in content.
  **Status:** OPEN

- **`GLUTEN_TERMS` has no term for a pasta shape** (`rust/crates/food-domain/src/restriction.rs`, `GLUTEN_TERMS`; `RULE_VERSION` at `:135`) -- rule version 1 matches `"pasta"`, `"spaghetti"` and `"noodles"` but no shape: not `macaroni`, `rotini`, `orzo`, `ravioli`, `ziti`, `farfalle`, `rigatoni`, `linguine` or `cavatappi`. A content drop that names a line by its shape alone therefore ships a wheat dish gluten-free *by declaration*, and `every_recipe_matches_its_declared_conflicts` passes because the declaration matches what `assess` returns. That is how `crockpot-mac-and-cheese`, `sausage-and-broccoli-orzo` and `taco-pasta` shipped on 2026-09-06; they were closed by naming the lines accurately (see `policy.expected_conflicts_note`), which is a content fix and does not generalise. `ravioli-bake` still declares gluten only through its separate "pasta sauce" line, pinned in the negative by `ravioli_bake_declares_gluten_only_through_its_sauce_line`. Fix: add the shape terms, `RULE_VERSION` 1 -> 2. Note that `every_term_matches_its_own_kind` hard-codes the gluten table length (15). The same bump is where the two other recorded rule-version-1 misses belong -- soy sauce matching no gluten term, and `"ranch dressing"`/`"ranch seasoning"` matching no dairy term, both currently recorded only in `policy.expected_conflicts_note` -- so settle all three together rather than spending the version bump on one. Unlike a content rename, a matcher change reaches already-installed devices, because assessments derive at read time rather than being stored. Revisit before `MVP-022` beta readiness.
  **Status:** OPEN

- **A starter recipe keeps its source attribution however far the user edits it** (`rust/crates/kimatta-storage/src/lib.rs`, `editing_a_starter_recipe_keeps_its_rights_and_slug`; `rust/src/api/recipe.rs:323`) -- the MVP-008 edit path loads a `RecipeDto` whose rights scalars are output-only, and the storage layer deliberately preserves the rights columns and the starter slug rather than blanking them on re-save. That is right for a small edit and wrong for a large one: the test itself renames a starter to "Renamed by the user" and asserts the attribution survives, so a household can edit an NHLBI dish beyond recognition while it still cites NHLBI. Harmless while the recipe is private on device. It matters at `MVP-020`, whose AC-3 requires attribution to survive onto a public web preview -- publishing a heavily edited dish under a federal citation misattributes it. Pre-existing behaviour, not introduced by MVP-032, but MVP-032 makes it reachable: before it, nothing shipped, so no household held a recipe carrying a third-party citation. Fix: decide at `MVP-020` whether publication should drop or qualify the attribution when a starter has diverged -- comparing against the shipped entry for that slug is one available test, since the slug is preserved too.
  **Status:** OPEN

- **`#[frb(ignore)]` was needed on a private struct before codegen could run** (`rust/src/api/health.rs:68`) -- `struct Aside` is internal to the restore path and appears in no `pub fn` signature, but flutter_rust_bridge generated bindings naming it, and the crate then failed to compile with `error[E0603]: struct \`Aside\` is private`. The committed generated files had been stale since the struct was added, so the breakage was latent: any card re-running `flutter_rust_bridge_codegen generate` would have hit it. Found and fixed by MVP-032, which had to regenerate for a DTO doc change. No further action; recorded so the class is visible -- a new non-`pub` type in `rust/src/api/**` needs `#[frb(ignore)]` or codegen will try to bridge it.
  **Status:** RESOLVED 2026-09-05 (MVP-032)

---

## orch/4 -- 2026-08-28

Full review: /home/davidlinux/.claude/reviews/redteam-mvp003-android-shell-2026-08-28T1805-b91e.md

### MEDIUM

- **No card has observed an on-device first create of `kimatta.db`** (`docs/ROADMAP.md:172`, cf. `:171` and `:22`) -- MVP-003's AC-3 records `run-as dev.mealmate.temp ls -l files/kimatta.db` returning a real app-private entry, but its `2026-08-25 09:33` mtime is MVP-002's on-device session, so both cards inspected the same file: MVP-002 created it and MVP-003 reopened it. Correct behaviour, and the migration is idempotent, but no run has yet exercised create-and-migrate against a device with no database present. `docs/ROADMAP.md:22` makes `EMULATOR-PERSISTENCE-READY` conditional on "MVP-003 through MVP-005 Done **with on-device SQLite persistence evidence**", and MVP-004 does not close this by default: its evidence plan asks only for "reuse on second open" (`docs/task/mvp/MVP-004_EMULATOR_AUTH_HOUSEHOLD_PERSISTENCE.md:70`) and "on-device database inspection" (`:72`), both of which pass against the same pre-existing file. Fix: in MVP-004 or MVP-005, precede one on-device run with `pm clear dev.mealmate.temp` (or an uninstall/reinstall) and record the resulting `ls -l` mtime as post-clear, which demonstrates first create; then resolve this entry. Deferred: the behaviour is correct and no gate is being closed today -- this entry is the tracking carrier for the gate at MVP-005.
  **Status:** RESOLVED 2026-08-28 -- MVP-004: `tools/emulator.sh reset` (`pm clear`) at 04:05:54Z, then first launch in airplane mode; `run-as … ls -l files/kimatta.db` reads `2026-08-28 23:05` device-local (post-clear), schema v1, one household + one member created. Evidence `~/mvp004_evidence/`.

---

## master -- 2026-08-26

Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-v3-card-rederivation-2026-08-26T1626-ad3a.md

### MEDIUM

- **MVP-003 contradicts its own v3 amendment banner** (`docs/task/mvp/MVP-003_ANDROID_APP_SHELL_NAVIGATION.md:17`, `:29`, `:52`) -- the banner at `:17` instructs that "Authoritative source adds `docs/PRD_v3.md` §15–16" and adds a sixth "Cover My Week" destination placeholder, but `:29` still cites `docs/PRD_v2.md` alone and AC-1 at `:52` still reads "All five primary destinations". The banner also says "Re-derive this card after DEC-005 is Done" -- DEC-005 is Done (`docs/ROADMAP.md:36`), so the re-derivation is unblocked and overdue rather than merely deferred. Found while fixing the D-034 seam defects (D-036); MVP-003 was deliberately not edited because D-034 fenced it off and `docs/task/SEQUENCE.txt` requires a current approved plan to re-derive it. `MVP-024`'s decision gate was changed to a stop condition so it no longer defers to a re-derived card that does not exist. Fix: fold the banner into the card during its own approved re-derivation plan -- add the PRD v3 sources and correct the destination count together. Deferred: the card is fenced pending that plan; no current card depends on the contradiction being resolved first.
  **Status:** RESOLVED 2026-08-28 (MVP-003 step 2 folded the v3 amendment into the card: PRD v3 sources added, AC-1 corrected to five destinations plus /plan/cover)

- **MVP-022 AC-1 still restates the §26 traceability table it declares as its authority** (`docs/task/mvp/MVP-022_ANDROID_BETA_READINESS.md:57`, cf. `docs/ROADMAP.md:95`, `:99`) -- the table's preamble states "This table is the single authority for §26 traceability; cards do not repeat it, and `MVP-022` AC-1 reads from it", but AC-1 restates row 1's split verbatim ("Item 1's iOS half is carried by `PRE-003` post-launch under D-015 and invariant 14 and is **not** an MVP criterion") and hard-codes the "items **2–17**" range, which silently breaks if the table gains or loses a row. Found by the D-036 review after that pass removed the same defect from `MVP-024`'s two sites. Left in place deliberately: D-034 tiers `MVP-018`–`MVP-022` as content preserved verbatim pending `DEC-003`/`DEC-004`, and this is release-gate content. Fix: narrow AC-1 to "every §26 item the table assigns to a card, per the table" and drop the row-1 restatement, when `DEC-003` releases that band. Deferred: the table and AC-1 agree today, so the risk is drift, not a live contradiction.
  **Status:** OPEN

---

## master -- 2026-08-21

Full review: /home/davidlinux/.claude/reviews/redteam-dec-001-naming-identity-2026-08-21T1001-911d.md

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
  **Status:** RESOLVED 2026-08-28 (prose sweep discharged across the 2026-08-26 task-system rework, the 2026-08-27 MVP-002 closeout, and MVP-003 step 1c; tools/emulator.sh tracked)

## master -- 2026-08-25

Full review: /home/davidlinux/.claude/reviews/arch-dir-src-2026-08-25T0900-9639.md

### MEDIUM

- **The Dart error contract and FRB's app initializer live inside the `health` api module** (`rust/src/api/health.rs`, the `KimattaError` enum, its `From<StorageError>` impl, and `init_app`) -- `KimattaError` is documented as "Bridge error surface. Variants are the Dart-matchable contract", a whole-crate concern, and `init_app` is FRB's app-global initializer; both sit beside the health probe. FRB mirrors module structure into Dart, so the contract is published at `lib/src/rust/api/health.dart` and `lib/main.dart` imports the error type from a health module. A second api module must then either `use crate::api::health::KimattaError` -- making a persistence module read as a dependent of a health probe -- or declare its own enum, splitting the Dart error contract into two unrelated sealed classes. Fix: move `KimattaError` and its `From` impl to `api/error.rs`; `init_app` should move too, though its case is weaker since Dart never names it (only `RustLib.init()` calls it) and moving it rewires `frb_generated.*`. Deferred: gated on MVP-002's AC-3 artifact -- a codegen cycle rebuilds the release APK whose on-device evidence was captured 2026-08-25 and not yet closed. Trigger is a second `api` module existing, which no card has yet created; the cost then is a breaking change to a Dart import path app code depends on, so it belongs inside that card rather than after it.
  **Status:** RESOLVED 2026-08-28 -- MVP-004: `KimattaError` and its `From` impls moved to `rust/src/api/error.rs` (Dart: `lib/src/rust/api/error.dart`); `init_app` stays in `health.rs` per the weaker case noted.

- **`insert_household`'s parameters cannot be constructed by the bridge crate** (`rust/crates/kimatta-storage/src/lib.rs`, `insert_household`; `rust/Cargo.toml`) -- the signature takes `&Household` and `&[HouseholdMember]` from `household_core`, but `kimatta-storage` re-exports nothing and the bridge crate does not depend on `household-core`, so the storage crate's only write API is uncallable from its only intended consumer. Invisible today because `insert_household` has no caller outside its own test module. Fix: **change** the existing `use household_core::{Household, HouseholdMember};` to `pub use household_core::{Household, HouseholdId, HouseholdMember, IdError, MemberId};` -- adding a second import of those names alongside the existing one is `E0252`, so this is a conversion of that line, not an addition. That keeps the sanctioned edge list (bridge -> storage) intact; adding `household-core` as a second direct bridge dependency also works but widens the bridge's dependency surface. Deferred: gated on MVP-002's AC-3 artifact, and it is a one-line edit at the moment MVP-004's write path needs it -- a re-export with no consumer is plumbing ahead of demand, the same ground on which this function's mis-parent fix is deferred below.
  **Status:** RESOLVED 2026-08-28 -- MVP-004: the `use` line converted to the `pub use` re-export (plus `pub use rusqlite::Connection`); the bridge's `api/household.rs` and `db.rs` consume it without a `household-core` or `rusqlite` dependency.

- **`health_check` establishes an open-and-migrate-per-call pattern with no connection ownership** (`rust/src/api/health.rs`, `health_check`; `rust/crates/kimatta-storage/src/lib.rs`, `open`) -- every call opens a fresh `Connection`, sets `foreign_keys` OFF, runs `to_latest`, sets it back ON, reads `user_version`, then drops the connection. Nothing holds or shares a connection and no `busy_timeout` is set, so SQLite's default no-busy-handler applies and overlapping calls would contend; FRB dispatches non-`sync` functions on a worker pool, so overlap is reachable in principle. **Not reachable today:** `lib/app/app.dart` makes exactly one storage-touching call -- a single `ref.listen(healthReportProvider, …)` subscription held for the app's lifetime, so the provider opens and migrates once at startup and `SettingsScreen`'s `ref.watch` reuses that same result rather than creating a second one (pinned by `test/app_test.dart`'s "the health provider is created once per launch"). So no two calls can contend on the file. This is the shape MVP-004's real persistence calls will copy. Fix: decide connection ownership before a second storage-touching bridge function exists -- a process-wide `OnceLock<Mutex<Connection>>` in the bridge with migrations run once at `init_app`, or an open-once handle returned to Dart; a single owned connection also makes `busy_timeout` moot. Deferred: gated on MVP-002's AC-3 artifact; a design decision for MVP-004, and settling an ownership model before its first real caller fixes that caller's shape prematurely.
  **Status:** RESOLVED 2026-08-28 -- MVP-004: `rust/src/db.rs` holds one process-wide `Mutex<Option<Connection>>`; `open_database` (renamed from `health_check`) installs it once and `bootstrap_household`/`rename_household` borrow it (`NotOpen` otherwise, pinned by `with_no_connection_is_not_open` and the first Dart bridge test).

- **`insert_household` accepts a member belonging to a different existing household** (`rust/crates/kimatta-storage/src/lib.rs`, `insert_household`) -- **moved here from `KNOWN_ISSUES-low.md` on 2026-08-25 and re-rated MEDIUM**; originally raised 2026-08-25 by the MVP-002 redteam fix pass, re-rated by the arch review above, which classes it MEDIUM as the only data-integrity finding in its set. The foreign key only tests existence, so a member whose `household_id` names another *existing* household is inserted successfully even though the function's name and position imply the members are that household's. `orphan_member_rejected` cannot detect it: it uses a household id that does not exist at all, so it proves foreign-key enforcement rather than the pairing invariant. The 2213 review's MEDIUM was closed on its doc half only -- the doc comment now states that callers own the parent/child pairing. The arch review adds two things: (a) a characterization test for the mis-parented case needs no `StorageError` variant and no caller, so the "wait for the caller" reasoning does not by itself reach the missing test; (b) it offers a second fix option alongside the guard -- verbatim, *"drop `household_id` from the member parameter entirely and derive it from `household.id` at insert time, which makes the class of bug unrepresentable"*. Fix: either enforce `m.household_id == household.id`, or take the parameter-shape option, plus the two-household test either way. Deferred: both options change `insert_household`'s contract -- the guard by adding a `StorageError` variant, the parameter-shape option by changing what a caller passes -- and MVP-004's write path is what determines the right shape, so choosing now settles that caller's contract prematurely. Resolve with MVP-004's write path; add the test in the same pass.
  **Status:** RESOLVED 2026-08-28 -- MVP-004: guard `m.household_id == household.id` before any write, `StorageError::MemberHouseholdMismatch`, and the two-household test `mismatched_member_rejected`.

## orch/16 -- 2026-08-29

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-16-2026-08-29T1022-c507.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-16-2026-08-29T1040-68c3.md

### MEDIUM

- **Restriction-load failure breaks every recipe read, and in `save_recipe_in` lands post-commit** (`rust/src/api/recipe.rs:373`) -- one unparseable `household_restriction` row (`CorruptRestriction`, reachable by downgrade past a vocabulary addition) kills list/load/save/archive/restore for the whole library. `db::with` is not transactional (`rust/src/db.rs:23`), so the save at `:399` has committed before `stored_recipe` errors: the user sees a write failure on a stored recipe, and because a create sends an empty id minted in Rust, each retry stores a duplicate. Deterministic, so it repeats. Fix: keep the recipe read paths independent of restriction-load failure, or keep the read-back free of newly fallible work.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-16-2026-08-29T1040-68c3.md
  **Status:** PARTIALLY RESOLVED 2026-09-03 -- MVP-017 fixed the write-path half: the bridge save/archive/restore compositions now run load -> write -> read-back inside one IMMEDIATE transaction (`save_recipe_in`/`archive_recipe_in`/`restore_recipe_in` seams in kimatta-storage; `a_failing_read_back_rolls_the_save_back` pins rollback and no-duplicate-on-retry). The read-path half stays OPEN: one corrupt restriction row still fails every recipe list/load; out of MVP-017's minimal scope.

## orch/33 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-33-2026-09-02T1146-243b.md
Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md

### MEDIUM

- **Tier-5 pantry fit and ingredient overlap can outweigh the repeat penalty** (`rust/crates/food-domain/src/planner/score.rs:218`) -- four terms share tier 5: `REPEAT_IN_CYCLE` at `-2 * repeats` (:218) against `INGREDIENT_OVERLAP` and `PANTRY_FIT` at `+min(count, 3)` each (:243, :252). A repeated dish with three pantry marks and three shared refs scores +4 while a novel dish scores 0, so the beam repeats it. AC-3's `sequence_fixture_picks_cycle_optimal_over_slot_optimal` asserts no adjacent repeat on one fixture whose dishes carry no pantry marks, so the property is unpinned. Deferred: a weight-tuning question MVP-025's fixtures and beam benchmark are chartered to settle; retuning without them risks a different miscalibration. Fix: scale the repeat penalty above the positive cap, or give variety its own sub-tier ahead of reuse, plus a fixture where pantry fit and repetition oppose.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** OPEN

- **`selected_action` cannot record a declined apply** (`rust/crates/kimatta-application/src/lib.rs:68`) -- `will_apply = req.apply && status != NeedsAttention`, and the ledger writes `ACTION_APPLY` or `ACTION_PROPOSE` from that single bool (:90-95), so "the user asked to automate and the controller refused" and "the user asked for a preview" produce byte-identical rows in the column that records intent. Pinned by the crate's own `apply_under_needs_attention_records_and_does_not_write` (:413, :433). No wrong output; the ledger loses the most diagnostic event it exists to capture. Deferred: a third stored token needs the kernel token-table treatment (`ALL`/`as_str`/`parse` plus a `CorruptLedgerEntry` case in `list_ledger_entries`) and a ledger-schema decision belonging with MVP-024's explain surface, the first consumer of the distinction. Fix: an `ACTION_DECLINED` token on the `req.apply && !will_apply` path.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** RESOLVED 2026-09-02 (MVP-024) -- the token already ships: `ACTION_DECLINED = "declined"` written on the `req.apply && !will_apply` path (`kimatta-application/src/lib.rs`), pinned by `a_declined_apply_is_distinguishable_from_a_preview`. Residual ("declined distinction never surfaced in any UI") recorded as a new low entry in `KNOWN_ISSUES-low.md`, not a re-defer of this one.

- **`offset_cycles` is unvalidated at the bridge and surfaces a pure input error as a database failure** (`rust/src/api/planner.rs:352`) -- the rustdoc at :341-342 promises "dates are parsed before storage is touched, so a malformed `today` is a typed `Planning` error" and :347 honours it, but the sibling input `offset_cycles` is passed through untouched and validated only inside `load_planning_snapshot` -> `window_containing` -> `CycleWindowOverflow` -> `StorageError::Planning` -> `KimattaError::Storage`. `coverCycle(offsetCycles: 2147483647)` therefore reads as "Household unavailable: ..." (`lib/features/household/household_screen.dart:27`). No panic -- `window_containing` is overflow-safe. `today`'s half is pinned by `test/bridge_native_test.dart:586-596`; this half is not. Deferred: wrong error *category*, no data effect; pairs with MVP-024's error-surface work where the Dart copy is decided. Fix: range-check `offset_cycles` beside the `today` parse and return the `Planning` variant, plus a mirroring Dart test.
  Full review: /home/davidlinux/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1158-e249.md
  **Status:** RESOLVED 2026-09-02 (MVP-024) -- `cover_in` and `record_in` reject `|offset_cycles| > 520` with `KimattaError::Planning` before storage is touched (`rust/src/api/planner.rs`, `MAX_OFFSET_CYCLES`); pinned by `offset_cycles_beyond_the_bound_is_a_typed_planning_error` and the mirroring Dart assertion in `test/bridge_native_test.dart`.

## orch/33 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-33-2026-09-02T1354-52de.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1401-4b38.md

### MEDIUM

- **`save_planned_meal` accepts a `locked` flag it never writes** (`rust/crates/kimatta-storage/src/lib.rs:2146`) -- `PlannedMeal::new(..., locked)` takes a lock but the INSERT names only `(id, household_id, date, slot)` and the `ON CONFLICT` arm updates `date` and `slot`, so a caller constructing a locked occurrence gets an unlocked row and no error. Not a slip: documented at :2079-2081, at the DTO field (`rust/src/api/planned_meals.rs:38-39`), at :137 and on `stored()` at :159, and pinned by `a_sent_locked_flag_is_ignored_on_save` (:333) and `set_lock_then_save_keeps_it_locked` (:344). Both production writers hardcode `locked: false`, so nothing is presently wrong; the smell is a constructor parameter that is inert on the save path. Deferred: reversing it would break a contract three files depend on, for no caller. Fix: a second constructor for the loader (`meal_from_rows` at :2309-2329 needs the parameter), then drop it from `new` so `set_planned_meal_lock` is the only way to set a lock.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-33-2026-09-02T1401-4b38.md
  **Status:** OPEN

## orch/37 -- 2026-09-02

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-37-2026-09-02T1936-1e59.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T1949-115e.md

### MEDIUM

- **Swap outside the assessed window records evidence about a window it did not change** (`rust/crates/kimatta-application/src/lib.rs:203`) -- `record_decision` snapshots `(today, offset_cycles)` but `PlanDecision::Swap` writes at `decision.date`, unchecked against that window; `save_planned_meal_in` checks household/cycle/slot, never the date. A far-dated swap commits a locked row beside a `correct` row whose hash and statuses say nothing changed. Deferred: unreachable from the UI (dates come from rendered slots). Fix: derive the window as `cover_in` does and reject an out-of-window date, or snapshot the window containing `date`.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T1949-115e.md
  **Status:** RESOLVED 2026-09-02 -- `record_decision` now derives the window from the `before` snapshot (`PlanningSnapshot::dates()`) and refuses a `Swap` outside it with `ApplicationError::DecisionOutsideWindow`, before any write, so the transaction rolls back. `Veto`/`RestrictionsReviewed` stay un-gated: their `date` is ledger-only.

- **No command can read, edit or delete a `policy` row, so every policy write is permanent** (`rust/crates/kimatta-application/src/lib.rs`, `PolicyTypes::HARD_VETO` in `record_decision`) -- `record_decision` is the only policy writer and it only ever inserts or enables. `rust/src/api/` exposes no policies module and no Dart screen reads one, so a household cannot undo anything it has told the app. Two instances ship today: `food.hard_veto` -- "Never suggest X" tapped on the wrong tile rejects that dish at Tier 0 for the life of the install -- and `food.restrictions_reviewed`, where a mis-tap on "We don't have any" cannot be taken back from any screen. MVP-024 made the veto dialog's copy honest about this rather than pointing at a settings surface that does not exist, and made the reviewed marker retire itself when the restriction set changes; neither gives the household a way back. Fix: a policy list/disable command plus a surface on the restrictions or household screen, then restore reversal wording to `vetoConfirmBody`. Full review: ~/.claude/reviews/redteam-impl-handoff-orch-37-2026-09-02T1949-115e.md
  **Status:** OPEN

## orch/39 -- 2026-09-03

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-39-2026-09-02T2310-dff6.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-02T2320-5b75.md

### MEDIUM

- **`.pre-restore` is preserved but unreachable from any UI path** (`lib/features/settings/backup_provider.dart:44`) -- when a restore fails at both the swap and the put-back, `preserved_aside_error` names `<db_path>.pre-restore` as the surviving copy. The only restore affordance is `latestExport()`, which lists `<applicationSupport>/exports/` and keeps `.endsWith('.db')`, so that file matches neither the directory nor the suffix, and on Android the application-support directory is not reachable from a file manager either. The message was softened to stop promising a retry that cannot work, which makes it truthful but leaves AC-3's "recoverable" half open. Fix: a sibling of `latestExport()` offering `<db_path>.pre-restore` when it exists -- `restore_database` already validates whatever path it is handed.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-02T2320-5b75.md
  **Status:** OPEN

- **`export_database` unlinks the destination before `VACUUM INTO` can replace it** (`rust/crates/kimatta-storage/src/lib.rs:599`) -- the pre-existing file at `dest` is removed first because `VACUUM INTO` refuses to overwrite, so a re-export that then fails (disk full, corruption found mid-read) has destroyed the previous export at that path and written nothing in its place. Same shape as the `.pre-restore` ordering defect fixed on the restore path, and more reachable: export filenames are second-granularity, so a retry within the same second targets the same path. Found while red-teaming the fix plan for that defect; out of scope for orch/39, which closed only the nine review findings. Fix: `VACUUM INTO` a temporary sibling, then rename over `dest`.
  Full review: ~/.claude/reviews/plan-review-fixplan-mvp017-offline-durability-2026-09-02T2351.md
  **Status:** OPEN

## orch/39 -- 2026-09-03

Source: /home/davidlinux/.claude/reviews/impl-handoff-orch-39-2026-09-03T0005-e036.md
Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-03T0012-89d3.md

### MEDIUM

- **No guard against two concurrent restores sharing one staging path** (`rust/src/api/health.rs:71`) -- `{db_path}.restore-staging` is a fixed path and everything before `db::swap` runs outside the `DB` mutex, so a double tap on Restore (the button is never disabled in flight) lets one call's `fs::copy` truncate a staged file another call already verified and is about to commit. Fix: unique per-call staging name, or hold the mutex for all of `restore_database`.
  Full review: ~/.claude/reviews/redteam-impl-handoff-orch-39-2026-09-03T0012-89d3.md
  **Status:** RESOLVED 2026-09-03 -- the whole of `restore_database` runs inside `crate::db::swap`, and both destructive buttons are disabled while an operation is in flight (`BackupBusy`): serialising alone would still let a second confirmed restore overwrite the single kept generation with the first restore's result.

- **A failed restore that leaves no database behind reads as healthy in Settings** (`lib/features/settings/health_provider.dart:12`) -- `healthReportProvider` calls `openDatabase`, and `kimatta_storage::open` creates a fresh database when the file is missing (`rust/crates/kimatta-storage/src/lib.rs:515`). In the one branch where `recover_original` cannot put the original back, the invalidation added for the stale-cache fix therefore re-opens into a *new empty* database: the diagnostics tile reads "schema v10", no recovery row appears (it is gated on `health is AsyncError`), and the only mention of the user's data at `.pre-restore` is a snackbar that has since been dismissed. The sibling entry above ("`.pre-restore` is preserved but unreachable from any UI path") is the other half of this. Found while red-teaming the fix plan for the stale-cache finding; accepted deliberately, since a non-creating probe would change the create-on-missing contract first launch depends on. Fix: a health probe that distinguishes "missing" from "opened", or hold the last destructive failure in a provider the diagnostics tile renders.
  Full review: ~/.claude/reviews/plan-review-fixplan-orch39-backup-swap-2026-09-03T0026.md
  **Status:** OPEN

## integration -- 2026-09-03

Full review: /home/davidlinux/.claude/reviews/redteam-integration-ratification-2026-09-03T1119-b77c.md

### MEDIUM

- **Historical-origin migration tests stop at v4** (`rust/crates/kimatta-storage/src/lib.rs:3726`) -- no test stands a DB up at v5-v9 with rows in the tables those versions introduced (`planned_meal`, `pantry_item`, `shopping_line_state`) and migrates to v10, so the first future table-rebuild migration has no data-preservation harness. Fix: extend the existing v1-v4 origin-test pattern to v7/v8/v9 origins when a card next touches storage.
  Full review: /home/davidlinux/.claude/reviews/redteam-integration-ratification-2026-09-03T1119-b77c.md
  **Status:** OPEN

---
