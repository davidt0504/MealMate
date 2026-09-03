use kimatta_storage::{
    format_civil_date, Connection, HouseholdId, MealComponent, PlannedMeal, PlannedMealId,
    RecipeId, StorageError, WriteSource,
};

use crate::api::error::KimattaError;
use crate::api::planning::{slot_from_domain, slot_to_domain, MealSlotDto};
use crate::api::recipe::id_or_minted;

/// A serving multiplier of the recipe as written; absent means 1×.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleDto {
    pub numer: u32,
    pub denom: u32,
}

/// `kind` is a `MealComponent` token (`recipe`, `leftovers`, `dining_out`, `frozen_quick`,
/// `freeform`, `open`), carried as a string for the reason `UnitDto`/`RestrictionDto` give.
/// `recipe_id` and `scale` belong to `recipe` only; `note` to the rest, required for
/// `freeform`. Any other pairing is a typed `PlannedMeal` error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MealComponentDto {
    pub kind: String,
    pub recipe_id: Option<String>,
    pub note: Option<String>,
    pub scale: Option<ScaleDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedMealDto {
    /// Blank mints a v4 UUID, as `RecipeDto.id` does.
    pub id: String,
    pub household_id: String,
    /// ISO civil date.
    pub date: String,
    pub slot: MealSlotDto,
    pub components: Vec<MealComponentDto>,
    /// **Output only** on save: `save_planned_meal` ignores it and never changes lock state;
    /// use `set_planned_meal_lock`. Every read-back reports the stored value.
    pub locked: bool,
}

/// Saves the whole occurrence and returns what was stored. Always a user write: automation
/// never crosses the bridge (MVP-023 calls storage directly with `WriteSource::Automation`).
pub fn save_planned_meal(meal: PlannedMealDto) -> Result<PlannedMealDto, KimattaError> {
    crate::db::with(|conn| save_in(conn, meal))
}

/// The occurrence in exactly `household_id`, or `None`.
pub fn load_planned_meal(
    household_id: String,
    meal_id: String,
) -> Result<Option<PlannedMealDto>, KimattaError> {
    crate::db::with(|conn| load_in(conn, &household_id, &meal_id))
}

/// Occurrences dated `from_date..=to_date`, by date then slot.
pub fn list_planned_meals(
    household_id: String,
    from_date: String,
    to_date: String,
) -> Result<Vec<PlannedMealDto>, KimattaError> {
    crate::db::with(|conn| list_in(conn, &household_id, &from_date, &to_date))
}

/// Sets or clears the user's lock; returns the occurrence as stored.
pub fn set_planned_meal_lock(
    household_id: String,
    meal_id: String,
    locked: bool,
) -> Result<PlannedMealDto, KimattaError> {
    crate::db::with(|conn| lock_in(conn, &household_id, &meal_id, locked))
}

pub fn delete_planned_meal(household_id: String, meal_id: String) -> Result<(), KimattaError> {
    crate::db::with(|conn| delete_in(conn, &household_id, &meal_id))
}

/// The component kind vocabulary, one source for a picker (as `known_unit_kinds` is).
pub fn known_meal_component_kinds() -> Vec<String> {
    MealComponent::KINDS
        .iter()
        .map(|k| (*k).to_owned())
        .collect()
}

pub(crate) fn component_to_domain(c: MealComponentDto) -> Result<MealComponent, KimattaError> {
    let recipe_id = c.recipe_id.map(RecipeId::new).transpose()?;
    let scale = c
        .scale
        .map(|s| MealComponent::parse_scale(s.numer, s.denom))
        .transpose()?;
    // `None` from `parse_row` is a kind/column mismatch; on this side it is bad input, so it
    // reads as the domain's own vocabulary error rather than a storage corruption.
    MealComponent::parse_row(&c.kind, recipe_id, c.note, scale)?.ok_or_else(|| {
        KimattaError::PlannedMeal {
            message: format!("component fields do not match kind {:?}", c.kind),
        }
    })
}

pub(crate) fn component_from_domain(c: &MealComponent) -> MealComponentDto {
    let (recipe_id, note, scale) = match c {
        MealComponent::Recipe { recipe_id, scale } => (
            Some(recipe_id.as_str().to_owned()),
            None,
            scale.map(|s| ScaleDto {
                numer: s.numer(),
                denom: s.denom(),
            }),
        ),
        MealComponent::Leftovers { note }
        | MealComponent::DiningOut { note }
        | MealComponent::FrozenQuick { note }
        | MealComponent::Open { note } => (None, note.clone(), None),
        MealComponent::Freeform { note } => (None, Some(note.clone()), None),
    };
    MealComponentDto {
        kind: c.kind_str().to_owned(),
        recipe_id,
        note,
        scale,
    }
}

/// Validates fully into the domain before storage is touched, as `recipe_to_domain` does. A
/// malformed date is a `Planning` error, the `archive_recipe_in` precedent.
fn to_domain(dto: PlannedMealDto) -> Result<PlannedMeal, KimattaError> {
    let id = PlannedMealId::new(id_or_minted(dto.id))?;
    let household_id = HouseholdId::new(dto.household_id)?;
    let date = kimatta_storage::parse_civil_date(&dto.date)?;
    let components = dto
        .components
        .into_iter()
        .map(component_to_domain)
        .collect::<Result<Vec<_>, _>>()?;
    // `dto.locked` is dropped here: storage never writes it from a save either.
    Ok(PlannedMeal::new(
        id,
        household_id,
        date,
        slot_to_domain(&dto.slot),
        components,
        false,
    )?)
}

fn from_domain(m: &PlannedMeal) -> PlannedMealDto {
    PlannedMealDto {
        id: m.id().as_str().to_owned(),
        household_id: m.household_id().as_str().to_owned(),
        date: format_civil_date(m.date()),
        slot: slot_from_domain(m.slot()),
        components: m.components().iter().map(component_from_domain).collect(),
        locked: m.locked(),
    }
}

/// Read back rather than echo: `locked` is not something a save can set, so only the stored
/// row knows it.
fn stored(
    conn: &Connection,
    household: &HouseholdId,
    id: &PlannedMealId,
) -> Result<PlannedMealDto, KimattaError> {
    match kimatta_storage::load_planned_meal(conn, household, id)? {
        Some(meal) => Ok(from_domain(&meal)),
        None => Err(StorageError::NoSuchPlannedMeal {
            meal: id.as_str().to_owned(),
            household: household.as_str().to_owned(),
        }
        .into()),
    }
}

pub(crate) fn save_in(
    conn: &mut Connection,
    dto: PlannedMealDto,
) -> Result<PlannedMealDto, KimattaError> {
    let meal = to_domain(dto)?;
    kimatta_storage::save_planned_meal(conn, &meal, WriteSource::User)?;
    stored(conn, meal.household_id(), meal.id())
}

fn load_in(
    conn: &Connection,
    household_id: &str,
    meal_id: &str,
) -> Result<Option<PlannedMealDto>, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let id = PlannedMealId::new(meal_id)?;
    Ok(kimatta_storage::load_planned_meal(conn, &household, &id)?
        .as_ref()
        .map(from_domain))
}

fn list_in(
    conn: &Connection,
    household_id: &str,
    from_date: &str,
    to_date: &str,
) -> Result<Vec<PlannedMealDto>, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let from = kimatta_storage::parse_civil_date(from_date)?;
    let to = kimatta_storage::parse_civil_date(to_date)?;
    Ok(
        kimatta_storage::list_planned_meals(conn, &household, from, to)?
            .iter()
            .map(from_domain)
            .collect(),
    )
}

fn lock_in(
    conn: &mut Connection,
    household_id: &str,
    meal_id: &str,
    locked: bool,
) -> Result<PlannedMealDto, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let id = PlannedMealId::new(meal_id)?;
    kimatta_storage::set_planned_meal_lock(conn, &household, &id, locked, WriteSource::User)?;
    stored(conn, &household, &id)
}

fn delete_in(conn: &mut Connection, household_id: &str, meal_id: &str) -> Result<(), KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let id = PlannedMealId::new(meal_id)?;
    Ok(kimatta_storage::delete_planned_meal(
        conn,
        &household,
        &id,
        WriteSource::User,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kimatta_storage::{
        Household, HouseholdMember, MealScope, MealSlot, MemberId, PlanningCycle, ProvenanceKind,
        Recipe, RecipeProvenance,
    };

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
        let cycle = PlanningCycle::new(
            h.id.clone(),
            kimatta_storage::parse_civil_date("2026-08-29").unwrap(),
            7,
            MealScope::new([MealSlot::Lunch, MealSlot::Dinner]).unwrap(),
        )
        .unwrap();
        kimatta_storage::save_planning_cycle(conn, &cycle).unwrap();
        let recipe = Recipe::new(
            RecipeId::new(format!("r-{id}")).unwrap(),
            h.id,
            "Toast",
            None,
            None,
            "",
            vec![],
            RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
        )
        .unwrap();
        kimatta_storage::save_recipe(conn, &recipe).unwrap();
    }

    fn open_seeded(ids: &[&str]) -> Connection {
        let mut conn = kimatta_storage::open(":memory:").unwrap();
        for id in ids {
            seed(&mut conn, id);
        }
        conn
    }

    fn component(kind: &str, recipe_id: Option<&str>, note: Option<&str>) -> MealComponentDto {
        MealComponentDto {
            kind: kind.to_owned(),
            recipe_id: recipe_id.map(str::to_owned),
            note: note.map(str::to_owned),
            scale: None,
        }
    }

    fn dinner(household: &str, id: &str, date: &str) -> PlannedMealDto {
        PlannedMealDto {
            id: id.to_owned(),
            household_id: household.to_owned(),
            date: date.to_owned(),
            slot: MealSlotDto::Dinner,
            components: vec![
                MealComponentDto {
                    scale: Some(ScaleDto { numer: 2, denom: 4 }),
                    ..component("recipe", Some(&format!("r-{household}")), None)
                },
                component("leftovers", None, Some("chili")),
            ],
            locked: false,
        }
    }

    fn rows(conn: &Connection) -> u32 {
        conn.query_row("SELECT COUNT(*) FROM planned_meal", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn save_then_load_returns_an_equal_dto() {
        let mut conn = open_seeded(&["h"]);
        let saved = save_in(&mut conn, dinner("h", "", "2026-08-30")).unwrap();
        assert!(!saved.id.is_empty());
        assert!(!saved.locked);
        assert_eq!(
            saved.components[0].scale,
            Some(ScaleDto { numer: 1, denom: 2 })
        );
        let loaded = load_in(&conn, "h", &saved.id).unwrap();
        assert_eq!(loaded, Some(saved));
    }

    /// Adversarial: the request's `locked` is dropped, on create and on update.
    #[test]
    fn a_sent_locked_flag_is_ignored_on_save() {
        let mut conn = open_seeded(&["h"]);
        let mut sent = dinner("h", "pm", "2026-08-30");
        sent.locked = true;
        assert!(!save_in(&mut conn, sent.clone()).unwrap().locked);
        lock_in(&mut conn, "h", "pm", true).unwrap();
        sent.locked = false;
        assert!(save_in(&mut conn, sent).unwrap().locked);
    }

    #[test]
    fn set_lock_then_save_keeps_it_locked() {
        let mut conn = open_seeded(&["h"]);
        save_in(&mut conn, dinner("h", "pm", "2026-08-30")).unwrap();
        assert!(lock_in(&mut conn, "h", "pm", true).unwrap().locked);
        let again = save_in(&mut conn, dinner("h", "pm", "2026-08-30")).unwrap();
        assert!(again.locked);
        assert!(!lock_in(&mut conn, "h", "pm", false).unwrap().locked);
    }

    #[test]
    fn commands_are_scoped_to_the_named_household() {
        let mut conn = open_seeded(&["h1", "h2"]);
        let saved = save_in(&mut conn, dinner("h1", "pm", "2026-08-30")).unwrap();
        assert_eq!(load_in(&conn, "h2", "pm").unwrap(), None);
        assert!(list_in(&conn, "h2", "2026-08-01", "2026-12-31")
            .unwrap()
            .is_empty());
        assert!(matches!(
            lock_in(&mut conn, "h2", "pm", true).unwrap_err(),
            KimattaError::Storage { .. }
        ));
        assert!(matches!(
            delete_in(&mut conn, "h2", "pm").unwrap_err(),
            KimattaError::Storage { .. }
        ));
        assert!(matches!(
            save_in(&mut conn, dinner("h2", "pm", "2026-08-30")).unwrap_err(),
            KimattaError::Storage { .. }
        ));
        assert_eq!(load_in(&conn, "h1", "pm").unwrap(), Some(saved));
        delete_in(&mut conn, "h1", "pm").unwrap();
        assert_eq!(rows(&conn), 0);
    }

    #[test]
    fn an_unknown_kind_token_is_a_typed_planned_meal_error_and_writes_nothing() {
        let mut conn = open_seeded(&["h"]);
        let mut sent = dinner("h", "pm", "2026-08-30");
        sent.components.push(component("takeout", None, None));
        assert!(matches!(
            save_in(&mut conn, sent).unwrap_err(),
            KimattaError::PlannedMeal { .. }
        ));
        // A known kind with the wrong columns is the same typed error.
        let mut sent = dinner("h", "pm", "2026-08-30");
        sent.components
            .push(component("leftovers", Some("r-h"), None));
        assert!(matches!(
            save_in(&mut conn, sent).unwrap_err(),
            KimattaError::PlannedMeal { .. }
        ));
        let mut sent = dinner("h", "pm", "2026-08-30");
        sent.components.clear();
        assert!(matches!(
            save_in(&mut conn, sent).unwrap_err(),
            KimattaError::PlannedMeal { .. }
        ));
        assert_eq!(rows(&conn), 0);
    }

    #[test]
    fn a_non_civil_date_is_a_typed_planning_error() {
        let mut conn = open_seeded(&["h"]);
        for raw in ["2026-08-30T00:00:00Z", "20260830", ""] {
            assert!(
                matches!(
                    save_in(&mut conn, dinner("h", "pm", raw)).unwrap_err(),
                    KimattaError::Planning { .. }
                ),
                "{raw:?} must be a planning error"
            );
        }
        assert!(matches!(
            list_in(&conn, "h", "2026-08-01", "bad").unwrap_err(),
            KimattaError::Planning { .. }
        ));
        assert_eq!(rows(&conn), 0);
    }

    #[test]
    fn a_zero_scale_is_a_typed_planned_meal_error() {
        let mut conn = open_seeded(&["h"]);
        for (numer, denom) in [(0, 1), (1, 0)] {
            let mut sent = dinner("h", "pm", "2026-08-30");
            sent.components[0].scale = Some(ScaleDto { numer, denom });
            assert!(
                matches!(
                    save_in(&mut conn, sent).unwrap_err(),
                    KimattaError::PlannedMeal { .. }
                ),
                "{numer}/{denom} must be a planned-meal error"
            );
        }
        assert_eq!(rows(&conn), 0);
    }

    #[test]
    fn list_orders_by_date_then_slot() {
        let mut conn = open_seeded(&["h"]);
        let mut lunch = dinner("h", "l", "2026-08-30");
        lunch.slot = MealSlotDto::Lunch;
        save_in(&mut conn, dinner("h", "d2", "2026-08-30")).unwrap();
        save_in(&mut conn, lunch).unwrap();
        save_in(&mut conn, dinner("h", "d1", "2026-08-29")).unwrap();
        save_in(&mut conn, dinner("h", "out", "2026-09-05")).unwrap();
        let ids: Vec<String> = list_in(&conn, "h", "2026-08-29", "2026-08-30")
            .unwrap()
            .into_iter()
            .map(|m| m.id)
            .collect();
        assert_eq!(ids, ["d1", "l", "d2"]);
    }

    #[test]
    fn known_kinds_lists_every_kind() {
        assert_eq!(
            known_meal_component_kinds(),
            [
                "recipe",
                "leftovers",
                "dining_out",
                "frozen_quick",
                "freeform",
                "open"
            ]
        );
    }
}
