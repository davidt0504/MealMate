//! MVP-025 synthetic household fixtures (PRD §18, pivot prompt). Version-controlled,
//! deterministic, and entirely synthetic: no real household data, names, or restrictions.
//! Compiled for tests and for the `beam_width` benchmark example (feature `fixtures`), and
//! kept reusable for a later MILP/CP-SAT comparison (PRD §22) — which is why each fixture
//! carries the full [`Recipe`]s behind its snapshot, projected through [`candidate_info_of`]
//! exactly as the storage loader projects them.
//!
//! See `README.md` beside this file for the fixture table (stressor and source per entry).

use std::collections::BTreeMap;

use household_core::{HouseholdId, MemberId};

use crate::planner::snapshot::{FoodPolicies, PlanningSnapshot, RecipeCandidateInfo};
use crate::planner::PlanningResult;
use crate::{
    parse_civil_date, CivilDate, HouseholdRestrictions, IdentityInfo, IngredientId, IngredientLine,
    IngredientRef, MealComponent, MealScope, MealSlot, MemberPreference, MemberPreferences,
    PlannedMeal, PlannedMealId, ProvenanceKind, Quantity, QuantityRange, Rational, Recipe,
    RecipeId, RecipeProvenance, Restriction, RestrictionKind, Sentiment, ShoppingInput,
    StarterRecipe, Unit, UnitKind,
};

/// Which source list names the fixture (card AC-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureSource {
    /// PRD v3 §18's nine-household list.
    Prd18,
    /// The two `docs/CODEX_CLAUDE_PIVOT_PROMPT.md` "Tests" adds that §18 omits.
    PivotPrompt,
}

/// One synthetic household: the planner's input plus the full recipes behind
/// `snapshot.recipes`, so shopping tests and a later MILP/CP-SAT comparison (PRD §22)
/// share one source of truth.
#[derive(Debug, Clone)]
pub struct PlannerFixture {
    pub name: &'static str,
    pub source: FixtureSource,
    /// What the fixture is meant to stress (card AC-1).
    pub stressor: &'static str,
    pub snapshot: PlanningSnapshot,
    pub recipes: Vec<Recipe>,
}

/// Every fixture, in a stable documented order. Deterministic: two calls build equal values.
pub fn all() -> Vec<PlannerFixture> {
    vec![
        cold_start(),
        conservative_household(),
        high_variety_household(),
        multiple_strong_dislikes(),
        busy_week(),
        many_locked_meals(),
        sparse_pantry(),
        restrictions_set_and_skipped(),
        leftovers_fallback_heavy(),
        repetitive_meal_library(),
        intentionally_open_nights(),
    ]
}

/// The fixture named `name`; panics listing the known names so a typo fails loudly. Builds
/// the set once per process rather than per lookup: `all()` itself stays uncached, because
/// `fixtures_are_deterministic` compares two independent builds.
pub fn by_name(name: &str) -> PlannerFixture {
    static CACHE: std::sync::OnceLock<Vec<PlannerFixture>> = std::sync::OnceLock::new();
    let fixtures = CACHE.get_or_init(all);
    fixtures
        .iter()
        .find(|f| f.name == name)
        .cloned()
        .unwrap_or_else(|| {
            let known: Vec<&str> = fixtures.iter().map(|f| f.name).collect();
            panic!("unknown fixture {name:?}; known fixtures: {known:?}")
        })
}

/// The storage loader's projection of a recipe into what the planner reads — field for field
/// the shape `kimatta-storage`'s snapshot loader builds, so a fixture's `snapshot.recipes`
/// cannot drift from its `recipes`.
pub fn candidate_info_of(r: &Recipe) -> RecipeCandidateInfo {
    RecipeCandidateInfo {
        id: r.id().clone(),
        title: r.title().to_owned(),
        servings: r.servings(),
        prep_minutes: r.prep_minutes(),
        line_names: r.lines().iter().map(|l| l.name().to_owned()).collect(),
        ingredient_refs: r
            .lines()
            .iter()
            .filter_map(IngredientLine::ingredient)
            .cloned()
            .collect(),
        starter_slug: r.provenance().starter_slug().map(str::to_owned),
    }
}

/// The same household with restrictions left unconfigured — the "restrictions skipped" half
/// of the §18 pair. Skipping is the *absence* of configuration, so the variant is empty
/// restrictions, which `coverage::sufficiency` reports as `RESTRICTIONS_NOT_CONFIGURED`.
pub fn restrictions_skipped_variant(f: &PlannerFixture) -> PlanningSnapshot {
    let mut snapshot = f.snapshot.clone();
    snapshot.restrictions = HouseholdRestrictions::new([]);
    snapshot
}

/// A [`ShoppingInput`] over the fixture's cycle window with the planner's proposed meals
/// applied, the fixture's full recipes, and a deterministic identity table covering every
/// catalog ref the recipes name (both the categorised and the uncategorised branch).
pub fn shopping_input_of(f: &PlannerFixture, result: &PlanningResult) -> ShoppingInput {
    let dates = f.snapshot.dates();
    let from = *dates.first().expect("every fixture has at least one date");
    let to = *dates.last().expect("every fixture has at least one date");
    let meals = result
        .proposed
        .iter()
        .enumerate()
        .map(|(i, p)| {
            PlannedMeal::new(
                PlannedMealId::new(format!("fx-pm-{i}")).expect("non-blank id"),
                f.snapshot.household_id.clone(),
                p.date,
                p.slot,
                p.components.clone(),
                false,
            )
            .expect("proposed components are valid meal components")
        })
        .collect();
    let mut identities: Vec<(IngredientRef, IdentityInfo)> = Vec::new();
    for r in &f.recipes {
        for ingredient in r.lines().iter().filter_map(IngredientLine::ingredient) {
            if identities.iter().any(|(seen, _)| seen == ingredient) {
                continue;
            }
            let id_text = match ingredient {
                IngredientRef::Catalog(id) => id.as_str().to_owned(),
                IngredientRef::Custom(id) => id.as_str().to_owned(),
            };
            // Deterministic split so both the categorised and uncategorised grouping
            // branches occur: even-length identities get a store category, odd-length none.
            let store_category = (id_text.len() % 2 == 0).then(|| "fx-aisle".to_owned());
            identities.push((
                ingredient.clone(),
                IdentityInfo {
                    name: id_text,
                    store_category,
                },
            ));
        }
    }
    ShoppingInput {
        from,
        to,
        meals,
        recipes: f.recipes.clone(),
        identities,
        pantry_marked: f.snapshot.pantry_marked.clone(),
    }
}

// --- Construction helpers (local by design: `tests.rs` helpers are cfg(test)-private) -------

/// Every fixture's seven-day cycle starts here; a fixed date keeps hashes pinned.
const ANCHOR: &str = "2026-08-29";

fn d(raw: &str) -> CivilDate {
    parse_civil_date(raw).expect("fixture dates are valid")
}

fn cat(id: &str) -> IngredientRef {
    IngredientRef::Catalog(IngredientId::new(id).expect("fixture ids are non-blank"))
}

fn member(id: &str) -> MemberId {
    MemberId::new(id).expect("fixture ids are non-blank")
}

fn exact(numer: u32, denom: u32) -> Quantity {
    Quantity::Exact(Rational::new(numer, denom).expect("fixture denominators are non-zero"))
}

fn range(min: (u32, u32), max: (u32, u32)) -> Quantity {
    let lo = Rational::new(min.0, min.1).expect("fixture denominators are non-zero");
    let hi = Rational::new(max.0, max.1).expect("fixture denominators are non-zero");
    Quantity::Range(QuantityRange::new(lo, hi).expect("fixture ranges are ordered"))
}

/// A resolvable line: name doubles as its catalog identity, quantity/unit as given.
fn line(name: &str, quantity: Quantity, unit: Unit) -> IngredientLine {
    IngredientLine::new(name, name, Some(cat(name)), quantity, unit, None, false)
        .expect("fixture lines are non-blank")
}

/// The common case: one exact count of each named ingredient, no unit.
fn simple_lines(names: &[&str]) -> Vec<IngredientLine> {
    names
        .iter()
        .map(|n| line(n, exact(1, 1), Unit::None))
        .collect()
}

fn fx_recipe(
    household: &HouseholdId,
    id: &str,
    title: &str,
    prep_minutes: Option<u32>,
    servings: Option<u32>,
    lines: Vec<IngredientLine>,
) -> Recipe {
    Recipe::new(
        RecipeId::new(id).expect("fixture ids are non-blank"),
        household.clone(),
        title,
        servings,
        prep_minutes,
        String::new(),
        lines,
        RecipeProvenance::new(ProvenanceKind::Authored, None, None, None)
            .expect("fixture provenance is well-formed"),
    )
    .expect("fixture recipes are well-formed")
}

fn starter(slug: &str, title: &str, prep_minutes: Option<u32>, names: &[&str]) -> StarterRecipe {
    StarterRecipe {
        slug: slug.to_owned(),
        cook_review: None,
        expected_conflicts: vec![],
        title: title.to_owned(),
        servings: Some(4),
        prep_minutes,
        instructions: String::new(),
        lines: simple_lines(names),
        provenance: RecipeProvenance::with_rights(
            ProvenanceKind::Starter,
            None,
            None,
            None,
            None,
            Some(slug.to_owned()),
        )
        .expect("fixture provenance is well-formed"),
    }
}

fn meal(
    household: &HouseholdId,
    id: &str,
    date: &str,
    slot: MealSlot,
    components: Vec<MealComponent>,
    locked: bool,
) -> PlannedMeal {
    PlannedMeal::new(
        PlannedMealId::new(id).expect("fixture ids are non-blank"),
        household.clone(),
        d(date),
        slot,
        components,
        locked,
    )
    .expect("fixture meals are well-formed")
}

fn rc(id: &str) -> MealComponent {
    MealComponent::recipe(RecipeId::new(id).expect("fixture ids are non-blank"), None)
}

fn prefs(id: &str, items: &[(Sentiment, &str)]) -> (MemberId, MemberPreferences) {
    (
        member(id),
        MemberPreferences::new(items.iter().map(|(s, subject)| {
            MemberPreference::new(*s, *subject).expect("fixture subjects are non-blank")
        })),
    )
}

/// Seven dinner-only days from [`ANCHOR`], one member, nothing else configured. Every
/// fixture starts here and states only its deviations, so each stressor is legible.
fn base(household: &HouseholdId, recipes: &[Recipe]) -> PlanningSnapshot {
    PlanningSnapshot {
        household_id: household.clone(),
        anchor: d(ANCHOR),
        length_days: 7,
        scope: MealScope::dinner_only(),
        today: d(ANCHOR),
        members: vec![member("fx-m1")],
        restrictions: HouseholdRestrictions::new([]),
        preferences: vec![],
        policies: FoodPolicies::default(),
        recipes: recipes.iter().map(candidate_info_of).collect(),
        starter: vec![],
        existing: vec![],
        history: vec![],
        pantry_marked: vec![],
    }
}

fn household_of(name: &str) -> HouseholdId {
    HouseholdId::new(format!("fx-{name}")).expect("non-blank id")
}

// --- The fixtures ---------------------------------------------------------------------------

/// §18 №1: a brand-new household. Empty library, no preferences, restrictions unconfigured;
/// only starter content stands between the planner and an empty cycle.
fn cold_start() -> PlannerFixture {
    let hh = household_of("cold-start");
    let recipes = Vec::new();
    let mut snapshot = base(&hh, &recipes);
    snapshot.starter = vec![
        starter(
            "fx-lentil-stew",
            "Lentil stew",
            Some(35),
            &["lentils", "carrot", "onion"],
        ),
        starter("fx-oat-bowl", "Oat bowl", Some(10), &["oats", "milk"]),
        starter(
            "fx-veg-curry",
            "Vegetable curry",
            Some(30),
            &["potato", "peas", "rice"],
        ),
    ];
    PlannerFixture {
        name: "cold_start",
        source: FixtureSource::Prd18,
        stressor: "empty library, no preferences, restrictions unconfigured, starter present",
        snapshot,
        recipes,
    }
}

/// §18 №2: a conservative household. Six familiar recipes, three narrow likes, and a history
/// that already repeats the same two dinners — repetition is tolerated here, not a defect.
fn conservative_household() -> PlannerFixture {
    let hh = household_of("conservative");
    let recipes = vec![
        fx_recipe(
            &hh,
            "r-cons-1",
            "Bean chili",
            Some(30),
            Some(4),
            simple_lines(&["beans", "tomato", "onion"]),
        ),
        fx_recipe(
            &hh,
            "r-cons-2",
            "Cheese omelette",
            Some(10),
            Some(2),
            simple_lines(&["eggs", "cheese"]),
        ),
        fx_recipe(
            &hh,
            "r-cons-3",
            "Mac and cheese",
            Some(20),
            Some(4),
            simple_lines(&["pasta", "cheese", "milk"]),
        ),
        fx_recipe(
            &hh,
            "r-cons-4",
            "Rice and beans",
            Some(20),
            Some(4),
            simple_lines(&["rice", "beans"]),
        ),
        fx_recipe(
            &hh,
            "r-cons-5",
            "Tomato soup",
            Some(25),
            Some(4),
            simple_lines(&["tomato", "onion", "bread"]),
        ),
        fx_recipe(
            &hh,
            "r-cons-6",
            "Veggie tacos",
            Some(20),
            Some(4),
            simple_lines(&["tortilla", "beans", "cheese"]),
        ),
    ];
    let mut snapshot = base(&hh, &recipes);
    snapshot.preferences = vec![prefs(
        "fx-m1",
        &[
            (Sentiment::Like, "beans"),
            (Sentiment::Like, "cheese"),
            (Sentiment::Like, "rice"),
        ],
    )];
    let week_before = [
        "2026-08-22",
        "2026-08-23",
        "2026-08-24",
        "2026-08-25",
        "2026-08-26",
        "2026-08-27",
        "2026-08-28",
    ];
    snapshot.history = week_before
        .iter()
        .enumerate()
        .map(|(i, date)| {
            let id = if i % 2 == 0 { "r-cons-4" } else { "r-cons-1" };
            meal(
                &hh,
                &format!("fx-h-cons-{i}"),
                date,
                MealSlot::Dinner,
                vec![rc(id)],
                false,
            )
        })
        .collect();
    PlannerFixture {
        name: "conservative_household",
        source: FixtureSource::Prd18,
        stressor: "small repeat-tolerant library with narrow likes and a repetitive history",
        snapshot,
        recipes,
    }
}

/// §18 №3: a high-variety household. 48 distinct recipes over lunch and dinner with many
/// likes — the fixture that stresses the per-slot candidate cut (K) and the beam width (B).
fn high_variety_household() -> PlannerFixture {
    const MAINS: [&str; 16] = [
        "bean",
        "lentil",
        "tofu",
        "mushroom",
        "chickpea",
        "potato",
        "squash",
        "spinach",
        "barley",
        "cauliflower",
        "eggplant",
        "zucchini",
        "pea",
        "corn",
        "carrot",
        "beet",
    ];
    const DISHES: [&str; 12] = [
        "stew", "curry", "bake", "salad", "soup", "stir fry", "pasta", "tacos", "pilaf", "gratin",
        "chili", "skillet",
    ];
    let hh = household_of("high-variety");
    // (main, dish) pairs repeat only after lcm(16, 12) = 48, so every title is distinct.
    let recipes: Vec<Recipe> = (0..48)
        .map(|i| {
            let main = MAINS[i % 16];
            let dish = DISHES[i % 12];
            fx_recipe(
                &hh,
                &format!("r-hv-{i:02}"),
                &format!("{main} {dish}"),
                Some(10 + (i as u32 % 6) * 5),
                Some(2 + (i as u32 % 4)),
                simple_lines(&[main, MAINS[(i + 3) % 16], "onion"]),
            )
        })
        .collect();
    let mut snapshot = base(&hh, &recipes);
    snapshot.scope = MealScope::new([MealSlot::Lunch, MealSlot::Dinner]).expect("non-empty scope");
    snapshot.members = vec![member("fx-m1"), member("fx-m2")];
    snapshot.preferences = vec![
        prefs(
            "fx-m1",
            &[
                (Sentiment::Like, "bean"),
                (Sentiment::Like, "lentil"),
                (Sentiment::Like, "tofu"),
                (Sentiment::Like, "mushroom"),
                (Sentiment::Like, "chickpea"),
                (Sentiment::Dislike, "beet"),
            ],
        ),
        prefs(
            "fx-m2",
            &[
                (Sentiment::Like, "potato"),
                (Sentiment::Like, "squash"),
                (Sentiment::Like, "spinach"),
                (Sentiment::Like, "barley"),
                (Sentiment::Like, "corn"),
                (Sentiment::Dislike, "eggplant"),
            ],
        ),
    ];
    PlannerFixture {
        name: "high_variety_household",
        source: FixtureSource::Prd18,
        stressor: "48 recipes, many likes, lunch+dinner scope — stresses the K cut and beam",
        snapshot,
        recipes,
    }
}

/// §18 №4: multiple strong dislikes. Three members whose dislikes together hit seven of the
/// eight recipes, leaving very little the whole household is happy with.
fn multiple_strong_dislikes() -> PlannerFixture {
    let hh = household_of("dislikes");
    let recipes = vec![
        fx_recipe(
            &hh,
            "r-dis-1",
            "Mushroom risotto",
            Some(35),
            Some(4),
            simple_lines(&["mushroom", "rice", "onion"]),
        ),
        fx_recipe(
            &hh,
            "r-dis-2",
            "Olive tapenade pasta",
            Some(20),
            Some(4),
            simple_lines(&["olives", "pasta", "garlic"]),
        ),
        fx_recipe(
            &hh,
            "r-dis-3",
            "Tofu stir fry",
            Some(15),
            Some(4),
            simple_lines(&["tofu", "rice", "broccoli"]),
        ),
        fx_recipe(
            &hh,
            "r-dis-4",
            "Beet salad",
            Some(15),
            Some(2),
            simple_lines(&["beets", "lettuce"]),
        ),
        fx_recipe(
            &hh,
            "r-dis-5",
            "Broccoli bake",
            Some(30),
            Some(4),
            simple_lines(&["broccoli", "cheese"]),
        ),
        fx_recipe(
            &hh,
            "r-dis-6",
            "Cabbage rolls",
            Some(45),
            Some(4),
            simple_lines(&["cabbage", "rice", "onion"]),
        ),
        fx_recipe(
            &hh,
            "r-dis-7",
            "Stuffed peppers",
            Some(40),
            Some(4),
            simple_lines(&["peppers", "rice", "cheese"]),
        ),
        fx_recipe(
            &hh,
            "r-dis-8",
            "Plain rice bowl",
            Some(15),
            Some(4),
            simple_lines(&["rice", "beans"]),
        ),
    ];
    let mut snapshot = base(&hh, &recipes);
    snapshot.members = vec![member("fx-m1"), member("fx-m2"), member("fx-m3")];
    snapshot.preferences = vec![
        prefs(
            "fx-m1",
            &[
                (Sentiment::Dislike, "mushroom"),
                (Sentiment::Dislike, "olives"),
                (Sentiment::Like, "rice"),
            ],
        ),
        prefs(
            "fx-m2",
            &[
                (Sentiment::Dislike, "tofu"),
                (Sentiment::Dislike, "beets"),
                (Sentiment::Dislike, "cabbage"),
            ],
        ),
        prefs(
            "fx-m3",
            &[
                (Sentiment::Dislike, "broccoli"),
                (Sentiment::Dislike, "peppers"),
                (Sentiment::Like, "beans"),
            ],
        ),
    ];
    PlannerFixture {
        name: "multiple_strong_dislikes",
        source: FixtureSource::Prd18,
        stressor: "most recipes disliked by at least one of three members",
        snapshot,
        recipes,
    }
}

/// §18 №5: a busy week. A 30-minute dinner window, preps over and at the window, and two
/// recipes with no prep estimate at all — plus dining out enabled, the week where eating out
/// is the plausible escape. This is the corpus's one `CandidateSource::DiningOut` fixture
/// (`candidates.rs` gates that source on the policy), against `leftovers_fallback_heavy`'s
/// disabled half.
fn busy_week() -> PlannerFixture {
    let hh = household_of("busy-week");
    let recipes = vec![
        fx_recipe(
            &hh,
            "r-busy-1",
            "Slow braise",
            Some(55),
            Some(4),
            simple_lines(&["beef", "carrot", "onion"]),
        ),
        fx_recipe(
            &hh,
            "r-busy-2",
            "Weeknight stir fry",
            Some(30),
            Some(4),
            simple_lines(&["tofu", "rice", "peppers"]),
        ),
        fx_recipe(
            &hh,
            "r-busy-3",
            "Quick quesadilla",
            Some(15),
            Some(2),
            simple_lines(&["tortilla", "cheese"]),
        ),
        fx_recipe(
            &hh,
            "r-busy-4",
            "Mystery casserole",
            None,
            Some(4),
            simple_lines(&["potato", "cheese", "milk"]),
        ),
        fx_recipe(
            &hh,
            "r-busy-5",
            "Long roast",
            Some(45),
            Some(6),
            simple_lines(&["chicken", "potato"]),
        ),
        fx_recipe(
            &hh,
            "r-busy-6",
            "Handed-down soup",
            None,
            Some(4),
            simple_lines(&["carrot", "onion", "barley"]),
        ),
    ];
    let mut snapshot = base(&hh, &recipes);
    snapshot.policies.slot_windows = BTreeMap::from([(MealSlot::Dinner, 30)]);
    snapshot.policies.dining_out_enabled = true;
    snapshot.preferences = vec![prefs(
        "fx-m1",
        &[(Sentiment::Like, "cheese"), (Sentiment::Like, "rice")],
    )];
    PlannerFixture {
        name: "busy_week",
        source: FixtureSource::Prd18,
        stressor: "tight slot window with preps over and at the window, plus unknown preps \
                   and dining out enabled",
        snapshot,
        recipes,
    }
}

/// §18 №6: many locked meals. Five of seven dinners locked, one of them locked onto a
/// peanut recipe against a peanut restriction — the `LOCK_HELD_OVER` path.
fn many_locked_meals() -> PlannerFixture {
    let hh = household_of("locked");
    let recipes = vec![
        fx_recipe(
            &hh,
            "r-lk-1",
            "Peanut noodles",
            Some(20),
            Some(4),
            simple_lines(&["peanuts", "noodles", "scallion"]),
        ),
        fx_recipe(
            &hh,
            "r-lk-2",
            "Vegetable soup",
            Some(25),
            Some(4),
            simple_lines(&["carrot", "onion", "celery"]),
        ),
        fx_recipe(
            &hh,
            "r-lk-3",
            "Rice bowl",
            Some(15),
            Some(4),
            simple_lines(&["rice", "beans"]),
        ),
    ];
    let mut snapshot = base(&hh, &recipes);
    snapshot.restrictions =
        HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Peanuts)]);
    let locks = [
        ("2026-08-29", "r-lk-2"),
        ("2026-08-30", "r-lk-3"),
        // The conflicting lock: the household locked peanut noodles over its own restriction.
        ("2026-08-31", "r-lk-1"),
        ("2026-09-01", "r-lk-2"),
        ("2026-09-02", "r-lk-3"),
    ];
    snapshot.existing = locks
        .iter()
        .enumerate()
        .map(|(i, (date, id))| {
            meal(
                &hh,
                &format!("fx-e-lk-{i}"),
                date,
                MealSlot::Dinner,
                vec![rc(id)],
                true,
            )
        })
        .collect();
    PlannerFixture {
        name: "many_locked_meals",
        source: FixtureSource::Prd18,
        stressor: "5 of 7 dinners locked, one lock conflicting with a restriction",
        snapshot,
        recipes,
    }
}

/// §18 №7: a sparse/empty pantry. Ingredients shared across most recipes, real quantities
/// and units to aggregate, and nothing pantry-marked at all.
fn sparse_pantry() -> PlannerFixture {
    let hh = household_of("sparse-pantry");
    let unresolved_herbs = IngredientLine::new(
        "a handful of fresh herbs",
        "fresh herbs",
        None,
        Quantity::Unknown,
        Unit::None,
        None,
        true,
    )
    .expect("fixture lines are non-blank");
    let recipes = vec![
        fx_recipe(
            &hh,
            "r-sp-1",
            "Rice and beans",
            Some(25),
            Some(4),
            vec![
                line("rice", exact(2, 1), Unit::Known(UnitKind::Cup)),
                line("beans", exact(400, 1), Unit::Known(UnitKind::Gram)),
                line("tomato", range((2, 1), (3, 1)), Unit::None),
            ],
        ),
        fx_recipe(
            &hh,
            "r-sp-2",
            "Bean soup",
            Some(30),
            Some(4),
            vec![
                line("rice", exact(1, 1), Unit::Known(UnitKind::Cup)),
                line("beans", exact(200, 1), Unit::Known(UnitKind::Gram)),
                line("onion", exact(1, 1), Unit::None),
            ],
        ),
        fx_recipe(
            &hh,
            "r-sp-3",
            "Tomato rice",
            Some(20),
            Some(4),
            vec![
                line("rice", exact(3, 2), Unit::Known(UnitKind::Cup)),
                line("tomato", exact(2, 1), Unit::None),
                line("beans", Quantity::Unknown, Unit::None),
                unresolved_herbs,
            ],
        ),
    ];
    let snapshot = base(&hh, &recipes);
    PlannerFixture {
        name: "sparse_pantry",
        source: FixtureSource::Prd18,
        stressor: "heavily shared ingredients with quantities to aggregate, empty pantry",
        snapshot,
        recipes,
    }
}

/// §18 №8: restrictions set — one Known and one Other — with a paired skipped variant via
/// [`restrictions_skipped_variant`].
fn restrictions_set_and_skipped() -> PlannerFixture {
    let hh = household_of("restrictions");
    let recipes = vec![
        fx_recipe(
            &hh,
            "r-rx-1",
            "Peanut satay",
            Some(25),
            Some(4),
            simple_lines(&["peanuts", "noodles"]),
        ),
        fx_recipe(
            &hh,
            "r-rx-2",
            "Herb rice",
            Some(20),
            Some(4),
            simple_lines(&["rice", "cilantro"]),
        ),
        fx_recipe(
            &hh,
            "r-rx-3",
            "Plain soup",
            Some(25),
            Some(4),
            simple_lines(&["carrot", "onion"]),
        ),
    ];
    let mut snapshot = base(&hh, &recipes);
    snapshot.restrictions = HouseholdRestrictions::new([
        Restriction::Known(RestrictionKind::Peanuts),
        Restriction::other("cilantro").expect("non-blank restriction"),
    ]);
    PlannerFixture {
        name: "restrictions_set_and_skipped",
        source: FixtureSource::Prd18,
        stressor: "one Known and one Other restriction; skipped variant is the same \
                   household with restrictions unconfigured",
        snapshot,
        recipes,
    }
}

/// §18 №9: leftovers/fallback-heavy. Big-batch recipes, a history that already feeds the
/// first cycle day, and dining out disabled so fallbacks carry more of the week.
fn leftovers_fallback_heavy() -> PlannerFixture {
    let hh = household_of("leftovers");
    let recipes = vec![
        fx_recipe(
            &hh,
            "r-lo-1",
            "Big batch chili",
            Some(40),
            Some(8),
            simple_lines(&["beans", "tomato", "onion"]),
        ),
        fx_recipe(
            &hh,
            "r-lo-2",
            "Potato casserole",
            Some(45),
            Some(8),
            simple_lines(&["potato", "cheese", "milk"]),
        ),
        fx_recipe(
            &hh,
            "r-lo-3",
            "Side salad",
            Some(10),
            Some(2),
            simple_lines(&["lettuce", "tomato"]),
        ),
    ];
    let mut snapshot = base(&hh, &recipes);
    snapshot.members = vec![member("fx-m1"), member("fx-m2")];
    snapshot.history = vec![
        meal(
            &hh,
            "fx-h-lo-0",
            "2026-08-27",
            MealSlot::Dinner,
            vec![rc("r-lo-2")],
            false,
        ),
        meal(
            &hh,
            "fx-h-lo-1",
            "2026-08-28",
            MealSlot::Dinner,
            vec![rc("r-lo-1")],
            false,
        ),
    ];
    PlannerFixture {
        name: "leftovers_fallback_heavy",
        source: FixtureSource::Prd18,
        stressor: "big-batch servings and leftover-friendly history with dining out disabled",
        snapshot,
        recipes,
    }
}

/// Pivot №10: a repetitive meal library. Five near-duplicate chilis — identical lines and
/// prep, titles one word apart — plus one distinct dish: pure tie-break/repetition stress.
fn repetitive_meal_library() -> PlannerFixture {
    let hh = household_of("repetitive");
    let variants = ["First", "Second", "Third", "Fourth", "Fifth"];
    let mut recipes: Vec<Recipe> = variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            fx_recipe(
                &hh,
                &format!("r-rep-{i}"),
                &format!("{variant} bean chili"),
                Some(30),
                Some(4),
                simple_lines(&["beans", "tomato", "onion"]),
            )
        })
        .collect();
    recipes.push(fx_recipe(
        &hh,
        "r-rep-5",
        "Green salad",
        Some(10),
        Some(2),
        simple_lines(&["lettuce", "cucumber"]),
    ));
    let mut snapshot = base(&hh, &recipes);
    snapshot.preferences = vec![prefs(
        "fx-m1",
        &[(Sentiment::Like, "beans"), (Sentiment::Like, "chili")],
    )];
    PlannerFixture {
        name: "repetitive_meal_library",
        source: FixtureSource::PivotPrompt,
        stressor: "near-duplicate recipes — repetition and tie-break stress",
        snapshot,
        recipes,
    }
}

/// Pivot №11: intentionally open nights. Two slots the household explicitly left `Open` —
/// a human decision the planner must respect — and five slots with nothing at all.
fn intentionally_open_nights() -> PlannerFixture {
    let hh = household_of("open-nights");
    let recipes = vec![
        fx_recipe(
            &hh,
            "r-op-1",
            "Lentil curry",
            Some(30),
            Some(4),
            simple_lines(&["lentils", "rice", "onion"]),
        ),
        fx_recipe(
            &hh,
            "r-op-2",
            "Veggie pasta",
            Some(20),
            Some(4),
            simple_lines(&["pasta", "tomato", "zucchini"]),
        ),
    ];
    let mut snapshot = base(&hh, &recipes);
    snapshot.existing = vec![
        meal(
            &hh,
            "fx-e-op-0",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::Open { note: None }],
            false,
        ),
        meal(
            &hh,
            "fx-e-op-1",
            "2026-09-02",
            MealSlot::Dinner,
            vec![MealComponent::Open { note: None }],
            false,
        ),
    ];
    PlannerFixture {
        name: "intentionally_open_nights",
        source: FixtureSource::PivotPrompt,
        stressor: "two explicitly open slots amid an otherwise empty week",
        snapshot,
        recipes,
    }
}
