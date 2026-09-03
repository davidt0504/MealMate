//! MVP-023 steps 3–7. One fixture builder; every test names the property it pins.

use std::collections::BTreeMap;

use household_core::{
    EvidenceSource, HouseholdController, HouseholdId, MemberId, OutcomeStatus, Policy, PolicyId,
    Urgency,
};

use super::*;
use crate::planner::coverage::{
    CLAIM_BLOCKING, FALLBACK_PLACEHOLDER_CHOSEN, FREEFORM_DISH_UNVERIFIED, NO_PREP_TIME_ESTIMATES,
    PANTRY_INCOMPLETE, PLAN_INFEASIBLE, PREFERENCES_SPARSE, RESTRICTIONS_NOT_CONFIGURED,
};
use crate::planner::score::*;
use crate::planner::tier0::*;
use crate::{
    parse_civil_date, HouseholdRestrictions, IngredientId, IngredientLine, IngredientRef,
    MealScope, MealSlot, MemberPreference, MemberPreferences, PlannedMeal, PlannedMealId, Quantity,
    Rational, RecipeId, Restriction, RestrictionKind, Sentiment, StarterRecipe, Unit,
};
use crate::{ProvenanceKind, RecipeProvenance};

fn d(raw: &str) -> CivilDate {
    parse_civil_date(raw).unwrap()
}

fn cat(id: &str) -> IngredientRef {
    IngredientRef::Catalog(IngredientId::new(id).unwrap())
}

fn recipe(id: &str, title: &str, prep: Option<u32>, lines: &[&str]) -> RecipeCandidateInfo {
    RecipeCandidateInfo {
        id: RecipeId::new(id).unwrap(),
        title: title.to_owned(),
        servings: Some(4),
        prep_minutes: prep,
        line_names: lines.iter().map(|l| (*l).to_owned()).collect(),
        ingredient_refs: lines.iter().map(|l| cat(l)).collect(),
        starter_slug: None,
    }
}

fn starter(slug: &str, title: &str, lines: &[&str]) -> StarterRecipe {
    StarterRecipe {
        slug: slug.to_owned(),
        cook_review: None,
        expected_conflicts: vec![],
        title: title.to_owned(),
        servings: Some(4),
        prep_minutes: Some(20),
        instructions: String::new(),
        lines: lines
            .iter()
            .map(|l| {
                IngredientLine::new(
                    *l,
                    *l,
                    Some(cat(l)),
                    Quantity::Unknown,
                    Unit::None,
                    None,
                    false,
                )
                .unwrap()
            })
            .collect(),
        provenance: RecipeProvenance::with_rights(
            ProvenanceKind::Starter,
            None,
            None,
            None,
            None,
            Some(slug.to_owned()),
        )
        .unwrap(),
    }
}

fn meal(
    id: &str,
    date: &str,
    slot: MealSlot,
    components: Vec<MealComponent>,
    locked: bool,
) -> PlannedMeal {
    PlannedMeal::new(
        PlannedMealId::new(id).unwrap(),
        HouseholdId::new("h").unwrap(),
        d(date),
        slot,
        components,
        locked,
    )
    .unwrap()
}

fn rc(id: &str) -> MealComponent {
    MealComponent::recipe(RecipeId::new(id).unwrap(), None)
}

fn prefs(member: &str, items: &[(Sentiment, &str)]) -> (MemberId, MemberPreferences) {
    (
        MemberId::new(member).unwrap(),
        MemberPreferences::new(
            items
                .iter()
                .map(|(s, subject)| MemberPreference::new(*s, *subject).unwrap()),
        ),
    )
}

/// Two dinners, one member, three recipes with prep times and a 40-minute dinner window,
/// restrictions configured, four preferences: the "everything known" base every test bends.
fn base() -> PlanningSnapshot {
    PlanningSnapshot {
        household_id: HouseholdId::new("h").unwrap(),
        anchor: d("2026-08-29"),
        length_days: 2,
        scope: MealScope::dinner_only(),
        today: d("2026-08-29"),
        members: vec![MemberId::new("m1").unwrap()],
        restrictions: HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Peanuts)]),
        preferences: vec![prefs(
            "m1",
            &[
                (Sentiment::Like, "rice"),
                (Sentiment::Like, "beans"),
                (Sentiment::Dislike, "olives"),
                (Sentiment::Like, "tofu"),
            ],
        )],
        policies: FoodPolicies {
            dining_out_enabled: false,
            hard_vetoes: vec![],
            slot_windows: BTreeMap::from([(MealSlot::Dinner, 40)]),
            restrictions_reviewed: false,
            unknown_policy_types: vec![],
        },
        recipes: vec![
            recipe("r-rice", "Rice bowl", Some(15), &["rice", "beans"]),
            recipe("r-soup", "Soup", Some(15), &["carrot", "onion"]),
            recipe("r-tofu", "Tofu stir fry", Some(15), &["tofu", "rice"]),
        ],
        starter: vec![],
        existing: vec![],
        history: vec![],
        pantry_marked: vec![],
    }
}

fn run(s: &PlanningSnapshot) -> PlanningResult {
    cover_cycle(s, &SearchParams::default())
}

fn chosen_keys(r: &PlanningResult) -> Vec<String> {
    r.proposed
        .iter()
        .map(|p| snapshot::components_text(&p.components))
        .collect()
}

// --- Step 3: snapshot, canonical text, hash ---------------------------------------------------

#[test]
fn canonical_text_covers_every_field() {
    let s = base();
    let text = s.canonical_text();
    let mut variants: Vec<PlanningSnapshot> = Vec::new();
    let mut v = s.clone();
    v.household_id = HouseholdId::new("other").unwrap();
    variants.push(v);
    let mut v = s.clone();
    v.anchor = d("2026-08-30");
    variants.push(v);
    let mut v = s.clone();
    v.length_days = 3;
    variants.push(v);
    let mut v = s.clone();
    v.scope = MealScope::new([MealSlot::Lunch, MealSlot::Dinner]).unwrap();
    variants.push(v);
    let mut v = s.clone();
    v.today = d("2026-08-30");
    variants.push(v);
    let mut v = s.clone();
    v.members.push(MemberId::new("m2").unwrap());
    variants.push(v);
    let mut v = s.clone();
    v.restrictions = HouseholdRestrictions::new([Restriction::other("nightshades").unwrap()]);
    variants.push(v);
    let mut v = s.clone();
    v.preferences = vec![prefs("m1", &[(Sentiment::Like, "rice")])];
    variants.push(v);
    let mut v = s.clone();
    v.policies.dining_out_enabled = true;
    variants.push(v);
    let mut v = s.clone();
    v.policies.hard_vetoes = vec!["olives".to_owned()];
    variants.push(v);
    let mut v = s.clone();
    v.policies.slot_windows.insert(MealSlot::Dinner, 41);
    variants.push(v);
    let mut v = s.clone();
    v.policies.unknown_policy_types = vec!["food.mystery".to_owned()];
    variants.push(v);
    let mut v = s.clone();
    v.policies.restrictions_reviewed = true;
    variants.push(v);
    let mut v = s.clone();
    v.recipes[0].prep_minutes = None;
    variants.push(v);
    let mut v = s.clone();
    v.starter = vec![starter("s-1", "Starter", &["oats"])];
    variants.push(v);
    let mut v = s.clone();
    v.existing = vec![meal(
        "pm",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-soup")],
        false,
    )];
    variants.push(v);
    let mut v = s.clone();
    v.history = vec![meal(
        "pm",
        "2026-08-28",
        MealSlot::Dinner,
        vec![rc("r-soup")],
        false,
    )];
    variants.push(v);
    let mut v = s.clone();
    v.pantry_marked = vec![cat("rice")];
    variants.push(v);
    assert_eq!(variants.len(), 18, "one variant per field");
    for (i, v) in variants.iter().enumerate() {
        assert_ne!(v.canonical_text(), text, "variant {i} must change the text");
        assert_ne!(
            v.snapshot_hash(),
            s.snapshot_hash(),
            "variant {i} must change the hash"
        );
    }
    // A locked flag and a scale are facts too.
    let mut v = s.clone();
    v.existing = vec![meal(
        "pm",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-soup")],
        true,
    )];
    let mut w = s.clone();
    w.existing = vec![meal(
        "pm",
        "2026-08-29",
        MealSlot::Dinner,
        vec![MealComponent::recipe(
            RecipeId::new("r-soup").unwrap(),
            Some(Rational::new(3, 2).unwrap()),
        )],
        false,
    )];
    assert_ne!(v.canonical_text(), w.canonical_text());
}

/// Golden: the fixture's hash is pinned so a silent change to the encoder is a failure here,
/// not an unexplained ledger mismatch later. Recompute deliberately when the encoder changes.
#[test]
fn snapshot_hash_is_stable_for_a_fixed_fixture() {
    let s = base();
    let hash = s.snapshot_hash();
    assert_eq!(hash.len(), 16);
    assert!(hash
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    assert_eq!(hash, s.clone().snapshot_hash());
    assert_eq!(
        format!("{:016x}", snapshot::fnv1a_64(b"")),
        "cbf29ce484222325"
    );
    assert_eq!(
        format!("{:016x}", snapshot::fnv1a_64(b"a")),
        "af63dc4c8601ec8c"
    );
    assert_eq!(hash, "aa39711548f3a740");
}

#[test]
fn food_policies_parse_and_report_unknown_types() {
    let policy = |ty: &str, params: &[(&str, &str)], enabled: bool, domain: &str| {
        Policy::new(
            PolicyId::new(format!("p-{ty}-{enabled}")).unwrap(),
            HouseholdId::new("h").unwrap(),
            domain,
            ty,
            params
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            enabled,
            EvidenceSource::ExplicitUser,
        )
        .unwrap()
    };
    let policies = vec![
        policy(FoodPolicies::DINING_OUT, &[], true, "food"),
        policy(
            FoodPolicies::HARD_VETO,
            &[("subject", " olives ")],
            true,
            "food",
        ),
        policy(
            FoodPolicies::HARD_VETO,
            &[("subject", "cilantro")],
            false,
            "food",
        ),
        policy(FoodPolicies::HARD_VETO, &[], true, "food"),
        policy(
            FoodPolicies::SLOT_WINDOW,
            &[("slot", "dinner"), ("minutes", "45")],
            true,
            "food",
        ),
        policy(
            FoodPolicies::SLOT_WINDOW,
            &[("slot", "supper"), ("minutes", "45")],
            true,
            "food",
        ),
        policy(
            FoodPolicies::SLOT_WINDOW,
            &[("slot", "lunch"), ("minutes", "0")],
            true,
            "food",
        ),
        policy("food.mystery", &[], true, "food"),
        policy("schedule.window", &[], true, "schedule"),
    ];
    let parsed = FoodPolicies::from_policies(&policies);
    assert!(parsed.dining_out_enabled);
    assert_eq!(
        parsed.hard_vetoes,
        vec!["olives"],
        "trimmed; disabled veto dropped"
    );
    assert_eq!(
        parsed.slot_windows,
        BTreeMap::from([(MealSlot::Dinner, 45)])
    );
    assert_eq!(
        parsed.unknown_policy_types,
        vec![
            "food.hard_veto",
            "food.mystery",
            "food.slot_window",
            "schedule.window",
        ],
        "malformed known types and foreign domains are reported, sorted and deduplicated"
    );
    assert_eq!(FoodPolicies::from_policies(&[]), FoodPolicies::default());
    let mut s = base();
    s.policies = parsed;
    let r = run(&s);
    assert!(s.policies.unknown_policy_types.len() == 4);
    assert!(r
        .assessment
        .assumptions
        .iter()
        .any(|a| a.as_str() == coverage::UNKNOWN_POLICY_TYPE));
}

// --- Step 4: candidates and Tier 0 --------------------------------------------------------------

fn sources_of(slot: &SlotCandidates) -> Vec<CandidateSource> {
    let mut v: Vec<CandidateSource> = slot.candidates.iter().map(|c| c.source).collect();
    v.dedup();
    v
}

#[test]
fn every_source_appears_for_an_unresolved_slot_when_enabled() {
    let mut s = base();
    s.policies.dining_out_enabled = true;
    s.starter = vec![starter("s-1", "Oat thing", &["oats"])];
    s.existing = vec![meal(
        "pm",
        "2026-08-30",
        MealSlot::Dinner,
        vec![rc("r-soup")],
        false,
    )];
    let generated = candidates::generate(&s);
    assert_eq!(generated.len(), 2);
    // Day 2 has an existing meal and a possible leftovers source (day 1 recipes feed 4 > 2).
    assert_eq!(
        sources_of(&generated[1]),
        vec![
            CandidateSource::ExistingPlan,
            CandidateSource::HouseholdRecipe,
            CandidateSource::StarterMeal,
            CandidateSource::Leftovers,
            CandidateSource::DiningOut,
            CandidateSource::FrozenQuick,
            CandidateSource::IntentionallyOpen,
        ]
    );
    assert!(!generated[1].resolved);
    // Hard-coded seven: an eighth source must break this, not be mirrored by it.
    assert_eq!(CandidateSource::ALL.len(), 7);
    assert_eq!(
        CandidateSource::ALL.map(CandidateSource::as_str),
        [
            "existing_plan",
            "household_recipe",
            "starter_meal",
            "leftovers",
            "dining_out",
            "frozen_quick",
            "intentionally_open",
        ]
    );
}

#[test]
fn dining_out_is_absent_without_policy() {
    let s = base();
    for slot in candidates::generate(&s) {
        assert!(!sources_of(&slot).contains(&CandidateSource::DiningOut));
    }
    let mut s = base();
    s.policies.dining_out_enabled = true;
    assert!(candidates::generate(&s)
        .iter()
        .all(|slot| sources_of(slot).contains(&CandidateSource::DiningOut)));
}

#[test]
fn a_locked_slot_rejects_every_candidate_but_itself_with_lock_conflict() {
    let mut s = base();
    s.existing = vec![meal(
        "pm",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-soup")],
        true,
    )];
    let generated = candidates::generate(&s);
    assert!(generated[0].resolved);
    let (feasible, rejected) = tier0::filter(&s, &generated[0]);
    assert_eq!(feasible.len(), 1);
    assert_eq!(feasible[0].candidate.source, CandidateSource::ExistingPlan);
    assert!(feasible[0].candidate.locked);
    assert_eq!(rejected.len(), generated[0].candidates.len() - 1);
    assert!(
        rejected.iter().all(|r| r.code == LOCK_CONFLICT),
        "{rejected:?}"
    );
    assert!(rejected
        .iter()
        .all(|r| r.date == d("2026-08-29") && r.slot == MealSlot::Dinner));
    let r = run(&s);
    assert_eq!(r.slots[0].state, CoverageState::LockedByUser);
    assert_eq!(r.proposed[0].components, vec![rc("r-soup")]);
}

#[test]
fn an_existing_open_slot_is_resolved_as_a_human_decision() {
    let mut s = base();
    s.existing = vec![meal(
        "pm",
        "2026-08-29",
        MealSlot::Dinner,
        vec![MealComponent::Open { note: None }],
        false,
    )];
    let generated = candidates::generate(&s);
    assert!(generated[0].resolved);
    let (feasible, rejected) = tier0::filter(&s, &generated[0]);
    assert_eq!(feasible.len(), 1);
    assert!(feasible[0].candidate.is_open());
    assert!(rejected.iter().all(|r| r.code == LOCK_CONFLICT));
    let r = run(&s);
    assert_eq!(r.slots[0].state, CoverageState::IntentionallyOpen);
    assert!(
        !r.score.terms.iter().any(|t| t.code == CHURN_PENALTY),
        "an explicit Open is never churned: {:?}",
        r.score.terms
    );
    assert_eq!(
        r.proposed[0].components,
        vec![MealComponent::Open { note: None }]
    );
}

#[test]
fn restriction_conflict_is_rejected_with_rule_version_reason() {
    let mut s = base();
    s.recipes.push(recipe(
        "r-pb",
        "Peanut noodles",
        Some(10),
        &["peanut butter", "noodles"],
    ));
    let generated = candidates::generate(&s);
    let (feasible, rejected) = tier0::filter(&s, &generated[0]);
    assert!(feasible
        .iter()
        .all(|f| !f.candidate.text().contains("r-pb")));
    let hit: Vec<&Rejection> = rejected
        .iter()
        .filter(|r| r.candidate_text.contains("recipe:r-pb"))
        .collect();
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0].code, "RESTRICTION_CONFLICT:rules_v1");
    let r = run(&s);
    assert!(chosen_keys(&r).iter().all(|k| !k.contains("r-pb")));
    assert!(r
        .assessment
        .reason_codes
        .iter()
        .any(|c| c.as_str() == "RESTRICTION_CONFLICT:rules_v1"));
}

#[test]
fn hard_veto_is_rejected_by_title_and_by_line() {
    let mut s = base();
    s.policies.hard_vetoes = vec!["soup".to_owned(), "beans".to_owned()];
    let generated = candidates::generate(&s);
    let (feasible, rejected) = tier0::filter(&s, &generated[0]);
    let codes: Vec<(&str, &str)> = rejected
        .iter()
        .map(|r| (r.candidate_text.as_str(), r.code.as_str()))
        .collect();
    assert!(
        codes
            .iter()
            .any(|(t, c)| t.contains("recipe:r-soup") && *c == HARD_VETO),
        "by title: {codes:?}"
    );
    assert!(
        codes
            .iter()
            .any(|(t, c)| t.contains("recipe:r-rice") && *c == HARD_VETO),
        "by line: {codes:?}"
    );
    assert!(feasible
        .iter()
        .any(|f| f.candidate.text().contains("recipe:r-tofu")));
    // Whole-token: "bean" does not veto "beans", and a veto is never a score weight.
    let mut s2 = base();
    s2.policies.hard_vetoes = vec!["bean".to_owned()];
    let (f2, _) = tier0::filter(&s2, &candidates::generate(&s2)[0]);
    assert!(f2
        .iter()
        .any(|f| f.candidate.text().contains("recipe:r-rice")));
    let r = run(&s);
    assert!(chosen_keys(&r)
        .iter()
        .all(|k| !k.contains("r-soup") && !k.contains("r-rice")));
}

#[test]
fn prep_over_window_is_rejected_only_when_both_known() {
    let mut s = base();
    s.recipes
        .push(recipe("r-slow", "Slow braise", Some(90), &["beef"]));
    s.recipes
        .push(recipe("r-unknown", "Mystery", None, &["kale"]));
    let (feasible, rejected) = tier0::filter(&s, &candidates::generate(&s)[0]);
    assert!(rejected
        .iter()
        .any(|r| r.candidate_text.contains("recipe:r-slow") && r.code == PREP_WINDOW_IMPOSSIBLE));
    assert!(feasible
        .iter()
        .any(|f| f.candidate.text().contains("recipe:r-unknown")));
    // No window: nothing is rejected on time, even a 90-minute recipe.
    s.policies.slot_windows.clear();
    let (feasible, rejected) = tier0::filter(&s, &candidates::generate(&s)[0]);
    assert!(feasible
        .iter()
        .any(|f| f.candidate.text().contains("recipe:r-slow")));
    assert!(!rejected.iter().any(|r| r.code == PREP_WINDOW_IMPOSSIBLE));
    // Exactly the window fits.
    s.policies.slot_windows.insert(MealSlot::Dinner, 90);
    let (feasible, _) = tier0::filter(&s, &candidates::generate(&s)[0]);
    assert!(feasible
        .iter()
        .any(|f| f.candidate.text().contains("recipe:r-slow")));
}

/// Adversarial: an `Other` restriction matched only by its wording must never reject and
/// must always be carried as an open question.
#[test]
fn wording_only_restriction_never_rejects_and_is_carried_as_uncertainty() {
    let mut s = base();
    s.restrictions = HouseholdRestrictions::new([Restriction::other("nightshades").unwrap()]);
    s.recipes = vec![recipe(
        "r-n",
        "Nightshades bake",
        Some(10),
        &["nightshades mix"],
    )];
    let (feasible, rejected) = tier0::filter(&s, &candidates::generate(&s)[0]);
    // `assess` reports a wording *conflict* here — the text literally matches — so this
    // recipe is rejected; the restriction is still wording-only for the survivors.
    assert!(rejected
        .iter()
        .any(|r| r.candidate_text.contains("recipe:r-n")));
    s.recipes = vec![recipe("r-t", "Tomato bake", Some(10), &["tomato"])];
    let (feasible2, rejected2) = tier0::filter(&s, &candidates::generate(&s)[0]);
    let _ = feasible;
    assert!(!rejected2
        .iter()
        .any(|r| r.code.starts_with(RESTRICTION_CONFLICT)));
    let t = feasible2
        .iter()
        .find(|f| f.candidate.text().contains("recipe:r-t"))
        .unwrap();
    assert_eq!(t.wording_only, vec!["nightshades"]);
    assert!(t
        .assumptions
        .contains(&RESTRICTION_WORDING_UNVERIFIED.to_owned()));
    let r = run(&s);
    assert!(r.attention.iter().any(|a| a
        .reason_codes
        .iter()
        .any(|c| c.as_str() == "HARD_CONSTRAINT_UNRESOLVED")
        && a.urgency == Urgency::High));
    assert_ne!(r.assessment.status, OutcomeStatus::Covered);
}

#[test]
fn leftovers_require_enough_servings_for_the_household() {
    let mut s = base();
    s.members = vec![MemberId::new("m1").unwrap(), MemberId::new("m2").unwrap()];
    for r in &mut s.recipes {
        r.servings = Some(2);
    }
    assert!(candidates::generate(&s)
        .iter()
        .all(|slot| !sources_of(slot).contains(&CandidateSource::Leftovers)));
    s.recipes[0].servings = Some(3);
    let generated = candidates::generate(&s);
    assert!(
        !sources_of(&generated[0]).contains(&CandidateSource::Leftovers),
        "no day before"
    );
    assert!(sources_of(&generated[1]).contains(&CandidateSource::Leftovers));
    // A stored day-before meal at 4 × 1/2 = 2 servings does not feed three.
    s.recipes[0].servings = Some(2);
    s.history = vec![meal(
        "h1",
        "2026-08-28",
        MealSlot::Dinner,
        vec![MealComponent::recipe(
            RecipeId::new("r-rice").unwrap(),
            Some(Rational::new(1, 2).unwrap()),
        )],
        false,
    )];
    s.recipes[0].servings = Some(4);
    assert!(!sources_of(&candidates::generate(&s)[0]).contains(&CandidateSource::Leftovers));
    s.history = vec![meal(
        "h1",
        "2026-08-28",
        MealSlot::Dinner,
        vec![rc("r-rice")],
        false,
    )];
    assert!(sources_of(&candidates::generate(&s)[0]).contains(&CandidateSource::Leftovers));
    // Unknown servings never qualify.
    s.recipes[0].servings = None;
    s.history.clear();
    for r in &mut s.recipes {
        r.servings = None;
    }
    assert!(candidates::generate(&s)
        .iter()
        .all(|slot| !sources_of(slot).contains(&CandidateSource::Leftovers)));
}

#[test]
fn an_installed_starter_is_not_generated_twice() {
    let mut s = base();
    s.recipes[0].starter_slug = Some("s-rice".to_owned());
    s.starter = vec![
        starter("s-rice", "Rice bowl", &["rice"]),
        starter("s-oats", "Oats", &["oats"]),
    ];
    let slot = &candidates::generate(&s)[0];
    let starters: Vec<String> = slot
        .candidates
        .iter()
        .filter(|c| c.source == CandidateSource::StarterMeal)
        .map(Candidate::text)
        .collect();
    assert_eq!(starters.len(), 1);
    assert!(starters[0].contains("freeform:starter:s-oats"));
}

// --- Step 5: scoring -------------------------------------------------------------------------------

fn single(s: &PlanningSnapshot, pick: &str) -> Feasible {
    let (feasible, _) = tier0::filter(s, &candidates::generate(s)[0]);
    feasible
        .into_iter()
        .find(|f| f.candidate.text().contains(pick))
        .unwrap_or_else(|| panic!("{pick} not feasible"))
}

#[test]
fn score_orders_lexicographically_across_tiers() {
    assert!(Score([1, 0, 0, 0, 0, 0]) > Score([0, 9, 9, 9, 9, 9]));
    assert!(Score([1, 0, 0, 0, 0, 0]) > Score([0, 0, 0, 0, 0, i64::MAX]));
    assert!(Score([0, 1, 0, 0, 0, 0]) > Score([0, 0, 99, 99, 99, 99]));
    assert!(Score([0, 0, 0, -1, 0, 0]) < Score([0, 0, 0, 0, -50, -50]));
    assert_eq!(Score([1, 2, 3, 4, 5, 6]), Score([1, 2, 3, 4, 5, 6]));
    // Adversarial through the real scorer: huge T5 (pantry fit, cap 3) never beats T1.
    let mut s = base();
    s.pantry_marked = vec![cat("carrot"), cat("onion"), cat("rice"), cat("beans")];
    let soup = single(&s, "recipe:r-soup");
    let open = single(&s, "intentionally_open");
    let p = &SearchParams::default();
    let a = score_plan(&s, p, &[soup], 2).score;
    let b = score_plan(&s, p, &[open], 2).score;
    assert!(a > b);
    assert_eq!(a.0[0], 1);
    assert_eq!(b.0[0], 0);
}

#[test]
fn dislike_outweighs_like_and_contradiction_counts_as_dislike() {
    let mut s = base();
    s.preferences = vec![prefs(
        "m1",
        &[
            (Sentiment::Like, "rice"),
            (Sentiment::Like, "beans"),
            (Sentiment::Dislike, "beans"),
            (Sentiment::Dislike, "carrot"),
        ],
    )];
    let p = &SearchParams::default();
    let rice = score_plan(&s, p, &[single(&s, "recipe:r-rice")], 2);
    // rice +1, beans like+dislike → dislike −2 with the contradiction coded.
    assert_eq!(rice.score.0[2], -1);
    assert!(rice
        .terms
        .iter()
        .any(|t| t.code == CONTRADICTORY_PREFERENCE));
    assert_eq!(
        rice.terms
            .iter()
            .filter(|t| t.code == MEMBER_DISLIKE)
            .count(),
        1
    );
    let soup = score_plan(&s, p, &[single(&s, "recipe:r-soup")], 2);
    assert_eq!(soup.score.0[2], -2);
    let tofu = score_plan(&s, p, &[single(&s, "recipe:r-tofu")], 2);
    assert_eq!(tofu.score.0[2], 1);
    assert!(tofu.score > rice.score && rice.score > soup.score);
}

#[test]
fn floor_is_ordered_before_aggregate() {
    // Two members; plan A: m1 +2 / m2 −2 (sum 0, floor −2); plan B: m1 0 / m2 0 (floor 0).
    let mut s = base();
    s.members = vec![MemberId::new("m1").unwrap(), MemberId::new("m2").unwrap()];
    s.preferences = vec![
        prefs(
            "m1",
            &[(Sentiment::Like, "rice"), (Sentiment::Like, "beans")],
        ),
        prefs("m2", &[(Sentiment::Dislike, "rice")]),
    ];
    let p = &SearchParams::default();
    let a = score_plan(&s, p, &[single(&s, "recipe:r-rice")], 2);
    let b = score_plan(&s, p, &[single(&s, "recipe:r-soup")], 2);
    assert_eq!(a.score.0[1], -2);
    assert_eq!(a.score.0[2], 0);
    assert_eq!(b.score.0[1], 0);
    assert!(b.score > a.score, "floor decides before the sum does");
    // And a plan with a higher sum but the same floor wins on the sum.
    let mut s3 = s.clone();
    s3.preferences = vec![
        prefs(
            "m1",
            &[(Sentiment::Like, "rice"), (Sentiment::Like, "beans")],
        ),
        prefs("m2", &[(Sentiment::Like, "carrot")]),
    ];
    let c = score_plan(&s3, p, &[single(&s3, "recipe:r-rice")], 2);
    let e = score_plan(&s3, p, &[single(&s3, "recipe:r-soup")], 2);
    assert_eq!((c.score.0[1], c.score.0[2]), (0, 2));
    assert_eq!((e.score.0[1], e.score.0[2]), (0, 1));
    assert!(c.score > e.score);
}

#[test]
fn repeated_sacrifice_of_one_member_is_penalised() {
    let mut s = base();
    s.length_days = 4;
    s.members = vec![MemberId::new("m1").unwrap(), MemberId::new("m2").unwrap()];
    s.preferences = vec![
        prefs("m1", &[(Sentiment::Like, "rice")]),
        prefs("m2", &[(Sentiment::Like, "carrot")]),
    ];
    let p = &SearchParams::default();
    let rice = single(&s, "recipe:r-rice");
    let soup = single(&s, "recipe:r-soup");
    // Four rice dinners: m2 holds the lowest per-slot term on 4 > 4/2 slots.
    let all_rice = score_plan(
        &s,
        p,
        &[rice.clone(), rice.clone(), rice.clone(), rice.clone()],
        4,
    );
    let sacrifice = all_rice
        .terms
        .iter()
        .find(|t| t.code.starts_with(REPEATED_SACRIFICE))
        .expect("sacrifice coded");
    assert_eq!(sacrifice.code, "REPEATED_SACRIFICE:m2");
    assert_eq!(sacrifice.value, -8);
    assert_eq!(all_rice.score.0[2], 4 - 8);
    // Three rice and one soup: m2 lowest on 3 slots, 3·2 > 4 still.
    let mixed = score_plan(
        &s,
        p,
        &[rice.clone(), soup.clone(), rice.clone(), rice.clone()],
        4,
    );
    assert_eq!(mixed.score.0[2], 4 - 6);
    // Balanced two and two: nobody sacrificed more than half.
    let fair = score_plan(&s, p, &[rice.clone(), soup.clone(), rice, soup], 4);
    assert!(!fair
        .terms
        .iter()
        .any(|t| t.code.starts_with(REPEATED_SACRIFICE)));
    assert_eq!(fair.score.0[2], 4);
    assert_eq!(fair.score.0[1], 2);
    assert!(fair.score > mixed.score && mixed.score > all_rice.score);
}

#[test]
fn single_member_floor_equals_sum() {
    let s = base();
    let p = &SearchParams::default();
    let plan = score_plan(
        &s,
        p,
        &[single(&s, "recipe:r-rice"), single(&s, "recipe:r-tofu")],
        2,
    );
    assert_eq!(plan.score.0[1], plan.score.0[2]);
    assert_eq!(plan.score.0[1], 2 + 2);
    assert!(!plan
        .terms
        .iter()
        .any(|t| t.code.starts_with(REPEATED_SACRIFICE)));
}

#[test]
fn near_term_churn_costs_more_than_distant() {
    let mut s = base();
    s.length_days = 5;
    s.existing = vec![
        meal(
            "a",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-soup")],
            false,
        ),
        meal(
            "b",
            "2026-09-02",
            MealSlot::Dinner,
            vec![rc("r-soup")],
            false,
        ),
    ];
    let p = &SearchParams::default();
    let generated = candidates::generate(&s);
    let pick = |i: usize, text: &str| {
        tier0::filter(&s, &generated[i])
            .0
            .into_iter()
            .find(|f| f.candidate.text().contains(text))
            .unwrap()
    };
    let near = score_plan(&s, p, &[pick(0, "household_recipe recipe:r-rice")], 5);
    let churn: i64 = near
        .terms
        .iter()
        .filter(|t| t.code == CHURN_PENALTY || t.code == NEAR_TERM_CHURN)
        .map(|t| t.value)
        .sum();
    assert_eq!(churn, -4);
    let keep = score_plan(&s, p, &[pick(0, "existing_plan")], 5);
    assert!(!keep.terms.iter().any(|t| t.code == CHURN_PENALTY));
    // Indices 0..4 are 08-29..09-02; the second existing meal sits at index 4.
    let mut distant_plan: Vec<Feasible> = (0..4).map(|i| pick(i, "recipe:r-tofu")).collect();
    distant_plan.push(pick(4, "household_recipe recipe:r-rice"));
    let distant = score_plan(&s, p, &distant_plan, 5);
    let distant_churn: i64 = distant
        .terms
        .iter()
        .filter(|t| {
            t.date == Some(d("2026-09-02"))
                && (t.code == CHURN_PENALTY || t.code == NEAR_TERM_CHURN)
        })
        .map(|t| t.value)
        .sum();
    assert_eq!(distant_churn, -1);
    // Same components as the existing meal: no churn at all.
    let same = score_plan(&s, p, &[pick(0, "household_recipe recipe:r-soup")], 5);
    assert!(!same.terms.iter().any(|t| t.code == CHURN_PENALTY));
}

#[test]
fn slack_terms_need_both_prep_and_window() {
    let p = &SearchParams::default();
    let mut s = base();
    s.recipes = vec![
        recipe("r-fast", "Fast", Some(15), &["a"]),
        recipe("r-tight", "Tight", Some(35), &["b"]),
        recipe("r-mid", "Mid", Some(25), &["c"]),
        recipe("r-none", "None", None, &["d"]),
    ];
    let terms = |snap: &PlanningSnapshot, pick: &str| -> Vec<(String, i64)> {
        score_plan(snap, p, &[single(snap, pick)], 2)
            .terms
            .iter()
            .filter(|t| t.code == SLACK_GOOD || t.code == SLACK_FRAGILE)
            .map(|t| (t.code.clone(), t.value))
            .collect()
    };
    assert_eq!(terms(&s, "r-fast"), vec![(SLACK_GOOD.to_owned(), 1)]);
    assert_eq!(terms(&s, "r-tight"), vec![(SLACK_FRAGILE.to_owned(), -1)]);
    assert_eq!(terms(&s, "r-mid"), vec![]);
    assert_eq!(terms(&s, "r-none"), vec![]);
    let none = score_plan(&s, p, &[single(&s, "r-none")], 2);
    assert!(none.assumptions.contains(&PREP_FEASIBLE_UNKNOWN.to_owned()));
    s.policies.slot_windows.clear();
    assert_eq!(terms(&s, "r-fast"), vec![]);
    assert_eq!(terms(&s, "r-tight"), vec![]);
}

#[test]
fn every_term_carries_a_reason_code() {
    let mut s = base();
    s.length_days = 4;
    s.members = vec![MemberId::new("m1").unwrap(), MemberId::new("m2").unwrap()];
    s.preferences
        .push(prefs("m2", &[(Sentiment::Dislike, "rice")]));
    s.pantry_marked = vec![cat("rice")];
    s.history = vec![meal(
        "h",
        "2026-08-27",
        MealSlot::Dinner,
        vec![rc("r-soup")],
        false,
    )];
    s.existing = vec![meal(
        "e",
        "2026-08-30",
        MealSlot::Dinner,
        vec![rc("r-tofu")],
        false,
    )];
    s.starter = vec![starter("s-1", "Starter", &["oats"])];
    let r = run(&s);
    assert!(!r.score.terms.is_empty());
    for t in &r.score.terms {
        assert!(!t.code.is_empty());
        assert!(!t.code.chars().any(char::is_whitespace), "{t:?}");
        assert!(
            household_core::ReasonCode::new(t.code.clone()).is_ok(),
            "{t:?}"
        );
    }
    assert!(r
        .assessment
        .assumptions
        .iter()
        .any(|a| a.as_str() == NO_COST_DATA));
    assert!(r
        .assessment
        .assumptions
        .iter()
        .any(|a| a.as_str() == NO_NUTRITION_DATA));
}

// --- Step 6: beam -----------------------------------------------------------------------------------

#[test]
fn beam_is_deterministic_across_repeated_runs_and_input_permutations() {
    let mut s = base();
    s.length_days = 5;
    s.recipes
        .push(recipe("r-4", "Fourth", Some(20), &["egg", "rice"]));
    s.pantry_marked = vec![cat("rice"), cat("egg")];
    let first = run(&s);
    for _ in 0..3 {
        let again = run(&s);
        assert_eq!(again, first);
        assert_eq!(again.canonical_text(), first.canonical_text());
    }
    // Permute the input collections: the plan must not change (the hash may — the loader,
    // not the planner, owns canonical order, so this asserts plan equality only).
    let mut permuted = s.clone();
    permuted.recipes.reverse();
    permuted.pantry_marked.reverse();
    permuted.preferences = vec![prefs(
        "m1",
        &[
            (Sentiment::Like, "tofu"),
            (Sentiment::Dislike, "olives"),
            (Sentiment::Like, "beans"),
            (Sentiment::Like, "rice"),
        ],
    )];
    let other = run(&permuted);
    assert_eq!(other.proposed, first.proposed);
    assert_eq!(other.score.score, first.score.score);
    assert_eq!(other.assessment.status, first.assessment.status);
}

/// AC-3: the per-slot optimum (the pantry-fit recipe every night) is not the cycle optimum.
/// The property, not one exact plan: no adjacent repeat, and a score at least the greedy one.
#[test]
fn sequence_fixture_picks_cycle_optimal_over_slot_optimal() {
    let mut s = base();
    s.length_days = 4;
    s.preferences = vec![prefs(
        "m1",
        &[
            (Sentiment::Like, "rice"),
            (Sentiment::Like, "carrot"),
            (Sentiment::Like, "tofu"),
        ],
    )];
    s.recipes = vec![
        recipe("r-a", "Rice pilaf", Some(15), &["rice", "stock"]),
        recipe("r-b", "Carrot soup", Some(15), &["carrot", "stock"]),
        recipe("r-c", "Tofu bowl", Some(15), &["tofu", "stock"]),
    ];
    s.pantry_marked = vec![cat("rice")];
    let p = SearchParams::default();
    let generated = candidates::generate(&s);
    let per_slot: Vec<Vec<Feasible>> = generated.iter().map(|g| tier0::filter(&s, g).0).collect();
    // Greedy: best single-slot candidate every night.
    let mut greedy = Vec::new();
    for feasible in &per_slot {
        let best = feasible
            .iter()
            .max_by_key(|f| score_plan(&s, &p, std::slice::from_ref(*f), 4).score)
            .unwrap();
        greedy.push(best.clone());
    }
    assert!(
        greedy
            .iter()
            .all(|f| f.candidate.text().contains("recipe:r-a")),
        "greedy repeats the pantry dish"
    );
    let greedy_score = score_plan(&s, &p, &greedy, 4).score;
    let r = run(&s);
    let keys = chosen_keys(&r);
    for pair in keys.windows(2) {
        assert_ne!(pair[0], pair[1], "adjacent repeat in {keys:?}");
    }
    assert!(
        r.score.score >= greedy_score,
        "{:?} vs {greedy_score:?}",
        r.score.score
    );
    assert!(
        r.score.score > greedy_score,
        "the fixture must separate them"
    );
    assert!(keys.iter().all(|k| k.starts_with("recipe:")));
}

/// AC-1 at two beam widths, the lock and veto cases included: nothing Tier 0 rejected is ever
/// selected, and every rejection is recorded.
#[test]
fn tier0_rejections_are_never_selected_at_beam_widths_1_and_64() {
    let mut s = base();
    s.length_days = 4;
    s.recipes.push(recipe(
        "r-pb",
        "Peanut noodles",
        Some(10),
        &["peanut butter"],
    ));
    s.recipes
        .push(recipe("r-slow", "Slow", Some(120), &["beef"]));
    s.policies.hard_vetoes = vec!["soup".to_owned()];
    s.existing = vec![meal(
        "l",
        "2026-08-30",
        MealSlot::Dinner,
        vec![rc("r-tofu")],
        true,
    )];
    for width in [1usize, 64] {
        for k in [1usize, 12] {
            let params = SearchParams {
                beam_width: width,
                candidates_per_slot: k,
                commitment_horizon_days: 2,
            };
            let r = cover_cycle(&s, &params);
            assert_eq!(r.search.beam_width, width as u32);
            let rejected: Vec<&str> = r
                .rejections
                .iter()
                .map(|x| x.candidate_text.as_str())
                .collect();
            assert!(!rejected.is_empty());
            for (slot, p) in r.slots.iter().zip(&r.proposed) {
                let text = format!(
                    "{} {} {} {}",
                    format_civil_date(p.date),
                    p.slot.as_str(),
                    slot.source.unwrap().as_str(),
                    snapshot::components_text(&p.components)
                );
                assert!(
                    !rejected.contains(&text.as_str()),
                    "B={width} K={k} selected {text}"
                );
                let c = snapshot::components_text(&p.components);
                assert!(!c.contains("r-pb") && !c.contains("r-slow") && !c.contains("r-soup"));
            }
            assert_eq!(
                r.proposed[1].components,
                vec![rc("r-tofu")],
                "lock kept at B={width}"
            );
            assert_eq!(r.slots[1].state, CoverageState::LockedByUser);
            let codes: Vec<&str> = r.rejections.iter().map(|x| x.code.as_str()).collect();
            for expected in [
                LOCK_CONFLICT,
                HARD_VETO,
                PREP_WINDOW_IMPOSSIBLE,
                "RESTRICTION_CONFLICT:rules_v1",
            ] {
                assert!(codes.contains(&expected), "{expected} missing at B={width}");
            }
        }
    }
}

#[test]
fn ties_break_on_canonical_text() {
    // Two recipes indistinguishable to every term: the lexically smaller plan text wins, and
    // it wins the same way at every width.
    let mut s = base();
    s.length_days = 1;
    s.preferences = vec![prefs(
        "m1",
        &[
            (Sentiment::Like, "x"),
            (Sentiment::Like, "y"),
            (Sentiment::Like, "z"),
        ],
    )];
    s.recipes = vec![
        recipe("r-b", "B", Some(15), &["q"]),
        recipe("r-a", "A", Some(15), &["q"]),
    ];
    for width in [1usize, 2, 8] {
        let r = cover_cycle(
            &s,
            &SearchParams {
                beam_width: width,
                ..SearchParams::default()
            },
        );
        assert_eq!(r.proposed[0].components, vec![rc("r-a")], "B={width}");
    }
}

#[test]
fn states_scored_equals_the_per_slot_sum_from_the_trace() {
    let mut s = base();
    s.length_days = 3;
    s.existing = vec![meal(
        "l",
        "2026-08-30",
        MealSlot::Dinner,
        vec![rc("r-tofu")],
        true,
    )];
    let params = SearchParams {
        beam_width: 2,
        candidates_per_slot: 2,
        commitment_horizon_days: 2,
    };
    let r = cover_cycle(&s, &params);
    // Slot 1: 1 live × min(feasible, K=2) = 2; slot 2 resolved: 2 live × 1; slot 3: 2 × 3 —
    // K=2 plus the leftovers candidate the locked day-two dish sources, which is exempt from
    // the cut because ranked alone it can never find that source.
    assert_eq!(r.search.states_scored, 2 + 2 + 6);
    assert_eq!(r.search.slot_order, "canonical");
    assert_eq!(r.search.candidates_per_slot, 2);
    // The plan's worked example: 21 full slots at B=8, K=12. Slot 1 is 1 live × 12; day one's
    // other two slots are 8 × 12, because the day before the anchor is outside the window and
    // history is empty, so no leftovers candidate is generated for them; each of the 18 slots
    // on days two to seven is 8 × 13 — K plus the truncation-exempt leftovers candidate.
    let mut big = base();
    big.length_days = 7;
    big.scope = MealScope::new(MealSlot::ALL).unwrap();
    for i in 0..12 {
        big.recipes.push(recipe(
            &format!("r-{i}"),
            &format!("R{i}"),
            Some(10),
            &["z"],
        ));
    }
    let r = run(&big);
    assert_eq!(r.search.states_scored, 12 + 2 * 96 + 18 * 104);
}

#[test]
fn a_resolved_slot_expands_to_one_state() {
    let mut s = base();
    s.existing = vec![meal(
        "l",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-tofu")],
        true,
    )];
    let r = run(&s);
    let generated = candidates::generate(&s);
    let (feasible, _) = tier0::filter(&s, &generated[0]);
    assert_eq!(feasible.len(), 1);
    let second = tier0::filter(&s, &generated[1]).0.len().min(12) as u32;
    assert_eq!(r.search.states_scored, 1 + second);
}

/// A household with no preferences, no pantry marks and no history gives every tier-5 term
/// nothing to say; the frozen/quick placeholder must still lose to a real dish rather than
/// win the text tie-break.
#[test]
fn a_recipe_beats_the_frozen_quick_fallback_on_a_signal_less_household() {
    let mut s = base();
    s.preferences.clear();
    s.policies.slot_windows.clear();
    s.recipes = vec![recipe("z-only", "Pancakes", None, &[])];
    let r = run(&s);
    assert_eq!(r.proposed[0].components, vec![rc("z-only")]);
    // Day two may be the recipe again or its leftovers (both real meals); never the
    // placeholder the text tie-break used to pick.
    assert!(
        r.slots
            .iter()
            .all(|slot| slot.source != Some(CandidateSource::FrozenQuick)),
        "{:?}",
        r.proposed
    );
    assert!(r.score.terms.iter().all(|t| t.code != FALLBACK_PLACEHOLDER));
    // And when only fallbacks are feasible, the placeholder is still placed and coded.
    let mut s2 = s.clone();
    s2.policies.hard_vetoes = vec!["pancakes".to_owned()];
    let r2 = run(&s2);
    assert!(r2
        .score
        .terms
        .iter()
        .any(|t| t.code == FALLBACK_PLACEHOLDER && t.value == -1));
}

/// Three dislikes covering every stock recipe: each real dish takes a negative tier-2 floor
/// while the view-less placeholder keeps 0, and `Score`'s lexicographic `Ord` decides at index
/// 1 — long before the tier-4 `FALLBACK_PLACEHOLDER` that is supposed to discourage it.
fn every_recipe_disliked() -> PlanningSnapshot {
    let mut s = base();
    s.preferences = vec![prefs(
        "m1",
        &[
            (Sentiment::Dislike, "rice"),
            (Sentiment::Dislike, "carrot"),
            (Sentiment::Dislike, "tofu"),
        ],
    )];
    s
}

/// §10: a placeholder that *beat* a real meal option is not a covered slot. Every check in the
/// covered branch is gated on the candidate having a dish, so before the fix the placeholder
/// passed them all vacuously and the whole week reported `Covered` with no issue copy.
#[test]
fn a_chosen_placeholder_over_an_available_meal_is_not_covered() {
    let s = every_recipe_disliked();
    let r = run(&s);
    for slot in &r.slots {
        assert_eq!(
            slot.source,
            Some(CandidateSource::FrozenQuick),
            "the fixture must reach the contested branch"
        );
        assert_eq!(slot.state, CoverageState::TentativelyCovered);
        assert!(slot
            .reason_codes
            .contains(&FALLBACK_PLACEHOLDER_CHOSEN.to_owned()));
        // Not the `needs_attention` case: the recipes survived Tier 0, they just lost.
        assert!(!slot.reason_codes.contains(&PLAN_INFEASIBLE.to_owned()));
    }
    assert_ne!(r.assessment.status, OutcomeStatus::Covered);
    assert!(
        r.assessment
            .unresolved_issues
            .iter()
            .any(|t| t.contains("no dish is decided")),
        "{:?}",
        r.assessment.unresolved_issues
    );
}

/// The other side of the same rule: leftovers the search found a dish for is a real meal, so
/// the placeholder grading must not fire on it. `CandidateSource::is_meal` alone cannot tell.
#[test]
fn a_sourced_leftovers_slot_is_not_flagged_as_a_placeholder() {
    let mut s = base();
    s.policies.hard_vetoes = vec!["rice".to_owned(), "soup".to_owned(), "tofu".to_owned()];
    s.existing = vec![meal(
        "l",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-tofu")],
        true,
    )];
    let r = run(&s);
    let day_two = &r.slots[1];
    assert_eq!(
        day_two.source,
        Some(CandidateSource::Leftovers),
        "the locked dish the day before feeds it"
    );
    assert!(!day_two
        .reason_codes
        .contains(&FALLBACK_PLACEHOLDER_CHOSEN.to_owned()));
}

/// The placeholder grading reads what the slot *holds*, not who put it there. It used to key
/// on `candidate.source`, which made a stored frozen/quick occurrence (`ExistingPlan`, a meal
/// source) grade `Covered` while the identical components placed by the search graded
/// `TentativelyCovered` — and since `source` is the one input that changes identity across an
/// apply, the two graders disagreed over the same stored state. `cover_cycle` writes the
/// second into a ledger row's `resulting_status` and `assess` writes the first into the next
/// row's `prior_status`, so the disagreement became a permanent break in a table whose
/// `UPDATE` and `DELETE` are both refused by trigger.
#[test]
fn a_stored_frozen_quick_occurrence_is_graded_like_any_other_placeholder() {
    let mut s = every_recipe_disliked();
    s.existing = vec![meal(
        "own",
        "2026-08-29",
        MealSlot::Dinner,
        vec![MealComponent::FrozenQuick {
            note: Some("from the freezer".to_owned()),
        }],
        false,
    )];
    let r = run(&s);
    assert_eq!(
        r.slots[0].source,
        Some(CandidateSource::ExistingPlan),
        "the household's own occurrence must win for this test to mean anything"
    );
    assert!(r.slots[0]
        .reason_codes
        .contains(&FALLBACK_PLACEHOLDER_CHOSEN.to_owned()));
    assert_eq!(r.slots[0].state, CoverageState::TentativelyCovered);
}

/// The chain invariant at its source: for the same stored state, `assess` and `cover_cycle`
/// must reach the same cycle status, or the ledger's `prior`/`resulting` pair records a
/// transition nothing performed.
#[test]
fn assess_grades_a_stored_placeholder_the_same_way_the_search_does() {
    let mut s = every_recipe_disliked();
    let planned = run(&s);
    assert!(
        planned
            .slots
            .iter()
            .all(|slot| slot.source == Some(CandidateSource::FrozenQuick)),
        "the fixture must plan placeholders for this test to mean anything"
    );
    // Store the plan back, exactly as an apply would.
    s.existing = planned
        .proposed
        .iter()
        .enumerate()
        .map(|(i, p)| {
            meal(
                &format!("applied-{i}"),
                &format_civil_date(p.date),
                p.slot,
                p.components.clone(),
                false,
            )
        })
        .collect();
    assert_eq!(assess_existing(&s), planned.assessment.status);
}

/// `leftovers_sourced` says one specific thing — *this* leftovers candidate has a dish behind
/// it. It must never promote a frozen/quick placeholder that merely happens to sit a day after
/// a dish that feeds leftovers, or the placeholder grading silently stops firing on exactly
/// the households that cook in bulk.
#[test]
fn a_leftovers_source_does_not_promote_a_frozen_quick_placeholder() {
    let mut s = every_recipe_disliked();
    s.existing = vec![
        // Day one feeds leftovers (4 servings, one member) …
        meal(
            "day-one",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-rice")],
            false,
        ),
        // … and day two holds a frozen/quick placeholder, which is still a placeholder.
        meal(
            "day-two",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::FrozenQuick { note: None }],
            false,
        ),
    ];
    let r = run(&s);
    assert_eq!(r.slots[1].state, CoverageState::TentativelyCovered);
    assert!(r.slots[1]
        .reason_codes
        .contains(&FALLBACK_PLACEHOLDER_CHOSEN.to_owned()));
    // And the two graders still agree over that stored state.
    assert_eq!(assess_existing(&s), r.assessment.status);
}

/// The other half of the component rule: leftovers the previous day's stored dish can feed is
/// a real meal on the `assess` path too. Without this, closing the frozen/quick chain break
/// would have opened a leftovers one.
#[test]
fn a_stored_sourced_leftovers_slot_is_still_covered_on_assess() {
    let mut s = base();
    s.existing = vec![
        meal(
            "day-one",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-rice")],
            false,
        ),
        meal(
            "day-two",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::Leftovers { note: None }],
            false,
        ),
    ];
    let assessed = FoodController.assess(&s).unwrap();
    assert_eq!(assessed.status, OutcomeStatus::Covered);
    assert!(!assessed
        .reason_codes
        .iter()
        .any(|c| c.as_str() == FALLBACK_PLACEHOLDER_CHOSEN));
}

/// A starter's *candidate* carries `views`; the occurrence an apply writes for it is a bare
/// `Freeform` stub. Everything the two graders derive from views — the leftovers source, Tier
/// 0's restriction and hard-veto checks, the prep window — therefore has to survive the round
/// trip, or `cover_cycle` and `assess` write contradicting halves of one ledger row.
///
/// The fixture is starter-only with a 35-minute window: prep 20 leaves a slack of 15, which
/// earns no tier-3 term either way, and "oat" matches no preference, so tiers 1–3 tie between
/// two starters and starter-then-leftovers. Tier 4 then decides — a second `Freeform` stub
/// costs `STUB_NEEDS_DECISION` where leftovers cost nothing — which is what puts a starter and
/// a leftovers slot that depends on it in the same plan.
fn starter_only() -> PlanningSnapshot {
    let mut s = base();
    s.recipes = vec![];
    s.starter = vec![starter("s-01", "Oat bake", &["oats"])];
    s.policies.slot_windows = BTreeMap::from([(MealSlot::Dinner, 35)]);
    s
}

/// The stored plan an apply would leave behind, as `assess_grades_a_stored_placeholder_the_same_way_the_search_does`
/// builds it.
fn stored_as_applied(planned: &PlanningResult) -> Vec<PlannedMeal> {
    planned
        .proposed
        .iter()
        .enumerate()
        .map(|(i, p)| {
            meal(
                &format!("applied-{i}"),
                &format_civil_date(p.date),
                p.slot,
                p.components.clone(),
                false,
            )
        })
        .collect()
}

#[test]
fn a_starter_plan_grades_the_same_before_and_after_an_apply() {
    let mut s = starter_only();
    let planned = run(&s);
    assert_eq!(
        planned
            .slots
            .iter()
            .map(|slot| slot.source)
            .collect::<Vec<_>>(),
        vec![
            Some(CandidateSource::StarterMeal),
            Some(CandidateSource::Leftovers)
        ],
        "the fixture must plan a starter and then leftovers sourced from it: {:?}",
        planned.slots
    );
    assert_eq!(planned.assessment.status, OutcomeStatus::Covered);
    s.existing = stored_as_applied(&planned);
    assert_eq!(
        assess_existing(&s),
        planned.assessment.status,
        "the applied starter stub must still source the leftovers it was chosen to source"
    );
}

/// Tier 0's hard veto reads `candidate.views`. A stub with none passes it vacuously, so a veto
/// the household records *after* a starter was applied could never reject the occurrence — and
/// `NON_RECIPE_COMPONENT_UNVERIFIED`, the only code the stub did pick up, has no `ISSUE_TEXT`
/// entry and is not in `CLAIM_BLOCKING`, so nothing reached the user either.
#[test]
fn a_hard_veto_added_after_a_starter_was_applied_rejects_the_stored_stub() {
    let mut s = starter_only();
    s.existing = stored_as_applied(&run(&s));
    s.policies.hard_vetoes = vec!["oats".to_owned()];
    let assessed = FoodController.assess(&s).unwrap();
    assert_eq!(assessed.status, OutcomeStatus::NeedsAttention);
    assert!(
        assessed
            .reason_codes
            .iter()
            .any(|c| c.as_str() == HARD_VETO),
        "{assessed:?}"
    );
}

/// The loader drops a slug from `snapshot.starter` once an active recipe carries it, so after
/// the household installs the starter the recipe is the only place its facts live. The stub
/// still on the calendar has to follow the dish there.
#[test]
fn a_stored_starter_stub_resolves_to_the_recipe_once_installed() {
    let mut s = starter_only();
    s.existing = stored_as_applied(&run(&s));
    // Install it, exactly as the loader would then present it: out of `starter`, into `recipes`.
    let mut installed = recipe("r-oats", "Oat bake", Some(20), &["oats"]);
    installed.starter_slug = Some("s-01".to_owned());
    s.recipes = vec![installed];
    s.starter = vec![];
    assert_eq!(
        assess_existing(&s),
        OutcomeStatus::Covered,
        "the installed recipe still feeds the leftovers slot that followed it"
    );
    // And it is the *recipe's* facts that were picked up, not a name-only placeholder: the slug
    // `s-01` tokenises to nothing a veto on "oats" could match, so only a resolved view hits.
    let mut vetoed = s.clone();
    vetoed.policies.hard_vetoes = vec!["oats".to_owned()];
    assert_eq!(assess_existing(&vetoed), OutcomeStatus::NeedsAttention);
}

/// A slug is written to be human-readable, so carrying it in `RecipeView::title` fed a matchable
/// dish name to the two consumers that tokenize that field — `veto_hit` and `matches_subject` —
/// for a dish whose facts are entirely unknown. `key` keeps the slug for repeat detection; only
/// the matched string goes.
#[test]
fn an_unknown_starter_slug_is_not_matched_as_a_dish_name() {
    let mut s = starter_only();
    // `starter_only`'s own `s-01` tokenises to ["s", "01"] and could never reach the veto, and
    // four other tests share that fixture — so the slug is overridden here, not in the builder.
    s.starter = vec![starter("oat-bake", "Oat bake", &["oats"])];
    s.existing = stored_as_applied(&run(&s));
    // Installed, then archived: out of `starter` and out of `recipes`, so the stub falls to the
    // name-only arm and the slug `oat-bake` is all that is left of the dish.
    s.starter = vec![];
    s.policies.hard_vetoes = vec!["oat".to_owned()];
    let assessed = FoodController.assess(&s).unwrap();
    assert_ne!(
        assessed.status,
        OutcomeStatus::NeedsAttention,
        "an identifier is not a dish name: nothing is known about this slug, so the veto has \
         nothing to read: {assessed:?}"
    );
    assert!(
        !assessed
            .reason_codes
            .iter()
            .any(|c| c.as_str() == HARD_VETO),
        "{assessed:?}"
    );
    // Still withheld, by the route that is actually about knowledge: prep time is unknown.
    assert!(assessed
        .reason_codes
        .iter()
        .any(|c| c.as_str() == NO_PREP_TIME_ESTIMATES));
}

/// Installed and then archived: the loader's held-slug query counts archived recipes, so the
/// slug is out of `starter`, while its recipe query excludes them, so it is out of `recipes`
/// too. Nothing can be verified about the dish — which must read as unknown, not as a slot with
/// no schedule claim to make. An empty view list short-circuits `schedule_fit_known` to `true`.
#[test]
fn a_starter_stub_for_an_unknown_slug_withholds_the_schedule_claim() {
    let mut s = starter_only();
    s.existing = stored_as_applied(&run(&s));
    s.starter = vec![];
    let assessed = FoodController.assess(&s).unwrap();
    assert_eq!(assessed.status, OutcomeStatus::TentativelyCovered);
    assert!(
        assessed
            .reason_codes
            .iter()
            .any(|c| c.as_str() == NO_PREP_TIME_ESTIMATES),
        "{assessed:?}"
    );
}

/// The negative arm. Only the `starter:` prefix resolves, and it is a prefix match, not a
/// substring one — a household's own freeform note stays unverifiable. `NON_RECIPE_COMPONENT_
/// UNVERIFIED` records that internally and `FREEFORM_DISH_UNVERIFIED` is what says it to the
/// household, because the first has no `ISSUE_TEXT` entry.
#[test]
fn a_plain_freeform_occurrence_gains_no_view() {
    let mut s = base();
    s.policies.hard_vetoes = vec!["pizza".to_owned()];
    s.existing = vec![
        meal(
            "day-one",
            "2026-08-29",
            MealSlot::Dinner,
            vec![MealComponent::freeform("pizza night").unwrap()],
            false,
        ),
        meal(
            "day-two",
            "2026-08-30",
            MealSlot::Dinner,
            vec![rc("r-rice")],
            false,
        ),
    ];
    let assessed = FoodController.assess(&s).unwrap();
    assert_ne!(
        assessed.status,
        OutcomeStatus::NeedsAttention,
        "a note is not a dish, so there is nothing for the veto to read: {assessed:?}"
    );
    assert!(assessed
        .reason_codes
        .iter()
        .any(|c| c.as_str() == NON_RECIPE_COMPONENT_UNVERIFIED));
}

/// Every check in `slot_coverage`'s covered branch is gated on the candidate having a view, so a
/// note that resolves to none passed all of them vacuously and the slot claimed `Covered` — a
/// restriction check that ran against nothing reported no conflict, and the household was never
/// told. §10: never imply a claim the state cannot support.
#[test]
fn a_plain_freeform_occurrence_withholds_the_coverage_claim() {
    let mut s = base();
    s.existing = vec![
        meal(
            "day-one",
            "2026-08-29",
            MealSlot::Dinner,
            vec![MealComponent::freeform("satay night").unwrap()],
            false,
        ),
        meal(
            "day-two",
            "2026-08-30",
            MealSlot::Dinner,
            vec![rc("r-rice")],
            false,
        ),
    ];
    assert!(
        !s.restrictions.restrictions().is_empty(),
        "restrictions configured is the premise: the claim withheld is that they were checked"
    );
    let assessed = FoodController.assess(&s).unwrap();
    assert_eq!(assessed.status, OutcomeStatus::TentativelyCovered);
    assert!(
        assessed
            .reason_codes
            .iter()
            .any(|c| c.as_str() == FREEFORM_DISH_UNVERIFIED),
        "{assessed:?}"
    );
    assert!(
        assessed
            .unresolved_issues
            .iter()
            .any(|t| t == issue_text(FREEFORM_DISH_UNVERIFIED)),
        "the code has to reach the household as a sentence, not only the ledger: {assessed:?}"
    );
}

/// The consistency this is all for: a `starter:<slug>` stub whose slug is in neither list and a
/// plain note carry exactly as much knowledge about the dish — none — so they cannot grade
/// differently. The principle belongs to the component kind, not to one string prefix.
#[test]
fn a_freeform_note_and_an_unknown_starter_stub_grade_alike() {
    let dinner = |note: &str| {
        let mut s = base();
        s.starter = vec![];
        s.existing = vec![
            meal(
                "day-one",
                "2026-08-29",
                MealSlot::Dinner,
                vec![MealComponent::freeform(note).unwrap()],
                false,
            ),
            meal(
                "day-two",
                "2026-08-30",
                MealSlot::Dinner,
                vec![rc("r-rice")],
                false,
            ),
        ];
        FoodController.assess(&s).unwrap()
    };
    let note = dinner("satay night");
    let stub = dinner("starter:not-installed");
    assert_eq!(note.status, stub.status, "{note:?} vs {stub:?}");
    assert_eq!(note.status, OutcomeStatus::TentativelyCovered);
    // The same claim withheld, by the two routes that exist to withhold it: the stub has a
    // facts-free view whose prep time is unknown, the note has no view at all.
    assert!(stub
        .reason_codes
        .iter()
        .any(|c| c.as_str() == NO_PREP_TIME_ESTIMATES));
    assert!(note
        .reason_codes
        .iter()
        .any(|c| c.as_str() == FREEFORM_DISH_UNVERIFIED));
}

/// Expected-to-pass: the boundary the `views.is_empty()` conjunct draws. `add_component` refuses
/// only an `Open` beside another, so a recipe with a freeform side is a supported occurrence —
/// and its recipe half *is* checked, so the code's sentence would be false about it.
#[test]
fn a_recipe_with_a_freeform_side_is_not_called_unverified() {
    let mut s = base();
    s.existing = vec![
        meal(
            "day-one",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-rice"), MealComponent::freeform("side salad").unwrap()],
            false,
        ),
        meal(
            "day-two",
            "2026-08-30",
            MealSlot::Dinner,
            vec![rc("r-soup")],
            false,
        ),
    ];
    let assessed = FoodController.assess(&s).unwrap();
    assert!(
        !assessed
            .reason_codes
            .iter()
            .any(|c| c.as_str() == FREEFORM_DISH_UNVERIFIED),
        "{assessed:?}"
    );
    assert!(assessed
        .reason_codes
        .iter()
        .any(|c| c.as_str() == NON_RECIPE_COMPONENT_UNVERIFIED));
}

/// A one-serving recipe: the day-1 dinner has to be *filled* for the cycle not to be
/// `Unresolved`, but must not itself feed leftovers, or the out-of-scope lunch under test
/// stops being the only possible source.
fn small_plate() -> RecipeCandidateInfo {
    let mut r = recipe("r-small", "Small plate", Some(15), &["carrot"]);
    r.servings = Some(1);
    r
}

/// `existing` is loaded by date range with no slot filter, and `save_planned_meal_in` validates
/// against the *current* scope — so a household that narrows its cycle keeps stored occurrences
/// on slots the cycle no longer plans. The search cannot see them (it reads only what it
/// placed); `assess` could, and crediting them made one grader call a slot covered while the
/// other churned the user's leftovers away as unsourced.
#[test]
fn assess_does_not_credit_a_leftovers_source_outside_the_cycle_scope() {
    let mut s = base();
    s.recipes.push(small_plate());
    s.existing = vec![
        meal(
            "day-one-dinner",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-small")],
            false,
        ),
        // Planned while lunches were in scope; the cycle is dinner-only now.
        meal(
            "day-one-lunch",
            "2026-08-29",
            MealSlot::Lunch,
            vec![rc("r-rice")],
            false,
        ),
        meal(
            "day-two-dinner",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::Leftovers { note: None }],
            false,
        ),
    ];
    assert_eq!(s.scope.slots(), &[MealSlot::Dinner], "scope is the premise");
    let assessed = FoodController.assess(&s).unwrap();
    assert_eq!(assessed.status, OutcomeStatus::TentativelyCovered);
    assert!(
        assessed
            .reason_codes
            .iter()
            .any(|c| c.as_str() == FALLBACK_PLACEHOLDER_CHOSEN),
        "{assessed:?}"
    );
}

/// Expected-to-pass: the other side of the same partition. `score_plan` reads `snapshot.history`
/// whole into `history_views_by_date` with no slot filter, so the scope filter above must stop
/// at the anchor — pushing it into history would recreate the divergence facing the other way.
#[test]
fn a_pre_anchor_history_source_still_counts_whatever_its_slot() {
    let mut s = base();
    s.recipes.push(small_plate());
    s.history = vec![meal(
        "yesterday-lunch",
        "2026-08-28",
        MealSlot::Lunch,
        vec![rc("r-rice")],
        false,
    )];
    s.existing = vec![
        meal(
            "day-one-dinner",
            "2026-08-29",
            MealSlot::Dinner,
            vec![MealComponent::Leftovers { note: None }],
            false,
        ),
        meal(
            "day-two-dinner",
            "2026-08-30",
            MealSlot::Dinner,
            vec![rc("r-small")],
            false,
        ),
    ];
    let assessed = FoodController.assess(&s).unwrap();
    assert_eq!(assessed.status, OutcomeStatus::Covered);
    assert!(!assessed
        .reason_codes
        .iter()
        .any(|c| c.as_str() == FALLBACK_PLACEHOLDER_CHOSEN));
}

/// The other end of the same partition, enforced rather than documented. The loader gives
/// `history` nothing from the anchor on, but `PlanningSnapshot`'s fields are public and hand-built
/// snapshots are a supported case — every test here is one. `score::leftover_source` guards its
/// half with `previous < anchor`; without the same guard the two graders re-diverge on exactly
/// that input, and `assess` would call a slot covered that the search scores
/// `LEFTOVER_SOURCE_MISSING`, splitting a ledger row's `resulting_status` from the next row's
/// `prior_status`.
#[test]
fn assess_does_not_credit_a_history_source_from_inside_the_cycle() {
    let mut s = base();
    s.recipes.push(small_plate());
    // Dated *on* the anchor, which is the boundary the guard closes.
    s.history = vec![meal(
        "in-cycle",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-rice")],
        false,
    )];
    s.existing = vec![
        // One serving, so the in-scope `existing` half cannot credit the slot either and the
        // history row is the only thing under test. The day still has to be filled, or the
        // cycle is `Unresolved` on both sides of the guard and the test proves nothing.
        meal(
            "day-one-dinner",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-small")],
            false,
        ),
        meal(
            "day-two-dinner",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::Leftovers { note: None }],
            false,
        ),
    ];
    assert_eq!(s.anchor, d("2026-08-29"), "the boundary is the premise");
    let assessed = FoodController.assess(&s).unwrap();
    assert_eq!(assessed.status, OutcomeStatus::TentativelyCovered);
    assert!(
        assessed
            .reason_codes
            .iter()
            .any(|c| c.as_str() == FALLBACK_PLACEHOLDER_CHOSEN),
        "{assessed:?}"
    );
}

/// Expected-to-pass: `assess` now computes the leftovers-source flag only for a candidate that
/// actually holds leftovers, which is the only candidate `slot_coverage` reads it for. The gate
/// must not change any grade — `a_leftovers_source_does_not_promote_a_frozen_quick_placeholder`
/// pins this on the `cover_cycle` path; this is the `assess` half.
#[test]
fn a_stored_placeholder_after_a_leftovers_source_is_still_a_placeholder_on_assess() {
    let mut s = base();
    s.existing = vec![
        meal(
            "day-one",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-rice")],
            false,
        ),
        meal(
            "day-two",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::FrozenQuick { note: None }],
            false,
        ),
    ];
    let assessed = FoodController.assess(&s).unwrap();
    assert_eq!(assessed.status, OutcomeStatus::TentativelyCovered);
    assert!(
        assessed
            .reason_codes
            .iter()
            .any(|c| c.as_str() == FALLBACK_PLACEHOLDER_CHOSEN),
        "{assessed:?}"
    );
}

/// Fifteen recipes that every member dislikes, so the view-less leftovers candidate wins on
/// tier 2 — but only if it is still in the running. Ranked alone at index 0 a leftovers
/// candidate can never find its source, so it scores `LEFTOVER_SOURCE_MISSING` and sorts
/// strictly last; at `K = 12` truncation dropped it before the whole-plan search could judge
/// it, making PRD §9.3's leftovers source unreachable for any household past a dozen recipes.
fn fifteen_disliked_recipes() -> PlanningSnapshot {
    let mut s = base();
    s.preferences = vec![prefs(
        "m1",
        &[
            (Sentiment::Dislike, "rice"),
            (Sentiment::Dislike, "olives"),
            (Sentiment::Dislike, "capers"),
        ],
    )];
    s.recipes = (0..15)
        .map(|i| {
            recipe(
                &format!("r-{i:02}"),
                &format!("Dish {i:02}"),
                Some(15),
                &["rice"],
            )
        })
        .collect();
    s
}

#[test]
fn a_sourced_leftovers_slot_survives_k_truncation() {
    let mut s = fifteen_disliked_recipes();
    s.existing = vec![meal(
        "day-one",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-00")],
        true,
    )];
    assert!(
        s.recipes.len() > SearchParams::default().candidates_per_slot,
        "the fixture must exceed K for this test to mean anything"
    );
    let r = run(&s);
    assert_eq!(
        r.slots[1].source,
        Some(CandidateSource::Leftovers),
        "day two: {:?}",
        r.slots[1]
    );
    assert!(r
        .score
        .terms
        .iter()
        .any(|t| t.code == LEFTOVER_UTILITY && t.date == Some(d("2026-08-30"))));
}

/// The exemption must only stop premature truncation, never promote a candidate the search
/// would not otherwise place: with no dish the day before, no leftovers candidate is even
/// generated, so no slot may hold one.
#[test]
fn the_leftovers_exemption_does_not_place_unsourced_leftovers() {
    let s = fifteen_disliked_recipes();
    let r = run(&s);
    assert!(
        r.slots
            .iter()
            .all(|slot| slot.source != Some(CandidateSource::Leftovers)),
        "{:?}",
        r.slots
    );
}

/// The payload is append-only — the v10 trigger refuses `DELETE` — so no rejection code may
/// grow the row with the cycle's slot count. Before the budget a restriction- or veto-hit
/// recipe emitted one line per enabled slot: 93 slots × 40 hit recipes is ~279 KB in a single
/// row, and every preview appended another one.
fn twelve_vetoed_recipes(length_days: u32) -> PlanningSnapshot {
    let mut s = base();
    s.length_days = length_days;
    s.policies.hard_vetoes = vec!["quinoa".to_owned()];
    s.recipes = (0..12)
        .map(|i| {
            recipe(
                &format!("r-{i:02}"),
                &format!("Dish {i:02}"),
                Some(15),
                &["quinoa"],
            )
        })
        .collect();
    s
}

#[test]
fn the_ledger_payload_does_not_grow_with_slot_count_for_a_fixed_rejection_set() {
    let rejection_lines = |days: u32| -> usize {
        run(&twelve_vetoed_recipes(days))
            .canonical_text()
            .lines()
            .filter(|l| l.starts_with("rejected=") && !l.contains(LOCK_CONFLICT))
            .count()
    };
    let short = rejection_lines(2);
    let long = rejection_lines(14);
    assert!(short > 0, "the fixture must produce rejections");
    assert_eq!(short, long, "short={short} long={long}");
}

/// The budget must not cost the diagnostic in the common case: a payload under it keeps every
/// rejection line, candidate text and all.
#[test]
fn a_payload_under_the_budget_keeps_every_rejection_line() {
    let mut s = base();
    s.policies.hard_vetoes = vec!["carrot".to_owned()];
    let r = run(&s);
    let text = r.canonical_text();
    let expected = r
        .rejections
        .iter()
        .filter(|x| x.code != LOCK_CONFLICT)
        .count();
    assert!(expected > 0);
    assert_eq!(
        text.lines()
            .filter(|l| l.starts_with("rejected=") && !l.contains(LOCK_CONFLICT))
            .count(),
        expected
    );
    assert!(!text.contains("... and"));
}

/// The ledger payload is append-only — the v10 trigger refuses `DELETE` — so the one code that
/// scales with `slots × candidates` is summarised. Codes that name a dish a check refused keep
/// their candidate text: that text is the diagnostic.
#[test]
fn the_ledger_payload_collapses_lock_conflicts_to_counts() {
    let mut s = base();
    s.policies.hard_vetoes = vec!["soup".to_owned()];
    s.existing = vec![meal(
        "l",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-tofu")],
        true,
    )];
    let r = run(&s);
    let text = r.canonical_text();
    let locked: Vec<&str> = text
        .lines()
        .filter(|l| l.starts_with(&format!("rejected={LOCK_CONFLICT} ")))
        .collect();
    assert_eq!(
        locked.len(),
        1,
        "one line for the one locked slot: {locked:?}"
    );
    let expected = r
        .rejections
        .iter()
        .filter(|x| x.code == LOCK_CONFLICT)
        .count();
    assert!(
        expected > 1,
        "the locked slot must reject several candidates"
    );
    assert_eq!(
        locked[0],
        format!("rejected={LOCK_CONFLICT} 2026-08-29 dinner count={expected}")
    );
    // A veto rejection still carries the dish it refused, whole.
    assert!(
        text.lines()
            .any(|l| l.starts_with(&format!("rejected={HARD_VETO} ")) && l.contains("r-soup")),
        "{text}"
    );
}

/// The property the summary exists for: the payload's rejection lines stop growing with the
/// household's recipe count, while the domain result still carries every rejection.
#[test]
fn payload_lock_conflict_lines_do_not_grow_with_recipe_count() {
    let mut small = base();
    small.existing = vec![meal(
        "l",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-tofu")],
        true,
    )];
    let mut big = small.clone();
    for i in 0..21 {
        // Conflict-free by construction: no peanut restriction hit, no veto (`base()` has
        // none) and 15 minutes against the 40-minute dinner window, so the only code these
        // can produce is `LOCK_CONFLICT` on the locked slot.
        big.recipes.push(recipe(
            &format!("r-plain-{i}"),
            &format!("Plain {i}"),
            Some(15),
            &[&format!("z{i}")],
        ));
    }
    let (rs, rb) = (run(&small), run(&big));
    let codes: Vec<&str> = rb.rejections.iter().map(|x| x.code.as_str()).collect();
    assert!(
        codes.iter().all(|c| *c == LOCK_CONFLICT),
        "fixture must isolate the one code: {codes:?}"
    );
    assert!(
        rb.rejections.len() > rs.rejections.len(),
        "more recipes must mean more rejections"
    );
    let lines = |r: &PlanningResult| {
        r.canonical_text()
            .lines()
            .filter(|l| l.starts_with("rejected="))
            .count()
    };
    assert_eq!(lines(&rs), lines(&rb));
}

// --- Step 7: coverage, sufficiency, attention, controller ---------------------------------------

fn assumption(r: &PlanningResult, code: &str) -> bool {
    r.assessment.assumptions.iter().any(|a| a.as_str() == code)
}

#[test]
fn no_prep_time_withholds_schedule_fit_claim() {
    let mut s = base();
    for r in &mut s.recipes {
        r.prep_minutes = None;
    }
    let r = run(&s);
    assert!(assumption(&r, NO_PREP_TIME_ESTIMATES));
    assert_eq!(r.assessment.status, OutcomeStatus::TentativelyCovered);
    let recipe_slot = r
        .slots
        .iter()
        .find(|x| snapshot::components_text(&x.components).starts_with("recipe:"))
        .expect("a recipe chosen");
    assert_eq!(recipe_slot.state, CoverageState::TentativelyCovered);
    assert!(recipe_slot
        .reason_codes
        .contains(&NO_PREP_TIME_ESTIMATES.to_owned()));
    // One estimated recipe among unestimated ones: the estimated one is preferred (slack
    // is a Tier-3 term) and is `Covered` on its own; the cycle still withholds the claim
    // only if an unestimated recipe was placed.
    let mut s3 = base();
    s3.recipes[0].prep_minutes = None;
    let r3 = run(&s3);
    let unestimated_placed = r3
        .slots
        .iter()
        .any(|x| snapshot::components_text(&x.components).contains("r-rice"));
    assert_eq!(assumption(&r3, NO_PREP_TIME_ESTIMATES), unestimated_placed);
    assert!(r
        .assessment
        .unresolved_issues
        .iter()
        .any(|t| t == issue_text(NO_PREP_TIME_ESTIMATES)));
    // No window at all: the same withholding, with every recipe estimated.
    let mut s2 = base();
    s2.policies.slot_windows.clear();
    let r2 = run(&s2);
    assert!(assumption(&r2, NO_PREP_TIME_ESTIMATES));
    assert_ne!(r2.assessment.status, OutcomeStatus::Covered);
}

#[test]
fn pantry_never_yields_a_stock_completeness_claim() {
    let mut s = base();
    s.pantry_marked = vec![
        cat("rice"),
        cat("beans"),
        cat("tofu"),
        cat("carrot"),
        cat("onion"),
    ];
    let r = run(&s);
    assert!(assumption(&r, PANTRY_INCOMPLETE));
    assert!(r
        .assessment
        .unresolved_issues
        .iter()
        .any(|t| t == issue_text(PANTRY_INCOMPLETE)));
    // Not claim-blocking: with everything else known, the cycle is still `Covered`.
    assert_eq!(r.assessment.status, OutcomeStatus::Covered);
    assert!(!CLAIM_BLOCKING.contains(&PANTRY_INCOMPLETE));
}

#[test]
fn no_restrictions_configured_withholds_verification_claim() {
    let mut s = base();
    s.restrictions = HouseholdRestrictions::new([]);
    let r = run(&s);
    assert!(assumption(&r, RESTRICTIONS_NOT_CONFIGURED));
    assert_eq!(r.assessment.status, OutcomeStatus::TentativelyCovered);
    assert!(r
        .slots
        .iter()
        .all(|x| x.state == CoverageState::TentativelyCovered));
    assert!(r
        .assessment
        .unresolved_issues
        .iter()
        .any(|t| t == issue_text(RESTRICTIONS_NOT_CONFIGURED)));
}

/// §10: reviewed absence is confirmed absence. With the marker set and no restrictions
/// stored, no restriction check was skipped — the household said there is nothing to check —
/// so the code is suppressed at both the cycle and the slot level and `Covered` is reachable.
/// Households with configured restrictions never carried the code and are untouched.
#[test]
fn reviewed_and_empty_restrictions_suppress_the_code_and_allow_covered() {
    let mut s = base();
    s.restrictions = HouseholdRestrictions::new([]);
    s.policies.restrictions_reviewed = true;
    let r = run(&s);
    assert!(!assumption(&r, RESTRICTIONS_NOT_CONFIGURED));
    assert!(r.slots.iter().all(|x| {
        !x.reason_codes
            .contains(&RESTRICTIONS_NOT_CONFIGURED.to_owned())
    }));
    assert_eq!(r.assessment.status, OutcomeStatus::Covered);
    assert!(r.slots.iter().all(|x| x.state == CoverageState::Covered));
    // The paired unreviewed household: same fixture family the invariants use. Unreviewed +
    // empty still withholds the claim (the base-derived pin is
    // `no_restrictions_configured_withholds_verification_claim`).
    let f = fixtures::by_name("restrictions_set_and_skipped");
    let skipped = fixtures::restrictions_skipped_variant(&f);
    let r_skipped = run(&skipped);
    assert!(assumption(&r_skipped, RESTRICTIONS_NOT_CONFIGURED));
    let mut reviewed = skipped.clone();
    reviewed.policies.restrictions_reviewed = true;
    assert!(!assumption(&run(&reviewed), RESTRICTIONS_NOT_CONFIGURED));
}

/// AC-3 at the engine layer, scoped by urgency — `Urgency::High` is what the UI's
/// "N things need you" counts. Low-value side: `sparse_pantry` carries `PANTRY_INCOMPLETE`,
/// `RESTRICTIONS_NOT_CONFIGURED` and `PREFERENCES_SPARSE`, and `PREFERENCES_SPARSE` always
/// raises one Low request — so the honest assertion is zero *High*, not zero requests.
/// Material side: `restrictions_set_and_skipped` carries an `Other` restriction whose
/// wording-only match raises a High request (`busy_week` was tried first and raises none:
/// its over-window preps are steered around, not infeasible). Expected-to-pass: pins the
/// MVP-023 engine behavior AC-3 builds on, rather than proving a new change.
#[test]
fn low_value_uncertainty_raises_no_high_urgency_request_but_material_does() {
    let low = fixtures::by_name("sparse_pantry");
    let r_low = run(&low.snapshot);
    assert!(
        !r_low.attention.is_empty(),
        "sparse_pantry still raises its Low request — the High filter must not be vacuous"
    );
    assert!(
        r_low.attention.iter().all(|a| a.urgency != Urgency::High),
        "low-value uncertainty must not demand attention"
    );
    let material = fixtures::by_name("restrictions_set_and_skipped");
    let r_material = run(&material.snapshot);
    let high: Vec<_> = r_material
        .attention
        .iter()
        .filter(|a| a.urgency == Urgency::High)
        .collect();
    assert!(
        !high.is_empty(),
        "restrictions_set_and_skipped must raise a material (High) request; codes: {:?}",
        r_material
            .attention
            .iter()
            .map(|a| a.reason_codes.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn sparse_preferences_label_the_plan_conservative() {
    let mut s = base();
    s.preferences = vec![prefs(
        "m1",
        &[(Sentiment::Like, "rice"), (Sentiment::Like, "tofu")],
    )];
    let r = run(&s);
    assert!(assumption(&r, PREFERENCES_SPARSE));
    assert!(assumption(&r, coverage::CONSERVATIVE_DEFAULT));
    assert_eq!(r.assessment.status, OutcomeStatus::TentativelyCovered);
    let low = r
        .attention
        .iter()
        .find(|a| {
            a.reason_codes
                .iter()
                .any(|c| c.as_str() == PREFERENCES_SPARSE)
        })
        .expect("a low-urgency request");
    assert_eq!(low.urgency, Urgency::Low);
    assert_eq!(low.decision_benefit_band, household_core::Band::Medium);
    assert_eq!(low.estimated_effort_band, household_core::Band::Low);
    assert_eq!(r.proposals[0].confidence, household_core::Confidence::Low);
    // Three entries is the threshold.
    let three = base();
    let mut three_s = three.clone();
    three_s.preferences = vec![prefs(
        "m1",
        &[
            (Sentiment::Like, "rice"),
            (Sentiment::Like, "tofu"),
            (Sentiment::Dislike, "x"),
        ],
    )];
    assert!(!assumption(&run(&three_s), PREFERENCES_SPARSE));
}

#[test]
fn all_four_satisfied_yields_covered() {
    let r = run(&base());
    assert_eq!(r.assessment.status, OutcomeStatus::Covered);
    assert!(r.slots.iter().all(|x| x.state == CoverageState::Covered));
    assert!(!assumption(&r, NO_PREP_TIME_ESTIMATES));
    assert!(!assumption(&r, RESTRICTIONS_NOT_CONFIGURED));
    assert!(!assumption(&r, PREFERENCES_SPARSE));
    assert!(
        assumption(&r, PANTRY_INCOMPLETE),
        "always present, never blocking"
    );
    assert_eq!(r.proposals[0].confidence, household_core::Confidence::High);
    assert_eq!(r.proposals[0].action_type, APPLY_PLAN);
    assert_eq!(r.assessment.horizon.from, "2026-08-29");
    assert_eq!(r.assessment.horizon.to, "2026-08-30");
    assert_eq!(r.algorithm_version, PLANNER_ALGORITHM_VERSION);
    assert_eq!(r.snapshot_hash, base().snapshot_hash());
}

/// A hard veto is the household's Tier-0 safety word, so losing one silently is the wrong
/// failure mode. A subject the matcher can never match is malformed — and the matcher works on
/// `tokens()`, not on trimmed text, so that is every subject with no alphanumeric character,
/// not only the blank ones. Reported whether or not the policy is enabled, which is how the
/// `SLOT_WINDOW` arm already treats its own malformed cases. A *well-formed* disabled veto
/// stays silently ignored: known, not applied.
#[test]
fn an_enabled_unmatchable_subject_veto_is_reported_not_dropped() {
    let veto = |subject: &str, enabled: bool| {
        Policy::new(
            PolicyId::new("p").unwrap(),
            HouseholdId::new("h").unwrap(),
            "food",
            FoodPolicies::HARD_VETO,
            BTreeMap::from([("subject".to_owned(), subject.to_owned())]),
            enabled,
            EvidenceSource::ExplicitUser,
        )
        .unwrap()
    };
    // The last three carry a character but no *token*: `tokens` splits on
    // `!c.is_alphanumeric()` and drops the empties, so `contains_phrase`'s `!phrase.is_empty()`
    // guard makes them match nothing forever. `trim()` alone accepts all three.
    for (subject, enabled) in [
        (" ", true),
        ("", true),
        (" ", false),
        ("\u{0b}", true),
        ("🍝", true),
        ("—", true),
        ("★", false),
    ] {
        let parsed = FoodPolicies::from_policies(&[veto(subject, enabled)]);
        assert!(
            parsed.hard_vetoes.is_empty(),
            "{subject:?}/{enabled} must apply no veto"
        );
        assert_eq!(
            parsed.unknown_policy_types,
            vec![FoodPolicies::HARD_VETO],
            "{subject:?}/{enabled} is malformed and must be reported"
        );
    }
    // Well-formed but disabled: known and not applied, so nothing is reported.
    let off = FoodPolicies::from_policies(&[veto("tofu", false)]);
    assert!(off.hard_vetoes.is_empty());
    assert!(off.unknown_policy_types.is_empty());
    // The guard tests matchability, not spelling: one alphanumeric token is enough, so a
    // subject that merely *contains* an unmatchable character is still a real veto.
    let on = FoodPolicies::from_policies(&[veto("🍝 pasta", true)]);
    assert_eq!(on.hard_vetoes, vec!["🍝 pasta".to_owned()]);
    assert!(on.unknown_policy_types.is_empty());
    // And the reported case reaches the user as an assumption, not silence.
    let mut s = base();
    s.policies = FoodPolicies::from_policies(&[veto(" ", true)]);
    let r = run(&s);
    assert!(r
        .assessment
        .assumptions
        .iter()
        .any(|a| a.as_str() == coverage::UNKNOWN_POLICY_TYPE));
}

/// The reviewed marker is the household's explicit word that its restriction list is
/// complete as stored (§10: reviewed absence is confirmed absence). A disabled marker is
/// well-formed and silently ignored, like a disabled veto: known, not applied.
#[test]
fn restrictions_reviewed_policy_sets_the_flag_and_is_never_unknown() {
    let mark = |enabled: bool| {
        Policy::new(
            PolicyId::new("p").unwrap(),
            HouseholdId::new("h").unwrap(),
            "food",
            FoodPolicies::RESTRICTIONS_REVIEWED,
            BTreeMap::new(),
            enabled,
            EvidenceSource::ExplicitUser,
        )
        .unwrap()
    };
    let on = FoodPolicies::from_policies(&[mark(true)]);
    assert!(on.restrictions_reviewed);
    assert!(
        on.unknown_policy_types.is_empty(),
        "a known type must not be reported as unknown"
    );
    let off = FoodPolicies::from_policies(&[mark(false)]);
    assert!(!off.restrictions_reviewed);
    assert!(off.unknown_policy_types.is_empty());
}

/// The marker line is emitted only when set, so every pre-existing snapshot's canonical
/// text — and every pinned fixture hash — stays byte-identical, while presence/absence of
/// the labeled line keeps the encoding injective.
#[test]
fn reviewed_marker_changes_canonical_text_and_hash_only_when_set() {
    let s = base();
    assert!(
        !s.canonical_text().contains("policy.restrictions_reviewed"),
        "unset marker must emit no line"
    );
    let mut v = s.clone();
    v.policies.restrictions_reviewed = true;
    assert!(v
        .canonical_text()
        .contains("policy.restrictions_reviewed=true"));
    assert_ne!(v.canonical_text(), s.canonical_text());
    assert_ne!(v.snapshot_hash(), s.snapshot_hash());
}

/// `PlanningSnapshot`'s fields are public, so a caller other than the loader can build one
/// with no days in it. That used to index an empty `dates()` out of bounds.
#[test]
fn a_zero_length_cycle_is_unresolved_not_a_panic() {
    let mut s = base();
    s.length_days = 0;
    let r = run(&s);
    assert_eq!(r.assessment.status, OutcomeStatus::Unresolved);
    assert!(r.slots.is_empty());
    assert_eq!(r.proposed.len(), 0);
    let anchor = crate::format_civil_date(s.anchor);
    assert_eq!(r.assessment.horizon.from, anchor);
    assert_eq!(r.assessment.horizon.to, anchor);
}

/// AC-4 copy review, mechanised: every sentence in the table and every emitted string.
#[test]
fn issue_text_table_never_claims_safety_exact_stock_or_personalisation() {
    let banned = ["safe", "exact", "personali", "guarantee"];
    assert_eq!(ISSUE_TEXT.len(), 10);
    for (code, text) in ISSUE_TEXT {
        assert!(!text.is_empty(), "{code}");
        let lower = text.to_lowercase();
        for b in banned {
            assert!(!lower.contains(b), "{code}: {text:?} contains {b:?}");
        }
        assert_eq!(issue_text(code), text);
    }
    assert_eq!(issue_text("NOT_A_CODE"), "");
    // A parameterised code resolves on its family, so the copy is no longer dropped by the
    // caller's non-empty filter — but the rule never invents copy for a family with no entry,
    // and `LOCK_HELD_OVER:RESTRICTION_CONFLICT:rules_v1` resolves on its *first* segment.
    assert_eq!(
        issue_text("LOCK_HELD_OVER:HARD_VETO"),
        issue_text("LOCK_HELD_OVER")
    );
    assert_eq!(
        issue_text("LOCK_HELD_OVER:RESTRICTION_CONFLICT:rules_v1"),
        issue_text("LOCK_HELD_OVER")
    );
    assert_eq!(issue_text("UNKNOWN:x"), "");
    assert_eq!(issue_text("REPEATED_SACRIFICE:m_1"), "");
    let mut s = base();
    s.restrictions = HouseholdRestrictions::new([Restriction::other("nightshades").unwrap()]);
    s.recipes[0].prep_minutes = None;
    s.preferences.clear();
    s.recipes.push(recipe("r-pb", "PB", Some(5), &["peanuts"]));
    s.policies.unknown_policy_types = vec!["food.x".to_owned()];
    let r = run(&s);
    let mut emitted: Vec<String> = r.assessment.unresolved_issues.clone();
    emitted.extend(r.attention.iter().flat_map(|a| a.options.iter().cloned()));
    emitted.push(r.canonical_text());
    assert!(!emitted.is_empty());
    for text in emitted {
        let lower = text.to_lowercase();
        for b in banned {
            assert!(!lower.contains(b), "{text:?} contains {b:?}");
        }
    }
}

#[test]
fn only_fallbacks_feasible_yields_needs_attention_and_plan_infeasible() {
    let mut s = base();
    s.policies.hard_vetoes = vec!["rice".to_owned(), "soup".to_owned(), "tofu".to_owned()];
    let r = run(&s);
    assert_eq!(r.assessment.status, OutcomeStatus::NeedsAttention);
    for slot in &r.slots {
        assert_eq!(slot.state, CoverageState::NeedsAttention);
        assert!(slot.reason_codes.contains(&PLAN_INFEASIBLE.to_owned()));
        // The two placeholder branches are exclusive: nothing was available to lose to, so
        // this is the `needs_attention` case and not the "chose a placeholder anyway" one.
        assert!(!slot
            .reason_codes
            .contains(&FALLBACK_PLACEHOLDER_CHOSEN.to_owned()));
        assert_eq!(
            slot.source,
            Some(CandidateSource::FrozenQuick),
            "the beam still places a fallback"
        );
    }
    let infeasible: Vec<&household_core::AttentionRequest> = r
        .attention
        .iter()
        .filter(|a| a.reason_codes.iter().any(|c| c.as_str() == PLAN_INFEASIBLE))
        .collect();
    assert_eq!(infeasible.len(), 2);
    assert_eq!(infeasible[0].urgency, Urgency::High);
    assert_eq!(
        infeasible[0].decision_benefit_band,
        household_core::Band::High
    );
    assert_eq!(
        infeasible[0].estimated_effort_band,
        household_core::Band::Low
    );
    assert_eq!(infeasible[0].options.len(), 2, "frozen/quick and open");
    assert_eq!(
        infeasible[1].options.len(),
        3,
        "day two also lists sourceless leftovers"
    );
    assert_eq!(infeasible[0].deadline.as_deref(), Some("2026-08-29"));
    // Leftovers with a real source is a meal: a locked recipe the day before feeds it.
    let mut s2 = s.clone();
    s2.existing = vec![meal(
        "l",
        "2026-08-29",
        MealSlot::Dinner,
        vec![rc("r-tofu")],
        true,
    )];
    let r2 = run(&s2);
    // The lock holds even though "tofu" is vetoed — but a hard check that was skipped rather
    // than passed cannot support a `Covered` claim for the cycle, so the override is surfaced
    // and the claim degrades. The slot itself keeps the lock.
    assert_eq!(r2.slots[0].state, CoverageState::LockedByUser);
    assert!(
        r2.assessment
            .reason_codes
            .iter()
            .any(|c| c.as_str() == "LOCK_HELD_OVER:HARD_VETO"),
        "{:?}",
        r2.assessment.reason_codes
    );
    assert_eq!(r2.slots[1].source, Some(CandidateSource::Leftovers));
    assert_eq!(r2.slots[1].state, CoverageState::Covered);
    assert_eq!(r2.assessment.status, OutcomeStatus::TentativelyCovered);
    assert!(
        r2.assessment
            .unresolved_issues
            .iter()
            .any(|t| t == issue_text("LOCK_HELD_OVER")),
        "the held-over hard check reaches the user: {:?}",
        r2.assessment.unresolved_issues
    );
    assert!(r
        .assessment
        .unresolved_issues
        .iter()
        .any(|t| t == issue_text(PLAN_INFEASIBLE)));
}

#[test]
fn attention_requests_carry_every_field() {
    let mut s = base();
    s.policies.hard_vetoes = vec!["rice".to_owned(), "soup".to_owned(), "tofu".to_owned()];
    s.preferences.clear();
    let r = run(&s);
    assert!(r.attention.len() >= 3);
    for a in &r.attention {
        assert!(a.id.starts_with("food:"), "{a:?}");
        assert_eq!(a.controller_id, CONTROLLER_ID);
        assert!(!a.options.is_empty(), "{a:?}");
        assert!(!a.reason_codes.is_empty(), "{a:?}");
        let _ = (
            a.urgency,
            a.decision_benefit_band,
            a.estimated_effort_band,
            &a.deadline,
        );
    }
    let ids: Vec<&str> = r.attention.iter().map(|a| a.id.as_str()).collect();
    let mut unique = ids.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        ids.len(),
        "ids are deterministic and distinct"
    );
    assert_eq!(ids[0], "food:infeasible:2026-08-29:dinner");
}

/// Expected-to-pass, labelled: pins that nothing under `planner/` reaches for IO, a clock,
/// a database or the network — the AC-5 grep, run by the test itself over the source tree.
#[test]
fn planner_source_has_no_io_imports() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/planner");
    let banned = [
        "std::fs",
        "std::net",
        "std::time",
        "rusqlite",
        "reqwest",
        "SystemTime",
        "Instant::now",
    ];
    let mut checked = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") && !path.ends_with("tests.rs") {
            let text = std::fs::read_to_string(&path).unwrap();
            for b in banned {
                assert!(!text.contains(b), "{} mentions {b}", path.display());
            }
            checked += 1;
        }
    }
    // 8 MVP-023 modules plus MVP-025's `invariant_tests.rs`; `fixtures/mod.rs` sits in a
    // subdirectory this non-recursive walk does not reach, and is cfg-gated out of the
    // shipped library anyway.
    assert_eq!(checked, 9);
}

#[test]
fn locked_and_open_slots_report_their_states() {
    let mut s = base();
    s.length_days = 3;
    s.existing = vec![
        meal(
            "l",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-tofu")],
            true,
        ),
        meal(
            "o",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::Open { note: None }],
            false,
        ),
    ];
    let r = run(&s);
    assert_eq!(r.slots[0].state, CoverageState::LockedByUser);
    assert_eq!(r.slots[1].state, CoverageState::IntentionallyOpen);
    assert_eq!(r.slots[2].state, CoverageState::Covered);
    assert_eq!(r.assessment.status, OutcomeStatus::Covered);
    assert_eq!(
        CoverageState::ALL.map(CoverageState::as_str),
        [
            "unresolved",
            "tentatively_covered",
            "covered",
            "locked_by_user",
            "intentionally_open",
            "needs_attention",
        ]
    );
}

#[test]
fn controller_assess_reports_the_existing_plan_without_planning() {
    let c = FoodController;
    // Nothing planned: unresolved, and no search ran (no rejections, no proposals here).
    let a = c.assess(&base()).unwrap();
    assert_eq!(a.status, OutcomeStatus::Unresolved);
    assert_eq!(a.controller_id, "food");
    assert_eq!(a.horizon.from, "2026-08-29");
    // Fully planned with known recipes: covered.
    let mut s = base();
    s.existing = vec![
        meal(
            "a",
            "2026-08-29",
            MealSlot::Dinner,
            vec![rc("r-tofu")],
            false,
        ),
        meal(
            "b",
            "2026-08-30",
            MealSlot::Dinner,
            vec![rc("r-rice")],
            true,
        ),
    ];
    assert_eq!(c.assess(&s).unwrap().status, OutcomeStatus::Covered);
    assert_eq!(assess_existing(&s), OutcomeStatus::Covered);
    // A planned meal Tier 0 now rejects: needs attention, with the code.
    s.recipes.push(recipe("r-pb", "PB", Some(5), &["peanuts"]));
    s.existing[0] = meal("a", "2026-08-29", MealSlot::Dinner, vec![rc("r-pb")], false);
    let bad = c.assess(&s).unwrap();
    assert_eq!(bad.status, OutcomeStatus::NeedsAttention);
    assert!(bad
        .reason_codes
        .iter()
        .any(|r| r.as_str() == "RESTRICTION_CONFLICT:rules_v1"));
    // Half planned: unresolved.
    s.existing.remove(0);
    assert_eq!(c.assess(&s).unwrap().status, OutcomeStatus::Unresolved);
}

#[test]
fn controller_propose_returns_the_apply_proposal() {
    let p = FoodController.propose(&base()).unwrap();
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].action_type, APPLY_PLAN);
    assert_eq!(p[0].id, "food:apply_plan:2026-08-29");
    assert_eq!(
        p[0].required_authority,
        household_core::RequiredAuthority::HouseholdMember
    );
    assert_eq!(
        p[0].reversibility,
        household_core::Reversibility::Reversible
    );
    assert_eq!(p[0].expected_benefit_band, household_core::Band::High);
    assert_eq!(p, run(&base()).proposals);
    assert!(!p[0].reason_codes.is_empty());
}
