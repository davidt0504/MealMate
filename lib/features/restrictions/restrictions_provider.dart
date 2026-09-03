import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/src/rust/api/restrictions.dart';

class RestrictionsNotifier extends AsyncNotifier<List<RestrictionDto>> {
  @override
  Future<List<RestrictionDto>> build() async {
    // Only the id, not the whole household, for the reason `PlanningCycleNotifier` records:
    // watching the future would re-run this build on every rename — an extra bridge read and
    // a Settings subtitle that flickers to "Loading…".
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    return loadRestrictions(householdId: householdId);
  }

  /// Writes the whole set and publishes what the bridge says was stored — trimmed and
  /// de-duplicated — so the UI shows persisted truth rather than typed input. The state write
  /// goes through the notifier, which outlives any screen.
  Future<List<RestrictionDto>> save(
    String householdId,
    List<RestrictionDto> restrictions,
  ) async {
    final stored = await saveRestrictions(
      householdId: householdId,
      restrictions: restrictions,
    );
    state = AsyncData(stored);
    return stored;
  }
}

/// The household's restrictions (household-scoped — the owner's 2026-08-28 resolution).
/// Depends on `householdProvider`, so holding this holds the open database.
final restrictionsProvider =
    AsyncNotifierProvider<RestrictionsNotifier, List<RestrictionDto>>(
      RestrictionsNotifier.new,
    );

/// The known vocabulary, read from Rust so the editor's checkbox list has one source and
/// cannot drift from the domain. A provider rather than a direct call so widget tests can
/// override it without the native library.
final knownRestrictionKindsProvider = FutureProvider<List<String>>(
  (_) async => knownRestrictionKinds(),
);
