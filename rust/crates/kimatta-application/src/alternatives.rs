//! Another and the week action on a draft (OPT-007 §6), and the one initial proposal a new draft
//! makes for its empty slots. Each builds the search's roles from the draft — locked-in, past and
//! non-target slots fixed; targets with their exclusions — runs [`alternatives`], and writes the
//! per-slot outcome back into the payload. The whole update lands atomically in the caller's
//! transaction.

use std::collections::BTreeSet;

use food_domain::planner::alternatives::{alternatives, AltOutcome, AltRole, AltSlot};
use food_domain::planner::draft::{
    self, identities, DraftPayload, DraftSlot, OperationRecord, SlotOrigin, SlotOutcome,
    SlotOutcomeRecord,
};
use food_domain::planner::{PlanningSnapshot, SearchParams};
use food_domain::{format_civil_date, CivilDate, MealSlot};

use crate::ApplicationError;

/// Runs one search over the whole payload. `role` decides each slot's part; the returned
/// outcomes are addressed by date and slot.
fn search(
    snapshot: &PlanningSnapshot,
    payload: &DraftPayload,
    role: impl Fn(&DraftSlot) -> AltRole,
) -> Result<Vec<(CivilDate, MealSlot, AltOutcome)>, ApplicationError> {
    let slots = payload
        .slots
        .iter()
        .map(|s| {
            Ok(AltSlot {
                date: s.civil_date()?,
                slot: s.slot,
                display: s.meal()?,
                committed: s.committed,
                role: role(s),
            })
        })
        .collect::<Result<Vec<_>, ApplicationError>>()?;
    Ok(alternatives(snapshot, &slots, &SearchParams::default()).outcomes)
}

fn record(s: &DraftSlot, outcome: SlotOutcome) -> SlotOutcomeRecord {
    SlotOutcomeRecord {
        date: s.date.clone(),
        slot: s.slot,
        outcome,
    }
}

/// Writes a search's outcomes back: a changed target shows its new meal as a suggestion; an
/// exhausted one keeps its meal and says so. Returns whether any slot changed.
fn apply(
    payload: &mut DraftPayload,
    outcomes: Vec<(CivilDate, MealSlot, AltOutcome)>,
    op: &mut OperationRecord,
) -> Result<bool, ApplicationError> {
    let mut changed = false;
    for (date, slot, outcome) in outcomes {
        let index = payload
            .slot_index(date, slot)
            .expect("outcomes are addressed by the payload's own slots");
        let s = &mut payload.slots[index];
        match outcome {
            AltOutcome::Changed(components) => {
                s.components = draft::records(&components);
                s.origin = SlotOrigin::Suggested;
                s.outcome = Some(SlotOutcome::Changed);
                op.slots.push(record(s, SlotOutcome::Changed));
                changed = true;
            }
            AltOutcome::Exhausted => {
                s.outcome = Some(SlotOutcome::Exhausted);
                op.slots.push(record(s, SlotOutcome::Exhausted));
            }
            AltOutcome::Fixed => {}
        }
    }
    Ok(changed)
}

fn clear_outcomes(payload: &mut DraftPayload) {
    for s in payload.slots.iter_mut() {
        s.outcome = None;
    }
}

/// Another (§6): one occurrence, whole — every dish it shows is excluded for this slot, so
/// changing scale or order cannot evade the request. Every other slot is held as displayed.
/// A slot with no dish identity (a fallback, an open slot, a free note) is not guessed at: it
/// is reported blocked and Choose is the way forward.
pub(crate) fn another(
    snapshot: &PlanningSnapshot,
    payload: &mut DraftPayload,
    date: CivilDate,
    slot: MealSlot,
    today: CivilDate,
) -> Result<(OperationRecord, bool), ApplicationError> {
    let mut op = OperationRecord {
        operation: "another".to_owned(),
        slots: Vec::new(),
    };
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
    if payload.slots[index].committed {
        return Err(ApplicationError::SlotLockedIn {
            date: format_civil_date(date),
            slot: slot.as_str().to_owned(),
        });
    }
    clear_outcomes(payload);
    if !payload.slots[index].has_dish_identity() {
        let s = &mut payload.slots[index];
        s.outcome = Some(SlotOutcome::Blocked);
        op.slots.push(record(s, SlotOutcome::Blocked));
        return Ok((op, false));
    }
    let shown = identities(snapshot, &payload.slots[index].meal()?);
    payload.slots[index].excluded.extend(shown);
    let excluded: BTreeSet<String> = payload.slots[index]
        .excluded
        .union(&payload.excluded)
        .cloned()
        .collect();
    let target = payload.slots[index].date.clone();
    let outcomes = search(snapshot, payload, |s| {
        if s.date == target && s.slot == slot {
            AltRole::Target {
                excluded: excluded.clone(),
                fallbacks: false,
            }
        } else {
            AltRole::Fixed
        }
    })?;
    let changed = apply(payload, outcomes, &mut op)?;
    Ok((op, changed))
}

/// The week action (§6): every unlocked recipe suggestion from today on is a target. The dishes
/// they show join the draft-wide exclusions, which bind replaceable targets and never an
/// existing commitment; slot-local exclusions from Another still apply to their own slot.
pub(crate) fn week(
    snapshot: &PlanningSnapshot,
    payload: &mut DraftPayload,
    today: CivilDate,
) -> Result<(OperationRecord, bool), ApplicationError> {
    let mut op = OperationRecord {
        operation: "alternatives".to_owned(),
        slots: Vec::new(),
    };
    clear_outcomes(payload);
    let is_target = |s: &DraftSlot| s.is_week_target() && s.civil_date().is_ok_and(|d| d >= today);
    let mut displaced = BTreeSet::new();
    for s in payload.slots.iter().filter(|s| is_target(s)) {
        displaced.extend(identities(snapshot, &s.meal()?));
    }
    if displaced.is_empty() {
        return Ok((op, false));
    }
    payload.excluded.extend(displaced);
    let wide = payload.excluded.clone();
    let outcomes = search(snapshot, payload, |s| {
        if is_target(s) {
            AltRole::Target {
                excluded: s.excluded.union(&wide).cloned().collect(),
                fallbacks: false,
            }
        } else {
            AltRole::Fixed
        }
    })?;
    let changed = apply(payload, outcomes, &mut op)?;
    Ok((op, changed))
}

/// The one proposal a new draft makes: empty slots from today on, around the saved meals, with
/// the fallback kinds admitted as the planner always has for a first proposal.
pub(crate) fn initial(
    snapshot: &PlanningSnapshot,
    payload: &mut DraftPayload,
) -> Result<(), ApplicationError> {
    let today = snapshot.today;
    let is_target =
        |s: &DraftSlot| s.origin == SlotOrigin::Empty && s.civil_date().is_ok_and(|d| d >= today);
    if !payload.slots.iter().any(is_target) {
        return Ok(());
    }
    let outcomes = search(snapshot, payload, |s| {
        if is_target(s) {
            AltRole::Target {
                excluded: BTreeSet::new(),
                fallbacks: true,
            }
        } else {
            AltRole::Fixed
        }
    })?;
    let mut op = OperationRecord::default();
    apply(payload, outcomes, &mut op)?;
    clear_outcomes(payload);
    Ok(())
}
