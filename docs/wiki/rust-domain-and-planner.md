# Rust domain and planner

## Responsibility

The Rust core defines validated household and food concepts, evaluates policy and evidence, derives shopping requirements, and produces reproducible whole-cycle plans. It is independent of Flutter and does not read a clock, filesystem path, or network service.

## Household control kernel

`household-core/src/lib.rs` provides distinct validated IDs for households and members plus identity aggregates. `kernel.rs` adds domain-neutral controller concepts: policies, evidence sources, outcome assessments and statuses, reason codes, action proposals, attention requests, authority/reversibility/confidence bands, and immutable ledger entries.

The `HouseholdController` trait is the narrow assessment contract. The kernel says how a controller reports status, evidence, proposals, and attention; it does not know recipes, meals, or SQLite.

## Food domain

`food-domain` models civil dates and planning cycles, recipe identities and structured ingredient lines, provenance and rights, restrictions, member preferences, planned meal occurrences/components, pantry marks, starter content, and conservative shopping quantities.

Constructors enforce nonblank IDs/text, valid rational quantities, compatible ranges and units, household ownership, cycle scope, and other invariants before values reach storage. Shopping aggregation combines only compatible known quantities. Ambiguous or incompatible contributions remain separate and retain their source explanations. Pantry marks subtract only explicit binary-have knowledge; they do not imply quantified stock.

Starter content is compiled from `content/starter_recipes.json` through typed parsing and validation. The optional `fixtures` feature exposes deterministic synthetic planner scenarios to tests and the beam-width example without adding fixtures to the shipped library.

## Planning snapshot and policies

`planner/snapshot.rs` is the complete immutable planner input: household, calculated date/slot horizon, existing occurrences, recipe views, restrictions, preferences, pantry, policies, and recent history. Canonical text/hash functions make the exact assessed state reproducible.

`FoodPolicies` interprets supported food policy records, including explicit hard vetoes and restrictions-reviewed evidence. Unknown policy types are surfaced as assumptions rather than silently applied.

## Deterministic planning pipeline

`candidates.rs` creates candidates from existing plans, recipes, starter content, dining out, sourced leftovers, and bounded fallback placeholders. Canonical ordering prevents map/set iteration from changing results.

`tier0.rs` rejects hard restriction conflicts, explicit vetoes, disabled slots, invalid leftover sources, and automation that would move a lock. A held locked meal remains visible with a reason when a current hard check would reject it; the planner never silently violates the user's earlier authority.

`score.rs` uses lexicographic tiers rather than one compensating number. Higher-order requirements dominate lower-order preferences, then stability, feasibility, variety, reuse, pantry benefit, and leftovers shape the result. This preserves the rule that a soft benefit cannot buy through a hard constraint.

`beam.rs` performs bounded whole-cycle beam search. Candidates are ranked deterministically per slot, leftovers receive a bounded exemption from truncation, whole partial plans are rescored, and ties break on canonical plan text. Search output records beam width, per-slot cap, slot order, and states scored.

`coverage.rs` grades each slot as unresolved, tentatively covered, covered, locked by user, intentionally open, or needing attention. It withholds claims when restrictions, preferences, prep windows, or held-over locks leave material uncertainty. `attention.rs` turns high-value uncertainty into bounded requests and creates one reversible apply-plan proposal with explicit authority and confidence.

`planner/mod.rs` assembles filtering, search, coverage, assumptions, reason codes, proposal/attention output, algorithm version, and canonical evidence. The same assessment rules evaluate existing stored state and proposed results.

## Application orchestration

`kimatta-application::cover_cycle` loads one storage snapshot, assesses prior state, plans, appends a proposal/declined/apply ledger row, and conditionally applies unlocked changes. It accepts an ID-minting callback so UUID policy stays outside deterministic logic.

`record_decision` handles swap, hard-veto, and restrictions-reviewed corrections. It validates decision scope, protects locked slots, performs the mutation, reassesses, and appends a correction entry in one immediate transaction. Evidence names the ledger sequence being corrected.

## Verification seams

Planner unit tests cover each stage; invariant tests exercise household isolation, hard constraints, determinism, lock preservation, ledger immutability, and claim wording. Fixture documentation records scenario intent. The `beam_width` example measures bounded search alternatives using compiled fixtures.

## Coverage evidence

This page owns 27 files across `household-core`, `food-domain`, and `kimatta-application`, including their manifests, source, embedded starter content, fixtures, tests, and benchmark. Exact fingerprints and public declarations are in `../coverage-manifest.json`.

