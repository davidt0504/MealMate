# MVP-034 — Visual identity: appearance lab, owner pick, tokens

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation with a mid-card owner decision |
| Workstream | Design system |
| Depends on | MVP-003, MVP-024 |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | No |
| Recommended workflow | `plan-task`; `impeccable` for palette/token work |
| External actions | None. Fonts are downloaded once at authoring time from Google Fonts under the SIL Open Font License and committed as assets; the app never fetches at runtime (release manifest carries no INTERNET permission) |

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-034`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Completion amendment (owner direction, 2026-09-08)

The owner approved MVP-034 as the appearance-lab deliverable and directed that the lab remain
intact for continued private-build evaluation. Three further palettes were added, bringing the lab
to seven palettes and fourteen light/dark schemes. The winner selection and Phase B strip are no
longer completion criteria for this card; they are a hard pre-beta obligation in `MVP-022`. The lab
must still not ship in a beta.

## Outcome and user value

The app looks like Kimatta rather than a Material seed: calm, decided, and warm by voice. The owner
can live with every candidate combination in real use before deciding, so this card ships a
temporary appearance lab. The winner is chosen and applied at the pre-beta gate after private-build
evaluation rather than inside this card.

## Authoritative sources

- `.impeccable.md` — design context, palette and type candidates, design principles; the single source for this card's aesthetic direction
- `docs/PRD_v3.md` §2.3 (positioning without control vocabulary), principle 13 (no engagement mechanics)
- `docs/brand/` — icon and lockups (D-039: lockups stored, unused)
- `docs/ROADMAP.md` D-039, D-041
- `lib/app/theme.dart`, `lib/features/settings/settings_screen.dart`, `pubspec.yaml`
- `docs/task/MVP_INVARIANTS.md` 16

## Load-bearing constraints

- **Theme only.** No screen layout, copy or navigation changes; those belong to their own cards. The lab is a `ThemeData` factory over two axes plus a picker.
- **Two orthogonal axes.** Palette (7) × type pairing (4), each palette in light and dark (14 `ColorScheme`s). The picker holds two selections in Riverpod state; no persistence dependency is added for the lab. If reset-on-restart hinders multi-day testing, a `--dart-define` default per axis is the fallback, not a store.
- **Fonts are bundled Latin subsets** — full Mincho files carry Japanese glyphs at several MB each; Latin subsets are roughly 100 KB per face. Each family's OFL text is registered with `LicenseRegistry` so the in-app licenses page shows it.
- **Accessibility is a gate, not a preference.** Every text/surface token pair meets WCAG AA contrast (4.5:1 body, 3:1 large) in both brightnesses; every existing screen renders without overflow at 200% system text scale; reduced motion is honored. A palette that cannot meet AA is adjusted or dropped before the owner sees it.
- **The accent is rare.** In every palette the accent carries Accept and the "needs you" count and little else; no alarm red anywhere (hard constraints are stated in ink). Neutrals are tinted toward the palette's hue; no pure white or black.
- **The lab remains temporary** and may stay in private owner builds. A picker in a beta build is a stop condition; `MVP-022` owns the final pick and strip gate.
- Type candidates: Shippori Mincho + Atkinson Hyperlegible; Zen Old Mincho + Atkinson Hyperlegible; Zen Maru Gothic + Atkinson Hyperlegible; Atkinson Hyperlegible alone with hierarchy by size and weight. Palette candidates as listed in `.impeccable.md`.

### Planning clarifications (2026-09-07)

- “Theme only” permits the two Settings preview controls, one permanent standard Flutter
  licenses-page entry point required by AC-4, and wiring the accent token to the two semantic
  sites named above. It does not permit other layout, copy, route, or component redesign.
- `primary` remains quiet ink for ordinary controls; the rare accent is `tertiary`, consumed
  locally by **Accept** and a non-zero “needs you” banner so every filled button does not become
  accent-colored.
- The owner approves both light and dark schemes belonging to the selected palette. Runtime
  brightness continues to follow `ThemeMode.system`; brightness is not a third picker or a
  persisted preference.

## Scope

**Phase A — appearance lab**
- Fourteen `ColorScheme`s (seven palettes × light/dark) defined as explicit tokens, derived in OKLCH at authoring time and committed as hex; four `TextTheme` pairings on bundled fonts.
- An appearance provider and a Settings section (clearly marked as a preview) with the two pickers; theme switches live.
- Contrast test over every text/surface pair for every scheme; a golden-free widget test that pumps each primary screen at 200% text scale under each combination and asserts no overflow.
- GitHub release build for private owner evaluation, with emulator visual evidence in both modes.

**Deferred pre-beta work (`MVP-022`) — tokens and strip**
- `theme.dart` becomes the chosen palette's light and dark schemes and the chosen type pairing; the picker, provider, unused schemes and unused font assets are removed; the contrast and overflow tests stay, narrowed to the shipped tokens.
- `.impeccable.md` updated from candidates to the decision.

## Non-goals

- Screen redesign, iconography, motion design, a component library, illustration, and any theme persistence store. A later design card owns component-level work.

## Decision gates

- The owner's final pick is a pre-beta `MVP-022` gate; no workflow picks on the owner's behalf.
- If no palette reaches AA in dark without losing its character, the owner decides before beta
  whether dark ships as a desaturated variant or is deferred.

## Acceptance criteria

- **AC-1:** Every palette × brightness × type combination renders Plan, Cover, Recipes, Recipe detail, Shopping, Pantry and Settings at 100% and 200% text scale with no overflow (widget test matrix).
- **AC-2:** Every text/surface token pair in every scheme meets WCAG AA (unit test computing relative luminance contrast).
- **AC-3:** The pickers switch the live theme without restart; they appear only in the preview section and nowhere else.
- **AC-4:** Bundled fonts are Latin subsets, and each family's OFL text appears on the in-app licenses page.

### Deferred `MVP-022` obligations

- The owner's pick is recorded and dated before a beta build.
- No picker, appearance provider, unused scheme, or unused font asset remains in the beta; the
  release APK grows by at most 1.5 MB against the pre-card build; `flutter analyze`, `flutter test`,
  and the contrast test pass on the shipped tokens.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Test output for the full matrix |
| AC-2 | Test output listing each pair's ratio |
| AC-3 | Emulator recording or two screenshots; `grep` showing the picker's single mount point |
| AC-4 | Font file sizes in the handoff; screenshot of the licenses page |

## Stop/failure conditions

- Stop on an invariant conflict, stale dependency, destructive ambiguity, missing required environment, or need for unauthorized external action.
- Stop if the lab would ship in a beta (`MVP-022`) build, or if any candidate palette requires a non-OFL font or a runtime fetch.
- After two failed remediation cycles, return to planning rather than weakening acceptance criteria.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record this card's resulting status, PASS/FAIL/NOT VERIFIED
evidence, the deferred pre-beta pick/strip obligation, blockers, and **Next implementation task**.
