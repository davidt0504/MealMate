//! Recipe and ingredient entities (PRD v3 §8). Original text is kept beside every parsed
//! field; uncertainty is a value, never a default.

use std::cmp::Ordering;
use std::fmt;

use household_core::{HouseholdId, IdError};
use thiserror::Error;

use crate::{parse_civil_date, CivilDate};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecipeError {
    #[error("{field} must not be empty or whitespace")]
    Empty { field: &'static str },
    #[error("servings must be at least 1")]
    ZeroServings,
    #[error("a quantity denominator must not be zero")]
    ZeroDenominator,
    #[error("a quantity must be positive; an absent amount is Quantity::Unknown")]
    ZeroQuantity,
    #[error("range minimum {min} exceeds maximum {max}")]
    InvertedRange { min: String, max: String },
    #[error("unknown unit {0:?}")]
    UnknownUnit(String),
    #[error("unknown provenance kind {0:?}")]
    UnknownProvenanceKind(String),
    #[error("unknown quantity kind {0:?}")]
    UnknownQuantityKind(String),
    #[error("unknown rights basis {0:?}")]
    UnknownRightsBasis(String),
    #[error("{0:?} is not a civil date in YYYY-MM-DD form")]
    InvalidVerifiedOn(String),
    #[error("prep minutes must be at least 1")]
    ZeroPrepMinutes,
}

// Own copy of `household-core`'s macro: it is private there, and exporting a macro across
// the kernel boundary for three ids is more coupling than fifteen duplicated lines. The
// error type is shared so there is one id-error vocabulary across crates.
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

id_newtype!(IngredientId);
id_newtype!(CustomIngredientId);
id_newtype!(RecipeId);

fn non_blank(value: String, field: &'static str) -> Result<String, RecipeError> {
    if value.trim().is_empty() {
        Err(RecipeError::Empty { field })
    } else {
        Ok(value)
    }
}

/// Catalog ingredient — global, not household-owned; seeded by MVP-011, matched by MVP-009.
/// Private fields + `new`, like `Recipe`: a blank name is a domain error (`Empty{field}`), so
/// it reaches Dart as `KimattaError::Recipe` the same way a blank title does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ingredient {
    id: IngredientId,
    canonical_name: String,
    aliases: Vec<String>,
    store_category: Option<String>,
}

impl Ingredient {
    /// Aliases are held sorted and deduplicated so equal alias sets compare equal.
    pub fn new(
        id: IngredientId,
        canonical_name: impl Into<String>,
        aliases: Vec<String>,
        store_category: Option<String>,
    ) -> Result<Self, RecipeError> {
        let canonical_name = non_blank(canonical_name.into(), "canonical_name")?;
        let mut aliases = aliases
            .into_iter()
            .map(|a| non_blank(a, "alias"))
            .collect::<Result<Vec<_>, _>>()?;
        aliases.sort_unstable();
        aliases.dedup();
        Ok(Self {
            id,
            canonical_name,
            aliases,
            store_category: store_category
                .map(|c| non_blank(c, "store_category"))
                .transpose()?,
        })
    }

    pub fn id(&self) -> &IngredientId {
        &self.id
    }

    pub fn canonical_name(&self) -> &str {
        &self.canonical_name
    }

    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    pub fn store_category(&self) -> Option<&str> {
        self.store_category.as_deref()
    }
}

/// A household's own ingredient when the catalog has no match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomIngredient {
    id: CustomIngredientId,
    household_id: HouseholdId,
    name: String,
    store_category: Option<String>,
}

impl CustomIngredient {
    pub fn new(
        id: CustomIngredientId,
        household_id: HouseholdId,
        name: impl Into<String>,
        store_category: Option<String>,
    ) -> Result<Self, RecipeError> {
        Ok(Self {
            id,
            household_id,
            name: non_blank(name.into(), "name")?,
            store_category: store_category
                .map(|c| non_blank(c, "store_category"))
                .transpose()?,
        })
    }

    pub fn id(&self) -> &CustomIngredientId {
        &self.id
    }

    pub fn household_id(&self) -> &HouseholdId {
        &self.household_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn store_category(&self) -> Option<&str> {
        self.store_category.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvenanceKind {
    Authored,
    Imported,
    Starter,
}

impl ProvenanceKind {
    pub const ALL: [Self; 3] = [Self::Authored, Self::Imported, Self::Starter];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Authored => "authored",
            Self::Imported => "imported",
            Self::Starter => "starter",
        }
    }

    /// Matches exactly and does not trim, as `MealSlot::parse` does.
    pub fn parse(raw: &str) -> Result<Self, RecipeError> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == raw)
            .ok_or_else(|| RecipeError::UnknownProvenanceKind(raw.to_owned()))
    }
}

/// Where a recipe came from; the rights boundary MVP-020/DEC-003 build on (invariant 12).
/// Private fields + `new`, like `Recipe`: a whitespace-only source is rejected rather than
/// stored as a *present* attribution, since MVP-011/MVP-020 read presence as the rights signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeProvenance {
    kind: ProvenanceKind,
    source_url: Option<String>,
    source_name: Option<String>,
    source_author: Option<String>,
    rights: Option<RecipeRights>,
    starter_slug: Option<String>,
}

impl RecipeProvenance {
    /// Carries no rights and no starter slug: this is the shape every user-authored recipe
    /// takes, and the edit path cannot supply rights it was never shown.
    pub fn new(
        kind: ProvenanceKind,
        source_url: Option<String>,
        source_name: Option<String>,
        source_author: Option<String>,
    ) -> Result<Self, RecipeError> {
        Ok(Self {
            kind,
            source_url: source_url.map(|s| non_blank(s, "source_url")).transpose()?,
            source_name: source_name
                .map(|s| non_blank(s, "source_name"))
                .transpose()?,
            source_author: source_author
                .map(|s| non_blank(s, "source_author"))
                .transpose()?,
            rights: None,
            starter_slug: None,
        })
    }

    /// The MVP-011 seeding shape: the same four source fields plus the rights record and the
    /// content slug that makes a re-install idempotent.
    pub fn with_rights(
        kind: ProvenanceKind,
        source_url: Option<String>,
        source_name: Option<String>,
        source_author: Option<String>,
        rights: Option<RecipeRights>,
        starter_slug: Option<String>,
    ) -> Result<Self, RecipeError> {
        Ok(Self {
            rights,
            starter_slug: starter_slug
                .map(|s| non_blank(s, "starter_slug"))
                .transpose()?,
            ..Self::new(kind, source_url, source_name, source_author)?
        })
    }

    pub fn kind(&self) -> ProvenanceKind {
        self.kind
    }

    pub fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }

    pub fn source_name(&self) -> Option<&str> {
        self.source_name.as_deref()
    }

    pub fn source_author(&self) -> Option<&str> {
        self.source_author.as_deref()
    }

    pub fn rights(&self) -> Option<&RecipeRights> {
        self.rights.as_ref()
    }

    pub fn starter_slug(&self) -> Option<&str> {
        self.starter_slug.as_deref()
    }
}

/// The rights bases this project will ship. `CcBy` is deliberately absent: MVP-011 admits it only
/// if attribution survives every relevant surface, and that surface is MVP-020's unbuilt work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightsBasis {
    Original,
    UsFederalPublicDomain,
    Cc0,
}

impl RightsBasis {
    pub const ALL: [Self; 3] = [Self::Original, Self::UsFederalPublicDomain, Self::Cc0];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::UsFederalPublicDomain => "us_federal_public_domain",
            Self::Cc0 => "cc0",
        }
    }

    /// Matches exactly and does not trim, as `ProvenanceKind::parse` does.
    pub fn parse(raw: &str) -> Result<Self, RecipeError> {
        Self::ALL
            .into_iter()
            .find(|basis| basis.as_str() == raw)
            .ok_or_else(|| RecipeError::UnknownRightsBasis(raw.to_owned()))
    }
}

/// The rights record MVP-011's manifest carries and MVP-020 will read: what allows this text to
/// ship, who must be credited, what was changed, and when that was checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeRights {
    basis: RightsBasis,
    attribution: Option<String>,
    modifications: Option<String>,
    verified_on: CivilDate,
}

impl RecipeRights {
    /// `parse_civil_date` returns `PlanningError`, which this module has no `From` impl for, so
    /// the failure is mapped into `recipe.rs`'s own error vocabulary rather than importing a
    /// second one.
    pub fn new(
        basis: RightsBasis,
        attribution: Option<String>,
        modifications: Option<String>,
        verified_on: &str,
    ) -> Result<Self, RecipeError> {
        Ok(Self {
            basis,
            attribution: attribution
                .map(|a| non_blank(a, "attribution"))
                .transpose()?,
            modifications: modifications
                .map(|m| non_blank(m, "modifications"))
                .transpose()?,
            verified_on: parse_civil_date(verified_on)
                .map_err(|_| RecipeError::InvalidVerifiedOn(verified_on.to_owned()))?,
        })
    }

    pub fn basis(&self) -> RightsBasis {
        self.basis
    }

    pub fn attribution(&self) -> Option<&str> {
        self.attribution.as_deref()
    }

    pub fn modifications(&self) -> Option<&str> {
        self.modifications.as_deref()
    }

    pub fn verified_on(&self) -> CivilDate {
        self.verified_on
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Exact positive rational, gcd-normalised so equal values compare equal. Fields are private:
/// `new` is the only way in, so `2/4` cannot exist beside `1/2` and `denom` is never zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    numer: u32,
    denom: u32,
}

impl Rational {
    pub fn new(numer: u32, denom: u32) -> Result<Self, RecipeError> {
        if denom == 0 {
            return Err(RecipeError::ZeroDenominator);
        }
        if numer == 0 {
            return Err(RecipeError::ZeroQuantity);
        }
        let g = gcd(numer, denom);
        Ok(Self {
            numer: numer / g,
            denom: denom / g,
        })
    }

    pub fn numer(self) -> u32 {
        self.numer
    }

    pub fn denom(self) -> u32 {
        self.denom
    }

    /// `None` when the exact result does not fit `u32` after normalisation. Every intermediate
    /// step is a checked `u64` operation: two `u32` products each fit `u64`, but their *sum*
    /// can exceed it, so the addition is checked too — never a wrap, never a panic.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        let lhs = u64::from(self.numer).checked_mul(u64::from(other.denom))?;
        let rhs = u64::from(other.numer).checked_mul(u64::from(self.denom))?;
        let numer = lhs.checked_add(rhs)?;
        let denom = u64::from(self.denom).checked_mul(u64::from(other.denom))?;
        Self::from_u64(numer, denom)
    }

    pub fn checked_mul(self, other: Self) -> Option<Self> {
        Self::from_u64(
            u64::from(self.numer) * u64::from(other.numer),
            u64::from(self.denom) * u64::from(other.denom),
        )
    }

    fn from_u64(numer: u64, denom: u64) -> Option<Self> {
        let g = gcd_u64(numer, denom);
        let (numer, denom) = (numer / g, denom / g);
        Some(Self {
            numer: u32::try_from(numer).ok()?,
            denom: u32::try_from(denom).ok()?,
        })
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numer, self.denom)
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    /// Cross-multiplies in `u64`: two `u32` products cannot overflow there.
    fn cmp(&self, other: &Self) -> Ordering {
        let lhs = u64::from(self.numer) * u64::from(other.denom);
        let rhs = u64::from(other.numer) * u64::from(self.denom);
        lhs.cmp(&rhs)
    }
}

/// A closed range with `min <= max` by construction; private fields, `new` is the only
/// constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuantityRange {
    min: Rational,
    max: Rational,
}

impl QuantityRange {
    pub fn new(min: Rational, max: Rational) -> Result<Self, RecipeError> {
        if min > max {
            return Err(RecipeError::InvertedRange {
                min: min.to_string(),
                max: max.to_string(),
            });
        }
        Ok(Self { min, max })
    }

    pub fn min(self) -> Rational {
        self.min
    }

    pub fn max(self) -> Rational {
        self.max
    }
}

/// What the user entered about amount. `Unknown` is the honest value when no amount was
/// given — zero is rejected (`ZeroQuantity`) because "0 cups" is fabricated certainty, not an
/// absent amount. Open-ended ranges ("2+ cups") and zero-lower-bound ranges ("0–2 cups") have
/// no representation in this card: both are stored as `Unknown` with the original text
/// intact, and are named as deferred cases in the handoff row so the parser card does not
/// rediscover them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantity {
    Unknown,
    Exact(Rational),
    Range(QuantityRange),
}

impl Quantity {
    pub fn kind_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Exact(_) => "exact",
            Self::Range(_) => "range",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    Teaspoon,
    Tablespoon,
    Cup,
    FluidOunce,
    Milliliter,
    Liter,
    Gram,
    Kilogram,
    Ounce,
    Pound,
    Piece,
}

impl UnitKind {
    pub const ALL: [Self; 11] = [
        Self::Teaspoon,
        Self::Tablespoon,
        Self::Cup,
        Self::FluidOunce,
        Self::Milliliter,
        Self::Liter,
        Self::Gram,
        Self::Kilogram,
        Self::Ounce,
        Self::Pound,
        Self::Piece,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Teaspoon => "tsp",
            Self::Tablespoon => "tbsp",
            Self::Cup => "cup",
            Self::FluidOunce => "fl_oz",
            Self::Milliliter => "ml",
            Self::Liter => "l",
            Self::Gram => "g",
            Self::Kilogram => "kg",
            Self::Ounce => "oz",
            Self::Pound => "lb",
            Self::Piece => "piece",
        }
    }

    /// Matches exactly and does not trim, as `MealSlot::parse` does.
    pub fn parse(raw: &str) -> Result<Self, RecipeError> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == raw)
            .ok_or_else(|| RecipeError::UnknownUnit(raw.to_owned()))
    }
}

/// `Other` keeps an unrecognised unit verbatim rather than dropping or guessing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unit {
    None,
    Known(UnitKind),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngredientRef {
    Catalog(IngredientId),
    Custom(CustomIngredientId),
}

/// One ingredient line. `original_text` is the whole line as entered and is never derived;
/// `name` is the ingredient as entered, so display never depends on `ingredient` resolving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngredientLine {
    original_text: String,
    name: String,
    ingredient: Option<IngredientRef>,
    quantity: Quantity,
    unit: Unit,
    preparation: Option<String>,
    optional: bool,
}

impl IngredientLine {
    /// Nothing is trimmed: silent normalisation is MVP-008's stop condition. A present but
    /// whitespace-only `preparation` is rejected rather than mapped to `None`, for the same
    /// reason. `Unit::Other`'s text is held to the same rule, so `Unit::None` stays the one
    /// encoding of "this line has no unit" that MVP-015's aggregation has to handle.
    pub fn new(
        original_text: impl Into<String>,
        name: impl Into<String>,
        ingredient: Option<IngredientRef>,
        quantity: Quantity,
        unit: Unit,
        preparation: Option<String>,
        optional: bool,
    ) -> Result<Self, RecipeError> {
        let unit = match unit {
            Unit::Other(text) => Unit::Other(non_blank(text, "unit")?),
            unit => unit,
        };
        Ok(Self {
            original_text: non_blank(original_text.into(), "original_text")?,
            name: non_blank(name.into(), "name")?,
            ingredient,
            quantity,
            unit,
            preparation: preparation
                .map(|p| non_blank(p, "preparation"))
                .transpose()?,
            optional,
        })
    }

    pub fn original_text(&self) -> &str {
        &self.original_text
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ingredient(&self) -> Option<&IngredientRef> {
        self.ingredient.as_ref()
    }

    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    pub fn preparation(&self) -> Option<&str> {
        self.preparation.as_deref()
    }

    pub fn optional(&self) -> bool {
        self.optional
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    id: RecipeId,
    household_id: HouseholdId,
    title: String,
    servings: Option<u32>,
    prep_minutes: Option<u32>,
    instructions: String,
    lines: Vec<IngredientLine>,
    provenance: RecipeProvenance,
}

impl Recipe {
    /// `instructions` may be empty and `lines` may be empty: a stub recipe is legitimate.
    /// Any "at least one line" rule belongs to a UI, not the store. `prep_minutes` is
    /// `None` when no estimate was given — never `Some(0)`, which is fabricated certainty
    /// rather than an absent estimate, exactly as `servings` treats zero.
    ///
    /// Eight arguments, one per field. The lint wants a builder or a params struct, but this
    /// is a whole-aggregate constructor whose parameter list *is* the struct: every field is
    /// required, none has a sensible default, and a params struct would just be `Recipe` with
    /// the validation removed — which is the invariant the private fields exist to hold. This
    /// is the crate's first `allow`; grouping the fields is a refactor for whoever next adds
    /// one, not a change this card should make across 11 call sites.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RecipeId,
        household_id: HouseholdId,
        title: impl Into<String>,
        servings: Option<u32>,
        prep_minutes: Option<u32>,
        instructions: impl Into<String>,
        lines: Vec<IngredientLine>,
        provenance: RecipeProvenance,
    ) -> Result<Self, RecipeError> {
        if servings == Some(0) {
            return Err(RecipeError::ZeroServings);
        }
        if prep_minutes == Some(0) {
            return Err(RecipeError::ZeroPrepMinutes);
        }
        Ok(Self {
            id,
            household_id,
            title: non_blank(title.into(), "title")?,
            servings,
            prep_minutes,
            instructions: instructions.into(),
            lines,
            provenance,
        })
    }

    pub fn id(&self) -> &RecipeId {
        &self.id
    }

    pub fn household_id(&self) -> &HouseholdId {
        &self.household_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn servings(&self) -> Option<u32> {
        self.servings
    }

    pub fn prep_minutes(&self) -> Option<u32> {
        self.prep_minutes
    }

    pub fn instructions(&self) -> &str {
        &self.instructions
    }

    pub fn lines(&self) -> &[IngredientLine] {
        &self.lines
    }

    pub fn provenance(&self) -> &RecipeProvenance {
        &self.provenance
    }
}

#[cfg(test)]
mod tests {
    use household_core::IdError;

    use super::*;

    fn hid(raw: &str) -> HouseholdId {
        HouseholdId::new(raw).unwrap()
    }

    fn r(numer: u32, denom: u32) -> Rational {
        Rational::new(numer, denom).unwrap()
    }

    fn provenance() -> RecipeProvenance {
        RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap()
    }

    fn line(original: &str, name: &str) -> IngredientLine {
        IngredientLine::new(
            original,
            name,
            None,
            Quantity::Unknown,
            Unit::None,
            None,
            false,
        )
        .unwrap()
    }

    // --- Step 1: ids, Ingredient, CustomIngredient, RecipeProvenance ---------------------

    #[test]
    fn ids_reject_empty_and_never_trim() {
        assert_eq!(IngredientId::new("").unwrap_err(), IdError::Empty);
        assert_eq!(CustomIngredientId::new(" \t").unwrap_err(), IdError::Empty);
        assert_eq!(RecipeId::new("\n").unwrap_err(), IdError::Empty);
        assert_eq!(IngredientId::new(" i-1 ").unwrap().as_str(), " i-1 ");
        assert_eq!(CustomIngredientId::new(" c-1 ").unwrap().as_str(), " c-1 ");
        assert_eq!(RecipeId::new(" r-1 ").unwrap().as_str(), " r-1 ");
    }

    #[test]
    fn provenance_kind_strings_round_trip() {
        assert!(!ProvenanceKind::ALL.is_empty());
        for kind in ProvenanceKind::ALL {
            assert_eq!(ProvenanceKind::parse(kind.as_str()).unwrap(), kind);
        }
        assert_eq!(
            ProvenanceKind::ALL.map(ProvenanceKind::as_str),
            ["authored", "imported", "starter"]
        );
    }

    #[test]
    fn unknown_provenance_kind_is_rejected() {
        for raw in ["scraped", "", "Authored", " authored"] {
            assert_eq!(
                ProvenanceKind::parse(raw).unwrap_err(),
                RecipeError::UnknownProvenanceKind(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn blank_ingredient_and_custom_ingredient_names_are_rejected() {
        for blank in ["", "   ", "\t\n"] {
            let id = IngredientId::new("i").unwrap();
            assert_eq!(
                Ingredient::new(id, blank, vec![], None).unwrap_err(),
                RecipeError::Empty {
                    field: "canonical_name"
                },
                "{blank:?} must be rejected as a canonical name"
            );
            let id = IngredientId::new("i").unwrap();
            assert_eq!(
                Ingredient::new(id, "flour", vec![blank.to_owned()], None).unwrap_err(),
                RecipeError::Empty { field: "alias" },
                "{blank:?} must be rejected as an alias"
            );
            let id = CustomIngredientId::new("c").unwrap();
            assert_eq!(
                CustomIngredient::new(id, hid("h"), blank, None).unwrap_err(),
                RecipeError::Empty { field: "name" },
                "{blank:?} must be rejected as a custom name"
            );
        }
    }

    #[test]
    fn aliases_are_canonical_and_deduplicated() {
        let id = IngredientId::new("i").unwrap();
        let aliases = [
            "plain flour",
            "AP flour",
            "plain flour",
            "all-purpose flour",
        ]
        .map(str::to_owned)
        .to_vec();
        let ingredient = Ingredient::new(id, "flour", aliases, Some("baking".into())).unwrap();
        assert_eq!(ingredient.id().as_str(), "i");
        assert_eq!(ingredient.canonical_name(), "flour");
        assert_eq!(
            ingredient.aliases(),
            ["AP flour", "all-purpose flour", "plain flour"]
        );
        assert_eq!(ingredient.store_category(), Some("baking"));
        let custom = CustomIngredient::new(
            CustomIngredientId::new("c").unwrap(),
            hid("h"),
            "nana's spice mix",
            None,
        )
        .unwrap();
        assert_eq!(custom.id().as_str(), "c");
        assert_eq!(custom.household_id().as_str(), "h");
        assert_eq!(custom.name(), "nana's spice mix");
        assert_eq!(custom.store_category(), None);
    }

    // --- Step 2: Rational, Quantity, UnitKind, Unit --------------------------------------

    #[test]
    fn rational_normalises_by_gcd() {
        assert_eq!(r(2, 4), r(1, 2));
        assert_eq!(r(2, 4).numer(), 1);
        assert_eq!(r(2, 4).denom(), 2);
        assert_eq!(r(6, 3).numer(), 2);
        assert_eq!(r(6, 3).denom(), 1);
    }

    #[test]
    fn rational_orders_by_value() {
        assert!(r(1, 3) < r(1, 2));
        assert!(r(3, 2) > r(1, 1));
        assert_eq!(r(1, 2).cmp(&r(2, 4)), std::cmp::Ordering::Equal);
    }

    #[test]
    fn unit_strings_round_trip() {
        assert!(!UnitKind::ALL.is_empty());
        for kind in UnitKind::ALL {
            assert_eq!(UnitKind::parse(kind.as_str()).unwrap(), kind);
        }
        assert_eq!(
            UnitKind::ALL.map(UnitKind::as_str),
            ["tsp", "tbsp", "cup", "fl_oz", "ml", "l", "g", "kg", "oz", "lb", "piece"]
        );
    }

    #[test]
    fn zero_denominator_is_rejected() {
        assert_eq!(
            Rational::new(1, 0).unwrap_err(),
            RecipeError::ZeroDenominator
        );
        // Denominator is checked first: `0/0` is a malformed fraction before it is a zero.
        assert_eq!(
            Rational::new(0, 0).unwrap_err(),
            RecipeError::ZeroDenominator
        );
    }

    #[test]
    fn zero_numerator_is_rejected() {
        assert_eq!(Rational::new(0, 1).unwrap_err(), RecipeError::ZeroQuantity);
        assert_eq!(Rational::new(0, 7).unwrap_err(), RecipeError::ZeroQuantity);
    }

    #[test]
    fn range_with_equal_bounds_is_allowed() {
        let range = QuantityRange::new(r(2, 1), r(4, 2)).unwrap();
        assert_eq!(range.min(), r(2, 1));
        assert_eq!(range.max(), r(2, 1));
    }

    #[test]
    fn mixed_number_is_exact() {
        // "1 1/2" entered as 1 + 1/2 = 3/2, with no intermediate decimal.
        let whole = 1u32;
        let mixed = Rational::new(whole * 2 + 1, 2).unwrap();
        assert_eq!(mixed, r(3, 2));
        assert_eq!(Quantity::Exact(mixed).kind_str(), "exact");
        assert_eq!(Quantity::Unknown.kind_str(), "unknown");
        let range = QuantityRange::new(r(2, 1), r(3, 1)).unwrap();
        assert_eq!(Quantity::Range(range).kind_str(), "range");
    }

    #[test]
    fn inverted_range_is_rejected() {
        assert_eq!(
            QuantityRange::new(r(3, 1), r(2, 1)).unwrap_err(),
            RecipeError::InvertedRange {
                min: "3/1".to_owned(),
                max: "2/1".to_owned(),
            }
        );
    }

    #[test]
    fn unknown_unit_is_rejected() {
        for raw in ["cups", "Cup", " cup", ""] {
            assert_eq!(
                UnitKind::parse(raw).unwrap_err(),
                RecipeError::UnknownUnit(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn rational_ordering_does_not_overflow() {
        let big = r(u32::MAX, 1);
        let half = r(u32::MAX, 2);
        assert!(big > half);
        assert!(half < big);
        assert!(r(u32::MAX, u32::MAX - 1) > r(1, 1));
    }

    // --- MVP-015 step 2: checked arithmetic -----------------------------------------------

    #[test]
    fn rational_add_and_mul_are_exact_and_normalised() {
        assert_eq!(r(1, 2).checked_add(r(1, 3)), Some(r(5, 6)));
        assert_eq!(r(2, 3).checked_mul(r(3, 4)), Some(r(1, 2)));
        assert_eq!(r(1, 2).checked_add(r(1, 2)), Some(r(1, 1)));
        assert_eq!(r(1, 2).checked_add(r(1, 2)).unwrap().denom(), 1);
    }

    #[test]
    fn rational_add_reports_overflow_instead_of_wrapping() {
        assert_eq!(r(u32::MAX, 1).checked_add(r(1, 1)), None);
        assert_eq!(r(u32::MAX, 1).checked_mul(r(2, 1)), None);
        // Each cross product fits `u64`, but their sum does not: this pair fails against an
        // unchecked `u64` addition, which would wrap in release and panic in debug.
        let near = r(u32::MAX, u32::MAX - 1);
        assert_eq!(near.checked_add(near), None);
        // A large-denominator product that fits `u64` but not `u32` after normalisation.
        assert_eq!(r(1, u32::MAX).checked_add(r(1, u32::MAX - 1)), None);
        assert_eq!(r(1, 2).checked_add(r(1, 2)), Some(r(1, 1)));
    }

    #[test]
    fn rational_add_is_commutative_on_a_boundary_matrix() {
        let matrix = [r(1, 1), r(1, 3), r(u32::MAX, 1), r(1, u32::MAX)];
        assert!(!matrix.is_empty());
        for a in matrix {
            for b in matrix {
                assert_eq!(a.checked_add(b), b.checked_add(a), "{a} + {b}");
                assert_eq!(a.checked_mul(b), b.checked_mul(a), "{a} * {b}");
            }
        }
        assert_eq!(r(1, 3).checked_add(r(1, 1)), Some(r(4, 3)));
        assert_eq!(r(u32::MAX, 1).checked_mul(r(1, u32::MAX)), Some(r(1, 1)));
    }

    // --- Step 3: IngredientLine and Recipe ----------------------------------------------

    #[test]
    fn a_recipe_holds_its_lines_in_order() {
        let lines = vec![line("2 eggs", "eggs"), line("1 cup flour", "flour")];
        let recipe = Recipe::new(
            RecipeId::new("r").unwrap(),
            hid("h"),
            "Pancakes",
            Some(4),
            None,
            "Mix. Fry.",
            lines.clone(),
            provenance(),
        )
        .unwrap();
        assert_eq!(recipe.lines(), lines.as_slice());
        assert_eq!(recipe.lines()[0].name(), "eggs");
        assert_eq!(recipe.lines()[1].name(), "flour");
    }

    #[test]
    fn accessors_report_what_was_constructed() {
        let range = QuantityRange::new(r(2, 1), r(3, 1)).unwrap();
        let l = IngredientLine::new(
            "2-3 cloves garlic, minced (optional)",
            "garlic",
            Some(IngredientRef::Catalog(IngredientId::new("garlic").unwrap())),
            Quantity::Range(range),
            Unit::Known(UnitKind::Piece),
            Some("minced".to_owned()),
            true,
        )
        .unwrap();
        assert_eq!(l.original_text(), "2-3 cloves garlic, minced (optional)");
        assert_eq!(l.name(), "garlic");
        assert_eq!(
            l.ingredient(),
            Some(&IngredientRef::Catalog(
                IngredientId::new("garlic").unwrap()
            ))
        );
        assert_eq!(l.quantity(), Quantity::Range(range));
        assert_eq!(l.unit(), &Unit::Known(UnitKind::Piece));
        assert_eq!(l.preparation(), Some("minced"));
        assert!(l.optional());
        let p = RecipeProvenance::new(
            ProvenanceKind::Imported,
            Some("https://example.com/r".into()),
            Some("Example".into()),
            Some("A. Cook".into()),
        )
        .unwrap();
        let recipe = Recipe::new(
            RecipeId::new("r").unwrap(),
            hid("h"),
            "Garlic thing",
            Some(2),
            Some(25),
            "Cook.",
            vec![l.clone()],
            p.clone(),
        )
        .unwrap();
        assert_eq!(recipe.id().as_str(), "r");
        assert_eq!(recipe.household_id().as_str(), "h");
        assert_eq!(recipe.title(), "Garlic thing");
        assert_eq!(recipe.servings(), Some(2));
        assert_eq!(recipe.prep_minutes(), Some(25));
        assert_eq!(recipe.instructions(), "Cook.");
        assert_eq!(recipe.lines(), &[l]);
        assert_eq!(recipe.provenance(), &p);
    }

    #[test]
    fn empty_instructions_and_no_lines_are_a_valid_stub() {
        let recipe = Recipe::new(
            RecipeId::new("r").unwrap(),
            hid("h"),
            "Tuesday thing",
            None,
            None,
            "",
            vec![],
            provenance(),
        )
        .unwrap();
        assert_eq!(recipe.instructions(), "");
        assert!(recipe.lines().is_empty());
        assert_eq!(recipe.servings(), None);
    }

    #[test]
    fn line_with_unknown_quantity_no_unit_and_no_identity_is_valid() {
        let l = line("some flour", "flour");
        assert_eq!(l.quantity(), Quantity::Unknown);
        assert_eq!(l.unit(), &Unit::None);
        assert_eq!(l.ingredient(), None);
        assert_eq!(l.preparation(), None);
        assert!(!l.optional());
    }

    #[test]
    fn blank_title_original_text_name_and_preparation_are_rejected() {
        for blank in ["", "  ", "\t"] {
            assert_eq!(
                IngredientLine::new(
                    blank,
                    "flour",
                    None,
                    Quantity::Unknown,
                    Unit::None,
                    None,
                    false
                )
                .unwrap_err(),
                RecipeError::Empty {
                    field: "original_text"
                },
                "{blank:?} original_text"
            );
            assert_eq!(
                IngredientLine::new(
                    "flour",
                    blank,
                    None,
                    Quantity::Unknown,
                    Unit::None,
                    None,
                    false
                )
                .unwrap_err(),
                RecipeError::Empty { field: "name" },
                "{blank:?} name"
            );
            assert_eq!(
                IngredientLine::new(
                    "flour",
                    "flour",
                    None,
                    Quantity::Unknown,
                    Unit::None,
                    Some(blank.to_owned()),
                    false,
                )
                .unwrap_err(),
                RecipeError::Empty {
                    field: "preparation"
                },
                "{blank:?} preparation"
            );
            assert_eq!(
                Recipe::new(
                    RecipeId::new("r").unwrap(),
                    hid("h"),
                    blank,
                    None,
                    None,
                    "",
                    vec![],
                    provenance(),
                )
                .unwrap_err(),
                RecipeError::Empty { field: "title" },
                "{blank:?} title"
            );
        }
    }

    #[test]
    fn blank_unit_other_text_is_rejected() {
        for blank in ["", "   ", "\t\n"] {
            assert_eq!(
                IngredientLine::new(
                    "a splash",
                    "something",
                    None,
                    Quantity::Unknown,
                    Unit::Other(blank.to_owned()),
                    None,
                    false,
                )
                .unwrap_err(),
                RecipeError::Empty { field: "unit" },
                "{blank:?} unit"
            );
        }
        // The accept path is unchanged: a real unrecognised unit still round-trips verbatim.
        let l = IngredientLine::new(
            "2 handfuls",
            "rice",
            None,
            Quantity::Unknown,
            Unit::Other(" handful ".to_owned()),
            None,
            false,
        )
        .unwrap();
        assert_eq!(l.unit(), &Unit::Other(" handful ".to_owned()));
    }

    #[test]
    fn blank_provenance_strings_are_rejected() {
        for blank in ["", "  ", "\t"] {
            let cases = [
                ("source_url", (Some(blank), None, None)),
                ("source_name", (None, Some(blank), None)),
                ("source_author", (None, None, Some(blank))),
            ];
            for (field, (url, name, author)) in cases {
                assert_eq!(
                    RecipeProvenance::new(
                        ProvenanceKind::Imported,
                        url.map(str::to_owned),
                        name.map(str::to_owned),
                        author.map(str::to_owned),
                    )
                    .unwrap_err(),
                    RecipeError::Empty { field },
                    "{blank:?} {field}"
                );
            }
        }
        // All-absent and all-present both remain valid.
        assert!(RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).is_ok());
        let p = RecipeProvenance::new(
            ProvenanceKind::Imported,
            Some("https://example.com/r".to_owned()),
            Some("Example".to_owned()),
            Some("A. Cook".to_owned()),
        )
        .unwrap();
        assert_eq!(p.kind(), ProvenanceKind::Imported);
        assert_eq!(p.source_url(), Some("https://example.com/r"));
        assert_eq!(p.source_name(), Some("Example"));
        assert_eq!(p.source_author(), Some("A. Cook"));
    }

    #[test]
    fn blank_store_category_is_rejected() {
        for blank in ["", "   ", "\t\n"] {
            assert_eq!(
                Ingredient::new(
                    IngredientId::new("i").unwrap(),
                    "flour",
                    vec![],
                    Some(blank.to_owned()),
                )
                .unwrap_err(),
                RecipeError::Empty {
                    field: "store_category"
                },
                "{blank:?} catalog store_category"
            );
            assert_eq!(
                CustomIngredient::new(
                    CustomIngredientId::new("c").unwrap(),
                    hid("h"),
                    "nana's mix",
                    Some(blank.to_owned()),
                )
                .unwrap_err(),
                RecipeError::Empty {
                    field: "store_category"
                },
                "{blank:?} custom store_category"
            );
        }
    }

    #[test]
    fn zero_servings_is_rejected() {
        assert_eq!(
            Recipe::new(
                RecipeId::new("r").unwrap(),
                hid("h"),
                "Nothing",
                Some(0),
                None,
                "",
                vec![],
                provenance(),
            )
            .unwrap_err(),
            RecipeError::ZeroServings
        );
    }

    #[test]
    fn original_text_is_stored_verbatim() {
        let raw = "  2 Cups  Flour, sifted ";
        let l = IngredientLine::new(
            raw,
            " Flour ",
            None,
            Quantity::Exact(r(2, 1)),
            Unit::Known(UnitKind::Cup),
            Some(" sifted ".to_owned()),
            false,
        )
        .unwrap();
        assert_eq!(l.original_text(), raw);
        assert_eq!(l.name(), " Flour ");
        assert_eq!(l.preparation(), Some(" sifted "));
    }

    // --- MVP-011 step 2: rights on provenance -------------------------------------------

    #[test]
    fn rights_basis_strings_round_trip() {
        // Hard-coded rather than read off `RightsBasis::ALL`: a fourth basis added without a
        // rights review must break this test, not be mirrored by it.
        for raw in ["original", "us_federal_public_domain", "cc0"] {
            assert_eq!(RightsBasis::parse(raw).unwrap().as_str(), raw);
        }
        assert_eq!(RightsBasis::ALL.len(), 3);
    }

    #[test]
    fn unknown_rights_basis_is_rejected() {
        for raw in ["cc_by", "cc_by_sa", "scraped", "Original", " original", ""] {
            assert_eq!(
                RightsBasis::parse(raw).unwrap_err(),
                RecipeError::UnknownRightsBasis(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn a_non_civil_verified_on_is_rejected() {
        for raw in ["2026-13-45", "20260829", "2026-08-29T00:00:00Z", ""] {
            assert_eq!(
                RecipeRights::new(RightsBasis::Original, None, None, raw).unwrap_err(),
                RecipeError::InvalidVerifiedOn(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn blank_attribution_and_modifications_are_rejected() {
        for blank in ["", " ", "\t\n"] {
            assert_eq!(
                RecipeRights::new(RightsBasis::Cc0, Some(blank.to_owned()), None, "2026-08-29")
                    .unwrap_err(),
                RecipeError::Empty {
                    field: "attribution"
                },
                "{blank:?} attribution"
            );
            assert_eq!(
                RecipeRights::new(RightsBasis::Cc0, None, Some(blank.to_owned()), "2026-08-29")
                    .unwrap_err(),
                RecipeError::Empty {
                    field: "modifications"
                },
                "{blank:?} modifications"
            );
        }
    }

    #[test]
    fn blank_starter_slug_is_rejected() {
        for blank in ["", " ", "\t\n"] {
            assert_eq!(
                RecipeProvenance::with_rights(
                    ProvenanceKind::Starter,
                    None,
                    None,
                    None,
                    None,
                    Some(blank.to_owned()),
                )
                .unwrap_err(),
                RecipeError::Empty {
                    field: "starter_slug"
                },
                "{blank:?} starter_slug"
            );
        }
    }

    #[test]
    fn provenance_new_carries_no_rights_or_slug() {
        let p = RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap();
        assert_eq!(p.rights(), None);
        assert_eq!(p.starter_slug(), None);
    }

    #[test]
    fn with_rights_carries_both() {
        let rights = RecipeRights::new(
            RightsBasis::UsFederalPublicDomain,
            Some("USDA".to_owned()),
            Some("halved the salt".to_owned()),
            "2026-08-29",
        )
        .unwrap();
        let p = RecipeProvenance::with_rights(
            ProvenanceKind::Starter,
            Some("https://example.gov/r".to_owned()),
            Some("Example".to_owned()),
            Some("Kimatta".to_owned()),
            Some(rights),
            Some("beans-and-rice".to_owned()),
        )
        .unwrap();
        assert_eq!(p.kind(), ProvenanceKind::Starter);
        assert_eq!(p.source_url(), Some("https://example.gov/r"));
        assert_eq!(p.source_name(), Some("Example"));
        assert_eq!(p.source_author(), Some("Kimatta"));
        assert_eq!(p.starter_slug(), Some("beans-and-rice"));
        let stored = p.rights().unwrap();
        assert_eq!(stored.basis(), RightsBasis::UsFederalPublicDomain);
        assert_eq!(stored.attribution(), Some("USDA"));
        assert_eq!(stored.modifications(), Some("halved the salt"));
        assert_eq!(stored.verified_on().to_string(), "2026-08-29");
    }

    // --- MVP-011 step 3: prep_minutes ---------------------------------------------------

    #[test]
    fn zero_prep_minutes_is_rejected() {
        assert_eq!(
            Recipe::new(
                RecipeId::new("r").unwrap(),
                hid("h"),
                "Nothing",
                None,
                Some(0),
                "",
                vec![],
                provenance(),
            )
            .unwrap_err(),
            RecipeError::ZeroPrepMinutes
        );
    }

    #[test]
    fn prep_minutes_round_trips_through_new() {
        let recipe = Recipe::new(
            RecipeId::new("r").unwrap(),
            hid("h"),
            "Soup",
            Some(4),
            Some(1),
            "",
            vec![],
            provenance(),
        )
        .unwrap();
        assert_eq!(recipe.prep_minutes(), Some(1));
    }

    #[test]
    fn a_recipe_with_no_prep_estimate_is_legal() {
        // PRD §10: absence stays absent. Nothing defaults an unknown prep time to a number.
        let recipe = Recipe::new(
            RecipeId::new("r").unwrap(),
            hid("h"),
            "Soup",
            None,
            None,
            "",
            vec![],
            provenance(),
        )
        .unwrap();
        assert_eq!(recipe.prep_minutes(), None);
    }
}
