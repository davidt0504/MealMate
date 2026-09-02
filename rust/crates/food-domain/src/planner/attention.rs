//! Attention requests and action proposals (PRD §7.5, §7.6, §11). Produced, never acted on:
//! nothing here notifies, schedules or writes. Bands are hand-built thresholds, not VOI.

use household_core::{
    ActionProposal, AttentionRequest, Band, Confidence, ReasonCode, RequiredAuthority,
    Reversibility, Urgency,
};

use crate::planner::coverage::{
    CoverageState, SlotCoverage, HARD_CONSTRAINT_UNRESOLVED, PLAN_INFEASIBLE, PREFERENCES_SPARSE,
};
use crate::planner::tier0::Feasible;
use crate::planner::{reason, CONTROLLER_ID};
use crate::{format_civil_date, CivilDate, MealSlot};

pub const APPLY_PLAN: &str = "apply_plan";

fn id(kind: &str, date: CivilDate, slot: MealSlot) -> String {
    format!(
        "{CONTROLLER_ID}:{kind}:{}:{}",
        format_civil_date(date),
        slot.as_str()
    )
}

/// One request per `NeedsAttention` slot (High/High/Low; options = the slot's feasible
/// fallbacks), one per chosen recipe with a wording-only restriction match (High), and one
/// `PREFERENCES_SPARSE` (Low/Medium/Low) for the cycle when the assumption holds — a
/// low-urgency question to bundle into a planning moment, not an interruption.
pub fn requests(
    slots: &[SlotCoverage],
    feasible_by_slot: &[Vec<Feasible>],
    chosen: &[Feasible],
    assumptions: &[String],
    horizon_from: CivilDate,
) -> Vec<AttentionRequest> {
    let mut out = Vec::new();
    for (index, s) in slots.iter().enumerate() {
        if s.state == CoverageState::NeedsAttention {
            let options: Vec<String> = feasible_by_slot
                .get(index)
                .map(|fs| fs.iter().map(|f| f.candidate.text()).collect())
                .unwrap_or_default();
            out.push(AttentionRequest {
                id: id("infeasible", s.date, s.slot),
                controller_id: CONTROLLER_ID.to_owned(),
                urgency: Urgency::High,
                deadline: Some(format_civil_date(s.date)),
                decision_benefit_band: Band::High,
                estimated_effort_band: Band::Low,
                options,
                reason_codes: vec![reason(PLAN_INFEASIBLE)],
            });
        }
    }
    for f in chosen {
        if !f.wording_only.is_empty() {
            out.push(AttentionRequest {
                id: id("wording", f.candidate.date, f.candidate.slot),
                controller_id: CONTROLLER_ID.to_owned(),
                urgency: Urgency::High,
                deadline: Some(format_civil_date(f.candidate.date)),
                decision_benefit_band: Band::High,
                estimated_effort_band: Band::Low,
                options: f.wording_only.clone(),
                reason_codes: vec![reason(HARD_CONSTRAINT_UNRESOLVED)],
            });
        }
    }
    if assumptions.iter().any(|a| a == PREFERENCES_SPARSE) {
        out.push(AttentionRequest {
            id: format!(
                "{CONTROLLER_ID}:preferences:{}",
                format_civil_date(horizon_from)
            ),
            controller_id: CONTROLLER_ID.to_owned(),
            urgency: Urgency::Low,
            deadline: None,
            decision_benefit_band: Band::Medium,
            estimated_effort_band: Band::Low,
            options: vec![
                "record a few likes and dislikes per member".to_owned(),
                "keep the conservative default".to_owned(),
            ],
            reason_codes: vec![reason(PREFERENCES_SPARSE)],
        });
    }
    out
}

/// The one proposal a run makes: apply the plan. Reversible (unlocked slots only) and needing
/// a household member; confidence drops to `Low` when preferences are sparse.
pub fn proposals(
    assumptions: &[String],
    reason_codes: &[ReasonCode],
    horizon_from: CivilDate,
) -> Vec<ActionProposal> {
    let sparse = assumptions.iter().any(|a| a == PREFERENCES_SPARSE);
    vec![ActionProposal {
        id: format!(
            "{CONTROLLER_ID}:{APPLY_PLAN}:{}",
            format_civil_date(horizon_from)
        ),
        controller_id: CONTROLLER_ID.to_owned(),
        action_type: APPLY_PLAN.to_owned(),
        expected_benefit_band: Band::High,
        confidence: if sparse {
            Confidence::Low
        } else {
            Confidence::High
        },
        reversibility: Reversibility::Reversible,
        required_authority: RequiredAuthority::HouseholdMember,
        deadline: None,
        reason_codes: reason_codes.to_vec(),
    }]
}
