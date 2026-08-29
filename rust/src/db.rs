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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_no_connection_is_not_open() {
        // The only test in this crate that touches DB, so it never sees an installed connection.
        let err = with(|_| Ok(())).unwrap_err();
        assert!(matches!(err, KimattaError::NotOpen));
    }
}
