//! Candidate generation (PRD §9.3): the seven sources, for every enabled slot. Resolved slots
//! (locked, or an explicit `Open`) are generated too, so Tier 0 can reject every other
//! candidate with a recorded `LOCK_CONFLICT` rather than the lock being invisible.

use crate::planner::snapshot::{components_text, PlanningSnapshot};
use crate::{
    format_civil_date, CivilDate, IngredientLine, IngredientRef, MealComponent, MealSlot,
    PlannedMeal, Rational, StarterRecipe,
};

/// Exactly the seven §9.3 sources, in the order the PRD lists them; that order is also the
/// second tie-break key when two candidates score equally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CandidateSource {
    ExistingPlan,
    HouseholdRecipe,
    StarterMeal,
    Leftovers,
    DiningOut,
    FrozenQuick,
    IntentionallyOpen,
}

impl CandidateSource {
    pub const ALL: [Self; 7] = [
        Self::ExistingPlan,
        Self::HouseholdRecipe,
        Self::StarterMeal,
        Self::Leftovers,
        Self::DiningOut,
        Self::FrozenQuick,
        Self::IntentionallyOpen,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExistingPlan => "existing_plan",
            Self::HouseholdRecipe => "household_recipe",
            Self::StarterMeal => "starter_meal",
            Self::Leftovers => "leftovers",
            Self::DiningOut => "dining_out",
            Self::FrozenQuick => "frozen_quick",
            Self::IntentionallyOpen => "intentionally_open",
        }
    }

    /// The sources whose presence means the slot has a real meal option (§2.2
    /// `needs_attention` is the state where none of these survived Tier 0). `Leftovers` is
    /// a meal only when the search finds it a source, which `cover_cycle` checks separately.
    pub fn is_meal(self) -> bool {
        !matches!(
            self,
            Self::FrozenQuick | Self::IntentionallyOpen | Self::Leftovers
        )
    }
}

/// What scoring and Tier 0 read about a recipe-like component, whether it is a household
/// recipe, a starter entry, or an existing occurrence naming one. `key` identifies the dish
/// across slots and history (`recipe:<id>` / `starter:<slug>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeView {
    pub key: String,
    pub title: String,
    pub servings: Option<u32>,
    pub scale: Option<Rational>,
    pub prep_minutes: Option<u32>,
    pub line_names: Vec<String>,
    pub refs: Vec<IngredientRef>,
}

impl RecipeView {
    /// `servings × scale ≥ members + 1`: the dish plausibly leaves a second meal. Unknown
    /// servings never qualify — absence is not a serving count.
    pub fn feeds_leftovers(&self, members: usize) -> bool {
        let Some(servings) = self.servings else {
            return false;
        };
        let (numer, denom) = self
            .scale
            .map_or((1, 1), |s| (u64::from(s.numer()), u64::from(s.denom())));
        u64::from(servings) * numer >= (members as u64 + 1) * denom
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub date: CivilDate,
    pub slot: MealSlot,
    pub components: Vec<MealComponent>,
    pub source: CandidateSource,
    /// Carried from the existing occurrence; only `ExistingPlan` can be locked.
    pub locked: bool,
    pub views: Vec<RecipeView>,
}

impl Candidate {
    /// The one-line rendering every rejection, plan text and ledger payload uses.
    pub fn text(&self) -> String {
        format!(
            "{} {} {} {}",
            format_civil_date(self.date),
            self.slot.as_str(),
            self.source.as_str(),
            components_text(&self.components)
        )
    }

    pub fn is_open(&self) -> bool {
        self.components.iter().any(MealComponent::is_open)
    }

    pub fn is_leftovers(&self) -> bool {
        self.components
            .iter()
            .any(|c| matches!(c, MealComponent::Leftovers { .. }))
    }
}

/// One enabled slot's candidates, with whether an explicit human decision already holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotCandidates {
    pub date: CivilDate,
    pub slot: MealSlot,
    pub resolved: bool,
    pub candidates: Vec<Candidate>,
}

/// A lock, or an explicit `Open`, is a human decision at Tier 0 (§9.7).
pub fn is_resolved(existing: &PlannedMeal) -> bool {
    existing.locked() || existing.components().iter().any(MealComponent::is_open)
}

/// How a starter is spelled in both places it appears: the note of the `Freeform` occurrence a
/// `StarterMeal` persists as, and the `starter:<slug>` half of [`RecipeView::key`]. One
/// constant, so [`generate`] cannot write a note [`recipe_views_of`] no longer recognises.
pub(crate) const STARTER_NOTE_PREFIX: &str = "starter:";

pub(crate) fn recipe_views_of(snapshot: &PlanningSnapshot, meal: &PlannedMeal) -> Vec<RecipeView> {
    meal.components()
        .iter()
        .filter_map(|c| match c {
            MealComponent::Recipe { recipe_id, scale } => {
                Some(match snapshot.recipes.iter().find(|r| &r.id == recipe_id) {
                    Some(r) => RecipeView {
                        key: format!("recipe:{}", r.id.as_str()),
                        title: r.title.clone(),
                        servings: r.servings,
                        scale: *scale,
                        prep_minutes: r.prep_minutes,
                        line_names: r.line_names.clone(),
                        refs: r.ingredient_refs.clone(),
                    },
                    // An occurrence naming a recipe the snapshot does not carry (archived): known
                    // by id only, so nothing about it can be verified and nothing is guessed —
                    // the id stays in `key`, where repeat detection wants it, and out of `title`,
                    // whose only readers tokenize it as a dish name (`tier0::veto_hit`,
                    // `score::matches_subject`). `RecipeId` is any non-blank string, so an id
                    // there would let a veto or a preference match on an identifier.
                    None => RecipeView {
                        key: format!("recipe:{}", recipe_id.as_str()),
                        title: String::new(),
                        servings: None,
                        scale: *scale,
                        prep_minutes: None,
                        line_names: Vec::new(),
                        refs: Vec::new(),
                    },
                })
            }
            // A `StarterMeal` persists as this stub rather than as a `Recipe` component (see
            // [`generate`]), so without resolving it the occurrence the applier wrote reads back
            // with no views at all — and `feeds_leftovers`, Tier 0's restriction and hard-veto
            // checks and the prep window would every one of them flip across the apply, on a
            // slot `cover_cycle` and `assess` write the two halves of a ledger row about.
            MealComponent::Freeform { note } => note
                .strip_prefix(STARTER_NOTE_PREFIX)
                .map(|slug| starter_stub_view(snapshot, slug)),
            _ => None,
        })
        .collect()
}

fn starter_view(s: &StarterRecipe) -> RecipeView {
    RecipeView {
        key: format!("{STARTER_NOTE_PREFIX}{}", s.slug),
        title: s.title.clone(),
        servings: s.servings,
        scale: None,
        prep_minutes: s.prep_minutes,
        line_names: s.lines.iter().map(|l| l.name().to_owned()).collect(),
        refs: s
            .lines
            .iter()
            .filter_map(IngredientLine::ingredient)
            .cloned()
            .collect(),
    }
}

/// The view behind a stored `freeform:starter:<slug>` stub. Installed first: once the household
/// installs the slug the recipe *is* the dish, and the loader drops the slug from
/// `snapshot.starter` — so the recipe is the only place left holding its facts. A slug in
/// neither list (installed, then archived: the loader's held-slug query counts archived recipes
/// while its recipe query excludes them) is known by slug only, the same treatment
/// [`recipe_views_of`] gives an archived recipe — and, like there, the slug identifies the dish
/// in `key` without standing in for its name in `title`. A slug is written to be readable, so
/// tokenizing one as a dish name would let `score::matches_subject` credit a preference and
/// `tier0::veto_hit` reject, both on the strength of an identifier. That case still yields a view
/// rather than nothing, so an unknown prep time reads as unknown instead of as no schedule claim
/// to make.
fn starter_stub_view(snapshot: &PlanningSnapshot, slug: &str) -> RecipeView {
    if let Some(r) = snapshot
        .recipes
        .iter()
        .find(|r| r.starter_slug.as_deref() == Some(slug))
    {
        return RecipeView {
            key: format!("recipe:{}", r.id.as_str()),
            title: r.title.clone(),
            servings: r.servings,
            scale: None,
            prep_minutes: r.prep_minutes,
            line_names: r.line_names.clone(),
            refs: r.ingredient_refs.clone(),
        };
    }
    match snapshot.starter.iter().find(|s| s.slug == slug) {
        Some(s) => starter_view(s),
        None => RecipeView {
            key: format!("{STARTER_NOTE_PREFIX}{slug}"),
            title: String::new(),
            servings: None,
            scale: None,
            prep_minutes: None,
            line_names: Vec::new(),
            refs: Vec::new(),
        },
    }
}

/// Whether this component names a dish in free text that [`recipe_views_of`] leaves no view
/// behind — a `Freeform` note that is not the `starter:<slug>` stub a `StarterMeal` persists as.
/// Lives here rather than in `coverage`, so the prefix rule stays in the module that owns
/// [`STARTER_NOTE_PREFIX`] and cannot drift from the arm that applies it.
pub(crate) fn is_unverifiable_dish(component: &MealComponent) -> bool {
    matches!(component, MealComponent::Freeform { note } if !note.starts_with(STARTER_NOTE_PREFIX))
}

/// Whether a *stored* occurrence on the day before `date` feeds the household plus one. This
/// is the half of [`leftovers_possible`] that asks about state rather than possibility, so
/// `FoodController::assess` — which runs no search and so has no chosen plan to consult — can
/// read the same *input set* for the leftovers-source question that `score::leftover_source`
/// reads. Two partitions have to line up, not just the date one. Date: the loader gives
/// `existing` nothing before the anchor and `history` nothing from the anchor on — but
/// `PlanningSnapshot`'s fields are public and hand-built snapshots are a supported case, so the
/// `previous < anchor` guard enforces the history half rather than the comment doing it, which
/// is how `score::leftover_source` reads the same half. Slot: inside the cycle the
/// search sees only what it placed, which is the enabled slots, while `existing` is loaded by
/// date range alone and can hold a slot a later scope narrowing dropped — so `existing` is
/// filtered to the scope and `history`, which `score_plan` reads whole into
/// `history_views_by_date`, is not. Equal inputs, not equal verdicts: an in-scope slot the
/// search rejects at Tier 0 or churns away still differs, which is the search's business.
pub(crate) fn leftovers_sourced_from_stored(
    snapshot: &PlanningSnapshot,
    date: CivilDate,
    members: usize,
) -> bool {
    let Ok(previous) = date.yesterday() else {
        return false;
    };
    let slots = snapshot.scope.slots();
    let history = (previous < snapshot.anchor).then_some(&snapshot.history);
    snapshot
        .existing
        .iter()
        .filter(|m| slots.contains(&m.slot()))
        .chain(history.into_iter().flatten())
        .filter(|m| m.date() == previous)
        .flat_map(|m| recipe_views_of(snapshot, m))
        .any(|v| v.feeds_leftovers(members))
}

/// Whether any dish reachable on the day before `date` could leave leftovers: the previous
/// day's stored occurrence (history, for the first cycle date), or — for a later date — any
/// recipe or starter that feeds the household plus one. Which one actually precedes the
/// slot is the search's business; scoring credits the utility only when it does.
fn leftovers_possible(snapshot: &PlanningSnapshot, date: CivilDate, members: usize) -> bool {
    let Ok(previous) = date.yesterday() else {
        return false;
    };
    if leftovers_sourced_from_stored(snapshot, date, members) {
        return true;
    }
    let in_cycle = previous >= snapshot.anchor;
    in_cycle
        && (snapshot.recipes.iter().any(|r| {
            RecipeView {
                key: String::new(),
                title: String::new(),
                servings: r.servings,
                scale: None,
                prep_minutes: None,
                line_names: Vec::new(),
                refs: Vec::new(),
            }
            .feeds_leftovers(members)
        }) || snapshot
            .starter
            .iter()
            .any(|s| starter_view(s).feeds_leftovers(members)))
}

/// Every enabled slot in canonical `(date, slot)` order. Starter entries already installed
/// are absent from `snapshot.starter` by the loader's contract, so a starter never appears
/// twice; this function also drops any whose slug an active recipe carries, so a snapshot
/// built by hand cannot double one either.
pub fn generate(snapshot: &PlanningSnapshot) -> Vec<SlotCandidates> {
    let members = snapshot.members.len().max(1);
    let installed: Vec<&str> = snapshot
        .recipes
        .iter()
        .filter_map(|r| r.starter_slug.as_deref())
        .collect();
    let mut out = Vec::new();
    for date in snapshot.dates() {
        for slot in snapshot.scope.slots().iter().copied() {
            let existing = snapshot.existing_at(date, slot);
            let resolved = existing.is_some_and(is_resolved);
            let mut candidates = Vec::new();
            if let Some(existing) = existing {
                candidates.push(Candidate {
                    date,
                    slot,
                    components: existing.components().to_vec(),
                    source: CandidateSource::ExistingPlan,
                    locked: existing.locked(),
                    views: recipe_views_of(snapshot, existing),
                });
            }
            for r in &snapshot.recipes {
                candidates.push(Candidate {
                    date,
                    slot,
                    components: vec![MealComponent::recipe(r.id.clone(), None)],
                    source: CandidateSource::HouseholdRecipe,
                    locked: false,
                    views: vec![RecipeView {
                        key: format!("recipe:{}", r.id.as_str()),
                        title: r.title.clone(),
                        servings: r.servings,
                        scale: None,
                        prep_minutes: r.prep_minutes,
                        line_names: r.line_names.clone(),
                        refs: r.ingredient_refs.clone(),
                    }],
                });
            }
            for s in snapshot
                .starter
                .iter()
                .filter(|s| !installed.contains(&s.slug.as_str()))
            {
                // A starter is not yet a household recipe, so its occurrence is a freeform
                // stub naming the slug: installing it is the household's decision (T4).
                let note = format!("{STARTER_NOTE_PREFIX}{}", s.slug);
                let Ok(component) = MealComponent::freeform(note) else {
                    continue;
                };
                candidates.push(Candidate {
                    date,
                    slot,
                    components: vec![component],
                    source: CandidateSource::StarterMeal,
                    locked: false,
                    views: vec![starter_view(s)],
                });
            }
            if leftovers_possible(snapshot, date, members) {
                candidates.push(Candidate {
                    date,
                    slot,
                    components: vec![MealComponent::Leftovers { note: None }],
                    source: CandidateSource::Leftovers,
                    locked: false,
                    views: Vec::new(),
                });
            }
            if snapshot.policies.dining_out_enabled {
                candidates.push(Candidate {
                    date,
                    slot,
                    components: vec![MealComponent::DiningOut { note: None }],
                    source: CandidateSource::DiningOut,
                    locked: false,
                    views: Vec::new(),
                });
            }
            candidates.push(Candidate {
                date,
                slot,
                components: vec![MealComponent::FrozenQuick { note: None }],
                source: CandidateSource::FrozenQuick,
                locked: false,
                views: Vec::new(),
            });
            candidates.push(Candidate {
                date,
                slot,
                components: vec![MealComponent::Open { note: None }],
                source: CandidateSource::IntentionallyOpen,
                locked: false,
                views: Vec::new(),
            });
            out.push(SlotCandidates {
                date,
                slot,
                resolved,
                candidates,
            });
        }
    }
    out
}
