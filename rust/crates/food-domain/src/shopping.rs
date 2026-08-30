//! Shopping projection (MVP-015; PRD v3 §13, §16). A pure, clock-free derivation from a
//! snapshot of planned meals, recipes, identities and pantry marks: the same input always
//! yields the same list (invariants 17, 20). Nothing here guesses — a conversion happens only
//! inside an exact integer unit family, and every uncertain line stays separate with its
//! original text intact.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CivilDate, IngredientRef, MealComponent, MealSlot, PlannedMeal, PlannedMealId, Quantity,
    QuantityRange, Rational, Recipe, RecipeId, Unit, UnitKind,
};

/// Bump when a rule below changes what a given snapshot derives to (invariant 17).
pub const SHOPPING_ALGORITHM_VERSION: u32 = 1;

/// The exact integer conversion families. Cross-system pairs (cup↔ml, oz↔g) are deliberately
/// absent: 1 cup = 236.588 ml is not exact, and a guessed conversion is the card's stop
/// condition. Both `of` and `base_factor` match exhaustively, so a new `UnitKind` fails to
/// compile here rather than silently landing in no family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitFamily {
    VolumeUs,
    VolumeMetric,
    MassMetric,
    MassUs,
    Count,
}

impl UnitFamily {
    pub fn of(kind: UnitKind) -> Self {
        match kind {
            UnitKind::Teaspoon | UnitKind::Tablespoon | UnitKind::Cup | UnitKind::FluidOunce => {
                Self::VolumeUs
            }
            UnitKind::Milliliter | UnitKind::Liter => Self::VolumeMetric,
            UnitKind::Gram | UnitKind::Kilogram => Self::MassMetric,
            UnitKind::Ounce | UnitKind::Pound => Self::MassUs,
            UnitKind::Piece => Self::Count,
        }
    }

    pub fn base(self) -> UnitKind {
        match self {
            Self::VolumeUs => UnitKind::Teaspoon,
            Self::VolumeMetric => UnitKind::Milliliter,
            Self::MassMetric => UnitKind::Gram,
            Self::MassUs => UnitKind::Ounce,
            Self::Count => UnitKind::Piece,
        }
    }

    /// The token embedded in a merged line's `key`.
    pub fn key_token(self) -> &'static str {
        match self {
            Self::VolumeUs => "volume_us",
            Self::VolumeMetric => "volume_metric",
            Self::MassMetric => "mass_metric",
            Self::MassUs => "mass_us",
            Self::Count => "count",
        }
    }
}

/// How many of the family's base unit one of `kind` is; `1` for the base unit itself.
pub fn base_factor(kind: UnitKind) -> u32 {
    match kind {
        UnitKind::Teaspoon | UnitKind::Milliliter | UnitKind::Gram | UnitKind::Ounce => 1,
        UnitKind::Piece => 1,
        UnitKind::Tablespoon => 3,
        UnitKind::FluidOunce => 6,
        UnitKind::Cup => 48,
        UnitKind::Liter | UnitKind::Kilogram => 1000,
        UnitKind::Pound => 16,
    }
}

/// One recipe line × one component's scale: the provenance every line carries, so a shopper
/// can always answer "why is this here?" (AC-4). `original_text` is the line as entered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contribution {
    pub planned_meal_id: PlannedMealId,
    pub date: CivilDate,
    pub slot: MealSlot,
    pub component_position: usize,
    pub recipe_id: RecipeId,
    pub recipe_title: String,
    pub line_position: usize,
    pub original_text: String,
    /// The component's multiplier; `None` is 1×, as on `MealComponent::Recipe`.
    pub scale: Option<Rational>,
}

/// Why a line is not part of a larger one. On an `m:` line it is informational — it explains
/// why this identity has more than one line. `Unresolved` and `ArithmeticOverflow` are the
/// two genuinely per-contribution cases and carry `s:` keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparateReason {
    Unresolved,
    UnitNotCombinable,
    UnknownQuantity,
    ArithmeticOverflow,
}

/// `OmittedPantryMarked` means the household marked this identity as one it has. The line is
/// still returned in full so a consumer can restore it (pivot prompt: omission is never
/// silent and never final) — a mark suppresses a purchase but never proves quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    Needed,
    OmittedPantryMarked,
}

/// `key` is the stable identity MVP-016 keys its edits on: `m:…` for a merged group, `s:…`
/// for a per-contribution line. Keys embed no name or category, so a later canonicalisation
/// of either is non-breaking. Every caller-supplied segment — id, `Other` unit text, planned
/// meal id — is backslash-escaped (`\` as `\\`, `:` as `\:`), so a key splits into its fields
/// on its *unescaped* colons and no two groups can render the same string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingLine {
    pub key: String,
    pub name: String,
    pub ingredient: Option<IngredientRef>,
    pub quantity: Quantity,
    pub unit: Unit,
    pub optional: bool,
    pub status: LineStatus,
    pub separate_reason: Option<SeparateReason>,
    pub contributions: Vec<Contribution>,
}

/// Lines sharing one store category; `None` is "uncategorised" and sorts last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingGroup {
    pub category: Option<String>,
    pub lines: Vec<ShoppingLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingList {
    pub algorithm_version: u32,
    pub from: CivilDate,
    pub to: CivilDate,
    pub groups: Vec<ShoppingGroup>,
    /// Leftovers, dining out and the rest: counted so the caller can see they were seen, and
    /// contributing nothing.
    pub non_recipe_components: u32,
    /// Always equals the sum of every line's `contributions.len()`: nothing is dropped.
    pub contribution_count: u32,
}

/// What the derivation knows about a resolved identity: its canonical name (so a merged line
/// is not named by whichever recipe's spelling came first) and its store category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityInfo {
    pub name: String,
    pub store_category: Option<String>,
}

/// The whole input, read by storage in one transaction. Order of every `Vec` is irrelevant to
/// the output — `derive_shopping_list` sorts by explicit keys before emitting anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingInput {
    pub from: CivilDate,
    pub to: CivilDate,
    pub meals: Vec<PlannedMeal>,
    pub recipes: Vec<Recipe>,
    pub identities: Vec<(IngredientRef, IdentityInfo)>,
    pub pantry_marked: Vec<IngredientRef>,
}

/// One line × one component, before grouping. `scaled` is `None` when the scale multiplied the
/// quantity past `u32`; the group then falls back to per-contribution lines.
struct Raw {
    contribution: Contribution,
    name: String,
    ingredient: Option<IngredientRef>,
    original: Quantity,
    scaled: Option<Quantity>,
    unit: Unit,
    optional: bool,
}

impl Raw {
    /// The scaled quantity when it fits, else what was written: never a partial result.
    fn shown_quantity(&self) -> Quantity {
        self.scaled.unwrap_or(self.original)
    }

    fn separate_key(&self) -> String {
        format!(
            "s:{}:{}:{}",
            escape_segment(self.contribution.planned_meal_id.as_str()),
            self.contribution.component_position,
            self.contribution.line_position
        )
    }
}

/// Escapes the `:` separator inside one key segment, so a key's fields stay recoverable. Ids and
/// `Other` unit texts are caller-supplied verbatim — `id_newtype!` rejects only blank, and the
/// bridge does not validate the text — so without this two different groups could render one
/// string. One pass over the chars rather than two `replace` calls, which only work in one order:
/// escaping `:` before `\` would double-escape the backslashes it had just written.
fn escape_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch == '\\' || ch == ':' {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

fn ref_key(r: &IngredientRef) -> String {
    match r {
        IngredientRef::Catalog(id) => format!("catalog:{}", escape_segment(id.as_str())),
        IngredientRef::Custom(id) => format!("custom:{}", escape_segment(id.as_str())),
    }
}

/// The unit-class token: the exact family for a known unit, `none`, or the escaped text of an
/// `Other` unit (identical text is not a guess; anything else is). Escaping is injective, so it
/// leaves that byte-identical-text boundary exactly where it was.
fn unit_class(unit: &Unit) -> String {
    match unit {
        Unit::None => "none".to_owned(),
        Unit::Known(kind) => UnitFamily::of(*kind).key_token().to_owned(),
        Unit::Other(text) => format!("other={}", escape_segment(text)),
    }
}

fn scale_quantity(quantity: Quantity, scale: Option<Rational>) -> Option<Quantity> {
    match (quantity, scale) {
        (Quantity::Unknown, _) => Some(Quantity::Unknown),
        (quantity, None) => Some(quantity),
        (Quantity::Exact(q), Some(s)) => q.checked_mul(s).map(Quantity::Exact),
        (Quantity::Range(range), Some(s)) => {
            let min = range.min().checked_mul(s)?;
            let max = range.max().checked_mul(s)?;
            Some(Quantity::Range(
                QuantityRange::new(min, max).expect("scaling by a positive keeps min <= max"),
            ))
        }
    }
}

/// `Exact` + `Exact` stays `Exact`; anything with a `Range` is a `Range` (min+min, max+max).
/// `None` on overflow. Callers never pass `Unknown`.
fn add_known(a: Quantity, b: Quantity) -> Option<Quantity> {
    let bounds = |q: Quantity| match q {
        Quantity::Unknown => None,
        Quantity::Exact(r) => Some((r, r, true)),
        Quantity::Range(range) => Some((range.min(), range.max(), false)),
    };
    let (amin, amax, a_exact) = bounds(a)?;
    let (bmin, bmax, b_exact) = bounds(b)?;
    let min = amin.checked_add(bmin)?;
    let max = amax.checked_add(bmax)?;
    Some(if a_exact && b_exact {
        Quantity::Exact(min)
    } else {
        Quantity::Range(QuantityRange::new(min, max).expect("sums of ordered bounds stay ordered"))
    })
}

/// The unit a merged line renders in: the largest factor among its contributions, which is a
/// property of the set and so order-independent. `None`/`Other` classes have one unit.
fn render_unit(raws: &[Raw]) -> Unit {
    let mut best: Option<UnitKind> = None;
    for raw in raws {
        if let Unit::Known(kind) = raw.unit {
            best = Some(match best {
                Some(b) if base_factor(b) >= base_factor(kind) => b,
                _ => kind,
            });
        }
    }
    match best {
        Some(kind) => Unit::Known(kind),
        None => raws[0].unit.clone(),
    }
}

/// The group's summed quantity in `render`, or `None` if any step overflowed. Contributions
/// all of one kind add directly; mixed kinds go through the family base unit.
fn sum_group(raws: &[Raw], render: &Unit) -> Option<Quantity> {
    let render_kind = match render {
        Unit::Known(kind) => Some(*kind),
        _ => None,
    };
    let uniform = raws.iter().all(|raw| &raw.unit == render);
    let mut total: Option<Quantity> = None;
    for raw in raws {
        let mut q = raw.scaled?;
        if !uniform {
            if let (Unit::Known(kind), Some(_)) = (&raw.unit, render_kind) {
                let factor = Rational::new(base_factor(*kind), 1).expect("factors are >= 1");
                q = scale_quantity(q, Some(factor))?;
            }
        }
        total = Some(match total {
            None => q,
            Some(t) => add_known(t, q)?,
        });
    }
    let mut total = total?;
    if !uniform {
        if let Some(kind) = render_kind {
            let back = Rational::new(1, base_factor(kind)).expect("factors are >= 1");
            total = scale_quantity(total, Some(back))?;
        }
    }
    Some(total)
}

/// Everything a resolved contribution is grouped on: `(identity, optional, unit-class,
/// known|unknown)`. One group is emitted as one merged line whether it holds one contribution
/// or twenty.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GroupKey {
    ingredient: String,
    optional: bool,
    unit_class: String,
    known: bool,
}

impl GroupKey {
    fn line_key(&self) -> String {
        format!(
            "m:{}:{}:{}:{}",
            self.ingredient,
            self.unit_class,
            if self.optional { "opt" } else { "req" },
            if self.known { "known" } else { "unknown" }
        )
    }
}

struct Identities<'a> {
    info: BTreeMap<String, &'a IdentityInfo>,
    marked: BTreeSet<String>,
}

impl Identities<'_> {
    fn status(&self, ingredient: Option<&IngredientRef>) -> LineStatus {
        match ingredient {
            Some(r) if self.marked.contains(&ref_key(r)) => LineStatus::OmittedPantryMarked,
            _ => LineStatus::Needed,
        }
    }

    fn category(&self, ingredient: Option<&IngredientRef>) -> Option<String> {
        ingredient
            .and_then(|r| self.info.get(&ref_key(r)))
            .and_then(|i| i.store_category.clone())
    }
}

fn separate_line(raw: Raw, reason: SeparateReason, identities: &Identities<'_>) -> ShoppingLine {
    ShoppingLine {
        key: raw.separate_key(),
        name: raw.name.clone(),
        ingredient: raw.ingredient.clone(),
        quantity: raw.shown_quantity(),
        unit: raw.unit.clone(),
        optional: raw.optional,
        status: identities.status(raw.ingredient.as_ref()),
        separate_reason: Some(reason),
        contributions: vec![raw.contribution],
    }
}

/// Collects every contribution in `(date, slot, meal id, component, line)` order, so the
/// order inside each emitted line is fixed by the data, not by the caller's `Vec`.
fn collect_raws(input: &ShoppingInput) -> (Vec<Raw>, u32) {
    let mut meals: Vec<&PlannedMeal> = input.meals.iter().collect();
    meals.sort_by_key(|m| (m.date(), m.slot(), m.id().as_str().to_owned()));
    let mut recipes: BTreeMap<&str, &Recipe> = BTreeMap::new();
    for recipe in &input.recipes {
        recipes.entry(recipe.id().as_str()).or_insert(recipe);
    }
    let mut raws = Vec::new();
    let mut non_recipe = 0;
    for meal in meals {
        for (component_position, component) in meal.components().iter().enumerate() {
            let MealComponent::Recipe { recipe_id, scale } = component else {
                non_recipe += 1;
                continue;
            };
            let contribution =
                |title: &str, line_position: usize, original_text: &str| Contribution {
                    planned_meal_id: meal.id().clone(),
                    date: meal.date(),
                    slot: meal.slot(),
                    component_position,
                    recipe_id: recipe_id.clone(),
                    recipe_title: title.to_owned(),
                    line_position,
                    original_text: original_text.to_owned(),
                    scale: *scale,
                };
            let Some(recipe) = recipes.get(recipe_id.as_str()) else {
                // A snapshot inconsistency (storage's FK makes it unreachable from a real
                // read), reported rather than dropped.
                let id = recipe_id.as_str();
                raws.push(Raw {
                    contribution: contribution(id, 0, id),
                    name: id.to_owned(),
                    ingredient: None,
                    original: Quantity::Unknown,
                    scaled: Some(Quantity::Unknown),
                    unit: Unit::None,
                    optional: false,
                });
                continue;
            };
            for (line_position, line) in recipe.lines().iter().enumerate() {
                raws.push(Raw {
                    contribution: contribution(recipe.title(), line_position, line.original_text()),
                    name: line.name().to_owned(),
                    ingredient: line.ingredient().cloned(),
                    original: line.quantity(),
                    scaled: scale_quantity(line.quantity(), *scale),
                    unit: line.unit().clone(),
                    optional: line.optional(),
                });
            }
        }
    }
    (raws, non_recipe)
}

/// Pure and clock-free. Never errors: every input shape derives to a list in which
/// `contribution_count` equals the number of recipe-line × component pairs in range.
pub fn derive_shopping_list(input: &ShoppingInput) -> ShoppingList {
    let identities = Identities {
        info: input
            .identities
            .iter()
            .map(|(r, info)| (ref_key(r), info))
            .collect(),
        marked: input.pantry_marked.iter().map(ref_key).collect(),
    };
    let (raws, non_recipe_components) = collect_raws(input);
    let contribution_count = raws.len() as u32;

    let mut lines: Vec<ShoppingLine> = Vec::new();
    let mut groups: BTreeMap<GroupKey, Vec<Raw>> = BTreeMap::new();
    for raw in raws {
        match &raw.ingredient {
            None => {
                // An unresolved line whose scale overflowed shows its quantity as written,
                // exactly as the whole-group fallback does, so it carries the same reason
                // rather than reading as a correctly-scaled amount.
                let reason = match raw.scaled {
                    Some(_) => SeparateReason::Unresolved,
                    None => SeparateReason::ArithmeticOverflow,
                };
                lines.push(separate_line(raw, reason, &identities))
            }
            Some(r) => {
                let key = GroupKey {
                    ingredient: ref_key(r),
                    optional: raw.optional,
                    unit_class: unit_class(&raw.unit),
                    known: raw.original != Quantity::Unknown,
                };
                groups.entry(key).or_default().push(raw);
            }
        }
    }

    // Why an identity has more than one line, computed over the whole set of groups before any
    // is emitted. The class count is keyed on `known` as well, so a group is only ever told its
    // units did not combine by groups it could otherwise have combined with.
    let mut classes: BTreeMap<(&str, bool, bool), BTreeSet<&str>> = BTreeMap::new();
    let mut has_known: BTreeSet<(&str, bool)> = BTreeSet::new();
    for key in groups.keys() {
        classes
            .entry((&key.ingredient, key.optional, key.known))
            .or_default()
            .insert(&key.unit_class);
        if key.known {
            has_known.insert((&key.ingredient, key.optional));
        }
    }
    let reason_for = |key: &GroupKey| {
        let scope = (key.ingredient.as_str(), key.optional);
        // The unknown-quantity test comes first: a missing amount is a property of *this*
        // group, and it is the more actionable of the two facts. Every group inserted its own
        // class into its own bucket, so the index below always has an entry.
        if !key.known && has_known.contains(&scope) {
            Some(SeparateReason::UnknownQuantity)
        } else if classes[&(scope.0, scope.1, key.known)].len() > 1 {
            Some(SeparateReason::UnitNotCombinable)
        } else {
            None
        }
    };

    for (key, raws) in &groups {
        let unit = render_unit(raws);
        let quantity = if key.known {
            sum_group(raws, &unit)
        } else {
            Some(Quantity::Unknown)
        };
        let Some(quantity) = quantity else {
            // Whole group falls back, so no partial sum is ever shown and the result does
            // not depend on which contribution overflowed first.
            lines.extend(raws.iter().map(|raw| {
                separate_line(
                    Raw {
                        contribution: raw.contribution.clone(),
                        name: raw.name.clone(),
                        ingredient: raw.ingredient.clone(),
                        original: raw.original,
                        scaled: raw.scaled,
                        unit: raw.unit.clone(),
                        optional: raw.optional,
                    },
                    SeparateReason::ArithmeticOverflow,
                    &identities,
                )
            }));
            continue;
        };
        let ingredient = raws[0].ingredient.clone();
        let name = identities
            .info
            .get(&key.ingredient)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| raws[0].name.clone());
        lines.push(ShoppingLine {
            key: key.line_key(),
            name,
            ingredient: ingredient.clone(),
            quantity,
            unit,
            optional: key.optional,
            status: identities.status(ingredient.as_ref()),
            separate_reason: reason_for(key),
            contributions: raws.iter().map(|raw| raw.contribution.clone()).collect(),
        });
    }

    // `Option<String>` orders `None` first; the wrapper flips it so uncategorised is last.
    let mut by_category: BTreeMap<(bool, Option<String>), Vec<ShoppingLine>> = BTreeMap::new();
    for line in lines {
        let category = identities.category(line.ingredient.as_ref());
        by_category
            .entry((category.is_none(), category))
            .or_default()
            .push(line);
    }
    let groups = by_category
        .into_iter()
        .map(|((_, category), mut lines)| {
            lines.sort_by(|a, b| {
                (a.name.to_lowercase(), &a.key).cmp(&(b.name.to_lowercase(), &b.key))
            });
            ShoppingGroup { category, lines }
        })
        .collect();

    ShoppingList {
        algorithm_version: SHOPPING_ALGORITHM_VERSION,
        from: input.from,
        to: input.to,
        groups,
        non_recipe_components,
        contribution_count,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        derive_shopping_list, CivilDate, CustomIngredientId, IdentityInfo, IngredientId,
        IngredientLine, IngredientRef, LineStatus, MealComponent, MealSlot, PlannedMeal,
        PlannedMealId, Quantity, QuantityRange, Rational, Recipe, RecipeId, SeparateReason,
        ShoppingInput, ShoppingLine, ShoppingList, Unit, UnitFamily, UnitKind,
        SHOPPING_ALGORITHM_VERSION,
    };
    use crate::{parse_civil_date, ProvenanceKind, RecipeProvenance};
    use household_core::HouseholdId;

    fn r(numer: u32, denom: u32) -> Rational {
        Rational::new(numer, denom).unwrap()
    }

    fn exact(numer: u32, denom: u32) -> Quantity {
        Quantity::Exact(r(numer, denom))
    }

    fn range(min: (u32, u32), max: (u32, u32)) -> Quantity {
        Quantity::Range(QuantityRange::new(r(min.0, min.1), r(max.0, max.1)).unwrap())
    }

    fn cat(id: &str) -> IngredientRef {
        IngredientRef::Catalog(IngredientId::new(id).unwrap())
    }

    fn cus(id: &str) -> IngredientRef {
        IngredientRef::Custom(CustomIngredientId::new(id).unwrap())
    }

    fn known(kind: UnitKind) -> Unit {
        Unit::Known(kind)
    }

    fn line(
        name: &str,
        ingredient: Option<IngredientRef>,
        quantity: Quantity,
        unit: Unit,
        optional: bool,
    ) -> IngredientLine {
        IngredientLine::new(
            format!("<{name} as written>"),
            name,
            ingredient,
            quantity,
            unit,
            None,
            optional,
        )
        .unwrap()
    }

    fn recipe(id: &str, lines: Vec<IngredientLine>) -> Recipe {
        Recipe::new(
            RecipeId::new(id).unwrap(),
            HouseholdId::new("h").unwrap(),
            format!("Recipe {id}"),
            Some(4),
            None,
            "",
            lines,
            RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
        )
        .unwrap()
    }

    fn date(raw: &str) -> CivilDate {
        parse_civil_date(raw).unwrap()
    }

    fn meal(id: &str, day: &str, slot: MealSlot, components: Vec<MealComponent>) -> PlannedMeal {
        PlannedMeal::new(
            PlannedMealId::new(id).unwrap(),
            HouseholdId::new("h").unwrap(),
            date(day),
            slot,
            components,
            false,
        )
        .unwrap()
    }

    fn component(recipe: &str, scale: Option<Rational>) -> MealComponent {
        MealComponent::recipe(RecipeId::new(recipe).unwrap(), scale)
    }

    fn identity(id: &str, name: &str, category: Option<&str>) -> (IngredientRef, IdentityInfo) {
        (
            cat(id),
            IdentityInfo {
                name: name.to_owned(),
                store_category: category.map(str::to_owned),
            },
        )
    }

    fn input(
        meals: Vec<PlannedMeal>,
        recipes: Vec<Recipe>,
        identities: Vec<(IngredientRef, IdentityInfo)>,
        pantry_marked: Vec<IngredientRef>,
    ) -> ShoppingInput {
        ShoppingInput {
            from: date("2026-08-29"),
            to: date("2026-09-04"),
            meals,
            recipes,
            identities,
            pantry_marked,
        }
    }

    /// One dinner of recipe `r` at 1×, with the given lines and identities.
    fn single(
        lines: Vec<IngredientLine>,
        identities: Vec<(IngredientRef, IdentityInfo)>,
    ) -> ShoppingList {
        derive_shopping_list(&input(
            vec![meal(
                "pm-1",
                "2026-08-29",
                MealSlot::Dinner,
                vec![component("r", None)],
            )],
            vec![recipe("r", lines)],
            identities,
            vec![],
        ))
    }

    fn all_lines(list: &ShoppingList) -> Vec<&ShoppingLine> {
        list.groups.iter().flat_map(|g| g.lines.iter()).collect()
    }

    fn only_line(list: &ShoppingList) -> &ShoppingLine {
        let lines = all_lines(list);
        assert_eq!(lines.len(), 1, "expected one line, got {lines:#?}");
        lines[0]
    }

    fn contributions_total(list: &ShoppingList) -> usize {
        all_lines(list).iter().map(|l| l.contributions.len()).sum()
    }

    // --- AC-1: scaling ----------------------------------------------------------------------

    #[test]
    fn a_scaled_component_multiplies_every_known_quantity() {
        let lines = vec![
            line(
                "flour",
                Some(cat("flour")),
                exact(2, 1),
                known(UnitKind::Cup),
                false,
            ),
            line(
                "sugar",
                Some(cat("sugar")),
                range((1, 1), (2, 1)),
                known(UnitKind::Cup),
                false,
            ),
            line(
                "salt",
                Some(cat("salt")),
                Quantity::Unknown,
                Unit::None,
                false,
            ),
        ];
        let list = derive_shopping_list(&input(
            vec![meal(
                "pm-1",
                "2026-08-29",
                MealSlot::Dinner,
                vec![component("r", Some(r(3, 2)))],
            )],
            vec![recipe("r", lines)],
            vec![
                identity("flour", "flour", None),
                identity("sugar", "sugar", None),
                identity("salt", "salt", None),
            ],
            vec![],
        ));
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 3);
        let by_name = |name: &str| lines.iter().find(|l| l.name == name).unwrap();
        assert_eq!(by_name("flour").quantity, exact(3, 1));
        assert_eq!(by_name("sugar").quantity, range((3, 2), (3, 1)));
        assert_eq!(by_name("salt").quantity, Quantity::Unknown);
        for l in &lines {
            assert_eq!(l.contributions.len(), 1);
            assert_eq!(l.contributions[0].scale, Some(r(3, 2)));
        }
    }

    #[test]
    fn an_unscaled_component_is_one_times() {
        let list = single(
            vec![line(
                "flour",
                Some(cat("flour")),
                exact(2, 1),
                known(UnitKind::Cup),
                false,
            )],
            vec![identity("flour", "flour", None)],
        );
        let l = only_line(&list);
        assert_eq!(l.quantity, exact(2, 1));
        assert_eq!(l.contributions[0].scale, None);
    }

    // --- AC-3: unit families -------------------------------------------------------------------

    /// The factor table is the oracle. Hard-coded rather than read off `base_factor`, so a
    /// changed factor breaks here.
    fn expected_family(kind: UnitKind) -> (&'static str, u32) {
        match kind {
            UnitKind::Teaspoon => ("volume_us", 1),
            UnitKind::Tablespoon => ("volume_us", 3),
            UnitKind::FluidOunce => ("volume_us", 6),
            UnitKind::Cup => ("volume_us", 48),
            UnitKind::Milliliter => ("volume_metric", 1),
            UnitKind::Liter => ("volume_metric", 1000),
            UnitKind::Gram => ("mass_metric", 1),
            UnitKind::Kilogram => ("mass_metric", 1000),
            UnitKind::Ounce => ("mass_us", 1),
            UnitKind::Pound => ("mass_us", 16),
            UnitKind::Piece => ("count", 1),
        }
    }

    fn pair(a: UnitKind, b: UnitKind) -> ShoppingList {
        derive_shopping_list(&input(
            vec![
                meal(
                    "pm-1",
                    "2026-08-29",
                    MealSlot::Dinner,
                    vec![component("ra", None)],
                ),
                meal(
                    "pm-2",
                    "2026-08-30",
                    MealSlot::Dinner,
                    vec![component("rb", None)],
                ),
            ],
            vec![
                recipe(
                    "ra",
                    vec![line("x", Some(cat("x")), exact(1, 1), known(a), false)],
                ),
                recipe(
                    "rb",
                    vec![line("x", Some(cat("x")), exact(1, 1), known(b), false)],
                ),
            ],
            vec![identity("x", "x", None)],
            vec![],
        ))
    }

    #[test]
    fn every_same_family_pair_merges_and_every_cross_family_pair_stays_separate() {
        let mut same = 0;
        let mut cross = 0;
        for a in UnitKind::ALL {
            for b in UnitKind::ALL {
                let (fam_a, f_a) = expected_family(a);
                let (fam_b, f_b) = expected_family(b);
                assert_eq!(UnitFamily::of(a).key_token(), fam_a);
                let list = pair(a, b);
                let lines = all_lines(&list);
                if fam_a == fam_b {
                    same += 1;
                    assert_eq!(lines.len(), 1, "{a:?}+{b:?} must merge");
                    let l = lines[0];
                    assert!(l.key.starts_with("m:catalog:x:"), "{}", l.key);
                    assert_eq!(l.separate_reason, None);
                    assert_eq!(l.contributions.len(), 2);
                    let render = if f_a >= f_b { a } else { b };
                    assert_eq!(l.unit, known(render), "{a:?}+{b:?}");
                    assert_eq!(
                        l.quantity,
                        exact(f_a + f_b, f_a.max(f_b)),
                        "{a:?}+{b:?} value"
                    );
                } else {
                    cross += 1;
                    assert_eq!(lines.len(), 2, "{a:?}+{b:?} must stay apart");
                    for l in &lines {
                        assert!(l.key.starts_with("m:catalog:x:"), "{}", l.key);
                        assert_eq!(l.separate_reason, Some(SeparateReason::UnitNotCombinable));
                        assert_eq!(l.quantity, exact(1, 1));
                        assert_eq!(l.contributions.len(), 1);
                    }
                    assert_ne!(lines[0].key, lines[1].key);
                }
            }
        }
        // 4+2+2+2+1 kinds per family → 16+4+4+4+1 ordered same-family pairs, 121 total.
        assert_eq!(same, 29);
        assert_eq!(cross, 92);
    }

    #[test]
    fn merged_lines_render_in_the_largest_contributing_unit() {
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    exact(2, 1),
                    known(UnitKind::Tablespoon),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    known(UnitKind::Teaspoon),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    known(UnitKind::Cup),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        let l = only_line(&list);
        // 6 + 1 + 48 = 55 tsp = 55/48 cup.
        assert_eq!(l.unit, known(UnitKind::Cup));
        assert_eq!(l.quantity, exact(55, 48));
        assert_eq!(l.contributions.len(), 3);
    }

    #[test]
    fn same_kind_contributions_add_without_a_base_conversion() {
        // The largest representable value still merges when no conversion is needed.
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    exact(u32::MAX - 1, 1),
                    known(UnitKind::Cup),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    known(UnitKind::Cup),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        let l = only_line(&list);
        assert_eq!(l.quantity, exact(u32::MAX, 1));
        assert_eq!(l.unit, known(UnitKind::Cup));
    }

    #[test]
    fn none_merges_only_with_none_and_other_only_with_identical_text() {
        let list = single(
            vec![
                line("x", Some(cat("x")), exact(1, 1), Unit::None, false),
                line("x", Some(cat("x")), exact(2, 1), Unit::None, false),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    Unit::Other("clove".into()),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(3, 1),
                    Unit::Other("clove".into()),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    Unit::Other("Clove".into()),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    known(UnitKind::Piece),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 4, "{lines:#?}");
        let find = |unit: &Unit| lines.iter().find(|l| &l.unit == unit).unwrap();
        assert_eq!(find(&Unit::None).quantity, exact(3, 1));
        assert_eq!(find(&Unit::None).contributions.len(), 2);
        assert_eq!(find(&Unit::Other("clove".into())).quantity, exact(4, 1));
        assert_eq!(find(&Unit::Other("Clove".into())).quantity, exact(1, 1));
        assert_eq!(find(&known(UnitKind::Piece)).quantity, exact(1, 1));
        for l in &lines {
            assert_eq!(l.separate_reason, Some(SeparateReason::UnitNotCombinable));
        }
        assert_eq!(contributions_total(&list), 6);
    }

    #[test]
    fn a_line_with_other_unit_text_differing_only_by_case_stays_separate() {
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    Unit::Other("pinch".into()),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    Unit::Other("PINCH".into()),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        assert_eq!(all_lines(&list).len(), 2);
    }

    #[test]
    fn unknown_quantities_merge_into_one_unknown_line_and_never_with_known_ones() {
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    Quantity::Unknown,
                    known(UnitKind::Cup),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    Quantity::Unknown,
                    known(UnitKind::Cup),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    known(UnitKind::Cup),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        let unknown = lines
            .iter()
            .find(|l| l.quantity == Quantity::Unknown)
            .unwrap();
        let known_line = lines
            .iter()
            .find(|l| l.quantity != Quantity::Unknown)
            .unwrap();
        assert_eq!(unknown.contributions.len(), 2);
        assert_eq!(
            unknown.separate_reason,
            Some(SeparateReason::UnknownQuantity)
        );
        assert!(unknown.key.ends_with(":unknown"), "{}", unknown.key);
        assert_eq!(known_line.quantity, exact(1, 1));
        assert_eq!(known_line.separate_reason, None);
        assert!(known_line.key.ends_with(":known"), "{}", known_line.key);

        // Alone, an unknown group is not "separate" from anything.
        let alone = single(
            vec![
                line("x", Some(cat("x")), Quantity::Unknown, Unit::None, false),
                line("x", Some(cat("x")), Quantity::Unknown, Unit::None, false),
            ],
            vec![identity("x", "x", None)],
        );
        let l = only_line(&alone);
        assert_eq!(l.quantity, Quantity::Unknown);
        assert_eq!(l.separate_reason, None);
        assert_eq!(l.contributions.len(), 2);
    }

    /// The unit-class count is taken among groups of the same known-ness, so an unknown group
    /// is explained by its own missing amount rather than by a split it did not cause.
    #[test]
    fn an_unknown_group_beside_a_known_one_is_unknown_quantity_across_unit_classes() {
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    exact(2, 1),
                    known(UnitKind::Cup),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(500, 1),
                    known(UnitKind::Gram),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    Quantity::Unknown,
                    known(UnitKind::Cup),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 3);
        for l in &lines {
            let expected = if l.key.ends_with(":unknown") {
                Some(SeparateReason::UnknownQuantity)
            } else {
                Some(SeparateReason::UnitNotCombinable)
            };
            assert_eq!(l.separate_reason, expected, "{}", l.key);
        }
    }

    /// The mirror of the case above: the *known* group's only sibling is an unknown one in
    /// another unit class, so nothing prevented it from combining and it carries no reason.
    #[test]
    fn a_known_group_beside_a_differently_classed_unknown_group_is_not_unit_not_combinable() {
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    exact(2, 1),
                    known(UnitKind::Cup),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    Quantity::Unknown,
                    known(UnitKind::Gram),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        let known_line = lines.iter().find(|l| l.key.ends_with(":known")).unwrap();
        let unknown = lines.iter().find(|l| l.key.ends_with(":unknown")).unwrap();
        assert_eq!(known_line.separate_reason, None);
        assert_eq!(known_line.quantity, exact(2, 1));
        assert_eq!(
            unknown.separate_reason,
            Some(SeparateReason::UnknownQuantity)
        );
    }

    /// Expected-to-pass: two unknown groups with no known sibling really are kept apart by
    /// their units, so reordering the arms must not turn a genuine class split into `None`.
    #[test]
    fn an_unknown_group_with_no_known_sibling_keeps_unit_not_combinable() {
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    Quantity::Unknown,
                    known(UnitKind::Cup),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    Quantity::Unknown,
                    known(UnitKind::Gram),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        for l in &lines {
            assert_eq!(
                l.separate_reason,
                Some(SeparateReason::UnitNotCombinable),
                "{}",
                l.key
            );
        }
    }

    /// One dinner of recipe `r` at `scale`, with one unresolved line of `quantity` cups.
    fn unresolved_at_scale(quantity: Quantity, scale: Rational) -> ShoppingList {
        derive_shopping_list(&input(
            vec![meal(
                "pm-1",
                "2026-08-29",
                MealSlot::Dinner,
                vec![component("r", Some(scale))],
            )],
            vec![recipe(
                "r",
                vec![line("mystery", None, quantity, known(UnitKind::Cup), false)],
            )],
            vec![],
            vec![],
        ))
    }

    /// An unresolved line whose scale overflowed shows the quantity as *written*, so it must
    /// not read as a correctly-scaled one: it carries the same reason the whole-group
    /// fallback gives every other line that shows an unscaled amount.
    #[test]
    fn an_unresolved_line_whose_scale_overflows_is_tagged_arithmetic_overflow() {
        let list = unresolved_at_scale(exact(u32::MAX, 1), r(1000, 1));
        let l = only_line(&list);
        assert_eq!(l.key, "s:pm-1:0:0");
        assert_eq!(l.ingredient, None);
        assert_eq!(l.quantity, exact(u32::MAX, 1));
        assert_eq!(
            l.separate_reason,
            Some(SeparateReason::ArithmeticOverflow),
            "an unscaled quantity must never be presented as a scaled one"
        );
    }

    /// Expected-to-pass: the reason above must not over-fire on a scale that fits.
    #[test]
    fn an_unresolved_line_whose_scale_fits_is_still_unresolved() {
        let list = unresolved_at_scale(exact(1, 1), r(2, 1));
        let l = only_line(&list);
        assert_eq!(l.quantity, exact(2, 1));
        assert_eq!(l.separate_reason, Some(SeparateReason::Unresolved));
    }

    #[test]
    fn optional_and_required_lines_of_one_ingredient_stay_apart() {
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    known(UnitKind::Cup),
                    false,
                ),
                line("x", Some(cat("x")), exact(1, 1), known(UnitKind::Cup), true),
            ],
            vec![identity("x", "x", None)],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        let required = lines.iter().find(|l| !l.optional).unwrap();
        let optional = lines.iter().find(|l| l.optional).unwrap();
        assert!(required.key.contains(":req:"), "{}", required.key);
        assert!(optional.key.contains(":opt:"), "{}", optional.key);
        assert_eq!(required.separate_reason, None);
        assert_eq!(optional.separate_reason, None);
    }

    #[test]
    fn unresolved_lines_never_merge_and_keep_original_text() {
        let list = single(
            vec![
                line("onion", None, exact(1, 1), known(UnitKind::Piece), false),
                line("onion", None, exact(1, 1), known(UnitKind::Piece), false),
            ],
            vec![],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].key, "s:pm-1:0:0");
        assert_eq!(lines[1].key, "s:pm-1:0:1");
        for l in &lines {
            assert_eq!(l.separate_reason, Some(SeparateReason::Unresolved));
            assert_eq!(l.ingredient, None);
            assert_eq!(l.name, "onion");
            assert_eq!(l.status, LineStatus::Needed);
            assert_eq!(l.contributions.len(), 1);
            assert_eq!(l.contributions[0].original_text, "<onion as written>");
            assert_eq!(l.contributions[0].recipe_title, "Recipe r");
        }
        assert_eq!(list.groups.len(), 1);
        assert_eq!(list.groups[0].category, None);
    }

    /// Two meals whose `x` contributions share one group; `reversed` flips the input order of
    /// the meals (and recipes), which must not change which lines come out.
    fn overflow_input(reversed: bool) -> ShoppingInput {
        let mut meals = vec![
            meal(
                "pm-1",
                "2026-08-29",
                MealSlot::Dinner,
                vec![component("big", Some(r(1000, 1)))],
            ),
            meal(
                "pm-2",
                "2026-08-30",
                MealSlot::Dinner,
                vec![component("small", Some(r(1000, 1)))],
            ),
        ];
        let mut recipes = vec![
            recipe(
                "big",
                vec![line(
                    "x",
                    Some(cat("x")),
                    exact(u32::MAX, 1),
                    known(UnitKind::Cup),
                    false,
                )],
            ),
            recipe(
                "small",
                vec![line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    known(UnitKind::Cup),
                    false,
                )],
            ),
        ];
        if reversed {
            meals.reverse();
            recipes.reverse();
        }
        input(meals, recipes, vec![identity("x", "x", None)], vec![])
    }

    #[test]
    fn an_overflowing_group_falls_back_to_separate_lines_regardless_of_order() {
        let forward = derive_shopping_list(&overflow_input(false));
        let backward = derive_shopping_list(&overflow_input(true));
        assert_eq!(forward, backward);
        let lines = all_lines(&forward);
        assert_eq!(lines.len(), 2, "{lines:#?}");
        assert_eq!(lines[0].key, "s:pm-1:0:0");
        assert_eq!(lines[1].key, "s:pm-2:0:0");
        for l in &lines {
            assert_eq!(l.separate_reason, Some(SeparateReason::ArithmeticOverflow));
            assert_eq!(l.ingredient, Some(cat("x")));
            assert_eq!(l.contributions.len(), 1);
            assert_eq!(l.contributions[0].scale, Some(r(1000, 1)));
        }
        // The scalable contribution shows its scaled value; the other shows what was written,
        // never a partial sum.
        let quantities: Vec<Quantity> = lines.iter().map(|l| l.quantity).collect();
        assert!(quantities.contains(&exact(1000, 1)));
        assert!(quantities.contains(&exact(u32::MAX, 1)));
    }

    #[test]
    fn an_overflowing_sum_falls_back_even_when_every_scale_fits() {
        let list = single(
            vec![
                line(
                    "x",
                    Some(cat("x")),
                    exact(u32::MAX, 1),
                    known(UnitKind::Cup),
                    false,
                ),
                line(
                    "x",
                    Some(cat("x")),
                    exact(1, 1),
                    known(UnitKind::Cup),
                    false,
                ),
            ],
            vec![identity("x", "x", None)],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        for l in &lines {
            assert_eq!(l.separate_reason, Some(SeparateReason::ArithmeticOverflow));
        }
    }

    // --- AC-2: pantry -----------------------------------------------------------------------

    #[test]
    fn a_marked_identity_is_omitted_with_its_quantity_and_provenance_intact() {
        let list = derive_shopping_list(&input(
            vec![meal(
                "pm-1",
                "2026-08-29",
                MealSlot::Dinner,
                vec![component("r", Some(r(2, 1)))],
            )],
            vec![recipe(
                "r",
                vec![
                    line(
                        "rice",
                        Some(cat("rice")),
                        exact(1, 1),
                        known(UnitKind::Cup),
                        false,
                    ),
                    line(
                        "oil",
                        Some(cat("oil")),
                        exact(1, 1),
                        known(UnitKind::Tablespoon),
                        false,
                    ),
                ],
            )],
            vec![identity("rice", "rice", None), identity("oil", "oil", None)],
            vec![cat("rice")],
        ));
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        let rice = lines.iter().find(|l| l.name == "rice").unwrap();
        let oil = lines.iter().find(|l| l.name == "oil").unwrap();
        assert_eq!(rice.status, LineStatus::OmittedPantryMarked);
        assert_eq!(rice.quantity, exact(2, 1));
        assert_eq!(rice.contributions.len(), 1);
        assert_eq!(rice.contributions[0].original_text, "<rice as written>");
        assert_eq!(oil.status, LineStatus::Needed);
        assert_eq!(list.contribution_count, 2);
    }

    #[test]
    fn an_unmarked_identity_is_needed() {
        let list = single(
            vec![
                line("a", Some(cat("a")), exact(1, 1), Unit::None, false),
                line("b", Some(cat("b")), exact(1, 1), Unit::None, false),
            ],
            vec![identity("a", "a", None), identity("b", "b", None)],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| l.status == LineStatus::Needed));
    }

    #[test]
    fn an_unresolved_line_cannot_be_omitted() {
        let list = derive_shopping_list(&input(
            vec![meal(
                "pm-1",
                "2026-08-29",
                MealSlot::Dinner,
                vec![component("r", None)],
            )],
            vec![recipe(
                "r",
                vec![line("rice", None, exact(1, 1), known(UnitKind::Cup), false)],
            )],
            vec![identity("rice", "rice", None)],
            vec![cat("rice")],
        ));
        let l = only_line(&list);
        assert_eq!(l.status, LineStatus::Needed);
        assert_eq!(l.separate_reason, Some(SeparateReason::Unresolved));
    }

    #[test]
    fn a_marked_identity_on_an_overflow_fallback_line_is_still_omitted() {
        let mut snapshot = overflow_input(false);
        snapshot.pantry_marked = vec![cat("x")];
        let list = derive_shopping_list(&snapshot);
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        assert!(lines
            .iter()
            .all(|l| l.status == LineStatus::OmittedPantryMarked));
    }

    // --- AC-4: determinism and provenance ---------------------------------------------------

    fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
        if items.len() <= 1 {
            return vec![items.to_vec()];
        }
        let mut out = Vec::new();
        for i in 0..items.len() {
            let mut rest = items.to_vec();
            let head = rest.remove(i);
            for mut tail in permutations(&rest) {
                tail.insert(0, head.clone());
                out.push(tail);
            }
        }
        out
    }

    fn four_meal_fixture() -> ShoppingInput {
        input(
            vec![
                meal(
                    "pm-1",
                    "2026-08-29",
                    MealSlot::Dinner,
                    vec![component("ra", Some(r(3, 2))), component("rb", None)],
                ),
                meal(
                    "pm-2",
                    "2026-08-30",
                    MealSlot::Lunch,
                    vec![
                        MealComponent::Leftovers { note: None },
                        component("rb", Some(r(2, 1))),
                    ],
                ),
                meal(
                    "pm-3",
                    "2026-08-30",
                    MealSlot::Dinner,
                    vec![component("ra", None)],
                ),
                meal(
                    "pm-4",
                    "2026-08-31",
                    MealSlot::Dinner,
                    vec![MealComponent::DiningOut { note: None }],
                ),
            ],
            vec![
                recipe(
                    "ra",
                    vec![
                        line(
                            "Onions, diced",
                            Some(cat("onion")),
                            exact(2, 1),
                            known(UnitKind::Piece),
                            false,
                        ),
                        line(
                            "flour",
                            Some(cat("flour")),
                            exact(1, 1),
                            known(UnitKind::Cup),
                            false,
                        ),
                        line("mystery", None, Quantity::Unknown, Unit::None, false),
                        line(
                            "salt",
                            Some(cat("salt")),
                            Quantity::Unknown,
                            Unit::None,
                            false,
                        ),
                    ],
                ),
                recipe(
                    "rb",
                    vec![
                        line(
                            "onion",
                            Some(cat("onion")),
                            exact(1, 1),
                            known(UnitKind::Piece),
                            false,
                        ),
                        line(
                            "flour",
                            Some(cat("flour")),
                            exact(2, 1),
                            known(UnitKind::Tablespoon),
                            false,
                        ),
                        line(
                            "flour",
                            Some(cat("flour")),
                            exact(100, 1),
                            known(UnitKind::Gram),
                            true,
                        ),
                        line(
                            "salt",
                            Some(cat("salt")),
                            exact(1, 1),
                            known(UnitKind::Teaspoon),
                            false,
                        ),
                        line(
                            "butter",
                            Some(cat("butter")),
                            exact(1, 1),
                            known(UnitKind::Tablespoon),
                            false,
                        ),
                    ],
                ),
            ],
            vec![
                identity("onion", "onion", Some("produce")),
                identity("flour", "flour", Some("baking")),
                identity("salt", "salt", Some("baking")),
                identity("butter", "butter", Some("dairy")),
            ],
            vec![cat("butter")],
        )
    }

    #[test]
    fn every_permutation_of_meals_recipes_and_pantry_yields_the_same_list() {
        let base = four_meal_fixture();
        let expected = derive_shopping_list(&base);
        let meal_orders = permutations(&base.meals);
        assert_eq!(meal_orders.len(), 24);
        for meals in meal_orders {
            let mut snapshot = ShoppingInput {
                meals,
                ..base.clone()
            };
            assert_eq!(derive_shopping_list(&snapshot), expected);
            snapshot.recipes.reverse();
            snapshot.identities.reverse();
            snapshot.pantry_marked.reverse();
            assert_eq!(derive_shopping_list(&snapshot), expected);
        }
        // And the recipes' own line order, which changes contribution order but not the sum.
        assert_eq!(expected.algorithm_version, SHOPPING_ALGORITHM_VERSION);
    }

    #[test]
    fn every_contribution_is_accounted_for() {
        let list = derive_shopping_list(&four_meal_fixture());
        // ra ×2 components × 4 lines + rb ×2 components × 5 lines.
        assert_eq!(list.contribution_count, 18);
        assert_eq!(contributions_total(&list), 18);
        assert_eq!(list.non_recipe_components, 2);
        // Contributions are ordered by (date, slot, meal, component, line) within a line.
        for l in all_lines(&list) {
            let keys: Vec<_> = l
                .contributions
                .iter()
                .map(|c| {
                    (
                        c.date,
                        c.slot,
                        c.planned_meal_id.as_str().to_owned(),
                        c.component_position,
                        c.line_position,
                    )
                })
                .collect();
            let mut sorted = keys.clone();
            sorted.sort();
            assert_eq!(keys, sorted, "{}", l.key);
        }
    }

    #[test]
    fn groups_are_in_category_order_with_uncategorised_last() {
        let list = derive_shopping_list(&four_meal_fixture());
        let categories: Vec<Option<&str>> =
            list.groups.iter().map(|g| g.category.as_deref()).collect();
        assert_eq!(
            categories,
            vec![Some("baking"), Some("dairy"), Some("produce"), None]
        );
        for g in &list.groups {
            assert!(!g.lines.is_empty());
            let names: Vec<(String, &str)> = g
                .lines
                .iter()
                .map(|l| (l.name.to_lowercase(), l.key.as_str()))
                .collect();
            let mut sorted = names.clone();
            sorted.sort();
            assert_eq!(names, sorted);
        }
        // The uncategorised group holds exactly the unresolved `mystery` lines.
        let last = list.groups.last().unwrap();
        assert!(last.lines.iter().all(|l| l.name == "mystery"));
        assert_eq!(last.lines.len(), 2);
    }

    #[test]
    fn a_merged_line_is_named_by_its_identity_not_the_first_spelling() {
        let list = derive_shopping_list(&four_meal_fixture());
        let onion = all_lines(&list)
            .into_iter()
            .find(|l| l.ingredient == Some(cat("onion")))
            .unwrap();
        assert_eq!(onion.name, "onion");
        // 2×3/2 + 1 + 1×2 + 2 = 8 pieces across four contributions.
        assert_eq!(onion.quantity, exact(8, 1));
        assert_eq!(onion.contributions.len(), 4);
        assert_eq!(onion.key, "m:catalog:onion:count:req:known");
    }

    #[test]
    fn a_resolved_ref_with_no_identity_row_still_merges_and_is_uncategorised() {
        let list = single(
            vec![
                line("Ghost", Some(cat("ghost")), exact(1, 1), Unit::None, false),
                line(
                    "ghost pepper",
                    Some(cat("ghost")),
                    exact(1, 1),
                    Unit::None,
                    false,
                ),
            ],
            vec![],
        );
        let l = only_line(&list);
        assert_eq!(l.name, "Ghost");
        assert_eq!(l.quantity, exact(2, 1));
        assert_eq!(l.ingredient, Some(cat("ghost")));
        assert_eq!(list.groups[0].category, None);
    }

    #[test]
    fn non_recipe_components_are_counted_and_contribute_nothing() {
        let list = derive_shopping_list(&input(
            vec![meal(
                "pm-1",
                "2026-08-29",
                MealSlot::Dinner,
                vec![
                    MealComponent::Leftovers { note: None },
                    MealComponent::freeform("pizza").unwrap(),
                ],
            )],
            vec![],
            vec![],
            vec![],
        ));
        assert_eq!(list.non_recipe_components, 2);
        assert_eq!(list.contribution_count, 0);
        assert!(list.groups.is_empty());
    }

    #[test]
    fn a_component_whose_recipe_is_missing_from_the_snapshot_is_reported_not_dropped() {
        let list = derive_shopping_list(&input(
            vec![meal(
                "pm-1",
                "2026-08-29",
                MealSlot::Dinner,
                vec![component("absent", Some(r(2, 1)))],
            )],
            vec![],
            vec![],
            vec![],
        ));
        let l = only_line(&list);
        assert_eq!(l.key, "s:pm-1:0:0");
        assert_eq!(l.name, "absent");
        assert_eq!(l.separate_reason, Some(SeparateReason::Unresolved));
        assert_eq!(l.quantity, Quantity::Unknown);
        assert_eq!(l.contributions.len(), 1);
        assert_eq!(l.contributions[0].recipe_id.as_str(), "absent");
        assert_eq!(list.contribution_count, 1);
    }

    #[test]
    fn derived_quantities_are_never_zero_or_negative() {
        let list = derive_shopping_list(&four_meal_fixture());
        let floor = r(1, u32::MAX);
        let mut checked = 0;
        for l in all_lines(&list) {
            match l.quantity {
                Quantity::Unknown => {}
                Quantity::Exact(q) => {
                    assert!(q >= floor);
                    checked += 1;
                }
                Quantity::Range(range) => {
                    assert!(range.min() >= floor);
                    assert!(range.max() >= range.min());
                    checked += 1;
                }
            }
        }
        assert!(checked > 0);
    }

    #[test]
    fn the_list_echoes_its_range_and_version() {
        let list = derive_shopping_list(&input(vec![], vec![], vec![], vec![]));
        assert_eq!(list.from, date("2026-08-29"));
        assert_eq!(list.to, date("2026-09-04"));
        assert_eq!(list.algorithm_version, 1);
        assert!(list.groups.is_empty());
    }

    #[test]
    fn unit_families_partition_the_vocabulary() {
        for kind in UnitKind::ALL {
            let family = UnitFamily::of(kind);
            assert_eq!(UnitFamily::of(family.base()), family);
            assert_eq!(crate::base_factor(family.base()), 1);
            assert_eq!(crate::base_factor(kind), expected_family(kind).1);
        }
    }

    // --- Step 6: performance bound ------------------------------------------------------------

    /// 93 meals × 3 recipe components × 30 lines = 8,370 contributions. Asserts the output
    /// invariants and prints the wall time for the evidence row; no timing assertion, since
    /// PRD §18 wants budgets from measurement rather than invented.
    #[test]
    fn a_full_31_day_three_slot_cycle_derives_in_bounded_work() {
        let kinds = [UnitKind::Cup, UnitKind::Tablespoon, UnitKind::Gram];
        let recipes: Vec<Recipe> = (0..3)
            .map(|ri| {
                recipe(
                    &format!("r{ri}"),
                    (0..30)
                        .map(|li| {
                            line(
                                &format!("ingredient {li}"),
                                Some(cat(&format!("i{li}"))),
                                exact(li + 1, 2),
                                known(kinds[(ri + li as usize) % 3]),
                                li % 7 == 0,
                            )
                        })
                        .collect(),
                )
            })
            .collect();
        let mut meals = Vec::new();
        let mut day = date("2026-08-01");
        for d in 0..31 {
            for (s, slot) in MealSlot::ALL.into_iter().enumerate() {
                meals.push(meal(
                    &format!("pm-{d}-{s}"),
                    &crate::format_civil_date(day),
                    slot,
                    vec![
                        component("r0", Some(r(3, 2))),
                        component("r1", None),
                        component("r2", Some(r(1, 2))),
                    ],
                ));
            }
            day = day.tomorrow().unwrap();
        }
        let identities = (0..30)
            .map(|li| {
                identity(
                    &format!("i{li}"),
                    &format!("Ingredient {li}"),
                    Some(["produce", "baking", "dairy"][li % 3]),
                )
            })
            .collect();
        let snapshot = ShoppingInput {
            from: date("2026-08-01"),
            to: date("2026-08-31"),
            meals,
            recipes,
            identities,
            pantry_marked: vec![cat("i0")],
        };
        let started = std::time::Instant::now();
        let list = derive_shopping_list(&snapshot);
        let elapsed = started.elapsed();
        println!("31-day derivation: {elapsed:?}");
        assert_eq!(list.contribution_count, 8370);
        assert_eq!(contributions_total(&list), 8370);
        assert_eq!(list.non_recipe_components, 0);
        assert_eq!(list.groups.len(), 3);
        assert!(all_lines(&list)
            .iter()
            .all(|l| l.separate_reason != Some(SeparateReason::ArithmeticOverflow)));
    }

    // --- line-key identity ----------------------------------------------------------------

    fn key_set(list: &ShoppingList) -> std::collections::BTreeSet<&str> {
        all_lines(list).iter().map(|l| l.key.as_str()).collect()
    }

    /// A `m:` key is the identity MVP-016 keys its edits on, and both its variable segments are
    /// caller-supplied verbatim — `id_newtype!` never rejects `:`, and `unit_to_domain` passes an
    /// `Other` text through unvalidated — so two different groups must not render one string.
    #[test]
    fn line_keys_are_unique_when_ids_and_unit_text_contain_colons() {
        let list = single(
            vec![
                line(
                    "first",
                    Some(cus("x:other=b")),
                    exact(1, 1),
                    Unit::Other("c".to_owned()),
                    false,
                ),
                line(
                    "second",
                    Some(cus("x")),
                    exact(1, 1),
                    Unit::Other("b:other=c".to_owned()),
                    false,
                ),
            ],
            vec![],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        let keys = key_set(&list);
        assert_eq!(keys.len(), lines.len(), "colliding keys: {keys:?}");
    }

    /// Escaping has to be injective over `\` as well as `:`. The first pair below collides with
    /// no escaping at all; the second collides under a colon-only escape, where `s\` + `t:other=u`
    /// and `s:other=t\` + `u` both render `…s\:other=t\:other=u…`. Together they pin that both
    /// characters are handled, which neither pair does alone.
    #[test]
    fn line_keys_stay_unique_when_ids_and_unit_text_contain_backslashes() {
        let list = single(
            vec![
                line(
                    "a",
                    Some(cus("p:other=q")),
                    exact(1, 1),
                    Unit::Other("r".to_owned()),
                    false,
                ),
                line(
                    "b",
                    Some(cus("p")),
                    exact(1, 1),
                    Unit::Other("q:other=r".to_owned()),
                    false,
                ),
                line(
                    "c",
                    Some(cus("s\\")),
                    exact(1, 1),
                    Unit::Other("t:other=u".to_owned()),
                    false,
                ),
                line(
                    "d",
                    Some(cus("s:other=t\\")),
                    exact(1, 1),
                    Unit::Other("u".to_owned()),
                    false,
                ),
            ],
            vec![],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 4);
        let keys = key_set(&list);
        assert_eq!(keys.len(), lines.len(), "colliding keys: {keys:?}");
        // The exact encoding docs/ROADMAP.md publishes as MVP-016's contract.
        assert!(
            keys.contains("m:custom:p\\:other=q:other=r:req:known"),
            "{keys:?}"
        );
    }

    /// Expected to pass before and after the escape: escaping is injective, so byte-identical
    /// `Other` text still merges and differing text still splits. This pins that the key fix did
    /// not move the merge discriminator.
    #[test]
    fn other_unit_text_containing_a_colon_still_merges_only_with_identical_text() {
        let list = single(
            vec![
                line(
                    "spice",
                    Some(cus("i")),
                    exact(1, 1),
                    Unit::Other("a:b".to_owned()),
                    false,
                ),
                line(
                    "spice",
                    Some(cus("i")),
                    exact(1, 1),
                    Unit::Other("a:b".to_owned()),
                    false,
                ),
                line(
                    "spice",
                    Some(cus("i")),
                    exact(1, 1),
                    Unit::Other("a".to_owned()),
                    false,
                ),
            ],
            vec![],
        );
        let lines = all_lines(&list);
        assert_eq!(lines.len(), 2);
        assert_eq!(contributions_total(&list), 3);
        let merged = lines
            .iter()
            .find(|l| l.unit == Unit::Other("a:b".to_owned()))
            .unwrap();
        assert_eq!(merged.contributions.len(), 2);
        assert_eq!(merged.quantity, exact(2, 1));
        assert!(lines
            .iter()
            .all(|l| l.separate_reason == Some(SeparateReason::UnitNotCombinable)));
    }
}
