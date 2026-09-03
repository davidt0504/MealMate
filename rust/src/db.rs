//! The one process-wide SQLite connection, installed by `open_database` and borrowed by every
//! later bridge call — so migrations run once and FRB's worker threads serialise on one handle.
use std::sync::{Mutex, MutexGuard};

use kimatta_storage::Connection;

use crate::api::error::KimattaError;

static DB: Mutex<Option<Connection>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, Option<Connection>> {
    // A poisoned lock means a bridge call panicked mid-closure; storage functions hold their
    // own transactions, which roll back on drop, so the connection is still usable.
    DB.lock().unwrap_or_else(|e| e.into_inner())
}

/// Installs `conn` as the process-wide connection, replacing any previous one.
pub(crate) fn install(conn: Connection) {
    *lock() = Some(conn);
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
