# Known Issues

## master -- 2026-08-21

Source: /home/davidlinux/.claude/reviews/impl-handoff-master-2026-08-21T0955-712a.md
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

Source: /home/davidlinux/.claude/reviews/impl-handoff-pre001-android-toolchain-2026-08-22T1531-ac5c.md
Full review: /home/davidlinux/.claude/reviews/redteam-pre001-android-toolchain-2026-08-22T1549-0545.md

### MEDIUM

- **Done evidence lives entirely outside version control** (`docs/ROADMAP.md:109`) -- the PRE-001 Evidence log cell cites `C:\pre001_evidence\pre001_evidence.png`, emulator logs, and a WSL copy at `~/pre001_evidence/`. All verified present and genuine, but none of it is in the repository, one copy sits on a Windows drive root, and `.gitignore:3` (`*.log`) would block the logs from being committed as-is. `docs/ROADMAP.md:13` makes recorded evidence the condition for Done, so the first host cleanup makes an already-Done card unverifiable. Fix: either commit a small evidence directory (the PNG is 58KB; logs need a `.gitignore` exception or a `.txt` rename) or state in the Status contract that evidence is transient and the ROADMAP row is the durable record. Deferred: needs an owner retention policy, not a mechanical fix.
  **Status:** OPEN

---

### LOW

- **AC-5 evidence wording is contradicted by the row that states it** (`docs/ROADMAP.md:109`) -- the AC-5 cell reads "before/after `git status` identical; only pre-existing untracked `.claude/`", but writing that row modified `docs/ROADMAP.md`, so `git status` shows ` M docs/ROADMAP.md`. The claim is true for the moment the check ran; as written it is falsifiable by inspection. Fix: reword to "no Meal Mate application file changed; the only in-repo change is this documentation row". Deferred: cosmetic wording.
  **Status:** OPEN

- **Firewall prerequisites are unverifiable without elevation, and failure is silent** (`docs/ROADMAP.md:135-138`) -- prerequisites 1 and 2 are stated as established fact, but `Get-NetFirewallRule` from an unelevated WSL shell returns "Access is denied", so a later session cannot confirm the rules survived a Windows update or policy refresh. The documented failure mode is a silent timeout, indistinguishable from an adb server that simply is not running. Fix: record a one-line unelevated triage step (check for a listener on 5037 before blaming the firewall). Deferred: mitigation only, no current breakage.
  **Status:** OPEN

- **Gateway derivation assumes WSL2 NAT networking** (`docs/ROADMAP.md:129`) -- `ADB_SERVER_SOCKET=tcp:$(ip route | awk '/default/ {print $3}' | head -1):5037` re-derives at use time and so handles reboot drift, but under WSL2 mirrored networking the default route no longer points at the Windows host and the derived socket silently addresses the wrong target. Currently `172.21.80.1`, so nothing is broken today. Fix: one sentence noting the contract assumes NAT mode and that mirrored mode requires `localhost`. Deferred: speculative until the networking mode changes.
  **Status:** OPEN

## master -- 2026-08-23

Source: /home/davidlinux/.claude/reviews/impl-handoff-dec-002-engineering-experience-foundation-2026-08-23T1353-e5f5.md
Full review: /home/davidlinux/.claude/reviews/redteam-dec-002-engineering-experience-foundation-2026-08-23T1409-bf16.md

### MEDIUM

- **100% `lib/data` coverage gate has no Firebase-testability strategy** (`docs/task/decision/DEC-002_ENGINEERING_EXPERIENCE_FOUNDATION_DECISION.md:88-94`) -- Decision 2 defines `lib/data/` as "Firebase-backed repository implementations"; Decision 3 then requires 100% line coverage on `lib/data/` with no discussion of how Firebase SDK error/edge paths (network failures, permission-denied, emulator-vs-production divergence) will be exercised to 100% without a heavy mocking layer or emulator-backed integration tests, neither of which this decision selects. Doesn't block DEC-002 or MVP-002 (which excludes Firebase-backed implementations as a non-goal), but is a foreseeable landmine for MVP-004, the first card that populates `lib/data/` under this gate. Fix: commit to an interface-fake testing strategy for Firebase-backed repositories before MVP-004 starts, or narrow the 100% gate to exclude thin Firebase SDK wrapper methods. Deferred: surfaces at MVP-004 planning, not now.
  **Status:** OPEN
