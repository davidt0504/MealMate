//! OPT-007 draft contract tests (§11 H1, H2, H3, H5, M3 and first-run receipts) at the
//! application seam. Each test states the legacy behaviour it would fail against where that is
//! the point of it.

use std::collections::{BTreeMap, BTreeSet};

use food_domain::planner::draft::{SlotOrigin, SlotOutcome};
use food_domain::planner::{cover_cycle as plan, FoodPolicies, SearchParams};
use household_core::{EvidenceSource, Household, HouseholdMember, MemberId, Policy, PolicyId};
use kimatta_storage::{
    archive_recipe, insert_household, list_ledger_entries, list_planned_meals, load_draft_in, open,
    parse_civil_date, save_planned_meal, save_planning_cycle, save_policy, save_recipe,
    set_planned_meal_lock, shipped_starter_content, IngredientLine, MealScope, PlanningCycle,
    ProvenanceKind, Quantity, Rational, Recipe, RecipeId, RecipeProvenance, Unit, UnitKind,
};

use super::*;

fn hid(id: &str) -> HouseholdId {
    HouseholdId::new(id).unwrap()
}

fn d(s: &str) -> CivilDate {
    parse_civil_date(s).unwrap()
}

const MON: &str = "2026-09-28";
const TUE: &str = "2026-09-29";
const WED: &str = "2026-09-30";

fn recipe(household: &str, id: &str, title: &str, instructions: &str, cups: u32) -> Recipe {
    Recipe::new(
        RecipeId::new(id).unwrap(),
        hid(household),
        title,
        // One serving for one member: no dish leaves leftovers, so every suggestion is a dish.
        Some(1),
        Some(15),
        instructions,
        vec![IngredientLine::new(
            format!("{cups} cup rice"),
            "rice",
            None,
            Quantity::Exact(Rational::new(cups, 1).unwrap()),
            Unit::Known(UnitKind::Cup),
            None,
            false,
        )
        .unwrap()],
        RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
    )
    .unwrap()
}

/// A three-dinner window from Monday with five household recipes and the starter roster
/// dismissed, so every candidate is one of ours.
fn seeded(id: &str) -> Connection {
    let mut conn = open(":memory:").unwrap();
    seed_into(&mut conn, id);
    conn
}

fn seed_into(conn: &mut Connection, id: &str) {
    insert_household(
        conn,
        &Household {
            id: hid(id),
            name: None,
        },
        &[HouseholdMember {
            id: MemberId::new(format!("m-{id}")).unwrap(),
            household_id: hid(id),
            display_name: "Me".to_owned(),
        }],
    )
    .unwrap();
    save_planning_cycle(
        conn,
        &PlanningCycle::new(hid(id), d(MON), 3, MealScope::dinner_only()).unwrap(),
    )
    .unwrap();
    for (rid, title) in [
        ("a", "Alpha bowl"),
        ("b", "Bravo stew"),
        ("c", "Charlie tacos"),
        ("d", "Delta curry"),
        ("e", "Echo salad"),
    ] {
        save_recipe(conn, &recipe(id, &format!("{id}-{rid}"), title, "Cook.", 1)).unwrap();
    }
    for entry in shipped_starter_content().unwrap().recipes {
        let r = Recipe::new(
            RecipeId::new(format!("dismissed-{id}-{}", entry.slug)).unwrap(),
            hid(id),
            entry.title.clone(),
            None,
            None,
            "",
            vec![],
            RecipeProvenance::with_rights(
                ProvenanceKind::Starter,
                None,
                None,
                None,
                None,
                Some(entry.slug.clone()),
            )
            .unwrap(),
        )
        .unwrap();
        save_recipe(conn, &r).unwrap();
        archive_recipe(conn, &hid(id), r.id(), d(MON)).unwrap();
    }
}

fn ctx_on(today: &str) -> DraftContext {
    DraftContext {
        household_id: hid("h"),
        today: d(today),
        offset_cycles: 0,
        edits_enabled: true,
    }
}

fn ctx() -> DraftContext {
    ctx_on(MON)
}

/// Ids are unique across the whole database (ledger primary key), so every minter draws from
/// one process-wide counter whatever its prefix.
fn minter(prefix: &'static str) -> impl FnMut() -> String {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    move || {
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("{prefix}-{n}")
    }
}

fn env(view: &DraftView, request: &str) -> DraftEnvelope {
    env_on(view, request, MON)
}

fn env_on(view: &DraftView, request: &str, today: &str) -> DraftEnvelope {
    DraftEnvelope {
        context: ctx_on(today),
        draft_id: view.draft_id.clone(),
        expected_revision: view.revision,
        request_id: request.to_owned(),
    }
}

fn rc(id: &str) -> MealComponent {
    MealComponent::recipe(RecipeId::new(id).unwrap(), None)
}

fn saved(conn: &Connection) -> Vec<PlannedMeal> {
    list_planned_meals(conn, &hid("h"), d(MON), d(WED)).unwrap()
}

fn save_meal(conn: &mut Connection, id: &str, date: &str, recipe: &str, locked: bool) {
    let meal = PlannedMeal::new(
        PlannedMealId::new(id).unwrap(),
        hid("h"),
        d(date),
        MealSlot::Dinner,
        vec![rc(recipe)],
        false,
    )
    .unwrap();
    save_planned_meal(conn, &meal, WriteSource::User).unwrap();
    if locked {
        set_planned_meal_lock(conn, &hid("h"), meal.id(), true, WriteSource::User).unwrap();
    }
}

fn ledger_len(conn: &Connection) -> usize {
    list_ledger_entries(conn, &hid("h")).unwrap().len()
}

fn slot<'v>(view: &'v DraftView, date: &str) -> &'v DraftSlotView {
    view.slots.iter().find(|s| s.date == d(date)).unwrap()
}

fn choose(date: &str, recipe: &str, explicit_replace: bool) -> DraftAction {
    DraftAction::Choose {
        date: d(date),
        slot: MealSlot::Dinner,
        components: vec![rc(recipe)],
        explicit_replace,
    }
}

fn lock(date: &str, locked: bool) -> DraftAction {
    DraftAction::SetCommitment {
        date: d(date),
        slot: MealSlot::Dinner,
        locked,
    }
}

// --- open / resume ----------------------------------------------------------------------------

#[test]
fn open_shows_saved_meals_fills_empty_future_slots_and_resumes_without_searching() {
    let mut conn = seeded("h");
    save_meal(&mut conn, "pm-mon", MON, "h-a", false);
    let first = open_draft(&mut conn, &ctx(), &mut minter("o1")).unwrap();
    assert_eq!(slot(&first, MON).origin, SlotOrigin::Saved);
    assert_eq!(slot(&first, MON).components, vec![rc("h-a")]);
    for day in [TUE, WED] {
        assert_eq!(slot(&first, day).origin, SlotOrigin::Suggested);
        assert!(!slot(&first, day).components.is_empty());
    }
    assert!(first.pending_changes);
    // Opening wrote no meal.
    assert_eq!(saved(&conn).len(), 1);
    let rows = ledger_len(&conn);
    let again = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_eq!(again.draft_id, first.draft_id);
    assert_eq!(again.revision, first.revision);
    assert_eq!(again.slots, first.slots);
    assert_eq!(
        ledger_len(&conn),
        rows,
        "resume performs no search and records nothing"
    );
}

// --- H1: draft isolation -----------------------------------------------------------------------

/// Legacy `record_decision(Swap)` wrote the saved row immediately; a draft pick does not.
#[test]
fn draft_edits_leave_saved_meals_untouched_until_accept() {
    let mut conn = seeded("h");
    save_meal(&mut conn, "pm-mon", MON, "h-a", false);
    let before = saved(&conn);
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &choose(TUE, "h-e", false),
        &mut minter("m1"),
    )
    .unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r2"),
        &lock(WED, true),
        &mut minter("m2"),
    )
    .unwrap();
    assert_eq!(saved(&conn), before);
    assert!(slot(&view, TUE).committed && slot(&view, TUE).pending);
    let wed = slot(&view, WED).components.clone();
    let accepted = accept_draft(&mut conn, &env(&view, "r3"), &mut minter("a")).unwrap();
    assert_eq!(accepted.state, DraftState::Accepted);
    assert!(accepted.receipt.is_some());
    let after = saved(&conn);
    assert_eq!(after.len(), 3);
    assert_eq!(
        after[0], before[0],
        "the untouched saved meal keeps its row"
    );
    assert_eq!(after[1].components(), &[rc("h-e")]);
    assert!(after[1].locked());
    assert_eq!(after[2].components(), wed.as_slice());
    assert!(after[2].locked());
}

#[test]
fn discard_restores_the_saved_view_and_closes_the_draft() {
    let mut conn = seeded("h");
    save_meal(&mut conn, "pm-mon", MON, "h-a", false);
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &choose(MON, "h-b", false),
        &mut minter("m"),
    )
    .unwrap();
    let closed = mutate_draft(
        &mut conn,
        &env(&view, "r2"),
        &DraftAction::Discard,
        &mut minter("x"),
    )
    .unwrap();
    assert_eq!(closed.state, DraftState::Discarded);
    assert_eq!(slot(&closed, MON).components, vec![rc("h-a")]);
    assert!(!closed.pending_changes);
    assert_eq!(saved(&conn).len(), 1);
    let err = mutate_draft(
        &mut conn,
        &env(&view, "r3"),
        &lock(MON, true),
        &mut minter("y"),
    )
    .unwrap_err();
    assert!(matches!(err, ApplicationError::DraftClosed(_)), "{err:?}");
    // A new open starts a fresh draft from saved meals.
    let fresh = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_ne!(fresh.draft_id, view.draft_id);
    assert_eq!(slot(&fresh, MON).components, vec![rc("h-a")]);
}

#[test]
fn another_households_draft_is_not_found() {
    let mut conn = seeded("h");
    seed_into(&mut conn, "other");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let mut forged = env(&view, "r1");
    forged.context.household_id = hid("other");
    let err = mutate_draft(&mut conn, &forged, &lock(MON, true), &mut minter("m")).unwrap_err();
    assert!(matches!(err, ApplicationError::DraftNotFound(_)), "{err:?}");
    let err = accept_draft(&mut conn, &forged, &mut minter("a")).unwrap_err();
    assert!(matches!(err, ApplicationError::DraftNotFound(_)), "{err:?}");
    assert!(saved(&conn).is_empty());
}

/// Failure injected at the evidence write — a ledger id that already exists — rolls back the
/// draft change with it: no revision bump, no receipt.
#[test]
fn a_failed_evidence_write_rolls_back_the_draft_edit() {
    let mut conn = seeded("h");
    let mut ids = vec!["draft-1".to_owned(), "dup".to_owned()].into_iter();
    let view = open_draft(&mut conn, &ctx(), &mut move || ids.next().unwrap()).unwrap();
    let err = mutate_draft(&mut conn, &env(&view, "r1"), &lock(MON, true), &mut || {
        "dup".to_owned()
    })
    .unwrap_err();
    assert!(matches!(err, ApplicationError::Storage(_)), "{err:?}");
    let again = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    assert_eq!(again.revision, view.revision);
    assert!(!slot(&again, MON).committed);
    let tx = conn.transaction().unwrap();
    assert_eq!(load_receipt_in(&tx, &hid("h"), "r1").unwrap(), None);
}

// --- H2: exact acceptance ------------------------------------------------------------------------

/// Legacy apply re-planned at Accept, so a recipe added after the preview could replace what
/// the household reviewed. Accept now writes the displayed proposal.
#[test]
fn accept_writes_the_reviewed_proposal_not_a_fresh_plan() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let shown: Vec<_> = view.slots.iter().map(|s| s.components.clone()).collect();
    // A new recipe the member likes: a fresh plan now prefers it, and it is referenced by
    // nothing the draft shows, so the draft stays fresh.
    save_recipe(&mut conn, &recipe("h", "h-z", "Zulu rice", "Cook.", 1)).unwrap();
    kimatta_storage::save_member_preferences(
        &mut conn,
        &hid("h"),
        &MemberId::new("m-h").unwrap(),
        &kimatta_storage::MemberPreferences::new([kimatta_storage::MemberPreference::new(
            kimatta_storage::Sentiment::Like,
            "zulu",
        )
        .unwrap()]),
    )
    .unwrap();
    let fresh = kimatta_storage::load_planning_snapshot(&mut conn, &hid("h"), d(MON), 0).unwrap();
    assert!(plan(&fresh, &SearchParams::default())
        .proposed
        .iter()
        .any(|p| p.components == vec![rc("h-z")]));
    accept_draft(&mut conn, &env(&view, "r1"), &mut minter("a")).unwrap();
    let after: Vec<_> = saved(&conn)
        .iter()
        .map(|m| m.components().to_vec())
        .collect();
    assert_eq!(after, shown);
    assert!(
        saved(&conn).iter().all(|m| !m.locked()),
        "suggestions are written unlocked"
    );
}

#[test]
fn lock_only_and_unlock_only_edits_are_written_and_keep_row_ids() {
    let mut conn = seeded("h");
    for (id, day, r) in [
        ("pm-mon", MON, "h-a"),
        ("pm-tue", TUE, "h-b"),
        ("pm-wed", WED, "h-c"),
    ] {
        save_meal(&mut conn, id, day, r, false);
    }
    let view = open_draft(&mut conn, &ctx(), &mut minter("o1")).unwrap();
    assert!(!view.pending_changes);
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &lock(TUE, true),
        &mut minter("m1"),
    )
    .unwrap();
    accept_draft(&mut conn, &env(&view, "r2"), &mut minter("a1")).unwrap();
    let after = saved(&conn);
    assert_eq!(after[1].id().as_str(), "pm-tue");
    assert!(
        after[1].locked(),
        "legacy apply skipped equal components and lost the lock"
    );
    let view = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert!(slot(&view, TUE).committed);
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r3"),
        &lock(TUE, false),
        &mut minter("m2"),
    )
    .unwrap();
    accept_draft(&mut conn, &env(&view, "r4"), &mut minter("a2")).unwrap();
    let after = saved(&conn);
    assert_eq!(after[1].id().as_str(), "pm-tue");
    assert!(!after[1].locked());
    assert_eq!(after[0].id().as_str(), "pm-mon");
    assert_eq!(after[2].id().as_str(), "pm-wed");
}

#[test]
fn a_retried_request_applies_once_and_a_reused_id_with_other_content_is_refused() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let e = env(&view, "choose-1");
    let first = mutate_draft(&mut conn, &e, &choose(TUE, "h-e", false), &mut minter("m1")).unwrap();
    let rows = ledger_len(&conn);
    let replay =
        mutate_draft(&mut conn, &e, &choose(TUE, "h-e", false), &mut minter("m2")).unwrap();
    assert_eq!(replay.revision, first.revision);
    assert_eq!(replay.operation, first.operation);
    assert_eq!(ledger_len(&conn), rows);
    let err =
        mutate_draft(&mut conn, &e, &choose(TUE, "h-d", false), &mut minter("m3")).unwrap_err();
    assert!(
        matches!(
            err,
            ApplicationError::Storage(StorageError::ReceiptConflict { .. })
        ),
        "{err:?}"
    );
    let a = env(&first, "accept-1");
    let accepted = accept_draft(&mut conn, &a, &mut minter("a1")).unwrap();
    let rows = ledger_len(&conn);
    let meals = saved(&conn);
    let again = accept_draft(&mut conn, &a, &mut minter("a2")).unwrap();
    assert_eq!(again.receipt, accepted.receipt);
    assert_eq!(again.state, DraftState::Accepted);
    assert_eq!(ledger_len(&conn), rows);
    assert_eq!(saved(&conn), meals);
}

#[test]
fn a_failed_accept_rolls_back_meals_locks_receipt_and_evidence() {
    let mut conn = seeded("h");
    let mut ids = vec!["draft-1".to_owned(), "dup".to_owned()].into_iter();
    let view = open_draft(&mut conn, &ctx(), &mut move || ids.next().unwrap()).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &lock(MON, true),
        &mut minter("m"),
    )
    .unwrap();
    let rows = ledger_len(&conn);
    let mut n = 0;
    // Planned-meal ids are fresh; the ledger id collides with the draft_start row.
    let err = accept_draft(&mut conn, &env(&view, "r2"), &mut || {
        n += 1;
        if n <= 3 {
            format!("pm-{n}")
        } else {
            "dup".to_owned()
        }
    })
    .unwrap_err();
    assert!(matches!(err, ApplicationError::Storage(_)), "{err:?}");
    assert!(saved(&conn).is_empty());
    assert_eq!(ledger_len(&conn), rows);
    let still = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    assert_eq!(still.state, DraftState::Active);
    assert_eq!(still.revision, view.revision);
}

/// A lock written elsewhere after the draft read the slot is never overwritten.
#[test]
fn an_external_new_lock_stops_accept_until_review_and_then_survives() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    save_meal(&mut conn, "pm-ext", TUE, "h-e", true);
    let err = accept_draft(&mut conn, &env(&view, "r1"), &mut minter("a")).unwrap_err();
    assert!(matches!(err, ApplicationError::DraftNeedsReview), "{err:?}");
    let review = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_eq!(review.state, DraftState::NeedsReview);
    assert!(slot(&review, TUE).in_review);
    assert_eq!(
        slot(&review, TUE).saved,
        Some(SavedView {
            components: vec![rc("h-e")],
            locked: true
        })
    );
    assert!(!review.accept_allowed);
    let err = mutate_draft(
        &mut conn,
        &env(&review, "r2"),
        &lock(MON, true),
        &mut minter("m"),
    )
    .unwrap_err();
    assert!(matches!(err, ApplicationError::DraftNeedsReview), "{err:?}");
    let err = mutate_draft(
        &mut conn,
        &env(&review, "r3"),
        &DraftAction::Review {
            resolutions: Vec::new(),
        },
        &mut minter("m"),
    )
    .unwrap_err();
    assert!(
        matches!(err, ApplicationError::ReviewIncomplete { .. }),
        "{err:?}"
    );
    let reviewed = mutate_draft(
        &mut conn,
        &env(&review, "r4"),
        &DraftAction::Review {
            resolutions: vec![ReviewResolution {
                date: d(TUE),
                slot: MealSlot::Dinner,
                use_saved: true,
            }],
        },
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(reviewed.state, DraftState::Active);
    assert_eq!(slot(&reviewed, TUE).components, vec![rc("h-e")]);
    assert!(slot(&reviewed, TUE).committed);
    accept_draft(&mut conn, &env(&reviewed, "r5"), &mut minter("a2")).unwrap();
    let tue = saved(&conn)
        .into_iter()
        .find(|m| m.date() == d(TUE))
        .unwrap();
    assert_eq!(tue.id().as_str(), "pm-ext");
    assert!(tue.locked());
}

#[test]
fn use_my_choice_in_review_is_an_explicit_replacement_of_the_new_lock() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let mine = slot(&view, TUE).components.clone();
    save_meal(&mut conn, "pm-ext", TUE, "h-e", true);
    let review = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    let reviewed = mutate_draft(
        &mut conn,
        &env(&review, "r1"),
        &DraftAction::Review {
            resolutions: vec![ReviewResolution {
                date: d(TUE),
                slot: MealSlot::Dinner,
                use_saved: false,
            }],
        },
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(slot(&reviewed, TUE).origin, SlotOrigin::Chosen);
    accept_draft(&mut conn, &env(&reviewed, "r2"), &mut minter("a")).unwrap();
    let tue = saved(&conn)
        .into_iter()
        .find(|m| m.date() == d(TUE))
        .unwrap();
    assert_eq!(tue.components(), mine.as_slice());
    assert!(tue.locked(), "a household choice stays locked in");
}

// --- H3: authority ---------------------------------------------------------------------------------

#[test]
fn a_locked_slot_changes_only_by_explicit_replacement_and_scope_is_enforced() {
    let mut conn = seeded("h");
    save_meal(&mut conn, "pm-mon", MON, "h-a", true);
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    assert!(slot(&view, MON).committed);
    let err = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &choose(MON, "h-b", false),
        &mut minter("m"),
    )
    .unwrap_err();
    assert!(
        matches!(err, ApplicationError::SlotLockedIn { .. }),
        "{err:?}"
    );
    let err = mutate_draft(
        &mut conn,
        &env(&view, "r2"),
        &DraftAction::Choose {
            date: d(MON),
            slot: MealSlot::Lunch,
            components: vec![rc("h-b")],
            explicit_replace: true,
        },
        &mut minter("m"),
    )
    .unwrap_err();
    assert!(
        matches!(err, ApplicationError::SlotNotInDraft { .. }),
        "{err:?}"
    );
    let err = mutate_draft(
        &mut conn,
        &env(&view, "r3"),
        &choose(MON, "other-a", true),
        &mut minter("m"),
    )
    .unwrap_err();
    assert!(matches!(err, ApplicationError::InvalidChoice(_)), "{err:?}");
    // An explicit replacement is the household's word for that slot alone.
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r4"),
        &choose(MON, "h-b", true),
        &mut minter("m"),
    )
    .unwrap();
    accept_draft(&mut conn, &env(&view, "r5"), &mut minter("a")).unwrap();
    let mon = saved(&conn)
        .into_iter()
        .find(|m| m.date() == d(MON))
        .unwrap();
    assert_eq!(mon.components(), &[rc("h-b")]);
    assert!(mon.locked());
}

#[test]
fn a_past_slot_cannot_be_edited() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx_on(TUE), &mut minter("o")).unwrap();
    assert!(!slot(&view, MON).editable);
    assert!(
        slot(&view, MON).components.is_empty(),
        "a past empty slot is not filled"
    );
    let err = mutate_draft(
        &mut conn,
        &env_on(&view, "r1", TUE),
        &choose(MON, "h-b", false),
        &mut minter("m"),
    )
    .unwrap_err();
    assert!(
        matches!(err, ApplicationError::SlotInPast { .. }),
        "{err:?}"
    );
    // The past empty slot does not block Accept of the rest.
    assert!(view.accept_allowed);
}

// --- H5: freshness ---------------------------------------------------------------------------------

#[test]
fn an_instruction_or_quantity_edit_to_a_shown_recipe_stales_acceptance() {
    for edit in [("Cook differently.", 1), ("Cook.", 2)] {
        let mut conn = seeded("h");
        let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
        let MealComponent::Recipe { recipe_id, .. } = &slot(&view, MON).components[0] else {
            panic!("a recipe suggestion");
        };
        let title = kimatta_storage::load_recipe(&conn, &hid("h"), recipe_id)
            .unwrap()
            .unwrap()
            .recipe
            .title()
            .to_owned();
        save_recipe(
            &mut conn,
            &recipe("h", recipe_id.as_str(), &title, edit.0, edit.1),
        )
        .unwrap();
        let err = accept_draft(&mut conn, &env(&view, "r1"), &mut minter("a")).unwrap_err();
        assert!(matches!(err, ApplicationError::DraftNeedsReview), "{err:?}");
        assert!(saved(&conn).is_empty());
        let review = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
        assert_eq!(review.state, DraftState::NeedsReview);
        let reviewed = mutate_draft(
            &mut conn,
            &env(&review, "r2"),
            &DraftAction::Review {
                resolutions: Vec::new(),
            },
            &mut minter("m"),
        )
        .unwrap();
        accept_draft(&mut conn, &env(&reviewed, "r3"), &mut minter("a2")).unwrap();
        assert_eq!(saved(&conn).len(), 3);
    }
}

#[test]
fn a_new_rule_stales_the_draft_and_a_vetoed_suggestion_blocks_accept() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let MealComponent::Recipe { recipe_id, .. } = &slot(&view, MON).components[0] else {
        panic!("a recipe suggestion");
    };
    let title = kimatta_storage::load_recipe(&conn, &hid("h"), recipe_id)
        .unwrap()
        .unwrap()
        .recipe
        .title()
        .to_owned();
    save_policy(
        &mut conn,
        &Policy::new(
            PolicyId::new("veto").unwrap(),
            hid("h"),
            "food",
            FoodPolicies::HARD_VETO,
            BTreeMap::from([("subject".to_owned(), title)]),
            true,
            EvidenceSource::ExplicitUser,
        )
        .unwrap(),
    )
    .unwrap();
    let review = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_eq!(review.state, DraftState::NeedsReview);
    let reviewed = mutate_draft(
        &mut conn,
        &env(&review, "r1"),
        &DraftAction::Review {
            resolutions: Vec::new(),
        },
        &mut minter("m"),
    )
    .unwrap();
    // The suggestion is shown with its conflict and never promoted to a lock (H3).
    let mon = slot(&reviewed, MON);
    assert_eq!(mon.state, CoverageState::NeedsAttention);
    assert!(!mon.committed);
    assert!(!reviewed.accept_allowed);
    let err = accept_draft(&mut conn, &env(&reviewed, "r2"), &mut minter("a")).unwrap_err();
    assert!(matches!(err, ApplicationError::AcceptRefused(_)), "{err:?}");
    assert!(saved(&conn).is_empty(), "a refused Accept writes nothing");
}

#[test]
fn ledger_appends_do_not_stale_a_draft() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    crate::cover_cycle(
        &mut conn,
        &crate::CoverCycleRequest {
            household_id: hid("h"),
            today: d(MON),
            offset_cycles: 0,
            apply: false,
            params: SearchParams::default(),
        },
        &mut minter("preview"),
    )
    .unwrap();
    accept_draft(&mut conn, &env(&view, "r1"), &mut minter("a")).unwrap();
}

#[test]
fn a_date_rollover_drops_pending_past_changes_and_freezes_them() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &choose(MON, "h-e", false),
        &mut minter("m"),
    )
    .unwrap();
    let next_day = open_draft(&mut conn, &ctx_on(TUE), &mut minter("o2")).unwrap();
    assert_eq!(next_day.draft_id, view.draft_id);
    assert_eq!(
        next_day.state,
        DraftState::Active,
        "a rollover alone needs no review"
    );
    assert_eq!(next_day.past_changes_dropped, 1);
    assert!(!slot(&next_day, MON).editable);
    accept_draft(&mut conn, &env_on(&next_day, "r2", TUE), &mut minter("a")).unwrap();
    let dates: Vec<_> = saved(&conn).iter().map(PlannedMeal::date).collect();
    assert_eq!(
        dates,
        vec![d(TUE), d(WED)],
        "nothing is written to a past date"
    );
}

#[test]
fn a_stale_revision_is_refused() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &lock(MON, true),
        &mut minter("m"),
    )
    .unwrap();
    let err = mutate_draft(
        &mut conn,
        &env(&view, "r2"),
        &lock(TUE, true),
        &mut minter("m2"),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApplicationError::StaleDraft {
                expected: 0,
                found: 1
            }
        ),
        "{err:?}"
    );
    let err = accept_draft(&mut conn, &env(&view, "r3"), &mut minter("a")).unwrap_err();
    assert!(
        matches!(err, ApplicationError::StaleDraft { .. }),
        "{err:?}"
    );
}

// --- undo, reconsider, lifecycle (M3) --------------------------------------------------------

#[test]
fn undo_restores_one_step_survives_restart_and_has_no_redo() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    assert!(!view.undo_available);
    let before = slot(&view, TUE).clone();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &choose(TUE, "h-e", false),
        &mut minter("m"),
    )
    .unwrap();
    let resumed = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert!(resumed.undo_available);
    let undone = mutate_draft(
        &mut conn,
        &env(&resumed, "r2"),
        &DraftAction::Undo,
        &mut minter("u"),
    )
    .unwrap();
    assert_eq!(undone.revision, view.revision + 1);
    assert_eq!(slot(&undone, TUE).components, before.components);
    assert!(!slot(&undone, TUE).committed);
    assert!(!undone.undo_available);
    let err = mutate_draft(
        &mut conn,
        &env(&undone, "r3"),
        &DraftAction::Undo,
        &mut minter("u2"),
    )
    .unwrap_err();
    assert!(matches!(err, ApplicationError::NothingToUndo), "{err:?}");
}

#[test]
fn a_no_change_command_keeps_the_existing_undo_checkpoint() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &lock(MON, true),
        &mut minter("m"),
    )
    .unwrap();
    // Locking an already-locked slot changes nothing.
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r2"),
        &lock(MON, true),
        &mut minter("m2"),
    )
    .unwrap();
    let undone = mutate_draft(
        &mut conn,
        &env(&view, "r3"),
        &DraftAction::Undo,
        &mut minter("u"),
    )
    .unwrap();
    assert!(
        !slot(&undone, MON).committed,
        "Undo reverts the real change, not the no-op"
    );
}

#[test]
fn a_format_or_algorithm_change_requires_review() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    conn.execute(
        "UPDATE planning_draft SET algorithm_version = algorithm_version - 1",
        [],
    )
    .unwrap();
    let review = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_eq!(review.state, DraftState::NeedsReview);
    assert_eq!(review.draft_id, view.draft_id);
    let reviewed = mutate_draft(
        &mut conn,
        &env(&review, "r1"),
        &DraftAction::Review {
            resolutions: Vec::new(),
        },
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(reviewed.state, DraftState::Active);
    let tx = conn.transaction().unwrap();
    let row = load_draft_in(&tx, &hid("h"), &view.draft_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.algorithm_version, PLANNER_ALGORITHM_VERSION);
}

#[test]
fn an_elapsed_window_expires_stays_readable_and_can_be_discarded() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &choose(WED, "h-e", false),
        &mut minter("m"),
    )
    .unwrap();
    // Thursday starts the next window.
    let thu = "2026-10-01";
    let next = open_draft(&mut conn, &ctx_on(thu), &mut minter("o2")).unwrap();
    assert_ne!(next.draft_id, view.draft_id);
    assert_eq!(next.expired.len(), 1);
    assert_eq!(next.expired[0].draft_id, view.draft_id);
    let old = inspect_draft(&mut conn, &ctx_on(thu), &view.draft_id).unwrap();
    assert_eq!(old.state, DraftState::Expired);
    assert!(!old.accept_allowed);
    assert_eq!(slot(&old, WED).components, vec![rc("h-e")]);
    let mut e = env_on(&old, "r2", thu);
    e.expected_revision = old.revision;
    let err = accept_draft(&mut conn, &e, &mut minter("a")).unwrap_err();
    assert!(matches!(err, ApplicationError::DraftClosed(_)), "{err:?}");
    mutate_draft(&mut conn, &e, &DraftAction::Discard, &mut minter("x")).unwrap();
    let later = open_draft(&mut conn, &ctx_on(thu), &mut minter("o3")).unwrap();
    assert!(later.expired.is_empty());
    assert!(saved(&conn).is_empty(), "expiry never writes a meal");
}

#[test]
fn a_redefined_window_sets_the_old_draft_aside_unapplied() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    save_planning_cycle(
        &mut conn,
        &PlanningCycle::new(hid("h"), d(MON), 4, MealScope::dinner_only()).unwrap(),
    )
    .unwrap();
    let err = accept_draft(&mut conn, &env(&view, "r1"), &mut minter("a")).unwrap_err();
    assert!(
        matches!(err, ApplicationError::DraftWindowChanged),
        "{err:?}"
    );
    let next = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_ne!(next.draft_id, view.draft_id);
    assert_eq!(next.length_days, 4);
    assert_eq!(next.expired.len(), 1);
    assert!(saved(&conn).is_empty());
}

// --- first-run receipts ---------------------------------------------------------------------------

#[test]
fn a_choice_whose_recipe_was_archived_blocks_accept_without_substitution() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &choose(TUE, "h-e", false),
        &mut minter("m"),
    )
    .unwrap();
    archive_recipe(&mut conn, &hid("h"), &RecipeId::new("h-e").unwrap(), d(MON)).unwrap();
    let review = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    let reviewed = mutate_draft(
        &mut conn,
        &env(&review, "r2"),
        &DraftAction::Review {
            resolutions: Vec::new(),
        },
        &mut minter("m2"),
    )
    .unwrap();
    let tue = slot(&reviewed, TUE);
    assert_eq!(tue.components, vec![rc("h-e")], "never substituted");
    assert!(tue
        .reason_codes
        .iter()
        .any(|c| c == DRAFT_RECIPE_UNAVAILABLE));
    let err = accept_draft(&mut conn, &env(&reviewed, "r3"), &mut minter("a")).unwrap_err();
    assert!(matches!(err, ApplicationError::AcceptRefused(_)), "{err:?}");
    let ok = mutate_draft(
        &mut conn,
        &env(&reviewed, "r4"),
        &choose(TUE, "h-d", true),
        &mut minter("m3"),
    )
    .unwrap();
    let accepted = accept_draft(&mut conn, &env(&ok, "r5"), &mut minter("a2")).unwrap();
    assert_eq!(accepted.receipt.as_ref().map(|r| r.changed_slots), Some(3));
}

// --- Step 3: Another and week alternatives (§6, H3, R4) -------------------------------------------

fn another(date: &str) -> DraftAction {
    DraftAction::Another {
        date: d(date),
        slot: MealSlot::Dinner,
    }
}

fn outcome_of(view: &DraftView, date: &str) -> Option<SlotOutcome> {
    view.operation
        .as_ref()
        .and_then(|op| op.slots.iter().find(|s| s.date == date))
        .map(|s| s.outcome)
}

#[test]
fn another_changes_one_slot_and_holds_the_others_and_their_commitments() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &lock(MON, true),
        &mut minter("m1"),
    )
    .unwrap();
    let before = view.slots.clone();
    let after = mutate_draft(
        &mut conn,
        &env(&view, "r2"),
        &another(TUE),
        &mut minter("m2"),
    )
    .unwrap();
    assert_eq!(outcome_of(&after, TUE), Some(SlotOutcome::Changed));
    assert_ne!(slot(&after, TUE).components, before[1].components);
    assert!(!slot(&after, TUE).committed);
    for (b, a) in before.iter().zip(&after.slots) {
        if b.date != d(TUE) {
            assert_eq!(a.components, b.components);
            assert_eq!(a.committed, b.committed);
        }
    }
    assert!(saved(&conn).is_empty(), "Another is a draft edit");
    let undone = mutate_draft(
        &mut conn,
        &env(&after, "r3"),
        &DraftAction::Undo,
        &mut minter("u"),
    )
    .unwrap();
    assert_eq!(slot(&undone, TUE).components, before[1].components);
}

/// Repeated Another walks through every eligible dish once, then says so and keeps the meal:
/// no recycling, no fallback. Reconsider makes the history eligible again without searching.
#[test]
fn repeated_another_exhausts_without_recycling_and_reconsider_reopens_the_pool() {
    let mut conn = seeded("h");
    let mut view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let mut seen = BTreeSet::new();
    seen.insert(format!("{:?}", slot(&view, TUE).components));
    let mut n = 0;
    loop {
        n += 1;
        let request = format!("a{n}");
        let next = mutate_draft(
            &mut conn,
            &env(&view, &request),
            &another(TUE),
            &mut minter("m"),
        )
        .unwrap();
        match outcome_of(&next, TUE) {
            Some(SlotOutcome::Changed) => {
                assert!(
                    seen.insert(format!("{:?}", slot(&next, TUE).components)),
                    "a rejected dish came back"
                );
            }
            Some(SlotOutcome::Exhausted) => {
                assert_eq!(slot(&next, TUE).components, slot(&view, TUE).components);
                view = next;
                break;
            }
            other => panic!("unexpected outcome {other:?}"),
        }
        view = next;
        assert!(n < 10, "five recipes cannot take ten requests");
    }
    assert_eq!(
        seen.len(),
        5,
        "every household dish was offered exactly once"
    );
    // Asking again records the request and still changes nothing.
    let again = mutate_draft(
        &mut conn,
        &env(&view, "again"),
        &another(TUE),
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(outcome_of(&again, TUE), Some(SlotOutcome::Exhausted));
    assert_eq!(slot(&again, TUE).components, slot(&view, TUE).components);
    let reconsidered = mutate_draft(
        &mut conn,
        &env(&again, "rc"),
        &DraftAction::Reconsider {
            slot: Some((d(TUE), MealSlot::Dinner)),
        },
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(slot(&reconsidered, TUE).excluded, 0);
    assert_eq!(
        slot(&reconsidered, TUE).components,
        slot(&again, TUE).components,
        "Reconsider does not regenerate"
    );
    let fresh = mutate_draft(
        &mut conn,
        &env(&reconsidered, "a-new"),
        &another(TUE),
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(outcome_of(&fresh, TUE), Some(SlotOutcome::Changed));
}

#[test]
fn another_refuses_locked_past_and_identityless_slots() {
    let mut conn = seeded("h");
    save_meal(&mut conn, "pm-tue", TUE, "h-a", true);
    let view = open_draft(&mut conn, &ctx_on(TUE), &mut minter("o")).unwrap();
    let e = |r: &str| env_on(&view, r, TUE);
    let err = mutate_draft(&mut conn, &e("r1"), &another(TUE), &mut minter("m")).unwrap_err();
    assert!(
        matches!(err, ApplicationError::SlotLockedIn { .. }),
        "{err:?}"
    );
    let err = mutate_draft(&mut conn, &e("r2"), &another(MON), &mut minter("m")).unwrap_err();
    assert!(
        matches!(err, ApplicationError::SlotInPast { .. }),
        "{err:?}"
    );
    let frozen = mutate_draft(
        &mut conn,
        &e("r3"),
        &DraftAction::Choose {
            date: d(WED),
            slot: MealSlot::Dinner,
            components: vec![MealComponent::FrozenQuick { note: None }],
            explicit_replace: false,
        },
        &mut minter("m"),
    )
    .unwrap();
    let frozen = mutate_draft(
        &mut conn,
        &env_on(&frozen, "r4", TUE),
        &lock(WED, false),
        &mut minter("m"),
    )
    .unwrap();
    let blocked = mutate_draft(
        &mut conn,
        &env_on(&frozen, "r5", TUE),
        &another(WED),
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(outcome_of(&blocked, WED), Some(SlotOutcome::Blocked));
    assert_eq!(
        slot(&blocked, WED).components,
        vec![MealComponent::FrozenQuick { note: None }],
        "no recipe identity is guessed for a fallback"
    );
}

/// The whole occurrence is excluded, so a multi-dish meal cannot come back by one component.
#[test]
fn another_excludes_every_dish_of_a_multi_component_meal() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &DraftAction::Choose {
            date: d(TUE),
            slot: MealSlot::Dinner,
            components: vec![rc("h-a"), rc("h-b")],
            explicit_replace: false,
        },
        &mut minter("m"),
    )
    .unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r2"),
        &lock(TUE, false),
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(slot(&view, TUE).origin, SlotOrigin::Suggested);
    let next = mutate_draft(
        &mut conn,
        &env(&view, "r3"),
        &another(TUE),
        &mut minter("m"),
    )
    .unwrap();
    let shown = &slot(&next, TUE).components;
    assert!(!shown.contains(&rc("h-a")) && !shown.contains(&rc("h-b")));
    assert_eq!(slot(&next, TUE).excluded, 2);
}

#[test]
fn the_week_action_changes_unlocked_suggestions_only_and_excludes_them_draft_wide() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &lock(MON, true),
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(view.week_targets, 2);
    let before = view.slots.clone();
    let after = mutate_draft(
        &mut conn,
        &env(&view, "r2"),
        &DraftAction::Alternatives,
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(
        slot(&after, MON).components,
        before[0].components,
        "locked in stays"
    );
    for (i, day) in [(1, TUE), (2, WED)] {
        assert_eq!(outcome_of(&after, day), Some(SlotOutcome::Changed));
        let shown = &slot(&after, day).components;
        assert_ne!(shown, &before[1].components);
        assert_ne!(
            shown, &before[2].components,
            "displaced dishes bind every target"
        );
        let _ = i;
    }
    assert_eq!(after.excluded, 2);
    assert!(saved(&conn).is_empty());
    // One week request is one ledger action with per-slot detail.
    let entries = list_ledger_entries(&conn, &hid("h")).unwrap();
    let last = &entries.last().unwrap().1;
    assert_eq!(last.selected_action, ACTION_DRAFT_ALTERNATIVES);
    assert_eq!(last.payload.matches("slot=").count(), 2);
}

#[test]
fn a_partially_exhausted_week_keeps_the_rest_usable() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    // Three suggestions shown; two week requests displace six identities out of five: the
    // second request can place at most what is left.
    let first = mutate_draft(
        &mut conn,
        &env(&view, "w1"),
        &DraftAction::Alternatives,
        &mut minter("m"),
    )
    .unwrap();
    let second = mutate_draft(
        &mut conn,
        &env(&first, "w2"),
        &DraftAction::Alternatives,
        &mut minter("m"),
    )
    .unwrap();
    let outcomes: Vec<_> = [MON, TUE, WED]
        .iter()
        .map(|day| outcome_of(&second, day))
        .collect();
    assert!(outcomes.contains(&Some(SlotOutcome::Exhausted)));
    for day in [MON, TUE, WED] {
        if outcome_of(&second, day) == Some(SlotOutcome::Exhausted) {
            assert_eq!(slot(&second, day).components, slot(&first, day).components);
        }
        assert!(!slot(&second, day).components.is_empty());
    }
    assert!(second.accept_allowed, "a usable week stays usable");
}

#[test]
fn with_everything_locked_the_week_action_has_no_targets_and_changes_nothing() {
    let mut conn = seeded("h");
    let mut view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    for (n, day) in [MON, TUE, WED].iter().enumerate() {
        view = mutate_draft(
            &mut conn,
            &env(&view, &format!("l{n}")),
            &lock(day, true),
            &mut minter("m"),
        )
        .unwrap();
    }
    assert_eq!(view.week_targets, 0);
    let after = mutate_draft(
        &mut conn,
        &env(&view, "w"),
        &DraftAction::Alternatives,
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(
        after
            .slots
            .iter()
            .map(|s| &s.components)
            .collect::<Vec<_>>(),
        view.slots.iter().map(|s| &s.components).collect::<Vec<_>>()
    );
    assert!(after.operation.as_ref().unwrap().slots.is_empty());
}

/// R4 adversarial: a dish vetoed after it was left exhausted keeps its place visibly, with its
/// conflict, and cannot be accepted — exhaustion is never a way back to coverage.
#[test]
fn an_exhausted_slot_whose_dish_is_then_vetoed_cannot_be_accepted() {
    let mut conn = seeded("h");
    let mut view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    for n in 0..6 {
        view = mutate_draft(
            &mut conn,
            &env(&view, &format!("a{n}")),
            &another(TUE),
            &mut minter("m"),
        )
        .unwrap();
    }
    assert_eq!(outcome_of(&view, TUE), Some(SlotOutcome::Exhausted));
    let MealComponent::Recipe { recipe_id, .. } = &slot(&view, TUE).components[0] else {
        panic!("a recipe");
    };
    let title = kimatta_storage::load_recipe(&conn, &hid("h"), recipe_id)
        .unwrap()
        .unwrap()
        .recipe
        .title()
        .to_owned();
    save_policy(
        &mut conn,
        &Policy::new(
            PolicyId::new("veto").unwrap(),
            hid("h"),
            "food",
            FoodPolicies::HARD_VETO,
            BTreeMap::from([("subject".to_owned(), title)]),
            true,
            EvidenceSource::ExplicitUser,
        )
        .unwrap(),
    )
    .unwrap();
    let review = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    let reviewed = mutate_draft(
        &mut conn,
        &env(&review, "rv"),
        &DraftAction::Review {
            resolutions: Vec::new(),
        },
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(slot(&reviewed, TUE).state, CoverageState::NeedsAttention);
    let err = accept_draft(&mut conn, &env(&reviewed, "acc"), &mut minter("a")).unwrap_err();
    assert!(matches!(err, ApplicationError::AcceptRefused(_)), "{err:?}");
}

// --- Step 5: meal exclusions (§8, M1) --------------------------------------------------------------

use crate::exclusions::{add_dish_exclusions, list_exclusions, remove_exclusion, ExclusionKind};

fn dish_rule(
    conn: &mut Connection,
    dish: &str,
    acting: Option<&DraftEnvelope>,
) -> crate::exclusions::ExclusionOutcome {
    add_dish_exclusions(conn, &ctx(), &[dish.to_owned()], acting, &mut minter("x")).unwrap()
}

#[test]
fn a_dish_rule_is_listed_idempotent_removable_and_survives_discard() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let first = dish_rule(&mut conn, "recipe:h-a", None);
    let again = dish_rule(&mut conn, "recipe:h-a", None);
    assert_eq!(
        first.exclusions, again.exclusions,
        "an identical rule is a no-op"
    );
    assert_eq!(first.exclusions.len(), 1);
    let ExclusionKind::Dish {
        identity,
        title,
        available,
    } = &first.exclusions[0].kind
    else {
        panic!("an identity rule");
    };
    assert_eq!(identity, "recipe:h-a");
    assert_eq!(title.as_deref(), Some("Alpha bowl"));
    assert!(available);
    mutate_draft(
        &mut conn,
        &env(&view, "d"),
        &DraftAction::Discard,
        &mut minter("m"),
    )
    .ok();
    assert_eq!(
        list_exclusions(&conn, &hid("h")).unwrap().len(),
        1,
        "survives discard"
    );
    let removed = remove_exclusion(
        &mut conn,
        &ctx(),
        &first.exclusions[0].policy_id,
        None,
        &mut minter("x"),
    )
    .unwrap();
    assert!(removed.exclusions.is_empty());
    // History is kept: the row is disabled, not deleted, and both changes are in the ledger.
    let rows = kimatta_storage::list_policies(&conn, &hid("h"), "food").unwrap();
    assert!(rows
        .iter()
        .any(|p| p.policy_type == FoodPolicies::RECIPE_VETO && !p.enabled));
    let actions: Vec<_> = list_ledger_entries(&conn, &hid("h"))
        .unwrap()
        .into_iter()
        .map(|(_, e)| e.selected_action)
        .collect();
    assert!(actions.contains(&crate::exclusions::ACTION_POLICY_ADD.to_owned()));
    assert!(actions.contains(&crate::exclusions::ACTION_POLICY_REMOVE.to_owned()));
}

/// An identity rule rejects exactly the dish. The vetoed dish is called "Rice" and every other
/// recipe has a "rice" line: a rule read as its title's phrase would reject them all.
#[test]
fn a_dish_rule_never_rejects_a_dish_that_merely_shares_an_ingredient() {
    let mut conn = seeded("h");
    save_recipe(&mut conn, &recipe("h", "h-a", "Rice", "Cook.", 1)).unwrap();
    dish_rule(&mut conn, "recipe:h-a", None);
    let mut view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let mut offered = BTreeSet::new();
    for n in 0..6 {
        for s in &view.slots {
            offered.insert(format!("{:?}", s.components));
        }
        view = mutate_draft(
            &mut conn,
            &env(&view, &format!("a{n}")),
            &another(TUE),
            &mut minter("m"),
        )
        .unwrap();
    }
    assert!(
        !offered.iter().any(|o| o.contains("\"h-a\"")),
        "{offered:?}"
    );
    assert_eq!(
        offered.len(),
        4,
        "all four other dishes stay eligible: {offered:?}"
    );
}

#[test]
fn legacy_phrase_rules_keep_their_meaning_and_are_removed_one_at_a_time() {
    let mut conn = seeded("h");
    for (id, subject) in [("p1", "rice"), ("p2", "stew")] {
        save_policy(
            &mut conn,
            &Policy::new(
                PolicyId::new(id).unwrap(),
                hid("h"),
                "food",
                FoodPolicies::HARD_VETO,
                BTreeMap::from([("subject".to_owned(), subject.to_owned())]),
                true,
                EvidenceSource::ExplicitUser,
            )
            .unwrap(),
        )
        .unwrap();
    }
    let listed = list_exclusions(&conn, &hid("h")).unwrap();
    assert_eq!(
        listed.iter().map(|e| e.kind.clone()).collect::<Vec<_>>(),
        vec![
            ExclusionKind::Phrase {
                subject: "rice".to_owned()
            },
            ExclusionKind::Phrase {
                subject: "stew".to_owned()
            },
        ]
    );
    // "rice" is every recipe's ingredient line: the legacy rule still rejects them all.
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    assert!(view.slots.iter().all(|s| !s
        .components
        .iter()
        .any(|c| matches!(c, MealComponent::Recipe { .. }))));
    let after = remove_exclusion(&mut conn, &ctx(), "p1", None, &mut minter("x")).unwrap();
    assert_eq!(
        after
            .exclusions
            .iter()
            .map(|e| e.policy_id.as_str())
            .collect::<Vec<_>>(),
        vec!["p2"]
    );
}

#[test]
fn an_archived_dish_keeps_its_rule_and_reads_as_unavailable() {
    let mut conn = seeded("h");
    dish_rule(&mut conn, "recipe:h-c", None);
    archive_recipe(&mut conn, &hid("h"), &RecipeId::new("h-c").unwrap(), d(MON)).unwrap();
    let listed = list_exclusions(&conn, &hid("h")).unwrap();
    assert_eq!(
        listed[0].kind,
        ExclusionKind::Dish {
            identity: "recipe:h-c".to_owned(),
            title: Some("Charlie tacos".to_owned()),
            available: false,
        }
    );
}

/// The draft that made the rule keeps its choices, shows the conflict, clears Undo and stays
/// active; another draft meets the rule as a needs-review.
#[test]
fn a_rule_made_from_a_draft_reassesses_it_without_replanning() {
    let mut conn = seeded("h");
    let next_week = DraftContext {
        offset_cycles: 1,
        ..ctx()
    };
    let other = open_draft(&mut conn, &next_week, &mut minter("o0")).unwrap();
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let view = mutate_draft(
        &mut conn,
        &env(&view, "r1"),
        &lock(WED, true),
        &mut minter("m"),
    )
    .unwrap();
    assert!(view.undo_available);
    let MealComponent::Recipe { recipe_id, .. } = &slot(&view, MON).components[0] else {
        panic!("a recipe suggestion");
    };
    let dish = format!("recipe:{}", recipe_id.as_str());
    let e = env(&view, "rule");
    let out = dish_rule(&mut conn, &dish, Some(&e));
    let draft = out.draft.expect("the acting draft is returned");
    assert_eq!(draft.state, DraftState::Active);
    assert_eq!(
        slot(&draft, MON).components,
        slot(&view, MON).components,
        "no substitution"
    );
    assert_eq!(slot(&draft, MON).state, CoverageState::NeedsAttention);
    assert!(
        !draft.undo_available,
        "Undo cannot restore a pre-rule state as reviewed"
    );
    assert!(!draft.accept_allowed);
    let other_now = open_draft(&mut conn, &next_week, &mut minter("o2")).unwrap();
    assert_eq!(other_now.draft_id, other.draft_id);
    assert_eq!(other_now.state, DraftState::NeedsReview);
    // Another resolves the conflict and Accept then succeeds.
    let fixed = mutate_draft(
        &mut conn,
        &env(&draft, "a"),
        &another(MON),
        &mut minter("m"),
    )
    .unwrap();
    assert!(fixed.accept_allowed);
}

/// A draft already behind an edit made elsewhere is not carried along by the rule: that edit
/// still has to be reviewed.
#[test]
fn a_rule_does_not_absorb_an_unreviewed_external_edit() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    save_meal(&mut conn, "pm-ext", TUE, "h-e", true);
    let out = dish_rule(&mut conn, "recipe:h-d", Some(&env(&view, "rule")));
    assert!(out.draft.is_none());
    let again = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_eq!(again.state, DraftState::NeedsReview);
}

#[test]
fn rules_are_household_scoped() {
    let mut conn = seeded("h");
    seed_into(&mut conn, "other");
    let theirs = add_dish_exclusions(
        &mut conn,
        &DraftContext {
            household_id: hid("other"),
            ..ctx()
        },
        &["recipe:other-a".to_owned()],
        None,
        &mut minter("x"),
    )
    .unwrap();
    // Another household's dish is no dish of ours...
    let err = add_dish_exclusions(
        &mut conn,
        &ctx(),
        &["recipe:other-a".to_owned()],
        None,
        &mut minter("x"),
    )
    .unwrap_err();
    assert!(matches!(err, ApplicationError::InvalidChoice(_)), "{err:?}");
    // ...its rule is no rule of ours...
    let err = remove_exclusion(
        &mut conn,
        &ctx(),
        &theirs.exclusions[0].policy_id,
        None,
        &mut minter("x"),
    )
    .unwrap_err();
    assert!(
        matches!(err, ApplicationError::ExclusionNotFound(_)),
        "{err:?}"
    );
    assert!(list_exclusions(&conn, &hid("h")).unwrap().is_empty());
    assert_eq!(list_exclusions(&conn, &hid("other")).unwrap().len(), 1);
    // ...and a fallback or malformed identity names no dish at all.
    for bad in ["frozen_quick", "recipe:", "starter: ", "tacos"] {
        let err = add_dish_exclusions(&mut conn, &ctx(), &[bad.to_owned()], None, &mut minter("x"))
            .unwrap_err();
        assert!(
            matches!(err, ApplicationError::InvalidChoice(_)),
            "{bad}: {err:?}"
        );
    }
}

/// An installed starter is the same dish as its stub: a rule made from its recipe card names
/// the starter identity Tier 0 compares, or it would never match anything.
#[test]
fn a_rule_on_an_installed_starter_uses_the_starter_identity() {
    let mut conn = seeded("h");
    let slug = shipped_starter_content().unwrap().recipes[0].slug.clone();
    let installed = Recipe::new(
        RecipeId::new("h-starter").unwrap(),
        hid("h"),
        "Installed starter",
        Some(1),
        Some(15),
        "",
        vec![],
        RecipeProvenance::with_rights(
            ProvenanceKind::Starter,
            None,
            None,
            None,
            None,
            Some(slug.clone()),
        )
        .unwrap(),
    )
    .unwrap();
    save_recipe(&mut conn, &installed).unwrap();
    let out = dish_rule(&mut conn, "recipe:h-starter", None);
    let ExclusionKind::Dish {
        identity,
        available,
        ..
    } = &out.exclusions[0].kind
    else {
        panic!("an identity rule");
    };
    assert_eq!(identity, &format!("starter:{slug}"));
    assert!(available);
    let snapshot =
        kimatta_storage::load_planning_snapshot(&mut conn, &hid("h"), d(MON), 0).unwrap();
    let r = plan(&snapshot, &SearchParams::default());
    assert!(r
        .proposed
        .iter()
        .all(|p| p.components != vec![rc("h-starter")]));
}

/// Confirming "we have no restrictions" is not a fact a draft's choices depend on: it only
/// settles wording, so it never sends the draft to review.
#[test]
fn confirming_no_restrictions_does_not_stale_a_draft() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    crate::record_decision(
        &mut conn,
        &crate::PlanDecisionRequest {
            household_id: hid("h"),
            today: d(MON),
            offset_cycles: 0,
            decision: crate::PlanDecision::RestrictionsReviewed,
        },
        &mut minter("rr"),
    )
    .unwrap();
    let again = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_eq!(again.state, DraftState::Active);
    assert!(!again
        .assessment
        .assumptions
        .iter()
        .any(|a| a.as_str() == "RESTRICTIONS_NOT_CONFIGURED"));
    accept_draft(&mut conn, &env(&view, "a"), &mut minter("a")).unwrap();
}

/// §12 rollback: a build with draft edits switched off keeps every draft readable and
/// discardable, never starts one, and never sends a pending draft to Accept — legacy or not.
#[test]
fn with_edits_disabled_a_draft_can_only_be_read_or_discarded() {
    let mut conn = seeded("h");
    let off = DraftContext {
        edits_enabled: false,
        ..ctx()
    };
    let err = open_draft(&mut conn, &off, &mut minter("o0")).unwrap_err();
    assert!(
        matches!(err, ApplicationError::DraftEditsDisabled),
        "{err:?}"
    );
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let rows = ledger_len(&conn);
    let read = open_draft(&mut conn, &off, &mut minter("o2")).unwrap();
    assert_eq!(read.draft_id, view.draft_id);
    assert!(!read.accept_allowed);
    assert_eq!(ledger_len(&conn), rows);
    let mut e = env(&view, "r1");
    e.context.edits_enabled = false;
    let err = mutate_draft(&mut conn, &e, &lock(MON, true), &mut minter("m")).unwrap_err();
    assert!(
        matches!(err, ApplicationError::DraftEditsDisabled),
        "{err:?}"
    );
    let err = accept_draft(&mut conn, &e, &mut minter("a")).unwrap_err();
    assert!(
        matches!(err, ApplicationError::DraftEditsDisabled),
        "{err:?}"
    );
    assert!(saved(&conn).is_empty());
    assert!(inspect_draft(&mut conn, &off, &view.draft_id).is_ok());
    let gone = mutate_draft(&mut conn, &e, &DraftAction::Discard, &mut minter("x")).unwrap();
    assert_eq!(gone.state, DraftState::Discarded);
}

fn off() -> DraftContext {
    DraftContext {
        edits_enabled: false,
        ..ctx()
    }
}

fn revision_of(conn: &mut Connection, id: &str) -> i64 {
    let tx = conn.transaction().unwrap();
    load_draft_in(&tx, &hid("h"), id).unwrap().unwrap().revision
}

/// §12 rollback: a standing rule may still be written with draft edits switched off, but the
/// acting draft is left exactly as it was — the rule never carries its identity along.
#[test]
fn with_edits_disabled_an_exclusion_writes_the_rule_and_leaves_the_draft_alone() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let mut acting = env(&view, "x1");
    acting.context.edits_enabled = false;
    let added = add_dish_exclusions(
        &mut conn,
        &off(),
        &["recipe:h-a".to_owned()],
        Some(&acting),
        &mut minter("x"),
    )
    .unwrap();
    assert!(added.draft.is_none(), "no draft view under rollback");
    assert_eq!(added.exclusions.len(), 1);
    assert_eq!(revision_of(&mut conn, &view.draft_id), view.revision);
    let removed = remove_exclusion(
        &mut conn,
        &off(),
        &added.exclusions[0].policy_id,
        Some(&acting),
        &mut minter("x"),
    )
    .unwrap();
    assert!(removed.draft.is_none());
    assert!(removed.exclusions.is_empty());
    assert_eq!(revision_of(&mut conn, &view.draft_id), view.revision);
}

#[test]
fn with_edits_disabled_inspecting_an_open_draft_never_offers_accept() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    assert!(view.accept_allowed, "the fixture draft is acceptable");
    let read = inspect_draft(&mut conn, &off(), &view.draft_id).unwrap();
    assert!(!read.accept_allowed);
}

fn corrupt(conn: &Connection, id: &str) {
    conn.execute(
        "UPDATE planning_draft SET payload = '{\"slots\": [' WHERE id = ?1",
        [id],
    )
    .unwrap();
}

/// A payload that cannot be read is set aside, never a locked window: Cover opens a fresh
/// draft, and the old one is an expired tile that can be inspected and discarded.
#[test]
fn an_unreadable_open_draft_is_set_aside_and_a_fresh_one_opens() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    corrupt(&conn, &view.draft_id);
    let next = open_draft(&mut conn, &ctx(), &mut minter("o2")).unwrap();
    assert_ne!(next.draft_id, view.draft_id);
    assert_eq!(next.expired.len(), 1);
    assert_eq!(next.expired[0].draft_id, view.draft_id);
    let old = inspect_draft(&mut conn, &ctx(), &view.draft_id).unwrap();
    assert_eq!(old.state, DraftState::Expired);
    let mut e = env(&old, "x");
    e.expected_revision = old.revision;
    let gone = mutate_draft(&mut conn, &e, &DraftAction::Discard, &mut minter("m")).unwrap();
    assert_eq!(gone.state, DraftState::Discarded);
    assert!(saved(&conn).is_empty());
}

#[test]
fn an_unreadable_open_draft_can_be_discarded_directly() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    corrupt(&conn, &view.draft_id);
    let gone = mutate_draft(
        &mut conn,
        &env(&view, "x"),
        &DraftAction::Discard,
        &mut minter("m"),
    )
    .unwrap();
    assert_eq!(gone.state, DraftState::Discarded);
}

#[test]
fn with_edits_disabled_an_unreadable_draft_is_set_aside_but_none_is_started() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    corrupt(&conn, &view.draft_id);
    let err = open_draft(&mut conn, &off(), &mut minter("o2")).unwrap_err();
    assert!(
        matches!(err, ApplicationError::DraftEditsDisabled),
        "{err:?}"
    );
    let tx = conn.transaction().unwrap();
    let rows = kimatta_storage::list_unclosed_drafts_in(&tx, &hid("h")).unwrap();
    assert_eq!(rows.len(), 1, "no new draft under rollback");
    assert_eq!(rows[0].state, DraftState::Expired);
}

#[test]
fn an_unreadable_acting_draft_does_not_block_a_rule() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    corrupt(&conn, &view.draft_id);
    let added = dish_rule(&mut conn, "recipe:h-a", Some(&env(&view, "x1")));
    assert!(added.draft.is_none());
    assert_eq!(added.exclusions.len(), 1);
}

/// Expiry bound: a draft the household never acted on closes silently when its window ends,
/// so weekly visits without planning leave no tiles behind.
#[test]
fn an_untouched_draft_closes_at_expiry_instead_of_leaving_a_tile() {
    let mut conn = seeded("h");
    let view = open_draft(&mut conn, &ctx(), &mut minter("o")).unwrap();
    let next = open_draft(&mut conn, &ctx_on("2026-10-01"), &mut minter("o2")).unwrap();
    assert!(next.expired.is_empty());
    let tx = conn.transaction().unwrap();
    let old = load_draft_in(&tx, &hid("h"), &view.draft_id)
        .unwrap()
        .unwrap();
    assert_eq!(old.state, DraftState::Discarded);
    assert!(old.payload.is_none());
}
