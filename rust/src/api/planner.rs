//! The one Cover My Week command (MVP-023, invariant 21): the whole operation crosses in one
//! call and one result DTO; no per-field chatter and no SQLite handle. `today` is a
//! caller-supplied civil date — Rust never reads the clock (invariant 20).

use kimatta_application::{cover_cycle as run_cover_cycle, CoverCycleRequest};
use kimatta_storage::planner::SearchParams;
use kimatta_storage::{
    format_civil_date, ActionProposal, AttentionRequest, Band, Confidence, HouseholdId,
    OutcomeStatus, ReasonCode, RequiredAuthority, Reversibility, Urgency,
};
use uuid::Uuid;

use crate::api::error::KimattaError;
use crate::api::planned_meals::MealComponentDto;
use crate::api::planning::{slot_from_domain, MealSlotDto};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverCycleRequestDto {
    pub household_id: String,
    /// ISO civil date.
    pub today: String,
    /// 0 is the active window, as `planning_cycle_window` counts.
    pub offset_cycles: i32,
    /// `true` also writes the plan (unlocked, changed slots only) when its status is not
    /// `NeedsAttention`; the outcome says whether that happened.
    pub apply: bool,
    /// Absent fields take the recorded defaults (B=8, K=12, horizon 2 days).
    pub beam_width: Option<u32>,
    pub candidates_per_slot: Option<u32>,
    pub commitment_horizon_days: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeStatusDto {
    Unresolved,
    TentativelyCovered,
    Covered,
    NeedsAttention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageStateDto {
    Unresolved,
    TentativelyCovered,
    Covered,
    LockedByUser,
    IntentionallyOpen,
    NeedsAttention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSourceDto {
    ExistingPlan,
    HouseholdRecipe,
    StarterMeal,
    Leftovers,
    DiningOut,
    FrozenQuick,
    IntentionallyOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandDto {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceDto {
    Explicit,
    High,
    Low,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrgencyDto {
    Low,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReversibilityDto {
    Reversible,
    Irreversible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredAuthorityDto {
    None,
    HouseholdMember,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotCoverageDto {
    /// ISO civil date.
    pub date: String,
    pub slot: MealSlotDto,
    pub state: CoverageStateDto,
    pub source: Option<CandidateSourceDto>,
    pub components: Vec<MealComponentDto>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedMealDto {
    pub date: String,
    pub slot: MealSlotDto,
    pub components: Vec<MealComponentDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectionDto {
    pub date: String,
    pub slot: MealSlotDto,
    pub candidate_text: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionProposalDto {
    pub id: String,
    pub controller_id: String,
    pub action_type: String,
    pub expected_benefit_band: BandDto,
    pub confidence: ConfidenceDto,
    pub reversibility: ReversibilityDto,
    pub required_authority: RequiredAuthorityDto,
    pub deadline: Option<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionRequestDto {
    pub id: String,
    pub controller_id: String,
    pub urgency: UrgencyDto,
    pub deadline: Option<String>,
    pub decision_benefit_band: BandDto,
    pub estimated_effort_band: BandDto,
    pub options: Vec<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTraceDto {
    pub beam_width: u32,
    pub candidates_per_slot: u32,
    pub slot_order: String,
    pub states_scored: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningResultDto {
    pub algorithm_version: u32,
    pub snapshot_hash: String,
    pub status: OutcomeStatusDto,
    /// Inclusive ISO bounds of the planned window.
    pub horizon_from: String,
    pub horizon_to: String,
    pub slots: Vec<SlotCoverageDto>,
    pub proposed: Vec<ProposedMealDto>,
    pub rejections: Vec<RejectionDto>,
    pub unresolved_issues: Vec<String>,
    pub assumptions: Vec<String>,
    pub reason_codes: Vec<String>,
    pub proposals: Vec<ActionProposalDto>,
    pub attention: Vec<AttentionRequestDto>,
    pub search: SearchTraceDto,
    /// The six lexicographic tier totals, best plan.
    pub score_tiers: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverCycleOutcomeDto {
    pub result: PlanningResultDto,
    pub ledger_entry_id: String,
    pub applied: bool,
    pub changed_slots: u32,
}

/// Loads, assesses, plans, records — and applies when asked and safe — in one command.
pub fn cover_cycle(request: CoverCycleRequestDto) -> Result<CoverCycleOutcomeDto, KimattaError> {
    crate::db::with(|conn| cover_in(conn, request))
}

fn status_from_domain(s: OutcomeStatus) -> OutcomeStatusDto {
    match s {
        OutcomeStatus::Unresolved => OutcomeStatusDto::Unresolved,
        OutcomeStatus::TentativelyCovered => OutcomeStatusDto::TentativelyCovered,
        OutcomeStatus::Covered => OutcomeStatusDto::Covered,
        OutcomeStatus::NeedsAttention => OutcomeStatusDto::NeedsAttention,
    }
}

fn state_from_domain(s: kimatta_storage::planner::CoverageState) -> CoverageStateDto {
    use kimatta_storage::planner::CoverageState as S;
    match s {
        S::Unresolved => CoverageStateDto::Unresolved,
        S::TentativelyCovered => CoverageStateDto::TentativelyCovered,
        S::Covered => CoverageStateDto::Covered,
        S::LockedByUser => CoverageStateDto::LockedByUser,
        S::IntentionallyOpen => CoverageStateDto::IntentionallyOpen,
        S::NeedsAttention => CoverageStateDto::NeedsAttention,
    }
}

fn source_from_domain(s: kimatta_storage::planner::CandidateSource) -> CandidateSourceDto {
    use kimatta_storage::planner::CandidateSource as S;
    match s {
        S::ExistingPlan => CandidateSourceDto::ExistingPlan,
        S::HouseholdRecipe => CandidateSourceDto::HouseholdRecipe,
        S::StarterMeal => CandidateSourceDto::StarterMeal,
        S::Leftovers => CandidateSourceDto::Leftovers,
        S::DiningOut => CandidateSourceDto::DiningOut,
        S::FrozenQuick => CandidateSourceDto::FrozenQuick,
        S::IntentionallyOpen => CandidateSourceDto::IntentionallyOpen,
    }
}

fn band_from_domain(b: Band) -> BandDto {
    match b {
        Band::Low => BandDto::Low,
        Band::Medium => BandDto::Medium,
        Band::High => BandDto::High,
    }
}

fn confidence_from_domain(c: Confidence) -> ConfidenceDto {
    match c {
        Confidence::Explicit => ConfidenceDto::Explicit,
        Confidence::High => ConfidenceDto::High,
        Confidence::Low => ConfidenceDto::Low,
        Confidence::Unknown => ConfidenceDto::Unknown,
    }
}

fn codes(reason_codes: &[ReasonCode]) -> Vec<String> {
    reason_codes.iter().map(|c| c.as_str().to_owned()).collect()
}

fn proposal_from_domain(p: &ActionProposal) -> ActionProposalDto {
    ActionProposalDto {
        id: p.id.clone(),
        controller_id: p.controller_id.clone(),
        action_type: p.action_type.clone(),
        expected_benefit_band: band_from_domain(p.expected_benefit_band),
        confidence: confidence_from_domain(p.confidence),
        reversibility: match p.reversibility {
            Reversibility::Reversible => ReversibilityDto::Reversible,
            Reversibility::Irreversible => ReversibilityDto::Irreversible,
        },
        required_authority: match p.required_authority {
            RequiredAuthority::None => RequiredAuthorityDto::None,
            RequiredAuthority::HouseholdMember => RequiredAuthorityDto::HouseholdMember,
        },
        deadline: p.deadline.clone(),
        reason_codes: codes(&p.reason_codes),
    }
}

fn attention_from_domain(a: &AttentionRequest) -> AttentionRequestDto {
    AttentionRequestDto {
        id: a.id.clone(),
        controller_id: a.controller_id.clone(),
        urgency: match a.urgency {
            Urgency::Low => UrgencyDto::Low,
            Urgency::High => UrgencyDto::High,
        },
        deadline: a.deadline.clone(),
        decision_benefit_band: band_from_domain(a.decision_benefit_band),
        estimated_effort_band: band_from_domain(a.estimated_effort_band),
        options: a.options.clone(),
        reason_codes: codes(&a.reason_codes),
    }
}

fn components_from_domain(components: &[kimatta_storage::MealComponent]) -> Vec<MealComponentDto> {
    components
        .iter()
        .map(crate::api::planned_meals::component_from_domain)
        .collect()
}

fn result_from_domain(r: &kimatta_storage::planner::PlanningResult) -> PlanningResultDto {
    PlanningResultDto {
        algorithm_version: r.algorithm_version,
        snapshot_hash: r.snapshot_hash.clone(),
        status: status_from_domain(r.assessment.status),
        horizon_from: r.assessment.horizon.from.clone(),
        horizon_to: r.assessment.horizon.to.clone(),
        slots: r
            .slots
            .iter()
            .map(|s| SlotCoverageDto {
                date: format_civil_date(s.date),
                slot: slot_from_domain(s.slot),
                state: state_from_domain(s.state),
                source: s.source.map(source_from_domain),
                components: components_from_domain(&s.components),
                reason_codes: s.reason_codes.clone(),
            })
            .collect(),
        proposed: r
            .proposed
            .iter()
            .map(|p| ProposedMealDto {
                date: format_civil_date(p.date),
                slot: slot_from_domain(p.slot),
                components: components_from_domain(&p.components),
            })
            .collect(),
        rejections: r
            .rejections
            .iter()
            // A resolved slot rejects every other candidate with `LOCK_CONFLICT` by design
            // (AC-1), so these scale with `slots × candidates` and say nothing the slot's
            // `locked_by_user` state does not. The domain result keeps them whole and the
            // ledger payload keeps a per-slot count; the bridge does not carry them.
            .filter(|x| x.code != kimatta_storage::planner::tier0::LOCK_CONFLICT)
            .map(|x| RejectionDto {
                date: format_civil_date(x.date),
                slot: slot_from_domain(x.slot),
                candidate_text: x.candidate_text.clone(),
                code: x.code.clone(),
            })
            .collect(),
        unresolved_issues: r.assessment.unresolved_issues.clone(),
        assumptions: codes(&r.assessment.assumptions),
        reason_codes: codes(&r.assessment.reason_codes),
        proposals: r.proposals.iter().map(proposal_from_domain).collect(),
        attention: r.attention.iter().map(attention_from_domain).collect(),
        search: SearchTraceDto {
            beam_width: r.search.beam_width,
            candidates_per_slot: r.search.candidates_per_slot,
            slot_order: r.search.slot_order.clone(),
            states_scored: r.search.states_scored,
        },
        score_tiers: r.score.score.0.to_vec(),
    }
}

/// Split out from the command, as `derive_in` is, so it can be tested with more than one
/// household present. Dates are parsed before storage is touched, so a malformed `today` is
/// a typed `Planning` error.
pub(crate) fn cover_in(
    conn: &mut kimatta_storage::Connection,
    request: CoverCycleRequestDto,
) -> Result<CoverCycleOutcomeDto, KimattaError> {
    let household_id = HouseholdId::new(request.household_id)?;
    let today = kimatta_storage::parse_civil_date(&request.today)?;
    let defaults = SearchParams::default();
    let req = CoverCycleRequest {
        household_id,
        today,
        offset_cycles: request.offset_cycles,
        apply: request.apply,
        // The two search knobs are clamped at both ends: they cross from Dart, and the beam's
        // live set grows by a factor of K per slot until it hits B, each entry a deep clone of
        // the whole partial plan. Unbounded above, `beamWidth: 1_000_000` on a 21-slot cycle
        // reserves 12M tuples before any truncation. At the ceiling on the largest cycle (31
        // days × 3 slots), B=K=16 scores 16 + 92×256 = 23,568 states — 2.66× the B=8/K=12
        // default — and holds 256 states live at the peak slot, a few MB rather than a few
        // hundred.
        //
        // `commitment_horizon_days` carries no bound. It only feeds an `i64` date comparison
        // (`score.rs:103-109`), so there is no overflow or memory path; an absurd value instead
        // makes `near_term` true for every date, applying `NEAR_TERM_CHURN` (-3) uniformly and
        // so effectively freezing the existing plan against change. That is bounded behaviour,
        // not a bounded number, and a fixed ceiling would be wrong: `offset_cycles` lets a cycle
        // sit arbitrarily far from `today`, so the meaningful ceiling is the caller's own offset
        // rather than a day count.
        params: SearchParams {
            beam_width: request
                .beam_width
                .map_or(defaults.beam_width, |b| b.clamp(1, 16) as usize),
            candidates_per_slot: request
                .candidates_per_slot
                .map_or(defaults.candidates_per_slot, |k| k.clamp(1, 16) as usize),
            commitment_horizon_days: request
                .commitment_horizon_days
                .unwrap_or(defaults.commitment_horizon_days),
        },
    };
    let outcome = run_cover_cycle(conn, &req, &mut || Uuid::new_v4().to_string())?;
    Ok(CoverCycleOutcomeDto {
        result: result_from_domain(&outcome.result),
        ledger_entry_id: outcome.ledger_entry_id.as_str().to_owned(),
        applied: outcome.applied,
        changed_slots: outcome.changed_slots,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use kimatta_storage::planner::FoodPolicies;
    use kimatta_storage::{
        insert_household, list_ledger_entries, open, parse_civil_date, save_member_preferences,
        save_planning_cycle, save_policy, save_recipe, save_restrictions, Connection,
        EvidenceSource, Household, HouseholdMember, HouseholdRestrictions, IngredientLine,
        MealScope, MealSlot, MemberId, MemberPreference, MemberPreferences, PlanningCycle, Policy,
        PolicyId, ProvenanceKind, Quantity, Recipe, RecipeId, RecipeProvenance, Restriction,
        RestrictionKind, Sentiment, Unit,
    };

    use super::*;

    fn hid(raw: &str) -> HouseholdId {
        HouseholdId::new(raw).unwrap()
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
            &PlanningCycle::new(
                hid(id),
                parse_civil_date("2026-08-29").unwrap(),
                2,
                MealScope::new([MealSlot::Dinner]).unwrap(),
            )
            .unwrap(),
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
        for (rid, title, line_name) in [
            (format!("r-rice-{id}"), "Rice bowl", "rice"),
            (format!("r-peanut-{id}"), "Peanut noodles", "peanuts"),
        ] {
            let line = IngredientLine::new(
                format!("1 cup {line_name}"),
                line_name,
                None,
                Quantity::Unknown,
                Unit::None,
                None,
                false,
            )
            .unwrap();
            let r = Recipe::new(
                RecipeId::new(&rid).unwrap(),
                hid(id),
                title,
                Some(4),
                Some(15),
                "",
                vec![line],
                RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
            )
            .unwrap();
            save_recipe(conn, &r).unwrap();
        }
    }

    fn open_seeded(ids: &[&str]) -> Connection {
        let mut conn = open(":memory:").unwrap();
        for id in ids {
            seed(&mut conn, id);
        }
        conn
    }

    fn request(id: &str, apply: bool) -> CoverCycleRequestDto {
        CoverCycleRequestDto {
            household_id: id.to_owned(),
            today: "2026-08-29".to_owned(),
            offset_cycles: 0,
            apply,
            beam_width: None,
            candidates_per_slot: None,
            commitment_horizon_days: None,
        }
    }

    #[test]
    fn cover_cycle_maps_every_enum_and_nested_dto() {
        let mut conn = open_seeded(&["h"]);
        let out = cover_in(&mut conn, request("h", false)).unwrap();
        assert!(!out.applied);
        assert_eq!(out.changed_slots, 0);
        assert!(!out.ledger_entry_id.is_empty());
        let r = &out.result;
        assert_eq!(r.algorithm_version, 2);
        assert_eq!(r.snapshot_hash.len(), 16);
        assert_eq!(r.horizon_from, "2026-08-29");
        assert_eq!(r.horizon_to, "2026-08-30");
        assert_eq!(r.status, OutcomeStatusDto::Covered);
        assert_eq!(r.slots.len(), 2);
        assert_eq!(r.slots[0].date, "2026-08-29");
        assert_eq!(r.slots[0].slot, MealSlotDto::Dinner);
        assert_eq!(r.slots[0].state, CoverageStateDto::Covered);
        assert_eq!(r.slots[0].source, Some(CandidateSourceDto::HouseholdRecipe));
        assert_eq!(r.slots[0].components.len(), 1);
        assert_eq!(r.slots[0].components[0].kind, "recipe");
        assert_eq!(
            r.slots[0].components[0].recipe_id.as_deref(),
            Some("r-rice-h")
        );
        assert_eq!(r.proposed.len(), 2);
        assert_eq!(r.proposed[0].date, "2026-08-29");
        // The peanut recipe was rejected at Tier 0 and the rejection crossed whole.
        let rejected = r
            .rejections
            .iter()
            .find(|x| x.candidate_text.contains("r-peanut-h"))
            .expect("peanut rejection crossed");
        assert_eq!(rejected.code, "RESTRICTION_CONFLICT:rules_v1");
        assert_eq!(rejected.slot, MealSlotDto::Dinner);
        assert!(r
            .reason_codes
            .iter()
            .any(|c| c == "RESTRICTION_CONFLICT:rules_v1"));
        assert!(r.assumptions.iter().any(|c| c == "PANTRY_INCOMPLETE"));
        assert!(r
            .unresolved_issues
            .iter()
            .any(|t| t.contains("stock completeness")));
        assert_eq!(r.proposals.len(), 1);
        let p = &r.proposals[0];
        assert_eq!(p.action_type, "apply_plan");
        assert_eq!(p.expected_benefit_band, BandDto::High);
        assert_eq!(p.confidence, ConfidenceDto::High);
        assert_eq!(p.reversibility, ReversibilityDto::Reversible);
        assert_eq!(p.required_authority, RequiredAuthorityDto::HouseholdMember);
        assert!(r.attention.is_empty(), "{:?}", r.attention);
        assert_eq!(r.search.beam_width, 8);
        assert_eq!(r.search.candidates_per_slot, 12);
        assert_eq!(r.search.slot_order, "canonical");
        assert!(r.search.states_scored > 0);
        assert_eq!(r.score_tiers.len(), 6);
        assert_eq!(r.score_tiers[0], 2, "both slots covered");
    }

    /// A resolved slot rejects every other candidate with `LOCK_CONFLICT` by design, so those
    /// rejections scale with `slots × candidates` and say nothing the slot's `locked_by_user`
    /// state does not. They stay in the domain result and as a ledger count; they do not cross.
    #[test]
    fn lock_conflict_rejections_do_not_cross_the_bridge() {
        let mut conn = open_seeded(&["h"]);
        kimatta_storage::save_planned_meal(
            &mut conn,
            &kimatta_storage::PlannedMeal::new(
                kimatta_storage::PlannedMealId::new("locked").unwrap(),
                hid("h"),
                parse_civil_date("2026-08-29").unwrap(),
                MealSlot::Dinner,
                vec![kimatta_storage::MealComponent::recipe(
                    kimatta_storage::RecipeId::new("r-rice-h").unwrap(),
                    None,
                )],
                false,
            )
            .unwrap(),
            kimatta_storage::WriteSource::User,
        )
        .unwrap();
        // `save_planned_meal` does not write the `locked` column; locking is its own operation.
        kimatta_storage::set_planned_meal_lock(
            &mut conn,
            &hid("h"),
            &kimatta_storage::PlannedMealId::new("locked").unwrap(),
            true,
            kimatta_storage::WriteSource::User,
        )
        .unwrap();
        let out = cover_in(&mut conn, request("h", false)).unwrap();
        assert_eq!(out.result.slots[0].state, CoverageStateDto::LockedByUser);
        assert!(
            out.result
                .rejections
                .iter()
                .all(|x| x.code != "LOCK_CONFLICT"),
            "{:?}",
            out.result.rejections
        );
        // The ledger still records how many candidates the lock displaced.
        let rows = list_ledger_entries(&conn, &hid("h")).unwrap();
        assert!(
            rows[0].1.payload.contains("rejected=LOCK_CONFLICT "),
            "{}",
            rows[0].1.payload
        );
        // A rejection that names a dish a check refused still crosses whole.
        assert!(out
            .result
            .rejections
            .iter()
            .any(|x| x.candidate_text.contains("r-peanut-h")));
    }

    #[test]
    fn commands_are_scoped_to_the_named_household() {
        let mut conn = open_seeded(&["h1", "h2"]);
        let out = cover_in(&mut conn, request("h1", true)).unwrap();
        assert!(out.applied);
        assert_eq!(out.changed_slots, 2);
        assert!(out
            .result
            .proposed
            .iter()
            .all(|p| p.components[0].recipe_id.as_deref() == Some("r-rice-h1")));
        // h2's ledger is untouched; h1 holds exactly one row.
        assert_eq!(list_ledger_entries(&conn, &hid("h1")).unwrap().len(), 1);
        assert!(list_ledger_entries(&conn, &hid("h2")).unwrap().is_empty());
        let err = cover_in(&mut conn, request("ghost", false)).unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
    }

    #[test]
    fn a_non_civil_date_is_a_typed_planning_error() {
        let mut conn = open_seeded(&["h"]);
        for raw in ["2026-08-29T23:00:00Z", "20260829", "", "29/08/2026"] {
            let mut req = request("h", false);
            req.today = raw.to_owned();
            let err = cover_in(&mut conn, req).unwrap_err();
            assert!(
                matches!(err, KimattaError::Planning { .. }),
                "{raw:?} gave {err:?}"
            );
        }
        assert!(list_ledger_entries(&conn, &hid("h")).unwrap().is_empty());
    }

    /// These knobs cross from Dart untrusted. The beam's live set grows by a factor of K per
    /// slot until it hits B, and each live entry is a deep clone of the whole partial plan, so
    /// an unclamped `beamWidth` reserves `B × K` tuples per slot before any truncation —
    /// hundreds of MB to GB at a caller-supplied million, and an OOM kill on a phone. The
    /// ceiling is exercised over a seven-day, three-slot cycle rather than the two-slot
    /// fixture above, so the state count it bounds (`16 + 20×256` = 5,136) is representative.
    #[test]
    fn search_params_are_clamped_above() {
        let mut conn = open(":memory:").unwrap();
        let h = Household {
            id: hid("big"),
            name: None,
        };
        let m = HouseholdMember {
            id: MemberId::new("m-big").unwrap(),
            household_id: h.id.clone(),
            display_name: "Me".to_owned(),
        };
        insert_household(&mut conn, &h, std::slice::from_ref(&m)).unwrap();
        save_planning_cycle(
            &mut conn,
            &PlanningCycle::new(
                hid("big"),
                parse_civil_date("2026-08-29").unwrap(),
                7,
                MealScope::new([MealSlot::Breakfast, MealSlot::Lunch, MealSlot::Dinner]).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        for i in 0..6 {
            let r = Recipe::new(
                RecipeId::new(format!("r-{i}")).unwrap(),
                hid("big"),
                format!("Dish {i}"),
                Some(4),
                Some(15),
                "",
                vec![],
                RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
            )
            .unwrap();
            save_recipe(&mut conn, &r).unwrap();
        }
        let started = std::time::Instant::now();
        let out = cover_in(
            &mut conn,
            CoverCycleRequestDto {
                household_id: "big".to_owned(),
                today: "2026-08-29".to_owned(),
                offset_cycles: 0,
                apply: false,
                beam_width: Some(1_000_000),
                candidates_per_slot: Some(1_000_000),
                commitment_horizon_days: None,
            },
        )
        .unwrap();
        assert_eq!(out.result.search.beam_width, 16);
        assert_eq!(out.result.search.candidates_per_slot, 16);
        assert_eq!(out.result.slots.len(), 21);
        // The point of the ceiling is that the work stays bounded, so pin the wall clock too:
        // measured well under a second here, and the assertion is loose enough not to flake.
        assert!(
            started.elapsed() < std::time::Duration::from_secs(30),
            "clamped search took {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn search_params_default_when_absent() {
        let mut conn = open_seeded(&["h"]);
        let defaulted = cover_in(&mut conn, request("h", false)).unwrap();
        assert_eq!(defaulted.result.search.beam_width, 8);
        assert_eq!(defaulted.result.search.candidates_per_slot, 12);
        let mut req = request("h", false);
        req.beam_width = Some(2);
        req.candidates_per_slot = Some(3);
        req.commitment_horizon_days = Some(5);
        let tuned = cover_in(&mut conn, req).unwrap();
        assert_eq!(tuned.result.search.beam_width, 2);
        assert_eq!(tuned.result.search.candidates_per_slot, 3);
        // A zero from the caller is clamped, never a zero-width search.
        let mut req = request("h", false);
        req.beam_width = Some(0);
        let clamped = cover_in(&mut conn, req).unwrap();
        assert_eq!(clamped.result.search.beam_width, 1);
        // The boundary is inclusive: 16 passes through unchanged, not off by one.
        let mut req = request("h", false);
        req.beam_width = Some(16);
        req.candidates_per_slot = Some(16);
        let at_ceiling = cover_in(&mut conn, req).unwrap();
        assert_eq!(at_ceiling.result.search.beam_width, 16);
        assert_eq!(at_ceiling.result.search.candidates_per_slot, 16);
        // Same snapshot whatever the knobs: the hash is input identity, not search identity.
        assert_eq!(tuned.result.snapshot_hash, defaulted.result.snapshot_hash);
    }
}
