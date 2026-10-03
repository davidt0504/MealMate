//! Decides what each recipe-card link means and assembles the review draft (OPT-001 design
//! §4). A pure port of the research classifier measured on 2026-10-02 (167 real anchors, 0
//! false expansions): same rules, same order, same regexes. Deliberate differences are
//! marked. No I/O happens here; child pages arrive already fetched.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;
use url::Url;

use crate::extract::{decode_entities, Anchor, ParsedRecipe};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    RequiredComponent,
    OptionalAlternative,
    Unrelated,
    Unknown,
}

/// One link's classification. `scale` is set only for an explicit `N batch`/`N recipe`
/// amount at depth 0 that the parent does not modify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub href: Option<String>,
    pub text: String,
    pub label: Label,
    pub scale: Option<(u32, u32)>,
    pub target: Option<String>,
    pub reason: &'static str,
    pub ingredient_index: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    Expanded,
    Unresolved,
    Alternative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftComponent {
    pub position: u32,
    pub title: String,
    pub source_url: Option<String>,
    pub scale: Option<(u32, u32)>,
    pub replaced_text: String,
    pub kind: ComponentKind,
    pub links: Vec<String>,
    pub instructions: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftLine {
    pub text: String,
    pub component: Option<u32>,
}

/// What the review opens with. Every parent line is either kept or replaced in place by its
/// child's lines; none is ever dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assembly {
    pub lines: Vec<DraftLine>,
    pub components: Vec<DraftComponent>,
    pub notice: Option<String>,
    /// Lines that name a sub-recipe whose ingredients are already listed (review hint only).
    pub inline_lines: Vec<u32>,
}

/// At most this many child pages per import (design §3).
pub const MAX_CHILDREN: usize = 3;

/// A reduced `numer/denom` batch amount.
type Scale = (u32, u32);

pub(crate) fn norm(t: &str) -> String {
    let lowered = decode_entities(t).to_lowercase();
    let joined = lowered.split_whitespace().collect::<Vec<_>>().join(" ");
    joined.trim_start_matches(['▢', ' ']).to_owned()
}

static NON_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\W").unwrap());

/// Spacing- and punctuation-insensitive line identity.
pub(crate) fn key(t: &str) -> String {
    NON_WORD.replace_all(&norm(t), "").into_owned()
}

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex")
}

static ALT: LazyLock<Regex> =
    LazyLock::new(|| re(r"\bor\b|\boptional\b|store[- ]bought|purchased|if you got the time"));
static MODIFIED: LazyLock<Regex> = LazyLock::new(|| {
    re(r"\b(made with|half the|less|reduced?|extra|without|omit|double|halved?)\b")
});
static PURCHASE: LazyLock<Regex> =
    LazyLock::new(|| re(r"store[- ]bought|purchased|bottled|jarred|ready[- ]made"));
static CREDIT: LazyLock<Regex> =
    LazyLock::new(|| re(r"recipe source:|based on|recommended:|also see|use this for:"));
static NAVIGATION: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"\b(print|save recipe|reviews|newsletter|follow along|instagram|nutrition|calculate recipe costs)\b",
    )
});
static OUTSIDE_LINES: LazyLock<Regex> = LazyLock::new(|| {
    re(r"wprm-recipe-notes|wprm-recipe-summary|tasty-recipes-notes|tasty-recipes-description")
});
// Deliberate difference: `1 recipe` counts like `1 batch` (design §4).
static BATCH: LazyLock<Regex> =
    LazyLock::new(|| re(r"^(\d+(?:/\d+)?)\s+(?:batch(?:es)?|recipes?)\b"));
static PARENS: LazyLock<Regex> = LazyLock::new(|| re(r"\(([^)]*)\)"));

/// Deliberate difference: the registrable domain from the Public Suffix List, not the
/// research script's last-two-labels shortcut that breaks on `co.uk`.
fn site(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    Some(psl::domain_str(host).unwrap_or(host).to_owned())
}

/// Whether two URLs share a registrable domain.
pub(crate) fn same_site(a: &Url, b: &Url) -> bool {
    site(a).is_some() && site(a) == site(b)
}

/// `"1/2"` → `(1, 2)` reduced; zero or overflow is no batch amount at all.
fn batch_scale(raw: &str) -> Option<(u32, u32)> {
    let (n, d) = match raw.split_once('/') {
        Some((n, d)) => (n.parse::<u32>().ok()?, d.parse::<u32>().ok()?),
        None => (raw.parse::<u32>().ok()?, 1),
    };
    if n == 0 || d == 0 {
        return None;
    }
    let g = gcd(n, d);
    Some((n / g, d / g))
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Labels every anchor. `recipe_count` other than 1 and `depth` above 0 withhold expansion.
pub fn classify(
    anchors: &[Anchor],
    ingredients: &[String],
    base: &Url,
    recipe_count: usize,
    depth: u32,
) -> Vec<Decision> {
    // Later duplicates win, as the research dict comprehension did.
    let lines: HashMap<String, usize> = ingredients
        .iter()
        .enumerate()
        .map(|(i, l)| (key(l), i))
        .collect();
    let base_site = site(base);
    let mut out: Vec<Decision> = anchors
        .iter()
        .map(|a| {
            let c = norm(&a.context);
            let mut label = Label::Unknown;
            let mut scale = None;
            let mut reason = "insufficient association";
            let idx = if a.ingredient { lines.get(&key(&a.context)).copied() } else { None };
            let target = a.href.as_deref().and_then(|h| base.join(h).ok());
            let text = norm(&a.text);
            let rest = if a.text.is_empty() { c.clone() } else { c.replacen(&text, " ", 1) };
            if CREDIT.is_match(&c) {
                label = Label::Unrelated;
                reason = "attribution or recommendation";
            } else if a.href.is_none() {
                reason = "missing target";
            } else if a.href.as_deref().is_some_and(|h| {
                h.starts_with('#') || h.starts_with("mailto:") || h.starts_with("javascript:")
            }) || NAVIGATION.is_match(&c)
            {
                label = Label::Unrelated;
                reason = "navigation or ancillary information";
            } else if !target
                .as_ref()
                .is_some_and(|t| matches!(t.scheme(), "http" | "https"))
            {
                label = Label::Unrelated;
                reason = "non-web target";
            } else if target.as_ref().and_then(site) != base_site {
                label = Label::Unrelated;
                reason = "cross-site link (product, credit or tool); not a component candidate";
            } else if recipe_count != 1 {
                reason = "multiple recipe records";
            } else if idx.is_some() {
                let paren = PARENS
                    .captures_iter(&c)
                    .map(|m| m[1].to_owned())
                    .collect::<Vec<_>>()
                    .join(" ");
                let outside = PARENS.replace_all(&c, "");
                let siblings: HashSet<&str> = anchors
                    .iter()
                    .filter(|x| x.ingredient && key(&x.context) == key(&a.context))
                    .filter_map(|x| x.href.as_deref())
                    .collect();
                if !a.text.is_empty() && paren.contains(&text) && !outside.contains(&text) {
                    reason = "link only inside a parenthetical remark";
                } else if siblings.len() > 1 && !PURCHASE.is_match(&c) {
                    label = Label::RequiredComponent;
                    reason = "choice between linked child recipes; user picks the child";
                } else if ALT.is_match(&rest) {
                    label = Label::OptionalAlternative;
                    reason = "explicit alternative in ingredient";
                } else {
                    label = Label::RequiredComponent;
                    reason = "exact structured ingredient association";
                    if MODIFIED.is_match(&rest) {
                        reason = "parent modifies component amounts; expansion withheld";
                    } else if depth == 0 {
                        scale = BATCH.captures(&c).and_then(|m| batch_scale(&m[1]));
                    }
                    if depth > 0 {
                        reason = "exact structured ingredient association; depth cap withholds expansion";
                    }
                }
            } else if ALT.is_match(&c) {
                label = Label::OptionalAlternative;
                reason = "alternative wording, no exact default ingredient association";
            } else if OUTSIDE_LINES.is_match(&a.classes) {
                label = Label::Unrelated;
                reason = "context outside default ingredients/instructions";
            }
            Decision {
                href: a.href.clone(),
                text: a.text.clone(),
                label,
                scale,
                target: target.map(String::from),
                reason,
                ingredient_index: idx,
            }
        })
        .collect();
    // Duplicate references inherit an explicit default batch association, but never a
    // source-credit link (those are `Unrelated` and stay so).
    let mut required: HashMap<String, (Option<Scale>, Option<usize>)> = HashMap::new();
    for d in &out {
        if d.label == Label::RequiredComponent && d.scale.is_some() {
            if let Some(t) = &d.target {
                required
                    .entry(t.clone())
                    .or_insert((d.scale, d.ingredient_index));
            }
        }
    }
    for d in &mut out {
        if matches!(d.label, Label::OptionalAlternative | Label::Unknown) {
            if let Some(&(scale, idx)) = d.target.as_ref().and_then(|t| required.get(t)) {
                d.label = Label::RequiredComponent;
                d.scale = scale;
                d.ingredient_index = idx;
                d.reason = "same target as explicit default batch ingredient";
            }
        }
    }
    out
}

/// The child pages worth fetching: targets with at least one 1x use, in document order,
/// fetched once however many lines use them, at most `MAX_CHILDREN`. Deliberate difference:
/// the research fetched every scaled target (½ included) sorted by URL.
pub fn child_targets(decisions: &[Decision]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for d in decisions {
        if d.label == Label::RequiredComponent && d.scale == Some((1, 1)) {
            if let Some(t) = &d.target {
                if !out.contains(t) && out.len() < MAX_CHILDREN {
                    out.push(t.clone());
                }
            }
        }
    }
    out
}

/// Builds the draft. Never fails for an unresolved component and never drops a line.
pub fn assemble(
    parent: &ParsedRecipe,
    children: &HashMap<String, ParsedRecipe>,
    decisions: &[Decision],
    fetched: &[String],
    depth: u32,
) -> Assembly {
    // Per line, each target's first decision, in first-seen order.
    let mut by_idx: HashMap<usize, Vec<&Decision>> = HashMap::new();
    for d in decisions {
        let Some(idx) = d.ingredient_index else {
            continue;
        };
        if !matches!(
            d.label,
            Label::RequiredComponent | Label::OptionalAlternative
        ) {
            continue;
        }
        let uses = by_idx.entry(idx).or_default();
        if !uses.iter().any(|u| u.target == d.target) {
            uses.push(d);
        }
    }
    let parent_keys: HashSet<String> = parent.ingredients.iter().map(|l| key(l)).collect();
    let mut out = Assembly {
        lines: Vec::new(),
        components: Vec::new(),
        notice: None,
        inline_lines: Vec::new(),
    };
    let mut unresolved = 0;
    for (idx, line) in parent.ingredients.iter().enumerate() {
        let uses = by_idx.get(&idx).map(Vec::as_slice).unwrap_or_default();
        let required: Vec<&Decision> = uses
            .iter()
            .copied()
            .filter(|d| d.label == Label::RequiredComponent)
            .collect();
        let position = out.components.len() as u32;
        let title = |d: &Decision| {
            if d.text.trim().is_empty() {
                line.clone()
            } else {
                d.text.clone()
            }
        };
        if required.is_empty() {
            let links: Vec<String> = uses.iter().filter_map(|d| d.target.clone()).collect();
            if links.is_empty() {
                out.lines.push(DraftLine {
                    text: line.clone(),
                    component: None,
                });
            } else {
                out.components.push(DraftComponent {
                    position,
                    title: title(uses[0]),
                    source_url: None,
                    scale: None,
                    replaced_text: line.clone(),
                    kind: ComponentKind::Alternative,
                    links,
                    instructions: String::new(),
                });
                out.lines.push(DraftLine {
                    text: line.clone(),
                    component: Some(position),
                });
            }
            continue;
        }
        let d = required[0];
        let target = d.target.clone().unwrap_or_default();
        let child = (required.len() == 1)
            .then(|| children.get(&target))
            .flatten()
            .filter(|c| !c.ingredients.is_empty() && !c.instructions.trim().is_empty());
        let why = if required.len() > 1 {
            Some("choice between child recipes")
        } else if depth > 0 {
            Some("deeper dependency")
        } else if d.scale.is_none() {
            Some(if d.reason.contains("withheld") {
                d.reason
            } else {
                "component amount not stated as batches"
            })
        } else if d.scale != Some((1, 1)) {
            Some("raw-line scaling unsupported")
        } else if !fetched.contains(&target) {
            Some("too many components")
        } else if child.is_none() {
            Some("child recipe missing or incomplete")
        } else {
            None
        };
        if why.is_none() {
            let c = child.expect("checked above");
            if c.ingredients.iter().all(|l| parent_keys.contains(&key(l))) {
                // Already listed inline (ThermoWorks' Hollandaise): never expand twice.
                out.inline_lines.push(out.lines.len() as u32);
                out.lines.push(DraftLine {
                    text: line.clone(),
                    component: None,
                });
                continue;
            }
            out.components.push(DraftComponent {
                position,
                title: c.name.clone(),
                source_url: Some(target),
                scale: d.scale,
                replaced_text: line.clone(),
                kind: ComponentKind::Expanded,
                links: Vec::new(),
                instructions: c.instructions.clone(),
            });
            out.lines.extend(c.ingredients.iter().map(|l| DraftLine {
                text: l.clone(),
                component: Some(position),
            }));
            continue;
        }
        unresolved += 1;
        out.components.push(DraftComponent {
            position,
            title: title(d),
            source_url: None,
            scale: None,
            replaced_text: line.clone(),
            kind: ComponentKind::Unresolved,
            links: required.iter().filter_map(|x| x.target.clone()).collect(),
            instructions: String::new(),
        });
        out.lines.push(DraftLine {
            text: line.clone(),
            component: Some(position),
        });
    }
    if unresolved > 0 {
        out.notice = Some(format!(
            "{unresolved} part(s) of this recipe couldn't be added automatically. Their \
             original lines and links are kept — edit them or save as is."
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::locate;

    const ROOT: &str = "https://example.com/root";

    fn fixture(name: &str) -> &'static str {
        match name {
            "changed_amounts" => include_str!("../tests/fixtures/changed_amounts.html"),
            "conflicting_optional" => include_str!("../tests/fixtures/conflicting_optional.html"),
            "duplicate" => include_str!("../tests/fixtures/duplicate.html"),
            "half_batch" => include_str!("../tests/fixtures/half_batch.html"),
            "incomplete_child" => include_str!("../tests/fixtures/incomplete_child.html"),
            "inline_child" => include_str!("../tests/fixtures/inline_child.html"),
            "measured_missing_yield" => {
                include_str!("../tests/fixtures/measured_missing_yield.html")
            }
            "misleading_placement" => include_str!("../tests/fixtures/misleading_placement.html"),
            "missing_href" => include_str!("../tests/fixtures/missing_href.html"),
            "multiple_recipes" => include_str!("../tests/fixtures/multiple_recipes.html"),
            "nested" => include_str!("../tests/fixtures/nested.html"),
            "note_only_alternative" => {
                include_str!("../tests/fixtures/note_only_alternative.html")
            }
            "required_homemade" => include_str!("../tests/fixtures/required_homemade.html"),
            "same_name_unrelated" => include_str!("../tests/fixtures/same_name_unrelated.html"),
            "two_uses" => include_str!("../tests/fixtures/two_uses.html"),
            "shawarma_like" => include_str!("../tests/fixtures/shawarma_like.html"),
            "choice_of_two" => include_str!("../tests/fixtures/choice_of_two.html"),
            "inline_hollandaise" => include_str!("../tests/fixtures/inline_hollandaise.html"),
            "optional_linked" => include_str!("../tests/fixtures/optional_linked.html"),
            "cross_site_and_credits" => {
                include_str!("../tests/fixtures/cross_site_and_credits.html")
            }
            "four_children" => include_str!("../tests/fixtures/four_children.html"),
            other => panic!("no fixture {other}"),
        }
    }

    fn root() -> Url {
        Url::parse(ROOT).unwrap()
    }

    fn lines(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| (*s).to_owned()).collect()
    }

    fn decide(name: &str, ingredients: &[&str], recipe_count: usize, depth: u32) -> Vec<Decision> {
        classify(
            &locate(fixture(name)),
            &lines(ingredients),
            &root(),
            recipe_count,
            depth,
        )
    }

    fn recipe(name: &str, ingredients: &[&str], instructions: &str) -> ParsedRecipe {
        ParsedRecipe {
            name: name.to_owned(),
            ingredients: lines(ingredients),
            instructions: instructions.to_owned(),
            servings: None,
            prep_minutes: None,
            author: None,
        }
    }

    fn green() -> ParsedRecipe {
        recipe(
            "Green dressing",
            &["1 cup yogurt", "1 tbsp lemon juice", "1 pinch salt"],
            "Whisk everything.",
        )
    }

    fn texts(a: &Assembly) -> Vec<&str> {
        a.lines.iter().map(|l| l.text.as_str()).collect()
    }

    /// No parent line is lost or duplicated: each one appears exactly as often as in the
    /// parent, counting a line an expanded component replaced as present.
    fn assert_lines_conserved(parent: &ParsedRecipe, a: &Assembly) {
        assert!(!parent.ingredients.is_empty());
        let expanded = |c: Option<u32>| {
            c.is_some_and(|p| a.components[p as usize].kind == ComponentKind::Expanded)
        };
        for line in &parent.ingredients {
            let want = parent.ingredients.iter().filter(|l| *l == line).count();
            let kept = a
                .lines
                .iter()
                .filter(|l| &l.text == line && !expanded(l.component))
                .count();
            let replaced = a
                .components
                .iter()
                .filter(|c| c.kind == ComponentKind::Expanded && &c.replaced_text == line)
                .count();
            assert_eq!(kept + replaced, want, "{line}");
        }
    }

    // --- The frozen research expectations, one case per fixture --------------------------------

    #[test]
    fn the_frozen_synthetic_labels_hold() {
        use Label::*;
        // (fixture, ingredients, recipe count, depth, expected label and scale per anchor)
        type Case<'a> = (
            &'a str,
            &'a [&'a str],
            usize,
            u32,
            &'a [(Label, Option<Scale>)],
        );
        let cases: &[Case] = &[
            (
                "half_batch",
                &["1/2 batch Green dressing"],
                1,
                0,
                &[(RequiredComponent, Some((1, 2)))],
            ),
            (
                "measured_missing_yield",
                &["2 cups Green dressing"],
                1,
                0,
                &[(RequiredComponent, None)],
            ),
            (
                "nested",
                &["1 batch Green dressing"],
                1,
                1,
                &[(RequiredComponent, None)],
            ),
            (
                "duplicate",
                &["1 batch Green dressing recipe"],
                1,
                0,
                &[(RequiredComponent, Some((1, 1)))],
            ),
            (
                "same_name_unrelated",
                &["1 batch Green dressing"],
                1,
                0,
                &[(Unrelated, None)],
            ),
            (
                "multiple_recipes",
                &["1 batch Green dressing"],
                2,
                0,
                &[(Unknown, None)],
            ),
            (
                "missing_href",
                &["1 batch Green dressing"],
                1,
                0,
                &[(Unknown, None)],
            ),
            (
                "misleading_placement",
                &["2 eggs"],
                1,
                0,
                &[(Unrelated, None)],
            ),
            (
                "conflicting_optional",
                &["1 batch Green dressing or bottled dressing"],
                1,
                0,
                &[(OptionalAlternative, None)],
            ),
            (
                "required_homemade",
                &["1 batch homemade Green dressing"],
                1,
                0,
                &[(RequiredComponent, Some((1, 1)))],
            ),
            (
                "note_only_alternative",
                &["1 batch Green dressing"],
                1,
                0,
                &[(RequiredComponent, Some((1, 1)))],
            ),
            (
                "inline_child",
                &[
                    "1 batch Green dressing",
                    "1 cup yogurt",
                    "1 tbsp lemon juice",
                    "1 pinch salt",
                ],
                1,
                0,
                &[(RequiredComponent, Some((1, 1)))],
            ),
            (
                "two_uses",
                &[
                    "1 batch Green dressing",
                    "2 eggs",
                    "1/2 batch Green dressing",
                ],
                1,
                0,
                &[
                    (RequiredComponent, Some((1, 1))),
                    (RequiredComponent, Some((1, 2))),
                ],
            ),
            (
                "changed_amounts",
                &["1 batch Green dressing (made with half the yogurt)"],
                1,
                0,
                &[(RequiredComponent, None)],
            ),
            (
                "incomplete_child",
                &["1 batch Green dressing", "2 eggs"],
                1,
                0,
                &[(RequiredComponent, Some((1, 1)))],
            ),
        ];
        assert_eq!(cases.len(), 15);
        for (name, ingredients, count, depth, expected) in cases {
            let ds = decide(name, ingredients, *count, *depth);
            assert!(ds.len() >= expected.len(), "{name}: {ds:#?}");
            for (n, (label, scale)) in expected.iter().enumerate() {
                assert_eq!(
                    (ds[n].label, ds[n].scale),
                    (*label, *scale),
                    "{name}#{n}: {}",
                    ds[n].reason
                );
            }
        }
    }

    // --- Assembly ------------------------------------------------------------------------------

    #[test]
    fn a_1x_component_expands_in_place_keeping_order_and_wording() {
        let parent = recipe(
            "Chicken Shawarma",
            &[
                "1 kg chicken thighs",
                "2 garlic cloves",
                "1 batch Lemon yogurt sauce",
                "4 flatbreads",
            ],
            "1. Marinate.",
        );
        let sauce = recipe(
            "Lemon yogurt sauce",
            &["1 cup Greek yogurt", "1 garlic clove", "1 tbsp lemon juice"],
            "1. Whisk.",
        );
        let ds = classify(
            &locate(fixture("shawarma_like")),
            &parent.ingredients,
            &root(),
            1,
            0,
        );
        let targets = child_targets(&ds);
        assert_eq!(targets, ["https://example.com/lemon-yogurt-sauce"]);
        let children = HashMap::from([(targets[0].clone(), sauce)]);
        let a = assemble(&parent, &children, &ds, &targets, 0);
        assert_eq!(
            texts(&a),
            [
                "1 kg chicken thighs",
                "2 garlic cloves",
                "1 cup Greek yogurt",
                "1 garlic clove",
                "1 tbsp lemon juice",
                "4 flatbreads"
            ]
        );
        assert_eq!(a.lines[2].component, Some(0));
        assert_eq!(
            a.lines[5].component, None,
            "the flatbread link stays a plain main line"
        );
        assert_eq!(a.components.len(), 1);
        let c = &a.components[0];
        assert_eq!((c.kind, c.scale), (ComponentKind::Expanded, Some((1, 1))));
        assert_eq!(c.title, "Lemon yogurt sauce");
        assert_eq!(c.replaced_text, "1 batch Lemon yogurt sauce");
        assert_eq!(c.instructions, "1. Whisk.");
        assert_eq!(a.notice, None);
        assert_lines_conserved(&parent, &a);
    }

    #[test]
    fn a_missing_child_keeps_the_line_with_one_notice() {
        let parent = recipe("P", &["1 batch Green dressing", "2 eggs"], "1. Cook.");
        let ds = decide(
            "incomplete_child",
            &["1 batch Green dressing", "2 eggs"],
            1,
            0,
        );
        let fetched = child_targets(&ds);
        let a = assemble(&parent, &HashMap::new(), &ds, &fetched, 0);
        assert_eq!(texts(&a), ["1 batch Green dressing", "2 eggs"]);
        assert_eq!(a.components[0].kind, ComponentKind::Unresolved);
        assert_eq!(a.components[0].links, ["https://example.com/green"]);
        assert_eq!(a.components[0].title, "Green dressing");
        assert!(a
            .notice
            .as_deref()
            .unwrap()
            .starts_with("1 part(s) of this recipe"));
        assert_lines_conserved(&parent, &a);
    }

    #[test]
    fn an_incomplete_child_stays_unresolved() {
        let parent = recipe("P", &["1 batch Green dressing", "2 eggs"], "1. Cook.");
        let ds = decide(
            "incomplete_child",
            &["1 batch Green dressing", "2 eggs"],
            1,
            0,
        );
        let fetched = child_targets(&ds);
        let no_steps = recipe("Green dressing", &["1 cup yogurt"], "  ");
        let a = assemble(
            &parent,
            &HashMap::from([(fetched[0].clone(), no_steps)]),
            &ds,
            &fetched,
            0,
        );
        assert_eq!(texts(&a), ["1 batch Green dressing", "2 eggs"]);
        assert_eq!(a.components[0].kind, ComponentKind::Unresolved);
    }

    #[test]
    fn an_inline_child_is_never_expanded_twice() {
        let ingredients = [
            "1 batch Green dressing",
            "1 cup yogurt",
            "1 tbsp lemon juice",
            "1 pinch salt",
        ];
        let parent = recipe("P", &ingredients, "1. Cook.");
        let ds = decide("inline_child", &ingredients, 1, 0);
        let fetched = child_targets(&ds);
        let a = assemble(
            &parent,
            &HashMap::from([(fetched[0].clone(), green())]),
            &ds,
            &fetched,
            0,
        );
        assert_eq!(texts(&a), ingredients);
        assert!(a.components.is_empty());
        assert_eq!(a.inline_lines, [0]);
        assert_eq!(a.notice, None);
    }

    #[test]
    fn two_uses_fetch_once_and_resolve_separately() {
        let ingredients = [
            "1 batch Green dressing",
            "2 eggs",
            "1/2 batch Green dressing",
        ];
        let parent = recipe("P", &ingredients, "1. Cook.");
        let ds = decide("two_uses", &ingredients, 1, 0);
        let fetched = child_targets(&ds);
        assert_eq!(fetched, ["https://example.com/green"]);
        let a = assemble(
            &parent,
            &HashMap::from([(fetched[0].clone(), green())]),
            &ds,
            &fetched,
            0,
        );
        assert_eq!(
            texts(&a),
            [
                "1 cup yogurt",
                "1 tbsp lemon juice",
                "1 pinch salt",
                "2 eggs",
                "1/2 batch Green dressing"
            ]
        );
        assert_eq!(a.components[0].kind, ComponentKind::Expanded);
        assert_eq!(a.components[1].kind, ComponentKind::Unresolved);
        assert_eq!(a.lines[4].component, Some(1));
        assert!(a.notice.is_some());
        assert_lines_conserved(&parent, &a);
    }

    #[test]
    fn half_and_measured_and_modified_amounts_stay_unresolved() {
        for (name, line) in [
            ("half_batch", "1/2 batch Green dressing"),
            ("measured_missing_yield", "2 cups Green dressing"),
            (
                "changed_amounts",
                "1 batch Green dressing (made with half the yogurt)",
            ),
        ] {
            let parent = recipe("P", &[line], "1. Cook.");
            let ds = decide(name, &[line], 1, 0);
            let fetched = child_targets(&ds);
            assert!(fetched.is_empty(), "{name} must not fetch");
            let a = assemble(&parent, &HashMap::new(), &ds, &fetched, 0);
            assert_eq!(texts(&a), [line], "{name}");
            assert_eq!(a.components[0].kind, ComponentKind::Unresolved, "{name}");
        }
    }

    #[test]
    fn a_choice_of_two_children_is_unresolved_and_fetches_neither() {
        let line = "1 Homemade Pie Crust or All Butter Pie Crust";
        let parent = recipe("Pie", &[line, "6 apples"], "1. Bake.");
        let ds = classify(
            &locate(fixture("choice_of_two")),
            &parent.ingredients,
            &root(),
            1,
            0,
        );
        let fetched = child_targets(&ds);
        assert!(fetched.is_empty());
        let a = assemble(&parent, &HashMap::new(), &ds, &fetched, 0);
        assert_eq!(texts(&a), [line, "6 apples"]);
        let mut links = a.components[0].links.clone();
        links.sort();
        assert_eq!(
            links,
            [
                "https://example.com/all-butter-crust",
                "https://example.com/pie-crust"
            ]
        );
        assert_eq!(a.components[0].kind, ComponentKind::Unresolved);
    }

    #[test]
    fn an_optional_linked_line_becomes_an_alternative_without_a_notice() {
        let line = "1 lb gnocchi, store-bought or homemade";
        let parent = recipe("Soup", &[line, "1 onion"], "1. Simmer.");
        let ds = classify(
            &locate(fixture("optional_linked")),
            &parent.ingredients,
            &root(),
            1,
            0,
        );
        assert!(child_targets(&ds).is_empty());
        let a = assemble(&parent, &HashMap::new(), &ds, &[], 0);
        assert_eq!(texts(&a), [line, "1 onion"]);
        assert_eq!(a.components[0].kind, ComponentKind::Alternative);
        assert_eq!(a.components[0].links, ["https://example.com/gnocchi"]);
        assert_eq!(a.notice, None);
    }

    #[test]
    fn credits_parentheticals_and_cross_site_links_are_never_components() {
        let ingredients = ["1 cup flour (see my flour guide)", "1 jar tahini", "2 eggs"];
        let ds = classify(
            &locate(fixture("cross_site_and_credits")),
            &lines(&ingredients),
            &root(),
            1,
            0,
        );
        assert!(!ds.is_empty());
        assert!(
            ds.iter().all(|d| d.label != Label::RequiredComponent),
            "{ds:#?}"
        );
        assert!(child_targets(&ds).is_empty());
        let reasons: Vec<_> = ds.iter().map(|d| d.reason).collect();
        assert!(
            reasons.contains(&"link only inside a parenthetical remark"),
            "{reasons:?}"
        );
        assert!(
            reasons.iter().any(|r| r.starts_with("cross-site")),
            "{reasons:?}"
        );
        assert!(
            reasons.contains(&"attribution or recommendation"),
            "{reasons:?}"
        );
    }

    #[test]
    fn a_fourth_child_is_left_unresolved() {
        let ingredients = [
            "1 batch A sauce",
            "1 batch B sauce",
            "1 batch C sauce",
            "1 batch D sauce",
        ];
        let parent = recipe("P", &ingredients, "1. Cook.");
        let ds = classify(
            &locate(fixture("four_children")),
            &parent.ingredients,
            &root(),
            1,
            0,
        );
        let fetched = child_targets(&ds);
        assert_eq!(fetched.len(), MAX_CHILDREN);
        assert_eq!(
            fetched[0], "https://example.com/a",
            "document order, not URL order"
        );
        let children: HashMap<String, ParsedRecipe> = fetched
            .iter()
            .map(|t| {
                (
                    t.clone(),
                    recipe("Sauce", &[&format!("1 cup {t}")], "1. Mix."),
                )
            })
            .collect();
        let a = assemble(&parent, &children, &ds, &fetched, 0);
        let kinds: Vec<_> = a.components.iter().map(|c| c.kind).collect();
        assert_eq!(kinds[..3], [ComponentKind::Expanded; 3]);
        assert_eq!(kinds[3], ComponentKind::Unresolved);
        assert_lines_conserved(&parent, &a);
    }

    #[test]
    fn depth_one_never_expands() {
        let parent = recipe("P", &["1 batch Green dressing"], "1. Cook.");
        let ds = decide("nested", &["1 batch Green dressing"], 1, 1);
        let a = assemble(
            &parent,
            &HashMap::from([("https://example.com/green".to_owned(), green())]),
            &ds,
            &["https://example.com/green".to_owned()],
            1,
        );
        assert_eq!(texts(&a), ["1 batch Green dressing"]);
        assert_eq!(a.components[0].kind, ComponentKind::Unresolved);
    }

    #[test]
    fn batch_amounts_parse_without_panicking() {
        assert_eq!(batch_scale("1"), Some((1, 1)));
        assert_eq!(batch_scale("2/4"), Some((1, 2)));
        assert_eq!(batch_scale("0"), None);
        assert_eq!(batch_scale("1/0"), None);
        assert_eq!(batch_scale("99999999999"), None);
        let caps = BATCH.captures("1 recipe pizza dough").unwrap();
        assert_eq!(&caps[1], "1");
    }

    #[test]
    fn line_keys_ignore_entities_spacing_and_the_checkbox() {
        assert_eq!(
            key("▢ 1/4 cup (31g) all-purpose flour (spooned &amp; leveled)"),
            key("1/4 cup ( 31g ) all-purpose flour ( spooned & leveled )")
        );
    }
}
