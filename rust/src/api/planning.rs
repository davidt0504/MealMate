use kimatta_storage::{Connection, HouseholdId, MealScope, MealSlot, PlanningCycle};

use crate::api::error::KimattaError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MealSlotDto {
    Breakfast,
    Lunch,
    Dinner,
}

#[derive(Debug)]
pub struct PlanningCycleDto {
    pub household_id: String,
    pub anchor_date: String, // ISO YYYY-MM-DD
    pub length_days: u32,
    pub meal_slots: Vec<MealSlotDto>, // canonical order
    pub dates: Vec<String>,           // the cycle's civil dates, ISO, length_days of them
}

pub(crate) fn slot_to_domain(slot: &MealSlotDto) -> MealSlot {
    match slot {
        MealSlotDto::Breakfast => MealSlot::Breakfast,
        MealSlotDto::Lunch => MealSlot::Lunch,
        MealSlotDto::Dinner => MealSlot::Dinner,
    }
}

pub(crate) fn slot_from_domain(slot: MealSlot) -> MealSlotDto {
    match slot {
        MealSlot::Breakfast => MealSlotDto::Breakfast,
        MealSlot::Lunch => MealSlotDto::Lunch,
        MealSlot::Dinner => MealSlotDto::Dinner,
    }
}

/// Returns the household's planning cycle, creating the dinner-only seven-day default
/// anchored at `default_anchor_date` if none exists. The anchor is supplied by the caller
/// because "today" is a local civil date the platform owns; Rust never reads the clock, so
/// every value here is a pure function of its inputs (PRD v3 §8, invariant 20).
pub fn ensure_planning_cycle(
    household_id: String,
    default_anchor_date: String,
) -> Result<PlanningCycleDto, KimattaError> {
    crate::db::with(|conn| ensure_in(conn, &household_id, &default_anchor_date))
}

/// Replaces the household's cycle and meal scope, and returns what was stored.
pub fn save_planning_cycle(
    household_id: String,
    anchor_date: String,
    length_days: u32,
    meal_slots: Vec<MealSlotDto>,
) -> Result<PlanningCycleDto, KimattaError> {
    crate::db::with(|conn| save_in(conn, &household_id, &anchor_date, length_days, &meal_slots))
}

/// The cycle window `offset_cycles` cycles away from the one containing `today` (offset 0 is
/// the active window; the window before the anchor is reached with a negative offset). The
/// stored anchor is the rhythm's phase, never the only week the household can see. `today`
/// is caller-supplied for the reason `ensure_planning_cycle` gives.
pub fn planning_cycle_window(
    household_id: String,
    today: String,
    offset_cycles: i32,
) -> Result<PlanningCycleDto, KimattaError> {
    crate::db::with(|conn| window_in(conn, &household_id, &today, offset_cycles))
}

/// Split out from the command for the same reason `rename_in` is: it can be tested with more
/// than one household present without installing the process-wide connection.
fn ensure_in(
    conn: &mut Connection,
    household_id: &str,
    default_anchor_date: &str,
) -> Result<PlanningCycleDto, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    let anchor = kimatta_storage::parse_civil_date(default_anchor_date)?;
    // Validated before storage is touched: because `PlanningCycle`'s constructors carry every
    // invariant, an invalid request cannot reach a write.
    let default = PlanningCycle::default_for(id, anchor)?;
    let cycle = kimatta_storage::ensure_planning_cycle(conn, &default)?;
    Ok(to_dto(&cycle))
}

fn window_in(
    conn: &mut Connection,
    household_id: &str,
    today: &str,
    offset_cycles: i32,
) -> Result<PlanningCycleDto, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    let today = kimatta_storage::parse_civil_date(today)?;
    let default = PlanningCycle::default_for(id, today)?;
    let stored = kimatta_storage::ensure_planning_cycle(conn, &default)?;
    Ok(to_dto(&stored.window_containing(today, offset_cycles)?))
}

fn save_in(
    conn: &mut Connection,
    household_id: &str,
    anchor_date: &str,
    length_days: u32,
    meal_slots: &[MealSlotDto],
) -> Result<PlanningCycleDto, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    let anchor = kimatta_storage::parse_civil_date(anchor_date)?;
    let scope = MealScope::new(meal_slots.iter().map(slot_to_domain))?;
    let cycle = PlanningCycle::new(id, anchor, length_days, scope)?;
    kimatta_storage::save_planning_cycle(conn, &cycle)?;
    Ok(to_dto(&cycle))
}

/// `dates` is computed in Rust and shipped whole, so Flutter never does calendar arithmetic;
/// at the 31-day maximum that is 31 strings.
fn to_dto(cycle: &PlanningCycle) -> PlanningCycleDto {
    PlanningCycleDto {
        household_id: cycle.household_id().as_str().to_owned(),
        anchor_date: kimatta_storage::format_civil_date(cycle.anchor()),
        length_days: cycle.length_days(),
        meal_slots: cycle
            .scope()
            .slots()
            .iter()
            .copied()
            .map(slot_from_domain)
            .collect(),
        dates: cycle
            .dates()
            .into_iter()
            .map(kimatta_storage::format_civil_date)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kimatta_storage::{Household, HouseholdMember, MemberId};

    fn seed(conn: &mut Connection, id: &str) {
        let h = Household {
            id: HouseholdId::new(id).unwrap(),
            name: None,
        };
        let m = HouseholdMember {
            id: MemberId::new(format!("m-{id}")).unwrap(),
            household_id: h.id.clone(),
            display_name: "Me".to_owned(),
        };
        kimatta_storage::insert_household(conn, &h, std::slice::from_ref(&m)).unwrap();
    }

    fn open_seeded(ids: &[&str]) -> Connection {
        let mut conn = kimatta_storage::open(":memory:").unwrap();
        for id in ids {
            seed(&mut conn, id);
        }
        conn
    }

    #[test]
    fn ensure_returns_the_dinner_only_default_with_its_dates() {
        let mut conn = open_seeded(&["h"]);
        let dto = ensure_in(&mut conn, "h", "2026-08-29").unwrap();
        assert_eq!(dto.household_id, "h");
        assert_eq!(dto.length_days, 7);
        assert_eq!(dto.meal_slots, vec![MealSlotDto::Dinner]);
        assert_eq!(dto.dates.len(), 7);
        assert_eq!(dto.dates.first().unwrap(), "2026-08-29");
        assert_eq!(dto.dates.last().unwrap(), "2026-09-04");
    }

    #[test]
    fn save_then_ensure_returns_the_saved_cycle() {
        let mut conn = open_seeded(&["h"]);
        let saved = save_in(
            &mut conn,
            "h",
            "2026-01-05",
            14,
            &[MealSlotDto::Dinner, MealSlotDto::Breakfast],
        )
        .unwrap();
        assert_eq!(
            saved.meal_slots,
            vec![MealSlotDto::Breakfast, MealSlotDto::Dinner]
        );
        let again = ensure_in(&mut conn, "h", "2099-12-01").unwrap();
        assert_eq!(again.anchor_date, "2026-01-05");
        assert_eq!(again.length_days, 14);
        assert_eq!(again.dates.len(), 14);
    }

    #[test]
    fn rejects_an_invalid_anchor_date() {
        let mut conn = open_seeded(&["h"]);
        assert!(matches!(
            ensure_in(&mut conn, "h", "29/08/2026").unwrap_err(),
            KimattaError::Planning { .. }
        ));
    }

    /// The card's stop condition, pinned at the bridge boundary as well as in the domain.
    #[test]
    fn rejects_a_datetime_anchor() {
        let mut conn = open_seeded(&["h"]);
        for raw in ["2026-08-29T23:00:00Z", "2026-08-29T00:00:00-05:00"] {
            assert!(
                matches!(
                    ensure_in(&mut conn, "h", raw).unwrap_err(),
                    KimattaError::Planning { .. }
                ),
                "{raw:?} must not be accepted as an anchor"
            );
        }
    }

    #[test]
    fn rejects_an_out_of_range_length() {
        let mut conn = open_seeded(&["h"]);
        for bad in [0, 32] {
            assert!(
                matches!(
                    save_in(&mut conn, "h", "2026-08-29", bad, &[MealSlotDto::Dinner]).unwrap_err(),
                    KimattaError::Planning { .. }
                ),
                "length {bad} must be rejected"
            );
        }
    }

    /// The only bridge-boundary validation Dart can trivially trip — an unknown slot string is
    /// impossible because the slot is an enum — and the sole place the `EmptyMealScope`
    /// mapping is exercised.
    #[test]
    fn rejects_an_empty_meal_scope() {
        let mut conn = open_seeded(&["h"]);
        assert!(matches!(
            save_in(&mut conn, "h", "2026-08-29", 7, &[]).unwrap_err(),
            KimattaError::Planning { .. }
        ));
        let rows: u32 = conn
            .query_row("SELECT COUNT(*) FROM planning_cycle", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 0);
    }

    /// MVP-013: the window read materialises the same default `ensure_in` does, and offset 0
    /// on the anchor day *is* that cycle. The calendar cases live in `food-domain`.
    #[test]
    fn window_at_the_anchor_equals_the_ensured_cycle() {
        let mut conn = open_seeded(&["h"]);
        let ensured = ensure_in(&mut conn, "h", "2026-08-29").unwrap();
        let window = window_in(&mut conn, "h", "2026-08-29", 0).unwrap();
        assert_eq!(window.household_id, ensured.household_id);
        assert_eq!(window.anchor_date, ensured.anchor_date);
        assert_eq!(window.length_days, ensured.length_days);
        assert_eq!(window.meal_slots, ensured.meal_slots);
        assert_eq!(window.dates, ensured.dates);
        let next = window_in(&mut conn, "h", "2026-08-29", 1).unwrap();
        assert_eq!(next.anchor_date, "2026-09-05");
        assert_eq!(next.dates.len(), 7);
        let rows: u32 = conn
            .query_row("SELECT COUNT(*) FROM planning_cycle", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1, "a window read never stores a shifted cycle");
    }

    #[test]
    fn commands_are_scoped_to_the_named_household() {
        let mut conn = open_seeded(&["h1", "h2"]);
        save_in(&mut conn, "h1", "2026-08-29", 7, &[MealSlotDto::Dinner]).unwrap();
        save_in(&mut conn, "h2", "2026-01-05", 3, &[MealSlotDto::Lunch]).unwrap();
        let a = ensure_in(&mut conn, "h1", "2099-12-01").unwrap();
        let b = ensure_in(&mut conn, "h2", "2099-12-01").unwrap();
        assert_eq!((a.anchor_date.as_str(), a.length_days), ("2026-08-29", 7));
        assert_eq!(a.meal_slots, vec![MealSlotDto::Dinner]);
        assert_eq!((b.anchor_date.as_str(), b.length_days), ("2026-01-05", 3));
        assert_eq!(b.meal_slots, vec![MealSlotDto::Lunch]);
    }
}
