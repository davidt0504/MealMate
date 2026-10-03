# Recipe URL import — implementation design (OPT-001)

**Status:** Proposed design artifact. It does not authorize execution. It can be implemented only after the task gate in `docs/task/README.md` selects OPT-001 as Ready and next, and a separate implementation plan is approved. As of 2026-10-02, OPT-001 is Draft (`docs/ROADMAP.md:79`) and OPT-007 occupies the lane (`:86`).

**Evidence:** `docs/research/recipe-import-readiness-report.md` (2026-10-02 run) and its two predecessor runs. Code facts were inspected at `a5b2382`.

**How to read this:** sections 1–9 are settled behavior, fully specified. Section 10 lists four open product forks. Each fork has a recommended option, and the main sections are written assuming the recommendation. The other options are given as deltas. An owner answer to each fork is required before implementation planning; nothing here is left to implementer discretion.

## 1. Outcome and non-goals

The user pastes a public recipe URL, or shares one to Kimatta from another app. Kimatta fetches the page on the phone and shows one editable review. Nothing is saved until the user confirms, and then it saves privately.

When the main recipe is usable but a required component cannot be resolved safely, the review:
- keeps the original ingredient text and link;
- shows one consolidated notice;
- still allows editing and saving.

An ingredient is never silently omitted.

**Non-goals:**
- screenshots, PDFs, OCR, AI/LLM extraction;
- accounts, backend or proxy fetching;
- ongoing source sync;
- crawling, browser automation, bypassing 403s, CAPTCHAs or logins;
- public sharing of imported content;
- components more than one level deep;
- scaling raw child lines other than an exact 1x batch.

## 2. Inputs converge on one flow

- **Paste.** A "Paste link" action on the recipe list opens an import sheet with a single URL field, pre-filled from the clipboard only if the clipboard holds an http(s) URL.
- **Android Share.** `ACTION_SEND` with `text/plain`.
  - Add an intent filter to `MainActivity` in the main manifest. Today the main manifest has only MAIN/LAUNCHER and a PROCESS_TEXT `<queries>` entry (`android/app/src/main/AndroidManifest.xml:29-50`).
  - A small platform channel in `MainActivity` hands the shared text to Dart. A cold start reads it via `getIntent()`; a warm start reads it via `onNewIntent()`. No new plugin is needed; `share_plus` (`pubspec.yaml:43`) only sends.
  - The first `http(s)://…` token in the shared text is the URL. Text with no URL shows "No recipe link found in what was shared".
- **Convergence.** Both inputs call one `ImportController.start(url)`, which runs the same fetch → extract → resolve → review pipeline.
- **Repeated shares.** A second share while a review is open asks "Replace the recipe you're reviewing?" and never overwrites unasked. A share during an active fetch cancels that fetch and starts the new one.
- **Cancellation.** Back or Cancel abandons the in-memory draft. Nothing is persisted before Save.

## 3. Fetch (Rust, on device)

**Network prerequisite.** Add `<uses-permission android:name="android.permission.INTERNET"/>` to the **main** manifest. Today it exists only in the debug and profile manifests (`android/app/src/debug/AndroidManifest.xml:6`, `profile/…:6`), so release builds cannot fetch at all.

**Where the code lives.** A new `recipe-import` crate in the Rust workspace, using a blocking HTTPS client with rustls (`ureq`). There is no HTTP dependency today, in Cargo or pubspec. Validation, fetching and parsing therefore sit in one crate tested with `cargo test`, and are exposed through one FRB function.

**URL security, applied to the input URL and to every redirect hop:**
- **Scheme:** `https` and `http` only. Reject credentials in the URL (`user:pass@`), non-default ports other than 80 and 443, and IP-literal hosts.
- **Address checks:** resolve DNS once per hop, reject any non-global address (loopback, private, link-local, CGNAT, multicast, reserved, IPv6 ULA), and connect to the validated address. Pinning the address prevents DNS-rebinding between check and connect.
- **Limits:**
  - 5 redirects at most;
  - 10 s connect, 15 s read and 30 s total per page;
  - 3 MB body cap, enforced while streaming;
  - `text/html` or `application/xhtml+xml` only.
- **Request headers:** a truthful `User-Agent: Kimatta/<version> (recipe import)` and `Accept: text/html`. No cookies or retries.
- **Refusals are final.** A 401, 403, 429 or 451 response, or a challenge page, ends the fetch with "This site didn't let Kimatta read the page". Kimatta never retries with a different identity. The emulator probe (report §7) showed Allrecipes 403 on device as on the host.
- **Diagnostics are content-free:** status code, failure class, byte count and duration. Page bodies and URLs are never logged.

**Child fetches:**
- Only for required components that are expandable (§4).
- At most 3 distinct child URLs per import, fetched once each, under the same rules.
- They must be **same-site** as the parent. Any https child under the same registrable domain qualifies, determined with the Public Suffix List rather than the research script's last-two-labels shortcut.
- One level deep only.

## 4. Extraction and component resolution

**JSON-LD (always).**
- Parse every `<script type="application/ld+json">` with a JSON parser. Never evaluate it.
- Walk the result, including `@graph` and nested lists, and collect objects whose `@type` includes `Recipe`.
- Exactly one Recipe → draft. Zero → "No recipe data found on this page" (unsupported). More than one → the user picks a title; components are not resolved.

**Field mapping:**

| JSON-LD | Draft | Notes |
|---|---|---|
| `name` | title | Required. If missing, the page is treated as unsupported. |
| `recipeIngredient[]` | ingredient lines | Text kept verbatim, with HTML entities decoded |
| `recipeInstructions` | instructions | Strings, `HowToStep.text`, and `HowToSection.name` plus its steps are flattened into numbered text with section headings |
| `recipeYield` | servings | Filled only when a value is a bare integer or "N servings"; otherwise left blank and the original text shown in the review. Values like "6 x 20cm flatbreads" or "1 cup" are not servings. |
| `prepTime` | prep minutes | ISO-8601 duration. `totalTime` is shown but never stored: shawarma says 20 minutes total while needing hours of marinating. |
| `author.name`, site name | `source_author`, `source_name` | — |
| URL after redirects | `source_url` | Normalized as in §8 |

**Component discovery.** This is the step that depends on Open decision 1.
- The recommended option reads the visible recipe card for **WP Recipe Maker** and **Tasty Recipes** only, and maps anchors back to JSON-LD ingredient lines by normalized text (case, whitespace, punctuation and entities ignored).
- Any other card format yields no component links. Its lines are imported as plain text.

**Classification and expansion rules:**
- **Unrelated, never followed:**
  - anchors in notes, summary or description;
  - credits ("based on", "recipe source", "with thanks to");
  - recommendations;
  - print, save, jump and `javascript:` links;
  - cross-site links (products, affiliates, tools);
  - a link that appears only inside a parenthetical remark.
- **Optional alternative:** the source explicitly offers a choice ("or", "optional", "store-bought"). The line is kept as written, the link is kept as an alternative, nothing is fetched, and nothing homemade is forced. *(Plan review 2026-10-02, owner decision 5: the line becomes an `alternative` component that owns that one line and keeps ≥1 same-site link; it is not counted in the notice.)* "Homemade" alone does not make a line optional.
- **Required component, expandable** only when all of these hold:
  - exactly one linked child on the line;
  - the amount is an explicit `1 batch` (or `1 recipe`);
  - the parent does not modify the child ("made with half…", "less", "reduce", "omit"…);
  - the depth is 0;
  - the fetched child has exactly one Recipe with non-empty ingredients and instructions;
  - the child's ingredient lines are not already all present in the parent. That would be an inline component; ThermoWorks' Hollandaise is one.
- **Required component, unresolved** when any expansion condition fails. Examples:
  - a choice of two child recipes (Sally's crusts);
  - a ½ batch;
  - a measured amount ("2 cups dressing");
  - a modified amount;
  - an incomplete child;
  - a fetch failure.
- **Fetch planning (plan review 2026-10-02):** only targets with at least one 1x, unmodified, depth-0 use are fetched, in document order, at most 3; a 4th becomes unresolved ("too many components"). Unresolved components take their title from the anchor text, else the line text; an expanded component takes the child's `name`, and a child with no `name` counts as incomplete.
- **Multiple uses of the same child:** one fetch, but each ingredient occurrence is resolved separately. A 1x use expands; a ½ use stays unresolved. Uses are never collapsed.
- **Purchased alternatives in notes:** a note that mentions the component and offers a purchased alternative is kept as an alternative on that component, and the default relationship stays. *(Plan review 2026-10-02, owner decision 5: notes-only purchased alternatives are not imported; the default relationship still stays.)*

**Expansion.** The parent placeholder line is replaced, in place, by the child's lines. Each child line records its source URL, component and scale, and the placeholder text is stored on the component (§5). The child's instructions become a separate titled section.

## 5. Durable representation

This assumes Open decision 2's recommended option: recipe-owned component metadata.

**Domain.** In `rust/crates/food-domain/src/recipe.rs`:
- New type:
  ```rust
  pub struct RecipeComponent { position: u32, title: String, source_url: Option<String>, scale: Option<Rational>, replaced_text: String, status: ComponentStatus /* Expanded | Unresolved | Alternative */, links: Vec<String>, instructions: String }
  ```
- `Recipe` (`:663`) gains `components: Vec<RecipeComponent>`.
- `IngredientLine` (`:592`) gains `component: Option<u32>`, holding the position of a component. `None` means main.
- Invariants:
  - every `component` value names an existing component;
  - an `Unresolved` component owns exactly one line, its original text;
  - an `Expanded` component owns at least one line.

**Storage.** In `rust/crates/kimatta-storage/src/lib.rs`, add **migration 17**, appended after the 16 existing `M::up` entries ending at `:692`:
```sql
CREATE TABLE recipe_component (
  recipe_id TEXT NOT NULL REFERENCES recipe(id), position INTEGER NOT NULL,
  title TEXT NOT NULL, source_url TEXT, scale_numer INTEGER, scale_denom INTEGER,
  replaced_text TEXT NOT NULL, status TEXT NOT NULL, links TEXT NOT NULL DEFAULT '[]',
  instructions TEXT NOT NULL DEFAULT '',
  PRIMARY KEY (recipe_id, position),
  CHECK ((scale_numer IS NULL) = (scale_denom IS NULL)));
ALTER TABLE recipe_ingredient_line ADD COLUMN component_position INTEGER;
```
- **Old rows:** `component_position` reads as NULL, meaning main, and there are no component rows. Old recipes are therefore unchanged.
- **Writes:** `write_recipe` (`:296`) already deletes and reinserts lines inside the IMMEDIATE transaction of `save_recipe` (`:275`). It does the same for `recipe_component`, so a save is atomic. A failure rolls back the recipe, provenance, lines and components together.
- **Archive and restore:** archive (`:2818`) and restore (`:2849`) are soft operations and need no change, because components live and die with their recipe row.

**FRB bridge.** In `rust/src/api/recipe.rs`:
- Add `RecipeComponentDto` (fields mirror the domain; scale is `Option<(u32,u32)>`).
- `RecipeDto` (`:100`) gains `components: Vec<RecipeComponentDto>`.
- `IngredientLineDto` (`:48`) gains `component: Option<u32>`.
- `save_recipe` (`:149`) validates the invariants. Regenerate `lib/src/rust/api/recipe.dart`.

**Dart editing.** In `lib/features/recipes/recipe_form_screen.dart`, the form rebuilds every line from its rows on save (`:270-290`). It must therefore carry each row's `component` value through, or edits would silently strip component boundaries.
- Rows are grouped under component headings.
- Deleting every line of a component deletes the component.
- Moving a line is not offered across groups.
- An `Unresolved` component's row shows its links and a "Replace with ingredients" action. That action opens the link in the browser; Kimatta does not re-import.
- Editing a line keeps its component.
- Instructions stay one text field (`:469`). Component instructions are shown read-only beneath it, as titled sections, and are editable through an "Edit section" sheet.

**Planning and shopping are unchanged.**
- `collect_raws` (`food-domain/src/shopping.rs:466`) reads lines, not components. Expanded child lines therefore behave like any other line, and an unresolved placeholder remains one separate `Needed` line. Verified by the tests listed in report §6.
- Planned-meal components (`MealComponent::Recipe`, `planned_meal.rs:74`) are a different concept and are untouched.

**Export and restore.**
- An export is a raw SQLite copy (`export_database`, `kimatta-storage/src/lib.rs:788`, `VACUUM INTO`).
- Restoring an older export runs the migration chain inside `restore_database` (`rust/src/api/health.rs:90`), so a pre-17 export gains an empty `recipe_component` table and NULL `component_position`.
- Restoring a post-17 export into an older app is refused by the existing `NewerSchema` check (`validate_export`, `:818`).
- `examples/check_export_migration.rs` gains `recipe_component` row counts.

## 6. Draft, review and save data flow

```
start(url) ──► fetch_page(url) ─► FetchedPage{final_url, html} | FetchError
           ──► extract(html) ─► RecipeCandidate{fields, lines, card_links} | Unsupported
           ──► plan_children(candidate) ─► [child_url] (≤3, deduped)
           ──► fetch_page(child)…  (parallel, same limits)
           ──► resolve(candidate, children) ─► ImportDraft{RecipeDto (id ""), notice?, alternatives[]}
review (Dart, in memory) ─► user edits ─► save_recipe(RecipeDto)  [existing atomic save]
```

- **FRB surface:** `import_recipe_from_url(url: String) -> Result<ImportDraftDto, ImportErrorDto>`. It runs on the FRB worker. *(Implementation plan 2026-10-02: the signature is `import_recipe_from_url(household_id, url) -> Result<ImportResultDto, KimattaError>`, and there is no Rust cancellation token. Dart drops stale results with a generation counter, and every fetch is bounded by one deadline across its redirect hops plus a DNS timeout.)* `resolve` is pure: it does no I/O and is unit-tested with fixtures.
- **What the draft carries:**
  - `provenance.kind = "imported"`. `ProvenanceKind::Imported` exists (`recipe.rs:190`), and no code creates it yet.
  - `source_url`, `source_name` and `source_author`.
  - Components as in §5.
  - Lines per Open decision 4.
- **Review.** It reuses the recipe form, entered with a draft instead of an id. The `_existing?.provenance ?? authored` fallback (`recipe_form_screen.dart:291`) must take the draft's provenance.
- **Validation** is the existing `Recipe::new` and `IngredientLine::new`. Every imported line has a non-blank `name`; if no name is parsed, the name is the original text.
- **Save** uses the existing `save_recipe`. If the save fails, the draft stays in the form, so the user's edits are never lost.
- **No silent overwrite.** User edits are never replaced by later import steps, because all fetching finishes before the review opens.
- **Process death** during review loses the draft. This is accepted; the import can simply be repeated.

## 7. Unresolved components and notices

- **One consolidated notice** sits at the top of the review: "N part(s) of this recipe couldn't be added automatically. Their original lines and links are kept — edit them or save as is." Tapping it scrolls to the first unresolved row.
- Save is never blocked by unresolved components.
- The original text and links survive review, edit and save, and appear on the recipe detail screen as a "From <site>" link plus each unresolved component's links. The detail screen does not show provenance today; that is a new, small widget.
- **Shopping consequence, shown in the notice's help text:** an unresolved component appears on the list as its own line ("Homemade Pie Crust or All Butter Pie Crust…"). Its sub-ingredients do not. This is the simulated Sally case in report §6.
- **Inline placeholders** like "1 batch Hollandaise sauce (see below)" are flagged in the review with a hint: "This line names a sauce whose ingredients are listed below; you can delete it." They are not auto-deleted.

## 8. Duplicates and failure behavior

**URL normalization for duplicate checks:**
- lowercase the scheme and host;
- drop the fragment and `utm_*` parameters;
- drop a trailing slash.

Duplicate behavior follows Open decision 3. Under the recommendation, Kimatta opens the saved recipe and offers an explicit "Import another copy".

| Situation | Behavior |
|---|---|
| Offline / DNS failure | "You're offline or the site can't be reached." Retry button. |
| Timeout | "The site took too long." Retry. |
| 401/403/429/451 or challenge | "This site didn't let Kimatta read the page." No retry offered. |
| Blocked destination (private IP, bad scheme, credentials) | "That link can't be imported." |
| Not HTML, too large, malformed | "This page isn't a recipe Kimatta can read." |
| No Recipe JSON-LD | Same as above, plus "You can still add it by hand" → blank form with `source_url` prefilled |
| Child fetch fails | Main draft continues; component unresolved (§7) |
| 404, 410, other 4xx (plan review 2026-10-02) | Same as "Not HTML, too large, malformed" |
| 5xx (plan review 2026-10-02) | Same as offline: "the site can't be reached", Retry |

## 9. Tests and acceptance

Each tier lists standard (S), edge (E) and adversarial (A) cases. The prior HIGH findings get all three; sample coverage gets S and E.

- **URL guard, `recipe-import` crate:**
  - S: https passes.
  - E: http, port 443, IDN host.
  - A: credentials, `file:`, IP literal, private and CGNAT addresses, a DNS answer that changes between check and connect, a redirect to a private address, a 6th redirect, a 3 MB + 1 byte body, a slow-drip body hitting the total timeout.
- **Extraction (fixtures authored from structure, never committed publisher pages):**
  - `@graph`, `HowToSection`, entity decoding, multiple Recipes, malformed JSON next to valid JSON;
  - a hostile `<script>` payload staying inert;
  - non-integer yields;
  - `totalTime` ignored.
- **Resolver, pure; ports the 16 synthetic regressions from the research run plus:**
  - S: 1x expansion keeps exact order and wording.
  - E: ½ batch, measured amount, two uses of one URL, inline child, choice of two children, a note-only purchased alternative, homemade-required.
  - A: a same-name credit link, a link in a parenthetical, cross-site product links, a modified amount, an incomplete child, depth 1, a fourth child URL.
  - Every case asserts that no line is missing or duplicated.
- **Storage and bridge:**
  - a round trip of a recipe with expanded and unresolved components;
  - an old-row read (NULL `component_position`);
  - an invariant violation is rejected;
  - a save failure rolls back components;
  - migration 17 on a v16 fixture, plus the existing `an_existing_v4_database_migrates_to_latest_without_losing_data`;
  - export → restore keeps components;
  - a v16 export restores into a v17 app.
- **Dart review UI:**
  - the draft opens prefilled;
  - one notice is shown for N unresolved components;
  - Save is enabled with unresolved components;
  - editing a component line keeps its component;
  - deleting a component's last line removes the component;
  - provenance is kept on save;
  - the inline-placeholder hint appears.
- **App networking and share (device, release build):**
  - INTERNET is present in the merged release manifest (`aapt dump permissions`);
  - a cold-start share opens the review;
  - a warm-start share opens the review;
  - a repeated share prompts;
  - airplane mode gives the offline message;
  - a 403 site gives the refusal message.
- **Recipe → plan → shopping:**
  - an imported, expanded recipe planned at scale 2;
  - unresolved lines stay separate and `Needed`;
  - child lines appear once per use.

**Acceptance criteria:**
- **AC-1:** Pages with exactly one Recipe JSON-LD import into an editable review with provenance `imported`, `source_url`, `source_name` and `source_author`.
- **AC-2:** Every failure row in §8 produces its message, and nothing is saved.
- **AC-3:** Nothing is saved before Save. Edits round-trip through `save_recipe`, components included.
- **AC-4:** Component rules: zero false expansions across the resolver fixture suite, and unresolved components keep their text and links with one notice.
- **AC-5:** The release APK fetches over the network, and share works on cold and warm starts.
- **AC-6:** A fresh-context review confirms that no access-control bypass or auto-publication exists, and that diagnostics are content-free.

**Verification commands:**
- `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`;
- `flutter analyze`, `flutter test`;
- `tools/emulator.sh` device run for AC-5;
- `check_export_migration` on the owner's real export.

**Rollout and rollback.**
- Ship behind the "Paste link" action and share target in one release.
- Migration 17 is additive. A rollback build is refused on a v17 database by `NewerSchema`, which is the existing policy, so the rollback path is a forward fix, not a downgrade.
- To disable the feature, remove the action and the intent filter; imported recipes remain ordinary recipes.

## 10. Open decisions (owner)

### Open decision 1 — How far page reading goes

**Context.** OPT-001's gate (`docs/task/optional/OPT-001_STRUCTURED_URL_RECIPE_IMPORT.md:53`) stops the card if "useful coverage requires general scraping". Its non-goals exclude scraping, and its stop conditions include site-specific scraping. The measured evidence:
- 12 of 12 reachable pages had JSON-LD;
- **0 of 2 real linked components were visible in JSON-LD**.

| Option | What it finds (2026-10-02 sample) | Cost / risk |
|---|---|---|
| **A. JSON-LD only** | All 12 recipe drafts. No component links: "1 batch Lemon yogurt sauce" imports as one text line. | Inside today's card scope. No component feature at all; §4's discovery and §5's component storage are unnecessary. |
| **B. JSON-LD + WPRM/Tasty card adapter (recommended)** | Both real required components (4/4 anchors), 0 false expansions, all 16 regressions. 7 of 12 recipe pages are WPRM, 3 Tasty. | Explicit OPT-001 scope change, since this is plugin-specific DOM reading. Maintenance when plugin markup changes; other sites degrade to A. |
| C. Generic DOM heuristic | Same 4/4 required, 0 false expansions, but **8 non-recipe links labelled required** (BBC glossary, King Arthur shop) and 7 optional mislabels (adapter: 3). Each would raise a false "couldn't import" notice. | Noisier; needs a child fetch per suspect link to confirm a Recipe page (more fetches, slower imports). Closest to the card's "general scraping" stop. |

**Recommendation: B**, recorded as an explicit amendment to OPT-001. Two reasons:
- It is the smallest option that delivers components at all.
- Its failure mode, an unknown plugin, is the honest A behavior.

The difference from B:
- **A** removes §4's card step, the child fetches, §5 and §7's component parts; the remaining design is unchanged.
- **C** replaces the adapter with ingredient-line matching across the whole page, and requires the child to verify as a Recipe page before it counts.

### Open decision 2 — How sub-recipes are stored

| Option | Description | Migration / FRB / export | Behavior |
|---|---|---|---|
| **A. Recipe-owned component metadata (recommended)** | §5 as written | Migration 17 (one table, one column); 2 DTO changes; export via the migration chain | Boundaries, sources and scale kept for later; one recipe to plan; no sync |
| B. Flat lines with provenance | Child lines merged in; child instructions appended as a titled section in the text; child URL listed in instructions text | No migration, no DTO change | Smallest. Loses component boundaries and scale, so a later "swap to store-bought" can't be offered. Unresolved links live only in the text. |
| C. Component graph | Child saved as its own Recipe; the parent references it through a link table; planning expands it | Migration with a link table, cycle checks, archive semantics (archived child vs live parent); shopping must recurse | Reusable sauce, but edits to the child silently change parents. That is the synchronized graph the 2026-10-01 owner record ruled out ("No ongoing source synchronization") |

**Recommendation: A**, which keeps the owner's 2026-10-01 wish to preserve "component boundaries, source links and scaling information for future changes" without a graph.
- **If B:** delete §5 except provenance; the resolver output flattens; tests drop the storage, bridge and component cases.
- **If C:** add `recipe_dependency(parent_id, child_id, scale)`, recursive shopping expansion with a depth cap, and archive rules. This roughly doubles the storage and test work.

### Open decision 3 — Duplicate saved URL

| Option | Behavior |
|---|---|
| **A. Open existing + "Import another copy" (recommended)** | After the fetch succeeds, if a non-archived recipe has the same normalized `source_url`, the review is replaced by "You already saved this recipe" with **Open** (default) and **Import another copy**. The copy is a fully independent recipe. Existing edits are never touched. |
| B. Always import a new copy | No check. Duplicates pile up silently. |
| C. Refuse | Simplest, but blocks a legitimate second variant. |

**Recommendation: A**, which matches the owner's 2026-10-01 record ("Recognized duplicate links open the saved recipe with an explicit import-another-copy option").
- It needs `CREATE INDEX recipe_provenance_source_url ON recipe_provenance(source_url)` in migration 17, plus a `find_recipe_by_source_url(household_id, url)` bridge function. No recipe deduplication exists today (`list_recipes`, `lib.rs:2738`).
- An archived match offers "Restore it" instead of "Open".
- **If B:** drop the index and the function. **If C:** replace the dialog with an error.

### Open decision 4 — How imported text lines become ingredient lines

No line parser and no catalog matcher exist. The form's `parseQuantity` (`lib/features/recipes/recipe_fields.dart:87`) parses only a bare amount field, and nothing sets `ingredient`.

| Option | Line result | Shopping consequence |
|---|---|---|
| **A. Everything enters unresolved (recommended)** | `original_text` = `name` = the source line; `ingredient: None`; `QuantityDto::Unknown`; `UnitDto::None` | Verified: every line is a separate "Other" item, `Needed`, never merged, not scaled with the plan. Shawarma + sauce = 24 separate items, with garlic twice (report §6). The user can tidy any line in the review. |
| B. Parse amount and unit only | A new Rust line parser fills `Quantity`/`Unit` and the leftover becomes `name`; `ingredient` stays None | Still unresolved and unmerged, but quantities read cleanly and unresolved lines with exact quantities scale. Parser risk on "1 and 1/2", "½", "2 Tbsp + 1 tsp", "(31g)": a wrong parse silently changes a line the user didn't touch. |
| C. Parse + catalog match suggestions | B, plus a name→catalog suggestion confirmed per line in the review | Merged, categorised shopping and pantry marks work. Largest: a matching algorithm, a review UI per line, and wrong-match risk (pantry would hide something needed). |

**Recommendation: A** for the first release.
- It is the only option with no new interpretation risk.
- It matches the existing invariant that `original_text` "is never derived" (`recipe.rs:590`).
- Its shopping cost is visible and editable rather than wrong.

B and C are a natural follow-up card once real imports show which lines users actually fix.
- **If B:** add a `parse_line` module to `recipe-import` plus its fixture suite, and mark parsed fields in the review.
- **If C:** add B, a matcher over catalog and custom ingredients (`list_custom_ingredients`, `rust/src/api/recipe.rs:262`), and a per-line confirm control.

---

Any behavior not decided above must be raised as an owner decision before implementation, not left to implementer discretion. This document is a proposed design artifact, not execution authorization.
