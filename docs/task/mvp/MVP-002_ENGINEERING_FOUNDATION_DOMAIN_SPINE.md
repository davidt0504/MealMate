# MVP-002 — Rust household kernel and SQLite foundation

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

> Retitled 2026-08-24 under D-028; formerly the pure-Dart domain spine. The ID and filename are kept because the register and five other cards cite `MVP-002`. Library/crate choices below are confirmed or revised at DEC-005. DEC-002 is not a dependency: it governs the Dart side (MVP-003 onward); this card only edits its command-contract block (AC-4).

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Domain foundation (Rust) |
| Depends on | PRE-002, DEC-005 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`) |
| External actions | None beyond the toolchain PRE-002 installed |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-002`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Land the smallest Rust-owned, SQLite-backed foundation that later food-domain cards build on: typed household/member identity, a tested migration runner, and the bridge health call reading real schema state. Household ownership is the core data boundary (invariant 1); this card makes Rust its owner (invariant 17).

## Authoritative sources

- `docs/PRD_v3.md` §7.1 (household identity), §7.7 (audit ledger, deferred here), §12 (SQLite/schema discipline), §17 Phase 2
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §25 (kernel minimalism), §26 (Rust principles)
- `docs/ROADMAP.md` D-028, D-029, the DEC-005 resolution, the PRE-002 command contract
- `docs/task/MVP_INVARIANTS.md` 1, 17, 20, 21

## Load-bearing constraints

- The kernel crate never imports food-specific types (PRD §5 dependency rule).
- `#![forbid(unsafe_code)]` in the kernel and storage crates; unsafe stays isolated to generated bridge code.
- Foreign keys enabled on every connection; multi-record writes are transactions; explicit, tested migrations (PRD §12).
- No JSON blobs for domain relationships.
- Only primitives a call site needs: identity now; `Policy`, `OutcomeAssessment`, `ActionProposal`, `AttentionRequest`, and the planner ledger are deferred to MVP-023 (principles §25: second-domain evidence before universal abstractions).
- No user data exists yet (see `docs/V3_MIGRATION_PLAN.md`), so no compatibility migration is required; the migration runner is still tested from an empty database.

## Scope

- Turn `rust/` into a workspace holding `household-core`, `kimatta-storage`, and the PRE-002 bridge crate, without moving generated paths. Allowed edges: `kimatta-storage` → `household-core`; bridge → storage directly until `kimatta-application` exists.
- `household-core`: `HouseholdId` and `MemberId` newtypes that reject empty/whitespace input; `Household { id, name: Option<String> }`; `HouseholdMember { id, household_id, display_name }`.
- `kimatta-storage`: `open(path)` that applies migrations and leaves foreign keys enabled; migration v1 creating `household` and `household_member` (FK with cascade); `schema_version(&conn)`; a transactional `insert_household(household, members)`. Libraries as confirmed by DEC-005 (sketch: `rusqlite` with the `bundled` feature so Android builds do not need a system SQLite, `rusqlite_migration`).
- Bridge: `health_check` opens the database at the Dart-supplied path, applies migrations, and returns the schema version; storage errors map to the typed bridge error.
- Add the Rust gate to the DEC-002 command contract (the `lib/domain` lines were removed under D-028) (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`).

## Non-goals

- Food entities (MVP-005/007/012/014/015), Firebase, sync, planner, kernel primitives beyond identity, backup/export.

## Decision gates

- DEC-005 must be Done: it confirms the backend, the crate layout, and the storage libraries this card uses.

## Acceptance criteria

- **AC-1:** `cargo test --workspace` passes and covers: id validation (empty/whitespace rejected, valid round-trips, no silent trimming), migration from an empty database reaches v1, `MIGRATIONS.validate()`, foreign-key enforcement (orphan member rejected), transaction rollback when a later row fails, and idempotent reopen of an already-migrated file database (version unchanged, data kept). `cargo tree -i rusqlite` resolves exactly one version.
- **AC-2:** `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean; no `unsafe` in `household-core` or `kimatta-storage`.
- **AC-3:** The release APK with SQLite runs on the emulator and the health screen shows schema version 1. **D-027 block:** the emulator and adb bridge are owner-run on the Windows host (D-022); if unavailable after the bounded wait, `NOT VERIFIED` with discharge at MVP-004's persistence evidence, and Done requires the owner's `owner-accepted YYYY-MM-DD` clause.
- **AC-4:** The DEC-002 command contract in `docs/ROADMAP.md` carries the Rust gate, and the new gate has been exercised.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | `cargo test` output and `cargo tree -i rusqlite` output |
| AC-2 | Command output; `grep -rn unsafe rust/crates/household-core rust/crates/kimatta-storage` empty |
| AC-3 | Emulator screenshot via the D-022 bridge, or the D-027 block with owner acceptance |
| AC-4 | ROADMAP diff and gate output |

## Stop/failure conditions

- Stop on speculative abstraction (any kernel primitive without a call site), an invariant conflict, or a cross-compile failure that would require `unsafe` or hand-edited generated code.
- Two failed cycles return the card to planning.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record the evidence, resolved library versions, resulting status, DOMAIN-READY progress, and **Next implementation task**. Update the corresponding implementation checklist rows in `docs/V3_IMPLEMENTATION_STATUS.md` without duplicating live current/next status there.

## Dependencies added by this card

Recorded here rather than in a decision card: both are routine packages inside this card's stated scope, and the card already records its Rust crate choices the same way (AC-1). D-031 makes this the general rule — a package inside a card's stated scope is card evidence, not a Draft event; a package that moves an architectural boundary still returns the card to Draft. Fields follow the `DEC-002` §3 precedent.

**`path_provider` (production).**
- *Versions/constraints:* `^2.1.5`, compatible with Dart `^3.13.1`. Android-only use for now; the plugin is federated and ships iOS/desktop implementations this card does not exercise.
- *Rationale:* `health_check` opens and migrates a real database at the Dart-supplied path, so Dart must supply an app-private directory. Dart has no stdlib or platform API for one. `getApplicationSupportDirectory()` maps to `context.getFilesDir()` on Android. MVP-004 needs the same directory, so this is not speculative.
- *Rejected alternatives:* `Directory.systemTemp` — the pre-fix path, verified to resolve to `/data/local/tmp` on Android (see AC-3), which an app uid cannot write; `getApplicationDocumentsDirectory()` — on Android it returns the same data directory but is documented for user-visible documents, and `getApplicationSupportDirectory()` states the intent; deriving the path in Rust — not possible, the directory comes from a platform channel and must be handed in from Dart regardless.
- *Test implications:* none directly testable — the call sits in `main()`, which no test harness invokes, and a `PathProviderPlatform` mock would assert the mock rather than the device. Verification is AC-3's on-device evidence. See the MVP-003 constraint below.
- *Reversal cost:* Low — three lines in `main()` plus one `pubspec.yaml` entry; nothing else imports it.

**`tempfile` (dev-dependency of `kimatta-storage`).**
- *Versions/constraints:* `"3"`, dev-only. Pulls no `rusqlite`/`libsqlite3-sys`, so `cargo tree --workspace -i rusqlite` still resolves one version.
- *Rationale:* `reopen_is_idempotent` needs a real file database (`:memory:` cannot be reopened). The hand-rolled substitute it replaced was pid-named and cleaned up only on success, so a panic left a database that a later pid-reusing run reopened.
- *Rejected alternatives:* a hand-rolled guard struct whose `Drop` removes the file — roughly eight lines to reimplement what the standard crate does, and still needs unique naming solved separately.
- *Test implications:* deletes the manual `remove_file` teardown; `TempDir::drop` cleans up even on panic.
- *Reversal cost:* Low — dev-only, one test.

## Verification record — 2026-08-24

Condensed row in `docs/ROADMAP.md`'s Evidence log; this is the detailed record. Logs in `~/mvp002_evidence/`.

| Criterion | Result | Evidence |
|---|---|---|
| AC-1 | **PASS** | `cargo test --workspace`: `household-core` 2/2 (`rejects_empty_and_whitespace`, `round_trips_without_trimming` — ids kept verbatim, no trimming); `kimatta-storage` 7/7 (`empty_db_migrates_to_v1`, `migrations_validate`, `open_leaves_foreign_keys_on`, `orphan_member_rejected`, `rollback_on_later_row_failure`, `insert_round_trip`, `reopen_is_idempotent`). `cargo tree --workspace -i rusqlite` → `rusqlite v0.40.2` only (used by `kimatta-storage` and `rusqlite_migration v2.6.0`); `libsqlite3-sys v0.38.2`. |
| AC-2 | **PASS** | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `grep -rn unsafe rust/crates/household-core rust/crates/kimatta-storage` empty; both crates carry `#![forbid(unsafe_code)]`. |
| AC-3 | **PASS** | Verified on-device 2026-08-25, `emulator-5554` (AVD `pre001_avd`, API 36, x86_64) over the D-022 bridge. `flutter build apk --release` → `adb install -r` → `am start -n dev.mealmate.temp/.MainActivity`; the health screen reads `health: schema v1 at /data/user/0/dev.mealmate.temp/files/kimatta.db` (`ac3_health.png`), and the typed-error probe reads `probe: KimattaError.InvalidPath` (`ac3_probe.png`). No `AndroidRuntime` exception and no `KimattaError.Storage` in logcat. The path is `context.getFilesDir()` via `getApplicationSupportDirectory()`, so this also discharges the `path_provider` fix end to end: the pre-fix `Directory.systemTemp` would have resolved to `/data/local/tmp` and failed here. **The D-027 block no longer applies** — AC-3 was verified rather than owner-accepted, so Done needs no `owner-accepted` clause. |
| AC-4 | **PASS** | Rust gate line added to the DEC-002 contract (and mirrored in the PRE-002 block and `README.md`), ordered ahead of the Flutter lines to match the PRE-002 block so a Rust failure surfaces before the APK build; exercised: `rust_gate.log` exit 0. `flutter test` 5/5 — `test/bridge_native_test.dart` asserts schema version 1 against a real temp-file database, and `KimattaError_Storage` against a path under a nonexistent directory. |

Amended 2026-08-25 by the MVP-002 redteam fix pass (review `redteam-impl-handoff-master-mvp002-2026-08-24T2213-6556.md`): AC-1 and AC-4 re-run after the fixes and their logs re-captured, AC-3's release APK rebuilt. The original 2026-08-24 verification stands for everything the fix pass did not touch.

Baseline before changes: bridge crate clippy clean, `cargo fmt --check` dirty only on the hand-written `health.rs` stub (replaced by this card), `flutter test` 3/3. Cross-compile diagnostic: `flutter build apk --debug` with `rusqlite bundled` built first try under the native-assets hook (70.7 s cold Gradle). Codegen diff after the `health_check` rewrite was comment-only in `lib/src/rust/api/health.dart`. Owner-work guard: `KNOWN_ISSUES-low.md` and the DEC-005 card hashed identical before/after **the card's own implementation** (`owner_hashes.sha256`, baselined 2026-08-24 21:59). Re-running `sha256sum -c` after 2026-08-24 no longer passes for `KNOWN_ISSUES-low.md`: the 2107, 2152 and 2213 review passes appended LOW entries to it and the 2026-08-25 fix pass resolved two and added two more, all by the review workflow rather than by this card. D-032 has since scoped the guard to the decision cards, the PRD and the v3 docs, excluding the `KNOWN_ISSUES` files for exactly this reason. The DEC-005 card still verifies `OK`; `README.md` and `docs/ROADMAP.md` are in the same baseline but are edited by this card by design. pre-existing uncommitted DEC-005 hunks in `README.md`/`docs/ROADMAP.md` were left in place and ride along with this card's edits.
