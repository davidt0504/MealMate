use thiserror::Error;

/// Bridge error surface. Variants are the Dart-matchable contract.
#[derive(Debug, Error)]
pub enum KimattaError {
    #[error("database path must not be empty")]
    InvalidPath,
    #[error("storage error: {message}")]
    Storage { message: String },
}

pub struct HealthReport {
    pub db_path: String,
    pub schema_version: u32,
}

#[flutter_rust_bridge::frb(sync)]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl From<kimatta_storage::StorageError> for KimattaError {
    fn from(e: kimatta_storage::StorageError) -> Self {
        KimattaError::Storage {
            message: e.to_string(),
        }
    }
}

/// Opens the SQLite database at `db_path` (creating it), applies migrations, and reports the
/// resulting schema version. Empty/whitespace paths are rejected before touching storage.
pub fn health_check(db_path: String) -> Result<HealthReport, KimattaError> {
    if db_path.trim().is_empty() {
        return Err(KimattaError::InvalidPath);
    }
    let conn = kimatta_storage::open(&db_path)?;
    let schema_version = kimatta_storage::schema_version(&conn)?;
    Ok(HealthReport {
        db_path,
        schema_version,
    })
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
