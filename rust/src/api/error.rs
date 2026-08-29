use thiserror::Error;

/// Bridge error surface. Variants are the Dart-matchable contract.
#[derive(Debug, Error)]
pub enum KimattaError {
    #[error("database path must not be empty")]
    InvalidPath,
    #[error("database is not open; call open_database first")]
    NotOpen,
    #[error("storage error: {message}")]
    Storage { message: String },
}

impl From<kimatta_storage::StorageError> for KimattaError {
    fn from(e: kimatta_storage::StorageError) -> Self {
        KimattaError::Storage {
            message: e.to_string(),
        }
    }
}

impl From<kimatta_storage::IdError> for KimattaError {
    fn from(e: kimatta_storage::IdError) -> Self {
        KimattaError::Storage {
            message: e.to_string(),
        }
    }
}
