# OPT-009 — Database import: bring an exported file back into the app

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Reliability/offline |
| Depends on | MVP-017 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `grill-me` then `plan-task` |
| External actions | None |

## Workflow gate

Before planning or execution, read `docs/ROADMAP.md` and apply the mandatory planning or
execution gate in `docs/task/README.md` for `OPT-009`. Planning may not start until every
decision gate below records an owner answer folded into this card (D-042).

## Outcome and user value

A household that has an exported database file can put it back into the app — on the same
phone after a reinstall, or on a replacement phone. Today the export can leave but cannot
return, so a reinstall is unrecoverable data loss.

## Why this exists

Recorded 2026-09-15 during the CI signing work. The Rust side is already complete:
`validate_export` (`rust/crates/kimatta-storage/src/lib.rs:715`) opens a candidate read-only,
runs `integrity_check`, rejects only a *newer* schema, and its docstring states that an older
version passes because restore migrates the staged copy forward. `restoreDatabase`
(`lib/src/rust/api/health.dart:40`) performs prepare/stage/verify/commit under the
connection-slot mutex and keeps one `.pre-restore` generation. `Restore latest export` is
wired at `lib/features/settings/settings_screen.dart:240`.

The gap is that `BackupProvider._exportsDir()` (`lib/features/settings/backup_provider.dart:28`)
resolves to `getApplicationSupportDirectory()/exports` — internal storage, erased by uninstall —
and the only chooser is "latest in that directory". A file shared off the phone has no route
back in. `README.md:95` currently tells the user to wait until an import path exists.

The trigger: the project moved from the per-machine debug key to a dedicated release keystore,
so every phone holding a v1.0.x install must uninstall once to move onto the new signature. With
no import, that uninstall is unrecoverable.

## Authoritative sources

- `docs/PRD_v3.md` §12 (SQLite discipline; "Backup/export")
- `docs/task/MVP_INVARIANTS.md` 8 (destructive outcomes stated honestly), 17
- `docs/task/mvp/MVP-017_OFFLINE_CACHE_RECOVERY.md` — AC-4 is an export/restore round-trip on
  one install; importing a foreign file is outside what it discharged, so MVP-017 stays `Done`
  and this card does not return it to `Draft` under D-031
- `docs/task/decision/DEC-007_LEGAL_COMPLIANCE_POSTURE_DECISION.md` — data-rights posture

## Load-bearing constraints

- Restore is already destructive-with-a-net: the overwritten database is kept as
  `<db_path>.pre-restore`, never deleted (invariant 8). Import must not weaken that.
- A newer-schema export is refused, not migrated backwards (`StorageError::NewerSchema`), and
  the existing user-facing message already says to update the app.
- The whole restore runs under the connection-slot mutex; an import path must reuse
  `restoreDatabase` rather than opening its own connection.
- The app holds no `INTERNET` permission, so a file arrives only through the platform picker.

## Decision gates

Each needs an owner answer before planning (`grill-me` agenda):

1. **Where may a file come from?** Any file the system picker returns, or only files the app
   itself exported? A picker is one dependency and covers the replacement-phone case; an
   app-private list cannot see a file that arrived by Drive or Quick Share.
2. **What does the confirm say, and is a safety export taken first?** `restoreConfirmBody`
   already warns that current data is replaced. Taking an automatic export immediately before
   an import would make the action reversible, at the cost of a second copy on a device that
   may be short of space.
3. **Is import reachable before a household exists?** A replacement phone has no data and may
   sit on first run; Settings may not be reachable in the intended shape.
4. **What is shown when the file is refused?** Corrupt, not a database, and newer-schema are
   three different truths, and invariant 8 requires each be stated honestly.

## Non-goals

- Sync, merge, or any partial or per-entity import; an import replaces the database wholesale.
- Cloud backup, scheduled or automatic backup, or backup to a remote service.
- Changing the export format, which is the database's own file format (MVP-017 AC-4).
- Reading a newer-schema export.

## Acceptance criteria

- **AC-1:** A database file chosen from outside the app's own storage replaces the live
  database, and an export written by an older schema migrates forward on import.
- **AC-2:** The confirm names what is being replaced; cancelling leaves the live database
  byte-identical, and the `.pre-restore` generation exists afterwards.
- **AC-3:** A corrupt file, a non-database file, and a newer-schema export are each refused
  with distinct truthful copy, and the live database is untouched in all three cases.
- **AC-4:** An import on a device with no existing household succeeds and lands the user in the
  imported household rather than onboarding.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Widget test over the picker result; `kimatta-storage` test restoring an older-schema export and asserting forward migration |
| AC-2 | Widget tests for confirm and cancel; storage assertion that `.pre-restore` is present |
| AC-3 | Storage tests per refusal class; copy test for the three messages |
| AC-4 | Widget test from a database-less start; emulator walk-through |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, or a need for an
  unauthorized external action.
- Stop and eject to `docs/BACKLOG.md` if a gate above turns out to need a product decision this
  card does not hold.
- After two failed remediation cycles, return to planning rather than weakening acceptance
  criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT
VERIFIED evidence per criterion, decisions, blockers, and **Next implementation task**. Update
`README.md:95`, which currently tells the user to wait until an import path exists.
