import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/settings/health_provider.dart';
import 'package:meal_mate/src/rust/api/household.dart';

class HouseholdNotifier extends AsyncNotifier<HouseholdDto> {
  @override
  Future<HouseholdDto> build() async {
    await ref.watch(healthReportProvider.future);
    return bootstrapHousehold();
  }

  /// Renames and publishes the DTO the bridge already returns. The state write goes
  /// through the notifier, which outlives any screen, so a save that races the
  /// screen's disposal still reaches every listener — and no re-bootstrap is needed
  /// to read back what the write just told us.
  Future<HouseholdDto> rename(String householdId, String? name) async {
    final updated = await renameHousehold(householdId: householdId, name: name);
    state = AsyncData(updated);
    return updated;
  }
}

/// The local household, created anonymously on first launch (invariants 1, 8).
/// Depends on the open database, so holding this provider holds both.
final householdProvider =
    AsyncNotifierProvider<HouseholdNotifier, HouseholdDto>(
      HouseholdNotifier.new,
    );
