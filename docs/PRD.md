# MealMate — Product Requirements Document (MVP)

**Status:** Draft v1 · 2026-08-19

---

## 1. Product summary

MealMate is a mobile meal-planning app for busy household planners: plan the week's
meals, generate a smart shopping list that already knows what's in your pantry, and
share recipes with friends. Free at launch, built for viral, share-driven growth.

**One-line pitch:** Plan the week, shop once, never ask "what's for dinner?" again.

## 2. Target user

**Primary persona — the busy household planner.** Cooks for a family or partner,
plans weekly to control budget, time, and decision fatigue. Plans on the weekend,
shops once or twice a week, cooks most nights. Shares recipes with friends and
family naturally.

Design tie-breakers resolve in this persona's favor. Consequence: **household
sharing is the #1 Phase-2 feature.**

Secondary (served, not optimized for): solo cooks who plan loosely — covered by the
Unplanned bucket (§5.3).

## 3. Current state of the codebase

As of this writing the repo contains a fresh Flutter template plus:

- `lib/models/recipe.dart` — a `Recipe` class with parallel `ingredients` /
  `measurements` string lists. **Superseded** by the structured model in §6; no
  persisted data exists, so this is a rewrite, not a migration.
- `lib/screens/recipe_management/recipe_form_screen.dart` — unfinished form, not
  wired into the app, no persistence. Will be rebuilt against the new model.
- `lib/main.dart` — still the counter demo.
- `pubspec.yaml` pins Dart `>=2.19.6 <3.0.0` (Flutter 3.7-era, 2023). Current
  FlutterFire and ecosystem packages require Dart 3 — **upgrading Flutter/Dart
  and regenerating stale platform scaffolds is a prerequisite task** before any
  feature work in this PRD.
- No state management, no persistence, no backend, no non-default dependencies.

Everything in this PRD is therefore greenfield.

## 4. Goals and success metrics

**Business goal:** grow as fast as possible; monetize later without alienating
early users; friends & family free forever.

### Metric structure

- **North star — weekly planning retention:** % of new users who generate a
  shopping list from a meal plan in ≥3 distinct weeks within their first 28 days.
  Rationale: category research shows long-term retention in meal-planning apps is
  driven almost entirely by the weekly-plan + grocery-list habit; top-quartile
  habit-loop apps reach ~15%+ D30 retention vs the ~4% food-app baseline.
- **Guardrail 1 — activation rate:** % of installs that save a first recipe
  (authored, imported, or saved from the starter pack) AND generate a first
  shopping list within 48h. Fast dial while retention cohorts
  mature.
- **Guardrail 2 — K-factor:** installs generated per active user via share links.
  Floor target ~0.2 at MVP.
- **Phase-3 handoff:** at monetization, north star becomes LTV:CAC (target ≥3:1).

All tracked via Firebase Analytics from day one.

## 5. MVP feature set

All four README pillars ship in the MVP, each in the shape defined below.

### 5.1 Recipes

- Create, edit, delete, view recipes: title, structured ingredient lines (§6),
  serving size, instructions, optional photo.
- **Entry paths (one storage model, two doors):**
  1. Manual entry with instant autocomplete that snaps ingredient names to the
     catalog; inline creation of custom items on no-match.
  2. **LLM import (MVP-stretch, may slip to Phase 1.5):** paste recipe text or a
     URL → Cloud Function parses to structured lines snapped to the catalog →
     user confirms/edits before save. Parsing happens once at entry with human
     confirmation; stored data is always clean structured data.
- **Starter pack:** ~30–50 curated recipes (authored/curated by us) seeded as
  read-only content, targeted at weeknight household cooking. Fixes cold-start
  with zero UGC moderation; doubles as the ingredient catalog's coverage test.

### 5.2 Smart pantry

- **Have / don't-have model — no quantities.** The pantry is a checklist of
  catalog (or custom) items the household currently has.
- Shopping-list generation subtracts pantry items.
- Checking off a purchased list item adds it to the pantry.
- One-tap "used it up" removes an item from the pantry.
- Explicitly out: quantities, unit math, expiry tracking (Phase 2+ only if usage
  data demands it — quantified inventory is the category's abandonment trap).

### 5.3 Meal planning

- **Week grid + Unplanned bucket.** A plan entry is `{recipe, week, optional day,
  optional meal label}`. The week screen shows seven day rows plus an
  "Unplanned this week" bucket at the top.
- Day-level granularity, dinner-first framing; meal labels optional (no guilt
  grid of empty slots).
- Serves both workflows with one codepath: planners drag to days; queue-style
  users drop recipes in the bucket. A user-facing mode *setting* is Phase 2,
  contingent on analytics.

### 5.4 Shopping list

- "Generate list" consolidates ingredient lines across all current-week plan
  entries (dated and unplanned), merges duplicates by ingredient ref, subtracts
  pantry, and groups by store category (produce, dairy, …).
- Manual add/remove/check-off; checked items flow into the pantry (§5.2).

### 5.5 Social: share links + starter pack

- Any recipe (and, stretch: a week's plan) can be shared via deep link.
- Recipient with the app: opens in-app, can save a copy. Without the app: a
  **web preview page** (Firebase Hosting + universal links / app links — note:
  Firebase Dynamic Links is shut down; we host our own link endpoint) shows the
  recipe with an install prompt; deferred deep linking lands new installs on the
  shared recipe. The web preview is also our free SEO surface.
- A "report content" affordance on received shared content (app-store policy
  safety), but no public UGC in MVP → near-zero moderation load.
- Explicitly out of MVP: public gallery, follows, feed, likes, comments (§9).

## 6. Data model (spine)

- **IngredientLine** = `{quantity: number?, unit: string?, ingredientRef, note?}`.
- **Ingredient catalog:** ~300–500 seeded common items, each with a store
  category; plus per-user custom items (exact-name matched within the user's own
  items). The catalog is the join key for consolidation, pantry matching, and
  aisle grouping.
- **Recipe** = `{title, lines: [IngredientLine], servings, instructions, photo?,
  ownerUid, source?}`.
- **PlanEntry** = `{recipeRef, weekId, day?, mealLabel?}`.
- **PantryItem** = `{ingredientRef | customRef}` (presence = have).
- **User** = `{uid, entitlements: {complimentary?, foundingUser?},
  llmQuotaUsed, …}` — entitlement flags and the LLM quota meter exist in the
  schema **from day one** (§8).

## 7. Technical architecture

- **Client:** Flutter, Android + iOS. Android is the primary dev/test target
  (WSL2 dev machine); iOS builds via CI with a Mac runner (Codemagic or GitHub
  Actions). Full Flutter Web app: Phase 2. Desktop targets: not planned.
- **Backend: Firebase** — Auth, Firestore (offline persistence on), Storage
  (photos), Cloud Functions (LLM parse endpoint, keeps API keys off-device),
  Hosting (share/web-preview pages), FCM (push — a core growth lever), Analytics,
  Crashlytics, Remote Config (feature flags / A-B tests).
- **Lock-in mitigation:** all data access behind Dart repository interfaces
  (needed for tests anyway). A future migration swaps the data layer, not the app.
- **Offline:** Firestore's built-in offline cache must make recipes, pantry, the
  current plan, and the shopping list fully usable with no connectivity
  (kitchen / grocery-store dead zones are the primary usage context).

### Auth

- **Anonymous-first:** Firebase Anonymous Auth on first launch — real UID, full
  app functionality, zero friction. No sign-in wall (walls cost 20–40% of
  first-session users).
- **Hard gate only on outbound sharing** (a share needs durable identity);
  inbound shared links are never walled.
- Soft "back up your recipes" nudge once a user has 3+ recipes.
- Upgrade via `linkWithCredential` — same UID, all data kept, no migration code.
  Google Sign-In and Sign in with Apple (Apple requires the latter when offering
  the former).
- Known edge case (document, handle simply): linking a credential already used on
  another device fails → offer "keep cloud data" default.

## 8. Monetization

- **MVP: everything free.** No IAP code at launch.
- **Pre-declared premium categories** (so nothing free today is ever clawed
  back): LLM import beyond a generous free quota · household/shared planning ·
  advanced planning features (e.g., auto-plan suggestions). Categories, not
  exhaustive lists.
- **Entitlement flags shipped in schema now:** `complimentary` (friends & family,
  permanently free, set by admin) and `foundingUser` (everyone who signs in
  before the paywall ships gets a permanent perk, e.g. elevated LLM quota).
  Future paywall = configuration, not construction.
- **No retroactive gating:** premium-declared features ship gated or metered
  from their *first* release — household sharing launches in Phase 2 already
  under that framing, even though the paywall itself is Phase 3. A feature is
  never free first and paid later; `complimentary` and `foundingUser` users
  keep access regardless.
- Rejected: ads (hostile in cooking context, pennies at small scale), launch-day
  freemium (weeks of IAP plumbing during the growth window for noise-level
  revenue).

## 9. Out of scope for MVP (explicitly phased)

| Feature | Phase |
|---|---|
| Household sharing (shared plan/pantry/list) | **Phase 2 — first priority** (ships premium-framed per §8) |
| Public recipe gallery (pre-moderated) | Phase 2 |
| Follows / feed / likes / comments | Phase 3+ |
| Planner-vs-queue mode setting | Phase 2, data-contingent |
| Quantified pantry inventory, expiry | Phase 2+, data-contingent |
| Full Flutter Web app | Phase 2 |
| Nutrition/macro data | Not planned (off-persona) |
| Monetization/paywall | Phase 3 (north star handoff to LTV:CAC) |
| Desktop targets | Not planned |

## 10. Risks

1. **Solo-dev scope:** four pillars is a lot; the MVP shapes above are the
   already-minimized versions. If schedule slips, cut order is: LLM import →
   plan-sharing (keep recipe-sharing) → starter-pack size. Never cut list
   consolidation or pantry subtraction (they are the product).
2. **iOS from a Linux dev environment:** CI Mac runner + $99/yr Apple account is
   on the critical path; set up early, not at release time.
3. **Firestore cost curve:** per-read pricing; the repository seam and the
   no-feed MVP keep read patterns simple. Revisit before any feed feature.
4. **Anonymous data loss:** users who never upgrade lose data on uninstall;
   mitigated by nudges, accepted as residual risk.
5. **Catalog gaps:** custom items cover the tail; watch analytics for frequent
   custom-item names to fold into the catalog.
6. **LLM dependency:** import degrades gracefully to manual entry when offline
   or over quota.

## 11. Decision log (interview, 2026-08-19)

| Decision | Choice | Key alternative rejected |
|---|---|---|
| MVP scope | All four pillars, minimized shapes | Recipes+lists only |
| Backend | Firebase behind repository interfaces | Supabase (no offline story), local-first |
| Social shape | Share links + web preview + starter pack | Public gallery (moderation, cold-start circularity) |
| Pantry depth | Have/don't-have checklist | Quantified inventory (abandonment trap) |
| Planning model | Week grid + Unplanned bucket | Configurable mode setting (dual UI cost) |
| Ingredients | Structured + catalog + autocomplete + LLM import (stretch) | Free text + fuzzy matching (trust erosion) |
| Auth gate | Anonymous-first; sign-in only to share | Sign-in wall (20–40% funnel tax) |
| Platforms | Android + iOS at launch | Android-first (breaks share loop for iOS recipients) |
| Monetization | Free MVP + declared premium line + grandfathering + F&F flag | Launch freemium |
| North star | Weekly planning retention (+ activation & K-factor guardrails) | K-factor as north star (noise, leaky bucket) |
