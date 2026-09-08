//! Curated starter content (MVP-011). One JSON file is both the shipped data and the
//! reviewable provenance manifest, so the two cannot drift apart.
//!
//! The JSON never deserializes into a domain type directly: `Ingredient`, `IngredientLine`,
//! `RecipeProvenance` and `Rational` all have private fields and no `serde` derive, and a
//! derived `Deserialize` would bypass `non_blank`, the zero checks, gcd normalisation and the
//! sorted/deduplicated alias invariant. So the file lands in plain `Raw*` structs first and a
//! single conversion pass runs every domain constructor.

use serde::Deserialize;

use crate::recipe::{
    Ingredient, IngredientId, IngredientLine, IngredientRef, ProvenanceKind, Quantity,
    QuantityRange, Rational, RecipeError, RecipeProvenance, RecipeRights, RightsBasis, Unit,
    UnitKind,
};
use crate::restriction::RestrictionKind;
use crate::{parse_civil_date, CivilDate};

const CONTENT: &str = include_str!("../content/starter_recipes.json");

#[derive(Debug, PartialEq, Eq)]
pub enum StarterError {
    /// The file is not the shape the wire layer declares.
    Parse(String),
    /// One entry violates a content rule the domain constructors cannot express.
    Content { slug: String, problem: String },
    /// One entry failed a domain constructor.
    Recipe(RecipeError),
}

impl std::fmt::Display for StarterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(detail) => write!(f, "starter content does not parse: {detail}"),
            Self::Content { slug, problem } => {
                write!(f, "starter entry {slug:?} is not shippable: {problem}")
            }
            Self::Recipe(e) => write!(f, "starter content is not a valid recipe: {e}"),
        }
    }
}

impl std::error::Error for StarterError {}

impl From<RecipeError> for StarterError {
    fn from(e: RecipeError) -> Self {
        Self::Recipe(e)
    }
}

/// A recorded human cook review (MVP-011 AC-3). AC-3 requires "a recorded cook review **and**
/// actionable corrections resolved", which a boolean cannot carry — so this is the log itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookReview {
    pub cooked_on: CivilDate,
    pub by: String,
    pub corrections: Option<String>,
}

/// One authored entry, already through every domain constructor. It is not a `Recipe`: the
/// recipe id and the household are minted per install, not authored into the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarterRecipe {
    pub slug: String,
    /// `None` = no human has cooked it. An `original` entry with `None` is not shipped; a
    /// federal entry ships on its rights arm instead — see `is_shippable`.
    pub cook_review: Option<CookReview>,
    pub expected_conflicts: Vec<RestrictionKind>,
    pub title: String,
    pub servings: Option<u32>,
    pub prep_minutes: Option<u32>,
    pub instructions: String,
    pub lines: Vec<IngredientLine>,
    /// Carries the rights record and the starter slug.
    pub provenance: RecipeProvenance,
}

impl StarterRecipe {
    /// D-041's two-armed shipping rule, superseding MVP-011 AC-3's single arm. An entry ships
    /// when a human has cooked it, **or** when it is a US federal government publication
    /// carrying the source URL that makes that claim checkable: the federal kitchen that
    /// published it supplies the standing the cook review otherwise supplies. An `original`
    /// entry has no publisher behind it, so it still needs the cook.
    pub fn is_shippable(&self) -> bool {
        let federal = self
            .provenance
            .rights()
            .is_some_and(|r| r.basis() == RightsBasis::UsFederalPublicDomain)
            && self.provenance.source_url().is_some();
        self.cook_review.is_some() || federal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarterContent {
    pub catalog: Vec<Ingredient>,
    pub recipes: Vec<StarterRecipe>,
}

/// Every authored entry, reviewed or not — the manifest view, for tests and rights review.
pub fn all_starter_content() -> Result<StarterContent, StarterError> {
    parse_starter_content(CONTENT)
}

/// Only entries a recorded cook review or a federal publisher stands behind — the shipped
/// view, what the installer sees.
pub fn shipped_starter_content() -> Result<StarterContent, StarterError> {
    Ok(shipped_view(all_starter_content()?))
}

/// The shipping filter itself, separated so it can be exercised against a fixture: the
/// embedded content can never contain a federal entry missing its URL, which is one of the two
/// refusals AC-3 asks this filter to make. The catalog is not filtered — an ingredient name is
/// a fact, carrying no rights question and no cook question.
fn shipped_view(all: StarterContent) -> StarterContent {
    StarterContent {
        catalog: all.catalog,
        recipes: all
            .recipes
            .into_iter()
            .filter(StarterRecipe::is_shippable)
            .collect(),
    }
}

// --- wire layer ----------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContent {
    policy: RawPolicy,
    catalog: Vec<RawIngredient>,
    recipes: Vec<RawRecipe>,
}

// Deliberately NOT `deny_unknown_fields`: this block is the manifest's human-readable half,
// and schema-locking it would make adding a prose note to a rights manifest a test failure.
#[derive(Deserialize)]
struct RawPolicy {
    allowed_rights_bases: Vec<String>,
    /// Checked against `allowed_rights_bases` for disjointness in `parse_starter_content`: the
    /// allow-list is what gates a basis, so self-contradiction is what this half can be wrong about.
    excluded: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawIngredient {
    id: String,
    canonical_name: String,
    aliases: Vec<String>,
    store_category: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecipe {
    slug: String,
    title: String,
    servings: Option<u32>,
    prep_minutes: Option<u32>,
    instructions: String,
    provenance: RawProvenance,
    cook_review: Option<RawCookReview>,
    expected_conflicts: Vec<String>,
    lines: Vec<RawLine>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProvenance {
    kind: String,
    source_url: Option<String>,
    source_name: Option<String>,
    source_author: Option<String>,
    rights: RawRights,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRights {
    basis: String,
    attribution: Option<String>,
    modifications: Option<String>,
    verified_on: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCookReview {
    cooked_on: String,
    by: String,
    corrections: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLine {
    original_text: String,
    name: String,
    catalog_id: String,
    quantity: RawQuantity,
    unit: Option<String>,
    preparation: Option<String>,
    optional: bool,
}

/// `Unit::Other` has no representation here by construction: a starter recipe must be
/// aggregable by MVP-015, so an unrecognised unit token is an authoring error, not a value.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawQuantity {
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

fn parse_starter_content(raw: &str) -> Result<StarterContent, StarterError> {
    let content: RawContent =
        serde_json::from_str(raw).map_err(|e| StarterError::Parse(e.to_string()))?;
    // `excluded` is a rights record AC-1 reads. Its allow-list sibling is what actually gates a
    // basis (`convert_recipe` below), so the one thing this field can be wrong about is the
    // manifest contradicting itself — which would otherwise surface only by `RightsBasis`
    // happening to lack a variant for the basis in question.
    if let Some(basis) = content
        .policy
        .allowed_rights_bases
        .iter()
        .find(|b| content.policy.excluded.contains(b))
    {
        return Err(StarterError::Content {
            slug: String::new(),
            problem: format!("rights basis {basis:?} is both allowed and excluded"),
        });
    }
    let mut catalog = Vec::with_capacity(content.catalog.len());
    let mut catalog_ids = Vec::with_capacity(content.catalog.len());
    for entry in content.catalog {
        if catalog_ids.contains(&entry.id) {
            return Err(StarterError::Content {
                slug: entry.id,
                problem: "duplicate catalog id".to_owned(),
            });
        }
        catalog_ids.push(entry.id.clone());
        catalog.push(Ingredient::new(
            IngredientId::new(entry.id).map_err(|_| StarterError::Content {
                slug: String::new(),
                problem: "blank catalog id".to_owned(),
            })?,
            entry.canonical_name,
            entry.aliases,
            entry.store_category,
        )?);
    }
    let mut recipes = Vec::with_capacity(content.recipes.len());
    let mut slugs: Vec<String> = Vec::with_capacity(content.recipes.len());
    for entry in content.recipes {
        if slugs.contains(&entry.slug) {
            return Err(StarterError::Content {
                slug: entry.slug,
                problem: "duplicate slug".to_owned(),
            });
        }
        slugs.push(entry.slug.clone());
        recipes.push(convert_recipe(
            entry,
            &catalog_ids,
            &content.policy.allowed_rights_bases,
        )?);
    }
    Ok(StarterContent { catalog, recipes })
}

fn convert_recipe(
    entry: RawRecipe,
    catalog_ids: &[String],
    allowed_bases: &[String],
) -> Result<StarterRecipe, StarterError> {
    let slug = entry.slug;
    let fail = |problem: String| StarterError::Content {
        slug: slug.clone(),
        problem,
    };
    if slug.trim().is_empty() {
        return Err(fail("blank slug".to_owned()));
    }
    if entry.provenance.kind != ProvenanceKind::Starter.as_str() {
        return Err(fail(format!(
            "provenance kind is {:?}, not \"starter\"",
            entry.provenance.kind
        )));
    }
    if !allowed_bases.contains(&entry.provenance.rights.basis) {
        return Err(fail(format!(
            "rights basis {:?} is not in the file's allowed set",
            entry.provenance.rights.basis
        )));
    }
    if entry.lines.is_empty() {
        return Err(fail("no ingredient lines".to_owned()));
    }
    let raw_rights = entry.provenance.rights;
    let rights = RecipeRights::new(
        RightsBasis::parse(&raw_rights.basis)?,
        raw_rights.attribution,
        raw_rights.modifications,
        &raw_rights.verified_on,
    )?;
    let provenance = RecipeProvenance::with_rights(
        ProvenanceKind::parse(&entry.provenance.kind)?,
        entry.provenance.source_url,
        entry.provenance.source_name,
        entry.provenance.source_author,
        Some(rights),
        Some(slug.clone()),
    )?;
    let mut lines = Vec::with_capacity(entry.lines.len());
    for line in entry.lines {
        if !catalog_ids.contains(&line.catalog_id) {
            return Err(fail(format!(
                "line names catalog id {:?}, which this file does not define",
                line.catalog_id
            )));
        }
        let quantity = match line.quantity {
            RawQuantity::Unknown => Quantity::Unknown,
            RawQuantity::Exact { numer, denom } => Quantity::Exact(Rational::new(numer, denom)?),
            RawQuantity::Range {
                min_numer,
                min_denom,
                max_numer,
                max_denom,
            } => Quantity::Range(QuantityRange::new(
                Rational::new(min_numer, min_denom)?,
                Rational::new(max_numer, max_denom)?,
            )?),
        };
        let unit = match line.unit {
            None => Unit::None,
            Some(token) => Unit::Known(UnitKind::parse(&token)?),
        };
        lines.push(IngredientLine::new(
            line.original_text,
            line.name,
            Some(IngredientRef::Catalog(
                IngredientId::new(line.catalog_id)
                    .map_err(|_| fail("blank catalog id".to_owned()))?,
            )),
            quantity,
            unit,
            line.preparation,
            line.optional,
        )?);
    }
    let mut expected_conflicts = Vec::with_capacity(entry.expected_conflicts.len());
    for token in entry.expected_conflicts {
        expected_conflicts.push(
            RestrictionKind::parse(&token)
                .map_err(|e| fail(format!("unknown expected conflict: {e}")))?,
        );
    }
    let cook_review = entry
        .cook_review
        .map(|review| -> Result<CookReview, StarterError> {
            Ok(CookReview {
                cooked_on: parse_civil_date(&review.cooked_on)
                    .map_err(|_| fail(format!("cook review date {:?}", review.cooked_on)))?,
                by: review.by,
                corrections: review.corrections,
            })
        })
        .transpose()?;
    Ok(StarterRecipe {
        slug,
        cook_review,
        expected_conflicts,
        title: entry.title,
        servings: entry.servings,
        prep_minutes: entry.prep_minutes,
        instructions: entry.instructions,
        lines,
        provenance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::Recipe;
    use crate::restriction::{assess, HouseholdRestrictions, Restriction};
    use household_core::HouseholdId;

    /// The smallest file the wire layer accepts, as a template for the negative cases below.
    fn fixture(recipe_body: &str) -> String {
        format!(
            r#"{{
              "policy": {{ "allowed_rights_bases": ["original", "cc0"], "excluded": ["cc_by"] }},
              "catalog": [
                {{ "id": "ing-oil", "canonical_name": "olive oil",
                   "aliases": [], "store_category": "pantry" }}
              ],
              "recipes": [{recipe_body}]
            }}"#
        )
    }

    /// Both sides of `every_recipe_matches_its_declared_conflicts` go through this. `Vec::dedup`
    /// collapses only *consecutive* duplicates, so comparing raw would silently depend on two
    /// things that are not contracts: `assess`'s restriction-outer loop nesting, and the order
    /// `expected_conflicts` happens to be authored in.
    fn canonical(kinds: &[RestrictionKind]) -> Vec<RestrictionKind> {
        let mut out = kinds.to_vec();
        out.sort_by_key(|k| RestrictionKind::ALL.iter().position(|a| a == k).unwrap());
        out.dedup();
        out
    }

    fn body(overrides: &str) -> String {
        format!(
            r#"{{
              "slug": "s", "title": "T", "servings": 2, "prep_minutes": 10,
              "instructions": "Cook.",
              "provenance": {{ "kind": "starter", "source_url": null, "source_name": null,
                              "source_author": "Kimatta",
                              "rights": {{ "basis": "original", "attribution": null,
                                          "modifications": null, "verified_on": "2026-08-29" }} }},
              "cook_review": null,
              "expected_conflicts": [],
              "lines": [{{ "original_text": "1 tbsp olive oil", "name": "olive oil",
                          "catalog_id": "ing-oil",
                          "quantity": {{ "kind": "exact", "numer": 1, "denom": 1 }},
                          "unit": "tbsp", "preparation": null, "optional": false }}]
              {overrides}
            }}"#
        )
    }

    /// `fixture` with the federal basis allowed, for the rights arm of the shipping rule.
    /// Anchored on `"cc0"` alone — the allow-list's exact spacing is not this helper's
    /// business, and `"cc0"` occurs once in `fixture`'s output (`excluded` is `["cc_by"]`).
    fn federal_fixture(recipe_body: &str) -> String {
        fixture(recipe_body).replace(r#""cc0""#, r#""us_federal_public_domain", "cc0""#)
    }

    /// The federal arm's shape: basis swapped and a source URL supplied. The basis anchor is
    /// the whole key/value pair because `body` also contains `"original_text"`, which a bare
    /// `original` replace would corrupt.
    fn federal_body() -> String {
        body("")
            .replace(
                r#""basis": "original""#,
                r#""basis": "us_federal_public_domain""#,
            )
            .replace(
                r#""source_url": null"#,
                r#""source_url": "https://www.nhlbi.nih.gov/r""#,
            )
    }

    fn shipped_slugs(json: &str) -> Vec<String> {
        shipped_view(parse_starter_content(json).unwrap())
            .recipes
            .into_iter()
            .map(|r| r.slug)
            .collect()
    }

    // --- MVP-032 AC-3: one test per arm of D-041's two-armed shipping rule ----------------

    #[test]
    fn a_recorded_cook_review_ships_an_entry() {
        let reviewed = body("").replace(
            "\"cook_review\": null",
            r#""cook_review": { "cooked_on": "2026-09-01", "by": "David",
                                "corrections": null }"#,
        );
        assert_eq!(shipped_slugs(&fixture(&reviewed)), vec!["s".to_owned()]);
    }

    #[test]
    fn a_federal_entry_with_a_source_url_ships() {
        assert_eq!(
            shipped_slugs(&federal_fixture(&federal_body())),
            vec!["s".to_owned()]
        );
    }

    #[test]
    fn an_original_entry_without_a_review_does_not_ship() {
        assert!(shipped_slugs(&fixture(&body(""))).is_empty());
    }

    #[test]
    fn a_federal_entry_without_a_source_url_does_not_ship() {
        // The rights arm needs both halves: the basis alone is a claim with nothing behind it.
        let no_url = federal_body().replace(
            r#""source_url": "https://www.nhlbi.nih.gov/r""#,
            r#""source_url": null"#,
        );
        assert!(shipped_slugs(&federal_fixture(&no_url)).is_empty());
    }

    #[test]
    fn starter_content_parses() {
        // Risk 8: the real embedded string, so a malformed content file fails `cargo test`
        // rather than only the app at runtime.
        let content = all_starter_content().unwrap();
        assert!(!content.catalog.is_empty());
        assert!(!content.recipes.is_empty());
    }

    #[test]
    fn every_rights_basis_is_in_the_allowed_set() {
        // Hard-coded, not read off `RightsBasis::ALL`: a basis added without a rights review
        // must break this test rather than be mirrored by it.
        let allowed = ["original", "us_federal_public_domain", "cc0"];
        let content = all_starter_content().unwrap();
        assert!(!content.recipes.is_empty());
        for recipe in &content.recipes {
            let rights = recipe.provenance.rights().expect("every entry has rights");
            assert!(
                allowed.contains(&rights.basis().as_str()),
                "{}: {:?}",
                recipe.slug,
                rights.basis()
            );
        }
    }

    #[test]
    fn every_slug_is_unique() {
        let content = all_starter_content().unwrap();
        let mut slugs: Vec<&str> = content.recipes.iter().map(|r| r.slug.as_str()).collect();
        assert!(!slugs.is_empty());
        let total = slugs.len();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), total);
    }

    #[test]
    fn every_catalog_id_is_unique() {
        let content = all_starter_content().unwrap();
        let mut ids: Vec<&str> = content.catalog.iter().map(|i| i.id().as_str()).collect();
        assert!(!ids.is_empty());
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total);
    }

    #[test]
    fn every_catalog_reference_resolves() {
        let content = all_starter_content().unwrap();
        let ids: Vec<&str> = content.catalog.iter().map(|i| i.id().as_str()).collect();
        let mut checked = 0;
        for recipe in &content.recipes {
            assert!(!recipe.lines.is_empty(), "{} has no lines", recipe.slug);
            for line in &recipe.lines {
                match line.ingredient() {
                    Some(IngredientRef::Catalog(id)) => {
                        assert!(
                            ids.contains(&id.as_str()),
                            "{}: {}",
                            recipe.slug,
                            id.as_str()
                        );
                        checked += 1;
                    }
                    other => panic!("{}: line is not a catalog ref: {other:?}", recipe.slug),
                }
            }
        }
        assert!(checked > 0);
    }

    #[test]
    fn every_verified_on_is_a_civil_date() {
        let content = all_starter_content().unwrap();
        assert!(!content.recipes.is_empty());
        for recipe in &content.recipes {
            let rights = recipe.provenance.rights().unwrap();
            // `RecipeRights` only holds a parsed date, so re-formatting and re-parsing is the
            // falsifiable form of this claim.
            let formatted = rights.verified_on().to_string();
            assert!(parse_civil_date(&formatted).is_ok(), "{}", recipe.slug);
            assert_eq!(formatted.len(), 10);
        }
    }

    #[test]
    fn a_cc_by_basis_is_rejected() {
        // The excluded case must be unrepresentable, not merely absent from today's file.
        let json = fixture(&body("").replace("\"basis\": \"original\"", "\"basis\": \"cc_by\""));
        let err = parse_starter_content(&json).unwrap_err();
        assert_eq!(
            err,
            StarterError::Content {
                slug: "s".to_owned(),
                problem: "rights basis \"cc_by\" is not in the file's allowed set".to_owned(),
            }
        );
    }

    #[test]
    fn a_basis_both_allowed_and_excluded_is_a_content_error() {
        // `fixture`'s policy is hard-coded disjoint, so this case needs its own raw string. The
        // body's basis "original" is in the allowed set, so nothing else in the file objects: the
        // contradiction between the two halves of `policy` is the only thing under test.
        let json = format!(
            r#"{{
              "policy": {{ "allowed_rights_bases": ["original", "cc0"], "excluded": ["cc0"] }},
              "catalog": [
                {{ "id": "ing-oil", "canonical_name": "olive oil",
                   "aliases": [], "store_category": "pantry" }}
              ],
              "recipes": [{}]
            }}"#,
            body("")
        );
        let err = parse_starter_content(&json).unwrap_err();
        assert_eq!(
            err,
            StarterError::Content {
                slug: String::new(),
                problem: "rights basis \"cc0\" is both allowed and excluded".to_owned(),
            }
        );
    }

    #[test]
    fn an_unresolvable_catalog_reference_is_rejected() {
        let json = fixture(&body("").replace(
            "\"catalog_id\": \"ing-oil\"",
            "\"catalog_id\": \"ing-nope\"",
        ));
        let err = parse_starter_content(&json).unwrap_err();
        assert_eq!(
            err,
            StarterError::Content {
                slug: "s".to_owned(),
                problem: "line names catalog id \"ing-nope\", which this file does not define"
                    .to_owned(),
            }
        );
    }

    #[test]
    fn an_unknown_json_field_is_rejected() {
        // Proves `deny_unknown_fields` is live: a typoed `cook_reveiw` must not read as
        // pending review.
        let json = fixture(&body(", \"cook_reveiw\": null"));
        let err = parse_starter_content(&json).unwrap_err();
        let StarterError::Parse(detail) = err else {
            panic!("expected a parse error, got {err:?}");
        };
        assert!(detail.contains("cook_reveiw"), "{detail}");
    }

    #[test]
    fn an_unknown_unit_token_is_rejected() {
        let json = fixture(&body("").replace("\"unit\": \"tbsp\"", "\"unit\": \"handful\""));
        assert_eq!(
            parse_starter_content(&json).unwrap_err(),
            StarterError::Recipe(RecipeError::UnknownUnit("handful".to_owned()))
        );
    }

    #[test]
    fn a_zero_quantity_is_rejected() {
        let json = fixture(&body("").replace("\"numer\": 1", "\"numer\": 0"));
        assert_eq!(
            parse_starter_content(&json).unwrap_err(),
            StarterError::Recipe(RecipeError::ZeroQuantity)
        );
        let json = fixture(&body("").replace("\"denom\": 1", "\"denom\": 0"));
        assert_eq!(
            parse_starter_content(&json).unwrap_err(),
            StarterError::Recipe(RecipeError::ZeroDenominator)
        );
    }

    #[test]
    fn shipped_content_excludes_unreviewed_entries() {
        let all = all_starter_content().unwrap();
        let shipped = shipped_starter_content().unwrap();
        assert_eq!(
            shipped.catalog, all.catalog,
            "the catalog is never filtered"
        );
        // Expected-to-pass: this restates D-041's rule from its clauses, in both directions.
        // Asserting `shipped == all.filter(is_shippable)` would merely restate the
        // implementation and could not fail for any content file.
        let shipped_slugs: Vec<&str> = shipped.recipes.iter().map(|r| r.slug.as_str()).collect();
        for recipe in &shipped.recipes {
            // Included: the cook arm stays closed for `original` — the only way to ship
            // without a recorded review is the federal basis with a URL behind it.
            let rights = recipe.provenance.rights().expect("every entry has rights");
            assert!(
                recipe.cook_review.is_some()
                    || (rights.basis() == RightsBasis::UsFederalPublicDomain
                        && recipe.provenance.source_url().is_some()),
                "{} shipped by neither arm",
                recipe.slug
            );
        }
        for recipe in &all.recipes {
            if shipped_slugs.contains(&recipe.slug.as_str()) {
                continue;
            }
            // Excluded: and nothing shippable was dropped on the way out.
            let federal = recipe
                .provenance
                .rights()
                .is_some_and(|r| r.basis() == RightsBasis::UsFederalPublicDomain)
                && recipe.provenance.source_url().is_some();
            assert!(
                recipe.cook_review.is_none() && !federal,
                "{} was excluded but is shippable",
                recipe.slug
            );
        }
    }

    #[test]
    fn the_shipped_roster_meets_the_release_bar() {
        // AC-4's actual evidence. Without it a later content edit could drop the roster below
        // ten and re-open MVP-032's own release blocker with a green suite.
        //
        // The bar is the card's ten. The roster ships 26 by owner decision 2026-09-05 for a
        // reason this test deliberately does not pin: `RECENTLY_EATEN` spans a 14-day history
        // against 7 dinner slots, so at 14 or fewer dishes every candidate carries the same
        // penalty by week 2 and the variety term stops discriminating. Dropping toward ten
        // is legal here and still a real regression in plan quality.
        use RestrictionKind::{Gluten, Vegan, Vegetarian};
        let shipped = shipped_starter_content().unwrap().recipes;
        assert!(shipped.len() >= 10, "{} shipped", shipped.len());
        let covers = |absent: RestrictionKind| {
            shipped
                .iter()
                .any(|r| !r.expected_conflicts.contains(&absent))
        };
        assert!(covers(Vegan), "no vegan dish ships");
        // Documentation, not independent evidence: `components` makes Vegan's set a superset
        // of Vegetarian's, so `covers(Vegan)` already implies this. Kept so AC-4's four arms
        // are all visible in one place.
        assert!(covers(Vegetarian), "no vegetarian dish ships");
        assert!(covers(Gluten), "no gluten-free dish ships");
        // Omnivore is the `Vegetarian` conflict, not the `Vegan` one: only meat, fish and
        // shellfish produce it, whereas a meatless dairy dish carries a Vegan conflict and
        // would satisfy a `Vegan`-keyed check while containing no meat.
        assert!(
            shipped
                .iter()
                .any(|r| r.expected_conflicts.contains(&Vegetarian)),
            "no omnivore dish ships"
        );
    }

    /// The mechanizable half of the card's authorship constraint, and deliberately named for what
    /// it actually decides. `.gov` is **not** federal-only: CISA issues `.gov` to state, local,
    /// tribal and territorial governments too, and the card excludes state content as firmly as
    /// partner content. So this is necessary and nowhere near sufficient -- it catches a `.edu` or
    /// a `.com` reaching CI, and nothing subtler. Which body published the page, and whether that
    /// body wrote the recipe, is settled by AC-2's fresh-context rights review (section A of
    /// `docs/research/mvp-032-verifier-prompt.txt`); on 2026-09-07 that review failed 18 of 24
    /// entries whose hosts all pass this check, which is the measure of the gap.
    ///
    /// Deliberately not a URL parser: there is no url crate in the tree and none is worth adding
    /// for a test over our own content. A scheme-less string is rejected outright rather than
    /// guessed at, and the authority ends at the first `/`, `?` or `#`, so a query string cannot
    /// smuggle a `.gov` past the suffix test. Swap it for a parser the day this reads a URL from
    /// outside the repo.
    fn government_host(url: &str) -> bool {
        let Some((_, rest)) = url.split_once("://") else {
            return false;
        };
        let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let host = &rest[..end];
        host.ends_with(".gov") || host.ends_with(".mil")
    }

    #[test]
    fn government_host_reads_the_authority_and_nothing_else() {
        assert!(government_host(
            "https://www.nhlbi.nih.gov/health/heart-healthy-living/healthy-foods/healthy-eating-recipes"
        ));
        // No path: the arm no shipped entry reaches, since all 24 carry one.
        assert!(government_host("https://www.nhlbi.nih.gov"));
        assert!(!government_host("https://extension.psu.edu/recipe"));
        // Userinfo parses into the authority, so the whole string is the host and it fails.
        assert!(!government_host("https://www.nhlbi.nih.gov@evil.example/x"));
        // A query or fragment ends the authority; without that these two read as `.gov` hosts.
        assert!(!government_host("https://evil.example?src=nih.gov"));
        assert!(!government_host("https://evil.example#nih.gov"));
        // Scheme-less: rejected rather than guessed at, so a bare host cannot pass by accident.
        assert!(!government_host("nhlbi.nih.gov/health/x"));
    }

    #[test]
    fn every_federal_entry_names_its_agency_and_url() {
        // AC-2's automated half: the rights arm ships on a checkable claim, so both halves of
        // that claim must actually be present on every entry that uses it.
        let content = all_starter_content().unwrap();
        let mut federal = 0;
        for recipe in &content.recipes {
            let rights = recipe.provenance.rights().expect("every entry has rights");
            if rights.basis() != RightsBasis::UsFederalPublicDomain {
                continue;
            }
            federal += 1;
            assert!(
                recipe
                    .provenance
                    .source_url()
                    .is_some_and(|u| !u.is_empty()),
                "{}: federal basis without a source URL",
                recipe.slug
            );
            assert!(
                recipe
                    .provenance
                    .source_name()
                    .is_some_and(|n| !n.is_empty()),
                "{}: federal basis without a source name",
                recipe.slug
            );
            assert!(
                rights.attribution().is_some_and(|a| !a.is_empty()),
                "{}: federal basis without the captured attribution line",
                recipe.slug
            );
            let url = recipe.provenance.source_url().unwrap_or_default();
            assert!(
                government_host(url),
                "{}: federal basis on a host that is not even a government one -- {url:?}. \
                 Note the converse is not checked here and cannot be: `.gov` is open to state, \
                 local and tribal governments, so passing this proves very little.",
                recipe.slug
            );
        }
        assert!(federal > 0, "no federal entry to check");
    }

    #[test]
    fn a_recorded_cook_review_moves_an_entry_into_the_shipped_set() {
        // Proves the owner's future edit works with no code change: the same file with a
        // `cook_review` object filled in converts, and carries the log rather than a boolean.
        let reviewed = body("").replace(
            "\"cook_review\": null",
            r#""cook_review": { "cooked_on": "2026-09-01", "by": "David",
                                "corrections": "cut the salt in half" }"#,
        );
        let content = parse_starter_content(&fixture(&reviewed)).unwrap();
        let review = content.recipes[0].cook_review.as_ref().unwrap();
        assert_eq!(review.cooked_on.to_string(), "2026-09-01");
        assert_eq!(review.by, "David");
        assert_eq!(review.corrections.as_deref(), Some("cut the salt in half"));

        let pending = parse_starter_content(&fixture(&body(""))).unwrap();
        assert_eq!(pending.recipes[0].cook_review, None);
    }

    #[test]
    fn a_non_starter_provenance_kind_is_rejected() {
        let json = fixture(&body("").replace("\"kind\": \"starter\"", "\"kind\": \"imported\""));
        assert_eq!(
            parse_starter_content(&json).unwrap_err(),
            StarterError::Content {
                slug: "s".to_owned(),
                problem: "provenance kind is \"imported\", not \"starter\"".to_owned(),
            }
        );
    }

    #[test]
    fn a_duplicate_slug_is_rejected() {
        let json = fixture(&format!("{}, {}", body(""), body("")));
        assert_eq!(
            parse_starter_content(&json).unwrap_err(),
            StarterError::Content {
                slug: "s".to_owned(),
                problem: "duplicate slug".to_owned(),
            }
        );
    }

    // --- MVP-011 step 8: the authored roster --------------------------------------------

    #[test]
    fn every_recipe_matches_its_declared_conflicts() {
        // The mechanical check on the authored allergen profile: `assess` against every known
        // restriction must return exactly the conflicts the entry declares.
        let all_kinds =
            HouseholdRestrictions::new(RestrictionKind::ALL.into_iter().map(Restriction::Known));
        let content = all_starter_content().unwrap();
        assert!(!content.recipes.is_empty());
        for entry in &content.recipes {
            let recipe = Recipe::new(
                crate::recipe::RecipeId::new(entry.slug.clone()).unwrap(),
                HouseholdId::new("h").unwrap(),
                entry.title.clone(),
                entry.servings,
                entry.prep_minutes,
                entry.instructions.clone(),
                entry.lines.clone(),
                entry.provenance.clone(),
            )
            .unwrap();
            let assessment = assess(recipe.lines().iter().map(IngredientLine::name), &all_kinds);
            let raw: Vec<RestrictionKind> = assessment
                .conflicts
                .iter()
                .filter_map(|c| match &c.restriction {
                    Restriction::Known(kind) => Some(*kind),
                    Restriction::Other(_) => None,
                })
                .collect();
            let found = canonical(&raw);
            let declared = canonical(&entry.expected_conflicts);
            assert_eq!(
                found, declared,
                "{}: assess found {found:?}, the file declares {declared:?}",
                entry.slug
            );
        }
    }

    #[test]
    fn every_catalog_id_is_referenced_by_some_entry() {
        // The reverse of `every_catalog_reference_resolves`, which only checks that every
        // reference resolves. Four ids outlived the recipes that used them when two federal
        // entries were dropped at the 2026-09-06 roster review, and nothing caught it: the
        // catalog is installed unfiltered, so an unreferenced id becomes a permanently unusable
        // Pantry row on every device. `all_starter_content` is deliberate -- against the shipped
        // subset this would fail on the ids the ten unshipped entries legitimately hold.
        let content = all_starter_content().unwrap();
        let mut referenced: Vec<&str> = Vec::new();
        for recipe in &content.recipes {
            for line in &recipe.lines {
                if let Some(IngredientRef::Catalog(id)) = line.ingredient() {
                    referenced.push(id.as_str());
                }
            }
        }
        assert!(!referenced.is_empty());
        for ingredient in &content.catalog {
            assert!(
                referenced.contains(&ingredient.id().as_str()),
                "catalog id {} is referenced by no authored entry",
                ingredient.id().as_str()
            );
        }
    }

    /// `assess` reads `IngredientLine::name` and nothing else, so a line named only by a pasta
    /// shape -- or by a sauce or mix whose allergen is not in its name -- matches no term table
    /// and the dish ships free of that restriction *by declaration*. Three shipped wheat-pasta
    /// dishes did, and were closed by renaming the lines; the ones no source text lets us rename
    /// are recorded in `policy.expected_conflicts_note` instead and routed to the `RULE_VERSION`
    /// 1 -> 2 bump in KNOWN_ISSUES.md.
    ///
    /// This sweeps every authored line name against that vocabulary rather than checking a fixed
    /// list of slugs, so a new entry naming a line `ziti` goes red instead of shipping unnoticed.
    /// Each recorded miss is exempted per *entry*, not per word: a blanket exemption on the word
    /// would let the next carrier ship unrecorded, which is the blind spot this test exists to
    /// close. The exempt arm asserts the miss is still a miss, so the day `RULE_VERSION` goes to
    /// 2 this goes red and the exception table and the policy note have to be updated together.
    #[test]
    fn every_line_name_declares_its_known_allergen_words() {
        use RestrictionKind::{Dairy, Fish, Gluten};

        // A word a reader takes as naming an allergen, and the kind it should raise.
        const VOCAB: &[(&str, RestrictionKind)] = &[
            ("macaroni", Gluten),
            ("rotini", Gluten),
            ("orzo", Gluten),
            ("ravioli", Gluten),
            ("ziti", Gluten),
            ("farfalle", Gluten),
            ("rigatoni", Gluten),
            ("linguine", Gluten),
            ("cavatappi", Gluten),
            ("fusilli", Gluten),
            ("penne", Gluten),
            ("angel hair", Gluten),
            ("soy sauce", Gluten),
            ("gravy mix", Gluten),
            ("cream of chicken soup", Gluten),
            ("worcestershire", Fish),
            ("ranch", Dairy),
        ];

        // (slug, word, kind), one row per carrying entry, mirroring the instance lists in
        // `policy.expected_conflicts_note`. Keyed on the slug on purpose -- see the doc comment.
        const RECORDED_RULE_V1_MISSES: &[(&str, &str, RestrictionKind)] = &[
            ("peanut-noodle-bowl", "soy sauce", Gluten),
            ("vegetable-fried-rice", "soy sauce", Gluten),
            ("steamed-salmon-and-mushrooms", "soy sauce", Gluten),
            ("pineapple-chicken-skewers", "soy sauce", Gluten),
            ("egg-roll-in-a-bowl", "soy sauce", Gluten),
            ("crockpot-barbecue-chicken", "ranch", Dairy),
            ("crockpot-chicken-tacos", "ranch", Dairy),
            ("ravioli-bake", "ravioli", Gluten),
            ("barbecued-chicken", "worcestershire", Fish),
            ("easy-shepherds-pie", "worcestershire", Fish),
            (
                "crockpot-sausage-and-hashbrown-casserole",
                "cream of chicken soup",
                Gluten,
            ),
            ("smothered-pork-chops", "cream of chicken soup", Gluten),
            ("taco-soup", "cream of chicken soup", Gluten),
            ("smothered-pork-chops", "gravy mix", Gluten),
        ];

        let content = all_starter_content().unwrap();
        let mut failures: Vec<String> = Vec::new();
        let mut fired = vec![false; RECORDED_RULE_V1_MISSES.len()];
        for recipe in &content.recipes {
            for name in recipe.lines.iter().map(IngredientLine::name) {
                // Every kind, not the three VOCAB happens to use today: a new VOCAB row carrying
                // a fourth kind would otherwise never be swept, and nothing else would notice --
                // `fired` tracks the exception table, not the vocabulary. The `words.is_empty()`
                // guard below makes the unused kinds free.
                for kind in RestrictionKind::ALL {
                    let words: Vec<&str> = VOCAB
                        .iter()
                        .filter(|(word, k)| *k == kind && name.contains(word))
                        .map(|(word, _)| *word)
                        .collect();
                    if words.is_empty() {
                        continue;
                    }
                    // Resolve per (name, kind), never per word -- but a line is exempt only when
                    // *every* allergen word on it is recorded for this entry. Exempting the whole
                    // line as soon as one word matches would let "ziti with gravy mix" on
                    // smothered-pork-chops ride its gravy-mix row and ship the ziti miss green,
                    // which is the blind spot this test exists to close. The failure message
                    // interpolates `words`, so a mixed line still names both causes.
                    let mut recorded = 0;
                    for (i, (slug, word, k)) in RECORDED_RULE_V1_MISSES.iter().enumerate() {
                        if *k == kind && recipe.slug.as_str() == *slug && name.contains(word) {
                            recorded += 1;
                            fired[i] = true;
                        }
                    }
                    let exempt = recorded == words.len();
                    let restrictions = HouseholdRestrictions::new([Restriction::Known(kind)]);
                    let warns = !assess([name], &restrictions).conflicts.is_empty();
                    if exempt && warns {
                        failures.push(format!(
                            "{}: line {name:?} is a recorded rule-version-1 {kind:?} miss but now \
                             warns -- update RECORDED_RULE_V1_MISSES and \
                             policy.expected_conflicts_note together",
                            recipe.slug
                        ));
                    } else if !exempt && !warns {
                        failures.push(format!(
                            "{}: line {name:?} names {words:?} and raises no {kind:?} conflict, \
                             so a {kind:?}-restricted household is not warned",
                            recipe.slug
                        ));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");

        // A row that matches no line means the record and the content have diverged. Five VOCAB
        // words match nothing today, so a count of swept lines would not catch that.
        let cold: Vec<_> = RECORDED_RULE_V1_MISSES
            .iter()
            .zip(&fired)
            .filter(|(_, hit)| !**hit)
            .map(|(row, _)| row)
            .collect();
        assert!(
            cold.is_empty(),
            "recorded misses matching no line in the file: {cold:#?}"
        );
    }

    #[test]
    fn the_three_renamed_wheat_pasta_dishes_declare_gluten() {
        let content = all_starter_content().unwrap();
        for slug in [
            "crockpot-mac-and-cheese",
            "sausage-and-broccoli-orzo",
            "taco-pasta",
        ] {
            let entry = content
                .recipes
                .iter()
                .find(|r| r.slug == slug)
                .unwrap_or_else(|| panic!("{slug} is not in the file"));
            assert!(
                entry.expected_conflicts.contains(&RestrictionKind::Gluten),
                "{slug} declares {:?}, without gluten",
                entry.expected_conflicts
            );
        }
    }

    /// The assertion that would have caught the original defect. `meatball-casserole` declared
    /// gluten before its rotini line was renamed, but only through its separate "pasta sauce"
    /// line -- delete or rename that line and the declaration silently flips. Its pasta line now
    /// carries gluten on its own.
    #[test]
    fn meatball_casserole_carries_gluten_on_its_pasta_line_alone() {
        let content = all_starter_content().unwrap();
        let entry = content
            .recipes
            .iter()
            .find(|r| r.slug == "meatball-casserole")
            .unwrap();
        let gluten = HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Gluten)]);
        let without_sauce = entry
            .lines
            .iter()
            .map(IngredientLine::name)
            .filter(|n| *n != "pasta sauce");
        assert!(!assess(without_sauce, &gluten).conflicts.is_empty());
    }

    /// Written in the negative on purpose, and it is the one pasta entry still standing on the
    /// accident the test above closes: `ravioli-bake` declares gluten only through its "pasta
    /// sauce" line, because no source text lets "frozen cheese ravioli" be extended to name pasta.
    /// This pins the current state so the day `GLUTEN_TERMS` gains "ravioli" -- the MVP-009 rule
    /// change with a `RULE_VERSION` bump recorded in `KNOWN_ISSUES.md` -- this goes red and is
    /// deleted, rather than the residual passing unnoticed.
    #[test]
    fn ravioli_bake_declares_gluten_only_through_its_sauce_line() {
        let content = all_starter_content().unwrap();
        let entry = content
            .recipes
            .iter()
            .find(|r| r.slug == "ravioli-bake")
            .unwrap();
        assert!(entry.expected_conflicts.contains(&RestrictionKind::Gluten));
        let gluten = HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Gluten)]);
        let without_sauce = entry
            .lines
            .iter()
            .map(IngredientLine::name)
            .filter(|n| *n != "pasta sauce");
        assert!(assess(without_sauce, &gluten).conflicts.is_empty());
    }

    #[test]
    fn canonical_collapses_interleaved_duplicates_and_ignores_authoring_order() {
        // The first vector is the shape a lines-outer `assess` refactor would produce — same-kind
        // conflicts no longer adjacent, which is exactly what a bare `dedup` fails to collapse.
        // The second is the same set authored in a different order in the content file.
        use RestrictionKind::{Dairy, Gluten, Vegan};
        assert_eq!(
            canonical(&[Dairy, Gluten, Dairy, Vegan]),
            vec![Dairy, Gluten, Vegan]
        );
        assert_eq!(
            canonical(&[Vegan, Gluten, Dairy]),
            vec![Dairy, Gluten, Vegan]
        );
    }
}
