//! Similarity v1 (OPT-007 §7): a soft Tier-5 preference against proposing a dish from the same
//! family as a meal the household has locked in. A documented heuristic, not a learned or
//! calibrated preference: every higher tier still decides first, and nothing is ever banned.
//!
//! A family is claimed only on evidence. Each rule lists evidence groups; a dish belongs to the
//! family when every group is met by a catalog ingredient identity, or — for a line with no
//! catalog ingredient — by an exact whole token of its name. A title is never evidence, and
//! salt, oil, onion or a shared protein alone establish nothing. Missing or ambiguous evidence
//! is "unknown", which is no family and no penalty. Extend [`RULES`] only with an explicit
//! example and a regression test beside it.

use std::collections::BTreeSet;

use crate::planner::candidates::RecipeView;
use crate::restriction::tokens;
use crate::{CivilDate, IngredientRef, MealSlot};

/// Bumped with any change to [`RULES`] or [`WEIGHT`]; recorded in drafts and their ledger rows.
pub const SIMILARITY_VERSION: u32 = 1;
pub const SIMILAR_TO_COMMITMENT: &str = "SIMILAR_TO_COMMITMENT";
/// Per distinct locked-in occurrence a candidate resembles, once per pair.
pub const WEIGHT: i64 = -4;

struct Group {
    catalog: &'static [&'static str],
    tokens: &'static [&'static str],
}

struct Rule {
    family: &'static str,
    groups: &'static [Group],
}

const PASTA: Group = Group {
    catalog: &[
        "ing-spaghetti",
        "ing-whole-wheat-spaghetti",
        "ing-whole-wheat-penne",
        "ing-angel-hair",
        "ing-shell-pasta",
        "ing-small-pasta",
        "ing-orzo",
    ],
    tokens: &[
        "pasta",
        "spaghetti",
        "lasagna",
        "lasagne",
        "penne",
        "macaroni",
        "rigatoni",
        "ziti",
        "linguine",
        "fettuccine",
    ],
};

const TOMATO: Group = Group {
    catalog: &[
        "ing-canned-tomatoes",
        "ing-diced-tomatoes",
        "ing-tomato",
        "ing-tomato-sauce",
        "ing-tomato-paste",
        "ing-pasta-sauce",
    ],
    tokens: &["tomato", "tomatoes", "marinara"],
};

/// Meat a dish is built on. Broths are deliberately absent: "beef broth" flavours, it does not
/// make a meat dish — and as a catalog line its name is never read by token.
const MEAT: Group = Group {
    catalog: &[
        "ing-ground-beef",
        "ing-ground-turkey",
        "ing-meatballs",
        "ing-smoked-sausage",
        "ing-turkey-kielbasa",
        "ing-pork-chops",
    ],
    tokens: &["beef", "pork", "sausage", "meatballs", "meatball"],
};

pub const MEAT_TOMATO_PASTA: &str = "meat-tomato-pasta";

const RULES: &[Rule] = &[Rule {
    family: MEAT_TOMATO_PASTA,
    groups: &[PASTA, TOMATO, MEAT],
}];

fn met(view: &RecipeView, group: &Group) -> bool {
    let catalog = view.refs.iter().any(|r| match r {
        IngredientRef::Catalog(id) => group.catalog.contains(&id.as_str()),
        IngredientRef::Custom(_) => false,
    });
    catalog
        || view.untagged.iter().any(|name| {
            tokens(name)
                .iter()
                .any(|t| group.tokens.contains(&t.as_str()))
        })
}

/// The families one dish belongs to; empty is "unknown".
pub fn families(view: &RecipeView) -> BTreeSet<&'static str> {
    RULES
        .iter()
        .filter(|rule| rule.groups.iter().all(|g| met(view, g)))
        .map(|rule| rule.family)
        .collect()
}

/// A locked-in occurrence, as the scorer compares candidates with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commitment {
    pub date: CivilDate,
    pub slot: MealSlot,
    pub keys: BTreeSet<String>,
    pub families: BTreeSet<&'static str>,
}

impl Commitment {
    pub fn of(date: CivilDate, slot: MealSlot, views: &[RecipeView]) -> Self {
        Self {
            date,
            slot,
            keys: views.iter().map(|v| v.key.clone()).collect(),
            families: views.iter().flat_map(families).collect(),
        }
    }
}

/// Whether a candidate occurrence at `(date, slot)` with `views` resembles `c`. Never itself, and
/// never the same dish: an exact repeat is `REPEAT_IN_CYCLE`'s to charge, not a family match.
pub fn resembles(date: CivilDate, slot: MealSlot, views: &[RecipeView], c: &Commitment) -> bool {
    if (date, slot) == (c.date, c.slot) || c.families.is_empty() {
        return false;
    }
    if views.iter().any(|v| c.keys.contains(&v.key)) {
        return false;
    }
    views
        .iter()
        .any(|v| families(v).iter().any(|f| c.families.contains(f)))
}

/// Two different dishes of one family: their shared ingredients are not reuse worth rewarding.
pub fn near_duplicates(a: &RecipeView, b: &RecipeView) -> bool {
    a.key != b.key && families(a).iter().any(|f| families(b).contains(f))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IngredientId;

    fn view(key: &str, catalog: &[&str], untagged: &[&str]) -> RecipeView {
        RecipeView {
            key: key.to_owned(),
            title: key.to_owned(),
            servings: None,
            scale: None,
            prep_minutes: None,
            line_names: catalog
                .iter()
                .chain(untagged)
                .map(|s| (*s).to_owned())
                .collect(),
            refs: catalog
                .iter()
                .map(|c| IngredientRef::Catalog(IngredientId::new(*c).unwrap()))
                .collect(),
            untagged: untagged.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    #[test]
    fn spaghetti_bolognese_and_a_custom_lasagna_share_a_family() {
        let spaghetti = view(
            "recipe:s",
            &["ing-spaghetti", "ing-tomato-sauce", "ing-ground-beef"],
            &[],
        );
        let lasagna = view(
            "recipe:l",
            &["ing-ground-beef", "ing-tomato-paste"],
            &["lasagna noodles"],
        );
        assert_eq!(families(&spaghetti), BTreeSet::from([MEAT_TOMATO_PASTA]));
        assert_eq!(families(&lasagna), BTreeSet::from([MEAT_TOMATO_PASTA]));
        assert!(near_duplicates(&spaghetti, &lasagna));
    }

    #[test]
    fn pantry_staples_or_a_shared_protein_alone_are_unknown() {
        assert!(families(&view("recipe:a", &[], &["salt", "olive oil", "onion"])).is_empty());
        let burger = view("recipe:b", &["ing-ground-beef"], &["buns"]);
        let chili = view("recipe:c", &["ing-ground-beef", "ing-diced-tomatoes"], &[]);
        assert!(families(&burger).is_empty());
        assert!(families(&chili).is_empty(), "no pasta, no family");
        assert!(!near_duplicates(&burger, &chili));
    }

    /// A catalog line is judged by identity: "beef broth" is not meat, and the word in its
    /// name is never read as evidence.
    #[test]
    fn a_catalog_broth_is_not_meat_by_its_name() {
        let soup = view(
            "recipe:s",
            &["ing-beef-broth", "ing-tomato", "ing-small-pasta"],
            &[],
        );
        assert!(families(&soup).is_empty());
    }

    #[test]
    fn a_title_word_is_not_evidence() {
        let mut v = view("recipe:t", &["ing-tomato"], &["noodles"]);
        v.title = "Beef lasagna".to_owned();
        assert!(families(&v).is_empty());
    }

    #[test]
    fn a_dish_never_resembles_itself_or_its_own_slot() {
        let d = crate::parse_civil_date("2026-09-28").unwrap();
        let spaghetti = view(
            "recipe:s",
            &["ing-spaghetti", "ing-tomato-sauce", "ing-ground-beef"],
            &[],
        );
        let c = Commitment::of(d, MealSlot::Dinner, std::slice::from_ref(&spaghetti));
        let other_day = d.tomorrow().unwrap();
        assert!(!resembles(
            d,
            MealSlot::Dinner,
            std::slice::from_ref(&spaghetti),
            &c
        ));
        assert!(!resembles(
            other_day,
            MealSlot::Dinner,
            std::slice::from_ref(&spaghetti),
            &c
        ));
        let lasagna = view("recipe:l", &["ing-ground-beef", "ing-tomato"], &["lasagne"]);
        assert!(resembles(other_day, MealSlot::Dinner, &[lasagna], &c));
    }
}
