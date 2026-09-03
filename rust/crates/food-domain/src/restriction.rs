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

// --- MVP-009: deterministic term matching ------------------------------------------------

/// Bumped whenever a term, exception or matching rule changes, so a planner ledger row
/// (MVP-023, invariant 20) can say which rules produced its Tier-0 rejections.
pub const RULE_VERSION: u32 = 1;

/// A phrase that contains an allergen term but is definitionally not that allergen. Listed
/// explicitly and kept tiny: every entry here is a place where the matcher chooses *not* to
/// warn, which is the direction invariant 10 says to be slowest to take.
const DAIRY_EXCEPTIONS: &[&str] = &[
    "almond milk",
    "oat milk",
    "soy milk",
    "coconut milk",
    "rice milk",
    "peanut butter",
    "almond butter",
    "cocoa butter",
    "cashew butter",
];

const PEANUT_TERMS: &[&str] = &["peanut", "peanuts", "groundnut", "groundnuts"];
const TREE_NUT_TERMS: &[&str] = &[
    "almond",
    "almonds",
    "cashew",
    "cashews",
    "walnut",
    "walnuts",
    "pecan",
    "pecans",
    "pistachio",
    "pistachios",
    "hazelnut",
    "hazelnuts",
    "macadamia",
    "macadamias",
    "brazil nut",
    "brazil nuts",
    "pine nut",
    "pine nuts",
    "chestnut",
    "chestnuts",
];
const DAIRY_TERMS: &[&str] = &[
    "milk",
    "butter",
    "buttermilk",
    "cheese",
    "cream",
    "yogurt",
    "yoghurt",
    "whey",
    "ghee",
    "paneer",
    "mozzarella",
    "parmesan",
    "cheddar",
    "ricotta",
    "feta",
    "brie",
    "mascarpone",
    "custard",
];
const EGG_TERMS: &[&str] = &["egg", "eggs", "mayonnaise", "mayo", "meringue"];
const GLUTEN_TERMS: &[&str] = &[
    "wheat",
    "flour",
    "bread",
    "breadcrumbs",
    "pasta",
    "spaghetti",
    "noodles",
    "barley",
    "rye",
    "couscous",
    "seitan",
    "semolina",
    "bulgur",
    "farro",
    "spelt",
];
const SOY_TERMS: &[&str] = &["soy", "soya", "tofu", "edamame", "tempeh", "miso", "tamari"];
const FISH_TERMS: &[&str] = &[
    "fish",
    "salmon",
    "tuna",
    "cod",
    "anchovy",
    "anchovies",
    "sardine",
    "sardines",
    "trout",
    "tilapia",
    "haddock",
    "halibut",
    "mackerel",
];
const SHELLFISH_TERMS: &[&str] = &[
    "shellfish",
    "shrimp",
    "prawn",
    "prawns",
    "crab",
    "lobster",
    "clam",
    "clams",
    "mussel",
    "mussels",
    "oyster",
    "oysters",
    "scallop",
    "scallops",
    "squid",
    "calamari",
    "crawfish",
];
const SESAME_TERMS: &[&str] = &["sesame", "tahini"];
const MEAT_TERMS: &[&str] = &[
    "beef",
    "steak",
    "pork",
    "bacon",
    "ham",
    "sausage",
    "chicken",
    "turkey",
    "lamb",
    "veal",
    "duck",
    "venison",
    "gelatin",
    "gelatine",
    "lard",
    "prosciutto",
    "salami",
    "pepperoni",
    "chorizo",
    "mince",
];
const HONEY_TERMS: &[&str] = &["honey"];

/// One row per *component*: its terms and the phrases that are definitionally not it.
/// Composed kinds (vegetarian, vegan) list their components once, so terms and exceptions
/// travel together — a fish exception added later reaches both diets without a second edit.
struct Component {
    terms: &'static [&'static str],
    exceptions: &'static [&'static str],
}

const PEANUTS: Component = Component {
    terms: PEANUT_TERMS,
    exceptions: &[],
};
const TREE_NUTS: Component = Component {
    terms: TREE_NUT_TERMS,
    exceptions: &[],
};
const DAIRY: Component = Component {
    terms: DAIRY_TERMS,
    exceptions: DAIRY_EXCEPTIONS,
};
const EGGS: Component = Component {
    terms: EGG_TERMS,
    exceptions: &[],
};
const GLUTEN: Component = Component {
    terms: GLUTEN_TERMS,
    exceptions: &[],
};
const SOY: Component = Component {
    terms: SOY_TERMS,
    exceptions: &[],
};
const FISH: Component = Component {
    terms: FISH_TERMS,
    exceptions: &[],
};
const SHELLFISH: Component = Component {
    terms: SHELLFISH_TERMS,
    exceptions: &[],
};
const SESAME: Component = Component {
    terms: SESAME_TERMS,
    exceptions: &[],
};
// Components with no kind of their own: only reachable through the composed diets.
const MEAT: Component = Component {
    terms: MEAT_TERMS,
    exceptions: &[],
};
const HONEY: Component = Component {
    terms: HONEY_TERMS,
    exceptions: &[],
};

fn components(kind: RestrictionKind) -> &'static [Component] {
    match kind {
        RestrictionKind::Peanuts => &[PEANUTS],
        RestrictionKind::TreeNuts => &[TREE_NUTS],
        RestrictionKind::Dairy => &[DAIRY],
        RestrictionKind::Eggs => &[EGGS],
        RestrictionKind::Gluten => &[GLUTEN],
        RestrictionKind::Soy => &[SOY],
        RestrictionKind::Fish => &[FISH],
        RestrictionKind::Shellfish => &[SHELLFISH],
        RestrictionKind::Sesame => &[SESAME],
        RestrictionKind::Vegetarian => &[MEAT, FISH, SHELLFISH],
        RestrictionKind::Vegan => &[MEAT, FISH, SHELLFISH, DAIRY, EGGS, HONEY],
    }
}

/// Lowercased alphanumeric tokens. Non-ASCII letters are kept as they are (so `crème` is one
/// token that matches nothing) — a documented false negative, never a false "safe".
/// `pub(crate)` for the planner's veto and preference matching (MVP-023), so there is one
/// tokenizer in the crate rather than a second that could disagree with this one.
pub(crate) fn tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Whether a hard-veto subject can ever match. The matcher works on [`tokens`], not on trimmed
/// text, so a subject with no alphanumeric character at all (`"🍝"`, `"—"`) tokenises to
/// nothing and [`contains_phrase`]'s `!phrase.is_empty()` guard rejects it forever. Public —
/// rather than replicated at each guard — so the write refusal and the read-side malformed
/// report cannot drift from the matcher they both exist to agree with.
pub fn is_vetoable(subject: &str) -> bool {
    !tokens(subject).is_empty()
}

/// `phrase` (already tokenised) occurs contiguously in `hay`.
pub(crate) fn contains_phrase(hay: &[String], phrase: &[String]) -> bool {
    !phrase.is_empty() && hay.windows(phrase.len()).any(|w| w == phrase)
}

/// First matching term for `kind` in `name`, or `None`. Per component: every contiguous
/// window equal to one of its exception phrases is *masked* (those positions are replaced by
/// an empty token that matches nothing); tokens outside a masked window are untouched, so
/// `almond milk and whole milk` still warns dairy on the second `milk`, and `almond milk`
/// still warns tree nuts because masking is per component. Never token-set subtraction.
fn first_term(kind: RestrictionKind, name: &str) -> Option<&'static str> {
    let hay = tokens(name);
    components(kind).iter().find_map(|component| {
        let masked = mask_exceptions(&hay, component.exceptions);
        component
            .terms
            .iter()
            .copied()
            .find(|term| contains_phrase(&masked, &tokens(term)))
    })
}

/// Every contiguous window equal to one of `exceptions` is cleared. Windows are located
/// against the progressively masked hay rather than the original, so two exception phrases
/// that overlap cannot clear the union of their windows — over-masking suppresses a warning,
/// the direction invariant 10 says to be slowest to take. The cost is that an overlap's
/// outcome depends on table order; every ordering is conservative relative to masking both.
/// A cleared slot is `""`, which `tokens` can never produce, so it can never anchor a
/// second phrase.
fn mask_exceptions(hay: &[String], exceptions: &[&'static str]) -> Vec<String> {
    let mut masked = hay.to_vec();
    for exception in exceptions {
        let phrase = tokens(exception);
        if phrase.is_empty() {
            continue;
        }
        let mut start = 0;
        while start + phrase.len() <= masked.len() {
            if masked[start..start + phrase.len()] == phrase[..] {
                for slot in &mut masked[start..start + phrase.len()] {
                    slot.clear();
                }
            }
            start += 1;
        }
    }
    masked
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub restriction: Restriction,
    /// Zero-based position in the line list that was assessed.
    pub line_position: usize,
    pub line_name: String,
    /// The literal term (or, for `Other`, the restriction's own wording) that matched.
    pub term: String,
}

/// What was checked and what was found. Every field is a count or a list; there is
/// deliberately no boolean that could be read as "safe" (invariant 10, AC-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestrictionAssessment {
    pub rule_version: u32,
    pub restrictions_checked: usize,
    pub lines_checked: usize,
    /// In restriction order, then line order — deterministic for the same input.
    pub conflicts: Vec<Conflict>,
    /// `Other` restrictions: matched only by their own wording, so their absence from
    /// `conflicts` means "not found by wording", nothing more.
    pub wording_only: Vec<String>,
}

/// Pure and clock-free: the same names and set always give the same assessment. This is the
/// one conflict set both the warnings UI and MVP-023's Tier-0 rejection consume.
pub fn assess<'a>(
    names: impl IntoIterator<Item = &'a str>,
    restrictions: &HouseholdRestrictions,
) -> RestrictionAssessment {
    let names: Vec<&str> = names.into_iter().collect();
    let mut conflicts = Vec::new();
    let mut wording_only = Vec::new();
    for restriction in restrictions.restrictions() {
        for (line_position, name) in names.iter().enumerate() {
            let term = match restriction {
                Restriction::Known(kind) => first_term(*kind, name).map(str::to_owned),
                Restriction::Other(text) => {
                    contains_phrase(&tokens(name), &tokens(text)).then(|| text.clone())
                }
            };
            if let Some(term) = term {
                conflicts.push(Conflict {
                    restriction: restriction.clone(),
                    line_position,
                    line_name: (*name).to_owned(),
                    term,
                });
            }
        }
        if let Restriction::Other(text) = restriction {
            wording_only.push(text.clone());
        }
    }
    RestrictionAssessment {
        rule_version: RULE_VERSION,
        restrictions_checked: restrictions.restrictions().len(),
        lines_checked: names.len(),
        conflicts,
        wording_only,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- MVP-009 ------------------------------------------------------------------------

    fn set(items: impl IntoIterator<Item = Restriction>) -> HouseholdRestrictions {
        HouseholdRestrictions::new(items)
    }

    fn known(kind: RestrictionKind) -> Restriction {
        Restriction::Known(kind)
    }

    /// Overlapping exception phrases must not clear the union of their windows. Unreachable on
    /// today's `DAIRY_EXCEPTIONS` — no entry starts with `milk` or `butter`, so no two can
    /// overlap — so the pair is synthetic: the property is a guard, not an accident of the
    /// table, and adding `"butter milk"` or `"milk chocolate"` must not silently break it.
    #[test]
    fn overlapping_exceptions_do_not_mask_the_shared_token_twice() {
        let hay = tokens("butter milk chocolate");
        let masked = mask_exceptions(&hay, &["butter milk", "milk chocolate"]);
        assert_eq!(masked, vec!["", "", "chocolate"]);
        // Order-dependent when two phrases overlap, and conservative either way: the reverse
        // order leaves `butter` warning instead of `chocolate`, never neither.
        let reversed = mask_exceptions(&hay, &["milk chocolate", "butter milk"]);
        assert_eq!(reversed, vec!["butter", "", ""]);
    }

    fn terms_of(kind: RestrictionKind, names: &[&str]) -> Vec<String> {
        assess(names.iter().copied(), &set([known(kind)]))
            .conflicts
            .into_iter()
            .map(|c| c.term)
            .collect()
    }

    /// One literal row per kind — a positive and a negative — so each mapping is a decision
    /// pinned in the test, not an accident of the table.
    #[test]
    fn every_kind_has_a_positive_and_a_negative_fixture() {
        use RestrictionKind::*;
        let rows: [(RestrictionKind, &str, &str); 11] = [
            (Peanuts, "peanut butter", "pea shoots"),
            (TreeNuts, "almond milk", "nutmeg"),
            (Dairy, "Cheddar cheese", "almond milk"),
            (Eggs, "2 eggs", "eggplant"),
            (Gluten, "wheat flour", "rice"),
            (Soy, "firm tofu", "sorghum"),
            (Fish, "fish sauce", "fishless fingers"),
            (Shellfish, "prawns", "shellac"),
            (Vegetarian, "bacon", "tofu"),
            (Vegan, "honey", "oat milk"),
            // Sesame's negative is omitted below on purpose; see the next assertion.
            (Sesame, "tahini", "shellac"),
        ];
        assert_eq!(rows.len(), RestrictionKind::ALL.len());
        for (kind, yes, no) in rows {
            assert!(
                first_term(kind, yes).is_some(),
                "{kind:?} must warn on {yes:?}"
            );
            assert!(
                first_term(kind, no).is_none(),
                "{kind:?} must not warn on {no:?}"
            );
        }
        // Documented false positive, asserted so it is a decision, not an accident: the
        // matcher cannot read negation, and warning is the direction invariant 10 prefers.
        assert_eq!(first_term(Sesame, "sesame-free oil"), Some("sesame"));
    }

    /// Hard-coded lengths so an addition to any table is noticed here, not just compiled.
    #[test]
    fn every_term_matches_its_own_kind() {
        use RestrictionKind::*;
        let tables: [(RestrictionKind, &[&str], usize); 9] = [
            (Peanuts, PEANUT_TERMS, 4),
            (TreeNuts, TREE_NUT_TERMS, 20),
            (Dairy, DAIRY_TERMS, 18),
            (Eggs, EGG_TERMS, 5),
            (Gluten, GLUTEN_TERMS, 15),
            (Soy, SOY_TERMS, 7),
            (Fish, FISH_TERMS, 13),
            (Shellfish, SHELLFISH_TERMS, 17),
            (Sesame, SESAME_TERMS, 2),
        ];
        for (kind, table, len) in tables {
            assert_eq!(table.len(), len, "{kind:?} table length changed");
            for term in table {
                assert_eq!(first_term(kind, term), Some(*term), "{kind:?} / {term:?}");
            }
        }
        assert_eq!(MEAT_TERMS.len(), 20);
        assert_eq!(HONEY_TERMS.len(), 1);
        for term in MEAT_TERMS.iter().chain(FISH_TERMS).chain(SHELLFISH_TERMS) {
            assert_eq!(
                first_term(Vegetarian, term),
                Some(*term),
                "vegetarian / {term:?}"
            );
            assert_eq!(first_term(Vegan, term), Some(*term), "vegan / {term:?}");
        }
        for term in DAIRY_TERMS.iter().chain(EGG_TERMS).chain(HONEY_TERMS) {
            assert_eq!(first_term(Vegan, term), Some(*term), "vegan / {term:?}");
            assert_eq!(first_term(Vegetarian, term), None, "vegetarian / {term:?}");
        }
        assert_eq!(components(Vegetarian).len(), 3);
        assert_eq!(components(Vegan).len(), 6);
    }

    #[test]
    fn composed_kinds_inherit_every_component_exception() {
        assert_eq!(first_term(RestrictionKind::Vegan, "almond milk"), None);
        assert_eq!(first_term(RestrictionKind::Vegan, "peanut butter"), None);
        let reachable: usize = components(RestrictionKind::Vegan)
            .iter()
            .map(|c| c.exceptions.len())
            .sum();
        assert_eq!(reachable, DAIRY_EXCEPTIONS.len());
        assert_eq!(DAIRY_EXCEPTIONS.len(), 9);
    }

    /// Adversarial: masking is per window, never token-set subtraction.
    #[test]
    fn an_exception_masks_only_its_own_window() {
        assert_eq!(
            first_term(RestrictionKind::Dairy, "almond milk and whole milk"),
            Some("milk")
        );
        assert_eq!(
            first_term(RestrictionKind::Dairy, "peanut butter, salted butter"),
            Some("butter")
        );
    }

    #[test]
    fn matching_is_whole_token_case_folded_and_punctuation_blind() {
        assert_eq!(
            first_term(RestrictionKind::Dairy, "BUTTER, softened"),
            Some("butter")
        );
        assert_eq!(
            first_term(RestrictionKind::Dairy, "buttermilk"),
            Some("buttermilk")
        );
        assert_eq!(first_term(RestrictionKind::Dairy, "butterfly"), None);
    }

    #[test]
    fn peanut_butter_warns_peanuts_and_not_dairy() {
        assert_eq!(
            first_term(RestrictionKind::Peanuts, "peanut butter"),
            Some("peanut")
        );
        assert_eq!(first_term(RestrictionKind::Dairy, "peanut butter"), None);
    }

    #[test]
    fn almond_milk_warns_tree_nuts_and_not_dairy_but_vegan_still_sees_animal_milk() {
        assert_eq!(
            first_term(RestrictionKind::TreeNuts, "almond milk"),
            Some("almond")
        );
        assert_eq!(first_term(RestrictionKind::Dairy, "almond milk"), None);
        assert_eq!(first_term(RestrictionKind::Vegan, "almond milk"), None);
        assert_eq!(
            first_term(RestrictionKind::Vegan, "goat milk"),
            Some("milk")
        );
    }

    #[test]
    fn other_restrictions_match_by_wording_only_and_are_listed() {
        let a = assess(
            ["tomato", "nightshades mix"],
            &set([Restriction::other("nightshades").unwrap()]),
        );
        assert_eq!(
            a.conflicts,
            vec![Conflict {
                restriction: Restriction::Other("nightshades".to_owned()),
                line_position: 1,
                line_name: "nightshades mix".to_owned(),
                term: "nightshades".to_owned(),
            }]
        );
        assert_eq!(a.wording_only, vec!["nightshades"]);
        let b = assess(
            ["lean red meat"],
            &set([Restriction::other("red meat").unwrap()]),
        );
        assert_eq!(b.conflicts.len(), 1);
        assert_eq!(b.conflicts[0].term, "red meat");
        // Partial wording is not a match: "red" alone does not contain "red meat".
        let c = assess(
            ["red pepper"],
            &set([Restriction::other("red meat").unwrap()]),
        );
        assert!(c.conflicts.is_empty());
        assert_eq!(c.wording_only, vec!["red meat"]);
    }

    #[test]
    fn assessment_is_ordered_and_deterministic() {
        let dairy = known(RestrictionKind::Dairy);
        let peanuts = known(RestrictionKind::Peanuts);
        let lines = ["butter", "rice", "peanuts and cream"];
        let a = assess(lines, &set([dairy.clone(), peanuts.clone()]));
        let key: Vec<(Restriction, usize)> = a
            .conflicts
            .iter()
            .map(|c| (c.restriction.clone(), c.line_position))
            .collect();
        assert_eq!(
            key,
            vec![(dairy.clone(), 0), (dairy.clone(), 2), (peanuts.clone(), 2)]
        );
        assert_eq!(a, assess(lines, &set([dairy.clone(), peanuts.clone()])));
        assert_eq!(a.rule_version, RULE_VERSION);
        assert_eq!(a.lines_checked, 3);
        assert_eq!(a.restrictions_checked, 2);
        // Stored first-seen order, not sorted: swapping the set swaps the conflict order.
        let two = ["butter", "peanuts"];
        let forward = assess(two, &set([dairy.clone(), peanuts.clone()]));
        let reversed = assess(two, &set([peanuts.clone(), dairy.clone()]));
        assert_eq!(forward.conflicts.len(), 2);
        assert_eq!(forward.conflicts[0].restriction, dairy);
        assert_eq!(reversed.conflicts[0].restriction, peanuts);
        assert_ne!(forward.conflicts, reversed.conflicts);
    }

    #[test]
    fn empty_restrictions_check_nothing() {
        let a = assess(["butter", "peanuts"], &set([]));
        assert_eq!(a.restrictions_checked, 0);
        assert!(a.conflicts.is_empty());
        assert_eq!(a.lines_checked, 2);
        let b = assess([], &set([known(RestrictionKind::Dairy)]));
        assert_eq!(b.lines_checked, 0);
        assert_eq!(b.restrictions_checked, 1);
        assert!(b.conflicts.is_empty());
    }

    /// Expected-to-pass: pins bounded behaviour. Blank is rejected upstream; the matcher
    /// must still not panic on it.
    #[test]
    fn non_ascii_names_never_panic_and_stay_unmatched() {
        let all = set(RestrictionKind::ALL.map(known));
        let long = "x".repeat(10 * 1024);
        for name in ["crème fraîche", "", "   ", "—", long.as_str()] {
            let a = assess([name], &all);
            assert!(a.conflicts.is_empty(), "{name:?} must not match");
            assert_eq!(a.lines_checked, 1);
        }
        // The `Other` path with wording that tokenises to nothing must not match everything.
        let punct = set([Restriction::Other("—".to_owned())]);
        assert!(assess(["butter"], &punct).conflicts.is_empty());
        // And a non-ASCII `Other` still matches its own wording by whole token.
        let creme = set([Restriction::other("crème").unwrap()]);
        assert_eq!(assess(["Crème fraîche"], &creme).conflicts.len(), 1);
    }

    /// Expected-to-pass, compile-level documentation: the assessment is counts and lists
    /// only. Its `Debug` rendering names no field that reads as an assurance.
    #[test]
    fn the_assessment_has_no_safety_flag() {
        let a = assess(["rice"], &set([known(RestrictionKind::Dairy)]));
        let RestrictionAssessment {
            rule_version,
            restrictions_checked,
            lines_checked,
            conflicts,
            wording_only,
        } = &a;
        assert_eq!(*rule_version, 1);
        assert_eq!(*restrictions_checked, 1);
        assert_eq!(*lines_checked, 1);
        assert!(conflicts.is_empty());
        assert!(wording_only.is_empty());
        let rendered = format!("{a:?}").to_lowercase();
        assert!(!rendered.contains("safe"), "{rendered}");
        assert!(!rendered.contains("ok"), "{rendered}");
        assert_eq!(
            terms_of(RestrictionKind::Dairy, &["rice"]),
            Vec::<String>::new()
        );
    }

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
