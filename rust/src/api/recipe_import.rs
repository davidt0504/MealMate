//! Recipe import from a link (OPT-001): fetch and read the page in `recipe-import`, then hand
//! Dart unsaved drafts to review. Nothing is written here; the review saves through the
//! ordinary `save_recipe`.

use std::sync::Arc;

use kimatta_storage::{
    ComponentStatus, HouseholdId, HouseholdRestrictions, IngredientLine, ProvenanceKind, Quantity,
    Rational, Recipe, RecipeComponent, RecipeId, RecipeProvenance, RecipeRecord, Unit,
};
use recipe_import::{ComponentKind, ImportDraft, Limits, Resolver};

use crate::api::error::KimattaError;
use crate::api::recipe::{id_or_minted, recipe_from_domain, RecipeDto};

/// One reviewable recipe. `recipe.id` is empty, so saving it creates a new recipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDraftDto {
    pub recipe: RecipeDto,
    /// The one consolidated "couldn't add N parts" notice, or `None`.
    pub notice: Option<String>,
    /// Indexes into `recipe.lines` that name a sub-recipe already listed inline.
    pub inline_lines: Vec<u32>,
}

/// A recipe this household already saved from the same link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingRecipeDto {
    pub id: String,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportResultDto {
    /// One per Recipe on the page; more than one means the user picks.
    pub drafts: Vec<ImportDraftDto>,
    pub existing: Option<ExistingRecipeDto>,
}

/// Fetches `url` and returns its drafts plus any recipe already saved from it. Runs the
/// network part without the database lock, then takes the lock once for the lookup.
pub fn import_recipe_from_url(
    household_id: String,
    url: String,
) -> Result<ImportResultDto, KimattaError> {
    import_with(
        &household_id,
        &url,
        &Limits::default(),
        recipe_import::system_dns(),
    )
}

pub(crate) fn import_with(
    household_id: &str,
    url: &str,
    limits: &Limits,
    dns: Arc<dyn Resolver>,
) -> Result<ImportResultDto, KimattaError> {
    let household = HouseholdId::new(household_id)?;
    let drafts = recipe_import::import(url, limits, dns)?;
    crate::db::with(|conn| {
        let restrictions = kimatta_storage::load_restrictions(conn, &household)?;
        let existing =
            kimatta_storage::find_recipe_by_source_url(conn, &household, &drafts[0].source_url)?
                .map(|(id, archived)| ExistingRecipeDto {
                    id: id.as_str().to_owned(),
                    archived,
                });
        let drafts = drafts
            .into_iter()
            .map(|d| draft_dto(&household, d, &restrictions))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ImportResultDto { drafts, existing })
    })
}

/// Builds the draft through the domain constructors, so a draft that reaches Dart is one
/// `save_recipe` will accept unless the user breaks it.
fn draft_dto(
    household: &HouseholdId,
    d: ImportDraft,
    restrictions: &HouseholdRestrictions,
) -> Result<ImportDraftDto, KimattaError> {
    // Every line enters unresolved (owner decision 4): its text is the line, verbatim.
    let lines = d
        .lines
        .iter()
        .map(|l| {
            IngredientLine::new(
                l.text.clone(),
                l.text.clone(),
                None,
                Quantity::Unknown,
                Unit::None,
                None,
                false,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let owners: Vec<Option<u32>> = d.lines.iter().map(|l| l.component).collect();
    let components = d
        .components
        .into_iter()
        .map(|c| {
            let scale = c.scale.map(|(n, d)| Rational::new(n, d)).transpose()?;
            RecipeComponent::new(
                c.position,
                c.title,
                c.source_url,
                scale,
                c.replaced_text,
                match c.kind {
                    ComponentKind::Expanded => ComponentStatus::Expanded,
                    ComponentKind::Unresolved => ComponentStatus::Unresolved,
                    ComponentKind::Alternative => ComponentStatus::Alternative,
                },
                c.links,
                c.instructions,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let recipe = Recipe::new(
        // A placeholder id only to satisfy the constructor; the draft crosses with "".
        RecipeId::new(id_or_minted(String::new()))?,
        household.clone(),
        d.title,
        d.servings,
        d.prep_minutes,
        d.instructions,
        lines,
        RecipeProvenance::new(
            ProvenanceKind::Imported,
            Some(d.source_url),
            d.source_name,
            d.source_author,
        )?,
    )?
    .with_components(components, &owners)?;
    let mut dto = recipe_from_domain(
        &RecipeRecord {
            recipe,
            archived_at: None,
        },
        restrictions,
    );
    dto.id = String::new();
    Ok(ImportDraftDto {
        recipe: dto,
        notice: d.notice,
        inline_lines: d.inline_lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::error::ImportErrorKind;
    use crate::db::TEST_DB_LOCK;
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, SocketAddr, TcpListener};
    use std::time::Duration;

    #[derive(Debug)]
    struct Loopback;

    impl Resolver for Loopback {
        fn resolve(
            &self,
            uri: &recipe_import::Uri,
            _config: &recipe_import::Config,
            _timeout: recipe_import::NextTimeout,
        ) -> Result<recipe_import::ResolvedSocketAddrs, recipe_import::UreqError> {
            let mut out = self.empty();
            out.push(SocketAddr::new(
                Ipv4Addr::LOCALHOST.into(),
                uri.port_u16().unwrap_or(80),
            ));
            Ok(out)
        }
    }

    /// Serves `body` as HTML to every request, `requests` times.
    fn serve(requests: usize, status: &'static str, body: String) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming().take(requests) {
                let Ok(mut s) = stream else { return };
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                let _ = s.write_all(
                    format!(
                        "HTTP/1.1 {status}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
            }
        });
        port
    }

    fn recipe_page(json: &str) -> String {
        format!(r#"<script type="application/ld+json">{json}</script>"#)
    }

    fn limits() -> Limits {
        Limits::for_local_tests(Duration::from_secs(5))
    }

    fn open_db(dir: &tempfile::TempDir) -> String {
        let path = dir.path().join("kimatta.db").to_str().unwrap().to_owned();
        crate::api::health::open_database(path).unwrap();
        crate::api::household::bootstrap_household().unwrap().id
    }

    #[test]
    fn a_draft_carries_imported_provenance_and_unresolved_lines() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let household = open_db(&dir);
        let port = serve(
            1,
            "200 OK",
            recipe_page(
                r#"{"@type":"Recipe","name":"Toast","author":{"name":"  "},"prepTime":"PT0M",
                    "recipeIngredient":["1 slice bread","butter, to taste"],"recipeInstructions":"Toast."}"#,
            ),
        );
        let result = import_with(
            &household,
            &format!("http://public.test:{port}/toast/"),
            &limits(),
            Arc::new(Loopback),
        )
        .unwrap();
        assert_eq!(result.existing, None);
        let draft = &result.drafts[0];
        let r = &draft.recipe;
        assert_eq!(r.id, "");
        assert_eq!(r.household_id, household);
        assert_eq!(r.provenance.kind, "imported");
        assert_eq!(
            r.provenance.source_url.as_deref(),
            Some(format!("http://public.test:{port}/toast").as_str())
        );
        assert_eq!(
            r.provenance.source_author, None,
            "a blank author is dropped"
        );
        assert_eq!(r.prep_minutes, None, "PT0M is no estimate");
        assert_eq!(r.lines.len(), 2);
        for l in &r.lines {
            assert_eq!(l.name, l.original_text);
            assert_eq!(l.ingredient, None);
            assert_eq!(l.quantity, crate::api::recipe::QuantityDto::Unknown);
            assert_eq!(l.unit, crate::api::recipe::UnitDto::None);
        }
        // The draft saves as is.
        let saved = crate::api::recipe::save_recipe(r.clone()).unwrap();
        assert_eq!(saved.lines, r.lines);
        assert_eq!(saved.provenance.kind, "imported");
    }

    #[test]
    fn a_link_already_saved_is_reported_as_existing() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let household = open_db(&dir);
        let page = recipe_page(r#"{"@type":"Recipe","name":"Toast","recipeIngredient":["1 egg"]}"#);
        let port = serve(2, "200 OK", page);
        let url = format!("http://public.test:{port}/toast");
        let first = import_with(&household, &url, &limits(), Arc::new(Loopback)).unwrap();
        let saved = crate::api::recipe::save_recipe(first.drafts[0].recipe.clone()).unwrap();
        let again = import_with(
            &household,
            &format!("{url}/#top"),
            &limits(),
            Arc::new(Loopback),
        )
        .unwrap();
        assert_eq!(
            again.existing,
            Some(ExistingRecipeDto {
                id: saved.id,
                archived: false
            })
        );
    }

    #[test]
    fn several_recipes_give_several_drafts() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let household = open_db(&dir);
        let port = serve(
            1,
            "200 OK",
            recipe_page(r#"[{"@type":"Recipe","name":"A"},{"@type":"Recipe","name":"B"}]"#),
        );
        let result = import_with(
            &household,
            &format!("http://public.test:{port}/"),
            &limits(),
            Arc::new(Loopback),
        )
        .unwrap();
        let titles: Vec<_> = result
            .drafts
            .iter()
            .map(|d| d.recipe.title.as_str())
            .collect();
        assert_eq!(titles, ["A", "B"]);
    }

    #[test]
    fn failures_map_to_their_kinds() {
        let _guard = TEST_DB_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        let household = open_db(&dir);
        let kind = |e: KimattaError| match e {
            KimattaError::Import { kind, url } => (kind, url),
            other => panic!("{other:?}"),
        };
        let port = serve(1, "403 Forbidden", String::new());
        let err = import_with(
            &household,
            &format!("http://public.test:{port}/"),
            &limits(),
            Arc::new(Loopback),
        )
        .unwrap_err();
        assert_eq!(kind(err), (ImportErrorKind::Refused, None));
        let port = serve(1, "200 OK", "<p>no recipe</p>".to_owned());
        let err = import_with(
            &household,
            &format!("http://public.test:{port}/a/"),
            &limits(),
            Arc::new(Loopback),
        )
        .unwrap_err();
        assert_eq!(
            kind(err),
            (
                ImportErrorKind::NoRecipe,
                Some(format!("http://public.test:{port}/a"))
            )
        );
        let err = import_with(
            &household,
            "file:///etc/passwd",
            &limits(),
            Arc::new(Loopback),
        )
        .unwrap_err();
        assert_eq!(kind(err), (ImportErrorKind::Blocked, None));
        let err = import_with(
            &household,
            "https://example.com/",
            &Limits::default(),
            Arc::new(Loopback),
        )
        .unwrap_err();
        assert_eq!(
            kind(err),
            (ImportErrorKind::Blocked, None),
            "production limits refuse loopback"
        );
    }

    #[test]
    fn an_import_error_never_shows_its_url() {
        let e: KimattaError = recipe_import::ImportError::NoRecipe {
            url: "https://secret.example/x".into(),
        }
        .into();
        assert!(!e.to_string().contains("secret"), "{e}");
    }
}
