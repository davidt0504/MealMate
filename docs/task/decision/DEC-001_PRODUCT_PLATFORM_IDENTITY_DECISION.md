# DEC-001 — Product and platform identity

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Decision |
| Workstream | Product foundation |
| Depends on | — |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No implementation until Done |
| Recommended workflow | `deep-options`; then `grill-me` only for unresolved owner tradeoffs |
| External actions | None |

## Workflow gate

Before resolving `DEC-001`, read `docs/ROADMAP.md`, confirm every declared dependency is Done with evidence, and confirm the roadmap identifies this card as the current required decision or explicit owner-paced work. This decision does not need to be the Next implementation task and does not occupy the implementation lane. Its result and Done transition require explicit owner approval.

## Outcome and user value

Lock the minimum identity needed to create a clean Android scaffold without accidentally preserving legacy assumptions or blocking a later iOS client.

## Authoritative sources

- `docs/PRD_v2.md` §§2–7, 14.1, 23–24
- `docs/ROADMAP.md` D-004, D-007, D-015
- `docs/task/MVP_INVARIANTS.md`

## Locked constraints

- Android is the MVP/initial-launch client; iOS is post-launch priority.
- The old implementation is disposable; preserve only `.git`, docs, license, and deliberately selected product metadata.
- Flutter remains the client framework; web exists in MVP only for public share preview/endpoints.
- Production package/application identity must be durable and non-placeholder; any pre-production scaffold identity must be unmistakably temporary and traceable to the card that replaces it. *(Amended 2026-08-21 — see Resolution §1.)*

## Decisions to resolve

1. ~~Final user-facing product spelling and Android application/package identifiers.~~ **Descoped 2026-08-21 to `DEC-004`** — see Resolution §1.
2. Minimum supported Android SDK/device posture, based on current Flutter/Firebase compatibility and intended users.
3. Whether repository-level web support is scaffolded initially or added only with MVP-020.

## Options and tradeoffs

- Scaffold Android only: smallest surface and clearest MVP boundary; web must be added later.
- Scaffold Android plus minimal web: reduces later setup friction; creates a second platform surface earlier.
- Keep placeholder identifiers temporarily: fastest today; costly migration risk once Firebase, links, and signing depend on them.

## Completion criteria

- Each decision has one choice, rationale, rejected alternatives, and reversal cost.
- No decision implies iOS work, production resources, credentials, signing, or store submission.
- Record the result in `docs/ROADMAP.md`; promote MVP-001 only after DEC-001 and PRE-001 are Done.

## Resolution — 2026-08-21

Resolved via `deep-options` and `grill-me`. Decision 1 is split out to `DEC-004`; decisions 2 and 3 are final.

> **Amended 2026-08-21:** locked constraint 4 and decision 1 were rewritten to match this resolution (`docs/ROADMAP.md` D-019). No dependent card's inputs changed; every card affected by D-019 was already Draft. Completion criteria are unchanged.

### 1. Product spelling and identifiers — deferred to DEC-004

- **Choice:** `MealMate` is retired as the intended public brand. It remains a temporary internal codename only. The final public name and production application/package ID are deferred to `DEC-004` pending naming clearance. Until then the scaffold uses a clearly temporary development identity: `applicationId dev.mealmate.temp`, Dart package `meal_mate`, display name `MealMate (dev)`.
- **Rationale:** A collision check (2026-08-21) found a Play search-index entry for `com.mealmate.app` whose listing URL now returns 404. The index hit is the load-bearing evidence, not the 404: an indexed listing existed and was later unpublished, and Play never releases a published package ID — a 404 on its own is what any never-published identifier returns. Clearance results for `MealMate`, labelled per `DEC-004`'s checklist: Play package ID — **FAIL** (index entry above); Play title — **FAIL** (≥5 apps titled MealMate); web/search — **FAIL** (same-concept competitor at mealmateco.com); trademark — **NOT VERIFIED** (web-sourced USPTO 97352318, kitchen appliances class; no authoritative TESS search performed); Apple App Store title — **NOT VERIFIED** (not performed). The package ID is permanent once on Play, so no `com.mealmate.*` identifier may be committed.
- **Rejected:** `com.mealmate.app` (consumed); `com.<owner-domain>.mealmate` and `app.mealmate.household` (both keep a crowded brand that the owner declined to accept as an MVP-022 marketing problem); keeping `com.example.meal_mate` (rejected by Play, and the placeholder MVP-001 exists to remove); probing Play Console for ID availability (outside authority boundary).
- **Reversal cost:** The dev ID is disposable by design. It must be replaced before any card binds an identifier to a real Firebase project, App Links, signing, or Play (`MVP-018`, `MVP-021`, `MVP-022`); after Play submission the production ID is irreversible. The reverse-DNS reading `temp.mealmate.dev` was considered and accepted: `MVP-001` through `MVP-017` are emulator-only (D-008) and `DEC-004` gates the first card that registers a package name with any external system, so the identifier never leaves the development machine.

### 2. Minimum Android SDK — pin `minSdk 24`

- **Choice:** Declare `minSdk 24` explicitly in the Android build; `targetSdk`/`compileSdk` inherit the current Flutter defaults.
- **Rationale:** Android 7.0+ covers ~99% of active devices, satisfies current FlutterFire floors, and makes a value Firebase and Play both care about explicit rather than inherited. `PRE-001` confirms the upgraded Flutter default does not already exceed 24.
- **Rejected:** Inherit `flutter.minSdkVersion` (implicit, can shift silently on upgrade); pin 28–30 (excludes older household devices for no MVP feature).
- **Reversal cost:** One line.

### 3. Web scaffolding — Android only

- **Choice:** `MVP-001` scaffolds Android only. Web is added, if at all, at `MVP-020`, which must first decide whether share previews are Flutter Web or static Hosting pages.
- **Rationale:** Smallest surface and clearest MVP boundary (invariant 16). Scaffolding web now pre-judges `DEC-003` and taxes every plugin choice with web support for cards that do not need it.
- **Rejected:** Android plus minimal web now.
- **Reversal cost:** One `flutter create --platforms=web .` on a clean scaffold.

No decision implies iOS work, production resources, credentials, signing, or store submission.
