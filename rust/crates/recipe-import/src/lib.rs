//! Recipe import from a public URL (OPT-001): fetch under a strict network guard, read the
//! page's schema.org JSON-LD, find linked sub-recipes on WP Recipe Maker / Tasty cards, and
//! assemble one reviewable draft. Nothing here touches storage; the bridge saves only after
//! the user confirms.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use thiserror::Error;

mod extract;
mod fetch;
mod resolve;

pub use extract::ParsedRecipe;
pub use fetch::{fetch_page, guard_url, is_global, normalize_url, system_dns, FetchedPage, Limits};
pub use resolve::{ComponentKind, DraftComponent, DraftLine};
/// The DNS seam: `system_dns()` in production, a fixed answer in tests. Re-exported with
/// the types an implementation needs, so callers need no direct ureq dependency.
pub use ureq::config::Config;
pub use ureq::http::Uri;
pub use ureq::unversioned::resolver::{ResolvedSocketAddrs, Resolver};
pub use ureq::unversioned::transport::NextTimeout;
pub use ureq::Error as UreqError;

/// Why an import produced no draft. `Display` is content-free by design: no URL, page text
/// or address ever appears in it (design §3 diagnostics).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ImportError {
    #[error("offline or the site can't be reached")]
    Offline,
    #[error("the site took too long")]
    Timeout,
    #[error("the site refused the request")]
    Refused,
    #[error("that link can't be imported")]
    Blocked,
    #[error("the page can't be read")]
    Unreadable,
    /// No Recipe data on the page; `url` is the normalized final URL so the user can still
    /// add the recipe by hand with its source kept.
    #[error("no recipe data on the page")]
    NoRecipe { url: String },
}

/// One reviewable recipe, not yet saved. Every line enters unresolved (owner decision 4):
/// the bridge stores each as its own text with no catalog match, quantity or unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDraft {
    pub title: String,
    pub servings: Option<u32>,
    pub prep_minutes: Option<u32>,
    pub instructions: String,
    pub lines: Vec<DraftLine>,
    pub components: Vec<DraftComponent>,
    pub notice: Option<String>,
    pub inline_lines: Vec<u32>,
    /// The normalized final URL, also the duplicate-check key.
    pub source_url: String,
    pub source_name: Option<String>,
    pub source_author: Option<String>,
}

/// Fetches `url` and builds its draft(s): one per Recipe on the page. A page with several
/// Recipes gets no component resolution (the user picks one first). A child page that
/// can't be fetched or read leaves its component unresolved; it never fails the import.
pub fn import(
    url: &str,
    limits: &Limits,
    dns: Arc<dyn Resolver>,
) -> Result<Vec<ImportDraft>, ImportError> {
    let page = fetch_page(url, limits, dns.clone())?;
    let recipes = extract::recipes(&page.html);
    // The stored source is the normalized final URL: the duplicate-check key (design §8).
    let source_url = normalize_url(page.final_url.as_str(), limits)?.to_string();
    if recipes.is_empty() {
        return Err(if extract::looks_like_challenge(&page.html) {
            ImportError::Refused
        } else {
            ImportError::NoRecipe { url: source_url }
        });
    }
    let source_name = extract::site_name(&page.html);
    let draft = |recipe: &ParsedRecipe, assembly: resolve::Assembly| ImportDraft {
        title: recipe.name.clone(),
        servings: recipe.servings,
        prep_minutes: recipe.prep_minutes,
        instructions: recipe.instructions.clone(),
        lines: assembly.lines,
        components: assembly.components,
        notice: assembly.notice,
        inline_lines: assembly.inline_lines,
        source_url: source_url.clone(),
        source_name: source_name.clone(),
        source_author: recipe.author.clone(),
    };
    if recipes.len() > 1 {
        let none = HashMap::new();
        return Ok(recipes
            .iter()
            .map(|r| draft(r, resolve::assemble(r, &none, &[], &[], 0)))
            .collect());
    }
    let recipe = &recipes[0];
    let anchors = extract::locate(&page.html);
    let decisions = resolve::classify(&anchors, &recipe.ingredients, &page.final_url, 1, 0);
    let targets = resolve::child_targets(&decisions);
    let page_site = page.final_url.clone();
    // Each child gets its own page budget; the fetches overlap, so the whole import stays
    // within about two page budgets.
    let children: HashMap<String, ParsedRecipe> = thread::scope(|scope| {
        let handles: Vec<_> = targets
            .iter()
            .map(|t| {
                let dns = dns.clone();
                (t, scope.spawn(move || fetch_page(t, limits, dns)))
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|(t, h)| {
                let page = h.join().ok()?.ok()?;
                // The child link is same-site; where it redirected must be too.
                (resolve::same_site(&page.final_url, &page_site)).then_some(())?;
                let mut found = extract::recipes(&page.html);
                (found.len() == 1).then(|| (t.clone(), found.remove(0)))
            })
            .collect()
    });
    let assembly = resolve::assemble(recipe, &children, &decisions, &targets, 0);
    Ok(vec![draft(recipe, assembly)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::tests::{html, loopback, serve};
    use std::time::Duration;

    fn page(base: &str, json: &str, card: &str) -> String {
        format!(
            r#"<html><head><meta property="og:site_name" content="Test Kitchen">
            <script type="application/ld+json">{json}</script></head><body>{card}</body></html>"#
        )
        .replace("BASE", base)
    }

    fn limits() -> Limits {
        Limits::for_local_tests(Duration::from_secs(5))
    }

    #[test]
    fn a_page_with_a_linked_sauce_imports_expanded() {
        let port_cell = std::sync::Arc::new(std::sync::OnceLock::<u16>::new());
        let cell = port_cell.clone();
        let port = serve(2, move |path| {
            let base = format!("http://public.test:{}", cell.get().unwrap());
            if path == "/sauce" {
                html(&page(
                    &base,
                    r#"{"@type":"Recipe","name":"Lemon yogurt sauce","recipeIngredient":["1 cup yogurt","1 lemon"],"recipeInstructions":"Whisk."}"#,
                    "",
                ))
            } else {
                html(&page(
                    &base,
                    r#"{"@type":"Recipe","name":"Shawarma","author":{"name":"Cook"},"recipeIngredient":["1 kg chicken","1 batch Lemon yogurt sauce"],"recipeInstructions":"Grill.","recipeYield":"4"}"#,
                    r#"<div class="wprm-recipe-container"><li class="wprm-recipe-ingredient">1 batch <a href="BASE/sauce">Lemon yogurt sauce</a></li></div>"#,
                ))
            }
        });
        port_cell.set(port).unwrap();
        let drafts = import(
            &format!("http://public.test:{port}/shawarma/?utm_source=x"),
            &limits(),
            loopback(),
        )
        .unwrap();
        assert_eq!(drafts.len(), 1);
        let d = &drafts[0];
        assert_eq!(d.title, "Shawarma");
        assert_eq!(d.source_url, format!("http://public.test:{port}/shawarma"));
        assert_eq!(d.source_name.as_deref(), Some("Test Kitchen"));
        assert_eq!(d.source_author.as_deref(), Some("Cook"));
        assert_eq!(d.servings, Some(4));
        let lines: Vec<_> = d.lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(lines, ["1 kg chicken", "1 cup yogurt", "1 lemon"]);
        assert_eq!(d.components[0].kind, ComponentKind::Expanded);
        assert_eq!(d.components[0].instructions, "1. Whisk.");
        assert_eq!(d.notice, None);
    }

    #[test]
    fn a_failed_child_fetch_leaves_the_component_unresolved() {
        let cell = std::sync::Arc::new(std::sync::OnceLock::<u16>::new());
        let c2 = cell.clone();
        let port = serve(2, move |path| {
            if path == "/sauce" {
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
            } else {
                html(&page(
                    &format!("http://public.test:{}", c2.get().unwrap()),
                    r#"{"@type":"Recipe","name":"Shawarma","recipeIngredient":["1 batch Lemon yogurt sauce"],"recipeInstructions":"Grill."}"#,
                    r#"<div class="wprm-recipe-container"><li class="wprm-recipe-ingredient">1 batch <a href="BASE/sauce">Lemon yogurt sauce</a></li></div>"#,
                ))
            }
        });
        cell.set(port).unwrap();
        let d = &import(
            &format!("http://public.test:{port}/"),
            &limits(),
            loopback(),
        )
        .unwrap()[0];
        assert_eq!(d.lines[0].text, "1 batch Lemon yogurt sauce");
        assert_eq!(d.components[0].kind, ComponentKind::Unresolved);
        assert!(d.notice.is_some());
    }

    #[test]
    fn a_child_that_redirects_to_another_site_stays_unresolved() {
        let cell = std::sync::Arc::new(std::sync::OnceLock::<u16>::new());
        let c2 = cell.clone();
        let port = serve(3, move |path| {
            let port = c2.get().unwrap();
            match path {
                "/sauce" => format!(
                    "HTTP/1.1 302 Found\r\nLocation: http://elsewhere.test:{port}/sauce2\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .into_bytes(),
                "/sauce2" => html(&page(
                    "",
                    r#"{"@type":"Recipe","name":"Sauce","recipeIngredient":["1 cup yogurt"],"recipeInstructions":"Whisk."}"#,
                    "",
                )),
                _ => html(&page(
                    &format!("http://public.test:{port}"),
                    r#"{"@type":"Recipe","name":"Shawarma","recipeIngredient":["1 batch Lemon yogurt sauce"],"recipeInstructions":"Grill."}"#,
                    r#"<div class="wprm-recipe-container"><li class="wprm-recipe-ingredient">1 batch <a href="BASE/sauce">Lemon yogurt sauce</a></li></div>"#,
                )),
            }
        });
        cell.set(port).unwrap();
        let d = &import(
            &format!("http://public.test:{port}/"),
            &limits(),
            loopback(),
        )
        .unwrap()[0];
        assert_eq!(d.lines[0].text, "1 batch Lemon yogurt sauce");
        assert_eq!(d.components[0].kind, ComponentKind::Unresolved);
    }

    #[test]
    fn several_recipes_give_one_plain_draft_each() {
        let port = serve(1, |_| {
            html(&page(
                "",
                r#"[{"@type":"Recipe","name":"A","recipeIngredient":["1 egg"]},{"@type":"Recipe","name":"B"}]"#,
                "",
            ))
        });
        let drafts = import(
            &format!("http://public.test:{port}/"),
            &limits(),
            loopback(),
        )
        .unwrap();
        let titles: Vec<_> = drafts.iter().map(|d| d.title.as_str()).collect();
        assert_eq!(titles, ["A", "B"]);
        assert!(drafts.iter().all(|d| d.components.is_empty()));
    }

    #[test]
    fn a_page_without_recipe_data_or_behind_a_challenge_fails_clearly() {
        let port = serve(1, |_| html("<p>About us</p>"));
        let err = import(
            &format!("http://public.test:{port}/about/"),
            &limits(),
            loopback(),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ImportError::NoRecipe {
                url: format!("http://public.test:{port}/about")
            }
        );
        let port = serve(1, |_| {
            html("<div class=cf-chl-widget>Checking your browser</div>")
        });
        let err = import(
            &format!("http://public.test:{port}/"),
            &limits(),
            loopback(),
        )
        .unwrap_err();
        assert_eq!(err, ImportError::Refused);
    }

    #[test]
    fn error_messages_carry_no_url_or_page_text() {
        let all = [
            ImportError::Offline,
            ImportError::Timeout,
            ImportError::Refused,
            ImportError::Blocked,
            ImportError::Unreadable,
            ImportError::NoRecipe {
                url: "https://secret.example/x".into(),
            },
        ];
        for e in all {
            let text = e.to_string();
            assert!(!text.contains("http") && !text.contains("secret"), "{text}");
        }
    }
}
