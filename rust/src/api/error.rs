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
    #[error("invalid planning input: {message}")]
    Planning { message: String },
    #[error("invalid recipe input: {message}")]
    Recipe { message: String },
    #[error("invalid restriction input: {message}")]
    Restriction { message: String },
}

/// Same scope rule as the `Planning` conversion below: a `RestrictionError` raised while
/// parsing bridge arguments; one raised inside storage arrives as `StorageError::Restriction`
/// and stays `Storage`.
impl From<kimatta_storage::RestrictionError> for KimattaError {
    fn from(e: kimatta_storage::RestrictionError) -> Self {
        KimattaError::Restriction {
            message: e.to_string(),
        }
    }
}

/// Same scope rule as the `Planning` conversion below: a `RecipeError` raised while parsing
/// bridge arguments; one raised inside storage arrives as `StorageError::Recipe` and stays
/// `Storage`.
impl From<kimatta_storage::RecipeError> for KimattaError {
    fn from(e: kimatta_storage::RecipeError) -> Self {
        KimattaError::Recipe {
            message: e.to_string(),
        }
    }
}

impl From<kimatta_storage::StorageError> for KimattaError {
    fn from(e: kimatta_storage::StorageError) -> Self {
        KimattaError::Storage {
            message: e.to_string(),
        }
    }
}

/// Applies only where a `PlanningError` is produced *before* reaching storage — parsing and
/// validating bridge arguments. A `PlanningError` surfacing from inside storage arrives
/// wrapped as `StorageError::Planning` and stays a `Storage` error, which is correct: by then
/// the failure is a persistence-layer failure.
impl From<kimatta_storage::PlanningError> for KimattaError {
    fn from(e: kimatta_storage::PlanningError) -> Self {
        KimattaError::Planning {
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
