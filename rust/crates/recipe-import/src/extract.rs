//! Reads a page: schema.org Recipe JSON-LD (always), and the visible WP Recipe Maker / Tasty
//! Recipes card only to find links on ingredient lines (OPT-001 owner decision 1). The JSON
//! is parsed, never evaluated; markup is parsed, never rendered.

use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;
use std::sync::LazyLock;

/// One Recipe from the page's JSON-LD, already cleaned to what the domain accepts: blank
/// lines dropped, out-of-range servings and prep time left `None`, blank author `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRecipe {
    pub name: String,
    pub ingredients: Vec<String>,
    pub instructions: String,
    pub servings: Option<u32>,
    pub prep_minutes: Option<u32>,
    pub author: Option<String>,
}

/// A link inside the recipe card, with the text it sits in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub href: Option<String>,
    pub text: String,
    /// The whole ingredient line when `ingredient`, else the link's parent's text.
    pub context: String,
    pub ingredient: bool,
    /// Every class on the link and its ancestors, space-joined.
    pub classes: String,
}

const MAX_SERVINGS: u32 = 10_000;
const MAX_PREP_MINUTES: u32 = 10_000;

/// `BeautifulSoup.get_text(' ', strip=True)`: every text node trimmed, empties dropped, joined
/// by one space — the joining the research classifier's comparisons were measured with.
pub(crate) fn text_of(el: ElementRef<'_>) -> String {
    el.text()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The text of a JSON-LD string as a browser would show it: entities decoded ("spooned
/// &amp; leveled", which publishers often double-encode) and any markup reduced to its text,
/// so stray tags never reach a recipe.
pub(crate) fn decode_entities(s: &str) -> String {
    if !s.contains(['&', '<']) {
        return s.to_owned();
    }
    let fragment = Html::parse_fragment(s);
    fragment.root_element().text().collect()
}

fn selector(css: &str) -> Selector {
    Selector::parse(css).expect("static selector")
}

static LD_JSON: LazyLock<Selector> =
    LazyLock::new(|| selector(r#"script[type="application/ld+json"]"#));
static CARD_LINKS: LazyLock<Selector> = LazyLock::new(|| {
    selector(
        ".wprm-recipe-container a, .tasty-recipes a, .recipe__ingredients a, .recipe__instructions a",
    )
});
static SITE_NAME: LazyLock<Selector> =
    LazyLock::new(|| selector(r#"meta[property="og:site_name"]"#));

/// Every Recipe object in the page's JSON-LD, in document order. Scripts that fail to parse
/// are skipped, so one malformed block never hides a valid one beside it.
pub fn recipes(html: &str) -> Vec<ParsedRecipe> {
    let doc = Html::parse_document(html);
    let mut found = Vec::new();
    for script in doc.select(&LD_JSON) {
        let raw: String = script.text().collect();
        if let Ok(value) = serde_json::from_str::<Value>(&raw) {
            let mut in_script = Vec::new();
            collect_recipes(&value, &mut in_script);
            found.extend(in_script.into_iter().filter_map(parse_recipe));
        }
    }
    found
}

fn is_recipe(obj: &serde_json::Map<String, Value>) -> bool {
    match obj.get("@type") {
        Some(Value::String(t)) => t == "Recipe",
        Some(Value::Array(ts)) => ts.iter().any(|t| t.as_str() == Some("Recipe")),
        _ => false,
    }
}

/// Walks `@graph`, arrays and nested objects; a Recipe is collected whole and not descended.
fn collect_recipes<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Array(items) => items.iter().for_each(|v| collect_recipes(v, out)),
        Value::Object(obj) if is_recipe(obj) => out.push(value),
        Value::Object(obj) => obj.values().for_each(|v| collect_recipes(v, out)),
        _ => {}
    }
}

fn clean(s: &str) -> Option<String> {
    let text = decode_entities(s).trim().to_owned();
    (!text.is_empty()).then_some(text)
}

fn parse_recipe(value: &Value) -> Option<ParsedRecipe> {
    let name = value.get("name").and_then(Value::as_str).and_then(clean)?;
    let ingredients = match value.get("recipeIngredient") {
        Some(Value::Array(lines)) => lines
            .iter()
            .filter_map(Value::as_str)
            .filter_map(clean)
            .collect(),
        Some(Value::String(line)) => clean(line).into_iter().collect(),
        _ => Vec::new(),
    };
    Some(ParsedRecipe {
        name,
        ingredients,
        instructions: value
            .get("recipeInstructions")
            .map(instructions)
            .unwrap_or_default(),
        servings: value.get("recipeYield").and_then(servings),
        prep_minutes: value
            .get("prepTime")
            .and_then(Value::as_str)
            .and_then(iso_minutes)
            .filter(|m| (1..=MAX_PREP_MINUTES).contains(m)),
        author: value.get("author").and_then(author),
    })
}

/// Strings, `HowToStep.text` and `HowToSection.name` + its steps, as numbered text with a
/// heading line per section. Numbering restarts in each section.
fn instructions(value: &Value) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut n = 0;
    flatten_steps(value, &mut out, &mut n);
    out.join("\n")
}

fn flatten_steps(value: &Value, out: &mut Vec<String>, n: &mut usize) {
    match value {
        Value::String(s) => {
            if let Some(step) = clean(s) {
                *n += 1;
                out.push(format!("{n}. {step}"));
            }
        }
        Value::Array(items) => items.iter().for_each(|v| flatten_steps(v, out, n)),
        Value::Object(obj) => {
            let section = match obj.get("@type") {
                Some(Value::String(t)) => t == "HowToSection",
                _ => false,
            };
            if section {
                if let Some(title) = obj.get("name").and_then(Value::as_str).and_then(clean) {
                    out.push(title);
                }
                let mut inner = 0;
                if let Some(steps) = obj.get("itemListElement") {
                    flatten_steps(steps, out, &mut inner);
                }
            } else if let Some(text) = obj.get("text").or_else(|| obj.get("name")) {
                flatten_steps(text, out, n);
            }
        }
        _ => {}
    }
}

static SERVINGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*(\d{1,6})\s*(?:servings?)?\s*$").unwrap());

/// Only a bare integer or "N servings" is a serving count; "6 x 20cm flatbreads" or "1 cup"
/// is a yield, not servings, and stays out of the field.
fn servings(value: &Value) -> Option<u32> {
    let parse = |v: &Value| -> Option<u32> {
        let n = match v {
            Value::Number(n) => u32::try_from(n.as_u64()?).ok()?,
            Value::String(s) => SERVINGS.captures(s)?[1].parse().ok()?,
            _ => return None,
        };
        (1..=MAX_SERVINGS).contains(&n).then_some(n)
    };
    match value {
        Value::Array(items) => items.iter().find_map(parse),
        v => parse(v),
    }
}

static ISO_DURATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^P(?:(\d+)D)?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:\d+(?:\.\d+)?S)?)?$").unwrap()
});

/// Whole minutes of an ISO-8601 duration such as `PT1H30M`; seconds are dropped.
fn iso_minutes(raw: &str) -> Option<u32> {
    let caps = ISO_DURATION.captures(raw.trim())?;
    let part =
        |i: usize| -> Option<u64> { caps.get(i).map_or(Some(0), |m| m.as_str().parse().ok()) };
    let minutes = part(1)? * 1440 + part(2)? * 60 + part(3)?;
    u32::try_from(minutes).ok()
}

fn author(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => clean(s),
        Value::Object(obj) => obj.get("name").and_then(Value::as_str).and_then(clean),
        Value::Array(items) => items.iter().find_map(author),
        _ => None,
    }
}

/// The site's own name (`og:site_name`, else the JSON-LD publisher), or `None`.
pub fn site_name(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    if let Some(name) = doc
        .select(&SITE_NAME)
        .find_map(|m| m.value().attr("content"))
        .and_then(clean)
    {
        return Some(name);
    }
    let mut found = None;
    for script in doc.select(&LD_JSON) {
        let raw: String = script.text().collect();
        if let Ok(value) = serde_json::from_str::<Value>(&raw) {
            found = found.or_else(|| publisher_name(&value));
        }
    }
    found
}

fn publisher_name(value: &Value) -> Option<String> {
    match value {
        Value::Array(items) => items.iter().find_map(publisher_name),
        Value::Object(obj) => obj
            .get("publisher")
            .and_then(|p| p.get("name"))
            .and_then(Value::as_str)
            .and_then(clean)
            .or_else(|| obj.values().find_map(publisher_name)),
        _ => None,
    }
}

/// Whether a page with no Recipe looks like a bot challenge rather than a non-recipe page.
pub fn looks_like_challenge(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    lower.contains("cf-chl") || lower.contains("captcha")
}

fn has_class(el: &ElementRef<'_>, class: &str) -> bool {
    el.value().classes().any(|c| c == class)
}

/// The ingredient line an anchor sits in: a WPRM ingredient element, or a `<li>` inside a
/// Tasty ingredients list.
fn ingredient_line<'a>(chain: &[ElementRef<'a>]) -> Option<ElementRef<'a>> {
    if let Some(el) = chain
        .iter()
        .find(|e| has_class(e, "wprm-recipe-ingredient"))
    {
        return Some(*el);
    }
    chain.iter().enumerate().find_map(|(n, e)| {
        (e.value().name() == "li"
            && chain[n..]
                .iter()
                .any(|x| has_class(x, "tasty-recipes-ingredients")))
        .then_some(*e)
    })
}

/// Links inside a recognised recipe card, in document order. Any other card format yields
/// none, so its lines import as plain text.
pub fn locate(html: &str) -> Vec<Anchor> {
    let doc = Html::parse_document(html);
    doc.select(&CARD_LINKS)
        .map(|a| {
            let chain: Vec<ElementRef<'_>> = std::iter::once(a)
                .chain(a.ancestors().filter_map(ElementRef::wrap))
                .collect();
            let line = ingredient_line(&chain);
            let classes = chain
                .iter()
                .flat_map(|e| e.value().classes())
                .collect::<Vec<_>>()
                .join(" ");
            let context = match line {
                Some(li) => text_of(li),
                None => a
                    .parent()
                    .and_then(ElementRef::wrap)
                    .map(text_of)
                    .unwrap_or_default(),
            };
            Anchor {
                href: a.value().attr("href").map(str::to_owned),
                text: text_of(a),
                context,
                ingredient: line.is_some(),
                classes,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(json: &str) -> String {
        format!(r#"<html><head><script type="application/ld+json">{json}</script></head></html>"#)
    }

    #[test]
    fn a_graph_recipe_is_mapped_field_by_field() {
        let html = page(
            r#"{"@context":"https://schema.org","@graph":[
                {"@type":"WebSite","name":"Site"},
                {"@type":["Recipe"],"name":"Chicken &amp; Rice",
                 "recipeIngredient":["1/4 cup (31g) flour (spooned &amp; leveled)","  ","2 eggs"],
                 "recipeInstructions":[
                   {"@type":"HowToSection","name":"Sauce","itemListElement":[
                     {"@type":"HowToStep","text":"Whisk."},{"@type":"HowToStep","text":"Chill."}]},
                   {"@type":"HowToSection","name":"Chicken","itemListElement":[
                     {"@type":"HowToStep","text":"Grill."}]}],
                 "recipeYield":["4","4 servings"],"prepTime":"PT1H5M","totalTime":"PT20M",
                 "author":{"@type":"Person","name":"Nagi"}}]}"#,
        );
        let found = recipes(&html);
        assert_eq!(found.len(), 1);
        let r = &found[0];
        assert_eq!(r.name, "Chicken & Rice");
        assert_eq!(
            r.ingredients,
            ["1/4 cup (31g) flour (spooned & leveled)", "2 eggs"]
        );
        assert_eq!(
            r.instructions,
            "Sauce\n1. Whisk.\n2. Chill.\nChicken\n1. Grill."
        );
        assert_eq!(r.servings, Some(4));
        assert_eq!(r.prep_minutes, Some(65), "prepTime, never totalTime");
        assert_eq!(r.author.as_deref(), Some("Nagi"));
    }

    #[test]
    fn plain_string_instructions_and_steps_are_numbered() {
        let html = page(
            r#"{"@type":"Recipe","name":"Toast","recipeIngredient":["1 slice bread"],
                "recipeInstructions":["Toast it.",{"@type":"HowToStep","text":"Butter it."}]}"#,
        );
        assert_eq!(
            recipes(&html)[0].instructions,
            "1. Toast it.\n2. Butter it."
        );
    }

    #[test]
    fn yields_that_are_not_servings_stay_empty() {
        for raw in [
            r#""6 x 20cm flatbreads""#,
            r#""1 cup""#,
            r#""0""#,
            r#""0 servings""#,
            "0",
            r#""99999999""#,
        ] {
            let html = page(&format!(
                r#"{{"@type":"Recipe","name":"X","recipeYield":{raw}}}"#
            ));
            assert_eq!(recipes(&html)[0].servings, None, "{raw}");
        }
        let html = page(r#"{"@type":"Recipe","name":"X","recipeYield":"6 Servings"}"#);
        assert_eq!(recipes(&html)[0].servings, Some(6));
    }

    #[test]
    fn values_the_domain_would_reject_are_dropped() {
        let html = page(
            r#"{"@type":"Recipe","name":"X","author":{"name":"  "},"prepTime":"PT0M",
                "recipeIngredient":["\t"]}"#,
        );
        let r = &recipes(&html)[0];
        assert_eq!(r.author, None);
        assert_eq!(r.prep_minutes, None);
        assert!(r.ingredients.is_empty());
        assert_eq!(iso_minutes("PT10000M"), Some(10_000));
        let html = page(r#"{"@type":"Recipe","name":"X","prepTime":"PT10001M"}"#);
        assert_eq!(recipes(&html)[0].prep_minutes, None);
    }

    #[test]
    fn a_recipe_without_a_name_is_not_a_recipe() {
        let html = page(r#"{"@type":"Recipe","name":"   ","recipeIngredient":["1 egg"]}"#);
        assert!(recipes(&html).is_empty());
    }

    #[test]
    fn malformed_json_beside_valid_json_is_skipped() {
        let html = format!(
            "{}{}",
            page(r#"{"@type":"Recipe","name":"Broken""#),
            page(r#"{"@type":"Recipe","name":"Fine"}"#)
        );
        let found = recipes(&html);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Fine");
    }

    #[test]
    fn several_recipes_are_all_returned() {
        let html = page(r#"[{"@type":"Recipe","name":"A"},{"@type":"Recipe","name":"B"}]"#);
        let names: Vec<_> = recipes(&html).into_iter().map(|r| r.name).collect();
        assert_eq!(names, ["A", "B"]);
    }

    #[test]
    fn a_hostile_script_payload_stays_inert_text() {
        let html = page(
            r#"{"@type":"Recipe","name":"<img src=x onerror=alert(1)>","recipeIngredient":["</script><script>alert(1)</script>"]}"#,
        );
        // The HTML parser ends the script at the first `</script>`, so the JSON is cut short
        // and skipped: the payload is never parsed as a recipe, let alone run.
        assert!(recipes(&html).is_empty());
        let markup =
            page(r#"{"@type":"Recipe","name":"<b>Bold</b> &lt;3 <img src=x onerror=alert(1)>"}"#);
        assert_eq!(recipes(&markup)[0].name, "Bold <3");
    }

    #[test]
    fn site_name_prefers_og_then_publisher() {
        let og = r#"<meta property="og:site_name" content="RecipeTin Eats">"#;
        assert_eq!(site_name(og).as_deref(), Some("RecipeTin Eats"));
        let html = page(r#"{"@graph":[{"@type":"Recipe","name":"X","publisher":{"name":"Pub"}}]}"#);
        assert_eq!(site_name(&html).as_deref(), Some("Pub"));
        assert_eq!(site_name("<p>none</p>"), None);
    }

    #[test]
    fn card_links_carry_their_ingredient_line() {
        let html = r#"<div class="wprm-recipe-container"><ul>
            <li class="wprm-recipe-ingredient">▢ 1 batch <a href="/sauce">Lemon yogurt sauce</a></li>
            </ul><div class="wprm-recipe-notes">See <a href="/other">other</a></div></div>
            <div class="tasty-recipes"><div class="tasty-recipes-ingredients"><ul>
            <li>1 <a href="/crust">pie crust</a></li></ul></div></div>
            <p><a href="/outside">not in a card</a></p>"#;
        let found = locate(html);
        assert_eq!(found.len(), 3);
        assert_eq!(found[0].context, "▢ 1 batch Lemon yogurt sauce");
        assert!(found[0].ingredient);
        assert_eq!(found[0].href.as_deref(), Some("/sauce"));
        assert!(!found[1].ingredient);
        assert!(found[1].classes.contains("wprm-recipe-notes"));
        assert_eq!(found[2].context, "1 pie crust");
        assert!(found[2].ingredient);
    }

    #[test]
    fn a_challenge_page_is_recognised() {
        assert!(looks_like_challenge("<div id=cf-chl-widget></div>"));
        assert!(looks_like_challenge("Please complete the CAPTCHA"));
        assert!(!looks_like_challenge("<p>About us</p>"));
    }
}
