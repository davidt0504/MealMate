# MVP-034 Phase A appearance-lab evidence

Recorded 2026-09-07 and extended 2026-09-08. The owner approved MVP-034 as an appearance-lab
deliverable and explicitly retained the lab for continued private-build evaluation. Final palette
selection and lab removal are deferred to the pre-beta gate in `MVP-022`.

## 2026-09-08 extension

- Added three explicit light/dark directions: **Tea garden**, **Rainy veranda**, and
  **Cedar · clay**. They interpret calm warmth and confident competence through tinted paper
  surfaces, low-chroma ink colors, and rare earthen accents.
- The directional reference was Fumi of Speak Japanese Naturally as a person and teacher, not
  the channel's visual branding; no artwork, likeness, or channel palette was copied.
- All 14 schemes pass the existing opacity, no-pure-black/white, and WCAG contrast assertions.
  Among the added schemes, the lowest body pair is Rainy veranda light
  `onSurfaceVariant/surfaceDim` at **5.01:1** and the lowest outline pair is Tea garden or Rainy
  veranda light `outline/surface` at **4.13:1**.
- **AC-1 PASS (expanded):** the matrix now passes all **784 cells**: 7 palettes × 4 type
  pairings × 2 brightnesses × 2 scales × 7 routes.
- Regression gates remain green after the extension: `flutter analyze` passed, full
  `flutter test` passed **388/388**, and formatting checks passed.
- Emulator visual checks at 1080×2340: `MVP-034_TEA_GARDEN.png`,
  `MVP-034_RAINY_VERANDA.png`, `MVP-034_CEDAR_CLAY.png`, and
  `MVP-034_CEDAR_CLAY_DARK.png`.

## Candidate implementation

- Seven palette choices, each with explicit light and dark `ColorScheme` tokens: Aizome · linen,
  Navy · cream, Sumi · washi, Evening kitchen, Tea garden, Rainy veranda, and Cedar · clay.
- Four type choices: Shippori Mincho + Atkinson Hyperlegible, Zen Old Mincho + Atkinson
  Hyperlegible, Zen Maru Gothic + Atkinson Hyperlegible, and Atkinson Hyperlegible alone.
- Settings owns the only preview mount point (`rg -n 'appearance-preview' lib` returns one
  match). Selection is transient Riverpod state and switches both `theme` and `darkTheme` live;
  system brightness remains authoritative through `ThemeMode.system`.
- The warm tertiary token is local to Cover's **Accept** action and its non-zero “needs you”
  banner. Focused widget assertions pin both sites. Recipe-form error reveal uses a zero-duration
  scroll when system animations are disabled, also pinned by a widget test.

## Initial automated evidence (2026-09-07)

- **AC-1 PASS:** `flutter test test/app_test.dart --plain-name "all appearance candidates
  survive the route, brightness, and scale matrix"` passed all 448 cells: 4 palettes × 4 type
  pairings × 2 brightnesses × 2 scales (100%, 200%) × 7 routes (Plan, Cover, Recipes, Recipe
  detail, Shopping, Pantry, Settings).
- **AC-2 PASS:** `flutter test test/theme_test.dart --reporter expanded` passed all eight schemes
  and prints every measured pair. The lowest 4.5-threshold pair is Evening kitchen light
  `onSurfaceVariant/surfaceDim` at **4.87:1**. The lowest tested outline pair is Evening kitchen
  light `outline/surface` at **4.06:1**, above its 3:1 threshold. Every token is opaque and no
  scheme uses pure black or pure white.
- **AC-3 PASS:** focused picker test proves both axes update the existing live `App` state.
  Emulator screenshots: `MVP-034_SETTINGS_TOP.png` (default),
  `MVP-034_AC3_NAVY_LIVE.png` (Navy selected live), and `MVP-034_AC3_DEFAULT.png` (default Cover
  with the rare Accept accent).
- **AC-4 PASS:** the eight Latin-subset faces and four unmodified OFLs are present in the release
  APK. `LicenseRegistry` and standard license-page widget tests find all four families. Emulator
  screenshot: `MVP-034_AC4_LICENSES.png` (Atkinson entry visible; the test pins all four).
- Regression gates: `flutter analyze` passed; full `flutter test` passed **376/376**;
  `git diff --check` passed.

## Font asset sizes

| Family | Regular | Bold | OFL |
|---|---:|---:|---:|
| Atkinson Hyperlegible | 48,464 B | 49,396 B | 4,352 B |
| Shippori Mincho | 128,612 B | 128,684 B | 4,399 B |
| Zen Old Mincho | 61,372 B | 60,916 B | 4,399 B |
| Zen Maru Gothic | 57,664 B | 57,740 B | 4,402 B |

Source commit, subset command, source/output hashes, and OKLCH authoring anchors are recorded in
`assets/fonts/README.md`.

## Release and device handoff

- Release build PASS: `build/app/outputs/flutter-apk/app-release.apk`, **67,757,136 bytes**,
  SHA-256 `ac0bcf4dbf5aaaf55ffe757bf30f07f408d0261659b5baee6ac303c49df846c0`.
- Pre-card baseline: 67,371,765 bytes. Phase A growth: **385,371 bytes** (the Phase B 1.5 MB cap
  applies to the stripped shipped result).
- Manifest permission check PASS: only
  `dev.mealmate.temp.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION`; no `INTERNET` permission.
- Emulator install PASS on `emulator-5554`, API 36; the app launched and produced the screenshots
  above.
- APK export PASS:
  `C:\Users\David\Downloads\mealmate-dev-20260907-1924.apk`.
- **Two owner-phone installs NOT VERIFIED:** no physical device was attached during the initial
  run. On 2026-09-08 the owner accepted this residual private-build coverage risk and approved the
  card as Done; representative physical-device release verification remains part of `MVP-022`.

## Deferred owner decision

Before `MVP-022` can ship a beta, record a dated choice of one palette and one type pairing,
confirm its light and dark schemes under `ThemeMode.system`, then strip the preview controls and
unused candidates. The private-build lab remains intentionally available until then.
