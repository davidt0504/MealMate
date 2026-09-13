# Flutter features

## Responsibility and state model

Feature code translates whole Rust DTOs into accessible screens and sends coarse mutations through Riverpod notifiers. Durable truth remains in Rust. Notifiers generally merge the DTO returned by a successful write into current state, so the UI reflects stored values without an unnecessary full reload.

Refresh methods preserve usable content on failure: they return the error for a snackbar and publish `AsyncError` only when no prior data exists. Family providers for cycle-relative views are auto-disposed to avoid retaining every visited offset.

## Household, onboarding, and settings

`HouseholdNotifier.build` waits for database health, then bootstraps the anonymous household. `rename` and `completeOnboarding` publish the bridge-returned household. The Welcome screen completes onboarding; household and settings screens expose naming, planning scope, restrictions, health, export, restore, and reset flows.

`healthReportProvider` resolves `kimatta.db` beneath the application support directory and opens it once through Rust. Backup state invokes Rust export/restore/reset commands plus Dart file/share adapters. After a database swap it invalidates household and every dependent domain provider so no pre-restore DTO remains authoritative.

## Recipes and restrictions

Recipe providers load active and archived summaries separately, support save/archive/restore, and invalidate dependent planner/shopping views when recipe state changes. Form/detail/list screens render structured ingredients while retaining original line text. Field helpers format typed quantities and units.

Restrictions use Rust's known vocabulary and household-scoped DTOs. Presentation copy deliberately says warnings and uncertainty, never that a recipe is safe. `restriction_warnings.dart` summarizes structured conflicts, wording-only matches, and hidden counts for recipe and planner surfaces.

Starter installation is an app-lifetime concern rather than a Welcome-only action. `starter_provider.dart` invokes the idempotent bridge operation; the app shell chooses when to start it.

## Planning and Cover My Week

`planningCycleProvider` owns the saved cycle rhythm and enabled meal slots. The cycle editor writes the complete cycle definition. Date helpers use civil-date strings rather than hard-coded week assumptions.

`plannerAnchorProvider` captures one successful `today` value shared by every offset. This avoids different family members crossing a cycle boundary independently. `PlannerNotifier` fetches the calculated window and its planned meals, then supports save, lock, delete, and refresh. Mutations republish sorted stored DTOs. The grid surfaces recipe conflicts and respects the Rust-owned one-meal-per-slot rule.

`CoverNotifier` requests a preview with `apply: false`; every preview intentionally records proposal evidence. `accept` re-runs against the current snapshot with `apply: true`, publishes the returned result, and invalidates the planner. `decide` records a swap, veto, or restrictions-reviewed correction and then re-previews. Raw engine reason tokens are not used as UI copy; slots and attention DTOs carry the user-facing result.

## Pantry and shopping

The pantry is a household-scoped list of known ingredients plus binary marks. A successful mark replaces the matching entry. If starter installation introduced a previously unseen ingredient, the notifier re-lists to preserve Rust's name ordering.

Shopping is derived from the selected planning window and depends on planner state. Rust supplies quantities, contribution provenance, separate-line reasons, checked/hidden/restored state, manual items, and orphaned-state counts. The provider supports check/hide/restore, manual-item CRUD, reset, and adding checked items to pantry while preserving the derived/stored distinction. Copy helpers conservatively format rational, range, count, mass, volume, and unknown quantities.

## Coverage evidence

This page owns all 32 files under `lib/features`, covering household, onboarding, pantry, planning, recipes, restrictions, settings, and shopping. Exact fingerprints and declaration inventories are in `../coverage-manifest.json`.

