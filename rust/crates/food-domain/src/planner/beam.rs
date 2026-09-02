//! Whole-cycle beam search (PRD §9.6). Canonical `(date, slot)` order — most-constrained-first
//! is `MVP-025`'s benchmark question. Every state is scored whole, so sequence effects
//! (repeats, overlap, leftovers) are what the search optimises, not per-slot scores.

use crate::format_civil_date;
use crate::planner::score::{score_plan, PlanScore, Score};
use crate::planner::snapshot::{components_text, PlanningSnapshot, SearchParams};
use crate::planner::tier0::Feasible;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTrace {
    pub beam_width: u32,
    pub candidates_per_slot: u32,
    pub slot_order: String,
    /// `Σ_slots live_before × feasible_after_truncation`: 2,076 for 21 full slots at B=8,
    /// K=12, where the 18 slots that can source leftovers carry K+1; a resolved slot
    /// contributes `live_before × 1`.
    pub states_scored: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOutcome {
    pub chosen: Vec<Feasible>,
    pub score: PlanScore,
    pub trace: SearchTrace,
}

/// The deterministic tie-break key: one line per slot.
pub fn plan_text(chosen: &[Feasible]) -> String {
    chosen
        .iter()
        .map(|f| {
            format!(
                "{} {} {}",
                format_civil_date(f.candidate.date),
                f.candidate.slot.as_str(),
                components_text(&f.candidate.components)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `slots` is one feasible list per enabled slot, in canonical order. A slot with no feasible
/// candidate is skipped (its coverage is then `Unresolved`); generation makes that unreachable
/// today because the two fallbacks are never rejected on an unresolved slot.
pub fn search(
    snapshot: &PlanningSnapshot,
    params: &SearchParams,
    slots: &[Vec<Feasible>],
) -> SearchOutcome {
    let beam_width = params.beam_width.max(1);
    let per_slot = params.candidates_per_slot.max(1);
    let total = slots.len();
    let mut live: Vec<(Vec<Feasible>, PlanScore)> =
        vec![(Vec::new(), score_plan(snapshot, params, &[], total))];
    let mut states_scored: u32 = 0;
    for feasible in slots {
        if feasible.is_empty() {
            continue;
        }
        // Rank this slot's candidates alone, so the truncation to K is itself deterministic.
        let mut ranked: Vec<(Score, &Feasible)> = feasible
            .iter()
            .map(|f| {
                (
                    score_plan(snapshot, params, std::slice::from_ref(f), total).score,
                    f,
                )
            })
            .collect();
        ranked.sort_by(|(sa, a), (sb, b)| {
            sb.cmp(sa)
                .then(a.candidate.source.cmp(&b.candidate.source))
                .then(a.candidate.text().cmp(&b.candidate.text()))
        });
        // Leftovers are exempt from the `K` cut. Ranked alone at index 0 a leftovers candidate
        // has no preceding slot to source it, so `leftover_source` is false however good the
        // whole plan would be: it scores `LEFTOVER_SOURCE_MISSING` with `tiers[0] = 0` while
        // every other source scores `SLOT_COVERED` with 1, and `Score` compares from tier 1,
        // so it sorts strictly last and the cut dropped it before the whole-plan search could
        // judge it — making PRD §9.3's leftovers source unreachable past `K` recipes.
        // Generation pushes at most one, and a stored occurrence at most one more, so the
        // exemption is bounded at two per slot.
        let keep = ranked.len().min(per_slot);
        let dropped = ranked.split_off(keep);
        ranked.extend(
            dropped
                .into_iter()
                .filter(|(_, f)| f.candidate.is_leftovers()),
        );
        states_scored += (live.len() * ranked.len()) as u32;
        let mut next: Vec<(Vec<Feasible>, PlanScore)> =
            Vec::with_capacity(live.len() * ranked.len());
        for (partial, _) in &live {
            for (_, candidate) in &ranked {
                let mut extended = partial.clone();
                extended.push((*candidate).clone());
                let score = score_plan(snapshot, params, &extended, total);
                next.push((extended, score));
            }
        }
        next.sort_by(|(pa, sa), (pb, sb)| {
            sb.score
                .cmp(&sa.score)
                .then_with(|| plan_text(pa).cmp(&plan_text(pb)))
        });
        next.truncate(beam_width);
        live = next;
    }
    let (chosen, score) = live.into_iter().next().expect("the beam is never empty");
    SearchOutcome {
        chosen,
        score,
        trace: SearchTrace {
            beam_width: beam_width as u32,
            candidates_per_slot: per_slot as u32,
            slot_order: "canonical".to_owned(),
            states_scored,
        },
    }
}
