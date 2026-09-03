//! The one plan-decision command (MVP-024, invariant 21): one explicit user decision — swap,
//! veto, or the reviewed-restrictions marker — crosses in one call, and the mutation plus its
//! ledger evidence row land in one transaction on the other side. `today` is caller-supplied,
//! as every planner command's is (invariant 20).

use kimatta_application::{
    record_decision as run_record_decision, PlanDecision, PlanDecisionRequest,
};
use kimatta_storage::HouseholdId;
use uuid::Uuid;

use crate::api::error::KimattaError;
use crate::api::planned_meals::{component_to_domain, MealComponentDto};
use crate::api::planner::{
    status_from_domain, OutcomeStatusDto, MAX_OFFSET_CYCLES, OFFSET_OUT_OF_RANGE,
};
use crate::api::planning::{slot_to_domain, MealSlotDto};

/// One decision, data included. `Swap` is keyed by `(date, slot)` — preview slots have no
/// stored id to name — and writes the new occurrence locked under the user's word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanDecisionDto {
    Swap {
        /// ISO civil date.
        date: String,
        slot: MealSlotDto,
        components: Vec<MealComponentDto>,
    },
    Veto {
        subject: String,
        /// ISO civil date of the slot the veto was uttered on, for the ledger only.
        date: String,
        slot: MealSlotDto,
    },
    RestrictionsReviewed,
}

/// Carries the snapshot inputs explicitly: recording a decision re-assesses the same window
/// `cover_cycle` planned, so it needs the same `(today, offset_cycles)` addressing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDecisionRequestDto {
    pub household_id: String,
    /// ISO civil date.
    pub today: String,
    /// 0 is the active window, as `planning_cycle_window` counts.
    pub offset_cycles: i32,
    pub decision: PlanDecisionDto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDecisionOutcomeDto {
    pub ledger_entry_id: String,
    pub prior_status: OutcomeStatusDto,
    pub resulting_status: OutcomeStatusDto,
}

/// Records one decision: mutation, re-assessment and the `correct` ledger row in one
/// IMMEDIATE transaction.
pub fn record_plan_decision(
    request: PlanDecisionRequestDto,
) -> Result<PlanDecisionOutcomeDto, KimattaError> {
    crate::db::with(|conn| record_in(conn, request))
}

/// Split out from the command, as `cover_in` is, so it can be tested against a seeded
/// connection. Inputs are validated before storage is touched, mirroring `cover_in`.
pub(crate) fn record_in(
    conn: &mut kimatta_storage::Connection,
    request: PlanDecisionRequestDto,
) -> Result<PlanDecisionOutcomeDto, KimattaError> {
    let household_id = HouseholdId::new(request.household_id)?;
    let today = kimatta_storage::parse_civil_date(&request.today)?;
    if request.offset_cycles > MAX_OFFSET_CYCLES || request.offset_cycles < -MAX_OFFSET_CYCLES {
        return Err(KimattaError::Planning {
            message: OFFSET_OUT_OF_RANGE.to_owned(),
        });
    }
    let decision = match request.decision {
        PlanDecisionDto::Swap {
            date,
            slot,
            components,
        } => PlanDecision::Swap {
            date: kimatta_storage::parse_civil_date(&date)?,
            slot: slot_to_domain(&slot),
            components: components
                .into_iter()
                .map(component_to_domain)
                .collect::<Result<Vec<_>, _>>()?,
        },
        PlanDecisionDto::Veto {
            subject,
            date,
            slot,
        } => PlanDecision::Veto {
            subject,
            date: kimatta_storage::parse_civil_date(&date)?,
            slot: slot_to_domain(&slot),
        },
        PlanDecisionDto::RestrictionsReviewed => PlanDecision::RestrictionsReviewed,
    };
    let req = PlanDecisionRequest {
        household_id,
        today,
        offset_cycles: request.offset_cycles,
        decision,
    };
    let outcome = run_record_decision(conn, &req, &mut || Uuid::new_v4().to_string())?;
    Ok(PlanDecisionOutcomeDto {
        ledger_entry_id: outcome.ledger_entry_id.as_str().to_owned(),
        prior_status: status_from_domain(outcome.prior_status),
        resulting_status: status_from_domain(outcome.resulting_status),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use kimatta_application::ACTION_CORRECT;
    use kimatta_storage::planner::FoodPolicies;
    use kimatta_storage::{
        insert_household, list_ledger_entries, list_planned_meals, list_policies, open,
        parse_civil_date, save_member_preferences, save_planning_cycle, save_policy, save_recipe,
        save_restrictions, Connection, EvidenceSource, Household, HouseholdMember,
        HouseholdRestrictions, IngredientLine, MealScope, MealSlot, MemberId, MemberPreference,
        MemberPreferences, PlanningCycle, Policy, PolicyId, ProvenanceKind, Quantity, Recipe,
        RecipeId, RecipeProvenance, Restriction, RestrictionKind, Sentiment, Unit,
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
        let line = IngredientLine::new(
            "1 cup rice",
            "rice",
            None,
            Quantity::Unknown,
            Unit::None,
            None,
            false,
        )
        .unwrap();
        let r = Recipe::new(
            RecipeId::new(format!("r-{id}")).unwrap(),
            hid(id),
            "Rice bowl",
            Some(4),
            Some(15),
            "",
            vec![line],
            RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
        )
        .unwrap();
        save_recipe(conn, &r).unwrap();
    }

    fn open_seeded(ids: &[&str]) -> Connection {
        let mut conn = open(":memory:").unwrap();
        for id in ids {
            seed(&mut conn, id);
        }
        conn
    }

    fn request(id: &str, decision: PlanDecisionDto) -> PlanDecisionRequestDto {
        PlanDecisionRequestDto {
            household_id: id.to_owned(),
            today: "2026-08-29".to_owned(),
            offset_cycles: 0,
            decision,
        }
    }

    fn swap(id: &str) -> PlanDecisionDto {
        PlanDecisionDto::Swap {
            date: "2026-08-29".to_owned(),
            slot: MealSlotDto::Dinner,
            components: vec![MealComponentDto {
                kind: "recipe".to_owned(),
                recipe_id: Some(format!("r-{id}")),
                note: None,
                scale: None,
            }],
        }
    }

    #[test]
    fn swap_maps_and_writes_a_locked_row_and_a_correct_ledger_row() {
        let mut conn = open_seeded(&["h"]);
        let out = record_in(&mut conn, request("h", swap("h"))).unwrap();
        assert!(!out.ledger_entry_id.is_empty());
        let meals = list_planned_meals(
            &conn,
            &hid("h"),
            parse_civil_date("2026-08-29").unwrap(),
            parse_civil_date("2026-08-29").unwrap(),
        )
        .unwrap();
        assert_eq!(meals.len(), 1);
        assert!(meals[0].locked());
        let rows = list_ledger_entries(&conn, &hid("h")).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1.selected_action, ACTION_CORRECT);
        assert!(rows[0].1.payload.contains("decision=swap"));
        assert!(rows[0].1.payload.contains("corrects_seq=0"));
    }

    #[test]
    fn veto_and_reviewed_map_and_persist_their_policies() {
        let mut conn = open_seeded(&["h"]);
        record_in(
            &mut conn,
            request(
                "h",
                PlanDecisionDto::Veto {
                    subject: "olives".to_owned(),
                    date: "2026-08-29".to_owned(),
                    slot: MealSlotDto::Dinner,
                },
            ),
        )
        .unwrap();
        record_in(
            &mut conn,
            request("h", PlanDecisionDto::RestrictionsReviewed),
        )
        .unwrap();
        let policies = list_policies(&conn, &hid("h"), "food").unwrap();
        assert!(policies.iter().any(|p| p.policy_type == "food.hard_veto"
            && p.parameters.get("subject").map(String::as_str) == Some("olives")));
        assert!(policies
            .iter()
            .any(|p| p.policy_type == "food.restrictions_reviewed" && p.enabled));
        let rows = list_ledger_entries(&conn, &hid("h")).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows[0].1.payload.contains("decision=veto"));
        assert!(rows[1].1.payload.contains("decision=restrictions_reviewed"));
        // The second row names the first: seq is global, so read the actual value.
        assert!(rows[1]
            .1
            .payload
            .contains(&format!("corrects_seq={}", rows[0].0)));
    }

    #[test]
    fn household_scoping_is_enforced() {
        let mut conn = open_seeded(&["a", "b"]);
        record_in(&mut conn, request("a", swap("a"))).unwrap();
        assert!(list_ledger_entries(&conn, &hid("b")).unwrap().is_empty());
        assert!(list_planned_meals(
            &conn,
            &hid("b"),
            parse_civil_date("2026-08-29").unwrap(),
            parse_civil_date("2026-08-29").unwrap(),
        )
        .unwrap()
        .is_empty());
        let err = record_in(&mut conn, request("ghost", swap("a"))).unwrap_err();
        assert!(
            matches!(&err, KimattaError::Storage { message } if message.contains("ghost")),
            "{err:?}"
        );
    }

    /// Sibling of the offset check below: the application layer refuses these, and the bridge
    /// must surface them as `Planning` carrying user prose — `describeFailure` renders that
    /// message verbatim, so a shortcut forwarding the variant's `Display` would put engine
    /// addressing vocabulary in a snackbar.
    #[test]
    fn refused_decisions_arrive_as_planning_errors_in_user_language() {
        let mut conn = open_seeded(&["h"]);
        // The last four have no alphanumeric character at all: `trim()` keeps them, but the
        // matcher works on `tokens()`, so they could never match anything. A recipe title is
        // the veto subject and `Recipe::new` enforces only `non_blank`, so each is a legal
        // title and therefore reachable from the Cover screen's "Never suggest".
        for subject in ["", " ", "\t", "\n", "  \r\n ", "🍝", "—", "★", "!!!"] {
            let err = record_in(
                &mut conn,
                request(
                    "h",
                    PlanDecisionDto::Veto {
                        subject: subject.to_owned(),
                        date: "2026-08-29".to_owned(),
                        slot: MealSlotDto::Dinner,
                    },
                ),
            )
            .unwrap_err();
            assert!(
                matches!(&err, KimattaError::Planning { message }
                    if message == "a meal to never suggest must have a name we can match"),
                "{subject:?}: {err:?}"
            );
        }
        let err = record_in(
            &mut conn,
            request(
                "h",
                PlanDecisionDto::Swap {
                    date: "2027-06-01".to_owned(),
                    slot: MealSlotDto::Dinner,
                    components: vec![MealComponentDto {
                        kind: "recipe".to_owned(),
                        recipe_id: Some("r-h".to_owned()),
                        note: None,
                        scale: None,
                    }],
                },
            ),
        )
        .unwrap_err();
        assert!(
            matches!(&err, KimattaError::Planning { message }
                if message == "that decision is for a day outside the week being planned"),
            "{err:?}"
        );
        assert!(list_ledger_entries(&conn, &hid("h")).unwrap().is_empty());
    }

    /// A swap onto a slot the household locked is refused with prose, not with the *automation*
    /// refusal. Both storage sites that raise `LockedPlannedMeal` gate on
    /// `WriteSource::Automation`; this write is `WriteSource::User`, so borrowing that variant
    /// put a raw meal id and the word "automation" on the Cover screen for a write automation
    /// never made. The message is asserted whole *and* checked for both leaks, because
    /// `describeFailure` renders a `Planning` message verbatim into a snackbar.
    #[test]
    fn a_swap_onto_a_locked_slot_is_refused_in_user_language() {
        let mut conn = open_seeded(&["h"]);
        // The first swap writes the row locked itself — `record_in` locks what the user
        // decided — so no second lock call is needed to reach the refusal.
        record_in(&mut conn, request("h", swap("h"))).unwrap();
        let held = list_planned_meals(
            &conn,
            &hid("h"),
            parse_civil_date("2026-08-29").unwrap(),
            parse_civil_date("2026-08-29").unwrap(),
        )
        .unwrap();
        assert!(held[0].locked());
        let meal_id = held[0].id().as_str().to_owned();
        let err = record_in(&mut conn, request("h", swap("h"))).unwrap_err();
        assert!(
            matches!(&err, KimattaError::Planning { message }
                if message == "that meal is locked, so it cannot be swapped"),
            "{err:?}"
        );
        let rendered = err.to_string();
        assert!(!rendered.contains("automation"), "{rendered}");
        assert!(!rendered.contains(&meal_id), "{rendered}");
        // The refused swap rolls back whole: one row, still the first one, still locked.
        let after = list_planned_meals(
            &conn,
            &hid("h"),
            parse_civil_date("2026-08-29").unwrap(),
            parse_civil_date("2026-08-29").unwrap(),
        )
        .unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].id().as_str(), meal_id);
        assert_eq!(list_ledger_entries(&conn, &hid("h")).unwrap().len(), 1);
    }

    #[test]
    fn offset_cycles_validation_is_typed_here_too() {
        let mut conn = open_seeded(&["h"]);
        // Both signs at the bound and both i32 extremes: the check is a comparison rather than
        // `.abs()` precisely because `i32::MIN.abs()` panics, so both ends are exercised here
        // as they are at the `cover_in` site.
        for offset in [521, -521, i32::MAX, i32::MIN] {
            let err = record_in(
                &mut conn,
                PlanDecisionRequestDto {
                    offset_cycles: offset,
                    ..request("h", PlanDecisionDto::RestrictionsReviewed)
                },
            )
            .unwrap_err();
            assert!(
                matches!(&err, KimattaError::Planning { message }
                    if message == OFFSET_OUT_OF_RANGE),
                "{offset}: {err:?}"
            );
        }
        assert!(list_ledger_entries(&conn, &hid("h")).unwrap().is_empty());
    }
}
