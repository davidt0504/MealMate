//! Household dietary restrictions (PRD v3 §8, invariant 10). A restriction is a warning
//! surface, never a safety claim: `MVP-009` matches on the known tokens and reports only what
//! it knows about, and `Other` keeps what the vocabulary does not cover verbatim rather than
//! dropping or guessing it — the same shape as `Unit::Known | Other`.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RestrictionError {
    #[error("{field} must not be empty or whitespace")]
    Empty { field: &'static str },
    #[error("unknown restriction kind {0:?}")]
    UnknownKind(String),
}

/// The closed vocabulary `MVP-009` can attach match rules to. The first nine are the FDA
/// major allergens (milk → `dairy`, wheat → `gluten`); the last two are the diet constraints
/// that behave as hard filters. Allergen-vs-diet is deliberately not modelled here —
/// `MVP-009` branches on the token when it has an actual matching rule to attach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestrictionKind {
    Peanuts,
    TreeNuts,
    Dairy,
    Eggs,
    Gluten,
    Soy,
    Fish,
    Shellfish,
    Sesame,
    Vegetarian,
    Vegan,
}

impl RestrictionKind {
    pub const ALL: [Self; 11] = [
        Self::Peanuts,
        Self::TreeNuts,
        Self::Dairy,
        Self::Eggs,
        Self::Gluten,
        Self::Soy,
        Self::Fish,
        Self::Shellfish,
        Self::Sesame,
        Self::Vegetarian,
        Self::Vegan,
    ];

    /// snake_case, as `MealSlot::as_str` and `UnitKind::as_str` are. This token is what is
    /// persisted and what crosses the bridge; no other spelling is stored anywhere.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Peanuts => "peanuts",
            Self::TreeNuts => "tree_nuts",
            Self::Dairy => "dairy",
            Self::Eggs => "eggs",
            Self::Gluten => "gluten",
            Self::Soy => "soy",
            Self::Fish => "fish",
            Self::Shellfish => "shellfish",
            Self::Sesame => "sesame",
            Self::Vegetarian => "vegetarian",
            Self::Vegan => "vegan",
        }
    }

    /// Matches exactly and does not trim, as `MealSlot::parse` and `UnitKind::parse` do.
    pub fn parse(raw: &str) -> Result<Self, RestrictionError> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == raw)
            .ok_or_else(|| RestrictionError::UnknownKind(raw.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Restriction {
    Known(RestrictionKind),
    Other(String),
}

impl Restriction {
    /// Trims, and rejects blank: a restriction with no text warns about nothing, and storing
    /// one would put a row on screen that the user cannot interpret or act on.
    pub fn other(text: impl Into<String>) -> Result<Self, RestrictionError> {
        let text = text.into();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(RestrictionError::Empty {
                field: "restriction text",
            });
        }
        Ok(Self::Other(trimmed.to_owned()))
    }
}

/// `Other` is compared on its trimmed text, so two entries differing only by surrounding
/// space collapse whichever construction path they arrived by — the variant is public, so
/// `Restriction::other`'s own trimming is not the only way one can be built.
fn is_same(a: &Restriction, b: &Restriction) -> bool {
    match (a, b) {
        (Restriction::Known(x), Restriction::Known(y)) => x == y,
        (Restriction::Other(x), Restriction::Other(y)) => x.trim() == y.trim(),
        _ => false,
    }
}

/// A household's restriction set, in first-seen order and deduplicated. Order is preserved
/// rather than canonicalised because the list is shown back to the user as they entered it;
/// the empty set is legal, and is the state every household starts in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HouseholdRestrictions(Vec<Restriction>);

impl HouseholdRestrictions {
    pub fn new(items: impl IntoIterator<Item = Restriction>) -> Self {
        let mut kept: Vec<Restriction> = Vec::new();
        for item in items {
            if !kept.iter().any(|seen| is_same(seen, &item)) {
                kept.push(item);
            }
        }
        Self(kept)
    }

    pub fn restrictions(&self) -> &[Restriction] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hard-coded rather than mapped from `ALL`: a test that mirrors the production
    /// constant cannot catch an addition to it.
    #[test]
    fn every_kind_round_trips_its_token() {
        assert!(!RestrictionKind::ALL.is_empty());
        for kind in RestrictionKind::ALL {
            assert_eq!(RestrictionKind::parse(kind.as_str()).unwrap(), kind);
        }
        assert_eq!(
            RestrictionKind::ALL.map(RestrictionKind::as_str),
            [
                "peanuts",
                "tree_nuts",
                "dairy",
                "eggs",
                "gluten",
                "soy",
                "fish",
                "shellfish",
                "sesame",
                "vegetarian",
                "vegan",
            ]
        );
    }

    #[test]
    fn an_unknown_token_is_rejected() {
        for raw in ["", "nightshades", "Peanuts", "tree nuts", " peanuts"] {
            assert_eq!(
                RestrictionKind::parse(raw).unwrap_err(),
                RestrictionError::UnknownKind(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn a_blank_other_is_rejected() {
        for raw in ["", "   ", "\t\n"] {
            assert_eq!(
                Restriction::other(raw).unwrap_err(),
                RestrictionError::Empty {
                    field: "restriction text"
                },
                "{raw:?} must not be a restriction"
            );
        }
    }

    #[test]
    fn other_text_is_trimmed() {
        assert_eq!(
            Restriction::other("  nightshades  ").unwrap(),
            Restriction::Other("nightshades".to_owned())
        );
    }

    #[test]
    fn the_set_keeps_first_seen_order_and_collapses_duplicates() {
        let set = HouseholdRestrictions::new([
            Restriction::Known(RestrictionKind::Dairy),
            Restriction::other("nightshades").unwrap(),
            Restriction::Known(RestrictionKind::Dairy),
            // Same text, different surrounding space: still one entry.
            Restriction::Other("  nightshades  ".to_owned()),
            Restriction::Known(RestrictionKind::Peanuts),
        ]);
        assert_eq!(
            set.restrictions(),
            [
                Restriction::Known(RestrictionKind::Dairy),
                Restriction::Other("nightshades".to_owned()),
                Restriction::Known(RestrictionKind::Peanuts),
            ]
        );
    }

    /// A household that restricts nothing is the default state, not an error.
    #[test]
    fn the_empty_set_is_legal() {
        let set = HouseholdRestrictions::new([]);
        assert!(set.restrictions().is_empty());
        assert_eq!(set, HouseholdRestrictions::default());
    }
}
