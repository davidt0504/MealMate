# Recipe importer feasibility and implementation planning

**Status:** Proposed execution plan. This plan conducts bounded research and produces an importer implementation design whose open product forks are laid out for the owner to decide. It does not implement the importer or change roadmap/card status.

## Goal and outcome

Determine how reliably a simple website-link importer can produce useful, safe recipe drafts, including component recipes, then write an implementation design for OPT-001 that fully specifies settled behavior and presents each open product fork as options with a recommendation. The implementation plan is a design deliverable only; executing it requires OPT-001 to pass the repository's task-selection gate and a separately approved implementation plan.

Target experience: paste a public recipe URL or share it to Kimatta; Kimatta fetches it, presents one editable review, then saves privately after confirmation. Screenshot/PDF import, AI, accounts, and ongoing source synchronization remain out of scope.

When the main recipe is usable but a required component cannot be resolved safely, retain the original ingredient text and link, show one consolidated notice, and let the user edit or save the draft. Never silently omit an ingredient or block saving solely because a component could not be imported.

## Repository baseline and constraints

- OPT-001 is Draft and depends on MVP-008. OPT-007 is Verify; no next implementation task is selected. Apply `docs/task/README.md`'s mandatory gate before any later OPT-001 implementation planning or execution; this research does not change the roadmap.
- OPT-001 currently limits extraction to structured metadata and disallows general scraping. The prior experiment needed visible-page HTML to find RecipeTin Eats' linked sauce. Any proposed DOM-assisted support must be called out as an explicit scope change, not slipped into the existing card.
- The current Rust `Recipe` has flat ingredient lines, one instruction string, and recipe-level provenance. Existing planned meals support multiple components, but recipes do not currently persist component boundaries. Any proposed durable representation must account for storage migration, bridge DTOs, editing, and database export/restore.
- Preserve unrelated working-tree changes, especially `KNOWN_ISSUES-low.md` and `KNOWN_ISSUES-low.archive.md`. The repository is public and raw publisher pages are third-party content. Keep the run directory (captures, child pages, scripts, fixtures, outputs) outside the repository at `~/.codex/personal-workflows/pilot/research/meal-mate-recipe-import-readiness-<YYYY-MM-DD>/`, next to the prior runs. In the repository, write only `docs/research/recipe-import-readiness-report.md` and `docs/research/recipe-import-implementation-plan.md`, citing real pages by URL, SHA-256 and short excerpts, never full page bodies. Read the prior run directories in place and never modify them. Write nowhere else outside the repository, except the temporary files `tools/emulator.sh` writes.
- No app code, manifest, database, user recipe, task card, roadmap status, commit, or push is in scope.

## Execution steps

### 1. Establish an isolated run

Record current git status, roadmap/card status, tool versions, network availability, and emulator availability. Read the prior run, read-only: `~/.codex/personal-workflows/pilot/research/meal-mate-recipe-import-2026-10-01/` (REPORT.md, probe.py) and `~/.codex/personal-workflows/pilot/research/meal-mate-import-component-access-test-2026-10-01/` (REPORT.md, capture.py, classify.py, freeze_expectations.py, expected.json, runs/expectations.sha256, fixtures/, test_classify.py). The prior red-team findings are in `~/.codex/personal-workflows/pilot/plans/meal-mate-import-component-access-test.md`. Create the new run directory named above and copy the prior test directory's *contents* into it, so the scripts' own folder is the run directory (every script resolves paths from `ROOT=pathlib.Path(__file__).parent`). In the copy, `fixtures/0-6.html`, the nine synthetic fixtures, `runs/host-*.json`, `expected.json` and `runs/expectations.sha256` are the read-only baseline: never overwrite them. Do not re-run the copied `capture.py`, `freeze_expectations.py` or `test_classify.py` unchanged, because all three write over copied files. Change `capture.py` to take its allowlist from `urls.txt`, keeping the 7 prior URLs first and in their original order so `URLS[i]` still matches the baseline files. Make it write re-fetches of old URLs under `refetch/` and new cases under `new/`, at both of its host-JSON write sites (the fetch path and the subprocess error handler). Freeze new real-case and new synthetic-fixture expectations into `expected-new.json` with `runs/expectations-new.sha256`, using a changed freeze script that never writes `expected.json`. Change `test_classify.py` so it keeps the frozen-hash check and scoring of the old `expected.json`, also scores `expected-new.json` against the `new/` captures and new fixtures under a second hash guard, and writes its results to new output names. If direct fetching or writing the run directory is unavailable, record the limitation and continue with source inspection only; do not report a web-viewer result as a direct-fetch pass.

### 2. Freeze real-site expectations

Use the five existing root URLs, including Allrecipes, and the previously used RecipeTin Eats sauce and flatbread URLs. Add these three manually reviewed cases from different publishers:

- Love & Lemons pesto gnocchi with homemade pesto as an optional component and a reduced oil amount.
- Serious Eats chicken Parmesan sandwiches, with linked cooked chicken as a batch component and purchased sauce allowed by the notes.
- ThermoWorks Eggs Benedict, where Hollandaise is already included inline and external recipe links include attribution/references.

Before freezing anything, pin the exact root and child URLs for every new case in the run directory's `urls.txt` and fetch each once with the research user agent (`KimattaImportFeasibility/0.1`). Serious Eats returned 403 on 2026-10-02. Replace it, and any other blocked case, with a reachable recipe whose *required* component is a linked recipe. WP Recipe Maker markup is confirmed in the RecipeTin Eats and Budget Bytes captures; check it for each new captured page. The prior sample already has non-WPRM pages (King Arthur, BBC Good Food; Allrecipes was blocked) but no linked component on one. So at least one new *linked-component* case must be on a site that does not use WPRM. If none can be found and reached, record that as a measured negative for Open decision 1 rather than counting the slot as filled. Take each case's facts (alternatives, reduced amounts, notes) from the captured page, not from this plan's descriptions. Report results per recipe plugin as well as per publisher.

Fetch only explicitly selected root and directly relevant child pages; do not crawl recommendations, attribution, or arbitrary links. Freeze expected links, ingredient occurrences, source context, notes, alternatives, quantities, and inline components before running revised classification. Include wife-provided URLs unchanged if available; their absence does not block this run. Keep inaccessible pages in the access denominator and give them no fabricated classification result. Score classifier revisions against the prior run's archived captures plus the new cases' captures. Re-fetch old URLs only to report access and hash drift, and never re-freeze expectations for old captures. Freeze new-case expectations in a separate file with its own SHA-256 guard.

### 3. Repair component evidence and assembly rules

Use the prior `classify.py` (the copy in the run directory) only as a starting point. Keep URL fetch caching separate from per-ingredient-use assembly. Normalize HTML entities for matching while retaining original source text. Represent every candidate with its evidence, target, associated ingredient occurrence, quantity relationship, and confidence/unknown reason.

Apply these rules:

- The word “homemade” alone does not make an ingredient optional. An alternative must be explicitly offered by the source.
- Alternatives may appear in an ingredient or a note demonstrably associated with that component. If the note conflicts with the default ingredient and the relationship is unclear, abstain from expansion.
- A substitution does not erase the default component relationship. If a source offers a purchased sauce as an alternative, preserve the option and do not force the homemade recipe.
- Detect child recipe instructions or ingredients already included in the parent before appending them. Do not fetch/expand an inline component a second time.
- Fetch one URL once, but retain each separate parent ingredient occurrence. Two uses of one recipe with different quantities produce two scaled uses, not one collapsed placeholder.
- Expand only when the relationship, amount, child data, and source association are clear. Unknown yield never justifies inferring servings. Unsupported scaling, conflicting notes, missing child content, multiple ambiguous Recipe records, and deeper dependencies remain unresolved.
- If any component is unresolved, preserve its original ingredient and link, retain the usable rest of the recipe, show one consolidated review notice, and permit editing/saving.

Reuse the prior run's fixtures with their frozen labels: conflicting_optional, duplicate, half_batch, measured_missing_yield, misleading_placement, missing_href, multiple_recipes, nested, same_name_unrelated. Before changing the classifier, add fixtures only for the missing cases: required homemade sauce; purchased alternative only in notes; inline child instructions; two uses of one URL with different quantities; changed component amounts; incomplete child. Freeze expectations independently of the classifier. Keep synthetic and real-site results separate. Allow at most two fix/reverify cycles; report unresolved design limits after that.

### 4. Validate draft completeness and downstream implications

For every successfully fetched case, compare exact ingredient membership and wording, component occurrence and scale, original and child sources, alternatives, notes, and ordered instruction groups. Check that no ingredient disappears, duplicates, or gets merged across separate uses. Test the unresolved-component fallback: parent draft remains useful, unresolved text/link remain intact, one notice is produced, and save remains allowed.

Use the shawarma plus yogurt sauce as the exact 1x baseline: 18 parent ingredient lines minus the sauce placeholder plus seven child lines yields 24 lines, with every unaffected parent line unchanged. Keep flatbread unexpanded because it is optional. Do not equate that line-count check with a validated shopping list.

Produce representative draft-to-plan-to-shopping examples for: fully expanded components, inline components, optional purchased alternatives, and unresolved components. Use repository behavior/tests to determine whether a preserved unresolved line remains conservatively visible in shopping output. Record any required product behavior change; do not implement it here. No quantity parser or catalog matcher exists. Unless the design says otherwise, every imported line enters as `ingredient: None`, `QuantityDto::Unknown`, `UnitDto::None` with its original text. Label hand-built examples as simulated. Only output from a repository test that was actually run counts as verified (unresolved `ingredient: None` lines are exercised in `rust/src/api/shopping.rs` tests). The parser/matcher gap is Open decision 4.

### 5. Recheck Android access

Before any startup attempt, list emulator/qemu processes on the Windows host (`tasklist.exe`). On 2026-10-02, `emulator.exe` and `qemu-system-x86_64.exe` were still running from the prior run, and `tools/emulator.sh up` only waits on such processes. If they are present and no device is ready, ask the owner to end them (`taskkill /IM qemu-system-x86_64.exe /F`) or skip the attempt, and record Android NOT VERIFIED with that reason.

Check for a ready emulator. If none is available, make one ordinary startup attempt using `tools/emulator.sh up`, wait only within its documented boot timeout, and make no reset, reboot, bridge repair, or configuration change. Never select a physical phone implicitly. If unavailable, record Android NOT VERIFIED and continue the host-side work.

If a device is available, run a bounded network probe against the same five roots and record device/API identity, response/final URL, byte count, duration, error, and body hash. A shell-identity probe is only an environment signal; it does not prove Kimatta's app client can fetch. Do not install an APK or alter the manifest for this experiment. The future implementation plan must separately cover release INTERNET permission and app-level networking tests.

### 6. Produce the report and importer design

Write a report containing the frozen sample, direct-fetch outcomes, denominator, expected-versus-actual confusion counts, false expansions, missed dependencies, abstentions, assembly checks, downstream checks, Android status, limitations, and exact reproduction steps. Distinguish real pages from authored fixtures and inaccessible pages. Make no user-base success-rate claim from this sample.

Write `docs/research/recipe-import-implementation-plan.md` as the importer design. Specify settled behavior fully. For each open product fork, give the options, the measured evidence, the trade-offs and a recommendation, and leave the choice to the owner. For each open decision, fully specify the recommended option and give the other options with their deltas. It must specify:

- Paste and Android Share inputs converging on one review flow; URL fetching stays on device.
- Deterministic JSON-LD extraction plus the DOM-assistance options under Open decision 1, with the supported boundary stated for each and honest unsupported-page behavior.
- Component association evidence, inline detection, alternatives/notes treatment, duplicate-use handling, quantity scaling, unresolved-component behavior, and one-child-depth policy.
- Draft boundaries and explicit APIs/data flow between fetch, extraction, resolution, user edits, validation, and atomic save.
- A durable representation that preserves component boundaries and source/scale data while keeping ordinary planning/shopping behavior. Specify the actual Rust domain, storage migration, FRB bridge DTO, Dart editing, database export/restore compatibility, old-row defaults, transaction/rollback, and delete/edit semantics against inspected code. Recommend recipe-owned metadata over a shared synchronized component graph as the default for Open decision 2, specified fully as above, with the other options' migration/FRB/export deltas.
- One consolidated notice for unresolved components; original text and link survive review/edit/save, and user edits are never silently overwritten. Saving the useful main recipe remains allowed.
- URL security: scheme and credential rejection, each redirect revalidation, public destination/SSRF protections, time/body limits, bounded child fetches, safe HTML/JSON-LD parsing, and content-free diagnostics. Do not bypass authentication, CAPTCHAs, or publisher denials.
- Android INTERNET permission in the release manifest; `ACTION_SEND` plain-text URL receiving for cold and warm app starts; paste/share convergence, repeated shares, cancellation, offline/timeouts, blocked destinations, malformed content, duplicate saved URL behavior per Open decision 3 (recommend explicit “import another copy”).
- Tests at parser, component resolver, storage/bridge round-trip, migration/export-restore, review UI, app networking/share, and recipe-to-shopping levels. Include standard, edge, and adversarial tests for the prior HIGH findings and standard/edge tests for sample coverage.
- Exact acceptance criteria, verification commands/evidence, rollout/rollback boundaries, and explicit non-goals. Any behavior not decided by this plan must be surfaced as an owner decision before implementation, not left to implementer discretion.
- An "Open decisions" section covering at least: (1) extraction scope (JSON-LD only, a WPRM/plugin adapter, or generic DOM heuristics), set against OPT-001's no-general-scraping stop condition and backed by the per-plugin evidence from step 2; (2) durable component representation (recipe-owned metadata, flat lines with provenance, or a component graph) with migration, FRB, export/restore and old-row implications; (3) behavior for a duplicate saved URL; (4) how imported text lines become structured ingredient lines (quantity/unit parsing and catalog matching) versus entering unresolved.

The plan must explicitly say it is a proposed design artifact, not execution authorization. It becomes eligible for implementation only after the task gate selects OPT-001 as Ready/next and a separate implementation-plan approval is given.

### 7. Final scope and state check

Re-read both deliverables against this plan. Confirm every finding has evidence or is honestly unresolved, tests have not been described as passing unless run, and no new publisher source has been mislabeled as an automated import. Confirm git status shows no app/card/roadmap changes; preserve existing user changes. Leave any emulator running and record that fact.

## Finding ledger to carry into the report

| Red-team finding | Required evidence in this work |
|---|---|
| Research-to-implementation handoff | Separate implementation design with its open owner decisions listed, plus task-gate caveat |
| Required/optional misclassification | Homemade-required, note-only alternative, and conflicting-note regressions |
| Separate uses collapsed by URL | Same child URL referenced by two parent ingredient occurrences with different quantities; both preserved in assembly |
| No durable component persistence design | Explicit domain/storage/bridge/UI/export/restore/migration specification and round-trip test plan |
| Android access not established | Explicit host vs probe vs app evidence; release permission and real app-flow tests in implementation plan. If NOT VERIFIED, on-device fetching stays an open owner-visible risk; the design must not assume a device can fetch sites the host gets 403 from |
| Narrow sample and incomplete downstream checks | New cases, including a second real linked component and a linked component on a non-WPRM site (or a recorded measured negative), with per-plugin counts (a publisher count alone is not coverage); plan → shopping cases and omissions/duplication checks |

Finding 1 (prior findings: the plans file named in step 1) is a retracted defect in the prior research plan's scope: research correctly did not implement the app. The missing handoff to an eventual implementation plan is addressed here. Findings 2–6 remain unverified until the corresponding experiment evidence and plan deliverable exist.

## Completion criteria

- Every original and added URL has an explicit fetch result or NOT VERIFIED reason; failures remain in the denominator.
- Frozen expectations precede classifier revisions, and all requested counterexamples run as regressions.
- No false expansion is counted as acceptable; missed components, unknowns, partial drafts, and unresolved source notes are reported.
- Assembly and plan/shopping cases verify no missing or duplicated ingredient occurrences and preserve source wording.
- Android claims are correctly scoped to the evidence actually obtained.
- The two deliverables contain the experiment report and an OPT-001 implementation design whose open product forks are listed as owner decisions, subject only to the repository's separate task-selection/approval gate.
- No importer code, production data, task status, card, or roadmap state changes occur.
