import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/src/rust/api/planning.dart';

class PlanningCycleNotifier extends AsyncNotifier<PlanningCycleDto> {
  @override
  Future<PlanningCycleDto> build() async {
    // Only the id, not the whole household: `HouseholdNotifier.rename` republishes the DTO,
    // and watching the future would re-run this build — an extra bridge write and a Settings
    // tile that flickers to "Loading…" on every rename.
    final householdId = await ref.watch(
      householdProvider.selectAsync((h) => h.id),
    );
    return ensurePlanningCycle(
      householdId: householdId,
      defaultAnchorDate: todayCivilDate(),
    );
  }
}

/// The household's planning rhythm, materialised dinner-only on first read (AC-3).
/// Depends on `householdProvider`, so holding this holds the open database and the
/// bootstrapped household.
final planningCycleProvider =
    AsyncNotifierProvider<PlanningCycleNotifier, PlanningCycleDto>(
      PlanningCycleNotifier.new,
    );
