//! The Cover draft (OPT-007 §4): what the household is looking at between opening Cover My Week
//! and Accept. It is data only — storage keeps it as opaque text and the application layer owns
//! every transition — so this module holds the payload shape, its bounded serialization, and the
//! identity rules every other layer relies on. Recipe identity is `recipe:<id>` or
//! `starter:<slug>`, never a title.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::planner::snapshot::PlanningSnapshot;
use crate::{
    format_civil_date, parse_civil_date, CivilDate, MealComponent, MealSlot, PlannedMeal, Rational,
    RecipeId,
};

/// Bumped whenever the payload shape changes; a stored draft of another version is readable
/// only through Review (§12 M3).
pub const DRAFT_FORMAT_VERSION: u32 = 1;
/// A draft for the longest cycle (31 days × 3 slots) with multi-component meals and a few dozen
/// exclusions is a few tens of KB; anything past this is corrupt, not a plan.
pub const MAX_PAYLOAD_BYTES: usize = 256 * 1024;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DraftError {
    #[error("draft payload is {0} bytes, over the {MAX_PAYLOAD_BYTES}-byte bound")]
    TooLarge(usize),
    #[error("draft payload is unreadable: {0}")]
    Unreadable(String),
}

/// One meal component as the payload stores it: the same four columns `meal_component` uses,
/// so a round trip goes through [`MealComponent::parse_row`] and inherits every shape rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRecord {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ComponentRecord {
    pub fn from_domain(c: &MealComponent) -> Self {
        let (recipe_id, scale, note) = match c {
            MealComponent::Recipe { recipe_id, scale } => (
                Some(recipe_id.as_str().to_owned()),
                scale.map(|s| (s.numer(), s.denom())),
                None,
            ),
            MealComponent::Freeform { note } => (None, None, Some(note.clone())),
            MealComponent::Leftovers { note }
            | MealComponent::DiningOut { note }
            | MealComponent::FrozenQuick { note }
            | MealComponent::Open { note } => (None, None, note.clone()),
        };
        Self {
            kind: c.kind_str().to_owned(),
            recipe_id,
            scale,
            note,
        }
    }

    pub fn to_domain(&self) -> Result<MealComponent, DraftError> {
        let bad = |detail: &str| DraftError::Unreadable(format!("component: {detail}"));
        let recipe_id = self
            .recipe_id
            .clone()
            .map(RecipeId::new)
            .transpose()
            .map_err(|e| bad(&e.to_string()))?;
        let scale = self
            .scale
            .map(|(n, d)| Rational::new(n, d))
            .transpose()
            .map_err(|e| bad(&e.to_string()))?;
        MealComponent::parse_row(&self.kind, recipe_id, self.note.clone(), scale)
            .map_err(|e| bad(&e.to_string()))?
            .ok_or_else(|| bad(&format!("shape does not match kind {:?}", self.kind)))
    }
}

pub fn records(components: &[MealComponent]) -> Vec<ComponentRecord> {
    components
        .iter()
        .map(ComponentRecord::from_domain)
        .collect()
}

pub fn components(records: &[ComponentRecord]) -> Result<Vec<MealComponent>, DraftError> {
    records.iter().map(ComponentRecord::to_domain).collect()
}

/// Where the displayed meal came from. `Chosen` is the household's own pick (Choose, or
/// `Use my choice` in Review): it is locked in the draft and written under user authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotOrigin {
    /// Nothing is displayed: a past date with no saved meal, or a slot nothing could fill.
    Empty,
    Saved,
    Suggested,
    Chosen,
}

/// What the last operation did to a slot. Kept apart from coverage so a usable week stays
/// usable when one day ran out of alternatives (§9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlotOutcome {
    Changed,
    /// Every eligible alternative is excluded or fails a check; the meal shown is unchanged.
    Exhausted,
    /// The slot was addressed but may not change: locked in, past, or not a suggestion.
    Blocked,
}

/// The saved occurrence a draft slot was based on, as it stood when the draft started or was
/// last reviewed. Accept refuses to write over a saved row that no longer matches it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedRecord {
    pub id: String,
    pub components: Vec<ComponentRecord>,
    pub locked: bool,
}

impl SavedRecord {
    pub fn of(meal: &PlannedMeal) -> Self {
        Self {
            id: meal.id().as_str().to_owned(),
            components: records(meal.components()),
            locked: meal.locked(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DraftSlot {
    /// ISO civil date.
    pub date: String,
    pub slot: MealSlot,
    pub components: Vec<ComponentRecord>,
    pub origin: SlotOrigin,
    /// `Locked in` within the draft: protected from Another and week alternatives.
    pub committed: bool,
    /// The household explicitly unlocked or replaced a *saved* lock on this slot. The only
    /// authority Accept accepts for changing a saved locked row, and it is per slot.
    #[serde(default)]
    pub replaces_saved_lock: bool,
    pub base: Option<SavedRecord>,
    /// Slot-local exclusions from Another: every identity it has shown and been asked to replace.
    #[serde(default)]
    pub excluded: BTreeSet<String>,
    #[serde(default)]
    pub outcome: Option<SlotOutcome>,
}

impl DraftSlot {
    pub fn civil_date(&self) -> Result<CivilDate, DraftError> {
        parse_civil_date(&self.date).map_err(|e| DraftError::Unreadable(e.to_string()))
    }

    pub fn meal(&self) -> Result<Vec<MealComponent>, DraftError> {
        components(&self.components)
    }

    /// A week request may move this slot: an unlocked recipe suggestion, saved or not. Locked
    /// meals, manual picks, open slots and fallbacks stay (§6). Dates are the caller's check.
    pub fn is_week_target(&self) -> bool {
        !self.committed
            && matches!(self.origin, SlotOrigin::Saved | SlotOrigin::Suggested)
            && self.has_dish_identity()
    }

    /// Whether the displayed meal names at least one recipe or starter dish.
    pub fn has_dish_identity(&self) -> bool {
        self.components.iter().any(|c| {
            c.kind == "recipe"
                || (c.kind == "freeform"
                    && c.note.as_deref().is_some_and(|n| {
                        n.starts_with(crate::planner::candidates::STARTER_NOTE_PREFIX)
                    }))
        })
    }
}

/// What one draft command did, per addressed slot; kept on the receipt so a replayed request
/// returns the original outcome rather than re-running the command.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationRecord {
    pub operation: String,
    pub slots: Vec<SlotOutcomeRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotOutcomeRecord {
    pub date: String,
    pub slot: MealSlot,
    pub outcome: SlotOutcome,
}

impl OperationRecord {
    pub fn to_text(&self) -> String {
        serde_json::to_string(self).expect("plain data")
    }

    pub fn from_text(text: &str) -> Result<Self, DraftError> {
        if text.len() > MAX_PAYLOAD_BYTES {
            return Err(DraftError::TooLarge(text.len()));
        }
        serde_json::from_str(text).map_err(|e| DraftError::Unreadable(e.to_string()))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DraftPayload {
    /// Canonical `(date, slot)` order, every enabled slot of the window.
    pub slots: Vec<DraftSlot>,
    /// Draft-wide exclusions from week requests; they apply to replaceable targets only.
    #[serde(default)]
    pub excluded: BTreeSet<String>,
}

impl DraftPayload {
    /// Bounded on write as on read, so a row no build can read is never committed.
    pub fn to_text(&self) -> Result<String, DraftError> {
        let text = serde_json::to_string(self).expect("payload has no map with non-string keys");
        if text.len() > MAX_PAYLOAD_BYTES {
            return Err(DraftError::TooLarge(text.len()));
        }
        Ok(text)
    }

    pub fn from_text(text: &str) -> Result<Self, DraftError> {
        if text.len() > MAX_PAYLOAD_BYTES {
            return Err(DraftError::TooLarge(text.len()));
        }
        serde_json::from_str(text).map_err(|e| DraftError::Unreadable(e.to_string()))
    }

    pub fn slot_index(&self, date: CivilDate, slot: MealSlot) -> Option<usize> {
        let date = format_civil_date(date);
        self.slots
            .iter()
            .position(|s| s.date == date && s.slot == slot)
    }
}

/// The identity of each dish in an occurrence, deduplicated in first-seen order. A starter stub
/// and the recipe it was installed as are the same dish, so both canonicalise to
/// `starter:<slug>` through the recipe's provenance; changing scale or order cannot evade it.
/// A freeform note that is not a starter stub, and every fallback kind, has no identity.
pub fn identities(snapshot: &PlanningSnapshot, components: &[MealComponent]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for c in components {
        let id = match c {
            MealComponent::Recipe { recipe_id, .. } => Some(recipe_identity(snapshot, recipe_id)),
            MealComponent::Freeform { note } => note
                .strip_prefix(crate::planner::candidates::STARTER_NOTE_PREFIX)
                .map(|slug| format!("starter:{slug}")),
            _ => None,
        };
        if let Some(id) = id {
            if !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out
}

pub fn recipe_identity(snapshot: &PlanningSnapshot, recipe_id: &RecipeId) -> String {
    match snapshot
        .recipes
        .iter()
        .find(|r| &r.id == recipe_id)
        .and_then(|r| r.starter_slug.as_deref())
    {
        Some(slug) => format!("starter:{slug}"),
        None => format!("recipe:{}", recipe_id.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::snapshot::RecipeCandidateInfo;
    use household_core::HouseholdId;

    fn rid(id: &str) -> RecipeId {
        RecipeId::new(id).unwrap()
    }

    fn snapshot_with(recipes: Vec<RecipeCandidateInfo>) -> PlanningSnapshot {
        PlanningSnapshot {
            household_id: HouseholdId::new("h").unwrap(),
            anchor: parse_civil_date("2026-09-28").unwrap(),
            length_days: 7,
            scope: crate::MealScope::dinner_only(),
            today: parse_civil_date("2026-09-28").unwrap(),
            members: Vec::new(),
            restrictions: crate::HouseholdRestrictions::new(Vec::new()),
            preferences: Vec::new(),
            policies: Default::default(),
            recipes,
            starter: Vec::new(),
            existing: Vec::new(),
            history: Vec::new(),
            pantry_marked: Vec::new(),
        }
    }

    fn info(id: &str, slug: Option<&str>) -> RecipeCandidateInfo {
        RecipeCandidateInfo {
            id: rid(id),
            title: id.to_owned(),
            servings: None,
            prep_minutes: None,
            line_names: Vec::new(),
            ingredient_refs: Vec::new(),
            starter_slug: slug.map(str::to_owned),
            untagged_lines: Vec::new(),
        }
    }

    fn slot(date: &str, components: Vec<MealComponent>) -> DraftSlot {
        DraftSlot {
            date: date.to_owned(),
            slot: MealSlot::Dinner,
            components: records(&components),
            origin: SlotOrigin::Suggested,
            committed: false,
            replaces_saved_lock: false,
            base: None,
            excluded: BTreeSet::new(),
            outcome: None,
        }
    }

    #[test]
    fn every_component_kind_round_trips_through_the_payload() {
        let meal = vec![
            MealComponent::recipe(rid("r1"), Some(Rational::new(3, 2).unwrap())),
            MealComponent::recipe(rid("r2"), None),
            MealComponent::freeform("starter:chili").unwrap(),
            MealComponent::Leftovers {
                note: Some("soup".to_owned()),
            },
        ];
        let mut payload = DraftPayload {
            slots: vec![slot("2026-09-28", meal.clone())],
            excluded: BTreeSet::from(["recipe:r9".to_owned()]),
        };
        payload.slots[0].excluded.insert("recipe:r3".to_owned());
        payload.slots[0].outcome = Some(SlotOutcome::Exhausted);
        let back = DraftPayload::from_text(&payload.to_text().unwrap()).unwrap();
        assert_eq!(back, payload);
        assert_eq!(back.slots[0].meal().unwrap(), meal);
    }

    #[test]
    fn an_oversized_payload_is_refused_on_write() {
        let payload = DraftPayload {
            slots: vec![],
            excluded: (0..MAX_PAYLOAD_BYTES / 16)
                .map(|n| format!("recipe:r{n:08}"))
                .collect(),
        };
        assert!(matches!(payload.to_text(), Err(DraftError::TooLarge(n)) if n > MAX_PAYLOAD_BYTES));
    }

    #[test]
    fn an_oversized_or_malformed_payload_is_refused_not_coerced() {
        let huge = "x".repeat(MAX_PAYLOAD_BYTES + 1);
        assert_eq!(
            DraftPayload::from_text(&huge),
            Err(DraftError::TooLarge(MAX_PAYLOAD_BYTES + 1))
        );
        assert!(matches!(
            DraftPayload::from_text("{\"slots\":[],\"surprise\":1}"),
            Err(DraftError::Unreadable(_))
        ));
        // A recipe component carrying a note is a shape the table itself refuses.
        let bad = ComponentRecord {
            kind: "recipe".to_owned(),
            recipe_id: Some("r".to_owned()),
            scale: None,
            note: Some("n".to_owned()),
        };
        assert!(bad.to_domain().is_err());
        let zero = ComponentRecord {
            kind: "recipe".to_owned(),
            recipe_id: Some("r".to_owned()),
            scale: Some((0, 1)),
            note: None,
        };
        assert!(zero.to_domain().is_err());
    }

    #[test]
    fn identity_ignores_scale_and_order_and_canonicalises_installed_starters() {
        let s = snapshot_with(vec![info("r1", None), info("r-chili", Some("chili"))]);
        let a = identities(
            &s,
            &[
                MealComponent::recipe(rid("r1"), Some(Rational::new(2, 1).unwrap())),
                MealComponent::recipe(rid("r-chili"), None),
            ],
        );
        let b = identities(
            &s,
            &[
                MealComponent::freeform("starter:chili").unwrap(),
                MealComponent::recipe(rid("r1"), None),
            ],
        );
        assert_eq!(a, vec!["recipe:r1".to_owned(), "starter:chili".to_owned()]);
        let mut b_sorted = b.clone();
        b_sorted.sort();
        let mut a_sorted = a.clone();
        a_sorted.sort();
        assert_eq!(a_sorted, b_sorted);
    }

    #[test]
    fn fallbacks_and_free_notes_have_no_identity() {
        let s = snapshot_with(Vec::new());
        assert!(identities(
            &s,
            &[
                MealComponent::FrozenQuick { note: None },
                MealComponent::freeform("tacos at grandma's").unwrap(),
                MealComponent::Open { note: None },
            ],
        )
        .is_empty());
    }

    #[test]
    fn week_targets_are_unlocked_suggestions_only() {
        let mut s = slot("2026-09-28", vec![MealComponent::recipe(rid("r"), None)]);
        assert!(s.is_week_target());
        s.committed = true;
        assert!(!s.is_week_target());
        s.committed = false;
        s.origin = SlotOrigin::Chosen;
        assert!(!s.is_week_target());
        s.origin = SlotOrigin::Saved;
        assert!(
            s.is_week_target(),
            "an unlocked saved suggestion may change"
        );
        let open = slot("2026-09-28", vec![MealComponent::Open { note: None }]);
        assert!(!open.is_week_target());
        let frozen = slot(
            "2026-09-28",
            vec![MealComponent::FrozenQuick { note: None }],
        );
        assert!(
            !frozen.is_week_target(),
            "a fallback is not a recipe suggestion"
        );
        let starter = slot(
            "2026-09-28",
            vec![MealComponent::freeform("starter:chili").unwrap()],
        );
        assert!(starter.is_week_target());
    }
}
