import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/src/rust/api/planning.dart';

PlanningCycleDto cycle(List<MealSlotDto> slots, int lengthDays) =>
    PlanningCycleDto(
      householdId: 'h-1',
      anchorDate: '2026-08-29',
      lengthDays: lengthDays,
      mealSlots: slots,
      dates: const [],
    );

void main() {
  test('todayCivilDate formats the local date as ISO', () {
    expect(todayCivilDate(DateTime(2026, 1, 5, 9)), '2026-01-05');
  });

  // The single test pinning the card's stop condition on the Dart side: at 23:30 local in a
  // UTC-negative zone the UTC instant is already the next day, so a `toUtc()` implementation
  // returns '2026-03-09' and moves "Tuesday dinner" a day.
  test(
    'todayCivilDate uses the local date late at night, not the UTC date',
    () {
      expect(todayCivilDate(DateTime(2026, 3, 8, 23, 30)), '2026-03-08');
    },
  );

  test('todayCivilDate defaults to now', () {
    final now = DateTime.now();
    final adjacent = {
      todayCivilDate(now),
      todayCivilDate(now.add(const Duration(minutes: 1))),
    };
    // Membership rather than equality, so a midnight crossing cannot flake this.
    expect(adjacent, contains(todayCivilDate()));
  });

  test('describePlanningCycle renders the dinner-only default', () {
    expect(
      describePlanningCycle(cycle([MealSlotDto.dinner], 7)),
      'Dinner only · 7 days from 2026-08-29',
    );
  });

  test('describePlanningCycle renders a multi-slot scope', () {
    expect(
      describePlanningCycle(
        cycle([MealSlotDto.breakfast, MealSlotDto.dinner], 14),
      ),
      'Breakfast, dinner · 14 days from 2026-08-29',
    );
  });

  test('describePlanningCycle renders all three slots', () {
    expect(
      describePlanningCycle(
        cycle([
          MealSlotDto.breakfast,
          MealSlotDto.lunch,
          MealSlotDto.dinner,
        ], 7),
      ),
      'Breakfast, lunch, dinner · 7 days from 2026-08-29',
    );
  });

  // The MIN_CYCLE_DAYS case, reachable today.
  test('describePlanningCycle renders a one-day cycle in the singular', () {
    expect(
      describePlanningCycle(cycle([MealSlotDto.breakfast], 1)),
      'Breakfast only · 1 day from 2026-08-29',
    );
  });
}
