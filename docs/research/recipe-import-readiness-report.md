# Recipe import readiness — experiment report (2026-10-02)

**Status:** Research result for `docs/research/recipe-import-readiness-plan.md`. No app code, card, roadmap, manifest or database changed. Companion design: `docs/research/recipe-import-implementation-plan.md`.

The run directory is outside the public repository at `~/.codex/personal-workflows/pilot/research/meal-mate-recipe-import-readiness-2026-10-02/`. It holds raw page captures, scripts, fixtures and outputs. This report cites pages by URL, SHA-256 and short excerpts only. Prior runs it builds on: `meal-mate-recipe-import-2026-10-01/` and `meal-mate-import-component-access-test-2026-10-01/`, both siblings under the same research root and read only. Prior red-team findings: `~/.codex/personal-workflows/pilot/plans/meal-mate-import-component-access-test.md`.

## Outcome in brief

- Every reachable page carried exactly one schema.org `Recipe` in JSON-LD: 12 of 13 URLs; the 13th, Allrecipes, is still 403. None of those JSON-LD records contains a component link. Finding a linked sub-recipe always needed the visible recipe-card HTML.
- The revised classifier, using the plugin adapter, finished the sample with:
  - **0 false automatic expansions** and **0 missed required components**, across 167 real anchors and 16 synthetic cases;
  - all 16 synthetic regressions passing, including the six new ones the review asked for.
- The sample now has **two real linked required components**:
  - RecipeTin Eats lemon yogurt sauce, on WP Recipe Maker (WPRM);
  - Sally's Baking Addiction pie crust, on Tasty Recipes, the non-WPRM case.

  The pie crust is offered as a choice of two child recipes with the amount given only in prose, so the correct behavior is to keep it unresolved, not expand it. Only one real component was automatically expandable, the same sauce as last time.
- A plugin-free "generic" locator also produced 0 false expansions. It labelled 8 non-recipe links (BBC Good Food glossary pages, King Arthur shop pages) as required components, and each of those would raise a needless "couldn't import" notice. That is measured evidence against generic DOM heuristics for Open decision 1.
- The Android emulator (Android 16 / API 36, shell identity) fetched the same five roots with the same results as the host: 4 × 200 and Allrecipes 403. This is not app-level evidence.

No success rate for users' recipes can be inferred from 7 publishers and 13 URLs.

## 1. Run baseline

| Item | Value |
|---|---|
| Repo HEAD | `a5b2382` |
| Working tree | only the pre-existing `KNOWN_ISSUES-low.md` / `KNOWN_ISSUES-low.archive.md` changes, plus the untracked plan |
| Tracked diff SHA-256 | `2a9a7ce8…b811b` (same at start and end) |
| Roadmap | OPT-001 Draft (`docs/ROADMAP.md:79`); OPT-007 Verify (`:86`); no next task (`:9`) |
| Repo tests before work | `cargo test` in `rust/` 149 passed; `flutter test` 476 passed (both exit 0) |
| Tools | Python 3.12.3, bs4 4.14.3, cargo 1.98.0, Flutter 3.47.1, javac 1.8 + JDK 17 for d8 |
| Emulator | `emulator.exe` + `qemu-system-x86_64.exe` running; `emulator-5554` already `device` (no startup attempt needed) |

Copying the prior test directory gave the run its read-only baseline. Its files were hashed into `runs/baseline-files.sha256` before any script ran, and checked after the capture, the freeze, the tests and the final pass. All checks report unchanged:
- `fixtures/0-6.html` and the nine synthetic fixtures;
- `runs/host-0..6.json`;
- `expected.json` and `runs/expectations.sha256`.

## 2. Frozen sample and direct-fetch results

**Fetch bounds:**
- user agent `KimattaImportFeasibility/0.1`, `Accept: text/html`;
- fixed allowlist from `urls.txt`, public-address check, redirect revalidation with a cap of 5;
- 25 s connect/read timeout, 60 s wall clock per URL, 3 MB body cap;
- no crawling.

| # | URL | Role | Plugin | Host | Bytes | SHA-256 (prefix) | Recipes | Ingr. |
|---|---|---|---|---|---:|---|---:|---:|
| 0 | budgetbytes.com/creamy-tomato-spinach-pasta/ | prior root | WPRM | 200 | 627,439 | `b0af1d9d` | 1 | 15 |
| 1 | allrecipes.com/recipe/21014/… | prior root | — | **403** | 680,341 | `c132f8fb` | 0 | — |
| 2 | kingarthurbaking.com/recipes/simply-perfect-pancakes-recipe | prior root | custom | 200 | 325,165 | `85456a5d` | 1 | 8 |
| 3 | recipetineats.com/chicken-sharwama-middle-eastern/ | prior root | WPRM | 200 | 490,600 | `b55611b2` | 1 | 18 |
| 4 | bbcgoodfood.com/recipes/easy-chicken-curry | prior root | custom | 200 | 573,510 | `1f41c7a0` | 1 | 11 |
| 5 | recipetineats.com/lemon-yogurt-sauce/ | prior child | WPRM | 200 | 449,689 | `ec486d4a` | 1 | 7 |
| 6 | recipetineats.com/easy-soft-flatbread-yeast/ | prior child | WPRM | 200 | 459,206 | `5cd86a86` | 1 | 4 |
| 7 | loveandlemons.com/pesto-gnocchi/ | new root | WPRM | 200 | 684,425 | `f513aba4` | 1 | 8 |
| 8 | loveandlemons.com/pesto-recipe/ | new child | WPRM | 200 | 698,044 | `9dd82516` | 1 | 8 |
| 9 | blog.thermoworks.com/eggs-benedict-for-christmas-morning/ | new root | WPRM | 200 | 477,217 | `82f8226b` | 1 | 12 |
| 10 | sallysbakingaddiction.com/apple-pie-recipe/ | new root | **Tasty Recipes** | 200 | 629,265 | `f4962802` | 1 | 10 |
| 11 | sallysbakingaddiction.com/baking-basics-homemade-buttery-flaky-pie-crust/ | new child | Tasty Recipes | 200 | 598,434 | `5c7b4397` | 1 | 5 |
| 12 | sallysbakingaddiction.com/all-butter-pie-crust/ | new child | Tasty Recipes | 200 | 612,530 | `ebf022ba` | 1 | 5 |

Full hashes are in `refetch/host-*.json` and `new/host-*.json`.

**Access denominator:** 13 URLs; 12 fetched with 200 and 1 failure (Allrecipes 403, unchanged since 2026-10-01).

**Plugins among the 12 pages with a recipe:**
- WPRM: 7 pages from 4 publishers;
- Tasty Recipes: 3 pages from 1 publisher;
- custom: 2 pages.

**Re-fetch drift on the 7 prior URLs:**
- every page's bytes changed;
- the JSON-LD ingredient lists, structured instructions and recipe-card anchor lists are identical to the archived captures.

Scoring used the archived bytes (`fixtures/0-6.html`), never the re-fetches. Prior expectations were not re-frozen.

### Case selection and swaps

| Planned case | Result |
|---|---|
| Love & Lemons pesto gnocchi | Kept. Captured line: "½ cup store-bought pesto (or homemade pesto made with ⅓ cup olive oil)". Purchased pesto is the default, homemade is the linked alternative, and the parent changes the child's oil (child: "¼ cup extra-virgin olive oil (plus more…)"). Gnocchi is also "store-bought or homemade gnocchi" with a link. |
| Serious Eats chicken Parmesan sandwiches | **403** (`seriouseats.com/chicken-parmesan-sandwiches`) on 2026-10-02, so it was replaced. |
| ThermoWorks Eggs Benedict | Kept. Captured line "1 batch Hollandaise sauce (see below)" plus six inline sauce lines. No Hollandaise link exists; the card's external links are credits ("with thanks to J. Kenji López-Alt at SeriousEats.com…"), tools and images. |
| Replacement, non-WPRM linked component | Sally's apple pie (Tasty Recipes). Captured line: "Homemade Pie Crust or All Butter Pie Crust (both recipes make 2 crusts, 1 for bottom and 1 for top)", where both names link to child recipes. |

Screening fetches of rejected candidates (status only, bodies discarded):
- Pinch of Yum: 403.
- Two Sally URLs: 404.
- One Sally URL redirected to a different recipe.
- King Arthur Dutch apple pie: its only ingredient links are shop pages.

No wife-provided URLs were available; their absence did not block the run.

## 3. Frozen expectations

- `expected.json`, the prior baseline: 67 real anchors and 9 synthetic cases. SHA-256 `e22896f1…dd73c37`, guarded by `runs/expectations.sha256`.
- `expected-new.json`, new for this run, frozen before any classifier change: 100 real anchors (every recipe-card anchor on pages 7–12) and 6 synthetic cases (7 anchors). SHA-256 `10e5be48…e7883a`, guarded by `runs/expectations-new.sha256`.

New real labels:
- 2 required (Sally crust ×2);
- 2 optional (L&L gnocchi, L&L pesto);
- 96 unrelated.

New synthetic cases:
- required homemade;
- purchased alternative only in notes;
- inline child;
- two uses at 1 and ½ batch;
- changed component amounts;
- incomplete child.

## 4. Classification results

Two fix/reverify cycles were used, which is the limit:
- **Cycle 0** is the prior `classify.py`, unchanged.
- **Cycle 1** found one regression: the shawarma sauce link inside an instruction step lost its inherited required label because I had narrowed the duplicate-reference rule.
- **Cycle 2** restored that rule.

Remaining errors are reported below; they were not tuned away.

### Cycle 0: prior classifier against new cases (fail-first evidence)

| Set | n | Exact | False expansions | Missed required | Abstained / not found |
|---|---:|---:|---:|---:|---:|
| Real, prior pages | 67 | 50 | 0 | 0 | 16 |
| Real, new pages | 100 | 12 | 0 | **2** | 81 (63 not found) |
| Synthetic, prior | 9 | 9 | 0 | 0 | 2 |
| Synthetic, new | 7 | 5 | **1** | 1 | 0 |

The prior classifier failed in four ways:
- It never looks inside Tasty Recipes cards, so both required pie-crust anchors went undiscovered.
- It called "homemade" optional by itself.
- It expanded "1 batch Green dressing (made with half the yogurt)" at 1x, a **false expansion**.
- It labelled 7 affiliate/product links inside ingredient lines as required components (no expansion, but wrong).

### Cycle 2: revised classifier

| Set | Locator | n | Exact | False exp. | Missed req. | Abstained |
|---|---|---:|---:|---:|---:|---:|
| Real, prior pages | adapter | 67 | 54 | 0 | 0 | 12 |
| Real, new pages | adapter | 100 | 80 | 0 | 0 | 18 |
| Synthetic (16) | adapter | 16 | 16 | 0 | 0 | 2 (expected `unknown`) |
| Real, prior pages | generic | 67 | 44 | 0 | 0 | 11 |
| Real, new pages | generic | 100 | 75 | 0 | 0 | 21 |

**Confusion counts, adapter, all 167 real anchors** (rows are expected, columns are actual):

| | required | optional | unrelated | unknown |
|---|---:|---:|---:|---:|
| required (4) | 4 | 0 | 0 | 0 |
| optional (6) | 0 | 6 | 0 | 0 |
| unrelated (157) | 0 | 3 | 124 | 30 |

**Per plugin (real anchors):**

| Plugin | Locator | n | Exact | Mislabelled as required/optional | Unknown |
|---|---|---:|---:|---:|---:|
| WPRM (7 pages) | adapter | 88 | 76 | 0 | 12 |
| Tasty Recipes (3 pages) | adapter | 65 | 56 | 2 optional | 7 |
| custom (King Arthur, BBC) | adapter | 14 | 2 | 1 optional | 11 |
| WPRM | generic | 88 | 65 | 2 optional (and 2 true optionals missed: 1 unknown, 1 unrelated) | 19 |
| Tasty Recipes | generic | 65 | 52 | 4 optional | 9 |
| custom | generic | 14 | 2 | **8 required**, 1 optional | 3 |

### Rule changes, each covered by a regression

- **Homemade alone is not optional.** An alternative must be offered explicitly ("or", "optional", "store-bought").
- **Two or more linked child recipes in one ingredient line form a required choice.** The user picks the child, so there is no expansion. This is the Sally case.
- **The parent modifying a component's amounts withholds expansion.** Examples: "made with half the yogurt", "⅓ cup olive oil".
- **A purchased alternative in a note is kept** as an alternative without erasing the default relationship.
- **Cross-site links are not component candidates.** These are product, credit and tool links. Site comparison is a naive last-two-labels check.
- **A link only inside a parenthetical abstains.** Example: "(spooned & leveled)".
- **Line matching ignores spacing, punctuation and HTML entities.** For example, JSON-LD "&amp;" matches the card's "&".

### Remaining mislabels and abstentions (not tuned)

- **Adapter optional_alternative where "unrelated" was expected (3):**
  - King Arthur shop link in "…or 1/4 cup malted milk powder" (same as 2026-10-01);
  - Sally "lattice pie crust tutorial" and "crimp or flute" technique links in instruction text containing "or".

  None of these is expandable.
- **Unknown where "unrelated" was expected (30):**
  - instruction-step tool and image links (ThermoWorks);
  - BBC glossary links;
  - author/credit links;
  - parenthetical technique links.

  Each is an abstention: the link stays inside the original text and is not followed.
- **Same-site non-recipe links are the main error source.** Examples are BBC `/glossary/onion-glossary` and King Arthur `shop.` pages. Nothing in the markup separates them from a sub-recipe link. Only fetching the target and finding a `Recipe` record would.

## 5. Assembly and draft completeness

These are real-page drafts built from the captured JSON-LD (`runs/assembled-*-readiness.json`):

| Case | Lines in → out | Expanded | Unresolved | Notice | Checked |
|---|---|---|---|---|---|
| Shawarma + lemon yogurt sauce | 18 → 24 | sauce at 1x | 0 | none | exact order: parent[:idx] + 7 child lines + parent[idx+1:]; placeholder removed once; flatbread stays a main line; child instructions in their own component |
| Shawarma, sauce page missing | 18 → 18 | — | 1 (link kept) | one | every parent line kept; save allowed |
| Shawarma at depth 1 | 18 → 18 | — | 1 "deeper dependency" | one | — |
| Love & Lemons pesto gnocchi | 8 → 8 | — | 0 | none | gnocchi and pesto links recorded as alternatives; nothing fetched |
| ThermoWorks Eggs Benedict | 12 → 12 | — | 0 | none | inline Hollandaise untouched; no child fetch |
| Sally apple pie | 10 → 10 | — | 1 (both crust links kept) | one | crust choice left to the user; nothing fetched |

**Synthetic assembly cases:**
- **Inline child:** not expanded a second time; 1 component.
- **Two uses:** the URL is fetched once, and both uses are kept. The 1x use expands; the ½ use stays unresolved because raw lines cannot be scaled.
- **Changed amounts:** unresolved.
- **Incomplete child:** unresolved; one notice; save allowed.

No ingredient line disappeared or was duplicated in any draft. Duplicate references to one URL within a single use produce one expansion.

**Not validated:** source-note fidelity, overall elapsed time (shawarma metadata says 20 minutes total against hours of marinating), and the coherence of interleaved instructions across components.

## 6. Downstream: plan → shopping

**Verified (repo tests run 2026-10-02, all pass)** — `cargo test --workspace` filtered to:
- `rust/src/api/shopping.rs`: `dto_mapping_round_trips_every_unit_and_quantity_shape`. This is the bridge round trip with an `ingredient: None`, `QuantityDto::Unknown`, `UnitDto::None` line. The line stays separate and `Needed`, is never pantry-marked, and its quantity is not scaled at plan scale 2.
- `rust/crates/food-domain/src/shopping.rs`:
  - `an_unresolved_line_whose_scale_fits_is_still_unresolved`
  - `unresolved_lines_never_merge_and_keep_original_text`
  - `an_unresolved_line_cannot_be_omitted`
  - `a_component_whose_recipe_is_missing_from_the_snapshot_is_reported_not_dropped`

**Simulated, not app output** (`simulate_shopping.py` → `runs/shopping-simulated.json`). The mapping assumption is that every imported line enters unresolved with its original text, as the design specifies unless Open decision 4 changes it.

| Example | Draft lines | Shopping lines | Observation |
|---|---:|---:|---|
| Fully expanded (shawarma + sauce) | 24 | 24 | Garlic and lemon appear twice, once per use; correct, but never merged, because unresolved lines don't merge |
| Inline (ThermoWorks) | 12 | 12 | **The placeholder "1 batch Hollandaise sauce (see below)" shows as a shopping line next to its own six ingredients** |
| Optional alternatives (L&L) | 8 | 8 | The purchased pesto default is listed; nothing homemade is forced |
| Unresolved (Sally) | 10 | 10 | **The crust line is listed, but crust flour/butter/shortening are not.** The user must pick a crust or shop from the line |

**Product behavior gaps found** (recorded here, not implemented):
1. Every imported ingredient lands in "Other" as a separate unresolved line, because no quantity parser or catalog matcher exists. That is Open decision 4.
2. Plan scaling does not apply to unresolved lines.
3. Inline-component placeholder lines need a review prompt or a "not a purchase" marker.
4. An unresolved required component hides its child's ingredients from shopping until the user resolves it.

## 7. Android

The pre-check found `emulator.exe` and `qemu-system-x86_64.exe` running, and `tools/emulator.sh adb devices -l` already listed `emulator-5554 device` (pre001_avd, `sdk_gphone64_x86_64`, Android 16, SDK 36). So no startup attempt, reset, reboot or bridge change was needed, and no owner action was requested.

The prior unverified probe (`android/RecipeAccessProbe.java`, Java `HttpURLConnection` with the same bounds and user agent) was built and run:
1. Built with `javac -source 8` and d8 36.0.0. d8 needs JDK ≥ 11, so JDK 17 was put on `PATH` for that one command only.
2. Pushed to `/data/local/tmp/kimatta-import-probe-20261002/`.
3. Run through `app_process`.
4. The remote directory was then removed (verified "CLEANED").

| Root | Android status | Bytes | Seconds | Recipes / ingredients (host-side parse of the Android body) |
|---|---:|---:|---:|---|
| Budget Bytes | 200 | 627,439 | 0.63 | 1 / 15 |
| Allrecipes | **403** | 680,362 | 0.55 | — |
| King Arthur | 200 | 325,165 | 0.41 | 1 / 8 |
| RecipeTin shawarma | 200 | 490,547 | 0.64 | 1 / 18 |
| BBC Good Food | 200 | 573,506 | 0.46 | 1 / 11 |

Each run wrote its result and then failed to exit; the 90 s `timeout` killed it (exit 124). The written JSON bodies hash-match their pulled HTML.

**Scope of this evidence:**
- It covers the shell UID and Java networking on an emulator whose traffic leaves through the same host network as the workstation. It is not Kimatta's app UID, its client, the release manifest (which has no `INTERNET` permission today) or a phone network.
- Allrecipes 403 on both paths, so the design must not assume a device can fetch what the host cannot.

The emulator was left running as found.

## 8. Finding ledger

| Red-team finding | Evidence now |
|---|---|
| Research-to-implementation handoff | `recipe-import-implementation-plan.md`: settled behavior specified, four open owner decisions, task-gate caveat |
| Required/optional misclassification | Fixtures `required_homemade`, `note_only_alternative` and prior `conflicting_optional` all pass. Cycle 0 showed the prior failure. |
| Separate uses collapsed by URL | `two_uses`: one fetch, two occurrences (1 and ½) kept separately in assembly |
| No durable component persistence design | Design §5 and Open decision 2, against inspected code; round-trip tests listed |
| Android access not established | Shell-identity emulator probe matches the host. App-level networking, release permission and share flow are NOT VERIFIED and are in the design's tests. |
| Narrow sample, incomplete downstream checks | A second real linked component on a non-WPRM site (Sally, Tasty Recipes). Per-plugin counts. Shopping examples (simulated) plus five repo tests (run). Only one real component is *expandable*; still one publisher for that. |

Finding 1 of the prior plan review was retracted there: research correctly did not implement the app. Findings 2–6 are addressed by the evidence above, within the limits stated.

## 9. Limitations

- 13 URLs, 7 publishers, 3 card formats. Two of the Tasty pages and both WPRM child pages are from the same publishers as their roots.
- Expectations and the classifier were written in the same session, though frozen first and hash-guarded. Synthetic fixtures are in-sample engineering checks, not websites.
- The adapter keys on WPRM and Tasty Recipes class names. That is plugin-specific DOM reading, which OPT-001 currently excludes (see Open decision 1).
- The site-identity check is a naive last-two-labels comparison.
- Raw-line assembly supports only an exact 1x batch.

## 10. Reproduction

From the run directory:

```sh
python3 capture.py                       # re-fetches all 13 URLs into refetch/ and new/ (never fixtures/ or runs/host-*)
python3 score.py classify_prior cycle0   # prior classifier
python3 score.py classify cycle2         # revised classifier, both locators
python3 -m unittest -v test_classify     # 20 checks; writes runs/*-readiness.json
python3 simulate_shopping.py             # simulated shopping examples
sha256sum -c runs/baseline-files.sha256  # archived baseline unchanged
```

`freeze_expectations.py` refuses to run once `expected-new.json` exists. Do not delete it to re-freeze against a changed page.

The test run on 2026-10-02 had 20 tests, all OK; the output is in `runs/verification-readiness.txt`. Repo shopping tests: `cd rust && cargo test --workspace -- unresolved dto_mapping_round_trips_every_unit_and_quantity_shape a_component_whose_recipe_is_missing`.
