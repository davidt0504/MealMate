# DEC-004 — trademark screen for "Kimatta"

Evidence for `DEC-004` (naming clearance), which gates `MVP-018`, `MVP-021` and `MVP-022`.
`D-019` retired `MealMate` as the public brand after a clearance check found the name crowded
and `com.mealmate.app` already consumed on Play; this file is the same exercise for the
replacement, plus the method so it can be re-run before any Play submission.

**This is a screen, not a clearance, and nothing here is legal advice.** A screen finds obvious
conflicts cheaply. It does not establish that a mark is available — that requires a full search
and an attorney's opinion, which is worth buying before commercial use or a store listing, and
is not worth buying for a sideloaded APK shared with a handful of testers.

## Scope of the name

- **Word mark:** `Kimatta` — Japanese 決まった, "it is decided / settled", matching the
  planner's Cover My Week concept and the existing Rust crate names (`kimatta-application`,
  `kimatta-storage`, `kimatta_bridge`).
- **Likely class:** Nice class **9** (downloadable mobile application software) and possibly
  **42** (SaaS) if the cloud work in `MVP-018` ships.
- **Relevant competitors for confusion analysis:** meal planning, recipe, grocery and pantry
  apps; grocery retail (Amazon Fresh, Whole Foods, Instacart, Kroger) because the
  goods/services proximity is what makes an otherwise weak claim expensive to answer.

## Screen results, 2026-09-04

Three passes: an in-session web screen, and two independent deep-research runs against the
prompt below, filed verbatim in `docs/research/`. They agree on the important points and neither
overstated its reach.

### What is established

- **No exact `KIMATTA` mark surfaced in any software or food class**, in any of the three
  passes, and no US meal-planning, recipe, pantry or grocery product trades under the name.
- **`kimatta.com` is taken and live** — the single hard finding. Verified in-session by DNS
  rather than taken on report: `A 210.134.168.2`, `MX mx.kimatta.com`, nameservers
  `ns1/ns2.canonet.ne.jp` (a Japanese ISP). The web root returns **HTTP 403** behind a
  Shift_JIS page last modified **2008**, so the site is dormant but the domain still carries
  mail. It belongs to キマツタ三賞堂, an Osaka seal/stamp shop of roughly eighty years, which
  publishes `hanko@kimatta.com`. Note the kana is キマツタ (*ki-ma-tsu-ta*), **not** キマッタ
  (*ki-mat-ta*) — the businesses are not homographs in Japanese, but the Latin string is
  identical and the `.com` is unavailable.
- **`kimatta.app`, `.io` and `.dev` have no NS delegation at all**, which strongly indicates
  they are unregistered and available. Neither report checked these.
- **Closest spelling neighbours are `KIMATA` / `KIMATO` / `KINATA` / `KIMARA`, not `KISMET`.**
  Both reports converged on this independently. `KIMATA` appears in federal records mainly as a
  Japanese surname on unrelated goods; `github.com/kimata` is an occupied personal account.

### The substantive legal point, which neither the checklist nor I had anticipated

**Doctrine of Foreign Equivalents** (TMEP §1209.03(g)). Japanese is a common modern language, so
a USPTO examiner is expected to *translate* the mark during examination: `Kimatta` → "it is
decided". Against software whose stated purpose is deciding what a household eats, that invites
a **§2(e)(1) mere-descriptiveness refusal** (15 U.S.C. §1052(e)(1)) — the name arguably
describes the intended result of the product.

The counter-argument is that the mark is **suggestive rather than descriptive**: "it is decided"
names no feature, interface or technical characteristic of a recipe manager or pantry tracker,
and reaching the product from the word takes a step of imagination. That is a real argument, not
a certain one. Separately, in Japan an everyday conversational inflection like 決まった is weak
under JPO Article 3(1)(iii) unless stylised — relevant only if Madrid extension to Japan is ever
pursued.

This is the finding most likely to cost money later, and it is about **registrability**, not
about anyone else's rights.

### What is still not established

**The registry searches themselves.** All three passes report USPTO, EUIPO and WIPO Madrid
Monitor as **NOT SEARCHED** — none could execute the interactive queries. Every serial number in
`docs/research/` is a secondary-index lead marked `UNVERIFIED`, exactly as the prompt required.
Neither store was searched through its own index either.

Two cautions when reading `docs/research/kimatta-trademark-screening-plan.md` (renamed from
`Kimatta Trademark Screening Plan.md`; the spaces made it awkward to reference from tooling):

- **Its serial and registration numbers have footnote digits concatenated onto them** — `NQBQ`
  reads `Reg: 44444701`, which is registration `4444470` plus footnote `1`. Strip the trailing
  marker before pasting any number into TSDR.
- Several of its leads rest on weak aggregators (a hotel listing, a Dubai business directory)
  and are noise rather than signal. It marks them `UNVERIFIED`, which is correct, but they do
  not deserve follow-up.

## Manual checklist — about twenty minutes

| # | Source | Where | Looking for |
|---|---|---|---|
| 1 | USPTO word search | tmsearch.uspto.gov | `Kimatta`, plus `Kimata`, `Kimato`, `Kimatsu` as near-misses. Note serial + status of anything live in class 9 or 42 |
| 2 | EUIPO | euipo.europa.eu/eSearch | Same terms, if you might ever distribute in the EU |
| 3 | Google Play | play.google.com search | An existing app of this name — this is what killed `MealMate`, and it does not require a registration to be a problem |
| 4 | Apple App Store | apps.apple.com search | Same, for the post-launch iOS priority (`PRE-003`) |
| 5 | Domain | any registrar | `kimatta.com`, `.app`, `.io`. Taken-but-parked is a different problem from taken-and-in-use |
| 6 | Handles | github.com, npmjs.com, X, Instagram | Availability, and whether anything active already uses the name |

A live registration in class 9 for a similar mark is a stop. An unregistered app already on Play
under this exact name is also effectively a stop — it does not block you legally on its own, but
it makes the name useless for discovery, which is the practical reason `MealMate` was dropped.

## Deep-research prompt

Paste into Claude, ChatGPT or Gemini with web/deep-research enabled. The citation requirement is
the load-bearing part: **LLMs confidently invent both registrations and their absence**, and an
uncited "no conflicts found" is worth nothing.

---

> You are performing a preliminary trademark screen. You are not giving legal advice.
>
> **Mark:** `Kimatta` (word mark, standard characters). Japanese 決まった, "it is decided".
> **Goods/services:** downloadable mobile application software for household meal planning,
> recipe storage, pantry tracking and shopping-list generation. Nice class 9, possibly 42.
> **Territory:** United States primarily; note EU and Japan if anything material appears.
>
> Search and report on each of the following, **separately**:
>
> 1. **USPTO** — live and dead records for `Kimatta` and for phonetically or visually similar
>    marks (`Kimata`, `Kimato`, `Kimatsu`, `Kismet`, `Kinata`, `Kimara`). Prioritise classes 9,
>    35, 42 and any food/grocery class.
> 2. **EUIPO** and **WIPO Madrid Monitor** — the same terms.
> 3. **Common-law use** — any company, product, app or service trading as Kimatta in the US,
>    registered or not. Include Google Play and the Apple App Store.
> 4. **Domains and handles** — kimatta.com/.app/.io, and github/npm/X/Instagram.
> 5. **Japanese-language considerations** — 決まった is an ordinary word. Note any Japanese
>    company using it as a brand, and whether ordinary-word status weakens distinctiveness.
>
> **Evidence rules, which override any instinct to be helpful:**
>
> - Every claim about a registration MUST carry a **serial or registration number and a direct
>   link to the official record**. USPTO TSDR, EUIPO eSearch, or Madrid Monitor only.
> - Do **not** infer a mark exists from a blog, aggregator, listicle or law-firm article. Those
>   may be used to *find* a lead, never to confirm one.
> - If you cannot open the official record, say **"UNVERIFIED — could not access primary
>   source"** for that item. Do not substitute a summary.
> - "No results found" is only reportable if you state **which database you searched, with what
>   query, and what the result count was**. Otherwise report "NOT SEARCHED".
> - Do not soften or round conclusions. A partial search is more useful reported as partial.
>
> **Output:** a table with columns — Mark | Owner | Serial/Reg no. | Status | Class | Link |
> Similarity (high/medium/low) | Why. Then a short section listing what you could not verify and
> what a human must check by hand. Then a one-paragraph bottom line that explicitly states it is
> a screen, not a clearance.

---

## Findings

Fill in as the checklist is worked. `DEC-004` needs a recorded resolution with rationale, and
this table plus a dated decision is that record.

| Date | Source | Query | Result | Verified against primary source? |
|---|---|---|---|---|
| 2026-09-04 | General web search | `"Kimatta" trademark registration` | No Kimatta mark surfaced; nearest KIMING, KIMAIRY | No — indexed pages only |
| 2026-09-04 | General web search | `Kimatta app OR company OR brand` | No Kimatta entity surfaced | No — indexed pages only |
| 2026-09-04 | Justia trademarks | `kimatta` | **NOT SEARCHED** — HTTP 403 to automated fetch | n/a |
| 2026-09-04 | **DNS `A`/`MX`/`NS`** | `kimatta.com` | **TAKEN, live mail** — `210.134.168.2`, `mx.kimatta.com`, `canonet.ne.jp` NS | **Yes — authoritative DNS** |
| 2026-09-04 | **HTTP** | `http://kimatta.com` | **403**, Shift_JIS page, `Last-Modified 2008` — dormant site, live domain | **Yes — direct request** |
| 2026-09-04 | **DNS `NS`** | `kimatta.app`, `.io`, `.dev` | **No delegation** — very likely unregistered | **Yes — authoritative DNS** |
| 2026-09-04 | Deep research — `deep-research-report.md` | Full prompt | No exact KIMATTA; found kimatta.com third-party use; registries NOT SEARCHED | No — leads only |
| 2026-09-04 | Deep research — `kimatta-trademark-screening-plan.md` | Full prompt | No live federal Class 9/42 KIMATTA; raises Doctrine of Foreign Equivalents | No — leads only |
| — | USPTO Trademark Search | 7 terms, classes 9/35/42/29/30/43 | **NOT SEARCHED** — owner action | — |
| — | EUIPO eSearch Plus | 7 terms | **NOT SEARCHED** — owner action | — |
| — | WIPO Madrid Monitor | 7 terms, fuzzy + phonetic | **NOT SEARCHED** — owner action | — |
| — | Google Play / App Store | `Kimatta`, in-store search | **NOT SEARCHED** — owner action | — |

### Bottom line for DEC-004

Nothing found disqualifies `Kimatta`, and the practical failure that killed `MealMate` — an
existing app of the same name owning the search result — **does not appear to repeat here**.
Two live issues to carry forward: the `.com` is gone to a Japanese business (annoying for
launch logistics, not a rights problem on this evidence), and the Doctrine of Foreign
Equivalents makes a descriptiveness refusal a real possibility at filing.

**This does not close DEC-004.** The registry searches are the substance of the decision and
none of them has been run. What is recorded here is sufficient to justify using the name on a
sideloaded test build shared with known testers; it is not sufficient to bind the name to
Firebase (`MVP-018`), App Links (`MVP-021`) or a Play listing (`MVP-022`), which is precisely
the boundary D-019 drew.

## Related design-mark concern, logged here because it belongs to the same decision

The wordmark lockups in `docs/brand/` place a **tapered orange crescent that curls upward at its
right terminus** beneath the K. That shares orange colour, curvature, sub-wordmark placement and
the upward hook with Amazon's smile-arrow device, which Amazon registers separately from its
wordmark and enforces aggressively — and Amazon Fresh and Whole Foods put the goods/services
proximity to a meal-planning app closer than it first looks. Whole-mark confusion between
`Kimatta` and `Amazon` remains implausible, since the word is the dominant element.

**Disposition, 2026-09-04:** the lockups are not used anywhere. The shipped launcher icon
contains no such element — its orange is a roofline and a steam wisp — so nothing distributed
carries the exposure. Revisit if the wordmark is ever used publicly; the cheap fix is to drop
the crescent.

Separately: the brand art is AI-generated. US copyright generally does not attach to purely
AI-generated images, so the artwork itself is weakly held even though trademark rights can still
arise from use in commerce. Worth resolving before the name and mark become the public identity.
