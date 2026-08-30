//! Shopping projection (MVP-015) and its editable overlay (MVP-016), invariant 21. The list
//! itself is never stored: `load_shopping_view` re-derives it and lays the household's
//! window-keyed line states and manual items over it.
use std::collections::BTreeMap;

use kimatta_storage::{
    format_civil_date, quantity_token, Connection, Contribution, HouseholdId, LineStatus,
    SeparateReason, ShoppingError, ShoppingGroup, ShoppingLine, ShoppingLineState, ShoppingList,
    ShoppingManualItem, ShoppingManualItemId,
};

use crate::api::error::KimattaError;
use crate::api::planned_meals::ScaleDto;
use crate::api::planning::{slot_from_domain, MealSlotDto};
use crate::api::recipe::{id_or_minted, quantity_from_domain, ref_from_domain, unit_from_domain};
use crate::api::recipe::{IngredientRefDto, QuantityDto, UnitDto};

/// One derived line's user state. `changed` and `checked_against` are output only: `changed`
/// is `checked && checked_against != quantity_token(line)`, so a check made against a
/// different amount is reported rather than silently kept (MVP-016 decision 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingLineStateDto {
    pub key: String,
    pub checked: bool,
    pub hidden: bool,
    pub restored: bool,
    pub changed: bool,
    pub checked_against: Option<String>,
}

/// A household-owned manual item; a blank `id` mints one, as `RecipeDto` does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingManualItemDto {
    pub id: String,
    pub household_id: String,
    pub name: String,
    pub note: Option<String>,
    pub checked: bool,
}

/// The derivation plus both overlays, read sequentially (not in one transaction): a write
/// landing between the reads shows on the next refresh. States whose key is absent from the
/// derivation are dropped from `line_states` — they stay in storage, and
/// `orphaned_line_state_count` reports how many, so the screen can say that earlier edits no
/// longer apply instead of silently losing them (the card's stop condition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingViewDto {
    pub list: ShoppingListDto,
    pub line_states: Vec<ShoppingLineStateDto>,
    pub manual_items: Vec<ShoppingManualItemDto>,
    pub orphaned_line_state_count: u32,
}

pub fn load_shopping_view(
    household_id: String,
    from_date: String,
    to_date: String,
) -> Result<ShoppingViewDto, KimattaError> {
    crate::db::with(|conn| load_view_in(conn, &household_id, &from_date, &to_date))
}

/// Stores one line's state. The caller sends only the key and the flags: Rust re-derives the
/// list and takes the token from the line that key names, so a check can never be bound to an
/// amount the user did not see, and a key absent from the derivation is a typed error rather
/// than an invisible row. Returns what was stored.
pub fn set_shopping_line_state(
    household_id: String,
    from_date: String,
    to_date: String,
    state: ShoppingLineStateDto,
) -> Result<ShoppingLineStateDto, KimattaError> {
    crate::db::with(|conn| set_state_in(conn, &household_id, &from_date, &to_date, state))
}

pub fn save_shopping_manual_item(
    item: ShoppingManualItemDto,
) -> Result<ShoppingManualItemDto, KimattaError> {
    crate::db::with(|conn| save_item_in(conn, item))
}

pub fn delete_shopping_manual_item(
    household_id: String,
    item_id: String,
) -> Result<(), KimattaError> {
    crate::db::with(|conn| delete_item_in(conn, &household_id, &item_id))
}

/// "Start over": clears this window's line states and the checked manual items; unchecked
/// items and the derivation itself are untouched.
pub fn reset_shopping_list(
    household_id: String,
    from_date: String,
    to_date: String,
) -> Result<(), KimattaError> {
    crate::db::with(|conn| reset_in(conn, &household_id, &from_date, &to_date))
}

fn item_from_domain(i: ShoppingManualItem) -> ShoppingManualItemDto {
    ShoppingManualItemDto {
        id: i.id.as_str().to_owned(),
        household_id: i.household_id.as_str().to_owned(),
        name: i.name,
        note: i.note,
        checked: i.checked,
    }
}

fn load_view_in(
    conn: &mut Connection,
    household_id: &str,
    from_date: &str,
    to_date: &str,
) -> Result<ShoppingViewDto, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let from = kimatta_storage::parse_civil_date(from_date)?;
    let to = kimatta_storage::parse_civil_date(to_date)?;
    let list = kimatta_storage::load_shopping_list(conn, &household, from, to)?;
    let tokens: BTreeMap<&str, String> = list
        .groups
        .iter()
        .flat_map(|g| g.lines.iter())
        .map(|l| (l.key.as_str(), quantity_token(&l.quantity, &l.unit)))
        .collect();
    let mut orphaned_line_state_count: u32 = 0;
    let line_states = kimatta_storage::list_shopping_line_states(conn, &household, from, to)?
        .into_iter()
        .filter_map(|s| {
            let Some(token) = tokens.get(s.key.as_str()) else {
                orphaned_line_state_count += 1;
                return None;
            };
            Some(ShoppingLineStateDto {
                changed: s.checked && s.checked_against.as_deref() != Some(token.as_str()),
                key: s.key,
                checked: s.checked,
                hidden: s.hidden,
                restored: s.restored,
                checked_against: s.checked_against,
            })
        })
        .collect();
    let manual_items = kimatta_storage::list_shopping_manual_items(conn, &household)?
        .into_iter()
        .map(item_from_domain)
        .collect();
    Ok(ShoppingViewDto {
        list: from_domain(&list),
        line_states,
        manual_items,
        orphaned_line_state_count,
    })
}

fn set_state_in(
    conn: &mut Connection,
    household_id: &str,
    from_date: &str,
    to_date: &str,
    state: ShoppingLineStateDto,
) -> Result<ShoppingLineStateDto, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let from = kimatta_storage::parse_civil_date(from_date)?;
    let to = kimatta_storage::parse_civil_date(to_date)?;
    if state.key.trim().is_empty() {
        return Err(ShoppingError::BlankKey.into());
    }
    // A state with no flag set is a clear, which storage turns into a DELETE: it needs no
    // token, so it skips the derivation lookup rather than being rejected by it. That keeps
    // clearing possible for a key the derivation no longer names — but a caller has to know
    // the key to use it, and the view reports orphans only as a count, so from Dart today
    // `reset_shopping_list` is still the only reachable way to drop them. Surfacing the keys
    // is the UI half's call; this path is what makes a per-orphan command possible at all.
    let clearing = !state.checked && !state.hidden && !state.restored;
    let token = if clearing {
        None
    } else {
        let list = kimatta_storage::load_shopping_list(conn, &household, from, to)?;
        let found = list
            .groups
            .iter()
            .flat_map(|g| g.lines.iter())
            .find(|l| l.key == state.key)
            .ok_or_else(|| ShoppingError::UnknownLineKey(state.key.clone()))?;
        Some(quantity_token(&found.quantity, &found.unit))
    };
    let stored = kimatta_storage::set_shopping_line_state(
        conn,
        &household,
        from,
        to,
        &ShoppingLineState {
            key: state.key,
            checked: state.checked,
            checked_against: token.filter(|_| state.checked),
            hidden: state.hidden,
            restored: state.restored,
        },
    )?;
    Ok(ShoppingLineStateDto {
        key: stored.key,
        checked: stored.checked,
        hidden: stored.hidden,
        restored: stored.restored,
        changed: false,
        checked_against: stored.checked_against,
    })
}

fn save_item_in(
    conn: &mut Connection,
    item: ShoppingManualItemDto,
) -> Result<ShoppingManualItemDto, KimattaError> {
    if item.name.trim().is_empty() {
        return Err(ShoppingError::BlankName.into());
    }
    let stored = kimatta_storage::save_shopping_manual_item(
        conn,
        &ShoppingManualItem {
            id: ShoppingManualItemId::new(id_or_minted(item.id))?,
            household_id: HouseholdId::new(item.household_id)?,
            name: item.name,
            note: item.note,
            checked: item.checked,
        },
    )?;
    Ok(item_from_domain(stored))
}

fn delete_item_in(
    conn: &mut Connection,
    household_id: &str,
    item_id: &str,
) -> Result<(), KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let id = ShoppingManualItemId::new(item_id)?;
    Ok(kimatta_storage::delete_shopping_manual_item(
        conn, &household, &id,
    )?)
}

fn reset_in(
    conn: &mut Connection,
    household_id: &str,
    from_date: &str,
    to_date: &str,
) -> Result<(), KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let from = kimatta_storage::parse_civil_date(from_date)?;
    let to = kimatta_storage::parse_civil_date(to_date)?;
    Ok(kimatta_storage::reset_shopping_list(
        conn, &household, from, to,
    )?)
}

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

    /// The same ingredient in the other unit family, so a derivation can carry two flour
    /// lines at once (MVP-015 merges per family, not per ingredient).
    fn gram(n: u32) -> IngredientLineDto {
        line(
            &format!("{n} g flour"),
            QuantityDto::Exact { numer: n, denom: 1 },
            UnitDto::Known {
                unit: "g".to_owned(),
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

    // --- MVP-016 step 7: the view and its commands -----------------------------------------

    const FLOUR_KEY: &str = "m:catalog:flour:volume_us:req:known";
    const GRAM_FLOUR_KEY: &str = "m:catalog:flour:mass_metric:req:known";

    fn state(key: &str, checked: bool, hidden: bool, restored: bool) -> ShoppingLineStateDto {
        ShoppingLineStateDto {
            key: key.to_owned(),
            checked,
            hidden,
            restored,
            changed: false,
            checked_against: None,
        }
    }

    fn item_dto(id: &str, household: &str, name: &str, checked: bool) -> ShoppingManualItemDto {
        ShoppingManualItemDto {
            id: id.to_owned(),
            household_id: household.to_owned(),
            name: name.to_owned(),
            note: None,
            checked,
        }
    }

    fn view(conn: &mut Connection, household: &str) -> ShoppingViewDto {
        load_view_in(conn, household, "2026-08-29", "2026-09-04").unwrap()
    }

    /// AC-1: the view carries the MVP-015 derivation unchanged plus both overlays.
    #[test]
    fn a_view_carries_the_derivation_and_both_overlays() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        let stored = set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(FLOUR_KEY, true, false, false),
        )
        .unwrap();
        assert_eq!(
            stored.checked_against.as_deref(),
            Some("exact:1/1|known:cup")
        );
        let item = save_item_in(&mut conn, item_dto("", "h", " batteries ", false)).unwrap();
        assert!(!item.id.is_empty(), "a blank id mints one");
        assert_eq!(item.name, "batteries");
        let v = view(&mut conn, "h");
        assert_eq!(
            v.list,
            derive_in(&mut conn, "h", "2026-08-29", "2026-09-04").unwrap()
        );
        assert_eq!(v.line_states, vec![stored]);
        assert_eq!(v.manual_items, vec![item]);
        assert_eq!(v.orphaned_line_state_count, 0, "nothing was orphaned");
    }

    /// F1 pin: a second meal doubles the flour, so the check made against 1 cup is reported
    /// as changed, never kept.
    #[test]
    fn a_checked_line_whose_quantity_changed_is_reported_changed() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(FLOUR_KEY, true, false, false),
        )
        .unwrap();
        assert!(!view(&mut conn, "h").line_states[0].changed);
        plan(&mut conn, "h", "pm-2", "2026-08-30", "r", None);
        let s = &view(&mut conn, "h").line_states[0];
        assert!(s.changed, "{s:?}");
        assert!(s.checked);
        assert_eq!(s.checked_against.as_deref(), Some("exact:1/1|known:cup"));
        // Re-checking clears it: the same call now picks up the derivation's new 2 cups.
        set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(FLOUR_KEY, true, false, false),
        )
        .unwrap();
        assert!(!view(&mut conn, "h").line_states[0].changed);
    }

    #[test]
    fn an_orphaned_state_is_not_in_the_view() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(FLOUR_KEY, false, true, false),
        )
        .unwrap();
        kimatta_storage::delete_planned_meal(
            &mut conn,
            &HouseholdId::new("h").unwrap(),
            &kimatta_storage::PlannedMealId::new("pm-1").unwrap(),
            kimatta_storage::WriteSource::User,
        )
        .unwrap();
        let v = view(&mut conn, "h");
        assert!(v.list.groups.is_empty());
        assert!(v.line_states.is_empty());
        let kept = kimatta_storage::list_shopping_line_states(
            &conn,
            &HouseholdId::new("h").unwrap(),
            kimatta_storage::parse_civil_date("2026-08-29").unwrap(),
            kimatta_storage::parse_civil_date("2026-09-04").unwrap(),
        )
        .unwrap();
        assert_eq!(kept.len(), 1, "the row stays in storage");
        assert_eq!(
            v.orphaned_line_state_count, 1,
            "and the view says so rather than dropping it silently"
        );
    }

    /// The orphan the previous test leaves is still clearable one line at a time: a state with
    /// no flag set is a delete, so it skips the derivation lookup that would reject its key.
    #[test]
    fn an_orphaned_state_can_still_be_cleared_by_key() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(FLOUR_KEY, false, true, false),
        )
        .unwrap();
        kimatta_storage::delete_planned_meal(
            &mut conn,
            &HouseholdId::new("h").unwrap(),
            &kimatta_storage::PlannedMealId::new("pm-1").unwrap(),
            kimatta_storage::WriteSource::User,
        )
        .unwrap();
        assert_eq!(view(&mut conn, "h").orphaned_line_state_count, 1);
        set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(FLOUR_KEY, false, false, false),
        )
        .unwrap();
        assert_eq!(view(&mut conn, "h").orphaned_line_state_count, 0);
    }

    /// `hidden` and `restored` are opposites, so the view must carry each back on the flag it
    /// was written on. Asserted asymmetrically on a *live* line: every other test either
    /// leaves both false or orphans the row before the view reads it, which would let the two
    /// be swapped in `load_view_in` without any test noticing.
    #[test]
    fn hidden_and_restored_come_back_on_the_flags_they_were_written_on() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r-cup", vec![cup(1)]);
        save_recipe(&mut conn, "h", "r-gram", vec![gram(500)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r-cup", None);
        plan(&mut conn, "h", "pm-2", "2026-08-30", "r-gram", None);
        for (key, hidden, restored) in [(FLOUR_KEY, true, false), (GRAM_FLOUR_KEY, false, true)] {
            set_state_in(
                &mut conn,
                "h",
                "2026-08-29",
                "2026-09-04",
                state(key, false, hidden, restored),
            )
            .unwrap();
        }
        let v = view(&mut conn, "h");
        assert_eq!(v.orphaned_line_state_count, 0, "both lines are live");
        let cups = v.line_states.iter().find(|s| s.key == FLOUR_KEY).unwrap();
        assert!(cups.hidden && !cups.restored, "{cups:?}");
        let grams = v
            .line_states
            .iter()
            .find(|s| s.key == GRAM_FLOUR_KEY)
            .unwrap();
        assert!(grams.restored && !grams.hidden, "{grams:?}");
    }

    /// F1 pin: a key the derivation does not name is a typed error, not a row that is written
    /// and then silently filtered out of every later view.
    #[test]
    fn a_state_for_a_key_not_in_the_derivation_is_a_typed_error() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        let err = set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state("m:catalog:sugar:mass_metric:req:known", true, false, false),
        )
        .unwrap_err();
        assert!(matches!(err, KimattaError::Shopping { .. }), "{err:?}");
        let rows = kimatta_storage::list_shopping_line_states(
            &conn,
            &HouseholdId::new("h").unwrap(),
            kimatta_storage::parse_civil_date("2026-08-29").unwrap(),
            kimatta_storage::parse_civil_date("2026-09-04").unwrap(),
        )
        .unwrap();
        assert!(rows.is_empty(), "no invisible row was written: {rows:?}");
    }

    /// F1 adversarial: flour merges per unit family, so two lines can name the same
    /// ingredient. The check must be bound to the amount of the line its key names, not to
    /// whatever the caller happened to hold.
    #[test]
    fn a_check_is_bound_to_the_line_its_key_names() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r-cup", vec![cup(2)]);
        save_recipe(&mut conn, "h", "r-gram", vec![gram(500)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r-cup", None);
        plan(&mut conn, "h", "pm-2", "2026-08-30", "r-gram", None);
        let stored = set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(GRAM_FLOUR_KEY, true, false, false),
        )
        .unwrap();
        assert_eq!(
            stored.checked_against.as_deref(),
            Some("exact:500/1|known:g")
        );
        let v = view(&mut conn, "h");
        let s = v
            .line_states
            .iter()
            .find(|s| s.key == GRAM_FLOUR_KEY)
            .unwrap();
        assert!(
            !s.changed,
            "the grams line is checked against its own amount"
        );
    }

    /// AC-4, adversarial: h2's overlay never reaches h1's view, and h2 cannot edit h1's item.
    #[test]
    fn overlay_commands_are_scoped_to_the_named_household() {
        let mut conn = open_seeded(&["h1", "h2"]);
        save_recipe(&mut conn, "h1", "r1", vec![cup(1)]);
        save_recipe(&mut conn, "h2", "r2", vec![cup(1)]);
        plan(&mut conn, "h1", "pm-1", "2026-08-29", "r1", None);
        plan(&mut conn, "h2", "pm-2", "2026-08-29", "r2", None);
        set_state_in(
            &mut conn,
            "h2",
            "2026-08-29",
            "2026-09-04",
            state(FLOUR_KEY, true, false, false),
        )
        .unwrap();
        let theirs = save_item_in(&mut conn, item_dto("mi-2", "h2", "theirs", false)).unwrap();
        let v1 = view(&mut conn, "h1");
        assert!(v1.line_states.is_empty());
        assert!(v1.manual_items.is_empty());
        let err = save_item_in(&mut conn, item_dto("mi-2", "h1", "hijack", true)).unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
        let err = delete_item_in(&mut conn, "h1", "mi-2").unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
        reset_in(&mut conn, "h1", "2026-08-29", "2026-09-04").unwrap();
        let v2 = view(&mut conn, "h2");
        assert_eq!(v2.line_states.len(), 1);
        assert_eq!(v2.manual_items, vec![theirs]);
        let err = load_view_in(&mut conn, "ghost", "2026-08-29", "2026-09-04").unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
    }

    #[test]
    fn a_blank_manual_name_is_a_typed_shopping_error() {
        let mut conn = open_seeded(&["h"]);
        let err = save_item_in(&mut conn, item_dto("", "h", "  ", false)).unwrap_err();
        assert!(matches!(err, KimattaError::Shopping { .. }), "{err:?}");
        assert!(view(&mut conn, "h").manual_items.is_empty());
    }

    #[test]
    fn a_blank_line_key_is_a_typed_shopping_error() {
        let mut conn = open_seeded(&["h"]);
        let err = set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(" ", true, false, false),
        )
        .unwrap_err();
        assert!(matches!(err, KimattaError::Shopping { .. }), "{err:?}");
    }

    /// AC-3 at the bridge: a bulk mark flips the line to omitted; sending the returned refs
    /// back with `marked = false` restores it.
    #[test]
    fn bulk_pantry_marks_flip_lines_to_omitted_and_back() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        let flour = IngredientRefDto::Catalog {
            id: "flour".to_owned(),
        };
        let changed =
            crate::api::pantry::set_many_in(&mut conn, "h", vec![flour.clone()], true).unwrap();
        assert_eq!(changed, vec![flour.clone()]);
        assert_eq!(
            view(&mut conn, "h").list.groups[0].lines[0].status,
            ShoppingLineStatusDto::OmittedPantryMarked
        );
        // Idempotent: a second mark changes nothing and returns nothing.
        assert!(
            crate::api::pantry::set_many_in(&mut conn, "h", vec![flour.clone()], true)
                .unwrap()
                .is_empty()
        );
        let undone = crate::api::pantry::set_many_in(&mut conn, "h", changed, false).unwrap();
        assert_eq!(undone, vec![flour]);
        assert_eq!(
            view(&mut conn, "h").list.groups[0].lines[0].status,
            ShoppingLineStatusDto::Needed
        );
    }

    #[test]
    fn reset_leaves_the_derivation_and_unchecked_items() {
        let mut conn = open_seeded(&["h"]);
        save_recipe(&mut conn, "h", "r", vec![cup(1)]);
        plan(&mut conn, "h", "pm-1", "2026-08-29", "r", None);
        set_state_in(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            state(FLOUR_KEY, true, false, false),
        )
        .unwrap();
        save_item_in(&mut conn, item_dto("a", "h", "bought", true)).unwrap();
        let kept = save_item_in(&mut conn, item_dto("b", "h", "wanted", false)).unwrap();
        reset_in(&mut conn, "h", "2026-08-29", "2026-09-04").unwrap();
        let v = view(&mut conn, "h");
        assert_eq!(v.list.contribution_count, 1);
        assert!(v.line_states.is_empty());
        assert_eq!(v.manual_items, vec![kept]);
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
