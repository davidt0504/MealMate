//! The one process-wide SQLite connection, installed by `open_database` and borrowed by every
//! later bridge call — so migrations run once and FRB's worker threads serialise on one handle.
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use kimatta_storage::Connection;

use crate::api::error::KimattaError;

static DB: Mutex<Option<Connection>> = Mutex::new(None);

/// The database session (OPT-007 §5 identity 4): changes whenever the connection slot is
/// installed or swapped — open, restore, reset — and is read and compared only under the `DB`
/// mutex, so a command queued behind a restore sees the new session and is refused instead of
/// running against the restored file. The process nonce keeps a token from an earlier run of
/// the app from ever matching; no persistent schema token could, since a restored snapshot
/// carries the old one with it.
static SESSION: AtomicU64 = AtomicU64::new(0);
static PROCESS: OnceLock<String> = OnceLock::new();

fn session_token(counter: u64) -> String {
    let nonce = PROCESS.get_or_init(|| uuid::Uuid::new_v4().simple().to_string());
    format!("{nonce}:{counter}")
}

fn lock() -> MutexGuard<'static, Option<Connection>> {
    // A poisoned lock means a bridge call panicked mid-closure; storage functions hold their
    // own transactions, which roll back on drop, so the connection is still usable.
    DB.lock().unwrap_or_else(|e| e.into_inner())
}

/// Installs `conn` as the process-wide connection, replacing any previous one.
pub(crate) fn install(conn: Connection) {
    let mut guard = lock();
    *guard = Some(conn);
    SESSION.fetch_add(1, Ordering::SeqCst);
}

/// [`with`], refused with `SessionChanged` unless `token` is the current session: the caller's
/// view came from a database that has since been replaced.
pub(crate) fn with_session<T>(
    token: &str,
    f: impl FnOnce(&mut Connection, String) -> Result<T, KimattaError>,
) -> Result<T, KimattaError> {
    let mut guard = lock();
    let current = session_token(SESSION.load(Ordering::SeqCst));
    if token != current {
        return Err(KimattaError::Draft {
            kind: crate::api::error::DraftErrorKind::SessionChanged,
            message: "the data was replaced since this screen loaded; reopen it".to_owned(),
        });
    }
    let conn = guard.as_mut().ok_or(KimattaError::NotOpen)?;
    f(conn, current)
}

/// [`with`], also handing `f` the current session token.
pub(crate) fn with_current_session<T>(
    f: impl FnOnce(&mut Connection, String) -> Result<T, KimattaError>,
) -> Result<T, KimattaError> {
    let mut guard = lock();
    let current = session_token(SESSION.load(Ordering::SeqCst));
    let conn = guard.as_mut().ok_or(KimattaError::NotOpen)?;
    f(conn, current)
}

/// Runs `f` against the installed connection; `NotOpen` if `open_database` has not run.
pub(crate) fn with<T>(
    f: impl FnOnce(&mut Connection) -> Result<T, KimattaError>,
) -> Result<T, KimattaError> {
    let mut guard = lock();
    let conn = guard.as_mut().ok_or(KimattaError::NotOpen)?;
    f(conn)
}

/// Runs `f` with exclusive ownership of the whole connection slot, holding the mutex for the
/// duration — restore swaps the database file underneath the connection (MVP-017), and no
/// bridge call may observe the slot mid-swap. `f` may leave the slot `None`; every later
/// `with` then reports `NotOpen` honestly.
pub(crate) fn swap<T>(
    f: impl FnOnce(&mut Option<Connection>) -> Result<T, KimattaError>,
) -> Result<T, KimattaError> {
    let mut guard = lock();
    // Bumped whatever `f` does: a failed restore may still have closed and reopened the slot,
    // and refusing a queued command costs a reload, while running it on the wrong file costs
    // data.
    SESSION.fetch_add(1, Ordering::SeqCst);
    f(&mut guard)
}

/// Serialises the tests that touch the process-wide `DB` slot (this module's and
/// `api::health`'s restore/reset tests): they install and clear real connections, so running
/// them concurrently would swap each other's databases mid-assertion.
#[cfg(test)]
pub(crate) static TEST_DB_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_no_connection_is_not_open() {
        // Under TEST_DB_LOCK because other tests install a connection; the slot is cleared
        // here rather than assumed empty.
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        swap(|slot| {
            *slot = None;
            Ok(())
        })
        .unwrap();
        let err = with(|_| Ok(())).unwrap_err();
        assert!(matches!(err, KimattaError::NotOpen));
    }
}
