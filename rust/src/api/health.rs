use crate::api::error::KimattaError;

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

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
