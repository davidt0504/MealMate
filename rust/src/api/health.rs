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

/// Stub until MVP-002 opens SQLite here; rejects empty/whitespace paths so the
/// typed-error path is exercisable before storage exists.
pub fn health_check(db_path: String) -> Result<HealthReport, KimattaError> {
    let mut result = Err(KimattaError::InvalidPath);
    if !db_path.trim().is_empty() {
        result = Ok(HealthReport { db_path, schema_version: 0 });
    }
    result
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
