# MVP-032 — Starter content II: federal public-domain sources

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation/content |
| Workstream | Recipes/content |
| Depends on | MVP-011, MVP-009, MVP-015 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` (no `--auto`); `grill-me` first if the owner wants to challenge the rubric |
| External actions | Read-only web search and page reads of US federal government recipe sites for candidate research. No scraping tools, no partner/state/university content, no licensed purchases. The owner selects the shipped set at the Phase 0 stop |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-032`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

A fresh install has at least ten recipes in its library on the first launch, so Cover My Week has something to cover. Today `rust/crates/food-domain/content/starter_recipes.json` holds ten authored recipes, all `cook_review: null`, and `shipped_starter_content()` ships none of them; the app is empty by its own rule (`KNOWN_ISSUES.md`, MVP-011 AC-3 entry, OPEN since 2026-08-29). This card fills the library from sources that are public domain and already published by a federal kitchen, and keeps the owner-cook rule for everything else.

## Authoritative sources

- `docs/PRD_v3.md` §15 (quick path to starter meals), §16 (a "very broad starter pack" is demoted, not required), principle 7 (explicit provenance where automation depends on it)
- `docs/task/mvp/MVP-011_STARTER_CONTENT_READINESS.md` — rights model, rubric precedent, AC-3 as written
- `docs/ROADMAP.md` D-014 (narrow-rights hybrid: original + verified federal/public-domain/CC0), D-041 (this card's promotion and the AC-3 supersession)
- `rust/crates/food-domain/src/starter.rs` — `RawProvenance`, `RawRights`, `RawCookReview`, `shipped_starter_content`; the content test suite
- `docs/task/MVP_INVARIANTS.md` 2, 3, 10, 12

## Load-bearing constraints

- **Phase 0 precedes every content edit.** No recipe is converted until the candidates document exists and the owner has marked selections in it. The plan must place the owner stop before any change under `rust/crates/food-domain/content/`.
- **Federal authorship, proven per recipe.** Aggregator sites (MyPlate Kitchen, SNAP-Ed Connection) republish partner recipes from state programs and universities; those are the partner's copyright. A candidate qualifies only when its source line names a federal agency (USDA CNPP/FNS, NIH/NHLBI, CDC or another federal body) and that page's URL and source line are captured verbatim in the candidates document. Anything with a partner or unnamed source is excluded, however good the recipe.
- **The shipped rule becomes two-armed.** An entry ships when it has a recorded `cook_review`, **or** when `rights.basis == "us_federal_public_domain"` and `provenance.source_url` is present. `original` entries still need a human cook. This is D-041's supersession of MVP-011 AC-3's wording; the rationale it rests on is federal publisher standing, and the card records what the source sites themselves say about testing rather than asserting it.
- **Units stay in the allowed set** (`cup`, `g`, `l`, `ml`, `piece`, `tbsp`, `tsp`). US recipes in oz/lb are converted at authoring; the unit set is MVP-015's and is not extended here.
- **Restriction conflicts are declared honestly.** `expected_conflicts` records what rule version 1 of the matcher returns, including known false positives and misses (the soy-sauce/gluten precedent in the file's policy notes). Renaming an ingredient to dodge the matcher is not allowed; a matcher change is MVP-009 work and out of scope.
- **Instructions are written functionally in the project's words** even for public-domain text, so the file keeps one authorship posture and no third-party prose is reproduced.
- Owner's own household recipes are welcome but optional: they enter as `original` entries with a truthful `cook_review`, at any time, as in-scope evidence additions (D-031 second sentence). They are not required for Done.

## Scope

**Phase 0 — research and owner selection**
- Search federal recipe sources; the starting list is MyPlate Kitchen, SNAP-Ed Connection, NIH/NHLBI heart-healthy recipes, and any other `.gov` recipe collection the search surfaces. Read each candidate's page; capture URL, source line, servings, prep/cook time, ingredient count, and any statement the site makes about recipe testing.
- Write `docs/research/starter-candidates.md`: one row per candidate, roughly 25–30 rows, with the rubric columns — federal author verified (yes/no + source line), servings ≥4, prep ≤30 min, ≤10 ingredient lines, common-pantry ingredients, restriction kinds the dish triggers (vegan / vegetarian / gluten / dairy / nuts / fish / shellfish / soy / eggs / sesame), catalog additions needed, unit conversions needed, notes. Add an empty `owner decision` column and a summary line showing how many candidates cover each of vegan, vegetarian, gluten-free and omnivore.
- **Stop.** The owner marks each row `select` or `reject`, dates the review, and may append own-recipe rows. Execution resumes only after that.

**Phase 1 — conversion and rule change**
- Convert selected rows to `starter_recipes.json` entries: `provenance.kind: "starter"`, `source_url`, `source_name` = the agency, `source_author` as shown, `rights.basis: "us_federal_public_domain"`, `attribution` = the captured source line, `verified_on` = the read date, `cook_review: null`.
- Extend the ingredient catalog for new ingredients; keep ids unique and every reference resolving.
- Change `shipped_starter_content()` (and the split behind `install_starter_content`'s report) to the two-armed rule; update the `StarterInstallReportDto` field docs so `pending_cook_review` counts only entries that ship by neither arm.
- Run the content suite; record `expected_conflicts` from the matcher's actual output and have the owner eyeball the roster.
- Update the `KNOWN_ISSUES.md` MVP-011 AC-3 entry: resolved for federal entries, still open for `original` ones, pointing here.

## Non-goals

- Recipe photos (`MVP-010` is Cut), CC BY or partner content, matcher rule changes, unit-set changes, an in-app import path (`OPT-001`, `OPT-002`), and any change to how the installer runs (every launch, idempotent — already correct).

## Decision gates

- If fewer than ten candidates pass the rubric with federal authorship proven, stop at the Phase 0 report. The owner decides between adding own recipes, lowering a rubric threshold other than authorship, or accepting fewer than ten. Authorship proof is never the threshold lowered.
- If a source site explicitly states its recipes are *not* tested, the owner decides at the stop whether that source still qualifies; the D-041 row is amended to say so.

## Acceptance criteria

- **AC-1:** `docs/research/starter-candidates.md` exists with every rubric column filled for every row and a per-row source URL and source line; the owner's dated `select`/`reject` marks are recorded in it before any content change.
- **AC-2:** Every shipped federal entry names a federal agency in `source_name`, carries `source_url` and the captured attribution line, and no shipped entry derives from partner, state or university content.
- **AC-3:** `shipped_starter_content()` ships an entry with a `cook_review`, ships an entry with `us_federal_public_domain` + `source_url`, and refuses an `original` entry without a review and a federal entry without a URL — one test per arm.
- **AC-4:** At least ten entries ship; across them at least one vegan, one vegetarian, one gluten-free and one omnivore dish; every content test passes (`every_slug_is_unique`, `every_catalog_reference_resolves`, `every_recipe_matches_its_declared_conflicts`, `shipped_content_excludes_unreviewed_entries` updated for the new rule).
- **AC-5:** A fresh install (`pm clear` then launch, airplane mode) shows the shipped recipes in the Recipes tab; an existing install gains them on its next launch with no reinstall or clear.
- **AC-6:** `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `flutter analyze` and `flutter test` pass; `flutter_rust_bridge_codegen generate` re-run if the report DTO changed.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | The candidates document itself, with the owner's dated marks |
| AC-2 | Content test asserting `source_name` and `source_url` non-empty for every federal entry; a fresh-context rights review of the shipped set against the source lines |
| AC-3 | Unit tests in `starter.rs`, one per arm |
| AC-4 | `cargo test` output; a count and restriction-coverage table in the handoff |
| AC-5 | Emulator run after `tools/emulator.sh reset`; a second run on an already-installed build showing the new count |
| AC-6 | Command output in the handoff |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- Stop if any candidate's federal authorship cannot be shown from the page itself.
- Stop at the end of Phase 0 until the owner's marks exist. Do not select on the owner's behalf.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED evidence, the shipped count and restriction coverage, any D-041 amendment, blockers, and **Next implementation task** (`MVP-033` when this card is Done). Select a dependent next only when this card is `Done` and that dependent passes its planning gate.
