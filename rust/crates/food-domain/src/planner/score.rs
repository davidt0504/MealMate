//! Reason-coded scoring under the lexicographic tiers (PRD §9.5, §9.7, §9.8, §9.9, §9.10).
//! A `Score` is six integers compared in order, so no lower tier can ever compensate for a
//! higher one (invariant 18); every contribution is a `Term` with a code.

use std::collections::{BTreeMap, BTreeSet};

use crate::planner::candidates::{CandidateSource, RecipeView};
use crate::planner::snapshot::{PlanningSnapshot, SearchParams};
use crate::planner::tier0::{prep_minutes, Feasible};
use crate::restriction::{contains_phrase, tokens};
use crate::{CivilDate, MealComponent, MealSlot, Sentiment};

pub const SLOT_COVERED: &str = "SLOT_COVERED";
pub const SLOT_OPEN: &str = "SLOT_OPEN";
pub const LEFTOVER_SOURCE_MISSING: &str = "LEFTOVER_SOURCE_MISSING";
pub const PREP_FEASIBLE_UNKNOWN: &str = "PREP_FEASIBLE_UNKNOWN";
pub const MEMBER_LIKE: &str = "MEMBER_LIKE";
pub const MEMBER_DISLIKE: &str = "MEMBER_DISLIKE";
pub const CONTRADICTORY_PREFERENCE: &str = "CONTRADICTORY_PREFERENCE";
pub const REPEATED_SACRIFICE: &str = "REPEATED_SACRIFICE";
pub const CHURN_PENALTY: &str = "CHURN_PENALTY";
pub const NEAR_TERM_CHURN: &str = "NEAR_TERM_CHURN";
pub const SLACK_FRAGILE: &str = "SLACK_FRAGILE";
pub const SLACK_GOOD: &str = "SLACK_GOOD";
pub const STUB_NEEDS_DECISION: &str = "STUB_NEEDS_DECISION";
pub const FALLBACK_PLACEHOLDER: &str = "FALLBACK_PLACEHOLDER";
pub const REPEAT_IN_CYCLE: &str = "REPEAT_IN_CYCLE";
pub const RECENTLY_EATEN: &str = "RECENTLY_EATEN";
pub const INGREDIENT_OVERLAP: &str = "INGREDIENT_OVERLAP";
pub const PANTRY_FIT: &str = "PANTRY_FIT";
pub const LEFTOVER_UTILITY: &str = "LEFTOVER_UTILITY";
pub const LEFTOVER_ASSUMED: &str = "LEFTOVER_ASSUMED";
pub const NO_COST_DATA: &str = "NO_COST_DATA";
pub const NO_NUTRITION_DATA: &str = "NO_NUTRITION_DATA";

/// `[T1, T2_floor, T2_sum, T3, T4, T5]`, larger is better. The derived `Ord` on an array is
/// lexicographic, which is the whole point: tier 1 decides before tier 2 is looked at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(pub [i64; 6]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub date: Option<CivilDate>,
    pub slot: Option<MealSlot>,
    pub code: String,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanScore {
    pub score: Score,
    pub terms: Vec<Term>,
    pub assumptions: Vec<String>,
}

struct Ctx<'a> {
    snapshot: &'a PlanningSnapshot,
    params: &'a SearchParams,
    total_slots: usize,
    tiers: [i64; 6],
    terms: Vec<Term>,
    assumptions: Vec<String>,
    /// `member → (total, per-slot terms)`.
    members: BTreeMap<String, (i64, Vec<i64>)>,
    /// History is immutable for the whole search, but `recipe_views_of` linear-scans
    /// `snapshot.recipes` and deep-clones every matching title, line name and ref. Doing that
    /// per view, per slot, per scored state put it at the innermost position in the search.
    /// These two are built once per `score_plan` call instead — the beam still calls
    /// `score_plan` once per scored state, so the cost is `states_scored × |history|` rather
    /// than `states_scored × slots × views × |history|`.
    history_keys: BTreeSet<String>,
    history_views_by_date: BTreeMap<CivilDate, Vec<RecipeView>>,
}

impl Ctx<'_> {
    fn term(&mut self, tier: usize, f: &Feasible, code: &str, value: i64) {
        self.tiers[tier] += value;
        self.terms.push(Term {
            date: Some(f.candidate.date),
            slot: Some(f.candidate.slot),
            code: code.to_owned(),
            value,
        });
    }

    fn assume(&mut self, code: &str) {
        if !self.assumptions.iter().any(|a| a == code) {
            self.assumptions.push(code.to_owned());
        }
    }
}

fn matches_subject(f: &Feasible, subject: &str) -> bool {
    let phrase = tokens(subject);
    f.candidate.views.iter().any(|v| {
        contains_phrase(&tokens(&v.title), &phrase)
            || v.line_names
                .iter()
                .any(|n| contains_phrase(&tokens(n), &phrase))
    })
}

fn near_term(snapshot: &PlanningSnapshot, params: &SearchParams, date: CivilDate) -> bool {
    snapshot
        .today
        .until(date)
        .map(|span| i64::from(span.get_days()) < i64::from(params.commitment_horizon_days))
        .unwrap_or(true)
}

/// Whether the slot before `f` (in the partial plan, else the stored day before the cycle)
/// holds a dish that feeds the household plus one.
fn leftover_source(ctx: &Ctx<'_>, chosen: &[Feasible], index: usize) -> bool {
    let f = &chosen[index];
    let members = ctx.snapshot.members.len().max(1);
    let Ok(previous) = f.candidate.date.yesterday() else {
        return false;
    };
    let in_plan = chosen[..index]
        .iter()
        .filter(|c| c.candidate.date == previous)
        .flat_map(|c| c.candidate.views.iter())
        .any(|v| v.feeds_leftovers(members));
    if in_plan {
        return true;
    }
    previous < ctx.snapshot.anchor
        && ctx
            .history_views_by_date
            .get(&previous)
            .is_some_and(|views| views.iter().any(|v| v.feeds_leftovers(members)))
}

fn score_slot(ctx: &mut Ctx<'_>, chosen: &[Feasible], index: usize) {
    let f = &chosen[index];
    let c = &f.candidate;
    // --- Tier 1: coverage and feasibility ---------------------------------------------------
    if c.is_open() {
        ctx.term(0, f, SLOT_OPEN, 0);
    } else if c.is_leftovers() && !leftover_source(ctx, chosen, index) {
        ctx.term(0, f, LEFTOVER_SOURCE_MISSING, 0);
    } else {
        ctx.term(0, f, SLOT_COVERED, 1);
    }
    let window = ctx.snapshot.policies.slot_windows.get(&c.slot).copied();
    let prep = prep_minutes(c);
    if !c.views.is_empty() && (prep.is_none() || window.is_none()) {
        ctx.term(0, f, PREP_FEASIBLE_UNKNOWN, 0);
        ctx.assume(PREP_FEASIBLE_UNKNOWN);
    }
    // --- Tier 2: per-member acceptance, aggregated after the loop ---------------------------
    for member in &ctx.snapshot.members {
        let mut liked = Vec::new();
        let mut disliked = Vec::new();
        if let Some(prefs) = ctx.snapshot.preferences_of(member) {
            for p in prefs.preferences() {
                if matches_subject(f, p.subject()) {
                    match p.sentiment() {
                        Sentiment::Like => liked.push(p.subject().to_owned()),
                        Sentiment::Dislike => disliked.push(p.subject().to_owned()),
                    }
                }
            }
        }
        let mut slot_term = 0;
        for subject in &liked {
            if disliked.contains(subject) {
                ctx.term(2, f, CONTRADICTORY_PREFERENCE, 0);
            } else {
                slot_term += 1;
                ctx.term(2, f, MEMBER_LIKE, 1);
            }
        }
        for _ in &disliked {
            slot_term -= 2;
            ctx.term(2, f, MEMBER_DISLIKE, -2);
        }
        let entry = ctx
            .members
            .entry(member.as_str().to_owned())
            .or_insert_with(|| (0, Vec::new()));
        entry.0 += slot_term;
        entry.1.push(slot_term);
    }
    // --- Tier 3: churn and slack ------------------------------------------------------------
    if let Some(existing) = ctx.snapshot.existing_at(c.date, c.slot) {
        let human = existing.locked() || existing.components().iter().any(MealComponent::is_open);
        if !human && existing.components() != c.components.as_slice() {
            ctx.term(3, f, CHURN_PENALTY, -1);
            if near_term(ctx.snapshot, ctx.params, c.date) {
                ctx.term(3, f, NEAR_TERM_CHURN, -3);
            }
        }
    }
    if let (Some(prep), Some(window)) = (prep, window) {
        let slack = i64::from(window) - i64::from(prep);
        if slack < 10 {
            ctx.term(3, f, SLACK_FRAGILE, -1);
        } else if slack >= 20 {
            ctx.term(3, f, SLACK_GOOD, 1);
        }
    }
    // --- Tier 4: administrative attention ---------------------------------------------------
    for component in &c.components {
        if matches!(component, MealComponent::Freeform { .. }) {
            ctx.term(4, f, STUB_NEEDS_DECISION, -1);
        }
    }
    // A generated frozen/quick placeholder leaves the meal effectively undecided (§9.7 Tier 4:
    // minimize later swaps), so on an otherwise signal-less household a real dish still wins.
    // Scoring keys on `source` on purpose, and unlike `coverage::is_placeholder` it is right
    // to: this term exists to stop the search *choosing* a placeholder over a real dish, and
    // re-proposing what the household already has is not that choice. Coverage asks the other
    // question — whether the slot ends up with a dish decided in it — and a stored frozen/quick
    // answers that the same way a generated one does, whoever put it there.
    if c.source == CandidateSource::FrozenQuick {
        ctx.term(4, f, FALLBACK_PLACEHOLDER, -1);
    }
    // --- Tier 5: variety, reuse, pantry, leftovers ------------------------------------------
    for view in &c.views {
        let repeats = chosen[..index]
            .iter()
            .flat_map(|e| e.candidate.views.iter())
            .filter(|v| v.key == view.key)
            .count() as i64;
        if repeats > 0 {
            ctx.term(5, f, REPEAT_IN_CYCLE, -2 * repeats);
        }
        if ctx.history_keys.contains(&view.key) {
            ctx.term(5, f, RECENTLY_EATEN, -1);
        }
        // Reuse across *different* dishes only: a repeat sharing its own ingredients is the
        // `REPEAT_IN_CYCLE` case, not ingredient reuse.
        let overlap = view
            .refs
            .iter()
            .filter(|r| {
                chosen[..index]
                    .iter()
                    .flat_map(|e| e.candidate.views.iter())
                    .any(|v| v.key != view.key && v.refs.contains(r))
            })
            .count()
            .min(3) as i64;
        if overlap > 0 {
            ctx.term(5, f, INGREDIENT_OVERLAP, overlap);
        }
        let pantry = view
            .refs
            .iter()
            .filter(|r| ctx.snapshot.pantry_marked.contains(r))
            .count()
            .min(3) as i64;
        if pantry > 0 {
            ctx.term(5, f, PANTRY_FIT, pantry);
        }
    }
    if c.is_leftovers() && leftover_source(ctx, chosen, index) {
        ctx.term(5, f, LEFTOVER_UTILITY, 1);
        ctx.assume(LEFTOVER_ASSUMED);
    }
    for a in &f.assumptions {
        ctx.assume(a);
    }
}

/// Scores a (possibly partial) plan in canonical slot order. `total_slots` is the cycle's
/// enabled slot count, the denominator of the §9.10 step-6 sacrifice rule.
pub fn score_plan(
    snapshot: &PlanningSnapshot,
    params: &SearchParams,
    chosen: &[Feasible],
    total_slots: usize,
) -> PlanScore {
    let mut history_views_by_date: BTreeMap<CivilDate, Vec<RecipeView>> = BTreeMap::new();
    let mut history_keys = BTreeSet::new();
    for meal in &snapshot.history {
        // `recipe_views_of` synthesises a key for an occurrence naming a recipe the snapshot
        // no longer carries, so an archived dish still counts as recently eaten.
        for view in crate::planner::candidates::recipe_views_of(snapshot, meal) {
            history_keys.insert(view.key.clone());
            history_views_by_date
                .entry(meal.date())
                .or_default()
                .push(view);
        }
    }
    let mut ctx = Ctx {
        snapshot,
        params,
        total_slots,
        tiers: [0; 6],
        terms: Vec::new(),
        assumptions: Vec::new(),
        members: BTreeMap::new(),
        history_keys,
        history_views_by_date,
    };
    for index in 0..chosen.len() {
        score_slot(&mut ctx, chosen, index);
    }
    // §9.10 steps 4–6: floor, then aggregate minus repeated sacrifice of one member.
    let floor = ctx.members.values().map(|(t, _)| *t).min().unwrap_or(0);
    let sum: i64 = ctx.members.values().map(|(t, _)| *t).sum();
    let mut sacrifice_count: BTreeMap<&str, usize> = BTreeMap::new();
    for slot in 0..chosen.len() {
        let terms: Vec<(&str, i64)> = ctx
            .members
            .iter()
            .filter_map(|(m, (_, per_slot))| per_slot.get(slot).map(|v| (m.as_str(), *v)))
            .collect();
        let Some(lowest) = terms.iter().map(|(_, v)| *v).min() else {
            continue;
        };
        let lowest_members: Vec<&str> = terms
            .iter()
            .filter(|(_, v)| *v == lowest)
            .map(|(m, _)| *m)
            .collect();
        if lowest_members.len() == 1 && lowest_members.len() < terms.len() {
            *sacrifice_count.entry(lowest_members[0]).or_default() += 1;
        }
    }
    let mut sacrifice = 0;
    if let Some((member, count)) = sacrifice_count.iter().max_by_key(|(m, c)| (**c, *m)) {
        if *count * 2 > ctx.total_slots {
            sacrifice = *count as i64;
            ctx.terms.push(Term {
                date: None,
                slot: None,
                code: format!(
                    "{REPEATED_SACRIFICE}:{}",
                    member.replace(char::is_whitespace, "_")
                ),
                value: -2 * sacrifice,
            });
        }
    }
    ctx.tiers[1] = floor;
    ctx.tiers[2] = sum - 2 * sacrifice;
    ctx.assume(NO_COST_DATA);
    ctx.assume(NO_NUTRITION_DATA);
    PlanScore {
        score: Score(ctx.tiers),
        terms: ctx.terms,
        assumptions: ctx.assumptions,
    }
}
