# OPT-002 — Messy recipe import: photo, screenshot or URL to a reviewable recipe

> Optional planning input, not an MVP dependency or approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Optional post-MVP implementation |
| Workstream | Recipe import |
| Depends on | MVP-008, MVP-018, DEC-006; OPT-001 recommended first |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | Only when it cannot delay a required MVP or launch card |
| Recommended workflow | `deep-options` on the three decision gates, then `plan-task` (no `--auto`) |
| External actions | A cloud model API needs an owner-authorized project, key and spending cap (invariant 15); network access from the app needs the INTERNET permission decision below. Never production without `DEC-008` |

## Workflow gate

Before planning or execution, read `docs/ROADMAP.md` and apply the mandatory planning or execution gate in `docs/task/README.md` for `OPT-002`. Because this card is optional, the owner must explicitly select it without displacing required MVP or launch work.

## Outcome and user value

A household photographs a recipe card, screenshots a recipe from anywhere, or pastes a URL that has no structured metadata, and gets a filled-in private recipe to review and save. This is the messy-input boundary PRD v3 §9.1 reserves for an LLM: the model produces a *candidate*, the household confirms it, and only then does it become ordinary structured Rust-owned state. It is the feature the owner reached for when authoring starter content; starters were instead authored with Claude Code at development time (`MVP-032`), and this card records the design so the in-app version is ready when it is selected.

## Authoritative sources

- `docs/PRD_v3.md` §9.1 (LLMs at the messy boundary, never the recurring controller), §9.3 (an LLM may not silently invent a trusted action), §14 items 4–5 (external and LLM output are untrusted), §7.2 (`ImportedUntrusted` provenance), §20 ("messy recipe/photo/handwriting import" named as variable-cost premium convenience), §16 (cloud LLM dependency is Not MVP)
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §7 (candidate-action completeness), §12 (algorithm hierarchy), §21 (no artificially metered deterministic intelligence — this feature is generative, so it may be metered)
- `docs/task/optional/OPT-001_STRUCTURED_URL_RECIPE_IMPORT.md` — the deterministic lane this card falls back from; its constraint that an LLM/OCR lane produces candidates only
- `docs/task/decision/DEC-006_MONETIZATION_MODEL_DECISION.md` — whether this is a premium surface; `DEC-003` / D-037 — dev project has no billing and no Functions
- `docs/ROADMAP.md` D-041; `docs/task/MVP_INVARIANTS.md` 9, 12, 13, 15, 17

## Design (recorded now, decided at planning)

**Pipeline.** Input (camera photo, gallery image, share-sheet screenshot, or URL) → deterministic pre-pass (OPT-001's JSON-LD extraction for URLs; image normalisation for photos) → model call with a fixed output schema (title, servings, times, ingredient lines as `original_text` + parsed name/quantity/unit, instructions, detected restriction terms, per-field confidence) → deterministic validation in Rust (units in the allowed set or left as text, quantities parse, catalog match by name with unmatched names kept as custom ingredients) → review screen showing every field with its confidence and the source image alongside → save as a private recipe with `provenance.kind` imported, source captured, and the `ImportedUntrusted` marker until the household has edited or cooked it.

**Trust boundary.** The model output never touches restrictions, policies, plans or authority; it is a recipe draft and nothing else (invariant 17, PRD §14.4). Restriction warnings on the saved recipe come from the deterministic matcher, not from the model's guesses. Nothing persists before confirmation (OPT-001 AC-3 applies).

**Rights.** Imported content is private household content (invariant 12). A photo of a cookbook page is the household's own use; the recipe never enters public sharing (`MVP-020`) unless the household attests rights, per D-037's provenance-keyed rule.

**Privacy.** Images and text leave the device only for the model call, only over the authorized project, and are not retained server-side beyond the request where the provider allows that setting. Nothing about the household (members, restrictions, plans) is sent. Diagnostics exclude the content (invariant 13).

**Cost.** Variable per call. PRD §20 places this in the premium tier; the entitlement check is `MVP-027`/`MVP-028`'s and this card only consumes it.

## Load-bearing constraints

- Candidates only; deterministic validation before any persistence; review before save; no path from model output to authoritative state.
- The release manifest carries no INTERNET permission today. Adding it is a product decision (gate 1), not an implementation detail, and it must not be added by an earlier card as a side effect.
- Core planning keeps no LLM dependency (invariant 9): this feature can be absent, offline or failing with the loop unaffected.
- Provider abstraction is one adapter with one implementation until a second provider is real (no speculative interface).

## Scope

- Input capture (camera, gallery, share sheet, URL), the model adapter with schema-constrained output, Rust validation and catalog matching, the review/confirm screen, provenance and the untrusted marker, entitlement check, failure and offline messaging, adversarial fixtures (hostile images, prompt-injection text inside a screenshot, oversized inputs).

## Non-goals

- Automatic saving, batch import, public republication, training on household data, on-device model bring-up unless gate 2 chooses it, and any change to planner behaviour.

## Decision gates

1. **INTERNET permission in release.** Options: add it with this card and rely on `MVP-019`'s privacy posture; keep the manifest closed and ship this as a separate build flavour; defer the card. Resolve with `deep-options`.
2. **Cloud model vs on-device model.** Cloud is simplest and best; on-device keeps the manifest closed but costs APK size and quality. Decide against measured quality on a fixture set of real recipe photos.
3. **Premium gating.** Consume `DEC-006`'s answer; if unresolved, stop rather than assuming.

## Acceptance criteria

- **AC-1:** A photographed recipe card, a screenshot and a metadata-free URL each produce a reviewable draft with per-field confidence and the source shown alongside.
- **AC-2:** Nothing persists before confirmation; every saved field round-trips through the normal recipe model; unmatched ingredients save as custom ingredients with original text retained (invariant 2).
- **AC-3:** Model output cannot alter restrictions, policies, plans or another recipe; restriction warnings on the result come from the deterministic matcher (test with a hostile screenshot containing instructions to the model).
- **AC-4:** Offline, over quota, or with the feature disabled, the rest of the app is unaffected and the messaging is honest.
- **AC-5:** No household data beyond the input leaves the device; diagnostics exclude the content.
- **AC-6:** Rights/privacy/security review passes; the entitlement check matches `DEC-006`.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Fixture set of owner-supplied photos/screenshots/URLs; recorded results |
| AC-2 | Repository and widget tests |
| AC-3 | Adversarial fixture tests including prompt injection inside an image |
| AC-4 | Airplane-mode and quota-exhausted runs |
| AC-5 | Request payload inspection in the dev project |
| AC-6 | Fresh-context review |

## Stop/failure conditions

- Stop if this delays MVP or launch work, if any gate is unresolved on the point being implemented, if model output would reach authoritative state, or if the manifest would gain INTERNET without gate 1 recorded. Two cycles then defer.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, evidence, gate resolutions, blockers, and **Next implementation task**.
