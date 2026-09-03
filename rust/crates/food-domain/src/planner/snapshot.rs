//! The planner's whole input, read by storage in one transaction and identified by a hash of
//! its canonical text (PRD §7.7 "input snapshot hash"). Plain structs, no serde: the text
//! encoder is hand-written so the hash depends on nothing but the fields it names.

use std::collections::BTreeMap;

use household_core::{HouseholdId, MemberId, Policy};

use crate::{
    format_civil_date, is_vetoable, CivilDate, HouseholdRestrictions, IngredientRef, MealComponent,
    MealScope, MealSlot, MemberPreferences, PlannedMeal, RecipeId, Restriction, StarterRecipe,
};

/// Search knobs. The defaults are this card's choices, recorded in the card's decision gates
/// and open to `MVP-025`'s benchmark — never asserted as product truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchParams {
    pub beam_width: usize,
    pub candidates_per_slot: usize,
    /// Slots dated before `today + horizon` are near-term commitments (§9.9).
    pub commitment_horizon_days: u32,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            beam_width: 8,
            candidates_per_slot: 12,
            commitment_horizon_days: 2,
        }
    }
}

/// The `food.*` policies the controller interprets (§7.3). Anything else is carried as an
/// unknown type so the assessment can say it was seen and not applied.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoodPolicies {
    pub dining_out_enabled: bool,
    /// Subjects a household has hard-vetoed; matched by whole token against titles and line
    /// names at Tier 0, never weighted.
    pub hard_vetoes: Vec<String>,
    /// Minutes available per slot, when the household has said so.
    pub slot_windows: BTreeMap<MealSlot, u32>,
    /// The household explicitly reviewed its restriction list and confirmed it is complete
    /// as stored (§10: reviewed absence is confirmed absence, not a skipped check).
    pub restrictions_reviewed: bool,
    pub unknown_policy_types: Vec<String>,
}

impl FoodPolicies {
    pub const DINING_OUT: &'static str = "food.dining_out";
    pub const HARD_VETO: &'static str = "food.hard_veto";
    pub const RESTRICTIONS_REVIEWED: &'static str = "food.restrictions_reviewed";
    pub const SLOT_WINDOW: &'static str = "food.slot_window";

    /// Disabled policies are known but not applied; a malformed parameter set on a known
    /// type is reported as unknown rather than half-applied. Output lists are sorted so the
    /// canonical text does not depend on storage order.
    pub fn from_policies(policies: &[Policy]) -> Self {
        let mut out = Self::default();
        for policy in policies {
            if policy.domain != "food" {
                out.unknown_policy_types.push(policy.policy_type.clone());
                continue;
            }
            match policy.policy_type.as_str() {
                Self::DINING_OUT => out.dining_out_enabled |= policy.enabled,
                Self::RESTRICTIONS_REVIEWED => out.restrictions_reviewed |= policy.enabled,
                // A subject the matcher can never match is malformed, not "known but not
                // applied": it is reported whether or not the policy is enabled, which is how
                // the `SLOT_WINDOW` arm below already treats its own malformed cases. Only a
                // *well-formed* disabled policy is silently ignored. The test is `is_vetoable`,
                // not `trim().is_empty()`, because `veto_hit` matches through `tokens` — a
                // subject with no alphanumeric character survives a trim and then matches
                // nothing for the life of the row.
                Self::HARD_VETO => match policy.parameters.get("subject") {
                    Some(subject) if policy.enabled && is_vetoable(subject) => {
                        out.hard_vetoes.push(subject.trim().to_owned());
                    }
                    Some(subject) if !is_vetoable(subject) => {
                        out.unknown_policy_types.push(policy.policy_type.clone());
                    }
                    Some(_) => {}
                    None => out.unknown_policy_types.push(policy.policy_type.clone()),
                },
                Self::SLOT_WINDOW => {
                    let slot = policy
                        .parameters
                        .get("slot")
                        .and_then(|s| MealSlot::parse(s).ok());
                    let minutes = policy
                        .parameters
                        .get("minutes")
                        .and_then(|m| m.parse::<u32>().ok())
                        .filter(|m| *m > 0);
                    match (slot, minutes) {
                        (Some(slot), Some(minutes)) if policy.enabled => {
                            out.slot_windows.insert(slot, minutes);
                        }
                        (Some(_), Some(_)) => {}
                        _ => out.unknown_policy_types.push(policy.policy_type.clone()),
                    }
                }
                other => out.unknown_policy_types.push(other.to_owned()),
            }
        }
        out.hard_vetoes.sort_unstable();
        out.hard_vetoes.dedup();
        out.unknown_policy_types.sort_unstable();
        out.unknown_policy_types.dedup();
        out
    }
}

/// What the planner needs to know about one household recipe, without the instructions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeCandidateInfo {
    pub id: RecipeId,
    pub title: String,
    pub servings: Option<u32>,
    pub prep_minutes: Option<u32>,
    pub line_names: Vec<String>,
    pub ingredient_refs: Vec<IngredientRef>,
    pub starter_slug: Option<String>,
}

/// Every collection is held in canonical order by the loader (documented per field), so the
/// canonical text — and the hash — is a function of the state, not of insertion order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningSnapshot {
    pub household_id: HouseholdId,
    pub anchor: CivilDate,
    pub length_days: u32,
    pub scope: MealScope,
    pub today: CivilDate,
    /// Sorted by id.
    pub members: Vec<MemberId>,
    pub restrictions: HouseholdRestrictions,
    /// By member id.
    pub preferences: Vec<(MemberId, MemberPreferences)>,
    pub policies: FoodPolicies,
    /// Active recipes, by id.
    pub recipes: Vec<RecipeCandidateInfo>,
    /// Starter entries not yet installed (no active recipe carries the slug), by slug.
    pub starter: Vec<StarterRecipe>,
    /// Occurrences inside the window, by date then planned-meal id.
    pub existing: Vec<PlannedMeal>,
    /// Occurrences in `[anchor − 14, anchor − 1]`, by date then planned-meal id.
    pub history: Vec<PlannedMeal>,
    /// Sorted, catalog refs then custom.
    pub pantry_marked: Vec<IngredientRef>,
}

impl PlanningSnapshot {
    /// The cycle's dates: `length_days` from the anchor. Infallible for the same reason
    /// `PlanningCycle::dates` is — the loader built this from a cycle that already fits.
    pub fn dates(&self) -> Vec<CivilDate> {
        let mut dates = Vec::with_capacity(self.length_days as usize);
        let mut date = self.anchor;
        for step in 0..self.length_days {
            dates.push(date);
            if step + 1 < self.length_days {
                date = date.tomorrow().expect("loader proved the cycle fits");
            }
        }
        dates
    }

    pub fn existing_at(&self, date: CivilDate, slot: MealSlot) -> Option<&PlannedMeal> {
        self.existing
            .iter()
            .find(|m| m.date() == date && m.slot() == slot)
    }

    pub fn preferences_of(&self, member: &MemberId) -> Option<&MemberPreferences> {
        self.preferences
            .iter()
            .find(|(m, _)| m == member)
            .map(|(_, p)| p)
    }

    /// One line per fact. Ids and dates as text, enums by token; nested lists on one line
    /// each with `|` between items. Changing any field changes at least one line.
    pub fn canonical_text(&self) -> String {
        let mut out = String::new();
        let mut line = |key: &str, value: String| {
            out.push_str(key);
            out.push('=');
            out.push_str(&value);
            out.push('\n');
        };
        line("household", self.household_id.as_str().to_owned());
        line("anchor", format_civil_date(self.anchor));
        line("length_days", self.length_days.to_string());
        line(
            "scope",
            join(self.scope.slots().iter().map(|s| s.as_str().to_owned())),
        );
        line("today", format_civil_date(self.today));
        line(
            "members",
            join(self.members.iter().map(|m| m.as_str().to_owned())),
        );
        line(
            "restrictions",
            join(
                self.restrictions
                    .restrictions()
                    .iter()
                    .map(restriction_text),
            ),
        );
        for (member, prefs) in &self.preferences {
            line(
                &format!("preferences:{}", member.as_str()),
                join(
                    prefs
                        .preferences()
                        .iter()
                        .map(|p| format!("{}:{}", p.sentiment().as_str(), p.subject())),
                ),
            );
        }
        line(
            "policy.dining_out",
            self.policies.dining_out_enabled.to_string(),
        );
        line(
            "policy.hard_vetoes",
            join(self.policies.hard_vetoes.iter().cloned()),
        );
        line(
            "policy.slot_windows",
            join(
                self.policies
                    .slot_windows
                    .iter()
                    .map(|(s, m)| format!("{}:{m}", s.as_str())),
            ),
        );
        // Emitted only when set: every pre-existing snapshot's canonical text — and every
        // pinned fixture hash — stays byte-identical, and presence/absence of the labeled
        // line keeps the encoding injective.
        if self.policies.restrictions_reviewed {
            line("policy.restrictions_reviewed", "true".to_owned());
        }
        line(
            "policy.unknown",
            join(self.policies.unknown_policy_types.iter().cloned()),
        );
        for r in &self.recipes {
            line(
                &format!("recipe:{}", r.id.as_str()),
                format!(
                    "{}|{}|{}|{}|{}|{}",
                    r.title,
                    opt(r.servings),
                    opt(r.prep_minutes),
                    r.line_names.join(";"),
                    join(r.ingredient_refs.iter().map(ref_text)),
                    r.starter_slug.clone().unwrap_or_default(),
                ),
            );
        }
        for s in &self.starter {
            line(
                &format!("starter:{}", s.slug),
                format!(
                    "{}|{}|{}|{}|{}",
                    s.title,
                    opt(s.servings),
                    opt(s.prep_minutes),
                    s.lines
                        .iter()
                        .map(|l| l.name())
                        .collect::<Vec<_>>()
                        .join(";"),
                    join(s.lines.iter().filter_map(|l| l.ingredient()).map(ref_text)),
                ),
            );
        }
        for m in &self.existing {
            line(&format!("existing:{}", meal_key(m)), meal_text(m));
        }
        for m in &self.history {
            line(&format!("history:{}", meal_key(m)), meal_text(m));
        }
        line("pantry", join(self.pantry_marked.iter().map(ref_text)));
        out
    }

    /// FNV-1a 64 over the canonical text, lowercase hex. An identity for "same input", not a
    /// security property: collision resistance is not required of it anywhere.
    pub fn snapshot_hash(&self) -> String {
        format!("{:016x}", fnv1a_64(self.canonical_text().as_bytes()))
    }
}

pub(crate) fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn join(items: impl Iterator<Item = String>) -> String {
    items.collect::<Vec<_>>().join("|")
}

fn opt(value: Option<u32>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

pub(crate) fn ref_text(r: &IngredientRef) -> String {
    match r {
        IngredientRef::Catalog(id) => format!("catalog:{}", id.as_str()),
        IngredientRef::Custom(id) => format!("custom:{}", id.as_str()),
    }
}

fn restriction_text(r: &Restriction) -> String {
    match r {
        Restriction::Known(kind) => kind.as_str().to_owned(),
        Restriction::Other(text) => format!("other:{text}"),
    }
}

/// `kind:recipe-id:scale` or `kind:note`; the shape every plan text and rejection uses.
pub(crate) fn component_text(c: &MealComponent) -> String {
    match c {
        MealComponent::Recipe { recipe_id, scale } => format!(
            "recipe:{}:{}",
            recipe_id.as_str(),
            scale
                .map(|s| s.to_string())
                .unwrap_or_else(|| "1/1".to_owned())
        ),
        MealComponent::Leftovers { note }
        | MealComponent::DiningOut { note }
        | MealComponent::FrozenQuick { note }
        | MealComponent::Open { note } => {
            format!("{}:{}", c.kind_str(), note.clone().unwrap_or_default())
        }
        MealComponent::Freeform { note } => format!("freeform:{note}"),
    }
}

/// Public alongside the DTO layer's needs: the decision recorder's ledger payload names the
/// swapped components in exactly the shape every plan text and rejection uses.
pub fn components_text(components: &[MealComponent]) -> String {
    components
        .iter()
        .map(component_text)
        .collect::<Vec<_>>()
        .join(",")
}

fn meal_key(m: &PlannedMeal) -> String {
    format!("{}:{}", format_civil_date(m.date()), m.slot().as_str())
}

fn meal_text(m: &PlannedMeal) -> String {
    format!(
        "{}|{}|{}",
        m.id().as_str(),
        m.locked(),
        components_text(m.components())
    )
}
