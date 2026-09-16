//! Builds the pre-FIX-001 v12 fixture database for the on-device AC-1 migration check.
//!
//! Usage: `cargo run -p kimatta-storage --example make_v12_fixture -- <out.db>`
//!
//! Today's manifest already names the diced-tomato lines correctly, so installing it and
//! stopping there would leave the v13 migration nothing to remap and the device test would pass
//! while proving nothing. The rewrite at the end puts the rows back into the shape a phone that
//! installed before FIX-001 actually holds: the lines pointed at `ing-canned-tomatoes`, no
//! `ing-diced-tomatoes` catalog row, and none of the three aliases v13 adds.

use std::path::PathBuf;

use household_core::{Household, HouseholdId};
use kimatta_storage::{
    all_starter_content, insert_household, install_starter_content, schema_version, Recipe,
    RecipeId, StarterContent, StarterRecipe, MIGRATIONS,
};
use rusqlite::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out: PathBuf = std::env::args()
        .nth(1)
        .ok_or("usage: make_v12_fixture <out.db>")?
        .into();
    if out.exists() {
        std::fs::remove_file(&out)?;
    }

    let mut conn = Connection::open(&out)?;
    MIGRATIONS.to_version(&mut conn, 12)?;

    let household = Household {
        id: HouseholdId::new("fixture-household")?,
        name: Some("Fixture household".to_owned()),
    };
    insert_household(&mut conn, &household, &[])?;

    // Deterministic recipe ids rather than the bridge's UUIDs: a fixture is compared against
    // itself across runs, and this avoids pulling `uuid` in for a test artifact.
    let StarterContent { catalog, recipes } = all_starter_content()?;
    let shipped: Vec<StarterRecipe> = recipes
        .into_iter()
        .filter(StarterRecipe::is_shippable)
        .collect();
    let recipes: Vec<Recipe> = shipped
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            Recipe::new(
                RecipeId::new(format!("fixture-recipe-{i:03}"))?,
                household.id.clone(),
                entry.title.clone(),
                entry.servings,
                entry.prep_minutes,
                entry.instructions.clone(),
                entry.lines.clone(),
                entry.provenance.clone(),
            )
            .map_err(Into::into)
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;
    let report = install_starter_content(&mut conn, &household.id, &catalog, &recipes)?;

    // Exactly the three things migration v13 does, undone.
    conn.execute_batch(
        "UPDATE recipe_ingredient_line SET ingredient_id = 'ing-canned-tomatoes'
         WHERE ingredient_id = 'ing-diced-tomatoes';
         DELETE FROM ingredient_alias
         WHERE alias IN ('canned diced tomatoes', 'orzo pasta', 'elbow macaroni pasta');
         DELETE FROM ingredient WHERE id = 'ing-diced-tomatoes';",
    )?;

    // A fixture that does not actually carry the defect would make the device test vacuous, so
    // it is checked here rather than discovered as a green run on the emulator.
    let version = schema_version(&conn)?;
    let stranded: u32 = conn.query_row(
        "SELECT count(*) FROM recipe_ingredient_line
         WHERE ingredient_id = 'ing-canned-tomatoes' AND lower(name) LIKE '%diced%'",
        [],
        |r| r.get(0),
    )?;
    let leftover: u32 = conn.query_row(
        "SELECT count(*) FROM ingredient WHERE id = 'ing-diced-tomatoes'",
        [],
        |r| r.get(0),
    )?;
    if version != 12 {
        return Err(format!("fixture is schema {version}, expected 12").into());
    }
    if leftover != 0 {
        return Err("fixture still holds the ing-diced-tomatoes catalog row".into());
    }
    if stranded == 0 {
        return Err(
            "fixture holds no diced line on ing-canned-tomatoes; v13 would have nothing to remap"
                .into(),
        );
    }

    println!(
        "wrote {} -- schema {version}, {} recipes installed, {} catalog rows, {stranded} diced \
         lines awaiting the v13 remap",
        out.display(),
        report.installed,
        report.catalog_installed,
    );
    Ok(())
}
