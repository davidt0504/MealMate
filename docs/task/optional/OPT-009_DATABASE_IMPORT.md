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

All four resolved in the owner `grill-me` session of 2026-09-16; planning may proceed.

1. **Where may a file come from? — Any file the system picker returns (owner).** An "Import
   from file" action in Settings opens Android's document picker through the `file_picker`
   package, which copies the choice into app storage and returns a path for the existing
   `restoreDatabase`. No type filter: Android has no reliable MIME type for `.db`, and
   `validate_export` already refuses anything that is not a valid, not-newer database. Rejected
   for now: an "Open with"/share-target entry point — it would need manifest intent filters and
   a second package, and with no clean `.db` type it would claim generic binary files. It can be
   layered on later.
2. **Is a safety export taken first? — Yes (owner).** Import runs the existing `export()` into
   the app's exports directory before restoring, and aborts without replacing anything if that
   export fails. The safety copy becomes the newest export, so the existing "Restore latest
   export" button is the undo, and unlike the single `.pre-restore` generation a second import
   cannot overwrite it (the hazard `BackupBusy`'s comment names). Size: a 25-recipe database is
   307 KB; at an assumed 2 MB for heavy use, ten imports leave at most 20 MB, so pruning is
   deferred until it is measured to matter. Rejected: relying on `.pre-restore` alone (no in-app
   way back, lost after a second import) and a dedicated "Undo import" button (new UI covering
   only the latest import). **Planning note (2026-09-16):** because the safety export gates the
   import, import is unavailable while the live database is damaged or not open; the route there
   is Start fresh, then import.
3. **Is import reachable before a household exists? — Moot (resolved from code).** There is no
   household-less state: `householdProvider` calls `bootstrapHousehold()` on first launch, so a
   fresh install already holds a household and Settings is reachable. Import replaces that
   bootstrapped household wholesale, and `DatabaseGeneration.advanceAfterSwap()` invalidates
   every cached provider so the imported household is what the app then shows.
4. **What is shown when the file is refused? — Existing mapping, verified (resolved from code).**
   A corrupt file maps to `KimattaError::Corrupt`; a newer schema stays `Storage` with the
   message "This data was written by a newer version of the app… Update the app and try again";
   both reach the user through `describeFailure`. The plan must confirm that a file which is not
   a SQLite database at all yields a truthful message rather than a raw SQLite string, and add a
   copy rule if it does not. **Planning finding (2026-09-16):** it does not — a non-SQLite file maps
   to `Corrupt`, whose copy blames the *local* database, and an empty file or another app's SQLite
   database was accepted — at the current schema version it would swap in as a database with no
   household. The plan adds `StorageError::NotAnExport` (no SQLite header, or no `household`
   table) and a backup-specific `Corrupt` copy rule.

## Non-goals

- Sync, merge, or any partial or per-entity import; an import replaces the database wholesale.
- Cloud backup, scheduled or automatic backup, or backup to a remote service.
- Changing the export format, which is the database's own file format (MVP-017 AC-4).
- Reading a newer-schema export.

## Acceptance criteria

- **AC-1:** A database file chosen through the system picker replaces the live database, and an
  export written by an older schema migrates forward on import.
- **AC-2:** Before anything is replaced, import writes a safety export that then appears as the
  latest export; if that export fails, the import stops and the live database is untouched.
  The confirm names what is being replaced, and cancelling — at the picker or at the confirm —
  leaves the live database byte-identical and takes no safety export.
- **AC-3:** A corrupt file, a non-database file, and a newer-schema export are each refused
  with distinct truthful copy, and the live database is untouched in all three cases.
- **AC-4:** On a fresh install, importing replaces the bootstrapped household, and the app then
  shows the imported household's recipes rather than the bootstrapped starter set.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Widget test over an injected picker result; `kimatta-storage` restore test on an older-schema export asserting forward migration (the FIX-001 fixture path already exercises this on device) |
| AC-2 | Widget tests for order (export before restore), export-failure abort, and cancel at both steps with no export written |
| AC-3 | Storage tests per refusal class; copy test for the three messages |
| AC-4 | Widget test importing over a bootstrapped household; emulator walk-through |

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
