//! Orchestration between storage and the pure planner (`docs/V3_MIGRATION_PLAN.md` §3 places
//! this crate's arrival at MVP-023). One command: `cover_cycle` loads the snapshot, calls the
//! controller through its trait, runs the search, records the ledger row and — when asked and
//! when the plan is not `NeedsAttention` — applies the plan, all without reading a clock.
#![forbid(unsafe_code)]

use food_domain::planner::snapshot::components_text;
use food_domain::planner::{
    cover_cycle as plan, FoodController, FoodPolicies as PolicyTypes, PlanningResult, SearchParams,
    PLANNER_ALGORITHM_VERSION,
};
use food_domain::{format_civil_date, CivilDate, MealComponent, MealSlot, PlannedMeal};
use household_core::{
    EvidenceSource, HouseholdController, HouseholdId, LedgerEntry, LedgerEntryId, OutcomeStatus,
    Policy, PolicyId, ReasonCode,
};
use kimatta_storage::rusqlite::TransactionBehavior;
use kimatta_storage::{
    append_ledger_entry, append_ledger_entry_in, apply_plan_and_record, delete_planned_meal_in,
    list_planned_meals, load_planning_snapshot, load_planning_snapshot_in, max_ledger_seq_in,
    save_planned_meal_in, save_policy_in, set_planned_meal_lock_in, Connection, PlannedMealId,
    StorageError, WriteSource,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    /// A veto whose subject the Tier-0 matcher could never match — blank, or carrying no
    /// alphanumeric character at all, since `veto_hit` compares `tokens`, not trimmed text.
    /// `FoodPolicies::from_policies` reports such a row as an unknown policy type for the life
    /// of the database and no command can remove it, so it is refused rather than stored.
    #[error("veto subject has no matchable word")]
    UnmatchableVetoSubject,
    /// A swap addressed a slot the household had locked. Deliberately *not*
    /// `StorageError::LockedPlannedMeal`: that is the **automation** refusal, raised only where
    /// `source == WriteSource::Automation`. This write is `WriteSource::User`, so borrowing it
    /// told the household their own swap was blocked "against automation" and named a raw meal
    /// id while doing so. The text here is the developer surface — the bridge maps this to user
    /// prose rather than forwarding it.
    #[error("swap addresses locked planned meal {meal} at {date} {slot}")]
    SwapOntoLockedSlot {
        meal: String,
        date: String,
        slot: String,
    },
    /// A decision addressed a date the request's own `(today, offset_cycles)` window does not
    /// contain. The ledger row a decision writes is evidence about *that* window, so a
    /// mutation outside it would be recorded against a snapshot that never saw it. Not
    /// reachable from the Cover screen, whose dates come from rendered slots: this guards the
    /// other callers of a `pub fn`. The text is the developer surface — the bridge maps this
    /// to user prose rather than forwarding it.
    #[error("decision date {date} is outside the assessed window {first}..={last}")]
    DecisionOutsideWindow {
        date: String,
        first: String,
        last: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverCycleRequest {
    pub household_id: HouseholdId,
    pub today: CivilDate,
    pub offset_cycles: i32,
    /// `true` writes the plan as `Automation` when its status is not `NeedsAttention`;
    /// `false` only records the proposal.
    pub apply: bool,
    pub params: SearchParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverCycleOutcome {
    pub result: PlanningResult,
    pub ledger_entry_id: LedgerEntryId,
    /// `false` when `apply` was not asked, or was asked under `NeedsAttention` — the result
    /// still carries the slots and requests the UI needs to say why.
    pub applied: bool,
    pub changed_slots: u32,
}

pub const ACTION_APPLY: &str = "apply";
pub const ACTION_PROPOSE: &str = "propose";
/// `apply` was asked for and the controller refused it (`NeedsAttention`). Distinct from
/// `propose`, which is a plain preview: the two carry opposite user intent.
pub const ACTION_DECLINED: &str = "declined";
/// An explicit user decision (swap, veto, reviewed marker) correcting the controller's prior
/// word; the payload's `corrects_seq=` names the row it corrects (MVP-023 follow-up).
pub const ACTION_CORRECT: &str = "correct";

/// `mint_id` supplies every fresh id (the ledger row's and one per proposed slot), so this
/// crate owns no id scheme and no randomness — the bridge keeps `uuid`.
pub fn cover_cycle(
    conn: &mut Connection,
    req: &CoverCycleRequest,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<CoverCycleOutcome, ApplicationError> {
    let snapshot = load_planning_snapshot(conn, &req.household_id, req.today, req.offset_cycles)?;
    let controller = FoodController;
    let prior = controller
        .assess(&snapshot)
        .unwrap_or_else(|never| match never {});
    let result = plan(&snapshot, &req.params);
    let will_apply = req.apply && result.assessment.status != OutcomeStatus::NeedsAttention;
    let mut reason_codes: Vec<ReasonCode> = prior.reason_codes.clone();
    // Rejection codes are already inside `assessment.reason_codes` (`planner::mod` chains them
    // in), in the first-seen order the planner produced.
    for code in result.assessment.reason_codes.iter() {
        if !reason_codes.contains(code) {
            reason_codes.push(code.clone());
        }
    }
    let ledger_entry_id = LedgerEntryId::new(mint_id())?;
    let entry = LedgerEntry {
        id: ledger_entry_id.clone(),
        household_id: req.household_id.clone(),
        controller_id: result.assessment.controller_id.clone(),
        algorithm_version: result.algorithm_version,
        snapshot_hash: result.snapshot_hash.clone(),
        reason_codes,
        // Three tokens, not two: a refused automation request is the single most diagnostic
        // event the ledger exists to capture, and `propose` cannot say it happened.
        selected_action: if will_apply {
            ACTION_APPLY
        } else if req.apply {
            ACTION_DECLINED
        } else {
            ACTION_PROPOSE
        }
        .to_owned(),
        prior_status: prior.status,
        // Nothing was written on the non-apply path, so the resulting state *is* the prior
        // state; claiming the hypothetical status here would put a transition the database
        // never made into a row no writer can correct. The hypothetical is still recorded —
        // `canonical_text` carries its own `status=` line in `payload`.
        resulting_status: if will_apply {
            result.assessment.status
        } else {
            prior.status
        },
        payload: result.canonical_text(),
    };
    let (applied, changed_slots) = if will_apply {
        let ids = result
            .proposed
            .iter()
            .map(|_| PlannedMealId::new(mint_id()))
            .collect::<Result<Vec<_>, _>>()?;
        let changed = apply_plan_and_record(
            conn,
            &result.proposed,
            &ids,
            &entry,
            req.today,
            req.offset_cycles,
        )?;
        (true, changed as u32)
    } else {
        append_ledger_entry(conn, &entry)?;
        (false, 0)
    };
    Ok(CoverCycleOutcome {
        result,
        ledger_entry_id,
        applied,
        changed_slots,
    })
}

impl From<household_core::IdError> for ApplicationError {
    fn from(e: household_core::IdError) -> Self {
        Self::Storage(StorageError::Id(e))
    }
}

/// One explicit user decision from the Cover result view (MVP-024). Coarse by design
/// (invariant 21): the mutation and its evidence row are one command, one transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanDecision {
    /// Replace whatever holds `(date, slot)` — preview slots have no stored row, so absence
    /// is fine — with these components, written **locked** under `WriteSource::User`: the
    /// user explicitly decided the slot, so the next apply must not move it (invariant 18).
    Swap {
        date: CivilDate,
        slot: MealSlot,
        components: Vec<MealComponent>,
    },
    /// "Never suggest X": a `food.hard_veto` policy plus the evidence row. `date`/`slot` say
    /// where the veto was uttered, for the ledger only.
    Veto {
        subject: String,
        date: CivilDate,
        slot: MealSlot,
    },
    /// The household reviewed its (empty) restriction list and confirmed it is complete.
    RestrictionsReviewed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDecisionRequest {
    pub household_id: HouseholdId,
    pub today: CivilDate,
    pub offset_cycles: i32,
    pub decision: PlanDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDecisionOutcome {
    pub ledger_entry_id: LedgerEntryId,
    pub prior_status: OutcomeStatus,
    pub resulting_status: OutcomeStatus,
}

/// A user decision corrects the controller's last word, so its row's action is `correct` and
/// the payload names the household's highest ledger `seq` at the time (0 = nothing to
/// correct). Everything — mutation, re-assessment, ledger append — runs in one IMMEDIATE
/// transaction: a crash can never leave a mutation with no evidence.
pub fn record_decision(
    conn: &mut Connection,
    req: &PlanDecisionRequest,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<PlanDecisionOutcome, ApplicationError> {
    // Before storage is touched, as the bridge's own module doc promises: an unmatchable
    // subject commits a `food.hard_veto` row that reads back as an unknown policy type
    // forever, and no command can remove it. `Policy::new` checks parameter values for control
    // characters but never for emptiness, so this is the only layer that can refuse it. The
    // test is the matcher's own — `is_vetoable`, i.e. `tokens` — not `trim()`: a subject with
    // no alphanumeric character passes a trim and then matches nothing, which would store an
    // inert rule under a dialog that promises the dish is gone for good.
    if let PlanDecision::Veto { subject, .. } = &req.decision {
        if !food_domain::is_vetoable(subject) {
            return Err(ApplicationError::UnmatchableVetoSubject);
        }
    }
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StorageError::from)?;
    let before = load_planning_snapshot_in(&tx, &req.household_id, req.today, req.offset_cycles)?;
    let controller = FoodController;
    let prior_status = controller
        .assess(&before)
        .unwrap_or_else(|never| match never {})
        .status;
    let corrects_seq = max_ledger_seq_in(&tx, &req.household_id)?;
    let description = match &req.decision {
        PlanDecision::Swap {
            date,
            slot,
            components,
        } => {
            // A swap addresses a slot; the ledger row beside it is evidence about the window
            // `before`/`after` describe. Writing outside that window would file a mutation
            // under a snapshot that never saw it. `Veto` and `RestrictionsReviewed` are
            // deliberately not gated — their `date`/`slot` are for the ledger only and
            // address nothing, so a veto uttered while browsing another week is legitimate.
            let dates = before.dates();
            if !dates.contains(date) {
                return Err(ApplicationError::DecisionOutsideWindow {
                    date: format_civil_date(*date),
                    first: format_civil_date(dates[0]),
                    last: format_civil_date(*dates.last().expect("non-empty window")),
                });
            }
            let held = list_planned_meals(&tx, &req.household_id, *date, *date)?;
            if let Some(existing) = held.iter().find(|m| m.slot() == *slot) {
                // `WriteSource::User` may delete a locked row, but a swap must not: the lock
                // is the household's earlier explicit word, and silently replacing it would
                // move something locked (invariant 18). Failing here rolls the whole
                // transaction back.
                if existing.locked() {
                    return Err(ApplicationError::SwapOntoLockedSlot {
                        meal: existing.id().as_str().to_owned(),
                        date: format_civil_date(*date),
                        slot: slot.as_str().to_owned(),
                    });
                }
                delete_planned_meal_in(&tx, &req.household_id, existing.id(), WriteSource::User)?;
            }
            let id = PlannedMealId::new(mint_id())?;
            let occurrence = PlannedMeal::new(
                id.clone(),
                req.household_id.clone(),
                *date,
                *slot,
                components.clone(),
                true,
            )
            .map_err(StorageError::from)?;
            save_planned_meal_in(&tx, &occurrence, WriteSource::User)?;
            // The insert takes the column default (unlocked); the lock is a second statement
            // in the same transaction, as `save_planned_meal`'s contract documents.
            set_planned_meal_lock_in(&tx, &req.household_id, &id, true, WriteSource::User)?;
            format!(
                "decision=swap\ndate={}\nslot={}\ncomponents={}",
                format_civil_date(*date),
                slot.as_str(),
                components_text(components),
            )
        }
        PlanDecision::Veto {
            subject,
            date,
            slot,
        } => {
            let policy = Policy::new(
                PolicyId::new(mint_id())?,
                req.household_id.clone(),
                "food",
                PolicyTypes::HARD_VETO,
                std::collections::BTreeMap::from([("subject".to_owned(), subject.clone())]),
                true,
                EvidenceSource::ExplicitUser,
            )
            .map_err(StorageError::from)?;
            save_policy_in(&tx, &policy)?;
            format!(
                "decision=veto\nsubject={}\ndate={}\nslot={}",
                subject,
                format_civil_date(*date),
                slot.as_str(),
            )
        }
        PlanDecision::RestrictionsReviewed => {
            // Deterministic id: there is one reviewed marker per household, not one per
            // review. `save_policy_in` upserts on id, so a re-review after
            // `save_restrictions` disabled the marker re-enables that row instead of leaving
            // an unbounded run of indistinguishable rows behind it.
            let policy = Policy::new(
                PolicyId::new(format!(
                    "{}:{}",
                    req.household_id.as_str(),
                    PolicyTypes::RESTRICTIONS_REVIEWED
                ))?,
                req.household_id.clone(),
                "food",
                PolicyTypes::RESTRICTIONS_REVIEWED,
                std::collections::BTreeMap::new(),
                true,
                EvidenceSource::ExplicitUser,
            )
            .map_err(StorageError::from)?;
            save_policy_in(&tx, &policy)?;
            "decision=restrictions_reviewed".to_owned()
        }
    };
    let after = load_planning_snapshot_in(&tx, &req.household_id, req.today, req.offset_cycles)?;
    let assessment = controller
        .assess(&after)
        .unwrap_or_else(|never| match never {});
    let ledger_entry_id = LedgerEntryId::new(mint_id())?;
    let entry = LedgerEntry {
        id: ledger_entry_id.clone(),
        household_id: req.household_id.clone(),
        controller_id: assessment.controller_id.clone(),
        algorithm_version: PLANNER_ALGORITHM_VERSION,
        snapshot_hash: after.snapshot_hash(),
        reason_codes: assessment.reason_codes.clone(),
        selected_action: ACTION_CORRECT.to_owned(),
        prior_status,
        resulting_status: assessment.status,
        payload: format!("{description}\ncorrects_seq={corrects_seq}"),
    };
    append_ledger_entry_in(&tx, &entry)?;
    tx.commit().map_err(StorageError::from)?;
    Ok(PlanDecisionOutcome {
        ledger_entry_id,
        prior_status,
        resulting_status: assessment.status,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use food_domain::planner::{CoverageState, FoodPolicies};
    use household_core::{EvidenceSource, Household, HouseholdMember, MemberId, Policy, PolicyId};
    use kimatta_storage::{
        insert_household, list_ledger_entries, list_planned_meals, list_policies, open,
        parse_civil_date, save_member_preferences, save_planned_meal, save_planning_cycle,
        save_policy, save_recipe, save_restrictions, set_planned_meal_lock, HouseholdRestrictions,
        MealComponent, MealScope, MealSlot, MemberPreference, MemberPreferences, PlannedMeal,
        PlanningCycle, ProvenanceKind, Recipe, RecipeId, RecipeProvenance, Restriction,
        RestrictionKind, Sentiment, WriteSource,
    };

    use super::*;

    fn hid(id: &str) -> HouseholdId {
        HouseholdId::new(id).unwrap()
    }

    fn seed(conn: &mut Connection, id: &str) {
        let h = Household {
            id: hid(id),
            name: None,
        };
        let m = HouseholdMember {
            id: MemberId::new(format!("m-{id}")).unwrap(),
            household_id: h.id.clone(),
            display_name: "Me".to_owned(),
        };
        insert_household(conn, &h, std::slice::from_ref(&m)).unwrap();
        save_planning_cycle(
            conn,
            &PlanningCycle::new(hid(id), today(), 2, MealScope::dinner_only()).unwrap(),
        )
        .unwrap();
        save_restrictions(
            conn,
            &hid(id),
            &HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Peanuts)]),
        )
        .unwrap();
        save_member_preferences(
            conn,
            &hid(id),
            &MemberId::new(format!("m-{id}")).unwrap(),
            &MemberPreferences::new([
                MemberPreference::new(Sentiment::Like, "rice").unwrap(),
                MemberPreference::new(Sentiment::Like, "soup").unwrap(),
                MemberPreference::new(Sentiment::Dislike, "olives").unwrap(),
            ]),
        )
        .unwrap();
        save_policy(
            conn,
            &Policy::new(
                PolicyId::new(format!("w-{id}")).unwrap(),
                hid(id),
                "food",
                FoodPolicies::SLOT_WINDOW,
                BTreeMap::from([
                    ("slot".to_owned(), "dinner".to_owned()),
                    ("minutes".to_owned(), "40".to_owned()),
                ]),
                true,
                EvidenceSource::ExplicitUser,
            )
            .unwrap(),
        )
        .unwrap();
        for (rid, title) in [("r-rice", "Rice bowl"), ("r-soup", "Soup")] {
            let r = Recipe::new(
                RecipeId::new(rid).unwrap(),
                hid(id),
                title,
                Some(4),
                Some(15),
                "",
                vec![],
                RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
            )
            .unwrap();
            save_recipe(conn, &r).unwrap();
        }
    }

    fn today() -> CivilDate {
        parse_civil_date("2026-08-29").unwrap()
    }

    fn request(id: &str, apply: bool) -> CoverCycleRequest {
        CoverCycleRequest {
            household_id: hid(id),
            today: today(),
            offset_cycles: 0,
            apply,
            params: SearchParams::default(),
        }
    }

    /// Ids must be unique across a whole database (ledger PK), so every distinct
    /// `cover_cycle` call in a test takes its own prefix.
    fn minter_with(prefix: &'static str) -> impl FnMut() -> String {
        let mut n = 0;
        move || {
            n += 1;
            format!("{prefix}-{n}")
        }
    }

    fn minter() -> impl FnMut() -> String {
        minter_with("id")
    }

    fn rows(conn: &Connection, id: &str) -> Vec<(i64, LedgerEntry)> {
        list_ledger_entries(conn, &hid(id)).unwrap()
    }

    fn raw_rows(conn: &Connection) -> Vec<Vec<String>> {
        let mut stmt = conn
            .prepare("SELECT * FROM controller_ledger ORDER BY seq")
            .unwrap();
        let n = stmt.column_count();
        stmt.query_map([], |r| {
            (0..n)
                .map(|i| {
                    r.get_ref(i).map(|v| match v {
                        rusqlite_value::ValueRef::Null => "NULL".to_owned(),
                        rusqlite_value::ValueRef::Integer(i) => i.to_string(),
                        rusqlite_value::ValueRef::Real(f) => f.to_string(),
                        rusqlite_value::ValueRef::Text(t) => {
                            String::from_utf8_lossy(t).into_owned()
                        }
                        rusqlite_value::ValueRef::Blob(b) => format!("{b:?}"),
                    })
                })
                .collect()
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
    }

    mod rusqlite_value {
        pub use kimatta_storage::rusqlite::types::ValueRef;
    }

    /// AC-2: the ledger row carries version, hash, both statuses and the reason codes.
    #[test]
    fn cover_cycle_records_version_hash_prior_and_resulting_status_and_reason_codes() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let out = cover_cycle(&mut conn, &request("h", false), &mut minter()).unwrap();
        assert_eq!(out.ledger_entry_id.as_str(), "id-1");
        assert!(!out.applied);
        let rows = rows(&conn, "h");
        assert_eq!(rows.len(), 1);
        let e = &rows[0].1;
        assert_eq!(e.algorithm_version, 2);
        assert_eq!(e.snapshot_hash, out.result.snapshot_hash);
        assert_eq!(e.snapshot_hash.len(), 16);
        assert_eq!(e.prior_status, OutcomeStatus::Unresolved);
        // A preview writes nothing, so it records no transition; the hypothetical status the
        // search reached is still in `payload`.
        assert_eq!(e.resulting_status, e.prior_status);
        assert!(e
            .payload
            .contains(&format!("status={}", out.result.assessment.status.as_str())));
        assert_eq!(e.selected_action, ACTION_PROPOSE);
        assert_eq!(e.controller_id, "food");
        assert!(e.reason_codes.iter().any(|c| c.as_str() == "SLOT_COVERED"));
        assert_eq!(e.payload, out.result.canonical_text());
        assert!(e.payload.contains("algorithm_version=2"));
    }

    #[test]
    fn repeat_runs_and_reopen_produce_identical_results_and_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("k.db");
        let first = {
            let mut conn = open(&path).unwrap();
            seed(&mut conn, "h");
            let a = cover_cycle(&mut conn, &request("h", false), &mut minter_with("a")).unwrap();
            let b = cover_cycle(&mut conn, &request("h", false), &mut minter_with("b")).unwrap();
            assert_eq!(a.result, b.result);
            a.result
        };
        let mut conn = open(&path).unwrap();
        let again = cover_cycle(&mut conn, &request("h", false), &mut minter_with("c")).unwrap();
        assert_eq!(again.result, first);
        assert_eq!(again.result.snapshot_hash, first.snapshot_hash);
        assert_eq!(again.result.canonical_text(), first.canonical_text());
        let rows = rows(&conn, "h");
        assert_eq!(rows.len(), 3);
        assert!(rows
            .iter()
            .all(|(_, e)| e.snapshot_hash == first.snapshot_hash));
    }

    /// AC-6: every column of every earlier row is byte-identical after a later run over
    /// changed input.
    #[test]
    fn historical_ledger_rows_are_byte_unchanged_after_a_later_run() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        cover_cycle(&mut conn, &request("h", true), &mut minter_with("a")).unwrap();
        cover_cycle(&mut conn, &request("h", false), &mut minter_with("b")).unwrap();
        let before = raw_rows(&conn);
        assert_eq!(before.len(), 2);
        // Change the input: a new recipe and a lock, then replan and apply.
        let r = Recipe::new(
            RecipeId::new("r-new").unwrap(),
            hid("h"),
            "Rice and soup",
            Some(4),
            Some(10),
            "",
            vec![],
            RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
        )
        .unwrap();
        save_recipe(&mut conn, &r).unwrap();
        let m = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        set_planned_meal_lock(&mut conn, &hid("h"), m[0].id(), true, WriteSource::User).unwrap();
        let later = cover_cycle(&mut conn, &request("h", true), &mut minter_with("c")).unwrap();
        assert_ne!(
            later.result.snapshot_hash,
            rows(&conn, "h")[0].1.snapshot_hash
        );
        let after = raw_rows(&conn);
        assert_eq!(after.len(), 3);
        assert_eq!(&after[..2], &before[..]);
    }

    #[test]
    fn apply_writes_automation_rows_and_ledger_atomically() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let out = cover_cycle(&mut conn, &request("h", true), &mut minter()).unwrap();
        assert!(out.applied);
        assert_eq!(out.changed_slots, 2);
        let meals = list_planned_meals(
            &conn,
            &hid("h"),
            today(),
            parse_civil_date("2026-08-30").unwrap(),
        )
        .unwrap();
        assert_eq!(meals.len(), 2);
        assert!(meals.iter().all(|m| !m.locked()));
        assert_eq!(meals[0].id().as_str(), "id-2");
        assert_eq!(meals[1].id().as_str(), "id-3");
        let logged = rows(&conn, "h");
        assert_eq!(logged.len(), 1);
        assert_eq!(logged[0].1.selected_action, ACTION_APPLY);
        // A second apply over the same state changes nothing and still records.
        let again = cover_cycle(&mut conn, &request("h", true), &mut minter_with("b")).unwrap();
        assert!(again.applied);
        assert_eq!(again.changed_slots, 0);
        assert_eq!(rows_len(&conn), 2);
        assert_eq!(again.result.assessment.status, out.result.assessment.status);
        // Automation rows are visible to a later assess as the existing plan.
        assert_eq!(
            rows(&conn, "h")[1].1.prior_status,
            out.result.assessment.status
        );
    }

    fn rows_len(conn: &Connection) -> usize {
        rows(conn, "h").len()
    }

    #[test]
    fn apply_under_needs_attention_records_and_does_not_write() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        for (i, subject) in ["rice", "soup"].iter().enumerate() {
            save_policy(
                &mut conn,
                &Policy::new(
                    PolicyId::new(format!("veto-{i}")).unwrap(),
                    hid("h"),
                    "food",
                    FoodPolicies::HARD_VETO,
                    BTreeMap::from([("subject".to_owned(), (*subject).to_owned())]),
                    true,
                    EvidenceSource::ExplicitUser,
                )
                .unwrap(),
            )
            .unwrap();
        }
        let out = cover_cycle(&mut conn, &request("h", true), &mut minter()).unwrap();
        assert_eq!(out.result.assessment.status, OutcomeStatus::NeedsAttention);
        assert!(!out.applied);
        assert_eq!(out.changed_slots, 0);
        assert!(out
            .result
            .slots
            .iter()
            .all(|s| s.state == CoverageState::NeedsAttention));
        assert!(!out.result.attention.is_empty());
        assert!(list_planned_meals(
            &conn,
            &hid("h"),
            today(),
            parse_civil_date("2026-08-30").unwrap()
        )
        .unwrap()
        .is_empty());
        let rows = rows(&conn, "h");
        assert_eq!(rows.len(), 1);
        // `apply` was asked for and refused: distinguishable from a plain preview, and it
        // records no transition because nothing was written.
        assert_eq!(rows[0].1.selected_action, ACTION_DECLINED);
        assert_eq!(rows[0].1.resulting_status, rows[0].1.prior_status);
    }

    /// Every row's `prior` must equal the previous row's `resulting`, or the ledger reads as a
    /// state machine that took steps nothing performed. A preview writes nothing, so it must
    /// not claim one — before the fix each preview inserted a `Unresolved → Covered` step and
    /// the next run's `Unresolved` prior contradicted it.
    #[test]
    fn previews_do_not_break_the_ledger_chain() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        cover_cycle(&mut conn, &request("h", false), &mut minter_with("a")).unwrap();
        cover_cycle(&mut conn, &request("h", false), &mut minter_with("b")).unwrap();
        let applied = cover_cycle(&mut conn, &request("h", true), &mut minter_with("c")).unwrap();
        assert!(applied.applied);
        let rows = rows(&conn, "h");
        assert_eq!(rows.len(), 3);
        for pair in rows.windows(2) {
            assert_eq!(
                pair[0].1.resulting_status, pair[1].1.prior_status,
                "{:?} then {:?}",
                pair[0].1, pair[1].1
            );
        }
        // The two previews left the household where they found it; only the apply moved it.
        assert_eq!(rows[0].1.prior_status, rows[0].1.resulting_status);
        assert_eq!(rows[1].1.prior_status, rows[1].1.resulting_status);
        assert_ne!(rows[2].1.prior_status, rows[2].1.resulting_status);
        // The preview still records the status its search reached, in the payload.
        assert!(rows[0].1.payload.contains("status="));
    }

    /// The same chain invariant over a cycle the search fills with placeholders. Every recipe
    /// is disliked, so the view-less frozen/quick fallback wins on tier 2 long before tier 4's
    /// penalty is looked at, and the applied plan grades `TentativelyCovered`. The next run's
    /// `assess` reads those same occurrences back: before the fix it graded them `Covered`,
    /// because the placeholder rule keyed on `candidate.source` and a stored occurrence's
    /// source is `ExistingPlan`. `seed()` never reaches that branch, so
    /// `previews_do_not_break_the_ledger_chain` did not see it.
    #[test]
    fn an_applied_placeholder_plan_does_not_break_the_ledger_chain() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        // Three entries, so `PREFERENCES_SPARSE` stays quiet, and every seeded recipe hit.
        save_member_preferences(
            &mut conn,
            &hid("h"),
            &MemberId::new("m-h").unwrap(),
            &MemberPreferences::new([
                MemberPreference::new(Sentiment::Dislike, "rice").unwrap(),
                MemberPreference::new(Sentiment::Dislike, "soup").unwrap(),
                MemberPreference::new(Sentiment::Dislike, "olives").unwrap(),
            ]),
        )
        .unwrap();
        let applied = cover_cycle(&mut conn, &request("h", true), &mut minter_with("a")).unwrap();
        assert!(applied.applied);
        assert!(
            applied
                .result
                .slots
                .iter()
                .all(|s| s.state == CoverageState::TentativelyCovered),
            "the fixture must plan placeholders: {:?}",
            applied.result.slots
        );
        cover_cycle(&mut conn, &request("h", false), &mut minter_with("b")).unwrap();
        let rows = rows(&conn, "h");
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].1.resulting_status, rows[1].1.prior_status,
            "{:?} then {:?}",
            rows[0].1, rows[1].1
        );
    }

    /// `apply` refused and `apply` never asked are opposite intents; before the fix both wrote
    /// `propose`, so the one event the ledger most needs to capture was unrecoverable.
    #[test]
    fn a_declined_apply_is_distinguishable_from_a_preview() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        for (i, subject) in ["rice", "soup"].iter().enumerate() {
            save_policy(
                &mut conn,
                &Policy::new(
                    PolicyId::new(format!("veto-{i}")).unwrap(),
                    hid("h"),
                    "food",
                    FoodPolicies::HARD_VETO,
                    BTreeMap::from([("subject".to_owned(), (*subject).to_owned())]),
                    true,
                    EvidenceSource::ExplicitUser,
                )
                .unwrap(),
            )
            .unwrap();
        }
        let declined = cover_cycle(&mut conn, &request("h", true), &mut minter_with("a")).unwrap();
        assert!(!declined.applied);
        cover_cycle(&mut conn, &request("h", false), &mut minter_with("b")).unwrap();
        let rows = rows(&conn, "h");
        assert_eq!(rows[0].1.selected_action, ACTION_DECLINED);
        assert_eq!(rows[1].1.selected_action, ACTION_PROPOSE);
        assert_ne!(rows[0].1.selected_action, rows[1].1.selected_action);
    }

    /// The bridge lets a caller tune the search, so a run under non-default params must not
    /// abort. `commitment_horizon_days: 0` makes `near_term` false for every date, which drops
    /// `NEAR_TERM_CHURN` from the reason codes — the divergence the deleted `debug_assert_eq!`
    /// against a default-params `propose()` run would have panicked on in every debug build.
    #[test]
    fn tuned_search_params_do_not_panic() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        // An unlocked existing meal the plan will replace, so `CHURN_PENALTY` fires.
        save_planned_meal(
            &mut conn,
            &PlannedMeal::new(
                kimatta_storage::PlannedMealId::new("existing").unwrap(),
                hid("h"),
                today(),
                MealSlot::Dinner,
                vec![MealComponent::FrozenQuick { note: None }],
                false,
            )
            .unwrap(),
            WriteSource::User,
        )
        .unwrap();
        // Distinct id prefix per call: ledger ids are unique database-wide.
        for (prefix, params) in [
            (
                "t0",
                SearchParams {
                    commitment_horizon_days: 0,
                    ..SearchParams::default()
                },
            ),
            (
                "t1",
                SearchParams {
                    beam_width: 2,
                    candidates_per_slot: 3,
                    commitment_horizon_days: 0,
                },
            ),
        ] {
            let req = CoverCycleRequest {
                params: params.clone(),
                ..request("h", false)
            };
            let out = cover_cycle(&mut conn, &req, &mut minter_with(prefix)).unwrap();
            assert_eq!(out.result.search.beam_width, params.beam_width as u32);
        }
    }

    #[test]
    fn propose_writes_nothing_but_the_ledger() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let out = cover_cycle(&mut conn, &request("h", false), &mut minter()).unwrap();
        assert!(!out.applied);
        assert_eq!(out.result.proposed.len(), 2);
        assert!(list_planned_meals(
            &conn,
            &hid("h"),
            today(),
            parse_civil_date("2026-08-30").unwrap()
        )
        .unwrap()
        .is_empty());
        assert_eq!(rows_len(&conn), 1);
        let err = cover_cycle(&mut conn, &request("ghost", false), &mut minter()).unwrap_err();
        assert!(
            matches!(
                err,
                ApplicationError::Storage(StorageError::NoSuchHousehold(_))
            ),
            "{err:?}"
        );
    }

    /// `assess` runs before planning: with a full existing plan the prior status differs from
    /// the resulting one only through what the plan changed, and is never `Unresolved`.
    #[test]
    fn assess_is_called_before_planning() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let out = cover_cycle(&mut conn, &request("h", false), &mut minter()).unwrap();
        assert_eq!(
            rows(&conn, "h")[0].1.prior_status,
            OutcomeStatus::Unresolved
        );
        assert_ne!(out.result.assessment.status, OutcomeStatus::Unresolved);
        // Plan one dinner by hand with an `Open` and the other left blank: prior Unresolved
        // (a gap), resulting Covered (the gap filled, the Open kept).
        save_planned_meal(
            &mut conn,
            &PlannedMeal::new(
                kimatta_storage::PlannedMealId::new("open").unwrap(),
                hid("h"),
                today(),
                MealSlot::Dinner,
                vec![MealComponent::Open { note: None }],
                false,
            )
            .unwrap(),
            WriteSource::User,
        )
        .unwrap();
        let out = cover_cycle(&mut conn, &request("h", true), &mut minter_with("b")).unwrap();
        let e = &rows(&conn, "h")[1].1;
        assert_eq!(e.prior_status, OutcomeStatus::Unresolved);
        assert_eq!(e.resulting_status, out.result.assessment.status);
        assert_eq!(out.result.slots[0].state, CoverageState::IntentionallyOpen);
        assert_eq!(out.changed_slots, 1);
        // Now everything is planned: the next run's prior equals the last resulting.
        let third = cover_cycle(&mut conn, &request("h", false), &mut minter_with("c")).unwrap();
        assert_eq!(
            rows(&conn, "h")[2].1.prior_status,
            out.result.assessment.status
        );
        assert_eq!(third.result.assessment.status, out.result.assessment.status);
    }

    // --- MVP-024: record_decision -----------------------------------------------------------

    fn swap_request(id: &str, recipe: &str) -> PlanDecisionRequest {
        PlanDecisionRequest {
            household_id: hid(id),
            today: today(),
            offset_cycles: 0,
            decision: PlanDecision::Swap {
                date: today(),
                slot: MealSlot::Dinner,
                components: vec![MealComponent::recipe(RecipeId::new(recipe).unwrap(), None)],
            },
        }
    }

    /// AC-4: a swap is one coarse command — the mutation and its evidence row land together.
    #[test]
    fn swap_writes_a_locked_user_row_and_one_correct_ledger_row() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        cover_cycle(&mut conn, &request("h", true), &mut minter_with("a")).unwrap();
        let out = record_decision(
            &mut conn,
            &swap_request("h", "r-soup"),
            &mut minter_with("b"),
        )
        .unwrap();
        let meals = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        assert_eq!(meals.len(), 1);
        assert!(
            meals[0].locked(),
            "a swapped slot is the user's explicit word"
        );
        assert_eq!(
            meals[0].components(),
            &[MealComponent::recipe(
                RecipeId::new("r-soup").unwrap(),
                None
            )]
        );
        let rows = rows(&conn, "h");
        assert_eq!(rows.len(), 2);
        let (prior_seq, prior_row) = &rows[0];
        let (_, e) = &rows[1];
        assert_eq!(e.selected_action, ACTION_CORRECT);
        assert_eq!(e.id, out.ledger_entry_id);
        assert_eq!(e.prior_status, prior_row.resulting_status, "chain holds");
        assert_eq!(e.prior_status, out.prior_status);
        assert_eq!(e.resulting_status, out.resulting_status);
        assert!(
            e.payload.contains(&format!("corrects_seq={prior_seq}")),
            "{}",
            e.payload
        );
        assert!(e.payload.contains("decision=swap"), "{}", e.payload);
        assert!(e.payload.contains("recipe:r-soup"), "{}", e.payload);
    }

    /// Swap works pre-apply too: preview slots have no stored rows to replace.
    #[test]
    fn swap_into_an_empty_slot_needs_no_existing_row() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let out = record_decision(
            &mut conn,
            &swap_request("h", "r-rice"),
            &mut minter_with("a"),
        )
        .unwrap();
        let meals = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        assert_eq!(meals.len(), 1);
        assert!(meals[0].locked());
        let rows = rows(&conn, "h");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1.selected_action, ACTION_CORRECT);
        // No prior row to correct: the payload says so with seq 0.
        assert!(rows[0].1.payload.contains("corrects_seq=0"));
        assert_eq!(out.prior_status, OutcomeStatus::Unresolved);
    }

    /// Invariant 18: nothing locked moves — a swap onto a locked occupied slot rolls back
    /// whole, evidence row included.
    #[test]
    fn swap_onto_a_locked_slot_fails_typed_and_writes_nothing() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        cover_cycle(&mut conn, &request("h", true), &mut minter_with("a")).unwrap();
        let held = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        set_planned_meal_lock(&mut conn, &hid("h"), held[0].id(), true, WriteSource::User).unwrap();
        let ledger_before = rows(&conn, "h").len();
        let err = record_decision(
            &mut conn,
            &swap_request("h", "r-soup"),
            &mut minter_with("b"),
        )
        .unwrap_err();
        assert!(
            matches!(err, ApplicationError::SwapOntoLockedSlot { .. }),
            "{err:?}"
        );
        let after = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].id(), held[0].id());
        assert_eq!(after[0].components(), held[0].components());
        assert!(after[0].locked());
        assert_eq!(
            rows(&conn, "h").len(),
            ledger_before,
            "no evidence of nothing"
        );
    }

    /// AC-4: a veto is recorded as evidence and the next run Tier-0-rejects the dish.
    #[test]
    fn veto_persists_the_policy_and_the_next_cover_rejects_the_dish() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let out = record_decision(
            &mut conn,
            &PlanDecisionRequest {
                household_id: hid("h"),
                today: today(),
                offset_cycles: 0,
                decision: PlanDecision::Veto {
                    subject: "rice".to_owned(),
                    date: today(),
                    slot: MealSlot::Dinner,
                },
            },
            &mut minter_with("v"),
        )
        .unwrap();
        assert!(!out.ledger_entry_id.as_str().is_empty());
        let policies = list_policies(&conn, &hid("h"), "food").unwrap();
        assert!(policies.iter().any(|p| p.policy_type == "food.hard_veto"
            && p.enabled
            && p.parameters.get("subject").map(String::as_str) == Some("rice")));
        let cover = cover_cycle(&mut conn, &request("h", false), &mut minter_with("c")).unwrap();
        assert!(
            !cover.result.proposed.iter().any(|p| p
                .components
                .iter()
                .any(|c| matches!(c, MealComponent::Recipe { recipe_id, .. }
                    if recipe_id.as_str() == "r-rice"))),
            "the vetoed dish must not be proposed again"
        );
        let rows = rows(&conn, "h");
        assert_eq!(rows[0].1.selected_action, ACTION_CORRECT);
        assert!(rows[0].1.payload.contains("decision=veto"));
        assert!(rows[0].1.payload.contains("subject=rice"));
    }

    #[test]
    fn restrictions_reviewed_persists_and_clears_the_assumption() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_restrictions(&mut conn, &hid("h"), &HouseholdRestrictions::new([])).unwrap();
        let before = cover_cycle(&mut conn, &request("h", false), &mut minter_with("a")).unwrap();
        assert!(before
            .result
            .assessment
            .assumptions
            .iter()
            .any(|a| a.as_str() == "RESTRICTIONS_NOT_CONFIGURED"));
        record_decision(
            &mut conn,
            &PlanDecisionRequest {
                household_id: hid("h"),
                today: today(),
                offset_cycles: 0,
                decision: PlanDecision::RestrictionsReviewed,
            },
            &mut minter_with("m"),
        )
        .unwrap();
        let policies = list_policies(&conn, &hid("h"), "food").unwrap();
        assert!(policies
            .iter()
            .any(|p| p.policy_type == "food.restrictions_reviewed" && p.enabled));
        let after = cover_cycle(&mut conn, &request("h", false), &mut minter_with("b")).unwrap();
        assert!(!after
            .result
            .assessment
            .assumptions
            .iter()
            .any(|a| a.as_str() == "RESTRICTIONS_NOT_CONFIGURED"));
        let rows = rows(&conn, "h");
        assert!(rows[1].1.payload.contains("decision=restrictions_reviewed"));
    }

    /// An unmatchable subject would commit a `food.hard_veto` row that reads back as an unknown
    /// policy type forever, with no command able to remove it. Refused before the
    /// transaction opens, so nothing is written. The last four cases survive a `trim()` but
    /// tokenise to nothing, so the rule they would store could never fire.
    #[test]
    fn an_unmatchable_veto_subject_is_refused_before_anything_is_written() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        for subject in ["", " ", "\t", "\n", "  \r\n ", "🍝", "—", "★", "!!!"] {
            let err = record_decision(
                &mut conn,
                &PlanDecisionRequest {
                    household_id: hid("h"),
                    today: today(),
                    offset_cycles: 0,
                    decision: PlanDecision::Veto {
                        subject: subject.to_owned(),
                        date: today(),
                        slot: MealSlot::Dinner,
                    },
                },
                &mut minter(),
            )
            .unwrap_err();
            assert!(
                matches!(err, ApplicationError::UnmatchableVetoSubject),
                "{subject:?}: {err:?}"
            );
        }
        assert!(list_policies(&conn, &hid("h"), "food")
            .unwrap()
            .iter()
            .all(|p| p.policy_type != "food.hard_veto"));
        assert!(list_ledger_entries(&conn, &hid("h")).unwrap().is_empty());
    }

    /// A subject that is only padded is a real subject; it is stored as the user wrote it,
    /// because the ledger's `subject=` line records their word, and `from_policies` trims at
    /// read time.
    #[test]
    fn a_padded_veto_subject_is_accepted_and_stored_verbatim() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        record_decision(
            &mut conn,
            &PlanDecisionRequest {
                household_id: hid("h"),
                today: today(),
                offset_cycles: 0,
                decision: PlanDecision::Veto {
                    subject: "  olives  ".to_owned(),
                    date: today(),
                    slot: MealSlot::Dinner,
                },
            },
            &mut minter(),
        )
        .unwrap();
        assert!(list_policies(&conn, &hid("h"), "food")
            .unwrap()
            .iter()
            .any(|p| p.policy_type == "food.hard_veto"
                && p.parameters.get("subject").map(String::as_str) == Some("  olives  ")));
    }

    /// The ledger row a decision writes is evidence about the window `before`/`after`
    /// describe, so a swap outside it would file a mutation under a snapshot that never saw
    /// it. Not reachable from the Cover screen — this guards the crate's other callers.
    #[test]
    fn a_swap_outside_the_assessed_window_is_refused_and_writes_nothing() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let far = parse_civil_date("2027-06-01").unwrap();
        let err = record_decision(
            &mut conn,
            &PlanDecisionRequest {
                decision: PlanDecision::Swap {
                    date: far,
                    slot: MealSlot::Dinner,
                    components: vec![MealComponent::recipe(
                        RecipeId::new("r-soup").unwrap(),
                        None,
                    )],
                },
                ..swap_request("h", "r-soup")
            },
            &mut minter(),
        )
        .unwrap_err();
        assert!(
            matches!(err, ApplicationError::DecisionOutsideWindow { .. }),
            "{err:?}"
        );
        assert!(list_planned_meals(&conn, &hid("h"), far, far)
            .unwrap()
            .is_empty());
        assert!(list_ledger_entries(&conn, &hid("h")).unwrap().is_empty());
    }

    /// The window is inclusive at both ends, and only `Swap` is gated: a veto's `date` is for
    /// the ledger only, so one uttered while browsing another week stays legitimate.
    #[test]
    fn the_window_bound_is_inclusive_and_only_swap_is_gated() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        // The seeded cycle is two days anchored on `today()`.
        let last = parse_civil_date("2026-08-30").unwrap();
        let before_first = parse_civil_date("2026-08-28").unwrap();
        let after_last = parse_civil_date("2026-08-31").unwrap();
        // A distinct minter per accepted decision: ledger ids are append-only, so reusing one
        // would fail the second write for a reason that has nothing to do with the window.
        for (prefix, date, accepted) in [
            ("first", today(), true),
            ("last", last, true),
            ("before", before_first, false),
            ("after", after_last, false),
        ] {
            let out = record_decision(
                &mut conn,
                &PlanDecisionRequest {
                    decision: PlanDecision::Swap {
                        date,
                        slot: MealSlot::Dinner,
                        components: vec![MealComponent::recipe(
                            RecipeId::new("r-soup").unwrap(),
                            None,
                        )],
                    },
                    ..swap_request("h", "r-soup")
                },
                &mut minter_with(prefix),
            );
            assert_eq!(out.is_ok(), accepted, "{date:?}: {out:?}");
        }
        record_decision(
            &mut conn,
            &PlanDecisionRequest {
                decision: PlanDecision::Veto {
                    subject: "olives".to_owned(),
                    date: after_last,
                    slot: MealSlot::Dinner,
                },
                ..swap_request("h", "r-soup")
            },
            &mut minter_with("veto"),
        )
        .expect("a veto addresses nothing, so no window contains or excludes it");
    }

    /// One reviewed marker per household, not one per review: the id is derived, so
    /// `save_policy_in`'s upsert flips the same row instead of accumulating rows that carry
    /// no information distinguishing them.
    #[test]
    fn re_reviewing_updates_the_marker_row_rather_than_adding_one() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let reviewed = PlanDecisionRequest {
            household_id: hid("h"),
            today: today(),
            offset_cycles: 0,
            decision: PlanDecision::RestrictionsReviewed,
        };
        record_decision(&mut conn, &reviewed, &mut minter_with("a")).unwrap();
        record_decision(&mut conn, &reviewed, &mut minter_with("b")).unwrap();
        let markers = list_policies(&conn, &hid("h"), "food")
            .unwrap()
            .into_iter()
            .filter(|p| p.policy_type == "food.restrictions_reviewed")
            .count();
        assert_eq!(markers, 1);
    }

    #[test]
    fn record_decision_for_an_unknown_household_is_typed() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = record_decision(&mut conn, &swap_request("ghost", "r-rice"), &mut minter())
            .unwrap_err();
        assert!(
            matches!(
                err,
                ApplicationError::Storage(StorageError::NoSuchHousehold(_))
            ),
            "{err:?}"
        );
    }
}
