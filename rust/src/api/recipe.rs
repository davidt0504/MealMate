use kimatta_storage::{
    Connection, CustomIngredient, CustomIngredientId, HouseholdId, IngredientId, IngredientLine,
    IngredientRef, ProvenanceKind, Quantity, QuantityRange, Rational, Recipe, RecipeId,
    RecipeProvenance, Unit, UnitKind,
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

pub fn list_recipes(household_id: String) -> Result<Vec<RecipeSummaryDto>, KimattaError> {
    crate::db::with(|conn| list_recipes_in(conn, &household_id))
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

fn recipe_from_domain(r: &Recipe) -> RecipeDto {
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
    Ok(recipe_from_domain(&recipe))
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

fn list_recipes_in(
    conn: &Connection,
    household_id: &str,
) -> Result<Vec<RecipeSummaryDto>, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    Ok(kimatta_storage::list_recipes(conn, &household)?
        .into_iter()
        .map(|s| RecipeSummaryDto {
            id: s.id.as_str().to_owned(),
            title: s.title,
        })
        .collect())
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
        }
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
