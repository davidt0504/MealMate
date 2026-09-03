//! Planned meal occurrences (PRD v3 §8). A `PlannedMeal` asserts *planned* and nothing more:
//! confirmation that it was cooked belongs to the outcome ledger (invariant 19), so no field
//! here says "cooked", and none should be read as if it did.

use household_core::{HouseholdId, IdError};
use thiserror::Error;

use crate::{CivilDate, MealSlot, Rational, RecipeId};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlannedMealError {
    #[error("unknown meal component kind {0:?}")]
    UnknownComponentKind(String),
    #[error("a planned meal needs at least one component")]
    NoComponents,
    #[error("an open component must be the meal's only component")]
    OpenMustStandAlone,
    #[error("a freeform component needs a note")]
    FreeformNeedsANote,
    #[error("a serving scale must be a positive fraction; absent means as written")]
    ZeroScale,
    #[error("no component at position {0}")]
    NoSuchComponent(usize),
    #[error("component {0} is not a recipe, so it has no scale")]
    NotARecipeComponent(usize),
    #[error("cannot move component {from} to position {to}")]
    InvalidMove { from: usize, to: usize },
}

// Own copy of `household-core`'s macro, for the reason `recipe.rs` gives above its copy.
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

id_newtype!(PlannedMealId);

/// Who is writing an occurrence. A lock is a Tier-0 constraint (invariant 18, PRD §9.7):
/// storage refuses every `Automation` write to a locked occurrence, and there is deliberately
/// no numeric conversion — a lock is never a score weight. MVP-023 names this type so it can
/// never call storage without saying which side of the contract it is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteSource {
    User,
    Automation,
}

/// One thing a meal occurrence is made of. Non-recipe meals are kinds of their own, never an
/// absent recipe (card stop condition): `DiningOut` covers takeout and `Open` covers
/// "intentionally open or away", the two pairs PRD §8 lists as one kind each.
///
/// `scale` is a multiplier of the recipe *as written*; `None` is 1× and is never rendered as
/// a number. A multiplier rather than a target serving count because `Recipe::servings` is
/// nullable, so a multiplier is the only representation that is always defined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MealComponent {
    Recipe {
        recipe_id: RecipeId,
        scale: Option<Rational>,
    },
    Leftovers {
        note: Option<String>,
    },
    DiningOut {
        note: Option<String>,
    },
    FrozenQuick {
        note: Option<String>,
    },
    Freeform {
        note: String,
    },
    Open {
        note: Option<String>,
    },
}

impl MealComponent {
    /// The stored `kind` tokens, in the order a picker should offer them.
    pub const KINDS: [&'static str; 6] = [
        "recipe",
        "leftovers",
        "dining_out",
        "frozen_quick",
        "freeform",
        "open",
    ];

    pub fn recipe(recipe_id: RecipeId, scale: Option<Rational>) -> Self {
        Self::Recipe { recipe_id, scale }
    }

    /// A serving scale from its two stored or transmitted halves. Both zero cases are the same
    /// fault — "not a positive multiplier" — rather than `RecipeError`'s two quantity
    /// vocabularies, so a Dart caller sees one typed planned-meal error.
    pub fn parse_scale(numer: u32, denom: u32) -> Result<Rational, PlannedMealError> {
        Rational::new(numer, denom).map_err(|_| PlannedMealError::ZeroScale)
    }

    /// The note is stored verbatim and never trimmed, as `IngredientLine` text is; only a
    /// blank one is refused, because a freeform component with nothing written is not a meal.
    pub fn freeform(note: impl Into<String>) -> Result<Self, PlannedMealError> {
        let note = note.into();
        if note.trim().is_empty() {
            return Err(PlannedMealError::FreeformNeedsANote);
        }
        Ok(Self::Freeform { note })
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Recipe { .. } => "recipe",
            Self::Leftovers { .. } => "leftovers",
            Self::DiningOut { .. } => "dining_out",
            Self::FrozenQuick { .. } => "frozen_quick",
            Self::Freeform { .. } => "freeform",
            Self::Open { .. } => "open",
        }
    }

    /// The read path: one stored row back into a component. `Ok(None)` is a row whose columns
    /// do not describe its `kind` — a `recipe` row carrying a note, or a non-recipe row carrying
    /// a recipe or a scale — which the caller reports with its coordinates rather than coerces,
    /// as `restriction_from_row` does. The DDL CHECKs make those shapes unreachable through the
    /// writer; the reader still refuses them. An unknown `kind` is its own error because the
    /// token is the fault, not the shape.
    pub fn parse_row(
        kind: &str,
        recipe_id: Option<RecipeId>,
        note: Option<String>,
        scale: Option<Rational>,
    ) -> Result<Option<Self>, PlannedMealError> {
        if !Self::KINDS.contains(&kind) {
            return Err(PlannedMealError::UnknownComponentKind(kind.to_owned()));
        }
        let parsed = match (kind, recipe_id, note, scale) {
            ("recipe", Some(recipe_id), None, scale) => Some(Self::Recipe { recipe_id, scale }),
            ("leftovers", None, note, None) => Some(Self::Leftovers { note }),
            ("dining_out", None, note, None) => Some(Self::DiningOut { note }),
            ("frozen_quick", None, note, None) => Some(Self::FrozenQuick { note }),
            ("freeform", None, Some(note), None) => Some(Self::freeform(note)?),
            ("open", None, note, None) => Some(Self::Open { note }),
            _ => None,
        };
        Ok(parsed)
    }

    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open { .. })
    }
}

/// A household's meal occurrence on one civil date and slot. Fields are private because the
/// component list carries cross-item rules — at least one, and `Open` stands alone — so every
/// edit goes through a method that re-checks them, and a value of this type is always valid.
///
/// `locked` travels with the value for reads, but storage never writes it from a save: a lock
/// is set and cleared only through its own command, so a caller re-saving an occurrence
/// cannot flip it by accident.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedMeal {
    id: PlannedMealId,
    household_id: HouseholdId,
    date: CivilDate,
    slot: MealSlot,
    components: Vec<MealComponent>,
    locked: bool,
}

impl PlannedMeal {
    pub fn new(
        id: PlannedMealId,
        household_id: HouseholdId,
        date: CivilDate,
        slot: MealSlot,
        components: Vec<MealComponent>,
        locked: bool,
    ) -> Result<Self, PlannedMealError> {
        check_components(&components)?;
        Ok(Self {
            id,
            household_id,
            date,
            slot,
            components,
            locked,
        })
    }

    pub fn id(&self) -> &PlannedMealId {
        &self.id
    }

    pub fn household_id(&self) -> &HouseholdId {
        &self.household_id
    }

    pub fn date(&self) -> CivilDate {
        self.date
    }

    pub fn slot(&self) -> MealSlot {
        self.slot
    }

    pub fn components(&self) -> &[MealComponent] {
        &self.components
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    /// Appends; refused when it would put an `Open` component beside another.
    pub fn add_component(&mut self, component: MealComponent) -> Result<(), PlannedMealError> {
        if component.is_open() || self.components.iter().any(MealComponent::is_open) {
            return Err(PlannedMealError::OpenMustStandAlone);
        }
        self.components.push(component);
        Ok(())
    }

    /// Removes the component at `position`; the last one is refused, since a meal with no
    /// components is not a meal — delete the occurrence instead.
    pub fn remove_component(&mut self, position: usize) -> Result<(), PlannedMealError> {
        if position >= self.components.len() {
            return Err(PlannedMealError::NoSuchComponent(position));
        }
        if self.components.len() == 1 {
            return Err(PlannedMealError::NoComponents);
        }
        self.components.remove(position);
        Ok(())
    }

    /// Moves the component at `from` so that it sits at `to`; `from == to` is a no-op.
    pub fn move_component(&mut self, from: usize, to: usize) -> Result<(), PlannedMealError> {
        if from >= self.components.len() {
            return Err(PlannedMealError::NoSuchComponent(from));
        }
        if to >= self.components.len() {
            return Err(PlannedMealError::InvalidMove { from, to });
        }
        if from != to {
            let component = self.components.remove(from);
            self.components.insert(to, component);
        }
        Ok(())
    }

    /// Sets the serving scale of the recipe component at `position`; `None` is as written.
    pub fn set_scale(
        &mut self,
        position: usize,
        scale: Option<Rational>,
    ) -> Result<(), PlannedMealError> {
        match self.components.get_mut(position) {
            None => Err(PlannedMealError::NoSuchComponent(position)),
            Some(MealComponent::Recipe { scale: slot, .. }) => {
                *slot = scale;
                Ok(())
            }
            Some(_) => Err(PlannedMealError::NotARecipeComponent(position)),
        }
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }
}

fn check_components(components: &[MealComponent]) -> Result<(), PlannedMealError> {
    if components.is_empty() {
        return Err(PlannedMealError::NoComponents);
    }
    if components.len() > 1 && components.iter().any(MealComponent::is_open) {
        return Err(PlannedMealError::OpenMustStandAlone);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_civil_date;

    fn hid(raw: &str) -> HouseholdId {
        HouseholdId::new(raw).unwrap()
    }

    fn rid(raw: &str) -> RecipeId {
        RecipeId::new(raw).unwrap()
    }

    fn r(numer: u32, denom: u32) -> Rational {
        Rational::new(numer, denom).unwrap()
    }

    fn leftovers() -> MealComponent {
        MealComponent::Leftovers {
            note: Some("chili".to_owned()),
        }
    }

    fn open() -> MealComponent {
        MealComponent::Open { note: None }
    }

    fn meal(components: Vec<MealComponent>) -> Result<PlannedMeal, PlannedMealError> {
        PlannedMeal::new(
            PlannedMealId::new("pm").unwrap(),
            hid("h"),
            parse_civil_date("2026-08-29").unwrap(),
            MealSlot::Dinner,
            components,
            false,
        )
    }

    #[test]
    fn planned_meal_id_rejects_empty_and_never_trims() {
        assert_eq!(PlannedMealId::new(" \t").unwrap_err(), IdError::Empty);
        assert_eq!(PlannedMealId::new(" pm-1 ").unwrap().as_str(), " pm-1 ");
    }

    #[test]
    fn component_kind_tokens_round_trip_and_reject_unknown() {
        // Hard-coded, not read off `KINDS`: a seventh kind added without a schema review must
        // break this test, not be mirrored by it.
        let expected = [
            "recipe",
            "leftovers",
            "dining_out",
            "frozen_quick",
            "freeform",
            "open",
        ];
        assert_eq!(MealComponent::KINDS, expected);
        let samples = [
            MealComponent::recipe(rid("r"), None),
            MealComponent::Leftovers { note: None },
            MealComponent::DiningOut { note: None },
            MealComponent::FrozenQuick { note: None },
            MealComponent::freeform("pizza").unwrap(),
            open(),
        ];
        for (sample, token) in samples.iter().zip(expected) {
            assert_eq!(sample.kind_str(), token);
            let recipe_id = match sample {
                MealComponent::Recipe { recipe_id, .. } => Some(recipe_id.clone()),
                _ => None,
            };
            let note = match sample {
                MealComponent::Freeform { note } => Some(note.clone()),
                _ => None,
            };
            assert_eq!(
                MealComponent::parse_row(token, recipe_id, note, None).unwrap(),
                Some(sample.clone()),
                "{token} must parse back to itself"
            );
        }
        for raw in [" open", "Open", "takeout", "", "away"] {
            assert_eq!(
                MealComponent::parse_row(raw, None, None, None).unwrap_err(),
                PlannedMealError::UnknownComponentKind(raw.to_owned()),
                "{raw:?} must not parse"
            );
        }
    }

    #[test]
    fn a_mis_shaped_row_is_none_not_coerced() {
        // A recipe row with a note, a non-recipe row with a recipe, a scaled non-recipe row,
        // a recipe row with no recipe: each is refused, none is repaired.
        let shapes = [
            ("recipe", Some(rid("r")), Some("x".to_owned()), None),
            ("leftovers", Some(rid("r")), None, None),
            ("open", None, None, Some(r(1, 2))),
            ("recipe", None, None, None),
        ];
        for (kind, recipe_id, note, scale) in shapes {
            assert_eq!(
                MealComponent::parse_row(kind, recipe_id, note, scale).unwrap(),
                None,
                "{kind} shape must be refused"
            );
        }
        // A blank freeform note is shaped correctly; it is the value that is wrong.
        assert_eq!(
            MealComponent::parse_row("freeform", None, Some(" ".to_owned()), None).unwrap_err(),
            PlannedMealError::FreeformNeedsANote
        );
    }

    #[test]
    fn a_meal_needs_at_least_one_component() {
        assert_eq!(meal(vec![]).unwrap_err(), PlannedMealError::NoComponents);
        assert!(meal(vec![leftovers()]).is_ok());
    }

    #[test]
    fn open_must_be_the_only_component() {
        assert!(meal(vec![open()]).is_ok());
        assert_eq!(
            meal(vec![open(), leftovers()]).unwrap_err(),
            PlannedMealError::OpenMustStandAlone
        );
        assert_eq!(
            meal(vec![leftovers(), open()]).unwrap_err(),
            PlannedMealError::OpenMustStandAlone
        );
        let mut m = meal(vec![leftovers()]).unwrap();
        assert_eq!(
            m.add_component(open()).unwrap_err(),
            PlannedMealError::OpenMustStandAlone
        );
        let mut m = meal(vec![open()]).unwrap();
        assert_eq!(
            m.add_component(leftovers()).unwrap_err(),
            PlannedMealError::OpenMustStandAlone
        );
        assert_eq!(m.components(), &[open()]);
    }

    #[test]
    fn freeform_needs_a_non_blank_note_kept_verbatim() {
        for blank in ["", "  ", "\t\n"] {
            assert_eq!(
                MealComponent::freeform(blank).unwrap_err(),
                PlannedMealError::FreeformNeedsANote,
                "{blank:?} must be refused"
            );
        }
        assert_eq!(
            MealComponent::freeform(" pizza ").unwrap(),
            MealComponent::Freeform {
                note: " pizza ".to_owned()
            }
        );
    }

    #[test]
    fn a_zero_scale_is_rejected_and_none_means_as_written() {
        assert_eq!(
            MealComponent::parse_scale(0, 1).unwrap_err(),
            PlannedMealError::ZeroScale
        );
        assert_eq!(
            MealComponent::parse_scale(1, 0).unwrap_err(),
            PlannedMealError::ZeroScale
        );
        assert_eq!(MealComponent::parse_scale(2, 4).unwrap(), r(1, 2));
        let as_written = MealComponent::recipe(rid("r"), None);
        assert_eq!(
            as_written,
            MealComponent::Recipe {
                recipe_id: rid("r"),
                scale: None
            }
        );
    }

    #[test]
    fn a_mixed_meal_of_leftovers_and_a_recipe_is_valid() {
        let components = vec![leftovers(), MealComponent::recipe(rid("r"), Some(r(3, 2)))];
        let m = meal(components.clone()).unwrap();
        assert_eq!(m.components(), components.as_slice());
        assert_eq!(m.id().as_str(), "pm");
        assert_eq!(m.household_id().as_str(), "h");
        assert_eq!(m.date().to_string(), "2026-08-29");
        assert_eq!(m.slot(), MealSlot::Dinner);
        assert!(!m.locked());
    }

    #[test]
    fn remove_of_the_last_component_is_refused() {
        let mut m = meal(vec![leftovers(), quick("x")]).unwrap();
        assert_eq!(
            m.remove_component(2).unwrap_err(),
            PlannedMealError::NoSuchComponent(2)
        );
        m.remove_component(0).unwrap();
        assert_eq!(m.components(), &[quick("x")]);
        assert_eq!(
            m.remove_component(0).unwrap_err(),
            PlannedMealError::NoComponents
        );
        assert_eq!(m.components().len(), 1);
    }

    /// A non-open second component for reorder tests, so `Open`'s stand-alone rule stays out
    /// of the way.
    fn quick(note: &str) -> MealComponent {
        MealComponent::FrozenQuick {
            note: Some(note.to_owned()),
        }
    }

    #[test]
    fn move_component_reorders_and_rejects_out_of_range() {
        let a = MealComponent::recipe(rid("a"), Some(r(1, 2)));
        let b = leftovers();
        let c = quick("c");
        let mut m = meal(vec![a.clone(), b.clone(), c.clone()]).unwrap();
        m.move_component(2, 0).unwrap();
        assert_eq!(m.components(), &[c.clone(), a.clone(), b.clone()]);
        m.move_component(0, 2).unwrap();
        assert_eq!(m.components(), &[a.clone(), b.clone(), c.clone()]);
        m.move_component(1, 1).unwrap();
        assert_eq!(m.components(), &[a.clone(), b.clone(), c.clone()]);
        assert_eq!(
            m.move_component(0, 3).unwrap_err(),
            PlannedMealError::InvalidMove { from: 0, to: 3 }
        );
        assert_eq!(
            m.move_component(3, 0).unwrap_err(),
            PlannedMealError::NoSuchComponent(3)
        );
        assert_eq!(m.components(), &[a, b, c]);
    }

    #[test]
    fn set_scale_on_a_non_recipe_component_is_refused() {
        let mut m = meal(vec![leftovers(), MealComponent::recipe(rid("r"), None)]).unwrap();
        assert_eq!(
            m.set_scale(0, Some(r(2, 1))).unwrap_err(),
            PlannedMealError::NotARecipeComponent(0)
        );
        assert_eq!(
            m.set_scale(2, None).unwrap_err(),
            PlannedMealError::NoSuchComponent(2)
        );
        m.set_scale(1, Some(r(2, 1))).unwrap();
        assert_eq!(
            m.components()[1],
            MealComponent::recipe(rid("r"), Some(r(2, 1)))
        );
        m.set_scale(1, None).unwrap();
        assert_eq!(m.components()[1], MealComponent::recipe(rid("r"), None));
    }

    /// Doc-level contract for MVP-023: a lock is a boolean and `WriteSource` has exactly two
    /// sides. Nothing here converts either into a number.
    #[test]
    fn lock_is_a_plain_boolean_with_no_weight() {
        let mut m = meal(vec![leftovers()]).unwrap();
        assert!(!m.locked());
        m.set_locked(true);
        assert!(m.locked());
        m.set_locked(false);
        assert!(!m.locked());
        for source in [WriteSource::User, WriteSource::Automation] {
            match source {
                WriteSource::User => {}
                WriteSource::Automation => {}
            }
        }
    }
}
