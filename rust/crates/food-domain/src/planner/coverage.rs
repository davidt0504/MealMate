//! Coverage states (PRD §2.2) and model sufficiency (§10). "Covered" is relative to known
//! information: every claim the state cannot support is withheld by an assumption, and the
//! user-facing wording for each lives in `ISSUE_TEXT` so a copy review reads one table.

use household_core::OutcomeStatus;

use crate::planner::candidates::{is_unverifiable_dish, CandidateSource};
use crate::planner::snapshot::PlanningSnapshot;
use crate::planner::tier0::{prep_minutes, Feasible};
use crate::{CivilDate, MealComponent, MealSlot};

pub const NO_PREP_TIME_ESTIMATES: &str = "NO_PREP_TIME_ESTIMATES";
pub const PANTRY_INCOMPLETE: &str = "PANTRY_INCOMPLETE";
pub const RESTRICTIONS_NOT_CONFIGURED: &str = "RESTRICTIONS_NOT_CONFIGURED";
pub const PREFERENCES_SPARSE: &str = "PREFERENCES_SPARSE";
pub const CONSERVATIVE_DEFAULT: &str = "CONSERVATIVE_DEFAULT";
pub const PLAN_INFEASIBLE: &str = "PLAN_INFEASIBLE";
pub const HARD_CONSTRAINT_UNRESOLVED: &str = "HARD_CONSTRAINT_UNRESOLVED";
pub const UNKNOWN_POLICY_TYPE: &str = "UNKNOWN_POLICY_TYPE";
/// The bare form of `tier0::LOCK_HELD_OVER`, hoisted into the cycle assumptions by
/// [`sufficiency`] when any slot carries a `LOCK_HELD_OVER:<code>`. Held here rather than
/// imported so `CLAIM_BLOCKING` stays one table.
pub const LOCK_HELD_OVER: &str = "LOCK_HELD_OVER";
/// The slot holds a placeholder — a frozen/quick, or leftovers with no dish behind them.
/// Every check in [`slot_coverage`]'s covered branch is gated on the candidate having a dish,
/// so without this the placeholder would pass them all vacuously and claim `Covered` for a
/// slot with nothing decided in it. Keyed on the *components*, never on `candidate.source`:
/// see [`is_placeholder`].
pub const FALLBACK_PLACEHOLDER_CHOSEN: &str = "FALLBACK_PLACEHOLDER_CHOSEN";
/// The slot's only dish is a household-authored `Freeform` note, which resolves to no view —
/// the same vacuity [`FALLBACK_PLACEHOLDER_CHOSEN`] catches for a placeholder, reached by a
/// different route. Without it a note earns `Covered` off checks that ran against nothing, while
/// a `starter:<slug>` stub for an unknown slug — carrying exactly as much knowledge, none — is
/// held to `TentativelyCovered` by its facts-free view. Keyed on the components, like
/// [`is_placeholder`], so `cover_cycle` and `assess` grade a stored note identically.
pub const FREEFORM_DISH_UNVERIFIED: &str = "FREEFORM_DISH_UNVERIFIED";

/// The four assumptions that withhold a `Covered` claim for the whole cycle. Pantry is not
/// among them: it withholds the stock-completeness claim, not coverage (invariant 6).
/// `LOCK_HELD_OVER` is: a lock is the household's own word and the slot keeps it, but a hard
/// check that was skipped rather than passed cannot support an affirmative coverage claim
/// (§10 — never imply a claim the state cannot support). Every Tier-0 family counts, prep
/// window included; each is a check that did not run.
pub const CLAIM_BLOCKING: [&str; 4] = [
    LOCK_HELD_OVER,
    NO_PREP_TIME_ESTIMATES,
    RESTRICTIONS_NOT_CONFIGURED,
    PREFERENCES_SPARSE,
];

/// Every user-facing sentence the assessment can emit, keyed by the code that emits it. None
/// says "safe", "exact" or "personali…" — the test pins that, and so should the reviewer.
pub const ISSUE_TEXT: [(&str, &str); 10] = [
    (
        NO_PREP_TIME_ESTIMATES,
        "Schedule fit is not confirmed: a chosen recipe has no prep-time estimate, or the \
         slot has no time window.",
    ),
    (
        PANTRY_INCOMPLETE,
        "The shopping list can be ready, but stock completeness is unknown: pantry marks \
         are optional and say only that something was marked.",
    ),
    (
        RESTRICTIONS_NOT_CONFIGURED,
        "No dietary restrictions are configured, so no restriction check was applied to \
         this plan.",
    ),
    (
        PREFERENCES_SPARSE,
        "Fewer than three preferences are recorded, so this plan is a conservative default \
         rather than one tuned to the household.",
    ),
    (
        PLAN_INFEASIBLE,
        "Nothing but a fallback could be placed in this slot: no recipe, starter, sourced \
         leftovers or dining-out option was available for it.",
    ),
    (
        LOCK_HELD_OVER,
        "A meal you locked was kept even though a hard check would have rejected it; the \
         check is recorded, not applied.",
    ),
    (
        HARD_CONSTRAINT_UNRESOLVED,
        "A restriction written in the household's own words was matched by wording only and \
         not verified against a term list.",
    ),
    (
        UNKNOWN_POLICY_TYPE,
        "A policy of a type this planner does not interpret was present and not applied.",
    ),
    (
        FALLBACK_PLACEHOLDER_CHOSEN,
        "This slot holds a frozen/quick or unsourced-leftovers placeholder: no dish is \
         decided for it.",
    ),
    (
        FREEFORM_DISH_UNVERIFIED,
        "A free-text note names the meal for this slot, so no restriction, prep-time or \
         preference check could be applied to it.",
    ),
];

/// Resolves on the code's first `:`-separated segment, so a parameterised code
/// (`LOCK_HELD_OVER:HARD_VETO`) reaches the copy for its family instead of the empty string
/// the caller then filters away. No table key contains a `:`, so exact codes are unaffected,
/// and a segment with no entry still resolves to `""` — the prefix rule never invents copy.
pub fn issue_text(code: &str) -> &'static str {
    let family = code.split(':').next().unwrap_or(code);
    ISSUE_TEXT
        .iter()
        .find(|(c, _)| *c == family)
        .map(|(_, t)| *t)
        .unwrap_or("")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageState {
    Unresolved,
    TentativelyCovered,
    Covered,
    LockedByUser,
    IntentionallyOpen,
    NeedsAttention,
}

impl CoverageState {
    pub const ALL: [Self; 6] = [
        Self::Unresolved,
        Self::TentativelyCovered,
        Self::Covered,
        Self::LockedByUser,
        Self::IntentionallyOpen,
        Self::NeedsAttention,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unresolved => "unresolved",
            Self::TentativelyCovered => "tentatively_covered",
            Self::Covered => "covered",
            Self::LockedByUser => "locked_by_user",
            Self::IntentionallyOpen => "intentionally_open",
            Self::NeedsAttention => "needs_attention",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotCoverage {
    pub date: CivilDate,
    pub slot: MealSlot,
    pub state: CoverageState,
    pub source: Option<CandidateSource>,
    pub components: Vec<MealComponent>,
    pub reason_codes: Vec<String>,
}

/// Whether the chosen candidate's schedule fit can be claimed: every dish has a prep estimate
/// and the slot has a window. A candidate with no dish makes no schedule claim at all.
fn schedule_fit_known(snapshot: &PlanningSnapshot, f: &Feasible) -> bool {
    f.candidate.views.is_empty()
        || (prep_minutes(&f.candidate).is_some()
            && snapshot
                .policies
                .slot_windows
                .contains_key(&f.candidate.slot))
}

/// What a slot's search left, as [`slot_coverage`] needs it. `only_fallbacks` is the §2.2
/// `needs_attention` trigger: no meal source survived Tier 0, whatever the beam then placed.
/// `leftovers_sourced` is the one fact the components cannot carry — leftovers are a real meal
/// exactly when a dish they can come from precedes them, which only the caller knows. It says
/// nothing about any other placeholder: [`slot_coverage`] applies it only to a candidate whose
/// components actually are leftovers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SlotFallbacks {
    pub only_fallbacks: bool,
    pub leftovers_sourced: bool,
}

/// Whether the slot holds nothing but placeholders. Deliberately blind to
/// `CandidateSource`: the source of the *same* stored occurrence is `FrozenQuick` to the
/// search that placed it and `ExistingPlan` to the `assess` that reads it back, so grading on
/// it made the two disagree over one state — and `cover_cycle` and `assess` write the two
/// halves of every ledger row's `prior`/`resulting` pair, in a table whose `UPDATE` and
/// `DELETE` are refused by trigger. An empty component list is not a placeholder; it has
/// nothing to grade and keeps whatever the other checks make of it.
fn is_placeholder(components: &[MealComponent]) -> bool {
    !components.is_empty()
        && components.iter().all(|c| {
            matches!(
                c,
                MealComponent::FrozenQuick { .. } | MealComponent::Leftovers { .. }
            )
        })
}

/// One slot's state from what the search chose and what Tier 0 left.
pub fn slot_coverage(
    snapshot: &PlanningSnapshot,
    date: CivilDate,
    slot: MealSlot,
    chosen: Option<&Feasible>,
    fallbacks: SlotFallbacks,
) -> SlotCoverage {
    let mut reason_codes = Vec::new();
    let Some(f) = chosen else {
        return SlotCoverage {
            date,
            slot,
            state: CoverageState::Unresolved,
            source: None,
            components: Vec::new(),
            reason_codes,
        };
    };
    let no_meal_available = fallbacks.only_fallbacks && !fallbacks.leftovers_sourced;
    // Graded on what the slot *holds*, not on what Tier 0 left: the checks in the covered
    // branch are all gated on the candidate having a dish, so a placeholder would pass every
    // one of them vacuously (§10 — never imply a claim the state cannot support). The
    // `leftovers_sourced` escape applies only to leftovers, mirroring `cover_cycle`'s own
    // `is_leftovers()` gate — a frozen/quick that merely happens to follow a dish that feeds
    // leftovers is still a frozen/quick.
    let chosen_is_meal = !is_placeholder(&f.candidate.components)
        || (f.candidate.is_leftovers() && fallbacks.leftovers_sourced);
    let state = if f.candidate.locked {
        CoverageState::LockedByUser
    } else if f.candidate.is_open() {
        CoverageState::IntentionallyOpen
    } else if no_meal_available {
        reason_codes.push(PLAN_INFEASIBLE.to_owned());
        CoverageState::NeedsAttention
    } else {
        let mut tentative = false;
        if !chosen_is_meal {
            reason_codes.push(FALLBACK_PLACEHOLDER_CHOSEN.to_owned());
            tentative = true;
        }
        // The other route to the same vacuity: no placeholder, but no view either, so the three
        // checks below run against nothing. Gated on `views.is_empty()` — a `[Recipe, Freeform]`
        // occurrence is a supported shape whose recipe half *was* checked, and the code's
        // sentence would be false about it.
        if f.candidate.views.is_empty() && f.candidate.components.iter().any(is_unverifiable_dish) {
            reason_codes.push(FREEFORM_DISH_UNVERIFIED.to_owned());
            tentative = true;
        }
        if !schedule_fit_known(snapshot, f) {
            reason_codes.push(NO_PREP_TIME_ESTIMATES.to_owned());
            tentative = true;
        }
        if !f.candidate.views.is_empty() && snapshot.restrictions.restrictions().is_empty() {
            reason_codes.push(RESTRICTIONS_NOT_CONFIGURED.to_owned());
            tentative = true;
        }
        if !f.wording_only.is_empty() {
            reason_codes.push(HARD_CONSTRAINT_UNRESOLVED.to_owned());
            tentative = true;
        }
        if tentative {
            CoverageState::TentativelyCovered
        } else {
            CoverageState::Covered
        }
    };
    // Whatever the state, the uncertainties the candidate passed under travel with it — a
    // locked slot's `LOCK_HELD_OVER:*` included.
    for a in &f.assumptions {
        reason_codes.push(a.clone());
    }
    SlotCoverage {
        date,
        slot,
        state,
        source: Some(f.candidate.source),
        components: f.candidate.components.clone(),
        reason_codes,
    }
}

/// The §10 assumptions for a whole cycle, in a fixed order.
pub fn sufficiency(snapshot: &PlanningSnapshot, slots: &[SlotCoverage]) -> Vec<String> {
    let mut out = Vec::new();
    // First: a hard check the lock held over is the most severe thing a cycle can carry, and
    // this order is the `assumptions` order, the `unresolved_issues` order and the
    // `assumptions=` line in the ledger payload.
    if slots.iter().any(|s| {
        s.reason_codes
            .iter()
            .any(|c| c.split(':').next() == Some(LOCK_HELD_OVER))
    }) {
        out.push(LOCK_HELD_OVER.to_owned());
    }
    if slots
        .iter()
        .any(|s| s.reason_codes.iter().any(|c| c == NO_PREP_TIME_ESTIMATES))
    {
        out.push(NO_PREP_TIME_ESTIMATES.to_owned());
    }
    out.push(PANTRY_INCOMPLETE.to_owned());
    if snapshot.restrictions.restrictions().is_empty() {
        out.push(RESTRICTIONS_NOT_CONFIGURED.to_owned());
    }
    let entries: usize = snapshot
        .preferences
        .iter()
        .map(|(_, p)| p.preferences().len())
        .sum();
    if entries < 3 {
        out.push(PREFERENCES_SPARSE.to_owned());
        out.push(CONSERVATIVE_DEFAULT.to_owned());
    }
    if !snapshot.policies.unknown_policy_types.is_empty() {
        out.push(UNKNOWN_POLICY_TYPE.to_owned());
    }
    out
}

/// Cycle status from the slot states and the cycle assumptions. `Unresolved` slots make the
/// cycle `Unresolved` (an existing plan with gaps); otherwise `Covered` needs every slot
/// covered, locked or open and none of the claim-blocking assumptions.
pub fn cycle_status(slots: &[SlotCoverage], assumptions: &[String]) -> OutcomeStatus {
    if slots
        .iter()
        .any(|s| s.state == CoverageState::NeedsAttention)
    {
        return OutcomeStatus::NeedsAttention;
    }
    if slots.is_empty() || slots.iter().any(|s| s.state == CoverageState::Unresolved) {
        return OutcomeStatus::Unresolved;
    }
    let all_settled = slots.iter().all(|s| {
        matches!(
            s.state,
            CoverageState::Covered | CoverageState::LockedByUser | CoverageState::IntentionallyOpen
        )
    });
    let blocked = assumptions
        .iter()
        .any(|a| CLAIM_BLOCKING.contains(&a.as_str()));
    if all_settled && !blocked {
        OutcomeStatus::Covered
    } else {
        OutcomeStatus::TentativelyCovered
    }
}
