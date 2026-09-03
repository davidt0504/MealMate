# Known Issues — orchestration audit (owner-filed)

Filed by a read-only audit of `master..integration` on 2026-08-29 (orchestrated cards
MVP-003 → MVP-008). This file is written only by owner-driven audits; the orchestration
pipeline and its children never touch it, so it cannot collide with `KNOWN_ISSUES.md` /
`KNOWN_ISSUES-low.md` appends made inside worktrees. Line anchors are as of `integration`
`6aee375`. Fold into a fix card or into `KNOWN_ISSUES.md` when convenient.

## integration -- 2026-08-29

### HIGH

- **`load_restrictions` under-warns for an unknown household** (`rust/crates/kimatta-storage/src/lib.rs:576`) — returns `Ok(empty)` when the household id does not exist (documented as intended at `:574-575`), while `save_restrictions:546` and `load_member_preferences:695` reject. On the restriction warning surface an empty result reads as "no restrictions", so a stale or foreign id silently disables warnings. Fix: route reads through `require_household` like writes, or return a typed not-found; add a test with a second-household fixture.
  Full review: orchestration audit 2026-08-29 (reviewer agent, §6 item 1)
  **Status:** OPEN

- **Recipe edit re-stamps `householdId` from the live provider** (`lib/features/recipes/recipe_form_screen.dart:204`) — on edit the saved DTO takes `householdId` from `ref.watch(householdProvider)` rather than `_existing.householdId`, so a stale detail entry or a second household silently reassigns recipe ownership (invariant 1). Fix: carry `_existing.householdId` through edits and reject a mismatch.
  Full review: orchestration audit 2026-08-29 (§6 item 2)
  **Status:** OPEN

### MEDIUM

- **Cycle editor save discards the returned DTO and leaves controls live while saving** (`lib/features/planning/cycle_editor_screen.dart:40-65`, controls at `:97,:113,:121,:130`) — sibling screens resync from the returned DTO and gate inputs on `_saving` (`restrictions_screen.dart:146-150`); this one does neither, so a double-tap or server-side normalisation is lost. Fix: mirror the restrictions screen pattern.
  Full review: orchestration audit 2026-08-29 (§6 item 3)
  **Status:** OPEN

- **Bridge flattens every `StorageError` variant to `Storage { message }`** (`rust/src/api/error.rs:10-17`) — Dart cannot distinguish not-found from I/O failure; `describeFailure`'s fallback (`lib/features/household/household_screen.dart:37`) then renders raw `toString()` on six screens. Fix: surface a small typed enum across the bridge (NotFound / Conflict / Io) and map copy in one place. `api/error.rs` has no tests for its documented `From` rules (`:50-53`).
  Full review: orchestration audit 2026-08-29 (§6 item 4)
  **Status:** OPEN

- **Fake recipe notifier in widget tests never re-lists after save** (`test/app_test.dart:228-236` vs production `lib/features/recipes/recipes_provider.dart:47-51`) — ~20 form tests exercise a contract that differs from production; only the `_Seamed…` group (`:2495`) runs the real path. Fix: make the fake call the same re-list hook, or move those assertions into the seamed group.
  Full review: orchestration audit 2026-08-29 (§6 item 5)
  **Status:** OPEN

### LOW

- **Duplicated async-error / saving scaffolding across six screens** (`household_screen.dart:62-81`, `cycle_editor_screen.dart:40-65`, `restrictions_screen.dart:81-112`, `recipe_detail_screen.dart:54-68`, `recipe_list_screen.dart:68-83`, `recipe_form_screen.dart:249-270`); `describeFailure` lives in the household feature and is imported everywhere. Fix: one shared `AsyncValue` error/loading widget + a `_SavingScope` helper in `lib/app/`.
  Full review: orchestration audit 2026-08-29 (§2 Dart)
  **Status:** OPEN

- **Owner-probe SQL and `id_newtype!` duplicated in Rust** (`kimatta-storage/src/lib.rs:794,:897,:930` probe; EXISTS pattern `:432,:886,:1289`; macro in `household-core/src/lib.rs:12-33` and `food-domain/src/recipe.rs:33-54`). `insert_household:245` has no production caller. Fix: one `owner_of(conn, table, id)` helper; share the macro via `household-core` or accept and document.
  Full review: orchestration audit 2026-08-29 (§2 Rust)
  **Status:** OPEN

- **No property tests for `Rational` arithmetic** (`food-domain/src/recipe.rs:238-296`) — gcd/`Ord` cross-multiplication is example-tested only. Fix: `proptest` round-trip and ordering laws.
  Full review: orchestration audit 2026-08-29 (§2 Rust tests)
  **Status:** OPEN

- **Poisoned mutex reused unconditionally; no `busy_timeout`/`journal_mode`** (`rust/src/db.rs:14`, `kimatta-storage/src/lib.rs:230`) — latent behind the single-process mutex; matters once a second connection or background isolate appears (MVP-017).
  Full review: orchestration audit 2026-08-29 (§6 honourable mentions)
  **Status:** OPEN

### Register / process (not code)

- **MVP-003 AC-3 and MVP-004 AC-4 recorded `Done` with verifier `NOT_VERIFIED`** (`docs/ROADMAP.md:173` on `integration` says `NOT_VERIFIED` yet `Done`) — approved before the 2026-08-29 05:16 rule that now stops NOT_VERIFIED at `Verify`. Both are emulator-only evidence (Android smoke launch; no-network first run). Fix: re-verify on the emulator or add dated `owner-accepted` D-027 clauses to both evidence rows.
  **Status:** OPEN

- **`KNOWN_ISSUES-low.md` has five `## orch/14 -- 2026-08-29` section headers from one step**; one stray `~/.claude/reviews/impl-handoff-orch-14-2026-08-29T0829-a55a.meta` sidecar. Cosmetic; fold at the next `/ki-maintain`.
  **Status:** OPEN
