//! Optional binary pantry (MVP-014): two coarse service-level commands, invariant 21.
use kimatta_storage::{Connection, HouseholdId, IngredientRef, PantryEntry};

use crate::api::error::KimattaError;
use crate::api::recipe::{ref_from_domain, ref_to_domain, IngredientRefDto};

/// One browsable identity with this household's mark. `marked` means *the household marked this
/// as one it has*; `false` means **no record** — unknown, never an assertion that the household
/// lacks it (invariant 6, PRD §10). `MVP-015`/`MVP-016` must read it that way. `ingredient`
/// reuses `recipe.rs`'s `IngredientRefDto` rather than minting a second identity type, as
/// `recipe.rs` itself reuses `restrictions.rs`'s `RestrictionDto`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PantryEntryDto {
    pub ingredient: IngredientRefDto,
    pub name: String,
    pub aliases: Vec<String>,
    pub marked: bool,
}

/// Everything this household can mark — the catalog plus its own custom ingredients — each
/// with its current mark, ordered case-insensitively by name.
pub fn list_pantry(household_id: String) -> Result<Vec<PantryEntryDto>, KimattaError> {
    crate::db::with(|conn| list_in(conn, &household_id))
}

/// Marks or unmarks one identity and returns the entry as stored, mirroring
/// `save_restrictions`'s return-what-was-stored contract so the UI shows persisted truth.
/// Idempotent in both directions.
pub fn set_pantry_mark(
    household_id: String,
    ingredient: IngredientRefDto,
    marked: bool,
) -> Result<PantryEntryDto, KimattaError> {
    crate::db::with(|conn| set_in(conn, &household_id, ingredient, marked))
}

fn to_dto(entry: PantryEntry) -> PantryEntryDto {
    PantryEntryDto {
        ingredient: ref_from_domain(&entry.ingredient),
        name: entry.name,
        aliases: entry.aliases,
        marked: entry.marked,
    }
}

/// Split out from the commands for the same reason `restrictions.rs`'s `load_in` is: they can
/// be tested with more than one household present without installing the process-wide
/// connection. The storage functions are called fully qualified because importing
/// `set_pantry_mark` by name would collide with this module's own command.
fn list_in(conn: &Connection, household_id: &str) -> Result<Vec<PantryEntryDto>, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    let entries = kimatta_storage::list_pantry_entries(conn, &id)?
        .into_iter()
        .map(to_dto)
        .collect();
    Ok(entries)
}

fn set_in(
    conn: &mut Connection,
    household_id: &str,
    ingredient: IngredientRefDto,
    marked: bool,
) -> Result<PantryEntryDto, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    let reference: IngredientRef = ref_to_domain(ingredient)?;
    let stored = kimatta_storage::set_pantry_mark(conn, &id, &reference, marked)?;
    Ok(to_dto(stored))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kimatta_storage::{
        CustomIngredient, CustomIngredientId, Household, HouseholdMember, Ingredient, IngredientId,
        MemberId,
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
        kimatta_storage::upsert_custom_ingredient(
            conn,
            &CustomIngredient::new(
                CustomIngredientId::new(format!("c-{id}")).unwrap(),
                h.id.clone(),
                "mix",
                None,
            )
            .unwrap(),
        )
        .unwrap();
    }

    /// A catalog entry with an alias, so the alias projection is exercised, plus one custom
    /// ingredient per household.
    fn open_seeded(ids: &[&str]) -> Connection {
        let mut conn = kimatta_storage::open(":memory:").unwrap();
        kimatta_storage::upsert_ingredient(
            &mut conn,
            &Ingredient::new(
                IngredientId::new("chickpeas").unwrap(),
                "chickpeas",
                vec!["garbanzo beans".to_owned()],
                None,
            )
            .unwrap(),
        )
        .unwrap();
        for id in ids {
            seed(&mut conn, id);
        }
        conn
    }

    fn catalog(id: &str) -> IngredientRefDto {
        IngredientRefDto::Catalog { id: id.to_owned() }
    }

    fn custom(id: &str) -> IngredientRefDto {
        IngredientRefDto::Custom { id: id.to_owned() }
    }

    fn marked_refs(entries: &[PantryEntryDto]) -> Vec<IngredientRefDto> {
        entries
            .iter()
            .filter(|e| e.marked)
            .map(|e| e.ingredient.clone())
            .collect()
    }

    #[test]
    fn marking_and_unmarking_round_trips_through_the_bridge() {
        let mut conn = open_seeded(&["h"]);
        let stored = set_in(&mut conn, "h", catalog("chickpeas"), true).unwrap();
        assert!(stored.marked);
        assert_eq!(stored.name, "chickpeas");
        assert_eq!(stored.aliases, vec!["garbanzo beans"]);
        assert_eq!(stored.ingredient, catalog("chickpeas"));
        assert_eq!(
            marked_refs(&list_in(&conn, "h").unwrap()),
            vec![catalog("chickpeas")]
        );

        let cleared = set_in(&mut conn, "h", catalog("chickpeas"), false).unwrap();
        assert!(!cleared.marked);
        assert!(marked_refs(&list_in(&conn, "h").unwrap()).is_empty());
    }

    /// AC-2 at the bridge: nothing marked lists every identity, never an error and never a
    /// claim the household is out of anything.
    #[test]
    fn an_empty_household_lists_the_catalog_as_unmarked() {
        let conn = open_seeded(&["h"]);
        let entries = list_in(&conn, "h").unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| !e.marked));
    }

    /// Adversarial (AC-3): two seeded households, and neither one's command may read or write
    /// the other's rows.
    #[test]
    fn commands_are_scoped_to_the_named_household() {
        let mut conn = open_seeded(&["h1", "h2"]);
        set_in(&mut conn, "h1", catalog("chickpeas"), true).unwrap();
        set_in(&mut conn, "h2", custom("c-h2"), true).unwrap();
        assert_eq!(
            marked_refs(&list_in(&conn, "h1").unwrap()),
            vec![catalog("chickpeas")]
        );
        assert_eq!(
            marked_refs(&list_in(&conn, "h2").unwrap()),
            vec![custom("c-h2")]
        );
        // h1's listing never carries h2's custom ingredient at all, marked or not.
        assert!(list_in(&conn, "h1")
            .unwrap()
            .iter()
            .all(|e| e.ingredient != custom("c-h2")));
    }

    #[test]
    fn an_unknown_ingredient_is_rejected() {
        let mut conn = open_seeded(&["h"]);
        let err = set_in(&mut conn, "h", catalog("ghost"), true).unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
        assert!(marked_refs(&list_in(&conn, "h").unwrap()).is_empty());
    }

    /// Adversarial: a foreign custom ingredient is refused by the same check a recipe line
    /// gets, so it cannot be marked through the pantry side door.
    #[test]
    fn a_custom_ingredient_of_another_household_is_rejected() {
        let mut conn = open_seeded(&["h1", "h2"]);
        let err = set_in(&mut conn, "h1", custom("c-h2"), true).unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
        assert!(marked_refs(&list_in(&conn, "h2").unwrap()).is_empty());
    }
}
