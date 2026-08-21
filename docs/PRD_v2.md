# MealMate — Product Requirements Document (MVP v2)

**Status:** Draft v2 · 2026-08-20  
**Scope:** MVP requirements, architectural requirements, and explicitly phased product hypotheses

---

## 1. Product vision

MealMate is a mobile-first household food-planning app whose primary purpose is to reduce the cognitive labor required to feed a household and return time to the people doing that work.

The MVP centers on a simple recurring loop:

**recipes → meal plan → pantry-aware shopping list → shopping → repeat**

MealMate should become progressively more useful as it learns a household's preferences, routines, constraints, traditions, and planning patterns, while requiring as little explicit maintenance as possible.

**One-line pitch:** Spend less time figuring out food. MealMate helps your household plan, shop, and get dinner handled.

**Long-term product promise:** MealMate should be capable of moving households through a progression from **manual → assisted → delegated** planning, without making generative AI a dependency of the core experience.

---

## 2. Scope labels

To prevent long-term ideas from inflating the MVP, requirements in this document use three labels:

- **MVP requirement** — required for the initial public product.
- **Architectural requirement** — the MVP must preserve the data model, API seam, or UX direction needed for this capability, but the complete feature does not need to ship yet.
- **Future hypothesis** — strategically promising, but not authorized MVP scope without new evidence or an explicit scope decision.

---

## 3. Product principles

These principles are product-level constraints and should resolve future design disagreements.

1. **Reduce cognitive labor above all.** MealMate exists to reduce the time, decisions, remembering, coordination, and maintenance involved in feeding a household.
2. **Return time; do not capture attention.** Time-in-app is not a success metric. A five-minute planning session that replaces an hour of work is a better outcome than high engagement.
3. **Frictionless by default.** Do not require constant data entry, maintenance, or repeated confirmation to keep MealMate useful.
4. **Voluntary enrichment.** Additional information should usually be optional, clearly explained, easy to provide later, and collected because it improves the user's experience—not because the app wants more data.
5. **Learn instead of interrogate.** Infer preferences and routines from ordinary behavior where it is reliable to do so; ask users only when explicit input materially improves the result.
6. **Automation over configuration.** Prefer systems that learn or infer to large settings panels, while always leaving understandable controls available for households that want them.
7. **Algorithms before ML; ML before LLMs.** Use deterministic code when rules can solve the problem, lightweight ML when learned patterns create real value, and LLMs only where unstructured or generative tasks justify their latency and cost.
8. **Fast and local.** Core planning intelligence must not depend on network availability or remote model inference.
9. **Assist, do not control or manipulate.** Users can override, move, replace, skip, or ignore suggestions without friction, guilt, artificial penalties, streak loss, or dark patterns.
10. **Healthy eating should emerge naturally.** MealMate may gently favor better overall meal balance when similarly desirable choices exist, but it is not a diet, calorie-tracking, weight-loss, or medical nutrition app.
11. **Optimize the household, not an abstract user.** Shared food decisions may involve multiple people with different preferences, roles, restrictions, and philosophies about who gets input.
12. **Household conventions are data.** Shopping days, recurring traditions, fixed meals, typical leftovers, and other routines are useful inputs because remembering them is part of the cognitive burden MealMate is trying to remove.
13. **Adapt to the household's workflow.** MealMate should not assume every family plans Sunday–Saturday, shops once a week, seeks consensus, or plans all three meals.
14. **Social utility over social engagement.** Sharing should transfer useful food knowledge or coordinate real households, not evolve by default into an attention-maximizing public social network.

---

## 4. Target users and jobs to be done

### 4.1 Primary user

**The household food planner:** the person or people who carry most of the recurring mental work involved in deciding what the household will eat, checking what is already available, building grocery lists, accounting for schedules and preferences, shopping, and repeating the process.

This includes couples who plan together as well as households where one person does most of the planning.

### 4.2 Secondary users

- Solo cooks who want a lightweight planner and shopping workflow.
- Other household members who contribute preferences, recipes, shopping-list edits, or planning input.
- Managed child household profiles that influence recommendations without requiring a full independent account. **[Architectural requirement; UI is post-MVP.]**

### 4.3 Core jobs

MealMate should reduce the work involved in answering:

- What should we eat?
- Will the people eating it actually want it?
- Do we already have what it needs?
- What do we need to buy?
- Does it fit the time and circumstances of that day?
- Have we eaten this too recently?
- Can ingredients be reused across the week?
- Are there meals, traditions, restrictions, or events that must be respected?

---

## 5. Current codebase state

As documented in v1, the repository is effectively greenfield:

- existing recipe model uses parallel ingredient/measurement strings and is superseded by the structured model below;
- the existing recipe form is unfinished and unpersisted;
- `main.dart` is still a template/demo entry point;
- the project is pinned to an obsolete Flutter/Dart generation and must be upgraded to a current supported toolchain before feature work;
- no production state-management, persistence, backend, or app architecture is established.

**MVP prerequisite:** upgrade Flutter/Dart and regenerate/update platform scaffolding before feature implementation.

---

## 6. MVP goals and success metrics

### 6.1 Product goal

Demonstrate that MealMate can create a repeatable household planning habit by materially reducing the effort required to turn meal ideas into a usable grocery list.

### 6.2 Business goal

Maximize useful adoption and retention first, while instrumenting the product so monetization, packaging, and pricing can be tested later without rebuilding entitlement architecture.

### 6.3 North-star behavior

**Recurring planning-cycle completion:** percentage of activated households that generate a shopping list from a meal plan in at least three of their first four planning cycles.

A planning cycle is anchored to the household's configured planning/shopping rhythm rather than a hard-coded Monday–Sunday week.

No external retention benchmark is treated as a target until MealMate has its own baseline data.

### 6.4 Activation funnel

Track separately rather than collapsing activation into one brittle event:

1. first recipe saved/created;
2. first meal added to a plan;
3. first shopping list generated;
4. first completed planning cycle;
5. first repeat planning cycle.

When household collaboration ships, add:

6. household invitation sent;
7. household invitation accepted;
8. second household member becomes active.

For social sharing, prefer utility conversion metrics such as **share opened → recipe saved → recipe planned** rather than raw views or time spent browsing.

### 6.5 Cognitive-labor outcome metrics

Track product outcomes that align with the mission:

- median time from beginning an intentional planning session to plan/list readiness;
- number of manual decisions/actions required to produce a usable plan and list;
- plan edits/swaps before acceptance when recommendation features ship;
- proportion of planning sessions completed without network access;
- optional user-reported prior planning time, supplied voluntarily in settings or value-tracking UI.

**Anti-metric:** time spent in app is not optimized upward.

### 6.6 Value ledger

MealMate should distinguish verified observations from estimates.

**Verified examples:** planning-session duration, meals planned, pantry items reused, ingredient overlap, leftover meals intentionally planned, list items consolidated.

**Estimated examples:** grocery cost, money saved, food waste avoided, time returned versus a user-provided baseline.

Estimated values must be labeled as estimates and should become more personalized only as real household history accumulates.

---

## 7. MVP experience

### 7.1 Minimal, optional onboarding **[MVP requirement]**

Onboarding must be short, skippable where possible, and immediately lead to a usable app.

Required or near-required setup should be limited to information necessary for correct behavior:

- **Planning for:** `Just me` or `My household` — both create the same underlying household data structure; this is a UX distinction.
- Optional household name.
- A clear, skippable opportunity to record allergies/dietary exclusions. If skipped, MealMate must not imply that recommendations have been allergy-checked.

Do **not** require a long preference, cuisine, skill, budget, or lifestyle questionnaire.

### 7.2 Voluntary tuning **[Architectural requirement; first recommendation release]**

Provide an easy-to-find area such as **Tune MealMate** / **Planning Preferences** where users may voluntarily provide additional information that improves recommendation quality.

Potential inputs include:

- rapid, playful meal-preference calibration;
- usual shopping/planning day;
- meals to plan (dinner, breakfast, lunch);
- planning philosophy / whose preferences influence recommendations;
- estimated pre-MealMate planning time;
- budget sensitivity;
- desired novelty/variety;
- other household conventions.

The experience should make the benefit of each input clear and allow users to stop at any time.

### 7.3 Meal scope **[MVP requirement]**

- Dinner is enabled by default.
- Breakfast and lunch are independent optional meal types, enabled unobtrusively from planning preferences.
- The UX must avoid presenting a large grid of empty meal slots that creates guilt or maintenance pressure.
- The data model must treat meal type as first-class so recommendation logic can later use different repetition/variety expectations for breakfast, lunch, and dinner.

### 7.4 Planning cycle **[MVP requirement]**

MealMate must not assume Monday–Sunday or Sunday–Saturday is the meaningful planning period.

- A household may optionally choose its usual grocery/planning day.
- The default seven-day planning cycle is anchored to that day when configured.
- If no day is configured, use a sensible locale-aware default without requiring setup.
- Plan entries use real calendar dates; the planning-cycle boundary is a presentation/organization rule rather than the identity of the meal.

**Architectural requirement:** the shopping-schedule model must not make a second recurring grocery trip impossible later, even though MVP exposes at most one primary shopping day.

### 7.5 Recipes **[MVP requirement]**

Create, edit, delete, and view recipes with:

- title;
- structured ingredient lines;
- serving size;
- instructions;
- optional photo;
- source/provenance metadata;
- household ownership.

#### Entry paths

1. **Manual entry:** instant ingredient autocomplete against the catalog, with household custom items on no-match.
2. **Structured deterministic import — stretch:** when a URL exposes reliable structured recipe metadata, import it without an LLM, normalize it, and require user confirmation before save.
3. **Cloud-LLM fallback import — not required for MVP:** paid/entitled future capability for messy text, difficult pages, images, or other unstructured sources.

Parsing/enrichment is a one-time ingestion operation. Saved recipes become ordinary structured MealMate data and must never require an LLM for normal use.

### 7.6 Starter pack **[MVP requirement]**

Seed approximately 30–50 curated weeknight household recipes as read-only starter content.

Purposes:

- prevent an empty first-run experience;
- provide immediate planning material;
- exercise ingredient-catalog coverage;
- later provide known metadata for cold-start recommendation testing.

### 7.7 Smart pantry **[MVP requirement]**

Use a simple **have / don't-have** model.

- pantry use is optional; an empty or incomplete pantry never blocks planning/list generation;
- no required quantities;
- `present` means "assume I do not need to buy this ingredient," not "MealMate knows I have enough";
- shopping-list generation can omit items the household says it has, with an easy way to add an omitted item back when the household needs more;
- checking off a purchased item can add it to pantry state;
- one-tap removal marks an item used up/out;
- custom pantry items are household-scoped;
- pantry maintenance should happen opportunistically from normal planning/shopping flows rather than through recurring inventory-audit prompts.

Explicitly not required for MVP: exact quantities, expiry accounting, continuous inventory reconciliation.

### 7.8 Meal planning **[MVP requirement]**

- Seven-date planning view based on the current household planning cycle.
- **Unplanned this cycle** bucket for meals selected without a date.
- Dinner-first presentation; enabled breakfast/lunch types appear without creating an intimidating full matrix.
- Drag/move/reassign should be easy; "life happened" changes must never be framed as failure.
- A planned meal is conceptually a meal occurrence/slot, not permanently equivalent to exactly one recipe. The data model must support multiple components (for example a main plus a side) even if the first MVP UI optimizes for the common one-recipe case. **[Architectural requirement.]**
- The model must also be extensible to non-recipe components such as leftovers, dining out, a freeform meal, or no meal planned. **[Architectural requirement; full UI may be phased.]**
- Fixed/locked state must exist in the schema so future automation can respect immovable meals. Do not add a lock control to MVP unless a shipped feature can actually move/replace meals automatically. **[Architectural requirement.]**

Recipes retain their default serving size. A planned recipe component may optionally override planned servings so shopping-list quantities can scale when numeric quantities are available. **[MVP data support; keep the UI lightweight.]**

### 7.9 Shopping list **[MVP requirement]**

Generate a list from all relevant recipe components in dated and unplanned meals for the active cycle.

- scale numeric ingredient quantities when a planned-servings override differs from the recipe's default servings;
- group by store category;
- subtract pantry presence;
- allow manual add/remove/check-off;
- checked purchased items may flow into pantry state;
- consolidate only when ingredient identity and units can be combined with adequate confidence;
- convert/sum only within explicitly supported compatible unit families; preserve original quantity/unit text for display/recovery;
- if quantities/units are incompatible or uncertain, preserve separate lines rather than risk an incorrect total.

### 7.10 Recipe sharing **[MVP requirement]**

Any shareable recipe may generate a normal HTTPS share URL.

- For the Android MVP and initial launch, Android App Links should open the relevant in-app destination. iOS Universal Links are a post-launch requirement alongside the iOS client.
- If the app is not installed, show a lightweight web preview with an install/open prompt.
- Do not depend on Firebase Dynamic Links.
- Do not promise cross-install deferred deep linking in MVP; evaluate a dedicated provider or a separately engineered flow only if conversion data justifies it.
- Inbound recipe viewing must not require an account.
- Outbound creation of durable shared content may require upgrading the anonymous account to a durable sign-in.

### 7.11 Social boundary **[MVP requirement]**

Social features exist to help real people exchange useful meal information.

MVP does **not** include:

- public feed;
- follows;
- likes as public popularity signals;
- public comments;
- infinite-scroll discovery;
- engagement-ranking algorithms.

A report mechanism must exist where received user-generated shared content requires it. Sharing must expose only the recipe/share object deliberately selected by the user and must not leak household preference signals, restrictions, pantry state, member identities, or other private household context.

---

## 8. Household-first domain model

### 8.1 Ownership model **[MVP architectural requirement]**

Every account belongs to at least one Household object, even if the household contains only one user.

**Household-owned operational state:**

- recipes;
- custom ingredients;
- pantry;
- meal plans;
- shopping lists;
- planning settings;
- shared meal history when it exists.

**User-owned state:**

- identity/authentication;
- personal app/account settings that are not household-specific;
- purchase/account metadata.

**Household-member-profile state:**

- individual food preferences and feedback;
- household-specific restrictions/roles;
- other recommendation signals that may differ across households.

Preference data should attach to the household member/profile context rather than globally to a user so future multi-household membership does not force one preference profile onto every household.

**Future architectural flexibility:** a user may eventually belong to more than one household, but the MVP UI does not need to expose multi-household management.

### 8.2 Multi-user collaboration **[Future — high-priority early post-MVP capability]**

When collaboration ships:

- multiple authenticated users can join one household;
- recipes, pantry, plans, and shopping lists synchronize;
- individual preference profiles remain distinguishable;
- basic household collaboration is a free capability, not a premium gate;
- joining/inviting requires durable authentication.

### 8.3 Managed household-member profiles **[Future hypothesis]**

Parents may create managed profiles for children or other non-account household members.

These profiles may hold preferences/restrictions without requiring a device/account. A later limited-permission account may claim or link to such a profile when appropriate.

Preference feedback may be socially anonymous to other household members while remaining member-specific for computation.

### 8.4 Household planning philosophy **[Future hypothesis]**

Do not assume every household seeks consensus.

Potential user-facing philosophies include:

- everyone strongly influences recommendations;
- parents/adults choose while children influence but do not veto;
- primary planner leads;
- variety/exposure is intentionally weighted more heavily;
- advanced custom behavior.

Do not require this during onboarding. Use a neutral default and expose it only as voluntary tuning.

Hard safety restrictions are never overridden by a planning philosophy.

---

## 9. Data model spine

The exact Firestore document layout is an implementation detail, but the domain model must preserve these concepts.

### 9.1 Core entities

**Household**
`{id, name?, createdAt, planningPreferences, shoppingSchedule?, defaultServings?, ...}`

**HouseholdMembership**
`{householdId, userId, role, status, ...}`

**HouseholdMemberProfile**
`{id, householdId, linkedUserId?, displayName?, managed?, restrictions[], ...}`

**User**
`{uid, foundingUser?, purchaseMetadata?, ...}`

**HouseholdEntitlement / SubscriptionState**
`{householdId, plan, complimentary?, featureEntitlements, llmQuota, purchaserUserId?, ...}`

Shared premium capabilities should resolve at the household level so a household is not accidentally required to buy separate subscriptions for each participating member. Store billing ownership separately from effective household entitlement.

**Ingredient**
`{id, canonicalName, aliases[], storeCategory, allergenMetadata?, ...}`

**CustomIngredient**
`{id, householdId, name, storeCategory?, ...}`

**IngredientLine**
`{quantity?, unit?, ingredientRef|customRef, note?, originalText?}`

**Recipe**
`{id, householdId, title, lines[], servings?, instructions, photo?, provenance, metadata?, ...}`

**RecipeProvenance**
`{type: authored|imported|starter, sourceUrl?, sourceName?, sourceAuthor?, importedAt?}`

**PlannedMeal**
`{id, householdId, scheduledDate?, mealType, label?, locked, status?, source?, components[], ...}`

**MealComponent**
`{type: recipe|freeform|leftovers|diningOut|other, recipeRef?, label?, plannedServings?, ...}`

The MVP UI may emphasize one recipe per meal, but the persistent model must not require that assumption.

**PantryItem**
`{id, householdId, ingredientRef|customRef, present, updatedAt}`

**ShoppingList / ShoppingListItem**
Fine-grained items, not one monolithic mutable array.

### 9.2 Preference data **[Architectural requirement]**

Member-level preference signals must be capable of representing at least:

- `love`;
- `notForMe`;
- `dontSuggest`;
- unknown/no signal.

Do not treat absence of positive feedback as dislike.

Store explicit and inferred evidence separately or with confidence/source metadata so the system can distinguish a direct user statement from a weak behavioral assumption.

### 9.3 Planned versus actually eaten **[Architectural requirement]**

The model must not permanently equate a planned meal with a confirmed meal.

Potential status states include:

- planned;
- assumedCooked;
- confirmedCooked;
- skipped;
- moved;
- replaced.

Until explicit feedback exists, a past PlannedMeal may be treated as **probably cooked** for recency purposes, but this is low-confidence evidence and must not be treated as strong preference evidence.

### 9.4 Fixed meals and traditions **[Architectural requirement]**

`locked` plan entries represent meals that automated planning may not move or replace.

A future **MealTradition** entity should support recurring household conventions such as birthdays, holidays, weekly traditions, or annual meals. A tradition may represent one recipe, multiple meal components/a menu, or a non-recipe tradition, with behavior such as:

- always include;
- suggest automatically;
- remind only.

Recurring tradition UI is not MVP scope.

---

## 10. Deterministic intelligence and recommendation philosophy

### 10.1 Technology hierarchy

MealMate's recurring planning loop must use the least expensive/reliable method capable of solving the problem:

1. deterministic rules and algorithms;
2. heuristic scoring/optimization;
3. lightweight/local ML when enough behavioral data exists to justify it;
4. local generative models where a future desktop or sufficiently capable device makes them useful;
5. cloud LLMs only for unstructured/generative tasks where their benefit justifies cost and latency.

Core recommendation quality must not require a cloud LLM.

Preference learning should begin as household-local deterministic/adaptive logic. Any future cross-household model training using behavioral preference data requires an explicit privacy/product review; this PRD does not assume permission to pool household food behavior for ML training.

### 10.2 Recommendation objective hierarchy **[Architectural requirement]**

When assisted planning ships, optimization should use tiers rather than one uncontrolled score.

**Hard constraints**

- known declared allergy/dietary exclusions when the structured recipe data indicates a conflict; uncertainty must remain uncertainty rather than being treated as proof of safety;
- explicit `dontSuggest` where the household's planning philosophy treats it as a veto;
- impossible schedule constraints when known;
- fixed/locked meals;
- explicit user choices.

**Primary objectives**

- predicted household acceptance/satisfaction;
- schedule/practical fit;
- reduction of planning effort.

**Secondary objectives**

- appropriate variety;
- recency / avoiding unwanted repetition;
- ingredient reuse;
- pantry utilization;
- cost efficiency when data exists;
- useful leftovers;
- seasonality where useful.

**Gentle objectives**

- balanced eating / diet quality;
- controlled exploration and novelty.

Health or novelty must not override safety, explicit preference, or clear household acceptance merely to optimize a score.

### 10.3 Optimize the sequence, not isolated meals

A plan must eventually be scored as a whole.

Three individually high-scoring but nearly identical meals can make a poor week. Meal similarity should consider structured dimensions such as cuisine, primary ingredients/protein, dish form, flavor/texture profile, cooking method, effort, and recency as data becomes available.

### 10.4 Familiar novelty

The goal is neither maximum repetition nor maximum novelty.

MealMate should eventually learn each household's tolerance for novelty and likely preferred return interval for familiar meals. Initially, conservative heuristics should favor known-good meals with limited exploration; more personalized behavior can emerge from observed acceptance/rejection.

### 10.5 Household preference aggregation

Default assisted planning should combine:

1. hard restrictions;
2. strong-dislike protection;
3. overall household satisfaction;
4. fairness across the sequence/week rather than forcing every meal to maximize every member's score.

Do not use pure averaging or pure least-misery as the sole strategy.

Future user-configurable household planning philosophy may change how non-safety preferences are weighted.

### 10.6 Explainability

Recommendations should be explainable from real scoring inputs without an LLM, e.g.:

- family favorite;
- haven't had it recently;
- quick fit for this day;
- uses ingredients already on hand;
- reuses ingredients from another meal.

Do not generate post-hoc fictional reasons.

---

## 11. Cold start and preference learning

### 11.1 Cold-start strategy **[First recommendation release]**

Use a hybrid strategy:

- optional rapid preference calibration;
- strong learning from recipes users intentionally save/import;
- strong learning from meals users manually plan;
- accumulating evidence from suggestions retained, swapped, rejected, or later confirmed;
- conservative recommendations while confidence is low.

Do not gate app use behind calibration.

### 11.2 Preference calibration

A future **Tune MealMate** interaction may use a short, playful series of representative foods/meals and simple reactions rather than a long questionnaire.

The first implementation should be hand-designed for simplicity. More sophisticated active-learning question selection is a future optimization only if evidence shows it is worthwhile.

### 11.3 Feedback behavior

Primary explicit recipe reactions should remain simple, e.g.:

- **Love**;
- **Not for me**;
- **Don't suggest**.

The exact copy may change through usability testing.

### 11.4 Gentle retrospective

MealMate may occasionally offer low-pressure feedback at a contextually useful moment, such as the next planning session:

- confirm that last week's meals generally happened;
- rate a few meals;
- correct a skipped/replaced meal;
- dismiss the retrospective entirely.

Do not use frequent push notifications simply to collect recommendation data.

---

## 12. Health, dietary restrictions, allergy boundaries, and trust

### 12.1 Balanced recommendations

When recommendation features include diet-quality optimization:

- it should be transparent but unobtrusive;
- expose a clear setting such as **Balanced meal suggestions**;
- avoid calorie rings, guilt, streaks, moralized food labels, or weight-loss framing;
- evaluate the overall plan pattern rather than labeling individual meals good/bad;
- use health primarily as a gentle tie-breaker among similarly desirable options.

### 12.2 Dietary restrictions and allergens

Declared restrictions are hard planning constraints, not preferences.

However, MealMate must never represent absence of a detected allergen as proof that a recipe or packaged product is safe. Ingredient data can be incomplete, cross-contact information may be unavailable, and manufacturing can change.

**MVP positioning:** restriction-aware warnings/filtering based on known structured recipe ingredients, accompanied by clear limitations. Do not market MVP as an allergy-safety verification system.

### 12.3 Future allergy intelligence **[High-risk future hypothesis]**

Potential future capabilities may incorporate product/manufacturer/jurisdiction/source information to surface known allergen or cross-contact concerns.

If pursued:

- never use a binary "safe" claim without an evidence standard capable of supporting it;
- surface source, freshness, jurisdiction, and uncertainty;
- treat unknown as unknown;
- design this subsystem under a separate high-stakes safety review before launch.

### 12.4 Privacy

- minimize collection of household behavioral data;
- use local processing when practical;
- do not sell household meal/preference data for advertising;
- do not build advertising profiles from household food behavior;
- provide understandable deletion/export controls before mature public launch;
- collect sensitive or health-adjacent information only when needed for user-requested functionality.

---

## 13. Recipe provenance and content boundaries

MealMate must distinguish user-authored content from imported third-party content.

- store source URL/name/author where available;
- preserve provenance across internal copies where feasible;
- do not automatically publicly republish full third-party imported recipe text/photos merely because a user imported it privately;
- user-authored recipes may be shared in full subject to normal content rules;
- imported-recipe share flows should preserve attribution and may direct recipients to the original source rather than exposing copied protected content publicly;
- final sharing behavior for imported content must receive a launch-time policy/legal review.

MealMate is not intended to become a public recipe-content publisher in the MVP.

---

## 14. Technical architecture

### 14.1 Client

- Flutter with Android as the MVP and initial-launch client; iOS is the first post-launch platform priority.
- Android remains the primary development/test target on the current Linux/WSL2 environment.
- Establish iOS CI/build/signing later, when post-launch iOS work begins and appropriate Apple hardware/tooling is available.
- Mobile remains the primary product surface.
- Full web app is post-MVP; web is required in MVP only for share previews/endpoints.
- Desktop is a future hypothesis, potentially useful as a power-user companion for local-LLM bulk ingestion/cleanup, not a dependency of the mobile product.

### 14.2 Backend

Firebase remains the initial backend:

- Authentication;
- Firestore;
- Storage;
- Cloud Functions for network/server-only tasks;
- Hosting for share preview pages/endpoints;
- Analytics;
- Crashlytics;
- Remote Config / experiment flags;
- FCM only for user-benefiting, permissioned notifications—not as an engagement KPI.

All data access remains behind Dart repository/service interfaces to preserve testability and migration seams.

### 14.3 Offline/local-first behavior

Core capabilities must remain usable without connectivity:

- local recipe access/search;
- pantry;
- active planning cycle;
- shopping list;
- list edits/check-off;
- deterministic suggestions/optimization when they ship;
- preference scoring;
- variety/recency logic.

MVP may use Firestore offline persistence as the local data mechanism, but the app must deliberately cache/prefetch all household data required for the promised offline experience rather than assuming every document has previously been cached.

Re-evaluate a dedicated local database/sync layer only if measured performance, reliability, or recommendation requirements justify its additional complexity.

### 14.4 Conflict-safe shared state **[Architectural requirement]**

Because Firestore resolves competing offline writes to the same document using last-write-wins, frequently edited shared state must use sufficiently fine-grained records.

Examples:

- one shopping-list item per record rather than one entire mutable list array;
- one plan entry per record rather than one whole-week document;
- granular pantry items;
- avoid overwriting unrelated members' changes through whole-object writes.

When household collaboration ships, test offline concurrent edits explicitly.

### 14.5 Performance requirements

No core planning interaction may block on an LLM or unnecessary network round-trip.

Engineering should establish device-level performance budgets and regression tests for:

- recipe search;
- list generation;
- planner rendering;
- meal swap/recommendation;
- full-plan optimization when introduced.

The target experience is immediate/interactive on supported phones; exact millisecond budgets should be established from device profiling rather than invented in this PRD.

### 14.6 Authentication

- Anonymous-first first launch remains preferred to avoid a sign-in wall.
- A durable account is required before joining/inviting a shared household and may be required for creating durable outbound shared content.
- Upgrade anonymous accounts without losing local/cloud household state.
- After a user has created meaningful data worth protecting, a sparse contextual nudge may offer backup/account linking; do not repeatedly nag anonymous users.
- Inbound public recipe previews are not auth-gated.
- Before a later iOS release, iOS authentication must comply with then-current App Store login-service requirements; do not hard-code an outdated policy assumption into product logic.

### 14.7 Share-link architecture

Firebase Dynamic Links is unavailable and must not be used.

Use standard HTTPS share URLs with platform App Links / Universal Links for installed-app routing and a web preview fallback.

Cross-install deferred deep linking is explicitly not an MVP requirement.


### 14.8 Store-policy / launch compliance

Before public release:

- complete the Google Play Health apps declaration accurately; meal-planning/nutrition functionality can fall within its nutrition category even when MealMate is not positioned as a diet app;
- provide the required privacy-policy/disclosure surfaces;
- before the Android public release, re-check current Google authentication, UGC/sharing, subscription, and health-related rules rather than relying on assumptions captured months earlier in the PRD;
- before a later iOS release, perform the equivalent review of then-current Apple rules.

---

## 15. Analytics, experimentation, and recommendation telemetry

### 15.1 Product analytics

Track the activation and retention funnel in §6 without turning analytics into a reason to add user friction.

### 15.2 Recommendation telemetry **[When recommendations ship]**

For each recommendation event, retain enough structured telemetry to understand algorithm quality, subject to privacy minimization:

- algorithm/version identifier;
- candidate/selected recipe IDs;
- relevant score components or reason codes;
- shown position/context;
- accepted, swapped, rejected, or ignored;
- later cooked-status evidence where available;
- confidence level.

Algorithm versions must be comparable over time.

Analytics must not receive raw allergy/restriction values, household-member names, recipe instructions, pantry contents, or other sensitive household text merely for product telemetry. Prefer coarse event properties/reason codes and keep sensitive state in the product data layer.

### 15.3 Experiments

Remote configuration/experimentation should support controlled tests of:

- onboarding/calibration shape;
- recommendation variants;
- social/share conversion flows;
- notification usefulness;
- premium packaging;
- subscription price;
- trial structure;
- paywall presentation.

Do not personalize price based on inferred wealth, vulnerability, or willingness-to-pay characteristics. Cohort experiments should be explicit product experiments, not hidden individualized pricing.

---

## 16. Monetization philosophy

### 16.1 Launch

Core MVP launches free. Monetization infrastructure should be prepared architecturally without allowing payment work to delay validation of the core planning loop.

No advertising.

### 16.2 Free product principle

**Free MealMate helps a household plan together and experience the product's core intelligence.**

Do not artificially meter deterministic features that have negligible variable cost merely to create scarcity.

Basic household collaboration, when it ships, is free because it is part of the core problem and a potential acquisition loop.

Where a paid feature acts on shared household state, Plus should normally be a household-level benefit purchased once by an eligible member rather than a separate subscription required from each spouse/member. Exact family billing mechanics remain an implementation/store-policy decision.

### 16.3 Plus principle

**MealMate Plus increasingly removes planning work from the household and funds variable-cost capabilities.**

Potential paid categories:

- proactive/delegated weekly plan preparation;
- calendar-aware automatic planning;
- advanced cost/waste optimization;
- advanced household automation;
- cloud-LLM recipe generation;
- "cook what I have" / Fridge Rescue generation from pantry + time + desired style;
- difficult text/photo/handwritten recipe extraction;
- recipe transformations/adaptations;
- bulk ingestion;
- advanced grocery-service integrations.

Premium is primarily **delegation + expensive convenience**, not "the smart version of MealMate."

### 16.4 LLM economics

Cloud LLM functionality must be entitlement/quota aware from its first release.

- no recurring core workflow should incur LLM cost;
- use deterministic extraction first where reliable;
- one-time unstructured-to-structured transformation is preferred over repeated generation;
- saved LLM outputs become normal structured MealMate data;
- generated/imported recipes must pass the same deterministic restriction checks and user-confirmation flow as any other recipe before use; this still does not constitute an allergy-safety guarantee;
- Plus may include a generous quota, with additional usage/credits considered only if real COGS requires it.

If any LLM capability is exposed before the paywall ships, it must use a bounded preview/quota or special founding entitlement rather than an unbounded promise of permanently free inference.

### 16.5 Pricing

Do not lock a final subscription price into this PRD.

Plan for controlled pricing and packaging experiments across randomized cohorts once traffic is sufficient. Evaluate long-term realized revenue/retention rather than conversion alone.

### 16.6 Entitlements

Ship entitlement scaffolding from day one:

- household-level `complimentary` for administrator-granted permanent free access where desired (including friends/family cases without changing public pricing);
- `foundingUser` for a defined permanent early-adopter benefit;
- household-level feature/product entitlements;
- household-level LLM quota/meter state with billing ownership tracked separately.

Default assignment intent: users who establish a durable account before the public paywall launches may be tagged `foundingUser`. `foundingUser` does not automatically mean lifetime access to every future premium feature; the permanent benefit must be explicitly defined before monetization launches.

---

## 17. Post-MVP roadmap / hypotheses

This section preserves direction without authorizing MVP scope.

### Phase 1.5 — Assisted planning

- optional Tune MealMate calibration;
- explicit member preference reactions;
- meal history and gentle retrospective feedback;
- basic deterministic recommendation ranking;
- recency and simple variety heuristics;
- simple whole-cycle "suggest meals" / fill-empty-slots assistance;
- basic value-ledger reporting.

### Phase 2 — Household collaboration and richer routines

- free multi-user household sync;
- shared pantry/plan/list editing;
- member-specific preference signals;
- managed non-account household profiles;
- collections and richer relationship-based recipe sharing;
- optional quick household pick/voting interactions when they reduce coordination rather than create another chore;
- optional household planning philosophy;
- recurring MealTraditions;
- richer leftovers/non-recipe meal handling;
- improved shopping schedules/multiple trips if demand supports them.

### Phase 2.5 — Advanced deterministic intelligence

- whole-plan optimization rather than isolated meal ranking;
- learned novelty tolerance and meal return curves;
- week-level household fairness;
- richer meal-similarity modeling;
- transparent balanced-eating optimization;
- schedule/calendar-aware fit for assisted recommendations;
- cost/ingredient/waste optimization as reliable data becomes available;
- personalized value/savings comparisons against household history;
- lightweight/local ML only where it demonstrably improves recommendations.

### Phase 3 — Delegated MealMate Plus

- proactively prepared weekly plans;
- premium use of calendar/schedule context for proactive or automatic planning;
- optional automatic reshuffling around household schedules;
- premium LLM Fridge Rescue / recipe generation;
- advanced recipe import/transformation;
- grocery-service integrations where commercially useful;
- pricing/package experimentation and mature subscription funnel.

### Longer-term hypotheses

- desktop power-user companion using local LLMs for bulk recipe ingestion/cleanup;
- local generative models on capable consumer devices when quality/cost is justified;
- evidence-based allergen/manufacturer intelligence under a separate high-stakes safety design;
- actual receipt/cart integrations for stronger spending/waste measurement;
- personalized store/aisle ordering learned from shopping-list completion behavior;
- ingredient substitutions represented as equivalent / acceptable / context-dependent rather than casual generative replacements.

---

## 18. Explicit non-goals

MealMate is not being designed as:

- an AI chatbot with meal planning bolted on;
- a calorie tracker;
- a weight-loss app;
- a medical nutrition service;
- a guaranteed food-allergy safety verifier;
- an exact household inventory-management system;
- a public social network;
- an engagement-maximization feed;
- an advertising/data-broker product;
- a public recipe-content publisher;
- a desktop-first product.

---

## 19. MVP exclusions

| Capability | Status |
|---|---|
| Multi-user household collaboration | High-priority early post-MVP capability; **free when released** |
| Deterministic recommendation engine | Phase 1.5 |
| Preference calibration game | Phase 1.5 with recommendations |
| Meal history/retrospective learning | Phase 1.5 |
| Whole-week optimization | Phase 2.5 |
| Recurring traditions UI | Phase 2 |
| Managed child profiles | Phase 2+ |
| Calendar integration | Phase 2.5+ |
| Advanced health/nutrition scoring | Phase 2.5; transparent and optional |
| Quantified pantry / expiry | Not planned unless evidence demands it |
| Public recipe gallery/feed/follows/comments | Not planned by default |
| Full Flutter Web app | Post-MVP, only if justified |
| Desktop app | Long-term hypothesis |
| Cloud LLM generation/import | Paid future capability; not core MVP dependency |
| Manufacturer/supply-chain allergy intelligence | High-risk long-term hypothesis |
| Grocery commerce integrations | Future Plus/business-model hypothesis |

---

## 20. MVP cut order

If solo-development scope slips, preserve the product's core loop and architecture rather than every surface feature.

Recommended cut order:

1. deterministic URL import stretch;
2. recipe-photo support if storage/media handling becomes disproportionately expensive;
3. starter-pack breadth (not its existence);
4. nonessential social polish;
5. breakfast/lunch-specific presentation polish while preserving meal-type data support.

Do **not** cut:

- structured ingredients;
- pantry subtraction;
- shopping-list consolidation;
- manual planning;
- planning-cycle/date model;
- meal-occurrence/component model (even if MVP UI stays simple);
- household-first ownership model;
- offline core use;
- recipe provenance;
- restriction data model / honest limitation messaging.

---

## 21. Key risks and mitigations

1. **Solo-developer scope creep.**  
   Mitigation: scope labels, explicit exclusions, and phased intelligence. Long-term ideas are not implicit authorization to build them.

2. **Recommendation cold start.**  
   Mitigation: starter pack, optional calibration, learning from explicit user choices/imports, and conservative low-confidence suggestions when recommendations launch.

3. **Overfitted/annoying personalization.**  
   Mitigation: separate explicit from inferred evidence, preserve uncertainty, avoid treating one rejection as permanent unless explicitly `dontSuggest`, and allow voluntary tuning.

4. **Household preference conflict.**  
   Mitigation: hard safety constraints, strong-dislike protection, week-level fairness, and future configurable planning philosophy.

5. **Offline shared-state conflicts.**  
   Mitigation: granular records and explicit concurrent-edit testing before multi-user collaboration ships.

6. **Ingredient normalization errors.**  
   Mitigation: canonical catalog, preserve original text, conservative merging, and never combine uncertain quantities merely for a cleaner list.

7. **Allergy overclaim.**  
   Mitigation: distinguish filtering/warnings from verification; never imply unknown manufacturing/cross-contact data is safe.

8. **Recipe/content rights.**  
   Mitigation: provenance metadata, separate private import from public republication, and launch-time sharing-policy review.

9. **LLM cost creep.**  
   Mitigation: LLMs are optional premium transformations/generation, not recurring core intelligence; quotas/entitlements from first release.

10. **Notification creep.**  
    Mitigation: notifications must have clear household utility, remain permissioned, and are never used simply to manufacture engagement or collect more data.

11. **False savings claims.**  
    Mitigation: maintain verified-versus-estimated value ledger and prefer self-comparison/history over generic counterfactual claims.

12. **Firestore cost and local-data assumptions.**  
    Mitigation: repository abstraction, bounded/no-feed read patterns, proactive caching of promised offline data, and measurement before introducing heavier local-sync infrastructure.

13. **Anonymous-account data loss.**  
    Users who never attach a durable credential may lose cloud-associated access after uninstall/device loss. Mitigation: preserve anonymous-first onboarding but provide sparse, contextual backup/account-linking nudges only after the user has created meaningful value worth protecting.

14. **Health-app store-policy classification.**  
    Meal planning itself can fall within mobile-store health/nutrition declarations. Mitigation: complete required declarations/privacy disclosures accurately, avoid unsupported health/medical claims, and re-review store policy before each health-related feature release.

---

## 22. Open research / validation questions

These are intentionally not blockers for MVP:

1. Which rapid-calibration interactions provide the most recommendation lift per second of user effort?
2. What default household preference-aggregation/fairness strategy produces the lowest rejection without allowing one member to dominate?
3. Which meal-similarity dimensions best predict perceived repetition for real households?
4. How should variety/exploration weights adapt across breakfast, lunch, and dinner?
5. How much diet-quality improvement can be achieved as a gentle tie-breaker without reducing plan acceptance?
6. Which verified value metrics best predict subscription willingness?
7. Which delegated behaviors create enough perceived value to justify Plus pricing?
8. Which LLM capabilities have enough usage/value to justify their COGS?
9. Does a dedicated local database become necessary beyond Firestore offline persistence as household libraries and recommendation complexity grow?
10. Is cross-install deferred deep linking valuable enough to justify a specialized provider/implementation?

---

## 23. Decision log — v2

| Decision | v2 choice | Consequence |
|---|---|---|
| Product mission | Reduce cognitive labor and return time | Completion/outcome metrics over engagement |
| Intelligence hierarchy | Algorithms → ML → LLMs | Core planning stays fast/local; generative AI is optional |
| LLM role | Unstructured/generative boundary, usually paid | No recurring LLM dependency |
| Household model | User identity + shared Household operational state | Solo use and collaboration share one architecture |
| Collaboration pricing | Basic household sharing free | Preserves household utility and acquisition loop |
| Onboarding | Minimal + voluntary enrichment | No long intake questionnaire |
| Cold-start calibration | Optional, playful, short | Better early data without gating value |
| Preference feedback | Love / Not for me / Don't suggest | Simple member-level explicit evidence |
| Planned vs cooked | Planned may become low-confidence assumed-cooked | Recency works without nagging; preference learning stays cautious |
| Household aggregation | Safety → strong-dislike protection → group satisfaction → week fairness | Avoid pure average and pure least-misery |
| Planning philosophy | Future voluntary household setting | Supports consensus and parent-led families |
| Planning period | Household cycle anchored to shopping/planning rhythm | No hard-coded Monday/Sunday assumption |
| Meal scope | Dinner default; breakfast/lunch opt-in | Simple first experience, broader capability available |
| Fixed meals | Locked plan entry | Automation cannot move required meals |
| Traditions | First-class future recurring convention | Family culture/routines can become remembered state |
| Pantry | Have/don't-have | Avoid inventory-maintenance trap |
| Pantry participation | Optional/opportunistic | Incomplete pantry never blocks use or triggers inventory-audit chores |
| Planned meal model | Meal occurrence with one-or-more components | Supports mains+sides, traditions/menus, and future non-recipe meals without schema rewrite |
| Serving scale | Optional per planned recipe component | Keeps shopping quantities useful without quantified pantry inventory |
| Health | Transparent, unobtrusive, gentle | Better balance without becoming a diet app |
| Allergies/restrictions | Hard constraints with explicit limitations | No unsafe "safe" claims |
| Social | Relationship/utility-oriented sharing | No attention feed by default |
| Recipe import | Deterministic structured import first; LLM fallback later | Lower COGS and no AI dependency |
| Share links | Standard HTTPS + App/Universal Links | No Firebase Dynamic Links; no MVP deferred-link guarantee |
| Offline | Core data + intelligence local/offline | Network loss cannot break core planning |
| Shared sync | Fine-grained documents | Limits last-write-wins conflict damage |
| Monetization | Core intelligence free; delegation + costly LLM convenience paid | Free users experience differentiation |
| Pricing | Controlled cohort experiments | Do not hard-code price in PRD |
| Savings | Verified first, estimates labeled | Trust over inflated marketing claims |
| Notifications | Sparse, useful, permissioned | Never a growth/engagement objective by itself |
| Mobile/desktop | Android MVP/initial launch; iOS first post-launch platform priority; desktop future local-LLM companion | Optimize real planning/shopping workflow first |

---

## 24. MVP acceptance summary

The MVP is complete when a new user can, with no mandatory account wall:

1. start as an individual or household planner;
2. optionally record planning-day/meal-scope/restriction preferences without being forced through a long setup;
3. create or choose recipes stored in a household-native structured model;
4. maintain a simple have/don't-have pantry;
5. place meals into a planning cycle that respects the household's chosen start/shopping day;
6. optionally enable breakfast/lunch while dinner remains the default;
7. generate a conservative, pantry-aware, correctly grouped shopping list, including basic serving-scale math where quantities are numeric and compatible;
8. edit/check the list while offline and retain that state;
9. share eligible recipes through a web preview / installed-app link flow;
10. use the core loop without any LLM or permanent network dependency;
11. generate the analytics needed to determine whether households return and repeat the planning → list loop.

Everything beyond this acceptance summary requires its own scope decision even if this PRD preserves the architecture for it.
