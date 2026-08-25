# DEC-004 — Naming clearance and production identity

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | In Progress |
| Type | Decision |
| Workstream | Product foundation |
| Depends on | DEC-001 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No implementation until Done |
| Recommended workflow | Owner-led naming research; then `deep-options` on the shortlist |
| External actions | Read-only public searches (app stores, web, trademark databases). No Play Console, domain purchase, trademark filing, or Firebase project creation without separate owner authorization. |

## Outcome and user value

Choose a public product name and permanent production application ID that are clear of app-store, web, and trademark collisions, so that the first card binding an identifier to real infrastructure binds the right one.

## Authoritative sources

- `docs/task/decision/DEC-001_PRODUCT_PLATFORM_IDENTITY_DECISION.md` — Resolution §1
- `docs/PRD_v2.md` §§1, 14.1, 24
- `docs/ROADMAP.md` D-019

## Locked constraints

- `MealMate` is retired as the public brand (DEC-001). It survives only as an internal codename and the development identity `dev.mealmate.temp` / Dart package `meal_mate`.
- No `com.mealmate.*` production identifier.
- The production application ID is permanent once published; it must not be chosen before the name is cleared.
- Do not use Play Console to test identifier availability.
- Avoid repo-wide rename churn until the final name is chosen; the rename is one bounded change, done once.

## Decisions to resolve

1. Final public product name and display spelling.
2. Production Android application ID (and the namespace a future iOS bundle ID will share).
3. Whether the Dart package name `meal_mate` is renamed at the same time or left as codename.
4. Which share-URL domain the name implies (informs `MVP-020`/`MVP-021` App Links; purchase is a separate authorization).

## Clearance checklist

Each must be recorded as PASS, FAIL, or NOT VERIFIED with evidence for the chosen name:

- Google Play: no app with the same title positioned in meal planning/recipes; no published app using the intended package ID (direct listing URL check).
- Apple App Store: same title check, for the post-launch iOS client.
- Web/search: first page of results not dominated by a same-concept product; intended domain available or owned.
- Trademark: no live mark in software/app classes (US class 9/42 at minimum) for the name or a confusingly similar one.

## Completion criteria

- One name, one production application ID, rationale, rejected candidates, and the clearance checklist with evidence.
- A bounded rename card (or a scope line in `MVP-018`) listing every place the codename must change: `applicationId`, Android manifest label, pubspec name if renamed, docs.
- That same rename card (or `MVP-018` scope line) must additionally require the production application ID's availability to be re-checked immediately before the first Play upload that burns it — `MVP-018` binds only the dev Firebase project, and `MVP-022` is readiness-only, so that upload sits outside the register in the separately authorized submission. The fallback is an alternate application-ID **shape** for the chosen name — the three-segment form the table preamble records as unscreened — not an alternate name: a name change at that point would violate locked constraint line 33, and any other name's ID carries the identical unprovability. The read-only prescreen cannot discharge this.
- Recorded in `docs/ROADMAP.md`; `MVP-018`, `MVP-021`, and `MVP-022` remain blocked until Done.

## Stop/failure conditions

- Stop if clearance requires a paid search, legal opinion, or purchase; record as NOT VERIFIED and return to the owner.

## Shortlist and prescreen evidence — 2026-08-24

### What this section is, and what it is not

This is the shortlist register plus prescreen evidence. It **does not resolve decisions 1–4**. No
name is chosen, no application ID is committed, and no domain was purchased. All lookups were
read-only public searches; the Play Console was not used.

Five things a reader must know before trusting the table below.

1. **Deviation from the card's recommended workflow.** The card specifies "Owner-led naming
   research; then `deep-options` on the shortlist" under `Elevated` assurance. The research here
   ran agent-led: the owner supplied the lead name (`Kimatta`) and locked the register, depth, and
   method through a `grill-me` interview; the agent performed only read-only lookups and recorded
   what it observed. The `deep-options` half is not dropped — it is deferred to the resolution
   step, where a choice is actually made.

2. **A Play package-ID cell can never read `PASS`.** No public search can prove a package ID is
   *available*. Play burns an ID permanently on publication, and an app that was published and
   later unpublished — or that sits in closed testing — may have no search-index entry and no
   reachable listing. `DEC-001` escaped that trap only because the `com.mealmate.app` index entry
   happened to survive. The authoritative check is the Play Console, which is outside this card's
   authority. Every package-ID cell below **for a screened name** therefore reads
   `NOT VERIFIED (unprovable read-only)`, and **that residual risk does not clear until an
   identifier is actually reserved.** Locked constraint line 32 puts the authoritative check outside
   this card's own work, so under `docs/task/README.md`'s verification rule this item may stand at
   `NOT VERIFIED` when the card reaches Done: the discharge point is the availability re-check
   required immediately before the first Play upload (see Completion criteria), and the owner
   records dated acceptance of the residual risk in the Evidence-log row. Line 15's authorization
   carve-out does not extend to availability testing.

3. **Only the lead was trademark-screened.** Per the locked clearance depth, USPTO was attempted
   for `Kimatta` alone. No backup carries trademark evidence, so **no backup is decision-ready**;
   promoting one requires a second research round.

4. **The register contains no unencumbered shape-match to the lead.** `Savora` and `Norella` were
   the two candidates deliberately shaped like `Kimatta` (three syllables, vowel-final). `Savora`
   is rejected outright below, and `Norella` has both domains registered. If the owner wants a
   fallback that still sounds like the same brand family, this register does not supply a clean
   one.

5. **`NOT SCREENED` is a literal of this table only.** It never appears in the `docs/ROADMAP.md`
   Evidence log, whose contract admits `PASS` / `FAIL` / `NOT VERIFIED`.

### Method finding — Play's own search under-reports titles

**Pass 1 used Play's public search page; it produced at least one false negative.** A search for
`kondate` returned 11 app tiles, none titled Kondate. A later web search surfaced
`com.orangefox22.Kondate`, whose listing fetches with HTTP 200 and whose own description reads "a
menu management app… daily meal planning… create shopping lists" — a direct in-category collision
that Play's ranking simply did not show.

Every survivor was therefore re-checked by a second, independent method (web search restricted to
`play.google.com`). Both methods agree on all remaining names. This matters for how much weight a
`PASS` carries: it means "not found by two independent public methods", not "proven absent".

A second premise also failed, in the useful direction: the plan anticipated Play's listing being
unfetchable headlessly. It was not. All 12 Play searches and every listing fetch returned HTTP 200
with parseable HTML, so the budget branch for mass-unreachable title checks never fired.

### Summary table

All checks performed 2026-08-24. `REG` = registered; **bold** in a domain column marks an
unregistered domain for a name still in play. The package-ID column screens the two-segment form
`app.<name>` only — if the resolution picks a three-segment ID that string is unscreened, and it is
the fallback shape the Completion criteria name; per note 2 no package-ID cell can read `PASS` in
either shape.

| Name | Play title | `.com` | `.app` | Package ID `app.<name>` | Apple title | Web/SERP | Trademark | Disposition |
|---|---|---|---|---|---|---|---|---|
| **Kimatta** | PASS | REG | **FREE** | NOT VERIFIED (unprovable read-only) | PASS | PASS | NOT VERIFIED (unreachable) | **Lead** |
| Kondate | **FAIL** | REG | FREE | NOT VERIFIED (unprovable read-only) | **FAIL** | **FAIL** | NOT SCREENED (lead-only check) | **Rejected — Pass 1 Play title** |
| Ichiju | PASS | REG | **FREE** | NOT VERIFIED (unprovable read-only) | PASS | PASS | NOT SCREENED (lead-only check) | Viable — encumbered (generic culinary term) |
| Shitaku | PASS | REG | **FREE** | NOT VERIFIED (unprovable read-only) | **FAIL** | PASS | NOT SCREENED (lead-only check) | Viable — encumbered (Apple exact title, out of category) |
| Junbi | PASS | REG | REG | NOT VERIFIED (unprovable read-only) | PASS | **FAIL** | NOT SCREENED (lead-only check) | Viable — encumbered (crowded; one household-adjacent) |
| Osusume | PASS | REG | **FREE** | NOT VERIFIED (unprovable read-only) | PASS | PASS | NOT SCREENED (lead-only check) | Viable |
| Provi | PASS | REG | REG | NOT VERIFIED (unprovable read-only) | **FAIL** | **FAIL** | NOT SCREENED (lead-only check) | Viable — encumbered (heavy; funded beverage company) |
| Tavo | **FAIL** | REG | REG | NOT VERIFIED (unprovable read-only) | PASS | **FAIL** | NOT SCREENED (lead-only check) | Viable — encumbered (exact title in Food & Drink) |
| Kelda | PASS | REG | REG | NOT VERIFIED (unprovable read-only) | PASS | **FAIL** | NOT SCREENED (lead-only check) | Viable — encumbered |
| Norra | **FAIL** | REG | REG | NOT VERIFIED (unprovable read-only) | **FAIL** | **FAIL** | NOT SCREENED (lead-only check) | Viable — encumbered (heavy) |
| Norella | PASS | REG | REG | NOT VERIFIED (unprovable read-only) | PASS | PASS | NOT SCREENED (lead-only check) | Viable — encumbered (both domains taken) |
| Savora | **FAIL** | REG | REG | NOT SCREENED (triaged out at Pass 1) | NOT SCREENED (triaged out at Pass 1) | NOT SCREENED (triaged out at Pass 1) | NOT SCREENED (lead-only check) | **Rejected — Pass 1 Play title** |

A `FAIL` in the Play or Apple column means a titled match exists; it is a **kill only when at
least one match is positioned in meal planning or recipes**. That is true for Kondate and Savora
alone. Tavo's and Norra's title collisions are real but sit in other categories, so they are
recorded as encumbrances rather than rejections — the owner decides what that adjacency is worth.

Card line 48 carries two conditions: a SERP condition (page one not dominated by a same-concept
product) and a domain condition (intended domain available or owned). Neither is a kill at
prescreen. A `FAIL` in the Web/SERP column records a crowded or concept-adjacent results page for
the owner to weigh at resolution — `Junbi` crowds it with one household-coordination-adjacent
service among several unrelated ones, `Provi` with a single operating company that owns the whole
page. Domain unavailability is recorded in the `.com`/`.app` columns as an encumbrance, not a kill;
it is what makes `Junbi` and `Provi` weaker candidates than their Play and Apple rows suggest. Line
48 is discharged for the chosen name at resolution; this table does not discharge it.

### Per-name evidence

**Kimatta** (決まった, "it's decided") — the lead.
- Play title: no titled match in 28 result tiles; nearest were KiMoH, Kimera, KIMEA, kimkim,
  KimunAI. Second method (search restricted to `play.google.com`) returned KAMATTO, KMatt
  Supermarket, Kima App, Kimania, Kimap — no Kimatta.
- `kimatta.com`: REG, Japan Registry Services Co., Ltd., status `active`, created 2004-06-01.
- `kimatta.app`: **RDAP 404 — unregistered.** The preferred share domain is available.
- `app.kimatta`: listing URL HTTP 404; Play index search for the string returned 50 tiles, none
  matching. Both observations recorded; per the rule above this is `NOT VERIFIED`, not `PASS`.
- Apple: iTunes Search API, `entity=software`, US — 8 results, no exact title, no partial.
- Web/SERP: first page is Japanese-dictionary and translation entries for the word, plus a TikTok
  handle `@kimattamoon`. No same-concept product anywhere on the page.
- Trademark: see the open item below.

**Kondate** (献立, "menu / meal plan") — rejected.
- Play title: `com.orangefox22.Kondate`, titled "Kondate", Lifestyle. Own description: "a menu
  management app for busy people… daily meal planning. Manage ingredients for multiple dishes at
  once and create shopping lists." That is this product's exact category. Not surfaced by Play's
  own search (see the method finding above); found via web search and confirmed by direct listing
  fetch, HTTP 200.
- Apple: "Kondate - Smart Meal Planner" (Food & Drink), "Kondate Planner" (Food & Drink),
  "Kondate Memo - Daily Meal Log" (Food & Drink), "KONDATE.AI - 毎日の献立アプリ" (Lifestyle).
- Web/SERP: App Store and Play listings for the above rank on page one alongside general
  meal-planning roundups.
- JP storefront check for 献立 found a crowded concept space (me:new, DELISH KITCHEN, らくらく献立表,
  キッコーマンきょうの献立) but no latin-titled "Kondate" — the collision is the US listing, not the
  Japanese generic.
- `kondate.com` REG (GMO Internet Group / Onamae.com, 2006-03-20); `kondate.app` unregistered.
- Kondate was flagged as a probable failure when the register was assembled. It failed for a
  different reason than predicted: not Japanese genericness, but a direct English-language
  competitor of the same name.

**Ichiju** (一汁, from 一汁三菜).
- Play title: no titled match in 9 tiles. Second method found `jp.ichizyu.ichizy` — a conveyor-belt
  sushi restaurant app titled いちじゅう in kana, not the latin string.
- `ichiju.com` REG (GMO / Onamae.com, 2002-10-06); `ichiju.app` **unregistered**.
- Apple: 40 results, no exact or partial title match.
- Web/SERP: page one is culinary-education content about 一汁三菜 (Just One Cookbook, Wikipedia,
  cooking schools) — the concept space is food, but no competing *product* appears. Recorded
  `PASS` with the encumbrance noted: the term is a generic culinary phrase, which is good for
  trademark distinctiveness only if used arbitrarily, and bad for organic search.

**Shitaku** (支度, "preparation").
- Play title: no titled match in 30 tiles; results were dominated by "Shikaku" puzzle games.
- `shitaku.com` REG (Domeneshop AS, 2007-09-21); `shitaku.app` **unregistered**.
- Apple: exact title "Shitaku", bundle `com.yharu.Shitaku`, **Weather** — out of category. Also
  "Shitaku - 支度タイマー" (Productivity).
- Web/SERP: a GitHub user, dictionary entries, and `shitaku.com` (a personal blog). No product
  competitor.

**Junbi** (準備, "prep").
- Play title: no titled match in 30 tiles.
- `junbi.com` REG (Inames Co., Ltd., 1999-12-01); `junbi.app` REG (GoDaddy, 2023-01-28) and is
  currently presented as a premium domain listing for sale.
- Apple: "Junbi: Study Podcasts" (Education) — partial, not exact.
- Web/SERP: crowded. `junbi.ai` is a funded YouTube-ad analytics platform; `junbi.life` is "Calm
  preparation for busy family life" — smart checklists for school mornings and family routines,
  which is **adjacent to this product's household-coordination positioning**; plus JUNBI martial
  arts training. Recorded `FAIL`.

**Osusume** (おすすめ, "recommendation") — cleanest backup.
- Play title: no titled match in 30 tiles; nearest were Osome, OSUS, osu!stream, Osusu.
- `osusume.com` REG (NameCheap, 2002-06-09); `osusume.app` **unregistered**.
- Apple: 41 results, no exact or partial title match.
- Web/SERP: no same-concept product; results are the unrelated near-names above.

**Provi**.
- Play title: no exact title in 10 tiles; partials were "Radio Provi", "Alcohol Use and Misuse -
  Provi", ProvidusPlus, Providus Bank.
- `provi.com` REG (GoDaddy, 2001-05-03) — held by Provi, a funded B2B beverage-alcohol marketplace
  for bars and restaurants. `provi.app` REG (GoDaddy, 2018-05-08).
- Apple: exact title "Provi", bundle `com.company.ProviApp`, Productivity.
- Web/SERP: page one is owned by provi.com — company site, sign-up, inventory product pages,
  software-review listings. Food-and-drink adjacent, though B2B alcohol ordering rather than
  meal planning. Heavy encumbrance: an operating company owns the name, both domains, and the SERP.

**Tavo**.
- Play title: exact title "Tavo", `com.tavo.app`, **Food & Drink**. Own description: "Tavo –
  Restaurant Table Booking & Reservation Management". Same Play category as this product, but a
  different job — reservations, not meal planning or recipes. Also "TAVO Sleep".
- `tavo.com` REG (Squarespace, 2000-01-15); `tavo.app` REG (Squarespace, 2023-02-25) — same
  registrar, so both are plausibly one holder.
- Apple: no exact title; partials "Tavo - AI Roleplay Frontend" (Utilities), "Tavo - AI Volleyball
  Analytics" (Sports), "TAVO App" (Productivity).
- Web/SERP: multiple distinct Tavo apps, including "Tavo — Real-Time Vibes" covering bars and
  restaurants. Recorded `FAIL`.

**Kelda**.
- Play title: no titled match in 50 tiles.
- `kelda.com` REG (Abion AB, 1998-03-03); `kelda.app` REG (GoDaddy, 2018-11-15).
- Apple: only 2 results total, no exact match.
- Web/SERP: `kelda.health` (AI lab-analysis platform), Keldan (Icelandic finance app), Kelda
  Technology, Kelda Dynamics (Norwegian drilling), plus the Norse word itself. Recorded `FAIL` —
  crowded, though nothing in this product's category.

**Norra**.
- Play title: exact title "Norra", `io.norra.mobile`, **Business** — medical-equipment tracking for
  skilled nursing facilities. Out of category.
- `norra.com` REG (GoDaddy, 2001-06-30) — the National Off-Road Racing Association.
  `norra.app` REG (Hosting Concepts B.V. / Registrar.eu, 2026-01-17).
- Apple: exact title "Norra", bundle `com.verve.norra-io`, **Medical**. Also "NORRA by skovby",
  "Norran" (Swedish news), Nordic Regional Airlines.
- Web/SERP: `norra.io`, NORRA racing, the airline, and the Swedish newspaper. Recorded `FAIL`.
  Heavy encumbrance across every surface checked, none of it in this category.

**Norella**.
- Play title: no titled match in 30 tiles.
- `norella.com` REG (Launchpad.com Inc., 2005-07-28); `norella.app` REG (united-domains AG,
  **2026-07-08** — registered within the last two months).
- Apple: 4 results, no exact or partial match.
- Web/SERP: consumer brands (Norella UK hair care, Norella Goods), a Spotify artist, a YouTube
  creator; `norella.shop` carries a Scamadviser advisory. No app, no same-concept product.

**Savora** — rejected.
- Play title: multiple exact titles, several directly in category — "Savora: Pantry Recipes"
  (`io.savora.foodapp`, Food & Drink), "Savora: AI Recipes & Cuisines", "Savora: Meal Memories",
  "Savora: Tasting Journal", "Savora: Costco Price Tracker", plus "Savora Finance" and "SavorAI".
  `io.savora.foodapp`'s own description is food-recommendation saving from social video.
- `savora.com` REG (CSC Corporate Domains, 1998-10-13 — a corporate-brand registrar, consistent
  with the Amora **Savora** condiment brand this candidate was flagged against). `savora.app` REG
  (Squarespace, 2023-06-17).
- Triaged out at Pass 1; package ID, Apple, and SERP were not screened.

### Trademark — the one open item for the owner

**`Kimatta`: NOT VERIFIED (unreachable).** The check was attempted, not assumed. Receipt:

- `POST https://tmsearch.uspto.gov/api-v1-0-0/tmsearch/select` → **HTTP 405**, S3
  `MethodNotAllowed` — that path is static storage, not a search API.
- `GET https://tmsearch.uspto.gov/api-v1-0-0/tmsearch/select?q=kimatta` → **HTTP 404**, S3
  `NoSuchKey`.
- `GET https://tmsearch.uspto.gov/search/search-results?q=kimatta` → **HTTP 200, 125,660 bytes**,
  but the payload is the single-page-app shell: zero occurrences of the query term, no result
  markup, no discoverable API path in the HTML. TESS was retired and its replacement renders
  results client-side.

A secondary web search for `"Kimatta" trademark USPTO` returned no record of any such mark —
**indicative only**, not a clearance. `DEC-001` hit this same wall on 2026-08-21 and recorded the
same limitation.

**Routed to the owner:** an authoritative trademark search in classes 9 and 42 for `Kimatta` **and
for confusingly similar marks** — per clearance checklist line 49 that means phonetic and
near-string neighbours, not the exact term alone. The card's **Kimatta** Play-title evidence bullets
already surfaced unassessed candidates of exactly that kind: KAMATTO, Kimera, KIMEA, Kima App. It
needs either an interactive browser session at `tmsearch.uspto.gov` or a paid search. Only the paid
route falls under the stop condition above; the interactive session is a read-only trademark-database
search this card's External actions already permit, so it is work the owner can still do. **The
Done-gate exception in `docs/task/README.md` therefore does not cover this item** — it is outstanding
work inside the card's own scope, and it blocks Done until the owner runs the search or records a
decision not to. That is the difference from the package-ID item, which no method available to anyone
can resolve read-only and which the exception does cover. Either way it must be closed before
`MVP-018`, `MVP-021`, or `MVP-022` binds the name.

### Where this leaves DEC-004

`Kimatta` passed every store check that read-only methods can settle — Play title by two independent
methods, Apple title by the iTunes Search API alone — and **`kimatta.app` is unregistered**, so the
preferred identifier scheme (`applicationId app.kimatta`, share domain `kimatta.app`, matching
future iOS bundle ID) needs no alternate *name*. The fallback the Completion criteria require is an
alternate application-ID **shape** for this same name — the unscreened three-segment form — not a
different name. `app.kimatta` is a valid two-segment application ID; three segments is the more
common convention, and that remains a judgment call for the resolution step. Separately, the
`app.kimatta` package-ID check cannot be settled read-only at all and therefore stands at
`NOT VERIFIED` per note 2, notwithstanding a 404 listing URL and a null index search.

Two names are rejected (`Kondate`, `Savora`). `Osusume` is the only backup with no recorded
encumbrance, and `Ichiju`, `Shitaku`, and `Osusume` are the three *surviving* candidates whose
`.app` domain is still free — `kondate.app` is also unregistered, but Kondate is rejected.

Open items for resolution: the trademark search above, which remains the one open item for the
owner and blocks Done; decision 1 itself — no name is chosen here, and the choice additionally
awaits the `deep-options` step that note 1 defers to resolution; the two- versus three-segment
shape for the application ID, the three-segment form being unscreened; the `app.kimatta`
package-ID availability, which no read-only method can settle and which `D-027` covers on
owner-recorded acceptance; whether `kimatta.com` (registered since 2004 to a Japan Registry
Services registrant, no same-concept product observed) matters enough to pursue; and decisions
2–4 of this card, which this section does not touch.
