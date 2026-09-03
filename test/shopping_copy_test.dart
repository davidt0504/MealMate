import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/features/shopping/shopping_copy.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/shopping.dart';

const sampleContribution = ContributionDto(
  plannedMealId: 'pm-1',
  date: '2026-08-29',
  slot: MealSlotDto.dinner,
  componentPosition: 0,
  recipeId: 'r-1',
  recipeTitle: 'Pancakes',
  linePosition: 0,
  originalText: '2 cups flour',
  scale: ScaleDto(numer: 3, denom: 2),
);

ShoppingLineStateDto state({
  bool checked = false,
  bool hidden = false,
  bool restored = false,
  bool changed = false,
}) => ShoppingLineStateDto(
  key: 'k',
  checked: checked,
  hidden: hidden,
  restored: restored,
  changed: changed,
);

/// Every string the shopping screen renders that states something about food or the list.
final shoppingCopySamples = [
  shoppingTitle,
  shoppingHeading('2026-08-29', '2026-09-04'),
  shoppingEmptyCopy,
  lineLabel('flour', true),
  lineLabel('flour', false),
  describeQuantity(
    const QuantityDto.exact(numer: 1, denom: 2),
    const UnitDto.known(unit: 'cup'),
  ),
  describeCheckedAgainst('exact:1/2|known:cup'),
  changedCopy('1/2 cup'),
  optionalCopy,
  for (final reason in SeparateReasonDto.values) separateReasonCopy(reason),
  omittedCopy,
  neededHeading,
  alreadyHaveHeading,
  removedHeading,
  yourItemsHeading,
  uncategorisedHeading,
  explainContribution(sampleContribution),
  addedToPantryCopy(1),
  addedToPantryCopy(3),
  resetTitle,
  resetBody(1, 1),
  resetBody(2, 0),
  orphanedStatesCopy(1),
  orphanedStatesCopy(6),
  manualEditPolicyCopy,
  manualNameRequiredCopy,
  countsCopy(1, 2, 3),
];

void main() {
  // Expected-to-pass pin (invariant 10), the same regex `planner_copy_test.dart` uses.
  test('no shopping copy claims safety', () {
    final assurance = RegExp(
      r'\b(safe|safely|free-from|free of|allergen-free|suitable for|friendly|'
      r'contains no|ok for|okay for|cleared)\b',
      caseSensitive: false,
    );
    expect(shoppingCopySamples, isNotEmpty);
    for (final sample in shoppingCopySamples) {
      expect(assurance.hasMatch(sample), isFalse, reason: sample);
    }
  });

  test('describeQuantity renders every quantity and unit shape', () {
    const cup = UnitDto.known(unit: 'cup');
    expect(
      describeQuantity(const QuantityDto.exact(numer: 1, denom: 2), cup),
      '1/2 cup',
    );
    expect(
      describeQuantity(const QuantityDto.exact(numer: 2, denom: 1), cup),
      '2 cup',
    );
    expect(
      describeQuantity(
        const QuantityDto.range(
          minNumer: 2,
          minDenom: 1,
          maxNumer: 3,
          maxDenom: 1,
        ),
        const UnitDto.known(unit: 'tbsp'),
      ),
      '2-3 tbsp',
    );
    expect(
      describeQuantity(
        const QuantityDto.exact(numer: 3, denom: 1),
        const UnitDto.none(),
      ),
      '3',
    );
    expect(
      describeQuantity(
        const QuantityDto.exact(numer: 3, denom: 1),
        const UnitDto.other(text: 'handful'),
      ),
      '3 handful',
    );
    expect(
      describeQuantity(const QuantityDto.unknown(), cup),
      'amount not known',
    );
  });

  test('describeCheckedAgainst reads a token like the live line', () {
    expect(describeCheckedAgainst('exact:3/2|known:cup'), '3/2 cup');
    expect(describeCheckedAgainst('exact:2/1|none'), '2');
    expect(describeCheckedAgainst('range:1/1-2/1|known:tbsp'), '1-2 tbsp');
    expect(
      describeCheckedAgainst(r'unknown|other=clo\:ve'),
      'amount not known',
    );
    expect(describeCheckedAgainst(r'exact:1/1|other=clo\:ve'), '1 clo:ve');
    // Unparseable tokens are shown verbatim, never guessed at.
    expect(describeCheckedAgainst('garbage'), 'garbage');
    expect(describeCheckedAgainst('weird:1|known:cup'), 'weird:1|known:cup');
    // The prefix is recognised but the body is not: still verbatim, never relabelled with a
    // unit. `1.5` is the shape a `Rational` `Display` drift would produce (Known Risk 5).
    expect(
      describeCheckedAgainst('exact:1.5|known:cup'),
      'exact:1.5|known:cup',
    );
    expect(describeCheckedAgainst('exact:|none'), 'exact:|none');
    expect(
      describeCheckedAgainst('range:1/1-x|known:tbsp'),
      'range:1/1-x|known:tbsp',
    );
    expect(
      describeCheckedAgainst('range:1/1-2/1-3/1|none'),
      'range:1/1-2/1-3/1|none',
    );
  });

  test('lineLabel states the meaning before the platform state', () {
    expect(lineLabel('flour', true), 'flour, checked off');
    expect(lineLabel('flour', false), 'flour, not checked');
  });

  test('explainContribution names the occurrence and the scale', () {
    expect(
      explainContribution(sampleContribution),
      'Pancakes · 2026-08-29 Dinner · 2 cups flour (1½×)',
    );
    expect(
      explainContribution(
        const ContributionDto(
          plannedMealId: 'pm-1',
          date: '2026-08-30',
          slot: MealSlotDto.lunch,
          componentPosition: 1,
          recipeId: 'r-1',
          recipeTitle: 'Soup',
          linePosition: 2,
          originalText: 'salt',
        ),
      ),
      'Soup · 2026-08-30 Lunch · salt',
    );
  });

  test('resetBody names both counts', () {
    expect(
      resetBody(1, 1),
      "This clears 1 edit you made to this cycle's list and 1 checked-off item "
      'of your own. Unchecked items of your own stay.',
    );
    expect(
      resetBody(3, 0),
      "This clears 3 edits you made to this cycle's list and 0 checked-off "
      'items of your own. Unchecked items of your own stay.',
    );
  });

  test('orphanedStatesCopy says the rows are not shown, never that they were '
      'deleted', () {
    expect(
      orphanedStatesCopy(1),
      '1 earlier check or removal was made on a line this cycle no longer has, '
      'so it is not shown here.',
    );
    expect(
      orphanedStatesCopy(6),
      '6 earlier checks or removals were made on lines this cycle no longer '
      'has, so they are not shown here.',
    );
    // The rows survive in storage; no wording may say otherwise.
    final destroyed = RegExp(
      r'\b(deleted|removed|lost|discarded|erased|gone)\b',
      caseSensitive: false,
    );
    for (final n in [1, 6]) {
      expect(destroyed.hasMatch(orphanedStatesCopy(n)), isFalse);
    }
  });

  test('sectionFor: hidden wins, omitted needs restore, else needed', () {
    const needed = ShoppingLineStatusDto.needed;
    const omitted = ShoppingLineStatusDto.omittedPantryMarked;
    // Hard-coded table: status × hidden × restored.
    final table = <(ShoppingLineStatusDto, ShoppingLineStateDto?), Section>{
      (needed, null): Section.needed,
      (needed, state()): Section.needed,
      (needed, state(checked: true)): Section.needed,
      (needed, state(hidden: true)): Section.removed,
      (needed, state(restored: true)): Section.needed,
      (needed, state(hidden: true, restored: true)): Section.removed,
      (omitted, null): Section.alreadyHave,
      (omitted, state()): Section.alreadyHave,
      (omitted, state(restored: true)): Section.needed,
      (omitted, state(hidden: true)): Section.removed,
      (omitted, state(hidden: true, restored: true)): Section.removed,
    };
    expect(table, isNotEmpty);
    for (final entry in table.entries) {
      expect(
        sectionFor(entry.key.$1, entry.key.$2),
        entry.value,
        reason: '${entry.key}',
      );
    }
  });
}
