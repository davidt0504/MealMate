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
    #[error("invalid planned meal input: {message}")]
    PlannedMeal { message: String },
    #[error("invalid shopping input: {message}")]
    Shopping { message: String },
}

/// The application layer's storage failures stay `Storage`, exactly as they would arriving
/// from any other command — a ghost household reads the same on every screen. There is no
/// planner-specific arm: `ApplicationError` has one variant and this `match` is exhaustive
/// without a catch-all, so nothing could construct one. Add it back with the first failure
/// that needs it, rather than freezing a speculative variant into the generated Dart.
impl From<kimatta_application::ApplicationError> for KimattaError {
    fn from(e: kimatta_application::ApplicationError) -> Self {
        match e {
            kimatta_application::ApplicationError::Storage(inner) => inner.into(),
        }
    }
}

/// Same scope rule as the `Planning` conversion below: a `ShoppingError` raised while
/// validating bridge arguments; one raised inside storage arrives as `StorageError::Shopping`
/// and stays `Storage`.
impl From<kimatta_storage::ShoppingError> for KimattaError {
    fn from(e: kimatta_storage::ShoppingError) -> Self {
        KimattaError::Shopping {
            message: e.to_string(),
        }
    }
}

/// Same scope rule as the `Planning` conversion below: a `PlannedMealError` raised while
/// parsing bridge arguments; one raised inside storage arrives as `StorageError::PlannedMeal`
/// and stays `Storage`.
impl From<kimatta_storage::PlannedMealError> for KimattaError {
    fn from(e: kimatta_storage::PlannedMealError) -> Self {
        KimattaError::PlannedMeal {
            message: e.to_string(),
        }
    }
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
