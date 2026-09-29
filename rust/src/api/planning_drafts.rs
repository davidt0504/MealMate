//! The Cover draft commands (OPT-007 §5, invariant 21): open, inspect, mutate and accept each
//! cross in one call with one whole-view result; no field-level handles, no SQLite pointer.
//! Every command after `open` carries the database session its view came from, compared under
//! the connection mutex, so a command queued behind a restore or reset is refused rather than
//! run against the replaced file. `today` is caller-supplied (invariant 20).

use kimatta_application::planning_drafts::{
    accept_draft, inspect_draft, mutate_draft, open_draft, DraftAction, DraftContext,
    DraftEnvelope, DraftSlotView, DraftView, ReviewResolution,
};
use kimatta_storage::planner::draft::{SlotOrigin, SlotOutcome};
use kimatta_storage::{format_civil_date, parse_civil_date, DraftState, HouseholdId};
use uuid::Uuid;

use crate::api::error::KimattaError;
use crate::api::planned_meals::{component_from_domain, component_to_domain, MealComponentDto};
use crate::api::planner::{
    state_from_domain, status_from_domain, CoverageStateDto, OutcomeStatusDto, MAX_OFFSET_CYCLES,
    OFFSET_OUT_OF_RANGE,
};
use crate::api::planning::{slot_from_domain, slot_to_domain, MealSlotDto};

/// The §12 rollback switch: a forward-compatible build that must stop draft edits flips this
/// to `false`, keeping drafts readable and discardable and the saved plan untouched. Never a
/// route back to the legacy apply.
const DRAFT_EDITS_ENABLED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftContextDto {
    pub household_id: String,
    /// ISO civil date.
    pub today: String,
    /// 0 is the active window, as `planning_cycle_window` counts.
    pub offset_cycles: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftEnvelopeDto {
    pub context: DraftContextDto,
    /// From the view this command was built on.
    pub database_session: String,
    pub draft_id: String,
    pub expected_revision: i64,
    /// Fresh per user intent; a retry of the same intent reuses it.
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewResolutionDto {
    pub date: String,
    pub slot: MealSlotDto,
    pub use_saved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftActionDto {
    Another {
        date: String,
        slot: MealSlotDto,
    },
    Alternatives,
    SetCommitment {
        date: String,
        slot: MealSlotDto,
        locked: bool,
    },
    Choose {
        date: String,
        slot: MealSlotDto,
        components: Vec<MealComponentDto>,
        explicit_replace: bool,
    },
    Undo,
    Discard,
    /// Both `None` clears every exclusion in the draft.
    Reconsider {
        date: Option<String>,
        slot: Option<MealSlotDto>,
    },
    Review {
        resolutions: Vec<ReviewResolutionDto>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftStateDto {
    Active,
    NeedsReview,
    Accepted,
    Discarded,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotOriginDto {
    Empty,
    Saved,
    Suggested,
    Chosen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotOutcomeDto {
    Changed,
    Exhausted,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedMealDto {
    pub components: Vec<MealComponentDto>,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftSlotDto {
    pub date: String,
    pub slot: MealSlotDto,
    pub components: Vec<MealComponentDto>,
    pub origin: SlotOriginDto,
    pub committed: bool,
    pub saved: Option<SavedMealDto>,
    pub editable: bool,
    pub pending: bool,
    pub in_review: bool,
    pub state: CoverageStateDto,
    /// Engine tokens; the UI maps the ones it knows and never renders a raw one.
    pub reason_codes: Vec<String>,
    pub outcome: Option<SlotOutcomeDto>,
    pub excluded: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotOutcomeRecordDto {
    pub date: String,
    pub slot: MealSlotDto,
    pub outcome: SlotOutcomeDto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftOperationDto {
    pub operation: String,
    pub slots: Vec<SlotOutcomeRecordDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptReceiptDto {
    pub ledger_entry_id: String,
    pub changed_slots: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpiredDraftDto {
    pub draft_id: String,
    pub anchor: String,
    pub length_days: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftViewDto {
    pub database_session: String,
    pub draft_id: String,
    pub revision: i64,
    pub state: DraftStateDto,
    pub anchor: String,
    pub length_days: u32,
    pub slots: Vec<DraftSlotDto>,
    pub status: OutcomeStatusDto,
    pub unresolved_issues: Vec<String>,
    pub assumptions: Vec<String>,
    pub accept_allowed: bool,
    pub undo_available: bool,
    pub pending_changes: bool,
    pub week_targets: u32,
    pub excluded: u32,
    pub past_changes_dropped: u32,
    pub operation: Option<DraftOperationDto>,
    pub receipt: Option<AcceptReceiptDto>,
    pub expired: Vec<ExpiredDraftDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MealExclusionKindDto {
    /// Exactly this dish (`recipe:<id>` / `starter:<slug>`), whatever it is called.
    Dish {
        identity: String,
        title: Option<String>,
        available: bool,
    },
    /// A legacy rule: the words match any title or ingredient line.
    Phrase { subject: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MealExclusionDto {
    pub policy_id: String,
    pub kind: MealExclusionKindDto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddMealExclusionsDto {
    pub context: DraftContextDto,
    /// Dish identities the household confirmed.
    pub dishes: Vec<String>,
    /// The draft the rule was made from, re-assessed and returned with its choices kept.
    pub acting: Option<DraftEnvelopeDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveMealExclusionDto {
    pub context: DraftContextDto,
    pub policy_id: String,
    pub acting: Option<DraftEnvelopeDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MealExclusionOutcomeDto {
    pub exclusions: Vec<MealExclusionDto>,
    pub draft: Option<DraftViewDto>,
}

pub fn list_meal_exclusions(household_id: String) -> Result<Vec<MealExclusionDto>, KimattaError> {
    crate::db::with(|conn| {
        let household = HouseholdId::new(household_id)?;
        Ok(
            kimatta_application::exclusions::list_exclusions(conn, &household)?
                .into_iter()
                .map(exclusion_to_dto)
                .collect(),
        )
    })
}

pub fn add_meal_exclusions(
    request: AddMealExclusionsDto,
) -> Result<MealExclusionOutcomeDto, KimattaError> {
    let token = request.acting.as_ref().map(|a| a.database_session.clone());
    let run = move |conn: &mut kimatta_storage::Connection, session: String| {
        let ctx = context_to_domain(request.context)?;
        let acting = request.acting.map(envelope_to_domain).transpose()?;
        let out = kimatta_application::exclusions::add_dish_exclusions(
            conn,
            &ctx,
            &request.dishes,
            acting.as_ref(),
            &mut || Uuid::new_v4().to_string(),
        )?;
        exclusion_outcome_to_dto(out, session)
    };
    match token {
        Some(token) => crate::db::with_session(&token, run),
        None => crate::db::with_current_session(run),
    }
}

pub fn remove_meal_exclusion(
    request: RemoveMealExclusionDto,
) -> Result<MealExclusionOutcomeDto, KimattaError> {
    let token = request.acting.as_ref().map(|a| a.database_session.clone());
    let run = move |conn: &mut kimatta_storage::Connection, session: String| {
        let ctx = context_to_domain(request.context)?;
        let acting = request.acting.map(envelope_to_domain).transpose()?;
        let out = kimatta_application::exclusions::remove_exclusion(
            conn,
            &ctx,
            &request.policy_id,
            acting.as_ref(),
            &mut || Uuid::new_v4().to_string(),
        )?;
        exclusion_outcome_to_dto(out, session)
    };
    match token {
        Some(token) => crate::db::with_session(&token, run),
        None => crate::db::with_current_session(run),
    }
}

fn exclusion_to_dto(e: kimatta_application::exclusions::ExclusionView) -> MealExclusionDto {
    use kimatta_application::exclusions::ExclusionKind;
    MealExclusionDto {
        policy_id: e.policy_id,
        kind: match e.kind {
            ExclusionKind::Dish {
                identity,
                title,
                available,
            } => MealExclusionKindDto::Dish {
                identity,
                title,
                available,
            },
            ExclusionKind::Phrase { subject } => MealExclusionKindDto::Phrase { subject },
        },
    }
}

fn exclusion_outcome_to_dto(
    out: kimatta_application::exclusions::ExclusionOutcome,
    session: String,
) -> Result<MealExclusionOutcomeDto, KimattaError> {
    Ok(MealExclusionOutcomeDto {
        exclusions: out.exclusions.into_iter().map(exclusion_to_dto).collect(),
        draft: out.draft.map(|v| view_to_dto(v, session)).transpose()?,
    })
}

/// Resumes this window's draft, or starts one from the saved meals. Returns the session every
/// later command must carry.
pub fn open_planning_draft(context: DraftContextDto) -> Result<DraftViewDto, KimattaError> {
    crate::db::with_current_session(|conn, session| open_in(conn, session, context))
}

/// A read-only look at one draft, expired ones included.
pub fn inspect_planning_draft(
    context: DraftContextDto,
    draft_id: String,
) -> Result<DraftViewDto, KimattaError> {
    crate::db::with_current_session(|conn, session| {
        let ctx = context_to_domain(context)?;
        view_to_dto(inspect_draft(conn, &ctx, &draft_id)?, session)
    })
}

pub fn mutate_planning_draft(
    envelope: DraftEnvelopeDto,
    action: DraftActionDto,
) -> Result<DraftViewDto, KimattaError> {
    let token = envelope.database_session.clone();
    crate::db::with_session(&token, |conn, session| {
        mutate_in(conn, session, envelope, action)
    })
}

pub fn accept_planning_draft(envelope: DraftEnvelopeDto) -> Result<DraftViewDto, KimattaError> {
    let token = envelope.database_session.clone();
    crate::db::with_session(&token, |conn, session| accept_in(conn, session, envelope))
}

pub(crate) fn open_in(
    conn: &mut kimatta_storage::Connection,
    session: String,
    context: DraftContextDto,
) -> Result<DraftViewDto, KimattaError> {
    let ctx = context_to_domain(context)?;
    view_to_dto(
        open_draft(conn, &ctx, &mut || Uuid::new_v4().to_string())?,
        session,
    )
}

pub(crate) fn mutate_in(
    conn: &mut kimatta_storage::Connection,
    session: String,
    envelope: DraftEnvelopeDto,
    action: DraftActionDto,
) -> Result<DraftViewDto, KimattaError> {
    let env = envelope_to_domain(envelope)?;
    let action = action_to_domain(action)?;
    view_to_dto(
        mutate_draft(conn, &env, &action, &mut || Uuid::new_v4().to_string())?,
        session,
    )
}

pub(crate) fn accept_in(
    conn: &mut kimatta_storage::Connection,
    session: String,
    envelope: DraftEnvelopeDto,
) -> Result<DraftViewDto, KimattaError> {
    let env = envelope_to_domain(envelope)?;
    view_to_dto(
        accept_draft(conn, &env, &mut || Uuid::new_v4().to_string())?,
        session,
    )
}

fn context_to_domain(c: DraftContextDto) -> Result<DraftContext, KimattaError> {
    if c.offset_cycles > MAX_OFFSET_CYCLES || c.offset_cycles < -MAX_OFFSET_CYCLES {
        return Err(KimattaError::Planning {
            message: OFFSET_OUT_OF_RANGE.to_owned(),
        });
    }
    Ok(DraftContext {
        household_id: HouseholdId::new(c.household_id)?,
        today: parse_civil_date(&c.today)?,
        offset_cycles: c.offset_cycles,
        edits_enabled: DRAFT_EDITS_ENABLED,
    })
}

fn envelope_to_domain(e: DraftEnvelopeDto) -> Result<DraftEnvelope, KimattaError> {
    if e.request_id.trim().is_empty() || e.draft_id.trim().is_empty() {
        return Err(KimattaError::Draft {
            kind: crate::api::error::DraftErrorKind::Invalid,
            message: "that request is missing its draft or request id".to_owned(),
        });
    }
    Ok(DraftEnvelope {
        context: context_to_domain(e.context)?,
        draft_id: e.draft_id,
        expected_revision: e.expected_revision,
        request_id: e.request_id,
    })
}

fn action_to_domain(a: DraftActionDto) -> Result<DraftAction, KimattaError> {
    Ok(match a {
        DraftActionDto::Another { date, slot } => DraftAction::Another {
            date: parse_civil_date(&date)?,
            slot: slot_to_domain(&slot),
        },
        DraftActionDto::Alternatives => DraftAction::Alternatives,
        DraftActionDto::SetCommitment { date, slot, locked } => DraftAction::SetCommitment {
            date: parse_civil_date(&date)?,
            slot: slot_to_domain(&slot),
            locked,
        },
        DraftActionDto::Choose {
            date,
            slot,
            components,
            explicit_replace,
        } => DraftAction::Choose {
            date: parse_civil_date(&date)?,
            slot: slot_to_domain(&slot),
            components: components
                .into_iter()
                .map(component_to_domain)
                .collect::<Result<Vec<_>, _>>()?,
            explicit_replace,
        },
        DraftActionDto::Undo => DraftAction::Undo,
        DraftActionDto::Discard => DraftAction::Discard,
        DraftActionDto::Reconsider { date, slot } => DraftAction::Reconsider {
            slot: match (date, slot) {
                (Some(date), Some(slot)) => Some((parse_civil_date(&date)?, slot_to_domain(&slot))),
                (None, None) => None,
                _ => {
                    return Err(KimattaError::Draft {
                        kind: crate::api::error::DraftErrorKind::Invalid,
                        message: "reconsider needs both a day and a meal, or neither".to_owned(),
                    })
                }
            },
        },
        DraftActionDto::Review { resolutions } => DraftAction::Review {
            resolutions: resolutions
                .into_iter()
                .map(|r| {
                    Ok(ReviewResolution {
                        date: parse_civil_date(&r.date)?,
                        slot: slot_to_domain(&r.slot),
                        use_saved: r.use_saved,
                    })
                })
                .collect::<Result<Vec<_>, KimattaError>>()?,
        },
    })
}

fn state_to_dto(s: DraftState) -> DraftStateDto {
    match s {
        DraftState::Active => DraftStateDto::Active,
        DraftState::NeedsReview => DraftStateDto::NeedsReview,
        DraftState::Accepted => DraftStateDto::Accepted,
        DraftState::Discarded => DraftStateDto::Discarded,
        DraftState::Expired => DraftStateDto::Expired,
    }
}

fn origin_to_dto(o: SlotOrigin) -> SlotOriginDto {
    match o {
        SlotOrigin::Empty => SlotOriginDto::Empty,
        SlotOrigin::Saved => SlotOriginDto::Saved,
        SlotOrigin::Suggested => SlotOriginDto::Suggested,
        SlotOrigin::Chosen => SlotOriginDto::Chosen,
    }
}

fn outcome_to_dto(o: SlotOutcome) -> SlotOutcomeDto {
    match o {
        SlotOutcome::Changed => SlotOutcomeDto::Changed,
        SlotOutcome::Exhausted => SlotOutcomeDto::Exhausted,
        SlotOutcome::Blocked => SlotOutcomeDto::Blocked,
    }
}

fn slot_to_dto(s: DraftSlotView) -> DraftSlotDto {
    DraftSlotDto {
        date: format_civil_date(s.date),
        slot: slot_from_domain(s.slot),
        components: s.components.iter().map(component_from_domain).collect(),
        origin: origin_to_dto(s.origin),
        committed: s.committed,
        saved: s.saved.map(|v| SavedMealDto {
            components: v.components.iter().map(component_from_domain).collect(),
            locked: v.locked,
        }),
        editable: s.editable,
        pending: s.pending,
        in_review: s.in_review,
        state: state_from_domain(s.state),
        reason_codes: s.reason_codes,
        outcome: s.outcome.map(outcome_to_dto),
        excluded: s.excluded,
    }
}

fn view_to_dto(v: DraftView, session: String) -> Result<DraftViewDto, KimattaError> {
    let parse_slot = |slot: kimatta_storage::MealSlot| slot_from_domain(slot);
    Ok(DraftViewDto {
        database_session: session,
        draft_id: v.draft_id,
        revision: v.revision,
        state: state_to_dto(v.state),
        anchor: format_civil_date(v.anchor),
        length_days: v.length_days,
        slots: v.slots.into_iter().map(slot_to_dto).collect(),
        status: status_from_domain(v.assessment.status),
        unresolved_issues: v.assessment.unresolved_issues,
        assumptions: v
            .assessment
            .assumptions
            .iter()
            .map(|a| a.as_str().to_owned())
            .collect(),
        accept_allowed: v.accept_allowed,
        undo_available: v.undo_available,
        pending_changes: v.pending_changes,
        week_targets: v.week_targets,
        excluded: v.excluded,
        past_changes_dropped: v.past_changes_dropped,
        operation: v.operation.map(|o| DraftOperationDto {
            operation: o.operation,
            slots: o
                .slots
                .into_iter()
                .map(|s| SlotOutcomeRecordDto {
                    date: s.date,
                    slot: parse_slot(s.slot),
                    outcome: outcome_to_dto(s.outcome),
                })
                .collect(),
        }),
        receipt: v.receipt.map(|r| AcceptReceiptDto {
            ledger_entry_id: r.ledger_entry_id,
            changed_slots: r.changed_slots,
        }),
        expired: v
            .expired
            .into_iter()
            .map(|e| ExpiredDraftDto {
                draft_id: e.draft_id,
                anchor: format_civil_date(e.anchor),
                length_days: e.length_days,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use kimatta_storage::{
        save_planning_cycle, save_recipe, HouseholdId, MealScope, PlanningCycle, ProvenanceKind,
        Recipe, RecipeId, RecipeProvenance,
    };

    use super::*;
    use crate::api::error::DraftErrorKind;
    use crate::api::health::{export_database, open_database, restore_database};
    use crate::api::household::bootstrap_household;
    use crate::db::TEST_DB_LOCK;

    const MON: &str = "2026-09-28";

    fn seed(db_path: &str) -> String {
        open_database(db_path.to_owned()).unwrap();
        let h = bootstrap_household().unwrap();
        crate::db::with(|conn| {
            let hid = HouseholdId::new(h.id.clone()).unwrap();
            save_planning_cycle(
                conn,
                &PlanningCycle::new(
                    hid.clone(),
                    parse_civil_date(MON).unwrap(),
                    3,
                    MealScope::dinner_only(),
                )
                .unwrap(),
            )?;
            for id in ["a", "b", "c"] {
                save_recipe(
                    conn,
                    &Recipe::new(
                        RecipeId::new(format!("r-{id}")).unwrap(),
                        hid.clone(),
                        format!("Dish {id}"),
                        Some(4),
                        Some(15),
                        "",
                        vec![],
                        RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
                    )
                    .unwrap(),
                )?;
            }
            Ok(())
        })
        .unwrap();
        h.id
    }

    fn context(household: &str) -> DraftContextDto {
        DraftContextDto {
            household_id: household.to_owned(),
            today: MON.to_owned(),
            offset_cycles: 0,
        }
    }

    fn envelope(view: &DraftViewDto, household: &str, request: &str) -> DraftEnvelopeDto {
        DraftEnvelopeDto {
            context: context(household),
            database_session: view.database_session.clone(),
            draft_id: view.draft_id.clone(),
            expected_revision: view.revision,
            request_id: request.to_owned(),
        }
    }

    fn lock_monday() -> DraftActionDto {
        DraftActionDto::SetCommitment {
            date: MON.to_owned(),
            slot: MealSlotDto::Dinner,
            locked: true,
        }
    }

    /// H5 adversarial: restoring an older database that holds the same draft id and revision
    /// cannot let a command built on the replaced database through — the session changed under
    /// the mutex.
    #[test]
    fn a_command_from_before_a_restore_is_refused_even_when_ids_and_revisions_match() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kimatta.db").to_str().unwrap().to_owned();
        let export = dir.path().join("export.db").to_str().unwrap().to_owned();
        let h = seed(&db);
        let view = open_planning_draft(context(&h)).unwrap();
        export_database(export.clone()).unwrap();
        restore_database(export, db.clone()).unwrap();
        // The restored file holds this very draft at this very revision.
        let err = mutate_planning_draft(envelope(&view, &h, "r1"), lock_monday()).unwrap_err();
        assert!(
            matches!(
                err,
                KimattaError::Draft {
                    kind: DraftErrorKind::SessionChanged,
                    ..
                }
            ),
            "{err:?}"
        );
        let err = accept_planning_draft(envelope(&view, &h, "r2")).unwrap_err();
        assert!(matches!(
            err,
            KimattaError::Draft {
                kind: DraftErrorKind::SessionChanged,
                ..
            }
        ));
        // Reopening reads the restored draft under the new session.
        let reopened = open_planning_draft(context(&h)).unwrap();
        assert_ne!(reopened.database_session, view.database_session);
        assert_eq!(reopened.draft_id, view.draft_id);
        let locked = mutate_planning_draft(envelope(&reopened, &h, "r3"), lock_monday()).unwrap();
        assert!(locked.slots[0].committed);
    }

    /// M3 adversarial: a restore that fails leaves the original database and its draft as they
    /// were, and still retires the session, so nothing queued behind it runs unchecked.
    #[test]
    fn a_failed_restore_leaves_the_draft_intact() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kimatta.db").to_str().unwrap().to_owned();
        let junk = dir.path().join("junk.db");
        std::fs::write(&junk, b"not a database").unwrap();
        let h = seed(&db);
        let view = open_planning_draft(context(&h)).unwrap();
        let locked = mutate_planning_draft(envelope(&view, &h, "r1"), lock_monday()).unwrap();
        assert!(restore_database(junk.to_str().unwrap().to_owned(), db.clone()).is_err());
        let err = mutate_planning_draft(envelope(&locked, &h, "r2"), lock_monday()).unwrap_err();
        assert!(matches!(
            err,
            KimattaError::Draft {
                kind: DraftErrorKind::SessionChanged,
                ..
            }
        ));
        let reopened = open_planning_draft(context(&h)).unwrap();
        assert_eq!(reopened.draft_id, locked.draft_id);
        assert_eq!(reopened.revision, locked.revision);
        assert!(reopened.slots[0].committed);
    }

    #[test]
    fn a_draft_round_trips_through_the_bridge_and_accept_returns_a_receipt() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kimatta.db").to_str().unwrap().to_owned();
        let h = seed(&db);
        let view = open_planning_draft(context(&h)).unwrap();
        assert_eq!(view.state, DraftStateDto::Active);
        assert_eq!(view.slots.len(), 3);
        assert!(view.accept_allowed);
        let locked = mutate_planning_draft(envelope(&view, &h, "r1"), lock_monday()).unwrap();
        assert_eq!(locked.revision, view.revision + 1);
        let stale = mutate_planning_draft(envelope(&view, &h, "r2"), lock_monday()).unwrap_err();
        assert!(matches!(
            stale,
            KimattaError::Draft {
                kind: DraftErrorKind::Stale,
                ..
            }
        ));
        let accepted = accept_planning_draft(envelope(&locked, &h, "r3")).unwrap();
        assert_eq!(accepted.state, DraftStateDto::Accepted);
        assert_eq!(accepted.receipt.as_ref().unwrap().changed_slots, 3);
        // A retried Accept returns the same receipt, applying nothing twice.
        let again = accept_planning_draft(envelope(&locked, &h, "r3")).unwrap();
        assert_eq!(again.receipt, accepted.receipt);
        let closed =
            mutate_planning_draft(envelope(&accepted, &h, "r4"), lock_monday()).unwrap_err();
        assert!(matches!(
            closed,
            KimattaError::Draft {
                kind: DraftErrorKind::Closed,
                ..
            }
        ));
    }

    #[test]
    fn exclusions_cross_the_bridge_and_refuse_a_stale_session() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("kimatta.db").to_str().unwrap().to_owned();
        let h = seed(&db);
        let view = open_planning_draft(context(&h)).unwrap();
        let out = add_meal_exclusions(AddMealExclusionsDto {
            context: context(&h),
            dishes: vec!["recipe:r-a".to_owned()],
            acting: Some(envelope(&view, &h, "unused")),
        })
        .unwrap();
        assert_eq!(out.exclusions.len(), 1);
        let draft = out.draft.expect("the acting draft comes back");
        assert_eq!(draft.revision, view.revision + 1);
        assert_eq!(list_meal_exclusions(h.clone()).unwrap(), out.exclusions);
        // The old view's session is still current, but its revision is not: no draft returned.
        let stale = remove_meal_exclusion(RemoveMealExclusionDto {
            context: context(&h),
            policy_id: out.exclusions[0].policy_id.clone(),
            acting: Some(envelope(&view, &h, "unused")),
        })
        .unwrap();
        assert!(stale.exclusions.is_empty());
        assert!(stale.draft.is_none());
        let mut forged = envelope(&draft, &h, "unused");
        forged.database_session = "gone".to_owned();
        let err = add_meal_exclusions(AddMealExclusionsDto {
            context: context(&h),
            dishes: vec!["recipe:r-b".to_owned()],
            acting: Some(forged),
        })
        .unwrap_err();
        assert!(matches!(
            err,
            KimattaError::Draft {
                kind: DraftErrorKind::SessionChanged,
                ..
            }
        ));
        assert!(
            list_meal_exclusions(h).unwrap().is_empty(),
            "nothing was written"
        );
    }

    #[test]
    fn a_half_addressed_reconsider_and_a_blank_request_id_are_invalid() {
        assert!(matches!(
            action_to_domain(DraftActionDto::Reconsider {
                date: Some(MON.to_owned()),
                slot: None,
            }),
            Err(KimattaError::Draft {
                kind: DraftErrorKind::Invalid,
                ..
            })
        ));
        let e = DraftEnvelopeDto {
            context: context("h"),
            database_session: "s".to_owned(),
            draft_id: "d".to_owned(),
            expected_revision: 0,
            request_id: " ".to_owned(),
        };
        assert!(matches!(
            envelope_to_domain(e),
            Err(KimattaError::Draft {
                kind: DraftErrorKind::Invalid,
                ..
            })
        ));
    }
}
