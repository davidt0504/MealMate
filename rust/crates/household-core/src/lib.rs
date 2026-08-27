//! Household identity kernel primitives (PRD v3 §7.1). Knows nothing about food.
#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdError {
    #[error("id must not be empty or whitespace")]
    Empty,
}

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// Accepts the id verbatim; rejects empty/whitespace, never trims.
            pub fn new(raw: impl Into<String>) -> Result<Self, IdError> {
                let raw = raw.into();
                if raw.trim().is_empty() {
                    Err(IdError::Empty)
                } else {
                    Ok(Self(raw))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

id_newtype!(HouseholdId);
id_newtype!(MemberId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Household {
    pub id: HouseholdId,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HouseholdMember {
    pub id: MemberId,
    pub household_id: HouseholdId,
    pub display_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_and_whitespace() {
        assert_eq!(HouseholdId::new("").unwrap_err(), IdError::Empty);
        assert_eq!(HouseholdId::new(" \t\n").unwrap_err(), IdError::Empty);
        assert_eq!(MemberId::new("").unwrap_err(), IdError::Empty);
    }

    #[test]
    fn round_trips_without_trimming() {
        assert_eq!(HouseholdId::new("h-1").unwrap().as_str(), "h-1");
        assert_eq!(HouseholdId::new(" h-1 ").unwrap().as_str(), " h-1 ");
        assert_eq!(MemberId::new(" m-1 ").unwrap().as_str(), " m-1 ");
    }
}
