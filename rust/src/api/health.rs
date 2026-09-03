use std::path::Path;

use kimatta_storage::StorageError;

use crate::api::error::KimattaError;

#[derive(Debug)]
pub struct HealthReport {
    pub db_path: String,
    pub schema_version: u32,
}

#[flutter_rust_bridge::frb(sync)]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Opens the SQLite database at `db_path` (creating it), applies migrations, installs it as
/// the process-wide connection, and reports the schema version. Empty/whitespace paths are
/// rejected before touching storage. Calling again replaces the connection.
pub fn open_database(db_path: String) -> Result<HealthReport, KimattaError> {
    if db_path.trim().is_empty() {
        return Err(KimattaError::InvalidPath);
    }
    let conn = kimatta_storage::open(&db_path)?;
    let schema_version = kimatta_storage::schema_version(&conn)?;
    crate::db::install(conn);
    Ok(HealthReport {
        db_path,
        schema_version,
    })
}

pub struct ExportReport {
    pub path: String,
    pub schema_version: u32,
}

/// Exports a transactionally consistent copy of the live database to `dest_path` (creating
/// parent directories) and reports where it landed and which schema version it carries.
/// The database's own file format is the export format — see
/// `kimatta_storage::export_database`.
pub fn export_database(dest_path: String) -> Result<ExportReport, KimattaError> {
    if dest_path.trim().is_empty() {
        return Err(KimattaError::InvalidPath);
    }
    let schema_version =
        crate::db::with(|conn| Ok(kimatta_storage::export_database(conn, &dest_path)?))?;
    Ok(ExportReport {
        path: dest_path,
        schema_version,
    })
}

/// The sidecar files SQLite may keep beside a database. Everything that moves, clears or
/// puts back a database file drives off this one list, so no site can cover a subset: a
/// `-wal` left behind where the `-journal` was carried is the same mispairing in a
/// different file.
const SIDECARS: [&str; 3] = ["-journal", "-wal", "-shm"];

/// What [`commit_swap`] actually moved from the live path onto `pre`, so a failure can
/// reverse exactly that set — no more and no less. Both halves matter: `pre` is a fixed
/// name reused by every restore, so a `.pre-restore` this restore did not write is an
/// earlier generation's backup rather than the original (putting it back would install
/// stale data as the live database and consume the one generation kept), and a sidecar
/// moved but not put back would be reunited with the wrong generation (R0).
#[derive(Default)]
struct Aside {
    database: bool,
    sidecars: Vec<&'static str>,
}

/// Replaces the live database at `db_path` with the export at `export_path`, validated
/// first and staged beside the live file — prepare / stage / verify / commit (MVP-017
/// AC-4). The overwritten database is kept as `<db_path>.pre-restore` (one generation),
/// never destroyed; a failure mid-swap puts the original back and reopens it.
///
/// The whole restore, not merely the commit, runs under the connection-slot mutex:
/// `staging` is a fixed path beside the live database, so two restores in flight together
/// would copy over each other's staged file and commit one that was never verified. FRB
/// dispatches bridge calls on a worker pool, so the UI cannot be that guarantee.
pub fn restore_database(
    export_path: String,
    db_path: String,
) -> Result<HealthReport, KimattaError> {
    if export_path.trim().is_empty() || db_path.trim().is_empty() {
        return Err(KimattaError::InvalidPath);
    }
    let staging = format!("{db_path}.restore-staging");
    let pre = format!("{db_path}.pre-restore");
    crate::db::swap(|slot| {
        // Prepare: refuse a missing, corrupt, or newer-versioned export before touching
        // anything. Read-only, so a bad path is never silently created.
        kimatta_storage::validate_export(&export_path)?;

        // Stage: work on a private copy; the live file and connection are still untouched.
        let _ = std::fs::remove_file(&staging);
        // Before the copy, not after: a hot journal left beside `staging` by an earlier
        // attempt that failed in the verify block below would be rolled back by `open` into
        // the freshly copied export, and `integrity_check` cannot catch it — the rolled-back
        // pages are themselves structurally valid (R0). A stale staging *file* needs no such
        // care, since `fs::copy` truncates and overwrites it, but nothing overwrites a
        // sidecar: a removal that fails here must not be swallowed, because no later step
        // would notice.
        for suffix in SIDECARS {
            std::fs::remove_file(format!("{staging}{suffix}"))
                .or_else(ignore_not_found)
                .map_err(StorageError::from)?;
        }
        std::fs::copy(&export_path, &staging).map_err(StorageError::from)?;

        // Verify: migrate the staged copy to the current schema and prove its integrity, all
        // before the live slot is involved. `open` runs quick_check + migrations; the full
        // integrity_check is the stronger final pass.
        {
            let staged = kimatta_storage::open(&staging)?;
            let verdict: String = staged
                .query_row("PRAGMA integrity_check", [], |r| r.get(0))
                .map_err(StorageError::from)?;
            if verdict != "ok" {
                return Err(StorageError::CorruptDatabase { detail: verdict }.into());
            }
        }
        // The staged open may leave sidecars of its own; none of them may follow the file
        // into the live slot. Strict and three-suffix for the same reason as the clear
        // above, which this mirrors.
        for suffix in SIDECARS {
            std::fs::remove_file(format!("{staging}{suffix}"))
                .or_else(ignore_not_found)
                .map_err(StorageError::from)?;
        }

        // Commit: drop the live connection, move the live file and its sidecars aside as one
        // group (R0 — an orphaned hot journal would be rolled back into the restored file),
        // move the staged copy in, and reopen.
        *slot = None;
        let mut aside = Aside::default();
        match commit_swap(&db_path, &staging, &pre, &mut aside) {
            Ok((conn, schema_version)) => {
                *slot = Some(conn);
                Ok(HealthReport {
                    db_path: db_path.clone(),
                    schema_version,
                })
            }
            Err(e) => Err(recover_original(slot, &db_path, &pre, &aside, e)),
        }
    })
}

/// The forward path of the swap: every step that can fail while the live file is in motion.
/// Returns the connection onto the restored file, ready to install.
fn commit_swap(
    db_path: &str,
    staging: &str,
    pre: &str,
    aside: &mut Aside,
) -> Result<(kimatta_storage::Connection, u32), KimattaError> {
    if Path::new(db_path).exists() {
        // One generation of pre-restore kept. The rename replaces an existing `pre`
        // atomically, so the previous generation is only given up once its replacement is
        // in place — removing it first would cost the user a backup on a restore that had
        // nothing to stage aside, and a failure here leaves that generation whole, sidecars
        // included.
        std::fs::rename(db_path, pre).map_err(StorageError::from)?;
        aside.database = true;
        // Past this point the previous generation's *database* is gone, so every sidecar
        // still standing at `{pre}{suffix}` belongs to nothing: each is either replaced
        // atomically by the live file's own sidecar or removed outright, never left to be
        // read as part of the generation now at `pre` (R0). `reset_database` moves the same
        // files rather than deleting them for the same reason — its `{db_path}.corrupt-`
        // destination is timestamp-unique, so it has no earlier generation to displace.
        for suffix in SIDECARS {
            let live = format!("{db_path}{suffix}");
            // A non-file at a sidecar path is not a sidecar. Renaming a directory onto the
            // fixed `{pre}` name would poison that slot for every later restore, so it is
            // refused here exactly as the removal below refuses it (EISDIR).
            if Path::new(&live).exists() && !Path::new(&live).is_file() {
                return Err(KimattaError::Storage {
                    message: format!("a database sidecar file could not be moved aside: {live}"),
                });
            }
            if move_if_exists(&live, &format!("{pre}{suffix}"))? {
                aside.sidecars.push(suffix);
            } else {
                std::fs::remove_file(format!("{pre}{suffix}"))
                    .or_else(ignore_not_found)
                    .map_err(StorageError::from)?;
            }
        }
    } else {
        // Nothing to stage aside, so the kept generation stays whole, sidecars included. A
        // sidecar beside an absent database belongs to neither generation: carrying it onto
        // `{pre}{suffix}` would mispair it with the older `.pre-restore`, and leaving it
        // would let SQLite recover it into the restored file (R0). Deleting it is not the
        // silent destruction invariant 8 forbids — that protects the database file, and a
        // rollback journal or WAL whose database is already gone can be applied to nothing.
        for suffix in SIDECARS {
            std::fs::remove_file(format!("{db_path}{suffix}"))
                .or_else(ignore_not_found)
                .map_err(StorageError::from)?;
        }
    }
    std::fs::rename(staging, db_path).map_err(StorageError::from)?;
    let conn = kimatta_storage::open(db_path)?;
    let schema_version = kimatta_storage::schema_version(&conn)?;
    Ok((conn, schema_version))
}

/// Puts the original database back after a mid-swap failure and reopens it. If even the
/// rename-back fails, the slot is deliberately left empty — falling through to a fresh
/// database here would silently present an empty app (a stop condition); the error names
/// the surviving file instead.
fn recover_original(
    slot: &mut Option<kimatta_storage::Connection>,
    db_path: &str,
    pre: &str,
    aside: &Aside,
    original: KimattaError,
) -> KimattaError {
    // `aside.database`, not `Path::new(pre).exists()`: since `commit_swap` stopped removing
    // a `.pre-restore` it does not replace, a `pre` on disk no longer implies this restore
    // put it there. Renaming an earlier generation's backup into the live slot would install
    // stale data *and* consume the one generation kept. `db_path` is re-checked because the
    // failure may have come after the staged copy already landed there.
    if aside.database && !Path::new(db_path).exists() && Path::new(pre).exists() {
        let put_back = std::fs::rename(pre, db_path).is_ok();
        if put_back {
            for suffix in &aside.sidecars {
                let _ = std::fs::rename(format!("{pre}{suffix}"), format!("{db_path}{suffix}"));
            }
        }
        // Only the sidecars this restore moved travel back. Any other `{pre}{suffix}` is a
        // leftover of the generation whose database the move-aside above replaced, so it
        // belongs to no database on disk: it must neither follow the original home nor stay
        // beside a `pre` the user is about to be told to recover by hand (R0). Best-effort —
        // a leftover that cannot be removed is cleared by the next restore's move-or-remove.
        for suffix in SIDECARS.into_iter().filter(|s| !aside.sidecars.contains(s)) {
            let _ = std::fs::remove_file(format!("{pre}{suffix}"));
        }
        if !put_back {
            return preserved_aside_error(pre);
        }
    }
    match kimatta_storage::open(db_path) {
        Ok(conn) => {
            *slot = Some(conn);
            original
        }
        Err(_) if Path::new(pre).exists() => preserved_aside_error(pre),
        Err(_) => original,
    }
}

/// Moves the (presumed damaged) live database aside as `<db_path>.corrupt-<stamp>` together
/// with its sidecar files, then opens and installs a fresh database at `db_path`. The
/// damaged file is renamed, never deleted — deleting it would be silent destruction
/// (invariant 8), and keeping the journal with it both preserves forensic value and stops
/// SQLite from rolling an orphaned hot journal back into the fresh database (R0).
///
/// `stamp` is supplied by the caller — the UI owns timestamp formatting (compact sortable
/// `yyyyMMdd-HHmmss`, as export filenames use) — and must be a bare filename fragment.
pub fn reset_database(db_path: String, stamp: String) -> Result<HealthReport, KimattaError> {
    if db_path.trim().is_empty() {
        return Err(KimattaError::InvalidPath);
    }
    if stamp.trim().is_empty() || stamp.contains(['/', '\\']) || stamp.contains("..") {
        return Err(KimattaError::InvalidPath);
    }
    let aside = format!("{db_path}.corrupt-{stamp}");
    crate::db::swap(|slot| {
        *slot = None;
        move_if_exists(&db_path, &aside)?;
        for suffix in SIDECARS {
            move_if_exists(&format!("{db_path}{suffix}"), &format!("{aside}{suffix}"))?;
        }
        let conn = kimatta_storage::open(&db_path)?;
        let schema_version = kimatta_storage::schema_version(&conn)?;
        *slot = Some(conn);
        Ok(HealthReport {
            db_path: db_path.clone(),
            schema_version,
        })
    })
}

fn preserved_aside_error(pre: &str) -> KimattaError {
    KimattaError::Storage {
        message: format!(
            "The restore failed and the original database could not be put back. \
             Your data is preserved at {pre} and was not deleted, but the app cannot \
             reopen it from there — that file has to be moved back into place outside \
             the app."
        ),
    }
}

/// `Ok(true)` when `from` existed and was moved, `Ok(false)` when there was nothing to move —
/// `commit_swap` records which of the two happened per sidecar, so a failure later can put
/// back exactly what it took.
fn move_if_exists(from: &str, to: &str) -> Result<bool, KimattaError> {
    match std::fs::rename(from, to) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(StorageError::from(e).into()),
    }
}

fn ignore_not_found(e: std::io::Error) -> Result<(), std::io::Error> {
    if e.kind() == std::io::ErrorKind::NotFound {
        Ok(())
    } else {
        Err(e)
    }
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::api::household::{bootstrap_household, rename_household};
    use crate::db::TEST_DB_LOCK;

    /// One tempdir per test; every path the swap machinery may produce lives inside it.
    struct Rig {
        _dir: tempfile::TempDir,
        db_path: String,
        export_path: String,
    }

    fn rig() -> Rig {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("kimatta.db").to_str().unwrap().to_owned();
        let export_path = dir.path().join("export.db").to_str().unwrap().to_owned();
        Rig {
            _dir: dir,
            db_path,
            export_path,
        }
    }

    /// Open + bootstrap + name, so the database has recognisable content.
    fn seed(rig: &Rig, name: &str) -> String {
        open_database(rig.db_path.clone()).unwrap();
        let h = bootstrap_household().unwrap();
        rename_household(h.id.clone(), Some(name.to_owned())).unwrap();
        h.id
    }

    fn household_name() -> Option<String> {
        bootstrap_household().unwrap().name
    }

    /// The AC-4 proof at the bridge layer: seed → export → mutate → restore returns the
    /// exported content, versioned, with the pre-restore original kept aside.
    #[test]
    fn a_restore_returns_the_exported_content() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        let report = export_database(rig.export_path.clone()).unwrap();
        assert_eq!(report.schema_version, 10);

        let h = bootstrap_household().unwrap();
        rename_household(h.id, Some("Mutated".to_owned())).unwrap();
        assert_eq!(household_name().as_deref(), Some("Mutated"));

        let restored = restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap();
        assert_eq!(restored.schema_version, 10);
        assert_eq!(restored.db_path, rig.db_path);
        assert_eq!(household_name().as_deref(), Some("Casa"));
        // The overwritten database is kept aside, not destroyed (invariant 8).
        assert!(Path::new(&format!("{}.pre-restore", rig.db_path)).exists());
    }

    #[test]
    fn a_missing_or_garbage_export_is_refused_with_the_live_db_untouched() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");

        let missing = format!("{}.does-not-exist", rig.export_path);
        restore_database(missing.clone(), rig.db_path.clone()).unwrap_err();
        assert!(
            !Path::new(&missing).exists(),
            "the probe must not create it"
        );

        std::fs::write(&rig.export_path, [b'x'; 1024]).unwrap();
        let err = restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        assert!(
            matches!(err, KimattaError::Corrupt { .. }),
            "want Corrupt, got {err:?}"
        );
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    #[test]
    fn a_newer_versioned_export_is_refused_in_prose() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        kimatta_storage::rusqlite::Connection::open(&rig.export_path)
            .unwrap()
            .pragma_update(None, "user_version", 11)
            .unwrap();

        let err = restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        match err {
            KimattaError::Storage { ref message } => assert!(
                message.contains("newer version of the app"),
                "got {message:?}"
            ),
            other => panic!("want Storage, got {other:?}"),
        }
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// Adversarial (R2): a failure at the staging stage — before the live connection is
    /// ever dropped — leaves the original serving without a reopen.
    #[test]
    fn a_staging_failure_leaves_the_original_serving() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        // A directory where the staging copy must land makes `fs::copy` fail.
        std::fs::create_dir(format!("{}.restore-staging", rig.db_path)).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// Adversarial (R2): a failure mid-swap — after the live connection is dropped — must
    /// put the original back and reopen it.
    #[test]
    fn a_mid_swap_failure_restores_the_original() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        // A non-empty directory at the pre-restore slot: the move-aside rename cannot
        // replace it, so the swap fails before the live file has moved.
        let pre = format!("{}.pre-restore", rig.db_path);
        std::fs::create_dir(&pre).unwrap();
        std::fs::write(format!("{pre}/occupied"), b"x").unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// Adversarial (R2), the branch the test above cannot reach: a failure *after* the live
    /// file has been renamed aside must put it and its journal back and reopen it. A
    /// directory at the `-shm` sidecar trips the sidecar gate at that point; `-wal` would
    /// not do, since the reopen below would then fail on it too.
    #[test]
    fn a_failure_after_the_live_file_moved_puts_the_original_back() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        std::fs::write(format!("{}-journal", rig.db_path), [b'x'; 512]).unwrap();
        std::fs::create_dir(format!("{}-shm", rig.db_path)).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        let pre = format!("{}.pre-restore", rig.db_path);
        assert!(
            Path::new(&rig.db_path).exists(),
            "the original must be back in the live slot"
        );
        assert!(!Path::new(&pre).exists(), "nothing may be left aside");
        // The journal's content is not asserted: the reopen retires a journal whose header
        // does not validate, so only its departure from `.pre-restore` is stable.
        assert!(
            !Path::new(&format!("{pre}-journal")).exists(),
            "the journal must be put back with its database"
        );
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// Adversarial (R2): both the swap and the reopen fail. The surviving file is named and
    /// the slot is left empty rather than falling through to a fresh, empty database.
    #[test]
    fn a_double_failure_names_the_preserved_file() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        // The live file is damaged beyond reopening, and the pre-restore slot is occupied by
        // a directory the move-aside cannot replace: the swap fails, and so does the reopen.
        crate::db::swap(|slot| {
            *slot = None;
            Ok(())
        })
        .unwrap();
        std::fs::write(&rig.db_path, [b'x'; 1024]).unwrap();
        let pre = format!("{}.pre-restore", rig.db_path);
        std::fs::create_dir(&pre).unwrap();
        std::fs::write(format!("{pre}/occupied"), b"x").unwrap();

        let err = restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        match err {
            KimattaError::Storage { ref message } => {
                assert!(
                    message.contains(&pre),
                    "the surviving file must be named: {message:?}"
                );
                assert!(
                    message.contains("moved back into place"),
                    "the message must say what recovery takes: {message:?}"
                );
                assert!(
                    !message.contains("try again"),
                    "the app offers no retry that would work: {message:?}"
                );
            }
            other => panic!("want Storage, got {other:?}"),
        }
    }

    /// AC-3: start-fresh renames the damaged database and its journal aside — never
    /// deletes — and leaves no sidecar beside the fresh file for SQLite to "recover" from.
    #[test]
    fn reset_moves_the_damaged_db_and_its_journal_aside_and_starts_fresh() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        std::fs::write(&rig.db_path, [b'x'; 1024]).unwrap();
        // The user-visible precondition: the damaged file refuses to open, typed.
        let err = open_database(rig.db_path.clone()).unwrap_err();
        assert!(
            matches!(err, KimattaError::Corrupt { .. }),
            "want Corrupt, got {err:?}"
        );
        // After the failed open: SQLite retires an unreadable journal while probing, so a
        // journal fabricated earlier would already be gone.
        let journal = format!("{}-journal", rig.db_path);
        std::fs::write(&journal, [b'j'; 512]).unwrap();

        let report = reset_database(rig.db_path.clone(), "20260902-120000".to_owned()).unwrap();
        assert_eq!(report.schema_version, 10);
        bootstrap_household().unwrap();

        let aside = format!("{}.corrupt-20260902-120000", rig.db_path);
        assert_eq!(std::fs::read(&aside).unwrap(), vec![b'x'; 1024]);
        assert_eq!(
            std::fs::read(format!("{aside}-journal")).unwrap(),
            vec![b'j'; 512]
        );
        assert!(
            !Path::new(&journal).exists(),
            "no sidecar may remain beside the fresh database"
        );
    }

    /// Adversarial: a stamp that could escape the data directory is refused outright.
    #[test]
    fn reset_refuses_a_path_shaped_stamp() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        for stamp in ["", "  ", "a/b", "a\\b", ".."] {
            let err = reset_database(rig.db_path.clone(), stamp.to_owned()).unwrap_err();
            assert!(
                matches!(err, KimattaError::InvalidPath),
                "stamp {stamp:?}: want InvalidPath, got {err:?}"
            );
        }
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// R0, staging half: the staged copy's sidecars are cleared *before* the export is copied
    /// in, so a sidecar that cannot be cleared refuses the restore rather than letting `open`
    /// consume it. A `-shm` with no `-wal` beside it is the one sidecar SQLite ignores
    /// entirely, which is what makes it usable as an injection point here.
    #[test]
    fn a_staging_sidecar_that_cannot_be_cleared_refuses_the_restore() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        std::fs::create_dir(format!("{}.restore-staging-shm", rig.db_path)).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// The same clear on the ordinary path: a stale sidecar left by an earlier attempt is gone
    /// before the copy, not merely after the verify block.
    #[test]
    fn a_stale_staging_shm_is_cleared_before_the_copy() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        let shm = format!("{}.restore-staging-shm", rig.db_path);
        std::fs::write(&shm, [b's'; 64]).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap();
        assert!(
            !Path::new(&shm).exists(),
            "no staging sidecar may outlive the copy"
        );
    }

    /// Adversarial (R0): the scenario the pre-copy clear exists for — a second restore, of a
    /// different export, with a previous attempt's leavings still on disk. It must serve the
    /// export the user chose. A genuinely hot journal cannot be fabricated from a test (SQLite
    /// retires one whose header does not validate), so the leavings are the staging file and
    /// the one sidecar that survives an open untouched.
    #[test]
    fn a_second_restore_serves_the_export_it_named() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        let second = format!("{}.second", rig.export_path);
        let h = bootstrap_household().unwrap();
        rename_household(h.id, Some("Otra".to_owned())).unwrap();
        export_database(second.clone()).unwrap();

        let staging = format!("{}.restore-staging", rig.db_path);
        std::fs::write(&staging, [b'q'; 2048]).unwrap();
        let shm = format!("{staging}-shm");
        std::fs::write(&shm, [b's'; 64]).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap();
        assert_eq!(household_name().as_deref(), Some("Casa"));
        assert!(
            !Path::new(&shm).exists(),
            "no staging sidecar may outlive the copy"
        );
    }

    /// A restore with no live file to stage aside must not cost the user the generation that
    /// is already kept: nothing replaces it, so nothing may remove it.
    #[test]
    fn a_restore_with_no_live_file_keeps_the_earlier_generation() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        let pre = format!("{}.pre-restore", rig.db_path);
        std::fs::write(&pre, [b'p'; 128]).unwrap();
        std::fs::write(format!("{pre}-journal"), [b'j'; 64]).unwrap();
        // A journal whose database is gone: it belongs to neither generation, so it may
        // neither follow the older `.pre-restore` nor stay beside the restored file. The
        // other two sidecars are orphaned the same way and are cleared by the same loop.
        let stray = format!("{}-journal", rig.db_path);
        std::fs::write(&stray, [b'x'; 32]).unwrap();
        std::fs::write(format!("{}-wal", rig.db_path), [b'x'; 32]).unwrap();
        std::fs::write(format!("{}-shm", rig.db_path), [b'x'; 32]).unwrap();
        std::fs::remove_file(&rig.db_path).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap();
        assert_eq!(household_name().as_deref(), Some("Casa"));
        assert_eq!(std::fs::read(&pre).unwrap(), vec![b'p'; 128]);
        assert_eq!(
            std::fs::read(format!("{pre}-journal")).unwrap(),
            vec![b'j'; 64],
            "the kept generation's journal must stay paired with it"
        );
        assert!(
            !Path::new(&stray).exists(),
            "no journal may remain beside the restored database"
        );
        // Expected to pass before this session's change as well as after: the point is that
        // the branch keeps *deleting* orphans now that the other branch moves them.
        for suffix in ["-wal", "-shm"] {
            assert!(
                !Path::new(&format!("{}{suffix}", rig.db_path)).exists(),
                "no orphaned {suffix} may remain beside the restored database"
            );
            assert!(
                !Path::new(&format!("{pre}{suffix}")).exists(),
                "nor may an orphaned {suffix} be mispaired with the kept generation"
            );
        }
    }

    /// R0: a stale rollback journal beside the live database travels with it — left behind,
    /// SQLite would roll it back into the restored file and corrupt it.
    #[test]
    fn a_stale_journal_is_moved_aside_with_its_database() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        let journal = format!("{}-journal", rig.db_path);
        std::fs::write(&journal, [b'x'; 512]).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap();
        assert!(
            !Path::new(&journal).exists(),
            "no journal may remain beside the restored database"
        );
        assert!(
            Path::new(&format!("{}.pre-restore-journal", rig.db_path)).exists(),
            "the journal must be kept with the pre-restore file"
        );
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// Invariant 8, the other half of the generation: `-wal`/`-shm` beside the live file are
    /// *moved* with it, as `reset_database` moves them, not deleted. Nothing pins DELETE
    /// journal mode — `grep -rn journal_mode rust/` is empty — so a WAL beside the live
    /// database is possible, and deleting it would discard every committed page it holds
    /// from the generation the swap has just promised to keep.
    #[test]
    fn a_wal_and_shm_beside_the_live_file_are_kept_with_their_generation() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        std::fs::write(format!("{}-wal", rig.db_path), [b'w'; 96]).unwrap();
        std::fs::write(format!("{}-shm", rig.db_path), [b's'; 48]).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap();
        let pre = format!("{}.pre-restore", rig.db_path);
        assert_eq!(
            std::fs::read(format!("{pre}-wal")).unwrap(),
            vec![b'w'; 96],
            "the preserved generation keeps its WAL"
        );
        assert_eq!(std::fs::read(format!("{pre}-shm")).unwrap(), vec![b's'; 48]);
        for suffix in ["-wal", "-shm"] {
            assert!(
                !Path::new(&format!("{}{suffix}", rig.db_path)).exists(),
                "no {suffix} may remain beside the restored database"
            );
        }
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// Adversarial (R0): a restore with no live file that *fails* must not put the kept
    /// generation into the live slot. `commit_swap` no longer removes a `.pre-restore` it
    /// does not replace, so "`pre` exists" stopped meaning "the original moved there";
    /// `recover_original` may only put back what this restore actually took.
    #[test]
    fn a_failed_restore_with_no_live_file_keeps_the_earlier_generation() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        let pre = format!("{}.pre-restore", rig.db_path);
        std::fs::write(&pre, [b'p'; 128]).unwrap();
        std::fs::write(format!("{pre}-journal"), [b'j'; 64]).unwrap();
        std::fs::remove_file(&rig.db_path).unwrap();
        // A directory at the `-shm` sidecar: the no-live-file branch clears the three
        // sidecars strictly, so the swap fails inside that branch, with nothing staged aside.
        std::fs::create_dir(format!("{}-shm", rig.db_path)).unwrap();

        let err = restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        assert_eq!(
            std::fs::read(&pre).unwrap(),
            vec![b'p'; 128],
            "the kept generation must not be installed as the live database"
        );
        assert_eq!(
            std::fs::read(format!("{pre}-journal")).unwrap(),
            vec![b'j'; 64],
            "nor may its journal be consumed"
        );
        // Accepted and pinned rather than merely commented: with no original to put back,
        // the reopen creates a fresh database (`kimatta_storage::open` documents that), so
        // the caller is handed the original failure, not `preserved_aside_error`, and the
        // app is left in the state it was already in — an empty database at `db_path`.
        if let KimattaError::Storage { ref message } = err {
            assert!(
                !message.contains("moved back into place"),
                "want the original failure, got the preserved-aside message: {message:?}"
            );
        }
        assert_eq!(
            household_name(),
            None,
            "a fresh database, never the earlier generation"
        );
    }

    /// Adversarial (R0): a `{pre}-journal` the swap cannot clear belongs to the generation
    /// whose database the move-aside has just replaced. It must not travel back to the
    /// restored original — SQLite would roll it back into a database it never belonged to.
    /// Only the sidecars this restore moved are put back.
    #[test]
    fn a_foreign_pre_journal_is_not_carried_back_to_the_original() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        let pre = format!("{}.pre-restore", rig.db_path);
        // A directory the clear cannot remove, at the previous generation's journal. The
        // live file has no journal of its own (the connection closes cleanly first), so the
        // swap takes the remove arm and fails there — after `db_path` has moved onto `pre`.
        std::fs::create_dir(format!("{pre}-journal")).unwrap();
        std::fs::write(format!("{pre}-journal/occupied"), b"x").unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap_err();
        assert!(
            !Path::new(&format!("{}-journal", rig.db_path)).exists(),
            "the foreign journal must not follow the original back into the live slot"
        );
        assert_eq!(household_name().as_deref(), Some("Casa"));
    }

    /// R2: `staging` is a fixed path beside the live database, so two restores running
    /// together would copy over each other's staged file; the whole restore now runs under
    /// the slot mutex. Post-fix they are serialised by construction, so this is a
    /// liveness pin — the interleaving cannot be provoked deterministically, and the fix
    /// that removes it could just as easily deadlock.
    #[test]
    fn two_concurrent_restores_complete_without_deadlock() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();
        let second = format!("{}.second", rig.export_path);
        let h = bootstrap_household().unwrap();
        rename_household(h.id, Some("Otra".to_owned())).unwrap();
        export_database(second.clone()).unwrap();

        std::thread::scope(|s| {
            for export in [rig.export_path.clone(), second.clone()] {
                let db_path = rig.db_path.clone();
                s.spawn(move || restore_database(export, db_path).unwrap());
            }
        });
        assert!(
            matches!(household_name().as_deref(), Some("Casa") | Some("Otra")),
            "whichever committed last is the live content, and it is readable"
        );
        for suffix in ["", "-journal", "-wal", "-shm"] {
            assert!(
                !Path::new(&format!("{}.restore-staging{suffix}", rig.db_path)).exists(),
                "no staging artefact may outlive the restore"
            );
        }
    }

    /// The post-verify staging clear covers all three sidecars, strictly, mirroring the
    /// pre-copy clear. Expected to pass before and after the change: a sidecar planted by a
    /// test is caught by the pre-copy clear first, so the two sites cannot be told apart
    /// from outside — what this pins is that neither leaves anything behind.
    #[test]
    fn a_restore_leaves_no_staging_artefact_behind() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let rig = rig();
        seed(&rig, "Casa");
        export_database(rig.export_path.clone()).unwrap();

        restore_database(rig.export_path.clone(), rig.db_path.clone()).unwrap();
        for suffix in ["", "-journal", "-wal", "-shm"] {
            assert!(
                !Path::new(&format!("{}.restore-staging{suffix}", rig.db_path)).exists(),
                "the staging path{suffix} must not outlive the restore"
            );
        }
    }
}
