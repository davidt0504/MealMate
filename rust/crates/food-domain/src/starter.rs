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
    /// `None` = pending review. Nothing with `None` is shipped.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarterContent {
    pub catalog: Vec<Ingredient>,
    pub recipes: Vec<StarterRecipe>,
}

/// Every authored entry, reviewed or not — the manifest view, for tests and rights review.
pub fn all_starter_content() -> Result<StarterContent, StarterError> {
    parse_starter_content(CONTENT)
}

/// Only entries carrying a recorded cook review — the shipped view, what the installer sees.
/// The catalog is not filtered: an ingredient name is a fact, carrying no rights question and
/// no cook question.
pub fn shipped_starter_content() -> Result<StarterContent, StarterError> {
    let all = all_starter_content()?;
    Ok(StarterContent {
        catalog: all.catalog,
        recipes: all
            .recipes
            .into_iter()
            .filter(|r| r.cook_review.is_some())
            .collect(),
    })
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
        for recipe in &shipped.recipes {
            assert!(recipe.cook_review.is_some(), "{}", recipe.slug);
        }
        let reviewed = all
            .recipes
            .iter()
            .filter(|r| r.cook_review.is_some())
            .count();
        assert_eq!(shipped.recipes.len(), reviewed);
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
