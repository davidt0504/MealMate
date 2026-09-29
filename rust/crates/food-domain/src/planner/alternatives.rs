//! Alternatives for a Cover draft (OPT-007 §6): search a new meal for some slots while every
//! other displayed meal is held as fixed context. Pure, like the rest of `planner/`.
//!
//! Four concepts stay separate. A *locked* slot is the household's Tier-0 word and is never a
//! target. A *fixed* slot is only an operation boundary: its displayed meal is context for
//! scoring, carries no lock and borrows no Tier-0 exemption; whether it passes the checks is the
//! exact assessment's business, not this search's. A *target* is searched afresh with its
//! exclusions applied before ranking and the `K` cut, so an excluded dish never reappears however
//! it ranks. A target that no eligible dish can fill is *exhausted*: it keeps its displayed meal
//! as fixed context and the remaining targets are searched around it. Nothing here invents a
//! fallback to make an alternative exist.

use std::collections::BTreeSet;

use crate::planner::beam::{self, SearchTrace};
use crate::planner::candidates::{self, Candidate, CandidateSource, SlotCandidates};
use crate::planner::draft::identities;
use crate::planner::score::PlanScore;
use crate::planner::similarity::Commitment;
use crate::planner::snapshot::{PlanningSnapshot, SearchParams};
use crate::planner::tier0::{self, Feasible, Rejection};
use crate::{CivilDate, MealComponent, MealSlot};

/// How a slot takes part in one alternatives search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AltRole {
    /// Held as displayed: context only.
    Fixed,
    /// Searched afresh. `excluded` holds dish identities (`recipe:<id>` / `starter:<slug>`) that
    /// may not be proposed here. `fallbacks` admits the leftovers / dining-out / frozen-quick /
    /// open kinds, which only the initial proposal may place — Another never proposes one.
    Target {
        excluded: BTreeSet<String>,
        fallbacks: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AltSlot {
    pub date: CivilDate,
    pub slot: MealSlot,
    /// What the draft shows now; empty for a slot with nothing in it.
    pub display: Vec<MealComponent>,
    /// Locked in: a commitment every candidate is compared with for similarity (§7). Always
    /// `Fixed`; the caller never targets it.
    pub committed: bool,
    pub role: AltRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AltOutcome {
    /// A target that got a meal differing from its display in at least one dish identity.
    Changed(Vec<MealComponent>),
    /// A target no eligible dish could fill; its display is unchanged.
    Exhausted,
    Fixed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AltResult {
    /// One per input slot, in input order, addressed by date and slot — never by position in
    /// the search's chosen list.
    pub outcomes: Vec<(CivilDate, MealSlot, AltOutcome)>,
    pub rejections: Vec<Rejection>,
    pub trace: SearchTrace,
    pub score: PlanScore,
}

fn fixed(snapshot: &PlanningSnapshot, s: &AltSlot) -> Vec<Feasible> {
    if s.display.is_empty() {
        return Vec::new();
    }
    vec![Feasible {
        candidate: Candidate {
            date: s.date,
            slot: s.slot,
            components: s.display.clone(),
            source: CandidateSource::ExistingPlan,
            locked: false,
            views: candidates::views_of(snapshot, &s.display),
        },
        assumptions: Vec::new(),
        wording_only: Vec::new(),
    }]
}

fn eligible(source: CandidateSource, fallbacks: bool) -> bool {
    match source {
        CandidateSource::HouseholdRecipe | CandidateSource::StarterMeal => true,
        CandidateSource::ExistingPlan => false,
        _ => fallbacks,
    }
}

/// `slots` in canonical `(date, slot)` order, as a draft holds them.
pub fn alternatives(
    snapshot: &PlanningSnapshot,
    slots: &[AltSlot],
    params: &SearchParams,
) -> AltResult {
    let generated = candidates::generate(snapshot);
    let mut rejections = Vec::new();
    let mut lists: Vec<Vec<Feasible>> = Vec::with_capacity(slots.len());
    let mut exhausted = vec![false; slots.len()];
    for (index, s) in slots.iter().enumerate() {
        let AltRole::Target {
            excluded,
            fallbacks,
        } = &s.role
        else {
            lists.push(fixed(snapshot, s));
            continue;
        };
        // Built fresh rather than taken from generation's slot: a target is by definition not
        // resolved for this search (the caller never targets a locked slot), so no stored lock
        // may reject its candidates as `LOCK_CONFLICT`.
        let pool: Vec<Candidate> = generated
            .iter()
            .find(|g| g.date == s.date && g.slot == s.slot)
            .map(|g| {
                g.candidates
                    .iter()
                    .filter(|c| eligible(c.source, *fallbacks))
                    .filter(|c| {
                        identities(snapshot, &c.components)
                            .iter()
                            .all(|id| !excluded.contains(id))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        let (feasible, rejected) = tier0::filter(
            snapshot,
            &SlotCandidates {
                date: s.date,
                slot: s.slot,
                resolved: false,
                candidates: pool,
            },
        );
        rejections.extend(rejected);
        if feasible.is_empty() {
            exhausted[index] = true;
            lists.push(fixed(snapshot, s));
        } else {
            lists.push(feasible);
        }
    }
    let commitments: Vec<Commitment> = slots
        .iter()
        .filter(|s| s.committed && !s.display.is_empty())
        .map(|s| Commitment::of(s.date, s.slot, &candidates::views_of(snapshot, &s.display)))
        .filter(|c| !c.keys.is_empty())
        .collect();
    let outcome = beam::search_with(snapshot, params, &lists, &commitments);
    let mut chosen = outcome.chosen.iter();
    let mut outcomes = Vec::with_capacity(slots.len());
    for (index, s) in slots.iter().enumerate() {
        let picked = if lists[index].is_empty() {
            None
        } else {
            chosen.next()
        };
        let result = match (&s.role, picked) {
            (AltRole::Fixed, _) => AltOutcome::Fixed,
            (AltRole::Target { .. }, _) if exhausted[index] => AltOutcome::Exhausted,
            (AltRole::Target { .. }, Some(f)) => {
                debug_assert_eq!((f.candidate.date, f.candidate.slot), (s.date, s.slot));
                let before: BTreeSet<String> =
                    identities(snapshot, &s.display).into_iter().collect();
                let after: BTreeSet<String> = identities(snapshot, &f.candidate.components)
                    .into_iter()
                    .collect();
                // A change is a different dish, not a reordered or rescaled one — and a
                // fallback replacing an empty slot is a change too.
                if after != before || (s.display.is_empty() && !f.candidate.components.is_empty()) {
                    AltOutcome::Changed(f.candidate.components.clone())
                } else {
                    AltOutcome::Exhausted
                }
            }
            (AltRole::Target { .. }, None) => AltOutcome::Exhausted,
        };
        outcomes.push((s.date, s.slot, result));
    }
    AltResult {
        outcomes,
        rejections,
        trace: outcome.trace,
        score: outcome.score,
    }
}

#[cfg(test)]
mod tests {
    use household_core::HouseholdId;

    use super::*;
    use crate::planner::snapshot::{FoodPolicies, RecipeCandidateInfo};
    use crate::{parse_civil_date, HouseholdRestrictions, MealScope, RecipeId};

    fn d(s: &str) -> CivilDate {
        parse_civil_date(s).unwrap()
    }

    fn info(id: &str) -> RecipeCandidateInfo {
        RecipeCandidateInfo {
            id: RecipeId::new(id).unwrap(),
            title: format!("Dish {id}"),
            servings: Some(2),
            prep_minutes: Some(20),
            line_names: vec![format!("{id} base")],
            ingredient_refs: Vec::new(),
            starter_slug: None,
            untagged_lines: vec![format!("{id} base")],
        }
    }

    fn snapshot(recipes: &[&str], days: u32) -> PlanningSnapshot {
        PlanningSnapshot {
            household_id: HouseholdId::new("h").unwrap(),
            anchor: d("2026-09-28"),
            length_days: days,
            scope: MealScope::dinner_only(),
            today: d("2026-09-28"),
            members: Vec::new(),
            restrictions: HouseholdRestrictions::new(Vec::new()),
            preferences: Vec::new(),
            policies: FoodPolicies::default(),
            recipes: recipes.iter().map(|r| info(r)).collect(),
            starter: Vec::new(),
            existing: Vec::new(),
            history: Vec::new(),
            pantry_marked: Vec::new(),
        }
    }

    fn rc(id: &str) -> MealComponent {
        MealComponent::recipe(RecipeId::new(id).unwrap(), None)
    }

    fn date_n(n: u32) -> CivilDate {
        let mut out = d("2026-09-28");
        for _ in 0..n {
            out = out.tomorrow().unwrap();
        }
        out
    }

    fn fixed_slot(n: u32, recipe: &str) -> AltSlot {
        AltSlot {
            date: date_n(n),
            slot: MealSlot::Dinner,
            display: vec![rc(recipe)],
            committed: false,
            role: AltRole::Fixed,
        }
    }

    fn target(n: u32, recipe: &str, excluded: &[&str]) -> AltSlot {
        AltSlot {
            date: date_n(n),
            slot: MealSlot::Dinner,
            display: vec![rc(recipe)],
            committed: false,
            role: AltRole::Target {
                excluded: excluded.iter().map(|e| format!("recipe:{e}")).collect(),
                fallbacks: false,
            },
        }
    }

    fn changed(r: &AltResult, n: usize) -> Option<Vec<MealComponent>> {
        match &r.outcomes[n].2 {
            AltOutcome::Changed(c) => Some(c.clone()),
            _ => None,
        }
    }

    #[test]
    fn another_changes_only_the_target_and_never_to_an_excluded_dish() {
        let s = snapshot(&["a", "b", "c", "d"], 3);
        let slots = [
            fixed_slot(0, "a"),
            target(1, "b", &["b"]),
            fixed_slot(2, "c"),
        ];
        let r = alternatives(&s, &slots, &SearchParams::default());
        assert_eq!(r.outcomes[0].2, AltOutcome::Fixed);
        assert_eq!(r.outcomes[2].2, AltOutcome::Fixed);
        let new = changed(&r, 1).expect("a real alternative exists");
        assert_ne!(new, vec![rc("b")]);
        assert_eq!(new.len(), 1, "one whole occurrence, no fallback component");
        // Every date stays where it was.
        let dates: Vec<_> = r.outcomes.iter().map(|o| o.0).collect();
        assert_eq!(dates, vec![date_n(0), date_n(1), date_n(2)]);
    }

    /// R4: exclusions filter before the `K` cut, so a dish that would rank first is absent even
    /// at K=1, and an exhausted slot keeps its display — no fallback is invented.
    #[test]
    fn excluded_dishes_never_reappear_at_k_1_and_exhaustion_invents_nothing() {
        let s = snapshot(&["a", "b"], 3);
        let params = SearchParams {
            beam_width: 1,
            candidates_per_slot: 1,
            ..SearchParams::default()
        };
        let slots = [
            fixed_slot(0, "a"),
            target(1, "b", &["a", "b"]),
            fixed_slot(2, "a"),
        ];
        let r = alternatives(&s, &slots, &params);
        assert_eq!(r.outcomes[1].2, AltOutcome::Exhausted);
        let slots = [
            fixed_slot(0, "a"),
            target(1, "b", &["b"]),
            fixed_slot(2, "b"),
        ];
        let r = alternatives(&s, &slots, &params);
        assert_eq!(changed(&r, 1), Some(vec![rc("a")]));
    }

    /// R4: an exhausted first, middle or last target holds its date; every neighbour is
    /// addressed by its own date, never by a zip of filtered results.
    #[test]
    fn exhausting_first_middle_or_last_keeps_every_neighbour_on_its_date() {
        let s = snapshot(&["a", "b", "c", "d", "e"], 3);
        for exhausted_at in 0..3 {
            let slots: Vec<AltSlot> = (0..3)
                .map(|n| {
                    if n == exhausted_at {
                        target(n, "a", &["a", "b", "c", "d", "e"])
                    } else {
                        target(n, "b", &["b"])
                    }
                })
                .collect();
            let r = alternatives(&s, &slots, &SearchParams::default());
            for (n, (date, _, outcome)) in r.outcomes.iter().enumerate() {
                assert_eq!(*date, date_n(n as u32));
                if n as u32 == exhausted_at {
                    assert_eq!(*outcome, AltOutcome::Exhausted);
                } else {
                    let AltOutcome::Changed(c) = outcome else {
                        panic!("slot {n} should change: {outcome:?}");
                    };
                    assert_ne!(c, &vec![rc("b")]);
                }
            }
        }
    }

    /// Empty fixed slots (a past date with nothing planned) at the start, middle or end are
    /// skipped by the search without shifting any target's result.
    #[test]
    fn empty_fixed_slots_anywhere_do_not_shift_results() {
        let s = snapshot(&["a", "b", "c"], 3);
        for empty_at in 0..3 {
            let slots: Vec<AltSlot> = (0..3)
                .map(|n| {
                    if n == empty_at {
                        AltSlot {
                            date: date_n(n),
                            slot: MealSlot::Dinner,
                            display: Vec::new(),
                            committed: false,
                            role: AltRole::Fixed,
                        }
                    } else {
                        target(n, "a", &["a"])
                    }
                })
                .collect();
            let r = alternatives(&s, &slots, &SearchParams::default());
            for (n, (date, _, outcome)) in r.outcomes.iter().enumerate() {
                assert_eq!(*date, date_n(n as u32));
                if n as u32 == empty_at {
                    assert_eq!(*outcome, AltOutcome::Fixed);
                } else {
                    assert!(matches!(outcome, AltOutcome::Changed(_)), "{outcome:?}");
                }
            }
        }
        // Every slot empty and fixed: nothing to search, nothing claimed.
        let slots: Vec<AltSlot> = (0..3)
            .map(|n| AltSlot {
                date: date_n(n),
                slot: MealSlot::Dinner,
                display: Vec::new(),
                committed: false,
                role: AltRole::Fixed,
            })
            .collect();
        let r = alternatives(&s, &slots, &SearchParams::default());
        assert!(r.outcomes.iter().all(|o| o.2 == AltOutcome::Fixed));
    }

    /// A vetoed dish is rejected at Tier 0 as a target candidate, so it cannot come back as an
    /// alternative; and a target never receives a fallback kind unless the caller admits them.
    #[test]
    fn a_vetoed_dish_is_not_an_alternative_and_fallbacks_need_admission() {
        let mut s = snapshot(&["a", "b"], 1);
        s.policies.hard_vetoes = vec!["Dish a".to_owned()];
        let r = alternatives(&s, &[target(0, "b", &["b"])], &SearchParams::default());
        assert_eq!(r.outcomes[0].2, AltOutcome::Exhausted);
        assert!(r.rejections.iter().any(|x| x.code == tier0::HARD_VETO));
        let empty = AltSlot {
            date: date_n(0),
            slot: MealSlot::Dinner,
            display: Vec::new(),
            committed: false,
            role: AltRole::Target {
                excluded: ["recipe:a".to_owned(), "recipe:b".to_owned()].into(),
                fallbacks: true,
            },
        };
        let r = alternatives(&s, &[empty], &SearchParams::default());
        let Some(c) = changed(&r, 0) else {
            panic!("an admitted fallback fills an empty slot");
        };
        assert!(!c.iter().any(|x| matches!(x, MealComponent::Recipe { .. })));
    }

    #[test]
    fn identical_inputs_give_identical_alternatives() {
        let s = snapshot(&["a", "b", "c", "d", "e", "f"], 4);
        let slots: Vec<AltSlot> = (0..4).map(|n| target(n, "a", &["a"])).collect();
        let one = alternatives(&s, &slots, &SearchParams::default());
        let two = alternatives(&s.clone(), &slots, &SearchParams::default());
        assert_eq!(one, two);
    }
}
