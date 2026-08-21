# DEC-004 — Naming clearance and production identity

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | Draft |
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
- Recorded in `docs/ROADMAP.md`; `MVP-018`, `MVP-021`, and `MVP-022` remain blocked until Done.

## Stop/failure conditions

- Stop if clearance requires a paid search, legal opinion, or purchase; record as NOT VERIFIED and return to the owner.
