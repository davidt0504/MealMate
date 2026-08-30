//! Shopping projection (MVP-015): one coarse read-only query, invariant 21. Nothing here is
//! stored — MVP-016 owns persistence, edits and restore.
use kimatta_storage::{
    format_civil_date, Connection, Contribution, HouseholdId, LineStatus, SeparateReason,
    ShoppingGroup, ShoppingLine, ShoppingList,
};

use crate::api::error::KimattaError;
use crate::api::planned_meals::ScaleDto;
use crate::api::planning::{slot_from_domain, MealSlotDto};
use crate::api::recipe::{quantity_from_domain, ref_from_domain, unit_from_domain};
use crate::api::recipe::{IngredientRefDto, QuantityDto, UnitDto};

/// One recipe line × one component's scale. `scale` reuses `ScaleDto` so it crosses as an
/// integer pair rather than an opaque handle; `None` is 1×.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributionDto {
    pub planned_meal_id: String,
    /// ISO civil date.
    pub date: String,
    pub slot: MealSlotDto,
    pub component_position: u32,
    pub recipe_id: String,
    pub recipe_title: String,
    pub line_position: u32,
    pub original_text: String,
    pub scale: Option<ScaleDto>,
}

/// Mirrors `food_domain::SeparateReason`; see its doc for when each applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparateReasonDto {
    Unresolved,
    UnitNotCombinable,
    UnknownQuantity,
    ArithmeticOverflow,
}

/// `OmittedPantryMarked` lines are still returned in full so the UI can offer restore; a
/// pantry mark suppresses a purchase but never proves quantity (invariant 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShoppingLineStatusDto {
    Needed,
    OmittedPantryMarked,
}

/// `key` is stable across regenerations of the same snapshot: `m:…` for a merged group,
/// `s:…` for a per-contribution line. MVP-016 keys its edits on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingLineDto {
    pub key: String,
    pub name: String,
    pub ingredient: Option<IngredientRefDto>,
    pub quantity: QuantityDto,
    pub unit: UnitDto,
    pub optional: bool,
    pub status: ShoppingLineStatusDto,
    pub separate_reason: Option<SeparateReasonDto>,
    pub contributions: Vec<ContributionDto>,
}

/// `category` is the raw store category; `None` is "uncategorised" and comes last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingGroupDto {
    pub category: Option<String>,
    pub lines: Vec<ShoppingLineDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingListDto {
    pub algorithm_version: u32,
    pub from_date: String,
    pub to_date: String,
    pub groups: Vec<ShoppingGroupDto>,
    pub non_recipe_components: u32,
    pub contribution_count: u32,
}

/// Derives (never stores) the pantry-aware shopping projection for `from_date..=to_date`.
/// Dates are caller-supplied civil dates: Rust never reads the clock (invariant 20).
pub fn derive_shopping_list(
    household_id: String,
    from_date: String,
    to_date: String,
) -> Result<ShoppingListDto, KimattaError> {
    crate::db::with(|conn| derive_in(conn, &household_id, &from_date, &to_date))
}

fn contribution_from_domain(c: &Contribution) -> ContributionDto {
    ContributionDto {
        planned_meal_id: c.planned_meal_id.as_str().to_owned(),
        date: format_civil_date(c.date),
        slot: slot_from_domain(c.slot),
        component_position: c.component_position as u32,
        recipe_id: c.recipe_id.as_str().to_owned(),
        recipe_title: c.recipe_title.clone(),
        line_position: c.line_position as u32,
        original_text: c.original_text.clone(),
        scale: c.scale.map(|s| ScaleDto {
            numer: s.numer(),
            denom: s.denom(),
        }),
    }
}

fn reason_from_domain(r: SeparateReason) -> SeparateReasonDto {
    match r {
        SeparateReason::Unresolved => SeparateReasonDto::Unresolved,
        SeparateReason::UnitNotCombinable => SeparateReasonDto::UnitNotCombinable,
        SeparateReason::UnknownQuantity => SeparateReasonDto::UnknownQuantity,
        SeparateReason::ArithmeticOverflow => SeparateReasonDto::ArithmeticOverflow,
    }
}

fn status_from_domain(s: LineStatus) -> ShoppingLineStatusDto {
    match s {
        LineStatus::Needed => ShoppingLineStatusDto::Needed,
        LineStatus::OmittedPantryMarked => ShoppingLineStatusDto::OmittedPantryMarked,
    }
}

fn line_from_domain(l: &ShoppingLine) -> ShoppingLineDto {
    ShoppingLineDto {
        key: l.key.clone(),
        name: l.name.clone(),
        ingredient: l.ingredient.as_ref().map(ref_from_domain),
        quantity: quantity_from_domain(l.quantity),
        unit: unit_from_domain(&l.unit),
        optional: l.optional,
        status: status_from_domain(l.status),
        separate_reason: l.separate_reason.map(reason_from_domain),
        contributions: l
            .contributions
            .iter()
            .map(contribution_from_domain)
            .collect(),
    }
}

fn group_from_domain(g: &ShoppingGroup) -> ShoppingGroupDto {
    ShoppingGroupDto {
        category: g.category.clone(),
        lines: g.lines.iter().map(line_from_domain).collect(),
    }
}

fn from_domain(list: &ShoppingList) -> ShoppingListDto {
    ShoppingListDto {
        algorithm_version: list.algorithm_version,
        from_date: format_civil_date(list.from),
        to_date: format_civil_date(list.to),
        groups: list.groups.iter().map(group_from_domain).collect(),
        non_recipe_components: list.non_recipe_components,
        contribution_count: list.contribution_count,
    }
}

/// Split out from the command, as `list_in` is, so it can be tested with more than one
/// household present. Dates are parsed before storage is touched, so a malformed one is a
/// typed `Planning` error (the `archive_recipe_in` precedent).
fn derive_in(
    conn: &mut Connection,
    household_id: &str,
    from_date: &str,
    to_date: &str,
) -> Result<ShoppingListDto, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let from = kimatta_storage::parse_civil_date(from_date)?;
    let to = kimatta_storage::parse_civil_date(to_date)?;
    let list = kimatta_storage::load_shopping_list(conn, &household, from, to)?;
    Ok(from_domain(&list))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::planned_meals::{MealComponentDto, PlannedMealDto};
    use crate::api::recipe::{IngredientLineDto, RecipeDto, RecipeProvenanceDto};
    use kimatta_storage::{
        Household, HouseholdMember, Ingredient, IngredientId, IngredientRef, MealScope, MealSlot,
        MemberId, PlanningCycle,
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
    }

    fn open_seeded(ids: &[&str]) -> Connection {
        let mut conn = kimatta_storage::open(":memory:").unwrap();
        kimatta_storage::upsert_ingredient(
            &mut conn,
            &Ingredient::new(
                IngredientId::new("flour").unwrap(),
                "flour",
                vec![],
                Some("baking".to_owned()),
            )
            .unwrap(),
        )
        .unwrap();
        for id in ids {
            seed(&mut conn, id);
        }
        conn
    }

    fn line(original: &str, quantity: QuantityDto, unit: UnitDto) -> IngredientLineDto {
        IngredientLineDto {
            original_text: original.to_owned(),
            name: original.to_owned(),
            ingredient: Some(IngredientRefDto::Catalog {
                id: "flour".to_owned(),
            }),
            quantity,
            unit,
            preparation: None,
            optional: false,
        }
    }

    fn save_recipe(
        conn: &mut Connection,
        household: &str,
        id: &str,
        lines: Vec<IngredientLineDto>,
    ) {
        let dto = RecipeDto {
            id: id.to_owned(),
            household_id: household.to_owned(),
            title: format!("Recipe {id}"),
            servings: Some(2),
            prep_minutes: None,
            instructions: String::new(),
            lines,
            provenance: RecipeProvenanceDto {
                kind: "authored".to_owned(),
                source_url: None,
                source_name: None,
                source_author: None,
                rights_basis: None,
                attribution: None,
                modifications: None,
                verified_on: None,
                starter_slug: None,
            },
            archived_at: None,
            assessment: None,
        };
        crate::api::recipe::save_recipe_in(conn, dto).unwrap();
    }

    fn plan(
        conn: &mut Connection,
        household: &str,
        id: &str,
        date: &str,
        recipe: &str,
        scale: Option<ScaleDto>,
    ) {
        let dto = PlannedMealDto {
            id: id.to_owned(),
            household_id: household.to_owned(),
            date: date.to_owned(),
            slot: MealSlotDto::Dinner,
            components: vec![
                MealComponentDto {
                    kind: "recipe".to_owned(),
                    recipe_id: Some(recipe.to_owned()),
                    note: None,
                    scale,
                },
                MealComponentDto {
                    kind: "leftovers".to_owned(),
                    recipe_id: None,
                    note: None,
                    scale: None,
                },
            ],
            locked: false,
        };
        crate::api::planned_meals::save_in(conn, dto).unwrap();
    }

    fn cup(n: u32) -> IngredientLineDto {
        line(
            &format!("{n} cup flour"),
            QuantityDto::Exact { numer: n, denom: 1 },
            UnitDto::Known {
                unit: "cup".to_owned(),
            },
        )
    }

    #[test]
    fn a_planned_cycle_derives_a_grouped_list_through_the_bridge() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        plan(
            &mut conn,
            "h",
            "pm-2",
            "2026-08-30",
            "r",
            Some(ScaleDto { numer: 3, denom: 2 }),
        );
        let list = derive_in(&mut conn, "h", "2026-08-29", "2026-09-04").unwrap();
        assert_eq!(list.algorithm_version, 1);
        assert_eq!(list.from_date, "2026-08-29");
        assert_eq!(list.to_date, "2026-09-04");
        assert_eq!(list.contribution_count, 2);
        assert_eq!(list.non_recipe_components, 2);
        assert_eq!(list.groups.len(), 1);
        assert_eq!(list.groups[0].category.as_deref(), Some("baking"));
        let l = &list.groups[0].lines[0];
        assert_eq!(l.key, "m:catalog:flour:volume_us:req:known");
        assert_eq!(l.quantity, QuantityDto::Exact { numer: 5, denom: 2 });
        assert_eq!(l.status, ShoppingLineStatusDto::Needed);
        assert_eq!(l.separate_reason, None);
        assert_eq!(l.contributions.len(), 2);
        assert_eq!(l.contributions[0].planned_meal_id, "pm-1");
        assert_eq!(l.contributions[0].scale, None);
        assert_eq!(
            l.contributions[1].scale,
            Some(ScaleDto { numer: 3, denom: 2 })
        );
        assert_eq!(l.contributions[1].date, "2026-08-30");
        assert_eq!(l.contributions[1].slot, MealSlotDto::Dinner);
        assert_eq!(l.contributions[1].original_text, "1 cup flour");
        // Determinism at the bridge: the same read twice is equal.
        assert_eq!(
            derive_in(&mut conn, "h", "2026-08-29", "2026-09-04").unwrap(),
            list
        );
    }

    #[test]
    fn dto_mapping_round_trips_every_unit_and_quantity_shape() {
        let mut conn = open_seeded(&["h"]);
        let mut unresolved = line("a splash", QuantityDto::Unknown, UnitDto::None);
        unresolved.ingredient = None;
        let mut other = line(
            "2-3 handfuls flour",
            QuantityDto::Range {
                min_numer: 2,
                min_denom: 1,
                max_numer: 3,
                max_denom: 1,
            },
            UnitDto::Other {
                text: "handful".to_owned(),
            },
        );
        other.optional = true;
        save_recipe(&mut conn, "h", "r", vec![cup(1), other, unresolved]);
        plan(
            &mut conn,
            "h",
            "pm-1",
            "2026-08-29",
            "r",
            Some(ScaleDto { numer: 2, denom: 1 }),
        );
        kimatta_storage::set_pantry_mark(
            &mut conn,
            &HouseholdId::new("h").unwrap(),
            &IngredientRef::Catalog(IngredientId::new("flour").unwrap()),
            true,
        )
        .unwrap();
        let list = derive_in(&mut conn, "h", "2026-08-29", "2026-08-29").unwrap();
        assert_eq!(list.groups.len(), 2);
        let baking = &list.groups[0].lines;
        assert_eq!(baking.len(), 2);
        let range_line = baking.iter().find(|l| l.optional).unwrap();
        assert_eq!(
            range_line.quantity,
            QuantityDto::Range {
                min_numer: 4,
                min_denom: 1,
                max_numer: 6,
                max_denom: 1
            }
        );
        assert_eq!(
            range_line.unit,
            UnitDto::Other {
                text: "handful".to_owned()
            }
        );
        assert_eq!(
            range_line.status,
            ShoppingLineStatusDto::OmittedPantryMarked
        );
        assert_eq!(range_line.separate_reason, None);
        assert_eq!(range_line.key, "m:catalog:flour:other=handful:opt:known");
        assert_eq!(
            range_line.contributions[0].scale,
            Some(ScaleDto { numer: 2, denom: 1 })
        );
        let cup_line = baking.iter().find(|l| !l.optional).unwrap();
        assert_eq!(cup_line.quantity, QuantityDto::Exact { numer: 2, denom: 1 });
        assert_eq!(cup_line.status, ShoppingLineStatusDto::OmittedPantryMarked);
        let uncategorised = &list.groups[1];
        assert_eq!(uncategorised.category, None);
        let s = &uncategorised.lines[0];
        assert_eq!(s.key, "s:pm-1:0:2");
        assert_eq!(s.quantity, QuantityDto::Unknown);
        assert_eq!(s.unit, UnitDto::None);
        assert_eq!(s.ingredient, None);
        assert_eq!(s.separate_reason, Some(SeparateReasonDto::Unresolved));
        assert_eq!(s.status, ShoppingLineStatusDto::Needed);
    }

    /// Adversarial: h2's meals and marks never reach h1's list, and an unknown household is a
    /// typed storage error rather than an empty list.
    #[test]
    fn commands_are_scoped_to_the_named_household() {
        let mut conn = open_seeded(&["h1", "h2"]);
        save_recipe(&mut conn, "h1", "r1", vec![cup(1)]);
        save_recipe(&mut conn, "h2", "r2", vec![cup(7)]);
        plan(&mut conn, "h1", "pm-1", "2026-08-29", "r1", None);
        plan(&mut conn, "h2", "pm-2", "2026-08-29", "r2", None);
        kimatta_storage::set_pantry_mark(
            &mut conn,
            &HouseholdId::new("h2").unwrap(),
            &IngredientRef::Catalog(IngredientId::new("flour").unwrap()),
            true,
        )
        .unwrap();
        let l1 = derive_in(&mut conn, "h1", "2026-08-29", "2026-08-29").unwrap();
        assert_eq!(l1.contribution_count, 1);
        let line = &l1.groups[0].lines[0];
        assert_eq!(line.quantity, QuantityDto::Exact { numer: 1, denom: 1 });
        assert_eq!(line.status, ShoppingLineStatusDto::Needed);
        assert_eq!(line.contributions[0].planned_meal_id, "pm-1");
        let err = derive_in(&mut conn, "ghost", "2026-08-29", "2026-08-29").unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
    }

    #[test]
    fn a_non_civil_date_is_a_typed_planning_error() {
        let mut conn = open_seeded(&["h"]);
        for (from, to) in [
            ("2026-08-29T23:00:00Z", "2026-08-29"),
            ("2026-08-29", "20260830"),
            ("", "2026-08-29"),
        ] {
            let err = derive_in(&mut conn, "h", from, to).unwrap_err();
            assert!(
                matches!(err, KimattaError::Planning { .. }),
                "{from:?}..{to:?} gave {err:?}"
            );
        }
    }

    #[test]
    fn an_inverted_range_is_empty_not_an_error() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        let list = derive_in(&mut conn, "h", "2026-08-30", "2026-08-29").unwrap();
        assert_eq!(list.contribution_count, 0);
        assert!(list.groups.is_empty());
        assert_eq!(list.from_date, "2026-08-30");
        assert_eq!(list.to_date, "2026-08-29");
    }
}
