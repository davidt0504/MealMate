//! Tier-0 hard filtering (PRD §9.4, invariant 18). Everything here is a rejection with a
//! code or an uncertainty carried forward — never a weight. Absence of a detected conflict
//! is not proof of anything, which is why the `wording_only` case is an assumption and not
//! a pass.

use crate::planner::candidates::{Candidate, SlotCandidates};
use crate::planner::snapshot::PlanningSnapshot;
use crate::restriction::{assess, contains_phrase, tokens, RULE_VERSION};
use crate::{CivilDate, MealComponent, MealSlot};

pub const RESTRICTION_CONFLICT: &str = "RESTRICTION_CONFLICT";
pub const HARD_VETO: &str = "HARD_VETO";
pub const PREP_WINDOW_IMPOSSIBLE: &str = "PREP_WINDOW_IMPOSSIBLE";
pub const LOCK_CONFLICT: &str = "LOCK_CONFLICT";
pub const LOCK_HELD_OVER: &str = "LOCK_HELD_OVER";
pub const RESTRICTION_WORDING_UNVERIFIED: &str = "RESTRICTION_WORDING_UNVERIFIED";
pub const NON_RECIPE_COMPONENT_UNVERIFIED: &str = "NON_RECIPE_COMPONENT_UNVERIFIED";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub date: CivilDate,
    pub slot: MealSlot,
    pub candidate_text: String,
    /// `RESTRICTION_CONFLICT` carries the matcher's rule version, so a ledger row can say
    /// which term list produced it (`restriction.rs` `RULE_VERSION`).
    pub code: String,
}

/// A candidate that passed, with the uncertainties it passed *under*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feasible {
    pub candidate: Candidate,
    pub assumptions: Vec<String>,
    /// `Other` restrictions matched by wording only against this candidate's lines: never a
    /// rejection, always an open hard-constraint question (§11).
    pub wording_only: Vec<String>,
}

pub(crate) fn restriction_conflict_code() -> String {
    format!("{RESTRICTION_CONFLICT}:rules_v{RULE_VERSION}")
}

fn veto_hit(candidate: &Candidate, vetoes: &[String]) -> bool {
    vetoes.iter().any(|veto| {
        let phrase = tokens(veto);
        candidate.views.iter().any(|v| {
            contains_phrase(&tokens(&v.title), &phrase)
                || v.line_names
                    .iter()
                    .any(|name| contains_phrase(&tokens(name), &phrase))
        })
    })
}

/// The longest prep among the candidate's dishes, or `None` if any is unknown: a meal with
/// one unknown component has an unknown prep time, not the known part's.
pub(crate) fn prep_minutes(candidate: &Candidate) -> Option<u32> {
    if candidate.views.is_empty() {
        return None;
    }
    candidate
        .views
        .iter()
        .map(|v| v.prep_minutes)
        .try_fold(0u32, |acc, p| p.map(|p| acc.max(p)))
}

/// Splits one slot's candidates into the feasible and the rejected. A resolved slot keeps
/// only its own `ExistingPlan` candidate; every other candidate is rejected with
/// `LOCK_CONFLICT` so the lock has domain evidence at every beam width (AC-1).
pub fn filter(
    snapshot: &PlanningSnapshot,
    slot: &SlotCandidates,
) -> (Vec<Feasible>, Vec<Rejection>) {
    let mut feasible = Vec::new();
    let mut rejections = Vec::new();
    let reject = |candidate: &Candidate, code: String| Rejection {
        date: candidate.date,
        slot: candidate.slot,
        candidate_text: candidate.text(),
        code,
    };
    for candidate in &slot.candidates {
        if slot.resolved && !candidate.locked && !is_the_existing_open(candidate) {
            rejections.push(reject(candidate, LOCK_CONFLICT.to_owned()));
            continue;
        }
        // A lock is the user's own Tier-0 word: automation may never move it, so a check
        // that would reject it is carried as an assumption rather than a rejection —
        // the slot stays `LockedByUser` and the ledger still says what was overridden.
        let held = |code: String| -> Option<String> {
            candidate.locked.then(|| format!("{LOCK_HELD_OVER}:{code}"))
        };
        let mut assumptions = Vec::new();
        let mut wording_only = Vec::new();
        let mut conflict = false;
        for view in &candidate.views {
            let assessment = assess(
                view.line_names.iter().map(String::as_str),
                &snapshot.restrictions,
            );
            if !assessment.conflicts.is_empty() {
                conflict = true;
            }
            for text in assessment.wording_only {
                if !wording_only.contains(&text) {
                    wording_only.push(text);
                }
            }
        }
        if conflict {
            match held(restriction_conflict_code()) {
                Some(a) => assumptions.push(a),
                None => {
                    rejections.push(reject(candidate, restriction_conflict_code()));
                    continue;
                }
            }
        }
        if veto_hit(candidate, &snapshot.policies.hard_vetoes) {
            match held(HARD_VETO.to_owned()) {
                Some(a) => assumptions.push(a),
                None => {
                    rejections.push(reject(candidate, HARD_VETO.to_owned()));
                    continue;
                }
            }
        }
        let window = snapshot.policies.slot_windows.get(&candidate.slot).copied();
        if let (Some(prep), Some(window)) = (prep_minutes(candidate), window) {
            if prep > window {
                match held(PREP_WINDOW_IMPOSSIBLE.to_owned()) {
                    Some(a) => assumptions.push(a),
                    None => {
                        rejections.push(reject(candidate, PREP_WINDOW_IMPOSSIBLE.to_owned()));
                        continue;
                    }
                }
            }
        }
        if !wording_only.is_empty() {
            assumptions.push(RESTRICTION_WORDING_UNVERIFIED.to_owned());
        }
        if candidate
            .components
            .iter()
            .any(|c| !matches!(c, MealComponent::Recipe { .. } | MealComponent::Open { .. }))
        {
            assumptions.push(NON_RECIPE_COMPONENT_UNVERIFIED.to_owned());
        }
        feasible.push(Feasible {
            candidate: candidate.clone(),
            assumptions,
            wording_only,
        });
    }
    (feasible, rejections)
}

/// The `ExistingPlan` candidate of an `Open` slot is the human decision itself.
fn is_the_existing_open(candidate: &Candidate) -> bool {
    candidate.source == crate::planner::candidates::CandidateSource::ExistingPlan
        && candidate.is_open()
}
