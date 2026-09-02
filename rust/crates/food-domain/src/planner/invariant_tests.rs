//! MVP-025 property/invariant tests, quantified over the synthetic fixtures in
//! [`super::fixtures`]. Each test asserts an invariant across every fixture — never one
//! exact plan where a fixture admits several equally acceptable ones (card AC-3).

use std::collections::BTreeSet;

use super::candidates::STARTER_NOTE_PREFIX;
use super::fixtures::{self, FixtureSource, PlannerFixture};
use super::{cover_cycle, PlanningResult, SearchParams, PLANNER_ALGORITHM_VERSION};
use crate::restriction::assess;
use crate::{MealComponent, MealSlot, PlannedMeal, Restriction, Sentiment};

/// The (B, K) grid every invariant is checked at: degenerate, shipped, and wide-open —
/// 64 exceeds every fixture's candidate count, so the last cell is effectively uncut.
fn param_grid() -> [SearchParams; 3] {
    let p = |beam_width, candidates_per_slot| SearchParams {
        beam_width,
        candidates_per_slot,
        commitment_horizon_days: 2,
    };
    [p(1, 1), SearchParams::default(), p(64, 64)]
}

/// Whether this proposal is the fixture's own *locked* existing occurrence, unchanged. A
/// lock is the user's Tier-0 word: automation may never move it, so invariant 1 excludes it
/// (`tier0::filter` carries a failing check as `LOCK_HELD_OVER:*`, never as a selection).
fn is_locked_existing(f: &PlannerFixture, p: &super::ProposedMeal) -> bool {
    f.snapshot
        .existing_at(p.date, p.slot)
        .is_some_and(|m| m.locked() && m.components() == p.components)
}

/// The line names a proposed component puts on the table, resolved the way the planner
/// resolves them: recipes by id, starter stubs by slug. An unresolvable id or slug panics
/// rather than yielding an empty name vector — `assess` over no names reports no conflicts,
/// so a silent miss would pass invariant 1 without ever checking it.
fn line_names_of(f: &PlannerFixture, component: &MealComponent) -> Vec<String> {
    match component {
        MealComponent::Recipe { recipe_id, .. } => f
            .snapshot
            .recipes
            .iter()
            .find(|r| &r.id == recipe_id)
            .unwrap_or_else(|| {
                panic!(
                    "{}: proposal names unknown recipe {}",
                    f.name,
                    recipe_id.as_str()
                )
            })
            .line_names
            .clone(),
        // Only a note carrying the prefix is a starter stub; any other freeform note names
        // no dish, and is not a resolution failure.
        MealComponent::Freeform { note } => match note.strip_prefix(STARTER_NOTE_PREFIX) {
            Some(slug) => f
                .snapshot
                .starter
                .iter()
                .find(|s| s.slug == slug)
                .unwrap_or_else(|| {
                    panic!("{}: proposal names unknown starter slug {slug:?}", f.name)
                })
                .lines
                .iter()
                .map(|l| l.name().to_owned())
                .collect(),
            None => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// The recipe ids a stored occurrence names, for shape assertions over history/existing.
fn recipe_ids_of(meal: &PlannedMeal) -> Vec<String> {
    meal.components()
        .iter()
        .filter_map(|c| match c {
            MealComponent::Recipe { recipe_id, .. } => Some(recipe_id.as_str().to_owned()),
            _ => None,
        })
        .collect()
}

fn preference_count(f: &PlannerFixture) -> usize {
    f.snapshot
        .preferences
        .iter()
        .map(|(_, p)| p.preferences().len())
        .sum()
}

/// AC-1: the fixture set is deterministic — two builds are equal value-for-value, hash the
/// same, and every name is unique. (Expected-to-pass once the module exists: this pins the
/// no-RNG/no-clock construction contract rather than a defect.)
#[test]
fn fixtures_are_deterministic() {
    let a = fixtures::all();
    let b = fixtures::all();
    assert!(!a.is_empty(), "fixture set must not be empty");
    assert_eq!(a.len(), b.len());
    let mut names = BTreeSet::new();
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.name, y.name, "stable order");
        assert_eq!(x.snapshot, y.snapshot, "{}", x.name);
        assert_eq!(
            x.snapshot.snapshot_hash(),
            y.snapshot.snapshot_hash(),
            "{}",
            x.name
        );
        assert_eq!(x.recipes, y.recipes, "{}", x.name);
        assert!(names.insert(x.name), "duplicate fixture name {}", x.name);
    }
}

/// AC-1 determinism evidence: each fixture's snapshot hash is pinned as a literal, so any
/// edit to a fixture (or to the canonical-text encoder) fails here first. Update the pinned
/// value deliberately when a fixture changes — update, don't delete (fixtures README).
/// (Expected-to-pass once pinned: this documents intentional values, not a defect.)
#[test]
fn fixture_hashes_are_pinned() {
    let actual: Vec<(&str, String)> = fixtures::all()
        .iter()
        .map(|f| (f.name, f.snapshot.snapshot_hash()))
        .collect();
    let expected: [(&str, &str); 11] = [
        ("cold_start", "74544bb748c088c5"),
        ("conservative_household", "9722118888232ce7"),
        ("high_variety_household", "bd273258d5f7e20d"),
        ("multiple_strong_dislikes", "7c59d93cbdeaf93f"),
        ("busy_week", "44d54063a57a4b4c"),
        ("many_locked_meals", "1a82a735f5460c43"),
        ("sparse_pantry", "f0e3e4a29f7a6e92"),
        ("restrictions_set_and_skipped", "000fbc9237476e15"),
        ("leftovers_fallback_heavy", "2784f680c6ebdce7"),
        ("repetitive_meal_library", "3f194d0f6835eb99"),
        ("intentionally_open_nights", "955f887cb3a34622"),
    ];
    assert_eq!(
        actual
            .iter()
            .map(|(n, h)| (*n, h.as_str()))
            .collect::<Vec<_>>(),
        expected.to_vec(),
        "pinned fixture hashes drifted; recompute deliberately"
    );
}

/// AC-1: the co-located README documents every fixture by name, so the table cannot drift
/// from `all()` silently.
#[test]
fn readme_lists_every_fixture() {
    let readme = include_str!("fixtures/README.md");
    for f in fixtures::all() {
        assert!(
            readme.contains(f.name),
            "fixtures/README.md must document {}",
            f.name
        );
    }
}

/// AC-1: every fixture structurally exhibits the stressor its table row declares. Written
/// fail-first: red while a fixture is missing from `all()`.
#[test]
fn fixture_shapes_hold() {
    // §18 №1 — cold start: nothing configured, only starter content available.
    let f = fixtures::by_name("cold_start");
    assert_eq!(f.source, FixtureSource::Prd18);
    assert!(f.recipes.is_empty() && f.snapshot.recipes.is_empty());
    assert!(!f.snapshot.starter.is_empty());
    assert!(f.snapshot.preferences.is_empty());
    assert!(f.snapshot.restrictions.restrictions().is_empty());

    // §18 №2 — conservative household: small library, narrow likes, repeats tolerated.
    let f = fixtures::by_name("conservative_household");
    assert_eq!(f.source, FixtureSource::Prd18);
    assert!((4..=8).contains(&f.recipes.len()), "small library");
    let prefs = preference_count(&f);
    assert!((1..=4).contains(&prefs), "narrow likes, got {prefs}");
    let history_ids: Vec<String> = f.snapshot.history.iter().flat_map(recipe_ids_of).collect();
    let distinct: BTreeSet<&String> = history_ids.iter().collect();
    assert!(
        !history_ids.is_empty() && distinct.len() < history_ids.len(),
        "repeat-tolerant history repeats at least one recipe"
    );

    // §18 №3 — high variety: a big library and a second slot stress K and the beam.
    let f = fixtures::by_name("high_variety_household");
    assert_eq!(f.source, FixtureSource::Prd18);
    assert!(
        (40..=60).contains(&f.recipes.len()),
        "40–60 recipes, got {}",
        f.recipes.len()
    );
    assert!(f.snapshot.scope.contains(MealSlot::Lunch));
    assert!(f.snapshot.scope.contains(MealSlot::Dinner));
    assert!(preference_count(&f) >= 10, "many likes");

    // §18 №4 — multiple strong dislikes across 2–3 members hit most of the library.
    let f = fixtures::by_name("multiple_strong_dislikes");
    assert_eq!(f.source, FixtureSource::Prd18);
    assert!((2..=3).contains(&f.snapshot.members.len()));
    let disliked_subjects: Vec<&str> = f
        .snapshot
        .preferences
        .iter()
        .flat_map(|(_, p)| p.preferences())
        .filter(|p| p.sentiment() == Sentiment::Dislike)
        .map(|p| p.subject())
        .collect();
    assert!(!disliked_subjects.is_empty());
    let hit = f
        .snapshot
        .recipes
        .iter()
        .filter(|r| {
            disliked_subjects
                .iter()
                .any(|s| r.line_names.iter().any(|l| l == s))
        })
        .count();
    assert!(
        hit * 2 > f.snapshot.recipes.len(),
        "most recipes disliked by at least one member: {hit}/{}",
        f.snapshot.recipes.len()
    );

    // §18 №5 — busy week: a slot window below some preps, plus unknown preps.
    let f = fixtures::by_name("busy_week");
    assert_eq!(f.source, FixtureSource::Prd18);
    let window = *f
        .snapshot
        .policies
        .slot_windows
        .get(&MealSlot::Dinner)
        .expect("busy_week sets a dinner window");
    assert!(
        f.snapshot
            .recipes
            .iter()
            .any(|r| r.prep_minutes.is_some_and(|p| p > window)),
        "some prep exceeds the window"
    );
    assert!(
        f.snapshot.recipes.iter().any(|r| r.prep_minutes.is_none()),
        "some prep is unknown"
    );

    // §18 №6 — many locked meals, one of them held over a restriction conflict.
    let f = fixtures::by_name("many_locked_meals");
    assert_eq!(f.source, FixtureSource::Prd18);
    let locked: Vec<&PlannedMeal> = f.snapshot.existing.iter().filter(|m| m.locked()).collect();
    assert!(
        locked.len() >= 5,
        "≥5 of 7 slots locked, got {}",
        locked.len()
    );
    assert!(!f.snapshot.restrictions.restrictions().is_empty());
    let conflicting = locked.iter().any(|m| {
        recipe_ids_of(m).iter().any(|id| {
            f.snapshot
                .recipes
                .iter()
                .find(|r| r.id.as_str() == id)
                .is_some_and(|r| r.line_names.iter().any(|l| l == "peanuts"))
        })
    });
    assert!(
        conflicting,
        "one lock conflicts with the peanut restriction"
    );

    // §18 №7 — sparse pantry: shared ingredients everywhere, nothing marked.
    let f = fixtures::by_name("sparse_pantry");
    assert_eq!(f.source, FixtureSource::Prd18);
    assert!(f.snapshot.pantry_marked.is_empty());
    let shared = f
        .snapshot
        .recipes
        .first()
        .map(|first| {
            first.ingredient_refs.iter().any(|shared_ref| {
                f.snapshot
                    .recipes
                    .iter()
                    .filter(|r| r.ingredient_refs.contains(shared_ref))
                    .count()
                    >= 3
            })
        })
        .unwrap_or(false);
    assert!(shared, "some ingredient is shared by ≥3 recipes");

    // §18 №8 — restrictions set (one Known + one Other) and a paired skipped variant.
    let f = fixtures::by_name("restrictions_set_and_skipped");
    assert_eq!(f.source, FixtureSource::Prd18);
    let restrictions = f.snapshot.restrictions.restrictions();
    assert_eq!(restrictions.len(), 2);
    assert!(restrictions
        .iter()
        .any(|r| matches!(r, Restriction::Known(_))));
    assert!(restrictions
        .iter()
        .any(|r| matches!(r, Restriction::Other(_))));
    let skipped = fixtures::restrictions_skipped_variant(&f);
    assert!(skipped.restrictions.restrictions().is_empty());
    assert_ne!(
        skipped.snapshot_hash(),
        f.snapshot.snapshot_hash(),
        "the skipped variant is a different input"
    );

    // §18 №9 — leftovers/fallback-heavy: big batches, sourced history, no dining out.
    let f = fixtures::by_name("leftovers_fallback_heavy");
    assert_eq!(f.source, FixtureSource::Prd18);
    assert!(!f.snapshot.policies.dining_out_enabled);
    let members = f.snapshot.members.len() as u32;
    assert!(
        f.snapshot
            .recipes
            .iter()
            .any(|r| r.servings.is_some_and(|s| s > members)),
        "some recipe plausibly leaves leftovers"
    );
    let day_before = f.snapshot.anchor.yesterday().unwrap();
    assert!(
        f.snapshot.history.iter().any(|m| m.date() == day_before),
        "history feeds the first cycle day"
    );

    // Pivot №10 — repetitive meal library: near-duplicate recipes stress tie-breaking.
    let f = fixtures::by_name("repetitive_meal_library");
    assert_eq!(f.source, FixtureSource::PivotPrompt);
    let duplicates = f
        .snapshot
        .recipes
        .iter()
        .filter(|r| {
            f.snapshot
                .recipes
                .iter()
                .filter(|other| {
                    other.line_names == r.line_names && other.prep_minutes == r.prep_minutes
                })
                .count()
                >= 4
        })
        .count();
    assert!(
        duplicates >= 4,
        "at least four recipes share identical lines and prep, got {duplicates}"
    );

    // Pivot №11 — intentionally open nights: two explicit `Open` decisions, rest empty.
    let f = fixtures::by_name("intentionally_open_nights");
    assert_eq!(f.source, FixtureSource::PivotPrompt);
    let open = f
        .snapshot
        .existing
        .iter()
        .filter(|m| m.components().iter().any(MealComponent::is_open))
        .count();
    assert_eq!(open, 2, "exactly two intentionally open slots");
    assert_eq!(f.snapshot.existing.len(), 2, "every other slot is empty");
    assert!(!f.recipes.is_empty());

    assert_eq!(
        fixtures::all().len(),
        11,
        "the two source lists union to eleven"
    );

    // §9.3 lists dining out as a candidate source and `candidates.rs` gates it on this policy,
    // so the corpus must exercise both sides of the flag rather than pinning the struct default.
    let dining_out: Vec<&str> = fixtures::all()
        .iter()
        .filter(|f| f.snapshot.policies.dining_out_enabled)
        .map(|f| f.name)
        .collect();
    assert_eq!(
        dining_out,
        ["busy_week"],
        "exactly one fixture enables dining out"
    );
}

/// Invariant 1 (AC-2): no proposed component names a dish that conflicts with a configured
/// restriction — at any beam width. The one documented exception is a *locked* existing
/// occurrence, which Tier 0 carries as `LOCK_HELD_OVER` rather than selecting it.
/// (Expected-to-pass: the red demonstration saboteurs `tier0.rs`, see fixtures/README.md.)
#[test]
fn invariant_1_a_known_hard_restriction_is_never_selected_on_any_fixture() {
    let mut components_checked = 0usize;
    for f in fixtures::all() {
        for params in param_grid() {
            let result = cover_cycle(&f.snapshot, &params);
            assert!(!result.proposed.is_empty(), "{}: nothing proposed", f.name);
            for p in &result.proposed {
                if is_locked_existing(&f, p) {
                    continue;
                }
                for component in &p.components {
                    let names = line_names_of(&f, component);
                    let assessment =
                        assess(names.iter().map(String::as_str), &f.snapshot.restrictions);
                    assert!(
                        assessment.conflicts.is_empty(),
                        "{}: proposed {:?} on {} {} conflicts with a configured restriction",
                        f.name,
                        component,
                        p.date,
                        p.slot.as_str(),
                    );
                    components_checked += 1;
                }
            }
        }
    }
    assert!(components_checked > 0, "the loop must check something");
}

/// Invariant 2 (AC-2): a locked meal never moves — every locked in-scope occurrence comes
/// back in the proposal byte-identical, at every beam width.
/// (Expected-to-pass: the red demonstration deletes tier0's LOCK_CONFLICT rejection.)
#[test]
fn invariant_2_a_locked_meal_never_moves_on_any_fixture() {
    let mut locked_checked = 0usize;
    for f in fixtures::all() {
        for params in param_grid() {
            let result = cover_cycle(&f.snapshot, &params);
            for locked in f.snapshot.existing.iter().filter(|m| m.locked()) {
                if !f.snapshot.scope.contains(locked.slot()) {
                    continue;
                }
                let proposed = result
                    .proposed
                    .iter()
                    .find(|p| p.date == locked.date() && p.slot == locked.slot())
                    .unwrap_or_else(|| {
                        panic!(
                            "{}: locked slot {} {} missing from the proposal",
                            f.name,
                            locked.date(),
                            locked.slot().as_str()
                        )
                    });
                assert_eq!(
                    proposed.components,
                    locked.components(),
                    "{}: locked meal moved at {} {}",
                    f.name,
                    locked.date(),
                    locked.slot().as_str(),
                );
                locked_checked += 1;
            }
        }
    }
    assert!(locked_checked > 0, "many_locked_meals must exercise this");
}

/// Invariant 3 (AC-2): the same snapshot, params and algorithm version yield the identical
/// result — full struct equality and byte-identical ledger payload across repeated runs.
/// Input-permutation robustness stays with `tests.rs`'s
/// `beam_is_deterministic_across_repeated_runs_and_input_permutations`: snapshot
/// collections are canonical-order-by-contract, so a permuted input is a different input.
/// (Expected-to-pass: the red demonstration routes the tie-break through process state.)
#[test]
fn invariant_3_same_input_and_version_yield_identical_results_on_every_fixture() {
    for f in fixtures::all() {
        for params in param_grid() {
            let first: PlanningResult = cover_cycle(&f.snapshot, &params);
            let second = cover_cycle(&f.snapshot, &params);
            assert_eq!(first, second, "{}: two runs diverged", f.name);
            assert_eq!(
                first.canonical_text(),
                second.canonical_text(),
                "{}: ledger payload bytes diverged",
                f.name
            );
            assert_eq!(first.algorithm_version, PLANNER_ALGORITHM_VERSION);
            assert_eq!(first.snapshot_hash, f.snapshot.snapshot_hash());
        }
    }
}

/// The invariant-4 perturbation catalog: ways to *add* a hard constraint to a snapshot.
/// Locks are excluded by design: `tier0.rs` carries a locked candidate that fails a check
/// as a `LOCK_HELD_OVER:*` assumption — the lock is the user's own Tier-0 word — so locking
/// a slot can make an otherwise-rejected candidate feasible, and that is correct behavior,
/// not a monotonicity violation.
fn hard_constraint_perturbations(
    f: &PlannerFixture,
) -> Vec<(String, crate::planner::PlanningSnapshot)> {
    use crate::HouseholdRestrictions;
    let mut out = Vec::new();
    let all_line_names: BTreeSet<&str> = f
        .snapshot
        .recipes
        .iter()
        .flat_map(|r| r.line_names.iter().map(String::as_str))
        .chain(
            f.snapshot
                .starter
                .iter()
                .flat_map(|s| s.lines.iter().map(|l| l.name())),
        )
        .collect();
    // (a) each Known kind that actually hits at least one line somewhere in the fixture.
    for kind in crate::RestrictionKind::ALL {
        let restrictions = HouseholdRestrictions::new([Restriction::Known(kind)]);
        let hits = assess(all_line_names.iter().copied(), &restrictions);
        if hits.conflicts.is_empty() {
            continue;
        }
        let mut s = f.snapshot.clone();
        let mut merged: Vec<Restriction> = s.restrictions.restrictions().to_vec();
        merged.push(Restriction::Known(kind));
        s.restrictions = HouseholdRestrictions::new(merged);
        out.push((format!("known:{}", kind.as_str()), s));
    }
    // (b) an Other restriction worded as an existing line name (a wording-only match: never
    // a rejection, so the subset here is expected to be improper — that is the point).
    if let Some(name) = all_line_names.iter().next() {
        let mut s = f.snapshot.clone();
        let mut merged: Vec<Restriction> = s.restrictions.restrictions().to_vec();
        merged.push(Restriction::other(*name).expect("line names are non-blank"));
        s.restrictions = HouseholdRestrictions::new(merged);
        out.push((format!("other:{name}"), s));
    }
    // (c) a hard veto per distinct line token.
    for name in &all_line_names {
        let mut s = f.snapshot.clone();
        s.policies.hard_vetoes.push((*name).to_owned());
        s.policies.hard_vetoes.sort_unstable();
        out.push((format!("veto:{name}"), s));
    }
    // (d) tighten every in-scope slot's window below the longest known prep.
    let max_prep = f
        .snapshot
        .recipes
        .iter()
        .filter_map(|r| r.prep_minutes)
        .chain(f.snapshot.starter.iter().filter_map(|s| s.prep_minutes))
        .max();
    if let Some(max_prep) = max_prep {
        let mut s = f.snapshot.clone();
        let mut tightened_any = false;
        for slot in f.snapshot.scope.slots() {
            // Strictly below both the longest known prep and any existing window — a
            // perturbation that widened a window would not be adding a constraint.
            let current = f.snapshot.policies.slot_windows.get(slot).copied();
            let cap = current.unwrap_or(max_prep).min(max_prep);
            if cap > 1 {
                s.policies.slot_windows.insert(*slot, cap - 1);
                tightened_any = true;
            }
        }
        if tightened_any {
            out.push(("window:tightened".to_owned(), s));
        }
    }
    out
}

/// Invariant 4 (AC-2): adding a hard constraint never makes a previously infeasible
/// candidate feasible. The candidate *universe* is untouched (generation reads preferences,
/// history and the dining-out policy — never restrictions, vetoes or slot windows), and per
/// (date, slot) the feasible set after the perturbation is a subset of the one before.
/// Corollary at (1, 1) and (8, 12): no proposed component matches a candidate the same run
/// rejected for its slot.
/// (Expected-to-pass: the red demonstration gates tier0's restriction check on the veto
/// list being empty — a constraint-interaction bug this catalog is built to catch.)
#[test]
fn invariant_4_adding_a_hard_constraint_never_makes_an_infeasible_candidate_feasible() {
    use crate::planner::snapshot::components_text;
    use crate::planner::{candidates, tier0};
    let mut perturbations_checked = 0usize;
    for f in fixtures::all() {
        let base_slots = candidates::generate(&f.snapshot);
        for (label, perturbed) in hard_constraint_perturbations(&f) {
            let perturbed_slots = candidates::generate(&perturbed);
            assert_eq!(
                base_slots, perturbed_slots,
                "{}/{label}: the candidate universe must not change",
                f.name
            );
            for slot in &base_slots {
                let (before, _) = tier0::filter(&f.snapshot, slot);
                let (after, _) = tier0::filter(&perturbed, slot);
                let before_texts: BTreeSet<String> =
                    before.iter().map(|fe| fe.candidate.text()).collect();
                for fe in &after {
                    assert!(
                        before_texts.contains(&fe.candidate.text()),
                        "{}/{label}: {} became feasible under a tighter constraint",
                        f.name,
                        fe.candidate.text(),
                    );
                }
            }
            for params in [
                SearchParams {
                    beam_width: 1,
                    candidates_per_slot: 1,
                    commitment_horizon_days: 2,
                },
                SearchParams::default(),
            ] {
                let result = cover_cycle(&perturbed, &params);
                for p in &result.proposed {
                    let text = components_text(&p.components);
                    // `LOCK_CONFLICT` is excluded: it marks the lock holding the slot, not
                    // a constraint infeasibility, and it rejects even a same-dish duplicate
                    // of the locked occurrence — whose components text matches the (kept)
                    // locked proposal.
                    assert!(
                        !result.rejections.iter().any(|r| {
                            r.code != tier0::LOCK_CONFLICT
                                && r.date == p.date
                                && r.slot == p.slot
                                && r.candidate_text.ends_with(&format!(" {text}"))
                        }),
                        "{}/{label}: proposal at {} {} matches a rejected candidate",
                        f.name,
                        p.date,
                        p.slot.as_str(),
                    );
                }
            }
            perturbations_checked += 1;
        }
    }
    assert!(perturbations_checked > 0, "the catalog must not be empty");
}

/// Invariant 5 (AC-2): shopping quantities never go negative. Negativity is unrepresentable
/// (`Rational` is `u32/u32` and the derivation never subtracts), so the honest invariant
/// guards the boundary that keeps it so: (i) every emitted quantity is well-formed —
/// positive numerator and denominator, ordered ranges, contribution counts intact; and
/// (ii) pantry marks never subtract — marking every referenced ingredient changes line
/// *status* only, with quantities, units and contributions byte-identical.
/// (Expected-to-pass: the red demonstration makes the pantry branch halve the quantity —
/// a naive "reduce by what's at home" — and (ii) goes red.)
#[test]
fn invariant_5_shopping_quantities_are_well_formed_and_pantry_never_subtracts() {
    use crate::{derive_shopping_list, LineStatus, Quantity, ShoppingLine, ShoppingList};

    fn assert_positive(r: crate::Rational, context: &str) {
        assert!(r.numer() >= 1, "{context}: zero numerator");
        assert!(r.denom() >= 1, "{context}: zero denominator");
    }

    fn assert_well_formed(list: &ShoppingList, context: &str) {
        let mut contributions = 0usize;
        for group in &list.groups {
            for line in &group.lines {
                contributions += line.contributions.len();
                assert!(
                    !line.contributions.is_empty(),
                    "{context}: a line with no contribution"
                );
                match line.quantity {
                    Quantity::Unknown => {}
                    Quantity::Exact(r) => assert_positive(r, context),
                    Quantity::Range(range) => {
                        assert_positive(range.min(), context);
                        assert_positive(range.max(), context);
                        assert!(range.min() <= range.max(), "{context}: inverted range");
                    }
                }
            }
        }
        assert_eq!(
            list.contribution_count as usize, contributions,
            "{context}: contribution count drifted"
        );
    }

    /// Everything but the status, so the two derivations compare byte-for-byte on it.
    fn without_status(line: &ShoppingLine) -> ShoppingLine {
        let mut c = line.clone();
        c.status = LineStatus::Needed;
        c
    }

    let mut lines_seen = 0usize;
    let mut marked_seen = 0usize;
    for f in fixtures::all() {
        let result = cover_cycle(&f.snapshot, &SearchParams::default());
        let input = fixtures::shopping_input_of(&f, &result);
        let unmarked = derive_shopping_list(&input);
        assert_well_formed(&unmarked, f.name);

        let mut all_marked_input = input.clone();
        all_marked_input.pantry_marked = input
            .identities
            .iter()
            .map(|(ingredient, _)| ingredient.clone())
            .collect();
        let marked = derive_shopping_list(&all_marked_input);
        assert_well_formed(&marked, f.name);

        assert_eq!(unmarked.from, marked.from);
        assert_eq!(unmarked.to, marked.to);
        assert_eq!(unmarked.non_recipe_components, marked.non_recipe_components);
        assert_eq!(unmarked.contribution_count, marked.contribution_count);
        assert_eq!(unmarked.groups.len(), marked.groups.len(), "{}", f.name);
        for (before, after) in unmarked.groups.iter().zip(&marked.groups) {
            assert_eq!(before.category, after.category, "{}", f.name);
            assert_eq!(before.lines.len(), after.lines.len(), "{}", f.name);
            for (b, a) in before.lines.iter().zip(&after.lines) {
                lines_seen += 1;
                assert_eq!(
                    without_status(b),
                    without_status(a),
                    "{}: marking the pantry changed more than the status",
                    f.name
                );
                if a.status == LineStatus::OmittedPantryMarked {
                    marked_seen += 1;
                    assert!(
                        a.ingredient.is_some(),
                        "{}: only a resolved identity can be pantry-marked",
                        f.name
                    );
                }
            }
        }
    }
    assert!(lines_seen > 0, "some fixture must emit shopping lines");
    // Without this floor (ii) is vacuous: if `derive_shopping_list` ever stopped honouring
    // `pantry_marked`, `marked` would be bit-identical to `unmarked`, every equality above
    // would pass, and the status guard would never execute. The storage sibling carries the
    // same floor (`kimatta-storage/src/controller.rs`, "the two marked identities flip").
    assert!(
        marked_seen > 0,
        "marking every identity must flip at least one line to OmittedPantryMarked"
    );
}
