//! Starter-content installation (MVP-011): one coarse service-level command, invariant 21.

use kimatta_storage::{
    all_starter_content, HouseholdId, Recipe, RecipeId, StarterContent, StarterRecipe,
};
use uuid::Uuid;

use super::error::KimattaError;
use crate::db;

/// What one install call did. `available` and `pending_cook_review` exist so `installed: 0`
/// reads as *empty by design* — nothing here ships by either arm of D-041's rule — rather than
/// as a swallowed failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StarterInstallReportDto {
    pub installed: u32,
    pub skipped: u32,
    pub catalog_installed: u32,
    /// Shipped-set size: entries a recorded cook review or a federal publisher stands behind.
    pub available: u32,
    /// Authored but shipped by neither arm of D-041's rule — an `original` entry no one has
    /// cooked. These never install.
    pub pending_cook_review: u32,
}

/// Seeds the global ingredient catalog and any shippable starter recipe this household
/// does not already hold. Idempotent and cheap to re-run: once every catalog id and slug is
/// present it writes no rows, so it is safe to call on every app start.
pub fn install_starter_content(
    household_id: String,
) -> Result<StarterInstallReportDto, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let (shipped, pending) = split_authored(all_starter_content().map_err(starter_error)?);
    let recipes = to_recipes(&household, &shipped)?;
    let report = db::with(|conn| {
        Ok(kimatta_storage::install_starter_content(
            conn,
            &household,
            &shipped.catalog,
            &recipes,
        )?)
    })?;
    Ok(StarterInstallReportDto {
        installed: report.installed as u32,
        skipped: report.skipped as u32,
        catalog_installed: report.catalog_installed as u32,
        available: shipped.recipes.len() as u32,
        pending_cook_review: pending as u32,
    })
}

/// One parse, two views: the embedded manifest is deserialised once per install and split here,
/// rather than read a second time to count what the first parse already carries. Returns the
/// shipped view — entries shipping by either arm of D-041's rule — and the count of those
/// shipping by neither.
fn split_authored(authored: StarterContent) -> (StarterContent, usize) {
    let (shipped, pending): (Vec<_>, Vec<_>) = authored
        .recipes
        .into_iter()
        .partition(StarterRecipe::is_shippable);
    (
        StarterContent {
            catalog: authored.catalog,
            recipes: shipped,
        },
        pending.len(),
    )
}

/// Recipe ids are minted here, not authored into the content file: the same slug installs
/// into two households as two distinct rows.
fn to_recipes(
    household: &HouseholdId,
    content: &StarterContent,
) -> Result<Vec<Recipe>, KimattaError> {
    content
        .recipes
        .iter()
        .map(|entry| {
            Ok(Recipe::new(
                RecipeId::new(Uuid::new_v4().to_string())?,
                household.clone(),
                entry.title.clone(),
                entry.servings,
                entry.prep_minutes,
                entry.instructions.clone(),
                entry.lines.clone(),
                entry.provenance.clone(),
            )?)
        })
        .collect()
}

fn starter_error(e: kimatta_storage::StarterError) -> KimattaError {
    KimattaError::Recipe {
        message: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kimatta_storage::{
        insert_household, open, shipped_starter_content, Household, HouseholdMember, MemberId,
        StarterInstallReport,
    };

    fn seeded() -> kimatta_storage::Connection {
        let mut conn = open(":memory:").unwrap();
        let h = Household {
            id: HouseholdId::new("h").unwrap(),
            name: None,
        };
        let m = HouseholdMember {
            id: MemberId::new("m").unwrap(),
            household_id: h.id.clone(),
            display_name: "Me".to_owned(),
        };
        insert_household(&mut conn, &h, std::slice::from_ref(&m)).unwrap();
        conn
    }

    #[test]
    fn install_reports_installed_skipped_catalog_available_and_pending() {
        // The shipped set is whatever D-041's two-armed rule admits. The catalog seeds either
        // way, and the report says which of the two an `installed: 0` was.
        let mut conn = seeded();
        let shipped = shipped_starter_content().unwrap();
        let authored = all_starter_content().unwrap();
        let household = HouseholdId::new("h").unwrap();
        let recipes = to_recipes(&household, &shipped).unwrap();
        let report = kimatta_storage::install_starter_content(
            &mut conn,
            &household,
            &shipped.catalog,
            &recipes,
        )
        .unwrap();
        assert_eq!(report.installed, shipped.recipes.len());
        assert!(report.catalog_installed > 0);
        assert!(!authored.recipes.is_empty());
    }

    #[test]
    fn install_is_idempotent_through_the_bridge() {
        let mut conn = seeded();
        let shipped = shipped_starter_content().unwrap();
        let household = HouseholdId::new("h").unwrap();
        let recipes = to_recipes(&household, &shipped).unwrap();
        kimatta_storage::install_starter_content(&mut conn, &household, &shipped.catalog, &recipes)
            .unwrap();
        let second = kimatta_storage::install_starter_content(
            &mut conn,
            &household,
            &shipped.catalog,
            &recipes,
        )
        .unwrap();
        assert_eq!(
            second,
            StarterInstallReport {
                installed: 0,
                skipped: recipes.len(),
                catalog_installed: 0
            }
        );
    }

    #[test]
    fn install_for_an_unknown_household_is_a_storage_error() {
        let mut conn = seeded();
        let shipped = shipped_starter_content().unwrap();
        let err = kimatta_storage::install_starter_content(
            &mut conn,
            &HouseholdId::new("nope").unwrap(),
            &shipped.catalog,
            &[],
        )
        .unwrap_err();
        let mapped: KimattaError = err.into();
        assert!(
            matches!(&mapped, KimattaError::Storage { message } if message.contains("nope")),
            "{mapped:?}"
        );
    }

    #[test]
    fn the_one_parse_split_matches_the_two_view_functions() {
        // Pins the single-parse refactor: `split_authored` must give exactly what the two public
        // view functions give, so `available` and `pending_cook_review` cannot drift. The `all`
        // predicate catches an inverted partition, which the equality assertions alone would
        // not distinguish from a correct one on a roster where every entry ships.
        let authored = all_starter_content().unwrap();
        let total = authored.recipes.len();
        assert!(total > 0);
        let expected = shipped_starter_content().unwrap();
        let (shipped, pending) = split_authored(authored);
        assert!(shipped.recipes.iter().all(StarterRecipe::is_shippable));
        assert_eq!(shipped.recipes, expected.recipes);
        assert_eq!(shipped.catalog, expected.catalog);
        assert_eq!(shipped.recipes.len() + pending, total);
    }

    #[test]
    fn a_minted_recipe_id_is_unique_per_household() {
        let shipped = all_starter_content().unwrap();
        let h1 = to_recipes(&HouseholdId::new("h1").unwrap(), &shipped).unwrap();
        let h2 = to_recipes(&HouseholdId::new("h2").unwrap(), &shipped).unwrap();
        assert!(!h1.is_empty());
        for (a, b) in h1.iter().zip(h2.iter()) {
            assert_ne!(a.id().as_str(), b.id().as_str());
            assert_eq!(a.provenance().starter_slug(), b.provenance().starter_slug());
        }
    }
}
