# PRE-003 — iOS toolchain readiness

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Readiness |
| Workstream | Developer environment (post-launch) |
| Depends on | PRE-002, DEC-005 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | Requires a macOS host or CI runner that does not exist today. Provisioning profiles, signing identities, and any Apple Developer enrolment are owner-authorized and outside this card until separately approved (invariant 15) |

> **Body derived 2026-08-26 (D-029, D-034)** after DEC-005 (D-030). This card mirrors `PRE-002`'s shape deliberately: it is the same spike on a second platform.

## Workflow gate

Before planning or execution, read `docs/ROADMAP.md` and apply the mandatory planning or execution gate in `docs/task/README.md` for `PRE-003`. Do not begin implementation unless it passes. Any implementation plan must repeat the execution gate as its first execution step. Because this card is post-launch, it must not block MVP work.

## Outcome and user value

Prove iOS build and packaging of the Rust-bridged app on a macOS or CI environment. This is the discharge point for the iOS `NOT VERIFIED` item `PRE-002` carries as AC-8 (`owner-accepted 2026-08-24`), and it **re-opens the PRD v3 §22 kill criterion for iOS packaging** — the criterion DEC-005 could only close on Android evidence.

Post-launch by D-015. It must never gate MVP acceptance.

## Authoritative sources

- `docs/PRD_v3.md` §6.3 (Phase-0 requirement; bridge rules), §17 Phase 1, §22 ("Risk: Rust slows shipping" and its kill criterion)
- `docs/ROADMAP.md` D-015, D-027, D-028, D-030 (DEC-005's Android-only scope), D-034; the PRE-002 command contract
- `docs/task/decision/DEC-005_BRIDGE_BACKEND_AND_CORE_COMMITMENT.md` decisions 2 and 5
- `docs/task/MVP_INVARIANTS.md` 14, 15, 21

## Load-bearing constraints

- iOS must not block MVP acceptance (invariant 14, D-015). Nothing here may be added to a delivery gate at or before PRODUCTION-BETA-READY.
- This card **re-opens DEC-005 decision 2**. If release packaging proves unreliable on both integration backends after a bounded spike, the PRD §22 fallback (temporary Dart core behind the same interfaces) returns to the table and the commitment is re-decided, not quietly preserved.
- Generated bindings are never hand-edited. Integration glue that `integrate` writes may be edited only inside the bounded fix budget, and every such edit is recorded (PRE-002's rule).
- The bridge stays coarse whichever backend is used (invariant 21).
- No paid resource, credential, signing identity, or store action without separate owner authorization (invariant 15).
- Versions are re-verified at execution, not copied from this card — PRE-002's pins (Flutter 3.47.1, Rust 1.98.0, FRB 2.13.0) are the starting point, not an assumption.

## Scope

- Add the iOS Rust targets to the pinned toolchain (`rust/rust-toolchain.toml`) alongside the existing Android set.
- Prove `flutter build ios` in debug and release.
- Verify the Rust library is linked for every shipped device and simulator architecture.
- Run on a device or simulator and capture the health screen and the typed-error path, matching PRE-002 AC-5's evidence shape.
- Record the clean-checkout command contract for macOS in `docs/ROADMAP.md`, in the form the PRE-001/DEC-002/PRE-002 contracts already use.
- Record the incremental dev-cycle time (Rust edit → regenerated bindings → iOS build) in seconds.

## Non-goals

- App Store submission, TestFlight distribution, production signing, marketing assets.
- iOS-specific UI work or platform adapters beyond what the spike needs to launch.
- CI provisioning beyond the minimum the spike requires.
- Re-opening the Android commitment, which D-030 settled on its own evidence.

## Decision gates

- If the native-assets backend fails on iOS, try cargokit **within the same bounded fix budget PRE-002 used** — at most two diagnosable fixes per backend, at most one fallback cycle — before concluding anything. Only after both backends have been exhausted does the §22 fallback come into scope, and that is a decision card's call, not this card's.

## Acceptance criteria

- **AC-1:** iOS targets are pinned in `rust-toolchain.toml` and the toolchain, its install locations, and the chosen backend are documented.
- **AC-2:** A typed Rust error crosses the bridge and is observed in Dart on iOS as a matchable exception type.
- **AC-3:** `flutter build ios --debug` succeeds.
- **AC-4:** `flutter build ios --release` succeeds and the artifact contains the Rust library for every shipped architecture.
- **AC-5:** The app launches on an iOS device or simulator; the health screen and the error path are captured.
- **AC-6:** The incremental dev-cycle time is recorded in seconds.
- **AC-7:** DEC-005 decision 2 is explicitly confirmed or re-opened on this evidence, recorded as the next free D-number in `docs/ROADMAP.md`, and `PRE-002` AC-8's `NOT VERIFIED` is discharged or restated with its residual risk.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | `rustup target list --installed`, the pinned file, and the backend recorded in the ROADMAP contract |
| AC-2 | Output of a native-loading test on iOS, or the AC-5 capture of the error path; state which route was used |
| AC-3 | Build output and artifact path |
| AC-4 | Build output plus an architecture listing of the embedded library |
| AC-5 | Device or simulator screenshots of the health screen and the typed-error probe |
| AC-6 | Timed command output |
| AC-7 | The new decision row in `docs/ROADMAP.md` and the updated `PRE-002` evidence row |

## Stop/failure conditions

- Stop before any paid Apple Developer enrolment, signing identity creation, provisioning profile, or store action without separate owner authorization.
- Stop if a fix would require hand-editing generated bindings.
- Two failed backend cycles end the spike and route the evidence to a decision card rather than continuing.

## Handoff

Record PASS/FAIL/NOT VERIFIED evidence and the macOS command contract in `docs/ROADMAP.md`; discharge `PRE-002` AC-8 in its evidence row; update `docs/V3_IMPLEMENTATION_STATUS.md` Blockers ("iOS packaging cannot be verified on this host") and its Deferred row for the iOS build/signing proof.
