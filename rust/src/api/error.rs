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
    /// The database file itself is damaged (corrupt header, truncated file, page-level
    /// damage). Its own variant, not `Storage`, because the recovery differs: the UI may
    /// offer "start fresh" for corruption, which would silently lose data if shown for,
    /// say, a disk-full `Storage` failure.
    #[error("the local database is damaged: {message}")]
    Corrupt { message: String },
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
/// from any other command — a ghost household reads the same on every screen. The decision
/// refusals are the failures the earlier note here was waiting for, and they land on the
/// existing `Planning` variant rather than new ones: they are refusals of an out-of-contract
/// call, and no screen has a different recovery for them, so nothing is frozen into the
/// generated Dart. Their messages are written here, not forwarded from the variants'
/// `Display` — `describeFailure` renders `Planning`'s message verbatim, so the user-facing
/// sentence must be prose, while the `Display` keeps the addressing detail a caller debugging
/// the refusal needs. `SwapOntoLockedSlot` is why the locked case is its own variant rather
/// than a `StorageError`: forwarded, it put a raw meal id and the word "automation" on the
/// Cover screen for a `WriteSource::User` write.
impl From<kimatta_application::ApplicationError> for KimattaError {
    fn from(e: kimatta_application::ApplicationError) -> Self {
        match e {
            kimatta_application::ApplicationError::Storage(inner) => inner.into(),
            kimatta_application::ApplicationError::UnmatchableVetoSubject => {
                KimattaError::Planning {
                    // "we can match" rather than a bare "must have a name": the subject is the
                    // dish's own title, which the household has just read in the confirm
                    // dialog, so telling them it has no name contradicts the screen.
                    message: "a meal to never suggest must have a name we can match".to_owned(),
                }
            }
            kimatta_application::ApplicationError::SwapOntoLockedSlot { .. } => {
                KimattaError::Planning {
                    message: "that meal is locked, so it cannot be swapped".to_owned(),
                }
            }
            kimatta_application::ApplicationError::DecisionOutsideWindow { .. } => {
                KimattaError::Planning {
                    message: "that decision is for a day outside the week being planned".to_owned(),
                }
            }
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

/// `CorruptDatabase` gets its own bridge variant so the UI can offer corruption-specific
/// recovery; every other storage failure — `NewerSchema` included, whose recovery is "update
/// the app" and whose `Display` is already user prose — stays `Storage`.
impl From<kimatta_storage::StorageError> for KimattaError {
    fn from(e: kimatta_storage::StorageError) -> Self {
        match e {
            kimatta_storage::StorageError::CorruptDatabase { .. } => KimattaError::Corrupt {
                message: e.to_string(),
            },
            other => KimattaError::Storage {
                message: other.to_string(),
            },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_corrupt_database_maps_to_the_corrupt_variant() {
        let err: KimattaError = kimatta_storage::StorageError::CorruptDatabase {
            detail: "file is not a database".to_owned(),
        }
        .into();
        assert!(
            matches!(err, KimattaError::Corrupt { .. }),
            "want Corrupt, got {err:?}"
        );
    }

    /// Expected-to-pass pin: `NewerSchema` stays `Storage` — its recovery is "update the
    /// app", not start-fresh, and its `Display` is already user prose that `describeFailure`
    /// renders verbatim.
    #[test]
    fn a_newer_schema_stays_a_storage_error_with_its_prose() {
        let err: KimattaError = kimatta_storage::StorageError::NewerSchema {
            found: 11,
            supported: 10,
        }
        .into();
        match err {
            KimattaError::Storage { ref message } => {
                assert!(
                    message.contains("newer version of the app"),
                    "got {message:?}"
                );
            }
            other => panic!("want Storage, got {other:?}"),
        }
    }
}
