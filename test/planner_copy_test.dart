import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/features/planning/planner_copy.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planning.dart';

const sampleWindow = PlanningCycleDto(
  householdId: 'h-1',
  anchorDate: '2026-08-29',
  lengthDays: 7,
  mealSlots: [MealSlotDto.dinner],
  dates: [
    '2026-08-29',
    '2026-08-30',
    '2026-08-31',
    '2026-09-01',
    '2026-09-02',
    '2026-09-03',
    '2026-09-04',
  ],
);

/// Every string the planner renders that states something about a meal, a recipe or the
/// library — which is where a safety claim can hide. The planner's control labels (`Add`,
/// `Cancel`, `Remove`, `Move…`, `Scale…`, `Try again`, `Cover My Week`, `Locked`, `Note`, and
/// the two cycle tooltips) stay in the widget file as `pantry_screen.dart`'s do: they name an
/// action, not a fact about food. The prose the planner renders from shared modules is pinned
/// where it lives — `summariseConflicts` by `restriction_warnings_test.dart`, and
/// `describeFailure` by `household_screen.dart`, which every feature renders.
final plannerCopySamples = [
  plannerTitle,
  for (final slot in MealSlotDto.values) slotLabel(slot),
  windowHeading(sampleWindow),
  emptyCellCopy,
  plannerEmptyCopy,
  lockLabel(true),
  lockLabel(false),
  describeScale(null),
  describeScale(const ScaleDto(numer: 1, denom: 2)),
  describeScale(const ScaleDto(numer: 3, denom: 2)),
  describeScale(const ScaleDto(numer: 2, denom: 1)),
  describeScale(const ScaleDto(numer: 5, denom: 3)),
  for (final kind in [
    'recipe',
    'leftovers',
    'dining_out',
    'frozen_quick',
    'freeform',
    'open',
    'mystery',
  ])
    kindLabel(kind),
  archivedRecipeCopy('Pancakes'),
  unknownRecipeCopy,
  pendingRecipeCopy,
  unreadRecipeCopy,
  noteRequiredCopy,
  removeLastComponentTitle,
  removeLastComponentBody,
  noFreeSlotCopy,
  emptyLibraryCopy,
  libraryUnreadCopy,
  kindsUnreadCopy,
  recipesHeading,
  otherHeading,
  moveToHeading,
  scaleHeading,
];

void main() {
  // Expected-to-pass pin (invariant 10), the same regex `restriction_warnings_test.dart`
  // uses, copied verbatim: no planner string may read as a safety claim.
  test('no planner copy claims safety', () {
    final assurance = RegExp(
      r'\b(safe|safely|free-from|free of|allergen-free|suitable for|friendly|'
      r'contains no|ok for|okay for|cleared)\b',
      caseSensitive: false,
    );
    expect(plannerCopySamples, isNotEmpty);
    for (final sample in plannerCopySamples) {
      expect(assurance.hasMatch(sample), isFalse, reason: sample);
    }
  });

  test('slotLabel names each slot in title case', () {
    expect(slotLabel(MealSlotDto.breakfast), 'Breakfast');
    expect(slotLabel(MealSlotDto.lunch), 'Lunch');
    expect(slotLabel(MealSlotDto.dinner), 'Dinner');
  });

  test('windowHeading reads the first and last date verbatim', () {
    expect(windowHeading(sampleWindow), '2026-08-29 to 2026-09-04');
    // A one-day window has one date, and the heading still names both ends.
    expect(
      windowHeading(
        const PlanningCycleDto(
          householdId: 'h-1',
          anchorDate: '2026-12-31',
          lengthDays: 1,
          mealSlots: [MealSlotDto.lunch],
          dates: ['2026-12-31'],
        ),
      ),
      '2026-12-31 to 2026-12-31',
    );
  });

  test('lockLabel states what a lock binds', () {
    expect(lockLabel(true), 'Locked — automation will not change this meal');
    expect(lockLabel(false), 'Unlocked');
  });

  test('describeScale renders the five presets and a general fraction', () {
    expect(describeScale(null), '1×');
    expect(describeScale(const ScaleDto(numer: 1, denom: 2)), '½×');
    expect(describeScale(const ScaleDto(numer: 3, denom: 2)), '1½×');
    expect(describeScale(const ScaleDto(numer: 1, denom: 1)), '1×');
    expect(describeScale(const ScaleDto(numer: 2, denom: 1)), '2×');
    expect(describeScale(const ScaleDto(numer: 3, denom: 1)), '3×');
    expect(describeScale(const ScaleDto(numer: 5, denom: 3)), '5/3×');
  });

  test('kindLabel labels every known kind and falls back to the token', () {
    expect(kindLabel('recipe'), 'Recipe');
    expect(kindLabel('leftovers'), 'Leftovers');
    expect(kindLabel('dining_out'), 'Dining out');
    expect(kindLabel('frozen_quick'), 'Frozen / quick');
    expect(kindLabel('freeform'), 'Something else');
    expect(kindLabel('open'), 'Leave open');
    expect(kindLabel('mystery'), 'mystery');
  });

  test('archivedRecipeCopy marks the title without hiding it', () {
    expect(archivedRecipeCopy('Pancakes'), 'Pancakes (archived)');
  });
}
