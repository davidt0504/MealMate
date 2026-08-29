use kimatta_storage::{
    format_civil_date, Connection, CustomIngredient, CustomIngredientId, HouseholdId, IngredientId,
    IngredientLine, IngredientRef, ProvenanceKind, Quantity, QuantityRange, Rational, Recipe,
    RecipeId, RecipeListing, RecipeProvenance, RecipeRecord, StorageError, Unit, UnitKind,
};
use uuid::Uuid;

use crate::api::error::KimattaError;

/// Exact rationals cross as integer pairs; `Unknown` is the honest absent amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityDto {
    Unknown,
    Exact {
        numer: u32,
        denom: u32,
    },
    Range {
        min_numer: u32,
        min_denom: u32,
        max_numer: u32,
        max_denom: u32,
    },
}

/// `unit` is a `UnitKind` string (`tsp`, `cup`, …) rather than an enum-of-enum, so the
/// vocabulary MVP-015 may extend does not double the bridge surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitDto {
    None,
    Known { unit: String },
    Other { text: String },
}

/// `Catalog` names a row in the global ingredient catalog, which MVP-011 seeds; no bridge
/// command creates catalog rows yet, so a `Catalog` ref sent before then fails
/// `check_line_refs` as `NoSuchIngredient`. `Custom` is the only variant MVP-008 can populate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngredientRefDto {
    Catalog { id: String },
    Custom { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngredientLineDto {
    pub original_text: String,
    pub name: String,
    pub ingredient: Option<IngredientRefDto>,
    pub quantity: QuantityDto,
    pub unit: UnitDto,
    pub preparation: Option<String>,
    pub optional: bool,
}

/// `kind` is a `ProvenanceKind` string: `authored`, `imported` or `starter`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeProvenanceDto {
    pub kind: String,
    pub source_url: Option<String>,
    pub source_name: Option<String>,
    pub source_author: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDto {
    pub id: String,
    pub household_id: String,
    pub title: String,
    pub servings: Option<u32>,
    pub instructions: String,
    pub lines: Vec<IngredientLineDto>,
    pub provenance: RecipeProvenanceDto,
    /// Output only: `save_recipe` ignores it and never changes archive state; use
    /// `archive_recipe`/`restore_recipe`. ISO civil date, `None` while in the library.
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeSummaryDto {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomIngredientDto {
    pub id: String,
    pub household_id: String,
    pub name: String,
    pub store_category: Option<String>,
}

/// Saves the whole recipe; an empty `id` mints a v4 UUID (as `bootstrap_household` does).
/// Returns what was stored.
pub fn save_recipe(recipe: RecipeDto) -> Result<RecipeDto, KimattaError> {
    crate::db::with(|conn| save_recipe_in(conn, recipe))
}

/// The recipe in exactly `household_id`, or `None` — another household's recipe is `None`.
pub fn load_recipe(
    household_id: String,
    recipe_id: String,
) -> Result<Option<RecipeDto>, KimattaError> {
    crate::db::with(|conn| load_recipe_in(conn, &household_id, &recipe_id))
}

/// The household's library: active recipes only.
pub fn list_recipes(household_id: String) -> Result<Vec<RecipeSummaryDto>, KimattaError> {
    crate::db::with(|conn| list_recipes_in(conn, &household_id))
}

/// Recipes the household has archived, so they can be restored.
pub fn list_archived_recipes(household_id: String) -> Result<Vec<RecipeSummaryDto>, KimattaError> {
    crate::db::with(|conn| list_archived_recipes_in(conn, &household_id))
}

/// "Delete" (owner decision 2026-08-28: archive, never hard-delete). `archived_on` is the
/// local civil date, supplied by Dart because Rust never reads the clock (invariant 20).
/// Idempotent; returns the recipe as stored, with `archived_at` set.
pub fn archive_recipe(
    household_id: String,
    recipe_id: String,
    archived_on: String,
) -> Result<RecipeDto, KimattaError> {
    crate::db::with(|conn| archive_recipe_in(conn, &household_id, &recipe_id, &archived_on))
}

/// Undoes `archive_recipe`. Idempotent; returns the recipe as stored.
pub fn restore_recipe(household_id: String, recipe_id: String) -> Result<RecipeDto, KimattaError> {
    crate::db::with(|conn| restore_recipe_in(conn, &household_id, &recipe_id))
}

/// The known unit vocabulary, so the editor's dropdown has one source and cannot drift from
/// the domain (as `known_restriction_kinds` does for restrictions).
pub fn known_unit_kinds() -> Vec<String> {
    UnitKind::ALL
        .iter()
        .map(|kind| kind.as_str().to_owned())
        .collect()
}

/// Empty `id` mints a UUID. Returns the stored item.
pub fn add_custom_ingredient(
    item: CustomIngredientDto,
) -> Result<CustomIngredientDto, KimattaError> {
    crate::db::with(|conn| add_custom_ingredient_in(conn, item))
}

pub fn list_custom_ingredients(
    household_id: String,
) -> Result<Vec<CustomIngredientDto>, KimattaError> {
    crate::db::with(|conn| list_custom_ingredients_in(conn, &household_id))
}

/// A blank DTO id is replaced before the newtype sees it, so `IdError::Empty` is unreachable
/// from a blank id; the mint condition (`trim().is_empty()`) matches the newtype's own.
fn id_or_minted(raw: String) -> String {
    if raw.trim().is_empty() {
        Uuid::new_v4().to_string()
    } else {
        raw
    }
}

fn rational(numer: u32, denom: u32) -> Result<Rational, KimattaError> {
    Ok(Rational::new(numer, denom)?)
}

fn quantity_to_domain(q: QuantityDto) -> Result<Quantity, KimattaError> {
    Ok(match q {
        QuantityDto::Unknown => Quantity::Unknown,
        QuantityDto::Exact { numer, denom } => Quantity::Exact(rational(numer, denom)?),
        QuantityDto::Range {
            min_numer,
            min_denom,
            max_numer,
            max_denom,
        } => Quantity::Range(QuantityRange::new(
            rational(min_numer, min_denom)?,
            rational(max_numer, max_denom)?,
        )?),
    })
}

fn quantity_from_domain(q: Quantity) -> QuantityDto {
    match q {
        Quantity::Unknown => QuantityDto::Unknown,
        Quantity::Exact(r) => QuantityDto::Exact {
            numer: r.numer(),
            denom: r.denom(),
        },
        Quantity::Range(range) => QuantityDto::Range {
            min_numer: range.min().numer(),
            min_denom: range.min().denom(),
            max_numer: range.max().numer(),
            max_denom: range.max().denom(),
        },
    }
}

fn unit_to_domain(u: UnitDto) -> Result<Unit, KimattaError> {
    Ok(match u {
        UnitDto::None => Unit::None,
        UnitDto::Known { unit } => Unit::Known(UnitKind::parse(&unit)?),
        UnitDto::Other { text } => Unit::Other(text),
    })
}

fn unit_from_domain(u: &Unit) -> UnitDto {
    match u {
        Unit::None => UnitDto::None,
        Unit::Known(kind) => UnitDto::Known {
            unit: kind.as_str().to_owned(),
        },
        Unit::Other(text) => UnitDto::Other { text: text.clone() },
    }
}

fn line_to_domain(l: IngredientLineDto) -> Result<IngredientLine, KimattaError> {
    let ingredient = match l.ingredient {
        None => None,
        Some(IngredientRefDto::Catalog { id }) => {
            Some(IngredientRef::Catalog(IngredientId::new(id)?))
        }
        Some(IngredientRefDto::Custom { id }) => {
            Some(IngredientRef::Custom(CustomIngredientId::new(id)?))
        }
    };
    Ok(IngredientLine::new(
        l.original_text,
        l.name,
        ingredient,
        quantity_to_domain(l.quantity)?,
        unit_to_domain(l.unit)?,
        l.preparation,
        l.optional,
    )?)
}

fn line_from_domain(l: &IngredientLine) -> IngredientLineDto {
    IngredientLineDto {
        original_text: l.original_text().to_owned(),
        name: l.name().to_owned(),
        ingredient: l.ingredient().map(|r| match r {
            IngredientRef::Catalog(id) => IngredientRefDto::Catalog {
                id: id.as_str().to_owned(),
            },
            IngredientRef::Custom(id) => IngredientRefDto::Custom {
                id: id.as_str().to_owned(),
            },
        }),
        quantity: quantity_from_domain(l.quantity()),
        unit: unit_from_domain(l.unit()),
        preparation: l.preparation().map(str::to_owned),
        optional: l.optional(),
    }
}

/// Validates fully into domain types before storage is touched, as `save_in` does for
/// planning: every invariant lives in the constructors, so an invalid request cannot reach
/// a write.
fn recipe_to_domain(dto: RecipeDto) -> Result<Recipe, KimattaError> {
    let id = RecipeId::new(id_or_minted(dto.id))?;
    let household_id = HouseholdId::new(dto.household_id)?;
    let lines = dto
        .lines
        .into_iter()
        .map(line_to_domain)
        .collect::<Result<Vec<_>, _>>()?;
    let provenance = RecipeProvenance::new(
        ProvenanceKind::parse(&dto.provenance.kind)?,
        dto.provenance.source_url,
        dto.provenance.source_name,
        dto.provenance.source_author,
    )?;
    Ok(Recipe::new(
        id,
        household_id,
        dto.title,
        dto.servings,
        dto.instructions,
        lines,
        provenance,
    )?)
}

fn recipe_from_domain(record: &RecipeRecord) -> RecipeDto {
    let r = &record.recipe;
    RecipeDto {
        id: r.id().as_str().to_owned(),
        household_id: r.household_id().as_str().to_owned(),
        title: r.title().to_owned(),
        servings: r.servings(),
        instructions: r.instructions().to_owned(),
        lines: r.lines().iter().map(line_from_domain).collect(),
        provenance: RecipeProvenanceDto {
            kind: r.provenance().kind().as_str().to_owned(),
            source_url: r.provenance().source_url().map(str::to_owned),
            source_name: r.provenance().source_name().map(str::to_owned),
            source_author: r.provenance().source_author().map(str::to_owned),
        },
        archived_at: record.archived_at.map(format_civil_date),
    }
}

/// The recipe as stored, for the commands that have just written it and promise to return
/// what was stored — including the archive marker, which no write here can set.
fn stored_recipe(
    conn: &Connection,
    household: &HouseholdId,
    id: &RecipeId,
) -> Result<RecipeDto, KimattaError> {
    match kimatta_storage::load_recipe(conn, household, id)? {
        Some(record) => Ok(recipe_from_domain(&record)),
        None => Err(StorageError::NoSuchRecipe {
            recipe: id.as_str().to_owned(),
            household: household.as_str().to_owned(),
        }
        .into()),
    }
}

fn custom_from_domain(c: &CustomIngredient) -> CustomIngredientDto {
    CustomIngredientDto {
        id: c.id().as_str().to_owned(),
        household_id: c.household_id().as_str().to_owned(),
        name: c.name().to_owned(),
        store_category: c.store_category().map(str::to_owned),
    }
}

/// Split out from the commands, as `rename_in` and `save_in` are, so they can be tested with
/// more than one household present without installing the process-wide connection.
fn save_recipe_in(conn: &mut Connection, dto: RecipeDto) -> Result<RecipeDto, KimattaError> {
    let recipe = recipe_to_domain(dto)?;
    kimatta_storage::save_recipe(conn, &recipe)?;
    // Read back rather than echo the input: `archived_at` is not in the request, and an edit
    // of an archived recipe must keep reporting it archived.
    stored_recipe(conn, recipe.household_id(), recipe.id())
}

fn load_recipe_in(
    conn: &Connection,
    household_id: &str,
    recipe_id: &str,
) -> Result<Option<RecipeDto>, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let id = RecipeId::new(recipe_id)?;
    Ok(kimatta_storage::load_recipe(conn, &household, &id)?
        .as_ref()
        .map(recipe_from_domain))
}

fn summaries(
    conn: &Connection,
    household_id: &str,
    listing: RecipeListing,
) -> Result<Vec<RecipeSummaryDto>, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    Ok(kimatta_storage::list_recipes(conn, &household, listing)?
        .into_iter()
        .map(|s| RecipeSummaryDto {
            id: s.id.as_str().to_owned(),
            title: s.title,
        })
        .collect())
}

fn list_recipes_in(
    conn: &Connection,
    household_id: &str,
) -> Result<Vec<RecipeSummaryDto>, KimattaError> {
    summaries(conn, household_id, RecipeListing::Active)
}

fn list_archived_recipes_in(
    conn: &Connection,
    household_id: &str,
) -> Result<Vec<RecipeSummaryDto>, KimattaError> {
    summaries(conn, household_id, RecipeListing::Archived)
}

/// The date is parsed before storage is touched, as `ensure_in` parses the anchor, so a
/// malformed one is a typed `Planning` error and writes nothing.
fn archive_recipe_in(
    conn: &mut Connection,
    household_id: &str,
    recipe_id: &str,
    archived_on: &str,
) -> Result<RecipeDto, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let id = RecipeId::new(recipe_id)?;
    let at = kimatta_storage::parse_civil_date(archived_on)?;
    kimatta_storage::archive_recipe(conn, &household, &id, at)?;
    stored_recipe(conn, &household, &id)
}

fn restore_recipe_in(
    conn: &mut Connection,
    household_id: &str,
    recipe_id: &str,
) -> Result<RecipeDto, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let id = RecipeId::new(recipe_id)?;
    kimatta_storage::restore_recipe(conn, &household, &id)?;
    stored_recipe(conn, &household, &id)
}

fn add_custom_ingredient_in(
    conn: &mut Connection,
    dto: CustomIngredientDto,
) -> Result<CustomIngredientDto, KimattaError> {
    let item = CustomIngredient::new(
        CustomIngredientId::new(id_or_minted(dto.id))?,
        HouseholdId::new(dto.household_id)?,
        dto.name,
        dto.store_category,
    )?;
    kimatta_storage::upsert_custom_ingredient(conn, &item)?;
    Ok(custom_from_domain(&item))
}

fn list_custom_ingredients_in(
    conn: &Connection,
    household_id: &str,
) -> Result<Vec<CustomIngredientDto>, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    Ok(kimatta_storage::list_custom_ingredients(conn, &household)?
        .iter()
        .map(custom_from_domain)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kimatta_storage::{Household, HouseholdMember, Ingredient, MemberId};

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

    fn authored() -> RecipeProvenanceDto {
        RecipeProvenanceDto {
            kind: "authored".to_owned(),
            source_url: None,
            source_name: None,
            source_author: None,
        }
    }

    fn line(original: &str, quantity: QuantityDto, unit: UnitDto) -> IngredientLineDto {
        IngredientLineDto {
            original_text: original.to_owned(),
            name: original.to_owned(),
            ingredient: None,
            quantity,
            unit,
            preparation: None,
            optional: false,
        }
    }

    fn recipe(household: &str, id: &str, lines: Vec<IngredientLineDto>) -> RecipeDto {
        RecipeDto {
            id: id.to_owned(),
            household_id: household.to_owned(),
            title: format!("Recipe {id}"),
            servings: Some(2),
            instructions: "Cook.".to_owned(),
            lines,
            provenance: authored(),
            archived_at: None,
        }
    }

    fn ids(list: Vec<RecipeSummaryDto>) -> Vec<String> {
        list.into_iter().map(|s| s.id).collect()
    }

    fn titles(list: Vec<RecipeSummaryDto>) -> Vec<String> {
        list.into_iter().map(|s| s.title).collect()
    }

    /// The fixture's `Recipe {id}` titles sort the way their ids do, so the id assertions
    /// elsewhere cannot tell `ORDER BY title` from insertion order, nor prove `title` is
    /// populated at all. These two sort opposite to their ids.
    #[test]
    fn list_recipes_reports_titles_ordered_by_title_not_id() {
        let mut conn = open_seeded(&["h"]);
        let mut first = recipe("h", "r1", vec![]);
        first.title = "Zucchini".to_owned();
        let mut second = recipe("h", "r2", vec![]);
        second.title = "Apple".to_owned();
        save_recipe_in(&mut conn, first).unwrap();
        save_recipe_in(&mut conn, second).unwrap();
        assert_eq!(
            titles(list_recipes_in(&conn, "h").unwrap()),
            vec!["Apple", "Zucchini"]
        );
        assert_eq!(ids(list_recipes_in(&conn, "h").unwrap()), vec!["r2", "r1"]);
    }

    // --- MVP-008: archive commands ---------------------------------------------------------

    #[test]
    fn archive_then_restore_round_trips_through_the_bridge() {
        let mut conn = open_seeded(&["h"]);
        let saved = save_recipe_in(&mut conn, recipe("h", "r", vec![])).unwrap();
        assert_eq!(saved.archived_at, None);
        let archived = archive_recipe_in(&mut conn, "h", "r", "2026-08-29").unwrap();
        assert_eq!(archived.archived_at, Some("2026-08-29".to_owned()));
        assert_eq!(archived.title, "Recipe r");
        // `load_recipe` still resolves the id and reports the marker.
        let loaded = load_recipe_in(&conn, "h", "r").unwrap().unwrap();
        assert_eq!(loaded.archived_at, Some("2026-08-29".to_owned()));
        // Idempotent, first date kept.
        let again = archive_recipe_in(&mut conn, "h", "r", "2026-09-01").unwrap();
        assert_eq!(again.archived_at, Some("2026-08-29".to_owned()));
        let restored = restore_recipe_in(&mut conn, "h", "r").unwrap();
        assert_eq!(restored.archived_at, None);
        assert_eq!(load_recipe_in(&conn, "h", "r").unwrap().unwrap(), restored);
    }

    #[test]
    fn archived_recipes_leave_list_recipes_and_appear_in_list_archived_recipes() {
        let mut conn = open_seeded(&["h"]);
        save_recipe_in(&mut conn, recipe("h", "r1", vec![])).unwrap();
        save_recipe_in(&mut conn, recipe("h", "r2", vec![])).unwrap();
        assert!(list_archived_recipes_in(&conn, "h").unwrap().is_empty());
        archive_recipe_in(&mut conn, "h", "r1", "2026-08-29").unwrap();
        assert_eq!(ids(list_recipes_in(&conn, "h").unwrap()), vec!["r2"]);
        assert_eq!(
            ids(list_archived_recipes_in(&conn, "h").unwrap()),
            vec!["r1"]
        );
        restore_recipe_in(&mut conn, "h", "r1").unwrap();
        assert_eq!(ids(list_recipes_in(&conn, "h").unwrap()), vec!["r1", "r2"]);
        assert!(list_archived_recipes_in(&conn, "h").unwrap().is_empty());
    }

    /// The date is parsed before storage is touched, with the same shape rule as the planning
    /// anchor, so an instant or a bare number is a typed `Planning` error and writes nothing.
    #[test]
    fn archive_rejects_a_non_civil_date_as_planning_error() {
        let mut conn = open_seeded(&["h"]);
        save_recipe_in(&mut conn, recipe("h", "r", vec![])).unwrap();
        for bad in ["2026-08-29T23:00:00Z", "20260829", ""] {
            let err = archive_recipe_in(&mut conn, "h", "r", bad).unwrap_err();
            assert!(
                matches!(err, KimattaError::Planning { .. }),
                "{bad:?} gave {err:?}"
            );
        }
        assert_eq!(
            load_recipe_in(&conn, "h", "r")
                .unwrap()
                .unwrap()
                .archived_at,
            None
        );
    }

    #[test]
    fn archive_of_another_households_recipe_is_storage_error() {
        let mut conn = open_seeded(&["h1", "h2"]);
        save_recipe_in(&mut conn, recipe("h1", "r", vec![])).unwrap();
        let err = archive_recipe_in(&mut conn, "h2", "r", "2026-08-29").unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "got {err:?}");
        let err = restore_recipe_in(&mut conn, "h2", "r").unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "got {err:?}");
        let err = archive_recipe_in(&mut conn, "h1", "ghost", "2026-08-29").unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "got {err:?}");
        assert_eq!(
            load_recipe_in(&conn, "h1", "r")
                .unwrap()
                .unwrap()
                .archived_at,
            None
        );
        assert!(list_archived_recipes_in(&conn, "h2").unwrap().is_empty());
    }

    /// `archived_at` on the DTO is output only: an edit of an archived recipe sent with
    /// `archived_at: None` neither restores it nor reports it restored.
    #[test]
    fn saving_an_archived_recipe_keeps_reporting_it_archived() {
        let mut conn = open_seeded(&["h"]);
        save_recipe_in(&mut conn, recipe("h", "r", vec![])).unwrap();
        archive_recipe_in(&mut conn, "h", "r", "2026-08-29").unwrap();
        let mut edited = recipe("h", "r", vec![]);
        edited.title = "Renamed".to_owned();
        let stored = save_recipe_in(&mut conn, edited).unwrap();
        assert_eq!(stored.title, "Renamed");
        assert_eq!(stored.archived_at, Some("2026-08-29".to_owned()));
        assert!(list_recipes_in(&conn, "h").unwrap().is_empty());
        assert_eq!(
            ids(list_archived_recipes_in(&conn, "h").unwrap()),
            vec!["r"]
        );
    }

    /// Hard-coded rather than mapped from `UnitKind::ALL`, for the reason
    /// `known_restriction_kinds_are_the_eleven_domain_tokens` records: a test mirroring the
    /// constant cannot catch an addition to it, and this list is what the unit dropdown shows.
    #[test]
    fn known_unit_kinds_matches_the_domain_vocabulary() {
        assert_eq!(
            known_unit_kinds(),
            vec!["tsp", "tbsp", "cup", "fl_oz", "ml", "l", "g", "kg", "oz", "lb", "piece"]
        );
    }

    #[test]
    fn save_then_load_returns_an_equal_dto() {
        let mut conn = open_seeded(&["h"]);
        let custom = add_custom_ingredient_in(
            &mut conn,
            CustomIngredientDto {
                id: "c".to_owned(),
                household_id: "h".to_owned(),
                name: "nana's mix".to_owned(),
                store_category: None,
            },
        )
        .unwrap();
        assert_eq!(custom.id, "c");
        let dto = recipe(
            "h",
            "r",
            vec![
                IngredientLineDto {
                    original_text: "  1/2 cup Flour, sifted ".to_owned(),
                    name: "Flour".to_owned(),
                    ingredient: None,
                    quantity: QuantityDto::Exact { numer: 2, denom: 4 },
                    unit: UnitDto::Known {
                        unit: "cup".to_owned(),
                    },
                    preparation: Some("sifted".to_owned()),
                    optional: false,
                },
                IngredientLineDto {
                    original_text: "2-3 handfuls nana's mix (optional)".to_owned(),
                    name: "nana's mix".to_owned(),
                    ingredient: Some(IngredientRefDto::Custom { id: "c".to_owned() }),
                    quantity: QuantityDto::Range {
                        min_numer: 2,
                        min_denom: 1,
                        max_numer: 3,
                        max_denom: 1,
                    },
                    unit: UnitDto::Other {
                        text: "handful".to_owned(),
                    },
                    preparation: None,
                    optional: true,
                },
                line("a splash of something", QuantityDto::Unknown, UnitDto::None),
            ],
        );
        let saved = save_recipe_in(&mut conn, dto).unwrap();
        // The stored form is canonical: `2/4` comes back as `1/2`.
        assert_eq!(
            saved.lines[0].quantity,
            QuantityDto::Exact { numer: 1, denom: 2 }
        );
        let loaded = load_recipe_in(&conn, "h", "r").unwrap().unwrap();
        assert_eq!(loaded, saved);
        assert_eq!(loaded.lines[0].original_text, "  1/2 cup Flour, sifted ");
        assert_eq!(
            loaded.lines[1].ingredient,
            Some(IngredientRefDto::Custom { id: "c".into() })
        );
        assert_eq!(loaded.lines[2].quantity, QuantityDto::Unknown);
        assert_eq!(loaded.lines[2].unit, UnitDto::None);
        assert_eq!(
            list_recipes_in(&conn, "h").unwrap(),
            vec![RecipeSummaryDto {
                id: "r".to_owned(),
                title: "Recipe r".to_owned(),
            }]
        );
    }

    #[test]
    fn an_empty_id_is_minted_and_returned() {
        let mut conn = open_seeded(&["h"]);
        for blank in ["", "   "] {
            let saved = save_recipe_in(&mut conn, recipe("h", blank, vec![])).unwrap();
            assert!(!saved.id.trim().is_empty(), "{blank:?} must be replaced");
            assert!(load_recipe_in(&conn, "h", &saved.id).unwrap().is_some());
            let custom = add_custom_ingredient_in(
                &mut conn,
                CustomIngredientDto {
                    id: blank.to_owned(),
                    household_id: "h".to_owned(),
                    name: "x".to_owned(),
                    store_category: None,
                },
            )
            .unwrap();
            assert!(!custom.id.trim().is_empty());
        }
        assert_eq!(list_recipes_in(&conn, "h").unwrap().len(), 2);
        assert_eq!(list_custom_ingredients_in(&conn, "h").unwrap().len(), 2);
    }

    #[test]
    fn rejects_a_blank_title_as_recipe_error_not_storage() {
        let mut conn = open_seeded(&["h"]);
        let mut dto = recipe("h", "r", vec![]);
        dto.title = "   ".to_owned();
        assert!(matches!(
            save_recipe_in(&mut conn, dto).unwrap_err(),
            KimattaError::Recipe { .. }
        ));
        assert!(list_recipes_in(&conn, "h").unwrap().is_empty());
    }

    #[test]
    fn rejects_an_unknown_unit_string() {
        let mut conn = open_seeded(&["h"]);
        let dto = recipe(
            "h",
            "r",
            vec![line(
                "1 cups",
                QuantityDto::Exact { numer: 1, denom: 1 },
                UnitDto::Known {
                    unit: "cups".to_owned(),
                },
            )],
        );
        assert!(matches!(
            save_recipe_in(&mut conn, dto).unwrap_err(),
            KimattaError::Recipe { .. }
        ));
    }

    #[test]
    fn rejects_an_inverted_range() {
        let mut conn = open_seeded(&["h"]);
        let dto = recipe(
            "h",
            "r",
            vec![line(
                "3-2",
                QuantityDto::Range {
                    min_numer: 3,
                    min_denom: 1,
                    max_numer: 2,
                    max_denom: 1,
                },
                UnitDto::None,
            )],
        );
        assert!(matches!(
            save_recipe_in(&mut conn, dto).unwrap_err(),
            KimattaError::Recipe { .. }
        ));
    }

    #[test]
    fn rejects_a_zero_denominator() {
        let mut conn = open_seeded(&["h"]);
        let dto = recipe(
            "h",
            "r",
            vec![line(
                "1/0",
                QuantityDto::Exact { numer: 1, denom: 0 },
                UnitDto::None,
            )],
        );
        assert!(matches!(
            save_recipe_in(&mut conn, dto).unwrap_err(),
            KimattaError::Recipe { .. }
        ));
        let dto = recipe("h", "r", vec![]);
        let mut bad = dto;
        bad.provenance.kind = "scraped".to_owned();
        assert!(matches!(
            save_recipe_in(&mut conn, bad).unwrap_err(),
            KimattaError::Recipe { .. }
        ));
        assert!(list_recipes_in(&conn, "h").unwrap().is_empty());
    }

    #[test]
    fn rejects_a_blank_custom_ingredient_name_as_recipe_error() {
        let mut conn = open_seeded(&["h"]);
        let err = add_custom_ingredient_in(
            &mut conn,
            CustomIngredientDto {
                id: "c".to_owned(),
                household_id: "h".to_owned(),
                name: " ".to_owned(),
                store_category: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, KimattaError::Recipe { .. }));
        assert!(list_custom_ingredients_in(&conn, "h").unwrap().is_empty());
    }

    #[test]
    fn commands_are_scoped_to_the_named_household() {
        let mut conn = open_seeded(&["h1", "h2"]);
        save_recipe_in(&mut conn, recipe("h1", "r1", vec![])).unwrap();
        save_recipe_in(&mut conn, recipe("h2", "r2", vec![])).unwrap();
        assert!(load_recipe_in(&conn, "h2", "r1").unwrap().is_none());
        assert!(load_recipe_in(&conn, "h1", "r1").unwrap().is_some());
        let l1 = list_recipes_in(&conn, "h1").unwrap();
        let l2 = list_recipes_in(&conn, "h2").unwrap();
        assert_eq!(l1.len(), 1);
        assert_eq!(l1[0].id, "r1");
        assert_eq!(l2.len(), 1);
        assert_eq!(l2[0].id, "r2");
        add_custom_ingredient_in(
            &mut conn,
            CustomIngredientDto {
                id: "c1".to_owned(),
                household_id: "h1".to_owned(),
                name: "x".to_owned(),
                store_category: None,
            },
        )
        .unwrap();
        assert_eq!(list_custom_ingredients_in(&conn, "h1").unwrap().len(), 1);
        assert!(list_custom_ingredients_in(&conn, "h2").unwrap().is_empty());
    }

    /// Expected-to-pass: the `Catalog` mapping arms are correct today, so this documents the
    /// round trip and pins it — a swapped variant in either direction, or two mirrored swaps,
    /// fails here because the catalog and custom ids deliberately differ.
    #[test]
    fn a_catalog_ingredient_ref_round_trips_through_the_bridge() {
        let mut conn = open_seeded(&["h"]);
        kimatta_storage::upsert_ingredient(
            &mut conn,
            &Ingredient::new(
                IngredientId::new("cat-flour").unwrap(),
                "flour",
                vec![],
                None,
            )
            .unwrap(),
        )
        .unwrap();
        add_custom_ingredient_in(
            &mut conn,
            CustomIngredientDto {
                id: "cus-mix".to_owned(),
                household_id: "h".to_owned(),
                name: "nana's mix".to_owned(),
                store_category: None,
            },
        )
        .unwrap();
        let mut catalog_line = line("1 cup flour", QuantityDto::Unknown, UnitDto::None);
        catalog_line.ingredient = Some(IngredientRefDto::Catalog {
            id: "cat-flour".to_owned(),
        });
        let mut custom_line = line("a pinch of mix", QuantityDto::Unknown, UnitDto::None);
        custom_line.ingredient = Some(IngredientRefDto::Custom {
            id: "cus-mix".to_owned(),
        });
        save_recipe_in(&mut conn, recipe("h", "r", vec![catalog_line, custom_line])).unwrap();
        let loaded = load_recipe_in(&conn, "h", "r").unwrap().unwrap();
        assert_eq!(
            loaded.lines[0].ingredient,
            Some(IngredientRefDto::Catalog {
                id: "cat-flour".to_owned()
            })
        );
        assert_eq!(
            loaded.lines[1].ingredient,
            Some(IngredientRefDto::Custom {
                id: "cus-mix".to_owned()
            })
        );
    }

    /// Expected-to-pass: `check_line_refs` already rejects an unseeded catalog id. This pins
    /// the `Catalog` write path, which no other bridge test reaches.
    #[test]
    fn a_line_naming_an_unseeded_catalog_ingredient_is_rejected() {
        let mut conn = open_seeded(&["h"]);
        let mut l = line("1 cup flour", QuantityDto::Unknown, UnitDto::None);
        l.ingredient = Some(IngredientRefDto::Catalog {
            id: "ghost".to_owned(),
        });
        let err = save_recipe_in(&mut conn, recipe("h", "r", vec![l])).unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "got {err:?}");
        assert!(list_recipes_in(&conn, "h").unwrap().is_empty());
    }

    #[test]
    fn blank_unit_text_and_provenance_reach_dart_as_recipe_errors() {
        let mut conn = open_seeded(&["h"]);
        let dto = recipe(
            "h",
            "r",
            vec![line(
                "a splash",
                QuantityDto::Unknown,
                UnitDto::Other {
                    text: "".to_owned(),
                },
            )],
        );
        assert!(matches!(
            save_recipe_in(&mut conn, dto).unwrap_err(),
            KimattaError::Recipe { .. }
        ));
        let mut blank_source = recipe("h", "r", vec![]);
        blank_source.provenance.kind = "imported".to_owned();
        blank_source.provenance.source_url = Some("   ".to_owned());
        assert!(matches!(
            save_recipe_in(&mut conn, blank_source).unwrap_err(),
            KimattaError::Recipe { .. }
        ));
        assert!(list_recipes_in(&conn, "h").unwrap().is_empty());
    }

    #[test]
    fn re_submitting_a_custom_ingredient_id_updates_it() {
        let mut conn = open_seeded(&["h"]);
        let submit = |conn: &mut Connection, name: &str| {
            add_custom_ingredient_in(
                conn,
                CustomIngredientDto {
                    id: "c".to_owned(),
                    household_id: "h".to_owned(),
                    name: name.to_owned(),
                    store_category: None,
                },
            )
        };
        submit(&mut conn, "mix").unwrap();
        // MVP-008's editor re-submitting after a transient failure must not surface SQLite's
        // own "UNIQUE constraint failed" text.
        let second = submit(&mut conn, "nana's mix").unwrap();
        assert_eq!(second.name, "nana's mix");
        let listed = list_custom_ingredients_in(&conn, "h").unwrap();
        assert_eq!(listed, vec![second]);
    }

    #[test]
    fn a_custom_ingredient_of_another_household_is_rejected() {
        let mut conn = open_seeded(&["h1", "h2"]);
        add_custom_ingredient_in(
            &mut conn,
            CustomIngredientDto {
                id: "c2".to_owned(),
                household_id: "h2".to_owned(),
                name: "x".to_owned(),
                store_category: None,
            },
        )
        .unwrap();
        let mut l = line("x", QuantityDto::Unknown, UnitDto::None);
        l.ingredient = Some(IngredientRefDto::Custom {
            id: "c2".to_owned(),
        });
        let err = save_recipe_in(&mut conn, recipe("h1", "r", vec![l])).unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "got {err:?}");
        assert!(list_recipes_in(&conn, "h1").unwrap().is_empty());
    }
}
