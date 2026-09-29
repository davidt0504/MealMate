//! Cover drafts (OPT-007 §4–§6): Another, week alternatives, Lock in, Choose, Undo, Discard,
//! Reconsider and Review edit a persisted draft; Accept alone writes the saved plan, and writes
//! exactly the proposal the household reviewed — no re-planning. Every command is one IMMEDIATE
//! transaction carrying its ledger row and its receipt, so a failure leaves nothing half-done,
//! and a retried request returns its first result instead of running twice.
//!
//! Authority is per slot and explicit. A suggestion is written as `Automation`; a manual pick,
//! a commitment change and a replacement of a saved lock as `User`. A saved lock changes only
//! where the household explicitly unlocked or replaced it on that slot (`replaces_saved_lock`);
//! nothing marks the whole draft as the household's word.

use std::collections::BTreeSet;

use food_domain::planner::draft::{
    self, DraftPayload, DraftSlot, OperationRecord, SavedRecord, SlotOrigin, SlotOutcome,
    SlotOutcomeRecord, DRAFT_FORMAT_VERSION,
};
use food_domain::planner::snapshot::{components_text, text_digest};
use food_domain::planner::{
    assess_slots, CoverageState, PlanningSnapshot, SlotCoverage, PLANNER_ALGORITHM_VERSION,
    SIMILARITY_VERSION,
};
use food_domain::{format_civil_date, CivilDate, MealComponent, MealSlot, PlannedMeal};
use household_core::{
    HouseholdId, LedgerEntry, LedgerEntryId, OutcomeAssessment, OutcomeStatus, ReasonCode,
};
use kimatta_storage::rusqlite::{Transaction, TransactionBehavior};
use kimatta_storage::{
    append_ledger_entry_in, delete_planned_meal_in, expire_drafts_in, insert_draft_in,
    list_unclosed_drafts_in, load_draft_in, load_open_draft_in, load_planning_snapshot_in,
    load_receipt_in, load_recipe, save_planned_meal_in, save_receipt_in, set_planned_meal_lock_in,
    update_draft_in, Connection, DraftReceipt, DraftRow, DraftState, DraftWindow, PlannedMealId,
    StorageError, WriteSource,
};

use crate::ApplicationError;

pub const ACTION_DRAFT_START: &str = "draft_start";
pub const ACTION_DRAFT_ANOTHER: &str = "draft_another";
pub const ACTION_DRAFT_ALTERNATIVES: &str = "draft_alternatives";
pub const ACTION_DRAFT_LOCK: &str = "draft_lock";
pub const ACTION_DRAFT_UNLOCK: &str = "draft_unlock";
pub const ACTION_DRAFT_CHOOSE: &str = "draft_choose";
pub const ACTION_DRAFT_UNDO: &str = "draft_undo";
pub const ACTION_DRAFT_RECONSIDER: &str = "draft_reconsider";
pub const ACTION_DRAFT_DISCARD: &str = "draft_discard";
pub const ACTION_DRAFT_REVIEW: &str = "draft_review";
pub const ACTION_DRAFT_ACCEPT: &str = "draft_accept";
/// A displayed choice names a recipe the household no longer has (archived or deleted). The
/// draft never substitutes one; Another or Choose resolves it.
pub const DRAFT_RECIPE_UNAVAILABLE: &str = "DRAFT_RECIPE_UNAVAILABLE";
/// Expired drafts keep their last display this many days past the window end (§4).
pub const EXPIRED_RETENTION_DAYS: u32 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftContext {
    pub household_id: HouseholdId,
    pub today: CivilDate,
    pub offset_cycles: i32,
    /// `false` in a rollback build (§12): drafts stay readable and discardable, none is
    /// started, edited or accepted — and a pending one never reaches any other apply path.
    pub edits_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftEnvelope {
    pub context: DraftContext,
    pub draft_id: String,
    pub expected_revision: i64,
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewResolution {
    pub date: CivilDate,
    pub slot: MealSlot,
    /// `true` takes the saved meal as it now stands; `false` keeps the draft's meal as a new
    /// explicit choice, validated against current facts.
    pub use_saved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftAction {
    Another {
        date: CivilDate,
        slot: MealSlot,
    },
    /// The week action: every eligible suggestion from today on.
    Alternatives,
    SetCommitment {
        date: CivilDate,
        slot: MealSlot,
        locked: bool,
    },
    Choose {
        date: CivilDate,
        slot: MealSlot,
        components: Vec<MealComponent>,
        /// Required on a locked slot: the household saw that this replaces a commitment.
        explicit_replace: bool,
    },
    Undo,
    Discard,
    /// Clears exclusions for one slot, or the whole draft with `None`. Never regenerates.
    Reconsider {
        slot: Option<(CivilDate, MealSlot)>,
    },
    Review {
        resolutions: Vec<ReviewResolution>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedView {
    pub components: Vec<MealComponent>,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftSlotView {
    pub date: CivilDate,
    pub slot: MealSlot,
    pub components: Vec<MealComponent>,
    pub origin: SlotOrigin,
    pub committed: bool,
    /// The saved meal as it stands now, for side-by-side review and "changed" marks.
    pub saved: Option<SavedView>,
    /// Today or later: past dates are read-only context.
    pub editable: bool,
    /// Differs from the saved meal in components or commitment: Accept would write it.
    pub pending: bool,
    /// The saved meal changed after this draft read it; Review decides.
    pub in_review: bool,
    pub state: CoverageState,
    pub reason_codes: Vec<String>,
    pub outcome: Option<SlotOutcome>,
    /// How many dish identities Another has excluded here (slot-local).
    pub excluded: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptReceipt {
    pub ledger_entry_id: String,
    pub changed_slots: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpiredDraft {
    pub draft_id: String,
    pub anchor: CivilDate,
    pub length_days: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftView {
    pub draft_id: String,
    pub revision: i64,
    pub state: DraftState,
    pub anchor: CivilDate,
    pub length_days: u32,
    pub slots: Vec<DraftSlotView>,
    /// The exact displayed proposal assessed against current facts, dates from today on.
    pub assessment: OutcomeAssessment,
    pub accept_allowed: bool,
    pub undo_available: bool,
    pub pending_changes: bool,
    /// Slots a week request could change right now.
    pub week_targets: u32,
    /// Draft-wide exclusions from week requests.
    pub excluded: u32,
    /// Changes on dates that have since passed, dropped rather than written (§4).
    pub past_changes_dropped: u32,
    pub operation: Option<OperationRecord>,
    pub receipt: Option<AcceptReceipt>,
    pub expired: Vec<ExpiredDraft>,
}

// --- identity ------------------------------------------------------------------------------------

fn window_of(snapshot: &PlanningSnapshot) -> DraftWindow {
    let dates = snapshot.dates();
    DraftWindow {
        anchor: snapshot.anchor,
        length_days: snapshot.length_days,
        scope: snapshot
            .scope
            .slots()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(","),
        end: dates.last().copied().unwrap_or(snapshot.anchor),
    }
}

/// Canonical planner inputs plus what the draft holds fixed and excluded: identical identity,
/// identical proposal (§5). No ids minted here, no clock, no experiment label.
fn planning_identity(snapshot: &PlanningSnapshot, payload: &DraftPayload) -> String {
    let mut text = snapshot.canonical_text();
    text.push_str(&format!(
        "algorithm_version={PLANNER_ALGORITHM_VERSION}\nsimilarity_version={SIMILARITY_VERSION}\n"
    ));
    for s in &payload.slots {
        text.push_str(&format!(
            "draft:{}:{} {} {} [{}]\n",
            s.date,
            s.slot.as_str(),
            s.committed,
            payload_components_text(s),
            s.excluded.iter().cloned().collect::<Vec<_>>().join(" "),
        ));
    }
    text.push_str(&format!(
        "draft_excluded={}\n",
        payload
            .excluded
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    ));
    text_digest(&text)
}

fn payload_components_text(s: &DraftSlot) -> String {
    s.meal()
        .map(|m| components_text(&m))
        .unwrap_or_else(|_| "unreadable".to_owned())
}

/// The freshness precondition (§5 identity 2): the hard facts behind the saved plan plus the
/// full record of every recipe the draft or the saved window references — titles, instructions,
/// servings, lines with quantities and units, archive state. Soft inputs (preferences, pantry,
/// history) and `today` are left out, so marking a pantry item or the date rolling over does not
/// force a review; a ledger append never could. Compared as canonical text, not a digest.
fn review_identity(
    tx: &Transaction<'_>,
    snapshot: &PlanningSnapshot,
    payload: &DraftPayload,
) -> Result<String, ApplicationError> {
    const KEEP: [&str; 8] = [
        "household=",
        "anchor=",
        "length_days=",
        "scope=",
        "members=",
        "restrictions=",
        "policy.",
        "existing:",
    ];
    let mut text: String = snapshot
        .canonical_text()
        .lines()
        .filter(|l| KEEP.iter().any(|k| l.starts_with(k)))
        // Confirming "no restrictions" settles wording only; no choice depends on it.
        .filter(|l| !l.starts_with("policy.restrictions_reviewed="))
        .map(|l| format!("{l}\n"))
        .collect();
    let mut recipe_ids: BTreeSet<String> = BTreeSet::new();
    let mut slugs: BTreeSet<String> = BTreeSet::new();
    let mut note = |c: &MealComponent| match c {
        MealComponent::Recipe { recipe_id, .. } => {
            recipe_ids.insert(recipe_id.as_str().to_owned());
        }
        MealComponent::Freeform { note } => {
            if let Some(slug) = note.strip_prefix("starter:") {
                slugs.insert(slug.to_owned());
            }
        }
        _ => {}
    };
    for s in &payload.slots {
        for c in s.meal()? {
            note(&c);
        }
    }
    for m in &snapshot.existing {
        for c in m.components() {
            note(c);
        }
    }
    for id in recipe_ids {
        let rid = food_domain::RecipeId::new(id.clone())?;
        match load_recipe(tx, &snapshot.household_id, &rid)? {
            Some(record) => text.push_str(&format!("recipe:{id}={record:?}\n")),
            None => text.push_str(&format!("recipe:{id}=missing\n")),
        }
    }
    for slug in slugs {
        let entry = snapshot.starter.iter().find(|s| s.slug == slug);
        text.push_str(&format!("starter:{slug}={entry:?}\n"));
    }
    Ok(text)
}

fn request_digest(operation: &str, env: &DraftEnvelope, action: Option<&DraftAction>) -> String {
    text_digest(&format!(
        "{operation}|{}|{}|{}|{}|{action:?}",
        env.context.household_id.as_str(),
        format_civil_date(env.context.today),
        env.context.offset_cycles,
        env.draft_id,
    ))
}

// --- payload construction ----------------------------------------------------------------------

fn slot_from_saved(date: CivilDate, slot: MealSlot, saved: Option<&PlannedMeal>) -> DraftSlot {
    DraftSlot {
        date: format_civil_date(date),
        slot,
        components: saved
            .map(|m| draft::records(m.components()))
            .unwrap_or_default(),
        origin: if saved.is_some() {
            SlotOrigin::Saved
        } else {
            SlotOrigin::Empty
        },
        committed: saved.is_some_and(PlannedMeal::locked),
        replaces_saved_lock: false,
        base: saved.map(SavedRecord::of),
        excluded: BTreeSet::new(),
        outcome: None,
    }
}

/// The saved meals as the initial display; every empty slot from today on gets one proposal,
/// searched around them. Opening a draft never changes a saved meal (§4 "avoid surprise
/// changes").
fn initial_payload(snapshot: &PlanningSnapshot) -> Result<DraftPayload, ApplicationError> {
    let mut payload = saved_payload(snapshot);
    crate::alternatives::initial(snapshot, &mut payload)?;
    Ok(payload)
}

/// Reverts pending edits on dates before `today`: a past date is context, never a write (§4).
fn drop_past_changes(payload: &mut DraftPayload, today: CivilDate) -> u32 {
    let mut dropped = 0;
    for s in payload.slots.iter_mut() {
        if s.civil_date().is_ok_and(|d| d < today) && slot_pending(s) {
            let base = s.base.clone();
            s.components = base
                .as_ref()
                .map(|b| b.components.clone())
                .unwrap_or_default();
            s.committed = base.as_ref().is_some_and(|b| b.locked);
            s.origin = if base.is_some() {
                SlotOrigin::Saved
            } else {
                SlotOrigin::Empty
            };
            s.replaces_saved_lock = false;
            dropped += 1;
        }
    }
    dropped
}

fn slot_pending(s: &DraftSlot) -> bool {
    match &s.base {
        Some(b) => b.components != s.components || b.locked != s.committed,
        None => !s.components.is_empty(),
    }
}

// --- exact assessment ---------------------------------------------------------------------------

/// The snapshot as it would stand if the draft were accepted: its displayed meals from today on,
/// with Locked in as a lock; saved meals before today. A request-fixed suggestion is never a
/// lock here, so it cannot borrow Tier 0's lock exemption (H3).
fn exact_snapshot(
    snapshot: &PlanningSnapshot,
    payload: &DraftPayload,
) -> Result<PlanningSnapshot, ApplicationError> {
    let mut exact = snapshot.clone();
    let editable: Vec<&DraftSlot> = payload
        .slots
        .iter()
        .filter(|s| s.civil_date().is_ok_and(|d| d >= snapshot.today))
        .collect();
    exact.existing.retain(|m| {
        !editable
            .iter()
            .any(|s| s.civil_date().ok() == Some(m.date()) && s.slot == m.slot())
    });
    for s in editable {
        let components = s.meal()?;
        if components.is_empty() {
            continue;
        }
        let id = s
            .base
            .as_ref()
            .map(|b| b.id.clone())
            .unwrap_or_else(|| format!("draft:{}:{}", s.date, s.slot.as_str()));
        exact.existing.push(
            PlannedMeal::new(
                PlannedMealId::new(id)?,
                snapshot.household_id.clone(),
                s.civil_date()?,
                s.slot,
                components,
                s.committed,
            )
            .map_err(StorageError::from)?,
        );
    }
    exact
        .existing
        .sort_by(|a, b| (a.date(), a.id().as_str()).cmp(&(b.date(), b.id().as_str())));
    Ok(exact)
}

struct Assessed {
    slots: Vec<SlotCoverage>,
    assessment: OutcomeAssessment,
    accept_allowed: bool,
}

fn assess_draft(
    snapshot: &PlanningSnapshot,
    payload: &DraftPayload,
) -> Result<Assessed, ApplicationError> {
    let exact = exact_snapshot(snapshot, payload)?;
    let (mut slots, mut assessment) = assess_slots(&exact);
    // A pending choice naming a recipe the household no longer has is not coverage, whatever
    // Tier 0 made of its fact-free view: the draft keeps it visible and blocks Accept on it.
    for (cov, s) in slots.iter_mut().zip(&payload.slots) {
        if cov.date < snapshot.today || !slot_pending(s) {
            continue;
        }
        let missing = s.meal()?.iter().any(|c| match c {
            MealComponent::Recipe { recipe_id, .. } => {
                !snapshot.recipes.iter().any(|r| &r.id == recipe_id)
            }
            _ => false,
        });
        if missing {
            cov.state = CoverageState::NeedsAttention;
            cov.reason_codes.push(DRAFT_RECIPE_UNAVAILABLE.to_owned());
        }
    }
    let future: Vec<SlotCoverage> = slots
        .iter()
        .filter(|c| c.date >= snapshot.today)
        .cloned()
        .collect();
    let assumptions: Vec<String> = assessment
        .assumptions
        .iter()
        .map(|a| a.as_str().to_owned())
        .collect();
    let status = food_domain::planner::coverage::cycle_status(&future, &assumptions);
    assessment.status = status;
    if slots
        .iter()
        .any(|c| c.reason_codes.iter().any(|r| r == DRAFT_RECIPE_UNAVAILABLE))
    {
        let code = ReasonCode::new(DRAFT_RECIPE_UNAVAILABLE).map_err(StorageError::from)?;
        if !assessment.reason_codes.contains(&code) {
            assessment.reason_codes.push(code);
        }
    }
    let accept_allowed = !future.is_empty()
        && matches!(
            status,
            OutcomeStatus::Covered | OutcomeStatus::TentativelyCovered
        );
    Ok(Assessed {
        slots,
        assessment,
        accept_allowed,
    })
}

// --- views -------------------------------------------------------------------------------------

struct ViewParts<'a> {
    row: &'a DraftRow,
    payload: &'a DraftPayload,
    operation: Option<OperationRecord>,
    receipt: Option<AcceptReceipt>,
    past_changes_dropped: u32,
    expired: Vec<ExpiredDraft>,
}

fn build_view(
    snapshot: &PlanningSnapshot,
    parts: ViewParts<'_>,
) -> Result<DraftView, ApplicationError> {
    let assessed = assess_draft(snapshot, parts.payload)?;
    let open = parts.row.state.is_open();
    let mut slots = Vec::with_capacity(parts.payload.slots.len());
    let mut week_targets = 0;
    for (s, cov) in parts.payload.slots.iter().zip(&assessed.slots) {
        let date = s.civil_date()?;
        let saved = snapshot.existing_at(date, s.slot).map(|m| SavedView {
            components: m.components().to_vec(),
            locked: m.locked(),
        });
        let current_base = snapshot.existing_at(date, s.slot).map(SavedRecord::of);
        let editable = date >= snapshot.today;
        if open && editable && s.is_week_target() {
            week_targets += 1;
        }
        slots.push(DraftSlotView {
            date,
            slot: s.slot,
            components: s.meal()?,
            origin: s.origin,
            committed: s.committed,
            saved,
            editable,
            pending: slot_pending(s),
            in_review: current_base != s.base,
            state: cov.state,
            reason_codes: cov.reason_codes.clone(),
            outcome: s.outcome,
            excluded: s.excluded.len() as u32,
        });
    }
    Ok(DraftView {
        draft_id: parts.row.id.clone(),
        revision: parts.row.revision,
        state: parts.row.state,
        anchor: parts.row.window.anchor,
        length_days: parts.row.window.length_days,
        pending_changes: slots.iter().any(|s| s.pending && s.editable),
        slots,
        accept_allowed: parts.row.state == DraftState::Active && assessed.accept_allowed,
        assessment: assessed.assessment,
        undo_available: open && parts.row.undo_payload.is_some(),
        week_targets,
        excluded: parts.payload.excluded.len() as u32,
        past_changes_dropped: parts.past_changes_dropped,
        operation: parts.operation,
        receipt: parts.receipt,
        expired: parts.expired,
    })
}

/// A closed draft shows the saved plan as it now stands.
fn saved_payload(snapshot: &PlanningSnapshot) -> DraftPayload {
    let mut payload = DraftPayload::default();
    for date in snapshot.dates() {
        for slot in snapshot.scope.slots().iter().copied() {
            payload.slots.push(slot_from_saved(
                date,
                slot,
                snapshot.existing_at(date, slot),
            ));
        }
    }
    payload
}

fn expired_summaries(
    tx: &Transaction<'_>,
    household: &HouseholdId,
) -> Result<Vec<ExpiredDraft>, ApplicationError> {
    Ok(list_unclosed_drafts_in(tx, household)?
        .into_iter()
        .filter(|d| d.state == DraftState::Expired)
        .map(|d| ExpiredDraft {
            draft_id: d.id,
            anchor: d.window.anchor,
            length_days: d.window.length_days,
        })
        .collect())
}

// --- ledger ------------------------------------------------------------------------------------

struct Evidence<'a> {
    action: &'a str,
    draft_id: &'a str,
    revision: i64,
    request_id: &'a str,
    operation: &'a OperationRecord,
    draft_status: OutcomeStatus,
    extra: String,
}

/// One ledger row per household action (§6): the saved state's status on both sides, since a
/// draft edit writes no meal, and the draft's own status in the payload.
fn record(
    tx: &Transaction<'_>,
    snapshot: &PlanningSnapshot,
    evidence: Evidence<'_>,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<LedgerEntryId, ApplicationError> {
    let (_, saved) = assess_slots(snapshot);
    let id = LedgerEntryId::new(mint_id())?;
    let mut payload = format!(
        "draft={}\nrevision={}\nrequest={}\nalgorithm_version={PLANNER_ALGORITHM_VERSION}\n\
         similarity_version={SIMILARITY_VERSION}\ndraft_status={}\n",
        evidence.draft_id,
        evidence.revision,
        evidence.request_id,
        evidence.draft_status.as_str(),
    );
    for s in &evidence.operation.slots {
        payload.push_str(&format!(
            "slot={} {} {:?}\n",
            s.date,
            s.slot.as_str(),
            s.outcome
        ));
    }
    payload.push_str(&evidence.extra);
    append_ledger_entry_in(
        tx,
        &LedgerEntry {
            id: id.clone(),
            household_id: snapshot.household_id.clone(),
            controller_id: saved.controller_id.clone(),
            algorithm_version: PLANNER_ALGORITHM_VERSION,
            snapshot_hash: snapshot.snapshot_hash(),
            reason_codes: saved.reason_codes.clone(),
            selected_action: evidence.action.to_owned(),
            prior_status: saved.status,
            resulting_status: saved.status,
            payload,
        },
    )?;
    Ok(id)
}

// --- commands ------------------------------------------------------------------------------------

fn begin(conn: &mut Connection) -> Result<Transaction<'_>, ApplicationError> {
    Ok(conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StorageError::from)?)
}

fn purge_before(today: CivilDate) -> CivilDate {
    let mut out = today;
    for _ in 0..EXPIRED_RETENTION_DAYS {
        out = out.yesterday().unwrap_or(out);
    }
    out
}

fn overlaps(a: &DraftWindow, b: &DraftWindow) -> bool {
    a.anchor <= b.end && b.anchor <= a.end
}

/// Resumes the open draft for this window without searching, or starts one from the saved
/// meals. A draft whose window was redefined (anchor, length or scope) is set aside as expired:
/// it stays readable and discardable and is never applied.
pub fn open_draft(
    conn: &mut Connection,
    ctx: &DraftContext,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<DraftView, ApplicationError> {
    let tx = begin(conn)?;
    let snapshot = load_planning_snapshot_in(&tx, &ctx.household_id, ctx.today, ctx.offset_cycles)?;
    expire_drafts_in(&tx, &ctx.household_id, ctx.today, purge_before(ctx.today))?;
    let window = window_of(&snapshot);
    let mut operation = None;
    let existing = match load_open_draft_in(&tx, &ctx.household_id, &window)? {
        // A payload no build can read (corrupt, or written by a newer format) is set aside as
        // expired with nothing left to parse, so the window opens fresh and the old tile can
        // still be discarded.
        Some(mut row) if parse(&row).is_err() => {
            let expected = row.revision;
            row.revision += 1;
            row.state = DraftState::Expired;
            row.payload = None;
            row.undo_payload = None;
            update_draft_in(&tx, &row, expected)?;
            None
        }
        other => other,
    };
    if existing.is_none() && !ctx.edits_enabled {
        // Keep the set-aside, if any: the draft is unreadable whatever this build may do.
        tx.commit().map_err(StorageError::from)?;
        return Err(ApplicationError::DraftEditsDisabled);
    }
    let row = match existing {
        Some(mut row) => {
            let payload = parse(&row)?;
            let stale = !versions_current(&row)
                || review_identity(&tx, &snapshot, &payload)? != row.review_identity;
            if row.state == DraftState::Active && stale {
                let expected = row.revision;
                row.revision += 1;
                row.state = DraftState::NeedsReview;
                row.undo_payload = None;
                update_draft_in(&tx, &row, expected)?;
            }
            row
        }
        None => {
            for mut other in list_unclosed_drafts_in(&tx, &ctx.household_id)? {
                if other.state.is_open()
                    && other.window != window
                    && overlaps(&other.window, &window)
                {
                    let expected = other.revision;
                    other.revision += 1;
                    other.state = DraftState::Expired;
                    other.undo_payload = None;
                    update_draft_in(&tx, &other, expected)?;
                }
            }
            let payload = initial_payload(&snapshot)?;
            let row = DraftRow {
                id: mint_id(),
                household_id: ctx.household_id.clone(),
                window,
                revision: 0,
                state: DraftState::Active,
                planning_identity: planning_identity(&snapshot, &payload),
                review_identity: review_identity(&tx, &snapshot, &payload)?,
                format_version: DRAFT_FORMAT_VERSION,
                algorithm_version: PLANNER_ALGORITHM_VERSION,
                similarity_version: SIMILARITY_VERSION,
                payload: Some(payload.to_text()?),
                undo_payload: None,
                created_on: ctx.today,
            };
            insert_draft_in(&tx, &row)?;
            let op = OperationRecord {
                operation: ACTION_DRAFT_START.to_owned(),
                slots: payload
                    .slots
                    .iter()
                    .filter(|s| s.origin == SlotOrigin::Suggested)
                    .map(|s| SlotOutcomeRecord {
                        date: s.date.clone(),
                        slot: s.slot,
                        outcome: SlotOutcome::Changed,
                    })
                    .collect(),
            };
            let status = assess_draft(&snapshot, &payload)?.assessment.status;
            record(
                &tx,
                &snapshot,
                Evidence {
                    action: ACTION_DRAFT_START,
                    draft_id: &row.id,
                    revision: 0,
                    request_id: "",
                    operation: &op,
                    draft_status: status,
                    extra: format!("planning_identity={}\n", row.planning_identity),
                },
                mint_id,
            )?;
            operation = Some(op);
            row
        }
    };
    let mut payload = parse(&row)?;
    let dropped = drop_past_changes(&mut payload, ctx.today);
    let expired = expired_summaries(&tx, &ctx.household_id)?;
    let mut view = build_view(
        &snapshot,
        ViewParts {
            row: &row,
            payload: &payload,
            operation,
            receipt: None,
            past_changes_dropped: dropped,
            expired,
        },
    )?;
    view.accept_allowed &= ctx.edits_enabled;
    tx.commit().map_err(StorageError::from)?;
    Ok(view)
}

/// A read-only look at any draft of the household — the way an expired draft's last choices
/// stay readable (§9). Assessed against the draft's own window only when it is the current one.
pub fn inspect_draft(
    conn: &mut Connection,
    ctx: &DraftContext,
    draft_id: &str,
) -> Result<DraftView, ApplicationError> {
    let tx = begin(conn)?;
    let row = load_draft_in(&tx, &ctx.household_id, draft_id)?
        .ok_or_else(|| ApplicationError::DraftNotFound(draft_id.to_owned()))?;
    let mut snapshot =
        load_planning_snapshot_in(&tx, &ctx.household_id, ctx.today, ctx.offset_cycles)?;
    let payload = match &row.payload {
        Some(_) => parse(&row)?,
        None => saved_payload(&snapshot),
    };
    if window_of(&snapshot) != row.window {
        // Another window: show the choices with the facts that are still true of them, and no
        // saved meals to compare against, since those belong to the current window.
        snapshot.anchor = row.window.anchor;
        snapshot.length_days = row.window.length_days;
        snapshot.existing.clear();
    }
    let expired = expired_summaries(&tx, &ctx.household_id)?;
    let mut view = build_view(
        &snapshot,
        ViewParts {
            row: &row,
            payload: &payload,
            operation: None,
            receipt: None,
            past_changes_dropped: 0,
            expired,
        },
    )?;
    view.accept_allowed &= ctx.edits_enabled;
    tx.commit().map_err(StorageError::from)?;
    Ok(view)
}

fn parse(row: &DraftRow) -> Result<DraftPayload, ApplicationError> {
    let text = row
        .payload
        .as_deref()
        .ok_or_else(|| ApplicationError::DraftClosed(row.state.as_str().to_owned()))?;
    Ok(DraftPayload::from_text(text)?)
}

fn versions_current(row: &DraftRow) -> bool {
    row.format_version == DRAFT_FORMAT_VERSION
        && row.algorithm_version == PLANNER_ALGORITHM_VERSION
        && row.similarity_version == SIMILARITY_VERSION
}

/// Replays a request whose receipt exists: the original outcome and the draft as it now
/// stands. The same id with other content is refused.
fn replay(
    tx: &Transaction<'_>,
    snapshot: &PlanningSnapshot,
    receipt: DraftReceipt,
    digest: &str,
) -> Result<DraftView, ApplicationError> {
    if receipt.request_digest != digest {
        return Err(StorageError::ReceiptConflict {
            request: receipt.request_id,
        }
        .into());
    }
    let row = load_draft_in(tx, &snapshot.household_id, &receipt.draft_id)?
        .ok_or_else(|| ApplicationError::DraftNotFound(receipt.draft_id.clone()))?;
    let payload = match &row.payload {
        Some(_) => parse(&row)?,
        None => saved_payload(snapshot),
    };
    let accepted = receipt
        .ledger_entry_id
        .clone()
        .filter(|_| receipt.state == DraftState::Accepted)
        .map(|id| AcceptReceipt {
            ledger_entry_id: id,
            changed_slots: receipt
                .outcome
                .strip_prefix("changed=")
                .and_then(|n| n.parse().ok())
                .unwrap_or(0),
        });
    let operation = if accepted.is_some() {
        None
    } else {
        Some(OperationRecord::from_text(&receipt.outcome)?)
    };
    build_view(
        snapshot,
        ViewParts {
            row: &row,
            payload: &payload,
            operation,
            receipt: accepted,
            past_changes_dropped: 0,
            expired: expired_summaries(tx, &snapshot.household_id)?,
        },
    )
}

/// The checks every draft command shares, in order: receipt replay, ownership, open state,
/// window, revision, then freshness. A draft found stale moves to `needs_review` in its own
/// committed step and the command is refused — it never runs against facts the household has
/// not seen.
// One value per command, moved straight into a tuple: the size gap costs nothing, and boxing the
// transaction would only add an allocation.
#[allow(clippy::large_enum_variant)]
enum Checked<'a> {
    Replayed(DraftView),
    Ready {
        tx: Transaction<'a>,
        snapshot: PlanningSnapshot,
        row: DraftRow,
        payload: DraftPayload,
        dropped: u32,
    },
}

fn check<'a>(
    conn: &'a mut Connection,
    env: &DraftEnvelope,
    digest: &str,
    allow_review: bool,
    discarding: bool,
) -> Result<Checked<'a>, ApplicationError> {
    let ctx = &env.context;
    let tx = begin(conn)?;
    let snapshot = load_planning_snapshot_in(&tx, &ctx.household_id, ctx.today, ctx.offset_cycles)?;
    if let Some(receipt) = load_receipt_in(&tx, &ctx.household_id, &env.request_id)? {
        let view = replay(&tx, &snapshot, receipt, digest)?;
        tx.commit().map_err(StorageError::from)?;
        return Ok(Checked::Replayed(view));
    }
    let mut row = load_draft_in(&tx, &ctx.household_id, &env.draft_id)?
        .ok_or_else(|| ApplicationError::DraftNotFound(env.draft_id.clone()))?;
    let expired_discard = discarding && row.state == DraftState::Expired;
    if !row.state.is_open() && !expired_discard {
        return Err(ApplicationError::DraftClosed(row.state.as_str().to_owned()));
    }
    if row.revision != env.expected_revision {
        return Err(ApplicationError::StaleDraft {
            expected: env.expected_revision,
            found: row.revision,
        });
    }
    if expired_discard {
        // Discard closes the row whatever it holds: an unreadable payload must not trap it.
        let payload = row
            .payload
            .as_deref()
            .and_then(|text| DraftPayload::from_text(text).ok());
        return Ok(Checked::Ready {
            tx,
            snapshot,
            row,
            payload: payload.unwrap_or_default(),
            dropped: 0,
        });
    }
    if window_of(&snapshot) != row.window {
        return Err(ApplicationError::DraftWindowChanged);
    }
    let mut payload = if discarding {
        parse(&row).unwrap_or_default()
    } else {
        parse(&row)?
    };
    let stale = !versions_current(&row)
        || review_identity(&tx, &snapshot, &payload)? != row.review_identity;
    if row.state == DraftState::Active && stale && !discarding {
        let expected = row.revision;
        row.revision += 1;
        row.state = DraftState::NeedsReview;
        row.undo_payload = None;
        update_draft_in(&tx, &row, expected)?;
        tx.commit().map_err(StorageError::from)?;
        return Err(ApplicationError::DraftNeedsReview);
    }
    if row.state == DraftState::NeedsReview && !allow_review && !discarding {
        return Err(ApplicationError::DraftNeedsReview);
    }
    let dropped = drop_past_changes(&mut payload, ctx.today);
    Ok(Checked::Ready {
        tx,
        snapshot,
        row,
        payload,
        dropped,
    })
}

fn slot_mut(
    payload: &mut DraftPayload,
    date: CivilDate,
    slot: MealSlot,
    today: CivilDate,
) -> Result<&mut DraftSlot, ApplicationError> {
    let index = payload
        .slot_index(date, slot)
        .ok_or_else(|| ApplicationError::SlotNotInDraft {
            date: format_civil_date(date),
            slot: slot.as_str().to_owned(),
        })?;
    if date < today {
        return Err(ApplicationError::SlotInPast {
            date: format_civil_date(date),
            slot: slot.as_str().to_owned(),
        });
    }
    Ok(&mut payload.slots[index])
}

/// A pick must be a real, current dish of this household or a shipped starter, in a shape a
/// planned meal accepts (§5 "must obey current scope/validation").
fn validate_choice(
    snapshot: &PlanningSnapshot,
    components: &[MealComponent],
) -> Result<(), ApplicationError> {
    PlannedMeal::new(
        PlannedMealId::new("validate")?,
        snapshot.household_id.clone(),
        snapshot.anchor,
        MealSlot::Dinner,
        components.to_vec(),
        false,
    )
    .map_err(StorageError::from)?;
    for c in components {
        let known = match c {
            MealComponent::Recipe { recipe_id, .. } => {
                snapshot.recipes.iter().any(|r| &r.id == recipe_id)
            }
            MealComponent::Freeform { note } => match note.strip_prefix("starter:") {
                Some(slug) => {
                    snapshot.starter.iter().any(|s| s.slug == slug)
                        || snapshot
                            .recipes
                            .iter()
                            .any(|r| r.starter_slug.as_deref() == Some(slug))
                }
                None => true,
            },
            _ => true,
        };
        if !known {
            return Err(ApplicationError::InvalidChoice(components_text(
                std::slice::from_ref(c),
            )));
        }
    }
    Ok(())
}

fn outcome_for(s: &DraftSlot, outcome: SlotOutcome) -> SlotOutcomeRecord {
    SlotOutcomeRecord {
        date: s.date.clone(),
        slot: s.slot,
        outcome,
    }
}

/// What an action did: the new payload (unchanged when nothing happened), its record, and
/// whether it is a successful change that takes the undo checkpoint.
struct Applied {
    payload: DraftPayload,
    record: OperationRecord,
    ledger_action: &'static str,
    changed: bool,
    /// `Some` closes (Discard) or restores (Undo) instead of taking a checkpoint.
    close: Option<DraftState>,
    undo: bool,
    reviewed: bool,
}

fn apply_action(
    snapshot: &PlanningSnapshot,
    row: &DraftRow,
    mut payload: DraftPayload,
    action: &DraftAction,
    today: CivilDate,
) -> Result<Applied, ApplicationError> {
    let mut record = OperationRecord::default();
    let applied = |payload, record, ledger_action, changed| Applied {
        payload,
        record,
        ledger_action,
        changed,
        close: None,
        undo: false,
        reviewed: false,
    };
    match action {
        DraftAction::Another { date, slot } => {
            crate::alternatives::another(snapshot, &mut payload, *date, *slot, today)
                .map(|(record, changed)| applied(payload, record, ACTION_DRAFT_ANOTHER, changed))
        }
        DraftAction::Alternatives => crate::alternatives::week(snapshot, &mut payload, today)
            .map(|(record, changed)| applied(payload, record, ACTION_DRAFT_ALTERNATIVES, changed)),
        DraftAction::SetCommitment { date, slot, locked } => {
            record.operation = "set_commitment".to_owned();
            let s = slot_mut(&mut payload, *date, *slot, today)?;
            let changed = s.committed != *locked;
            if changed {
                s.committed = *locked;
                let base_locked = s.base.as_ref().is_some_and(|b| b.locked);
                // Unlocking a saved lock is the household's explicit word for this slot; locking
                // it again with the saved meal still shown restores the saved state exactly.
                if base_locked {
                    s.replaces_saved_lock =
                        !*locked || s.base.as_ref().map(|b| &b.components) != Some(&s.components);
                }
                if !*locked && s.origin == SlotOrigin::Chosen {
                    // An unlocked pick becomes an ordinary suggestion Another may replace; its
                    // exclusion history stays.
                    s.origin = SlotOrigin::Suggested;
                }
                s.outcome = None;
                record.slots.push(outcome_for(s, SlotOutcome::Changed));
            }
            let ledger = if *locked {
                ACTION_DRAFT_LOCK
            } else {
                ACTION_DRAFT_UNLOCK
            };
            Ok(applied(payload, record, ledger, changed))
        }
        DraftAction::Choose {
            date,
            slot,
            components,
            explicit_replace,
        } => {
            record.operation = "choose".to_owned();
            validate_choice(snapshot, components)?;
            let s = slot_mut(&mut payload, *date, *slot, today)?;
            if s.committed && !explicit_replace {
                return Err(ApplicationError::SlotLockedIn {
                    date: s.date.clone(),
                    slot: slot.as_str().to_owned(),
                });
            }
            let base_locked = s.base.as_ref().is_some_and(|b| b.locked);
            s.components = draft::records(components);
            s.origin = SlotOrigin::Chosen;
            s.committed = true;
            s.replaces_saved_lock =
                base_locked && s.base.as_ref().map(|b| &b.components) != Some(&s.components);
            s.outcome = Some(SlotOutcome::Changed);
            record.slots.push(outcome_for(s, SlotOutcome::Changed));
            Ok(applied(payload, record, ACTION_DRAFT_CHOOSE, true))
        }
        DraftAction::Undo => {
            record.operation = "undo".to_owned();
            let previous = row
                .undo_payload
                .as_deref()
                .ok_or(ApplicationError::NothingToUndo)?;
            let restored = DraftPayload::from_text(previous)?;
            let mut out = applied(restored, record, ACTION_DRAFT_UNDO, true);
            out.undo = true;
            Ok(out)
        }
        DraftAction::Discard => {
            record.operation = "discard".to_owned();
            let mut out = applied(payload, record, ACTION_DRAFT_DISCARD, true);
            out.close = Some(DraftState::Discarded);
            Ok(out)
        }
        DraftAction::Reconsider { slot } => {
            record.operation = "reconsider".to_owned();
            let changed = match slot {
                Some((date, slot)) => {
                    let s = slot_mut(&mut payload, *date, *slot, today)?;
                    let had = !s.excluded.is_empty();
                    s.excluded.clear();
                    s.outcome = None;
                    record.slots.push(outcome_for(s, SlotOutcome::Changed));
                    had
                }
                None => {
                    let had = !payload.excluded.is_empty()
                        || payload.slots.iter().any(|s| !s.excluded.is_empty());
                    payload.excluded.clear();
                    for s in payload.slots.iter_mut() {
                        s.excluded.clear();
                        s.outcome = None;
                    }
                    had
                }
            };
            Ok(applied(payload, record, ACTION_DRAFT_RECONSIDER, changed))
        }
        DraftAction::Review { resolutions } => {
            record.operation = "review".to_owned();
            for s in payload.slots.iter_mut() {
                let date = s.civil_date()?;
                let current = snapshot.existing_at(date, s.slot);
                let current_base = current.map(SavedRecord::of);
                if current_base == s.base {
                    continue;
                }
                let resolution = resolutions
                    .iter()
                    .find(|r| r.date == date && r.slot == s.slot)
                    .ok_or_else(|| ApplicationError::ReviewIncomplete {
                        date: s.date.clone(),
                        slot: s.slot.as_str().to_owned(),
                    })?;
                if resolution.use_saved || date < today || !slot_pending(s) {
                    *s = DraftSlot {
                        excluded: std::mem::take(&mut s.excluded),
                        ..slot_from_saved(date, s.slot, current)
                    };
                } else {
                    validate_choice(snapshot, &s.meal()?)?;
                    s.origin = SlotOrigin::Chosen;
                    s.committed = true;
                    s.replaces_saved_lock = current.is_some_and(PlannedMeal::locked);
                    s.base = current_base;
                }
                record.slots.push(outcome_for(s, SlotOutcome::Changed));
            }
            let mut out = applied(payload, record, ACTION_DRAFT_REVIEW, true);
            out.reviewed = true;
            Ok(out)
        }
    }
}

/// Applies one draft command (§5). Draft edits never touch a saved meal; Accept is the only
/// writer, through [`accept_draft`].
pub fn mutate_draft(
    conn: &mut Connection,
    env: &DraftEnvelope,
    action: &DraftAction,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<DraftView, ApplicationError> {
    let discarding = matches!(action, DraftAction::Discard);
    if !env.context.edits_enabled && !discarding {
        return Err(ApplicationError::DraftEditsDisabled);
    }
    let digest = request_digest("mutate", env, Some(action));
    let reviewing = matches!(action, DraftAction::Review { .. });
    let (tx, snapshot, mut row, payload, dropped) =
        match check(conn, env, &digest, reviewing, discarding)? {
            Checked::Replayed(view) => return Ok(view),
            Checked::Ready {
                tx,
                snapshot,
                row,
                payload,
                dropped,
            } => (tx, snapshot, row, payload, dropped),
        };
    if reviewing && row.state != DraftState::NeedsReview {
        return Err(ApplicationError::ReviewNotNeeded);
    }
    let before = payload.to_text()?;
    let applied = apply_action(&snapshot, &row, payload, action, env.context.today)?;
    let expected = row.revision;
    row.revision += 1;
    let mut view_payload = applied.payload.clone();
    if let Some(state) = applied.close {
        row.state = state;
        row.payload = None;
        row.undo_payload = None;
        view_payload = saved_payload(&snapshot);
    } else {
        if applied.undo {
            row.undo_payload = None;
        } else if applied.reviewed {
            row.undo_payload = None;
            row.state = DraftState::Active;
            row.format_version = DRAFT_FORMAT_VERSION;
            row.algorithm_version = PLANNER_ALGORITHM_VERSION;
            row.similarity_version = SIMILARITY_VERSION;
        } else if applied.changed {
            row.undo_payload = Some(before);
        }
        row.payload = Some(applied.payload.to_text()?);
        row.planning_identity = planning_identity(&snapshot, &applied.payload);
        row.review_identity = review_identity(&tx, &snapshot, &applied.payload)?;
    }
    update_draft_in(&tx, &row, expected)?;
    let draft_status = assess_draft(&snapshot, &applied.payload)?.assessment.status;
    let ledger_id = record(
        &tx,
        &snapshot,
        Evidence {
            action: applied.ledger_action,
            draft_id: &row.id,
            revision: row.revision,
            request_id: &env.request_id,
            operation: &applied.record,
            draft_status,
            extra: format!("planning_identity={}\n", row.planning_identity),
        },
        mint_id,
    )?;
    save_receipt_in(
        &tx,
        &DraftReceipt {
            household_id: env.context.household_id.clone(),
            request_id: env.request_id.clone(),
            draft_id: row.id.clone(),
            operation: applied.ledger_action.to_owned(),
            request_digest: digest,
            revision: row.revision,
            state: row.state,
            outcome: applied.record.to_text(),
            ledger_entry_id: Some(ledger_id.as_str().to_owned()),
        },
    )?;
    let expired = expired_summaries(&tx, &env.context.household_id)?;
    let view = build_view(
        &snapshot,
        ViewParts {
            row: &row,
            payload: &view_payload,
            operation: Some(applied.record),
            receipt: None,
            past_changes_dropped: dropped,
            expired,
        },
    )?;
    tx.commit().map_err(StorageError::from)?;
    Ok(view)
}

/// Whether the acting draft is open, at the envelope's revision and fresh against the facts as
/// they stand *before* a policy change — the only case in which the change may carry the draft's
/// identity along with it. A draft already behind some other edit keeps that for Review.
pub(crate) fn acting_draft_fresh(
    tx: &Transaction<'_>,
    env: &DraftEnvelope,
) -> Result<bool, ApplicationError> {
    let ctx = &env.context;
    let Some(row) = load_draft_in(tx, &ctx.household_id, &env.draft_id)? else {
        return Ok(false);
    };
    if row.state != DraftState::Active || row.revision != env.expected_revision {
        return Ok(false);
    }
    let snapshot = load_planning_snapshot_in(tx, &ctx.household_id, ctx.today, ctx.offset_cycles)?;
    if window_of(&snapshot) != row.window || !versions_current(&row) {
        return Ok(false);
    }
    // An unreadable draft is not fresh: the rule is still written, and Cover sets it aside.
    let Ok(payload) = parse(&row) else {
        return Ok(false);
    };
    Ok(review_identity(tx, &snapshot, &payload)? == row.review_identity)
}

/// After a policy change made from this draft (§8): the choices stay, the facts are re-read, and
/// the draft's identity moves with the rule the household just made — so the change does not
/// send its own draft to review. Undo is cleared: it could otherwise restore a choice made
/// before the rule existed as if it had been reviewed against it.
pub(crate) fn refresh_after_policy(
    tx: &Transaction<'_>,
    env: &DraftEnvelope,
    mut row: DraftRow,
) -> Result<DraftView, ApplicationError> {
    let ctx = &env.context;
    let snapshot = load_planning_snapshot_in(tx, &ctx.household_id, ctx.today, ctx.offset_cycles)?;
    if window_of(&snapshot) != row.window {
        return Err(ApplicationError::DraftWindowChanged);
    }
    let mut payload = parse(&row)?;
    let dropped = drop_past_changes(&mut payload, ctx.today);
    let expected = row.revision;
    row.revision += 1;
    row.undo_payload = None;
    row.planning_identity = planning_identity(&snapshot, &payload);
    row.review_identity = review_identity(tx, &snapshot, &payload)?;
    update_draft_in(tx, &row, expected)?;
    build_view(
        &snapshot,
        ViewParts {
            row: &row,
            payload: &payload,
            operation: None,
            receipt: None,
            past_changes_dropped: dropped,
            expired: expired_summaries(tx, &ctx.household_id)?,
        },
    )
}

/// Accept (§5): writes exactly the reviewed proposal, from today on, and closes the draft — in
/// one IMMEDIATE transaction with its ledger row and receipt. No search runs. A slot is skipped
/// only when both its components and its commitment already match the saved row, so a lock-only
/// or unlock-only edit is written; unchanged rows keep their ids.
pub fn accept_draft(
    conn: &mut Connection,
    env: &DraftEnvelope,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<DraftView, ApplicationError> {
    if !env.context.edits_enabled {
        return Err(ApplicationError::DraftEditsDisabled);
    }
    let digest = request_digest("accept", env, None);
    let (tx, snapshot, mut row, payload, _) = match check(conn, env, &digest, false, false)? {
        Checked::Replayed(view) => return Ok(view),
        Checked::Ready {
            tx,
            snapshot,
            row,
            payload,
            dropped,
        } => (tx, snapshot, row, payload, dropped),
    };
    let assessed = assess_draft(&snapshot, &payload)?;
    if !assessed.accept_allowed {
        return Err(ApplicationError::AcceptRefused(
            assessed.assessment.status.as_str().to_owned(),
        ));
    }
    let household = &env.context.household_id;
    let mut changed = 0u32;
    let mut record = OperationRecord {
        operation: "accept".to_owned(),
        slots: Vec::new(),
    };
    for s in &payload.slots {
        let date = s.civil_date()?;
        if date < env.context.today || !slot_pending(s) {
            continue;
        }
        let current = snapshot.existing_at(date, s.slot);
        if current.map(SavedRecord::of) != s.base {
            return Err(ApplicationError::DraftNeedsReview);
        }
        let components = s.meal()?;
        let base_locked = current.is_some_and(PlannedMeal::locked);
        let same_meal = current.is_some_and(|m| m.components() == components.as_slice());
        if base_locked && !s.replaces_saved_lock {
            return Err(ApplicationError::SlotLockedIn {
                date: s.date.clone(),
                slot: s.slot.as_str().to_owned(),
            });
        }
        let id = if same_meal {
            current.expect("same_meal implies a saved row").id().clone()
        } else {
            let source = if base_locked || s.origin == SlotOrigin::Chosen {
                WriteSource::User
            } else {
                WriteSource::Automation
            };
            if let Some(existing) = current {
                // `User` may delete a locked row; that authority reaches here only through
                // `replaces_saved_lock`, checked above for this slot alone.
                delete_planned_meal_in(&tx, household, existing.id(), source)?;
            }
            if components.is_empty() {
                changed += 1;
                record.slots.push(outcome_for(s, SlotOutcome::Changed));
                continue;
            }
            let id = PlannedMealId::new(mint_id())?;
            let meal = PlannedMeal::new(
                id.clone(),
                household.clone(),
                date,
                s.slot,
                components,
                false,
            )
            .map_err(StorageError::from)?;
            save_planned_meal_in(&tx, &meal, source)?;
            id
        };
        let now_locked = same_meal && base_locked;
        if s.committed != now_locked {
            set_planned_meal_lock_in(&tx, household, &id, s.committed, WriteSource::User)?;
        }
        changed += 1;
        record.slots.push(outcome_for(s, SlotOutcome::Changed));
    }
    let after =
        load_planning_snapshot_in(&tx, household, env.context.today, env.context.offset_cycles)?;
    let (_, prior) = assess_slots(&snapshot);
    let (_, resulting) = assess_slots(&after);
    let ledger_id = LedgerEntryId::new(mint_id())?;
    let mut ledger_payload = format!(
        "draft={}\nrevision={}\nrequest={}\nalgorithm_version={PLANNER_ALGORITHM_VERSION}\n\
         similarity_version={SIMILARITY_VERSION}\nplanning_identity={}\nchanged={changed}\n",
        row.id,
        row.revision + 1,
        env.request_id,
        row.planning_identity,
    );
    for s in &record.slots {
        let written = payload
            .slots
            .iter()
            .find(|p| p.date == s.date && p.slot == s.slot)
            .map(|p| format!("{} committed={}", payload_components_text(p), p.committed))
            .unwrap_or_default();
        ledger_payload.push_str(&format!("slot={} {} {written}\n", s.date, s.slot.as_str()));
    }
    append_ledger_entry_in(
        &tx,
        &LedgerEntry {
            id: ledger_id.clone(),
            household_id: household.clone(),
            controller_id: resulting.controller_id.clone(),
            algorithm_version: PLANNER_ALGORITHM_VERSION,
            snapshot_hash: after.snapshot_hash(),
            reason_codes: resulting.reason_codes.clone(),
            selected_action: ACTION_DRAFT_ACCEPT.to_owned(),
            prior_status: prior.status,
            resulting_status: resulting.status,
            payload: ledger_payload,
        },
    )?;
    let expected = row.revision;
    row.revision += 1;
    row.state = DraftState::Accepted;
    row.payload = None;
    row.undo_payload = None;
    update_draft_in(&tx, &row, expected)?;
    save_receipt_in(
        &tx,
        &DraftReceipt {
            household_id: household.clone(),
            request_id: env.request_id.clone(),
            draft_id: row.id.clone(),
            operation: ACTION_DRAFT_ACCEPT.to_owned(),
            request_digest: digest,
            revision: row.revision,
            state: DraftState::Accepted,
            outcome: format!("changed={changed}"),
            ledger_entry_id: Some(ledger_id.as_str().to_owned()),
        },
    )?;
    let expired = expired_summaries(&tx, household)?;
    let view = build_view(
        &after,
        ViewParts {
            row: &row,
            payload: &saved_payload(&after),
            operation: Some(record),
            receipt: Some(AcceptReceipt {
                ledger_entry_id: ledger_id.as_str().to_owned(),
                changed_slots: changed,
            }),
            past_changes_dropped: 0,
            expired,
        },
    )?;
    tx.commit().map_err(StorageError::from)?;
    Ok(view)
}

#[cfg(test)]
#[path = "planning_drafts_tests.rs"]
mod tests;
