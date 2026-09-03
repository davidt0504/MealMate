//! The FoodController planner core (MVP-023, PRD §9–§11): deterministic, offline, clock-free.
//! `cover_cycle` is a pure function of a [`PlanningSnapshot`] and [`SearchParams`]; nothing
//! under `planner/` touches IO, and the test `planner_source_has_no_io_imports` pins it.

pub mod attention;
pub mod beam;
pub mod candidates;
pub mod coverage;
pub mod score;
pub mod snapshot;
pub mod tier0;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(test)]
mod invariant_tests;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;

use household_core::{
    ActionProposal, AttentionRequest, Horizon, HouseholdController, OutcomeAssessment,
    OutcomeStatus, ReasonCode,
};

pub use attention::APPLY_PLAN;
pub use beam::{SearchOutcome, SearchTrace};
pub use candidates::{Candidate, CandidateSource, RecipeView, SlotCandidates};
pub use coverage::{
    issue_text, CoverageState, SlotCoverage, SlotFallbacks, FALLBACK_PLACEHOLDER_CHOSEN, ISSUE_TEXT,
};
pub use score::{PlanScore, Score, Term};
pub use snapshot::{FoodPolicies, PlanningSnapshot, RecipeCandidateInfo, SearchParams};
pub use tier0::{Feasible, Rejection};

use crate::planner::snapshot::components_text;
use crate::{format_civil_date, CivilDate, MealComponent, MealSlot};

/// Bumped whenever candidate generation, Tier 0, scoring, search or coverage changes
/// behaviour, so a ledger row says which planner produced it (invariant 20).
pub const PLANNER_ALGORITHM_VERSION: u32 = 2;
pub const CONTROLLER_ID: &str = "food";

/// Every code the planner emits is built by this crate without whitespace, so the
/// constructor cannot fail; a member id inside a code is whitespace-mapped first.
pub(crate) fn reason(code: &str) -> ReasonCode {
    ReasonCode::new(code.replace(char::is_whitespace, "_"))
        .expect("planner codes have no whitespace")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedMeal {
    pub date: CivilDate,
    pub slot: MealSlot,
    pub components: Vec<MealComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningResult {
    pub algorithm_version: u32,
    pub snapshot_hash: String,
    pub slots: Vec<SlotCoverage>,
    /// The chosen components for every enabled slot, resolved ones included; the applier
    /// skips slots whose components equal the existing occurrence (§9.9).
    pub proposed: Vec<ProposedMeal>,
    pub rejections: Vec<Rejection>,
    pub assessment: OutcomeAssessment,
    pub proposals: Vec<ActionProposal>,
    pub attention: Vec<AttentionRequest>,
    pub search: SearchTrace,
    pub score: PlanScore,
}

impl PlanningResult {
    /// One line per fact; the ledger payload. Stable for the same snapshot and version.
    pub fn canonical_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("algorithm_version={}\n", self.algorithm_version));
        out.push_str(&format!("snapshot_hash={}\n", self.snapshot_hash));
        out.push_str(&format!("status={}\n", self.assessment.status.as_str()));
        out.push_str(&format!("score={:?}\n", self.score.score.0));
        for s in &self.slots {
            out.push_str(&format!(
                "slot={} {} {} {} {} [{}]\n",
                format_civil_date(s.date),
                s.slot.as_str(),
                s.state.as_str(),
                s.source.map(CandidateSource::as_str).unwrap_or(""),
                components_text(&s.components),
                s.reason_codes.join(" "),
            ));
        }
        // `LOCK_CONFLICT` collapses to one line per slot with a count; every other code keeps
        // its candidate line. A resolved slot rejects every other candidate by design (AC-1),
        // so that one code grew the payload with `slots × candidates` — 28.9 KB on a seven-day
        // locked week with 50 recipes — into a table whose `DELETE` the v10 trigger refuses,
        // and its text is the generated set minus the lock, which the slot's own state already
        // says. A restriction, veto or window rejection names a dish a check refused: that text
        // is the diagnostic and it stays — but only up to `REJECTION_LINES_PER_CODE`, because
        // Tier 0 rejects a hit recipe independently on every enabled slot, so that text grew
        // the same row with `slots × rejected_recipes`: 93 slots × 40 hit recipes is ~279 KB,
        // and every preview appends another one. `PlanningResult::rejections` keeps every
        // rejection whole either way, so the domain result and the bridge are unaffected and
        // the lock still has domain evidence at every beam width.
        const REJECTION_LINES_PER_CODE: usize = 20;
        let mut locked_out: BTreeMap<(CivilDate, MealSlot), usize> = BTreeMap::new();
        let mut kept: BTreeMap<&str, usize> = BTreeMap::new();
        let mut over_budget: BTreeMap<&str, usize> = BTreeMap::new();
        for r in &self.rejections {
            if r.code == tier0::LOCK_CONFLICT {
                *locked_out.entry((r.date, r.slot)).or_default() += 1;
                continue;
            }
            let seen = kept.entry(r.code.as_str()).or_default();
            if *seen < REJECTION_LINES_PER_CODE {
                *seen += 1;
                out.push_str(&format!("rejected={} {}\n", r.code, r.candidate_text));
            } else {
                *over_budget.entry(r.code.as_str()).or_default() += 1;
            }
        }
        // After the kept lines and before the lock counts, in code order: the payload is
        // compared byte-for-byte across runs and across a SQLite round trip, so both the set
        // and the position of these lines have to be a function of the rejections alone.
        // ASCII `...`, like the rest of the format.
        for (code, count) in &over_budget {
            out.push_str(&format!("rejected={code} ... and {count} more\n"));
        }
        for ((date, slot), count) in &locked_out {
            out.push_str(&format!(
                "rejected={} {} {} count={}\n",
                tier0::LOCK_CONFLICT,
                format_civil_date(*date),
                slot.as_str(),
                count
            ));
        }
        for t in &self.score.terms {
            out.push_str(&format!(
                "term={} {} {} {}\n",
                t.date.map(format_civil_date).unwrap_or_default(),
                t.slot.map(MealSlot::as_str).unwrap_or(""),
                t.code,
                t.value
            ));
        }
        out.push_str(&format!(
            "assumptions={}\n",
            self.assessment
                .assumptions
                .iter()
                .map(ReasonCode::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        ));
        for a in &self.attention {
            out.push_str(&format!(
                "attention={} {} {} {} [{}]\n",
                a.id,
                a.urgency.as_str(),
                a.decision_benefit_band.as_str(),
                a.estimated_effort_band.as_str(),
                a.options.join("|")
            ));
        }
        out.push_str(&format!(
            "search=B{} K{} {} states={}\n",
            self.search.beam_width,
            self.search.candidates_per_slot,
            self.search.slot_order,
            self.search.states_scored
        ));
        out
    }
}

/// `PlanningSnapshot`'s fields are public, so a caller other than the loader can build one
/// with `length_days: 0` — for which `dates()` is empty. The anchor always exists, so the
/// horizon degenerates to a single day rather than the cycle indexing out of bounds; the
/// cycle's own status is then `Unresolved`, which `cycle_status` already returns for an empty
/// slot list.
fn horizon(snapshot: &PlanningSnapshot) -> Horizon {
    let dates = snapshot.dates();
    Horizon {
        from: format_civil_date(dates.first().copied().unwrap_or(snapshot.anchor)),
        to: format_civil_date(dates.last().copied().unwrap_or(snapshot.anchor)),
    }
}

/// First-appearance order is load-bearing — it is the `assumptions`, `unresolved_issues` and
/// payload order — so the set is only the membership test, never the ordering.
fn dedup_codes(codes: impl IntoIterator<Item = String>) -> Vec<ReasonCode> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::new();
    for c in codes {
        if seen.insert(c.clone()) {
            out.push(reason(&c));
        }
    }
    out
}

/// The food-specific command (PRD §7.8 keeps it outside the trait): generate, filter, search,
/// assess, and describe — with no IO and no clock.
pub fn cover_cycle(snapshot: &PlanningSnapshot, params: &SearchParams) -> PlanningResult {
    let generated = candidates::generate(snapshot);
    let mut feasible_by_slot = Vec::with_capacity(generated.len());
    let mut only_fallbacks = Vec::with_capacity(generated.len());
    let mut rejections = Vec::new();
    for slot in &generated {
        let (feasible, rejected) = tier0::filter(snapshot, slot);
        only_fallbacks.push(!feasible.iter().any(|f| f.candidate.source.is_meal()));
        feasible_by_slot.push(feasible);
        rejections.extend(rejected);
    }
    let outcome = beam::search(snapshot, params, &feasible_by_slot);
    let mut chosen_iter = outcome.chosen.iter();
    let mut slots = Vec::with_capacity(generated.len());
    for (index, slot) in generated.iter().enumerate() {
        let chosen = if feasible_by_slot[index].is_empty() {
            None
        } else {
            chosen_iter.next()
        };
        // Leftovers with a source the search actually placed before it is a meal too.
        let sourced_leftovers = chosen.is_some_and(|f| f.candidate.is_leftovers())
            && outcome.score.terms.iter().any(|t| {
                t.code == score::LEFTOVER_UTILITY
                    && t.date == Some(slot.date)
                    && t.slot == Some(slot.slot)
            });
        slots.push(coverage::slot_coverage(
            snapshot,
            slot.date,
            slot.slot,
            chosen,
            coverage::SlotFallbacks {
                only_fallbacks: only_fallbacks[index],
                leftovers_sourced: sourced_leftovers,
            },
        ));
    }
    let assumptions = {
        let mut a = coverage::sufficiency(snapshot, &slots);
        for extra in &outcome.score.assumptions {
            if !a.contains(extra) {
                a.push(extra.clone());
            }
        }
        a
    };
    let status = coverage::cycle_status(&slots, &assumptions);
    let unresolved_issues: Vec<String> = assumptions
        .iter()
        .chain(slots.iter().flat_map(|s| s.reason_codes.iter()))
        .map(|c| coverage::issue_text(c))
        .filter(|t| !t.is_empty())
        .fold(Vec::new(), |mut acc, t| {
            if !acc.iter().any(|s: &String| s == t) {
                acc.push(t.to_owned());
            }
            acc
        });
    let reason_codes = dedup_codes(
        slots
            .iter()
            .flat_map(|s| s.reason_codes.iter().cloned())
            .chain(rejections.iter().map(|r| r.code.clone()))
            .chain(outcome.score.terms.iter().map(|t| t.code.clone())),
    );
    let from = snapshot.dates().first().copied().unwrap_or(snapshot.anchor);
    let attention = attention::requests(
        &slots,
        &feasible_by_slot,
        &outcome.chosen,
        &assumptions,
        from,
    );
    let proposals = attention::proposals(&assumptions, &reason_codes, from);
    let proposed = outcome
        .chosen
        .iter()
        .map(|f| ProposedMeal {
            date: f.candidate.date,
            slot: f.candidate.slot,
            components: f.candidate.components.clone(),
        })
        .collect();
    PlanningResult {
        algorithm_version: PLANNER_ALGORITHM_VERSION,
        snapshot_hash: snapshot.snapshot_hash(),
        slots,
        proposed,
        rejections,
        assessment: OutcomeAssessment {
            controller_id: CONTROLLER_ID.to_owned(),
            horizon: horizon(snapshot),
            status,
            unresolved_issues,
            assumptions: dedup_codes(assumptions),
            reason_codes,
        },
        proposals,
        attention,
        search: outcome.trace,
        score: outcome.score,
    }
}

/// The controller. `assess` reads the existing plan without searching; `propose` runs the
/// default search and returns its proposals. Both are called by `kimatta-application`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FoodController;

impl HouseholdController for FoodController {
    type Context = PlanningSnapshot;
    type Error = Infallible;

    /// Coverage of what is already planned: each enabled slot's existing occurrence goes
    /// through Tier 0 alone. A slot with nothing planned is `Unresolved`; a planned meal that
    /// Tier 0 rejects is `NeedsAttention`. No candidate generation, no search.
    fn assess(&self, snapshot: &PlanningSnapshot) -> Result<OutcomeAssessment, Infallible> {
        let mut slots = Vec::new();
        let mut rejected_codes = Vec::new();
        for date in snapshot.dates() {
            for slot in snapshot.scope.slots().iter().copied() {
                let Some(existing) = snapshot.existing_at(date, slot) else {
                    slots.push(coverage::slot_coverage(
                        snapshot,
                        date,
                        slot,
                        None,
                        coverage::SlotFallbacks::default(),
                    ));
                    continue;
                };
                let generated = SlotCandidates {
                    date,
                    slot,
                    resolved: candidates::is_resolved(existing),
                    candidates: vec![Candidate {
                        date,
                        slot,
                        components: existing.components().to_vec(),
                        source: CandidateSource::ExistingPlan,
                        locked: existing.locked(),
                        views: candidates::recipe_views_of(snapshot, existing),
                    }],
                };
                let (feasible, rejected) = tier0::filter(snapshot, &generated);
                // The placeholder grading is deliberately reachable here: it reads the stored
                // components, not the candidate's source, so a frozen/quick occurrence grades
                // exactly as `cover_cycle` grades the same components. That agreement is what
                // keeps a row's `resulting_status` and the next row's `prior_status` from
                // contradicting each other. `only_fallbacks` stays false — `assess` runs no
                // candidate generation and cannot know whether a real option existed, and it
                // does not need to: that flag only ever escalates to `NeedsAttention`, which
                // this path reaches through `feasible.is_empty()` below.
                let mut cov = coverage::slot_coverage(
                    snapshot,
                    date,
                    slot,
                    feasible.first(),
                    coverage::SlotFallbacks {
                        only_fallbacks: false,
                        // Gated on the very candidate `slot_coverage` will read the flag for:
                        // it consults `leftovers_sourced` only inside its own `is_leftovers()`
                        // branch, and the scan rebuilds a `RecipeView` per stored `Recipe`
                        // component — the cost `score_plan` hoists out of the search's inner
                        // loop for the same reason. An empty `feasible` never reaches the flag
                        // at all, because `slot_coverage` returns on `chosen: None` first.
                        leftovers_sourced: feasible
                            .first()
                            .is_some_and(|f| f.candidate.is_leftovers())
                            && candidates::leftovers_sourced_from_stored(
                                snapshot,
                                date,
                                snapshot.members.len().max(1),
                            ),
                    },
                );
                if feasible.is_empty() {
                    cov.state = CoverageState::NeedsAttention;
                    cov.components = existing.components().to_vec();
                    cov.source = Some(CandidateSource::ExistingPlan);
                    cov.reason_codes = rejected.iter().map(|r| r.code.clone()).collect();
                    rejected_codes.extend(rejected.into_iter().map(|r| r.code));
                }
                slots.push(cov);
            }
        }
        let assumptions = coverage::sufficiency(snapshot, &slots);
        let status = coverage::cycle_status(&slots, &assumptions);
        let unresolved_issues = assumptions
            .iter()
            .chain(slots.iter().flat_map(|s| s.reason_codes.iter()))
            .map(|c| coverage::issue_text(c))
            .filter(|t| !t.is_empty())
            .fold(Vec::new(), |mut acc: Vec<String>, t| {
                if !acc.iter().any(|s| s == t) {
                    acc.push(t.to_owned());
                }
                acc
            });
        Ok(OutcomeAssessment {
            controller_id: CONTROLLER_ID.to_owned(),
            horizon: horizon(snapshot),
            status,
            unresolved_issues,
            assumptions: dedup_codes(assumptions),
            reason_codes: dedup_codes(
                slots
                    .iter()
                    .flat_map(|s| s.reason_codes.iter().cloned())
                    .chain(rejected_codes),
            ),
        })
    }

    fn propose(&self, snapshot: &PlanningSnapshot) -> Result<Vec<ActionProposal>, Infallible> {
        Ok(cover_cycle(snapshot, &SearchParams::default()).proposals)
    }
}

/// Convenience for callers that want the status the trait's `assess` reports.
pub fn assess_existing(snapshot: &PlanningSnapshot) -> OutcomeStatus {
    FoodController
        .assess(snapshot)
        .unwrap_or_else(|never| match never {})
        .status
}
