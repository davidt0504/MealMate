//! MVP-025 beam-width/candidate-limit benchmark (PRD §9.6, §18): sweeps (B, K) over the
//! version-controlled fixtures and compares plan quality lexicographically against the
//! shipped (8, 12). Quality (`Score`) and cost (`states_scored`) are hardware-independent;
//! wall time is host context only. Run with:
//! `cargo run -p food-domain --release --features fixtures --example beam_width`
//!
//! Verdict rule (pre-stated in the MVP-025 plan): B=8/K=12 survives unless some computed
//! cell achieves a strictly lexicographically better `Score` on any fixture. A first
//! difference at index 0–2 (coverage/acceptance/robustness) is adverse; one confined to
//! indices 3–5 is measured headroom.
//!
//! Why the exploratory grid runs on a subset without weakening the verdict: the search is
//! monotone in both knobs. `beam.rs` keeps `min(len, per_slot)` candidates per slot, so
//! `ranked_K ⊆ ranked_K'` for `K < K'`, and it truncates a totally ordered
//! `(score desc, plan_text asc)` sort, so a wider beam considers a superset of plans. The
//! `(64, feasible cap)` cell is therefore an upper bound on every `(B, K)` cell for that
//! fixture — and it is computed for *all* eleven. A fixture outside `SEARCH_HEAVY` can hide
//! no adverse cell that its `(64, cap)` cell does not already reveal; the grid buys
//! resolution on where quality saturates, not verdict safety.

use std::time::Instant;

use food_domain::planner::{candidates, cover_cycle, fixtures, tier0, SearchParams};

const EXPLORATORY_B: [usize; 7] = [1, 2, 4, 8, 16, 32, 64];
const EXPLORATORY_K: [usize; 7] = [2, 4, 6, 8, 12, 16, 24];
/// The fixtures whose candidate sets make the search do real work — selection criterion, so a
/// later reader can check the list rather than trust it: a large library
/// (`high_variety_household`, 48 recipes over two slots), a second candidate source competing
/// with recipes (`busy_week`'s tight window and dining out, `leftovers_fallback_heavy`'s
/// big-batch leftovers), near-duplicate ties (`repetitive_meal_library`), or dislikes that thin
/// the feasible set (`multiple_strong_dislikes`, which produced the recorded headroom cell).
const SEARCH_HEAVY: [&str; 5] = [
    "high_variety_household",
    "busy_week",
    "repetitive_meal_library",
    "leftovers_fallback_heavy",
    "multiple_strong_dislikes",
];

struct Cell {
    beam_width: usize,
    candidates_per_slot: usize,
    states_scored: u32,
    score: [i64; 6],
    median_ms: f64,
}

fn params(beam_width: usize, candidates_per_slot: usize) -> SearchParams {
    SearchParams {
        beam_width,
        candidates_per_slot,
        commitment_horizon_days: 2,
    }
}

fn run_cell(f: &fixtures::PlannerFixture, b: usize, k: usize) -> Cell {
    let p = params(b, k);
    let result = cover_cycle(&f.snapshot, &p); // warmup; also the measured artifact
    let mut times: Vec<f64> = (0..5)
        .map(|_| {
            let start = Instant::now();
            let timed = cover_cycle(&f.snapshot, &p);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(timed, result, "nondeterministic result during benchmark");
            elapsed
        })
        .collect();
    times.sort_by(|a, c| a.partial_cmp(c).expect("finite"));
    Cell {
        beam_width: b,
        candidates_per_slot: k,
        states_scored: result.search.states_scored,
        score: result.score.score.0,
        median_ms: times[times.len() / 2],
    }
}

/// The largest feasible candidate count any slot offers: the honest "uncut" K.
fn feasible_cap(f: &fixtures::PlannerFixture) -> usize {
    candidates::generate(&f.snapshot)
        .iter()
        .map(|slot| tier0::filter(&f.snapshot, slot).0.len())
        .max()
        .unwrap_or(1)
        .max(1)
}

fn first_diff(a: &[i64; 6], b: &[i64; 6]) -> Option<usize> {
    (0..6).find(|i| a[*i] != b[*i])
}

/// The CPU as `/proc/cpuinfo` names it. `model name` is the x86 key; ARM kernels — including
/// Android's, which the MVP-022 device run targets — emit `Hardware` or `Processor` instead,
/// so all three are tried before giving up. A present-but-blank value falls through.
fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            ["model name", "Hardware", "Processor"]
                .iter()
                .find_map(|key| {
                    text.lines()
                        .find(|l| l.starts_with(key))
                        .and_then(|l| l.split(':').nth(1))
                        .map(|s| s.trim().to_owned())
                        .filter(|s| !s.is_empty())
                })
        })
        .unwrap_or_else(|| "unknown CPU".to_owned())
}

/// Derived, never asserted: this binary is also the artefact the MVP-022 device run ships to
/// hardware, so a hard-coded platform name would make the device evidence self-contradicting.
fn host_label() -> String {
    format!(
        "{}/{} {}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        cpu_model()
    )
}

fn main() {
    println!(
        "host: {} — wall times are host context; states_scored and Score are \
         hardware-independent",
        host_label()
    );
    println!("verdict reference: shipped B=8 K=12; comparison is lexicographic Score");
    let mut adverse: Vec<String> = Vec::new();
    let mut headroom: Vec<String> = Vec::new();
    for f in fixtures::all() {
        let cap = feasible_cap(&f);
        let reference = run_cell(&f, 8, 12);
        let wide_open = run_cell(&f, 64, cap);
        println!("\n== {} (feasible cap K={cap}) ==", f.name);
        println!("  B   K     states  score                                med_ms");
        let mut cells = vec![wide_open];
        if SEARCH_HEAVY.contains(&f.name) {
            for b in EXPLORATORY_B {
                for k in EXPLORATORY_K {
                    if (b, k) == (8, 12) {
                        continue;
                    }
                    cells.push(run_cell(&f, b, k));
                }
            }
        }
        let print_cell = |c: &Cell, tag: &str| {
            println!(
                "  {:>3} {:>3} {:>8}  {:<36} {:>7.2}{tag}",
                c.beam_width,
                c.candidates_per_slot,
                c.states_scored,
                format!("{:?}", c.score),
                c.median_ms,
            );
        };
        print_cell(&reference, "  <- shipped");
        for c in &cells {
            let tag = match first_diff(&c.score, &reference.score) {
                Some(i) if c.score[i] > reference.score[i] => {
                    let line = format!(
                        "{}: B={} K={} score {:?} beats shipped {:?} at index {i}",
                        f.name, c.beam_width, c.candidates_per_slot, c.score, reference.score
                    );
                    if i <= 2 {
                        adverse.push(line);
                        "  <- STRICTLY BETTER (tier 0-2: ADVERSE)"
                    } else {
                        headroom.push(line);
                        "  <- strictly better (tier 3-5: headroom)"
                    }
                }
                Some(_) => "",
                None => "  (= shipped score)",
            };
            print_cell(c, tag);
        }
    }
    println!("\n== verdict ==");
    if adverse.is_empty() && headroom.is_empty() {
        println!(
            "B=8/K=12 SURVIVES: no computed cell achieved a strictly better Score on any fixture."
        );
    }
    for line in &headroom {
        println!("headroom: {line}");
    }
    for line in &adverse {
        println!("ADVERSE: {line}");
    }
    if !adverse.is_empty() {
        println!("verdict: ADVERSE — first differing Score index in 0-2; MVP-023 handback");
    } else if !headroom.is_empty() {
        println!("verdict: B=8/K=12 SURVIVES with recorded tail headroom (indices 3-5)");
    }
}
