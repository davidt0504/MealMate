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
