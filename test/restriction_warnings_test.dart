import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/features/recipes/restriction_warnings.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/restrictions.dart';

/// One sample per constant and per function the warnings module renders. It lives here rather
/// than in `lib/` because only this test reads it — a fixture in production code ships in the
/// release binary.
final restrictionCopySamples = <String>[
  noRestrictionsCopy,
  noKnownConflictCopy,
  hideConflictsLabel,
  warningsHeading,
  nothingHiddenCopy,
  hiddenCountCopy(1),
  hiddenCountCopy(3),
  wordingOnlyCopy(const ['nightshades', 'red meat']),
  describeConflict(
    const ConflictDto(
      restriction: RestrictionDto.known(kind: 'dairy'),
      linePosition: 0,
      lineName: 'butter',
      term: 'butter',
    ),
  ),
  summariseConflicts(
    const RestrictionAssessmentDto(
      ruleVersion: 1,
      restrictionsChecked: 2,
      linesChecked: 1,
      conflicts: [
        ConflictDto(
          restriction: RestrictionDto.known(kind: 'dairy'),
          linePosition: 0,
          lineName: 'butter',
          term: 'butter',
        ),
        ConflictDto(
          restriction: RestrictionDto.other(text: 'nightshades'),
          linePosition: 0,
          lineName: 'butter',
          term: 'nightshades',
        ),
      ],
      wordingOnly: ['nightshades'],
    ),
  ),
];

void main() {
  // Expected-to-pass pin (AC-4): the fresh-context copy review is the real evidence; this
  // keeps a later edit from reintroducing an assurance word by accident.
  test('no restriction copy claims safety', () {
    final assurance = RegExp(
      r'\b(safe|safely|free-from|free of|allergen-free|suitable for|friendly|'
      r'contains no|ok for|okay for|cleared)\b',
      caseSensitive: false,
    );
    expect(restrictionCopySamples, isNotEmpty);
    for (final sample in restrictionCopySamples) {
      expect(assurance.hasMatch(sample), isFalse, reason: sample);
    }
  });

  test('describeConflict names the restriction, the line and the term', () {
    expect(
      describeConflict(
        const ConflictDto(
          restriction: RestrictionDto.known(kind: 'tree_nuts'),
          linePosition: 2,
          lineName: 'almond milk',
          term: 'almond',
        ),
      ),
      'Tree nuts — "almond milk" matched "almond"',
    );
    expect(
      describeConflict(
        const ConflictDto(
          restriction: RestrictionDto.other(text: 'nightshades'),
          linePosition: 0,
          lineName: 'nightshades mix',
          term: 'nightshades',
        ),
      ),
      'nightshades — "nightshades mix" matched "nightshades"',
    );
  });

  test('summariseConflicts lists each restriction once, in order', () {
    const a = RestrictionAssessmentDto(
      ruleVersion: 1,
      restrictionsChecked: 2,
      linesChecked: 3,
      conflicts: [
        ConflictDto(
          restriction: RestrictionDto.known(kind: 'dairy'),
          linePosition: 0,
          lineName: 'butter',
          term: 'butter',
        ),
        ConflictDto(
          restriction: RestrictionDto.known(kind: 'dairy'),
          linePosition: 2,
          lineName: 'cream',
          term: 'cream',
        ),
        ConflictDto(
          restriction: RestrictionDto.known(kind: 'eggs'),
          linePosition: 1,
          lineName: '2 eggs',
          term: 'eggs',
        ),
      ],
      wordingOnly: [],
    );
    expect(summariseConflicts(a), 'May conflict: Dairy, Eggs');
    expect(hiddenCountCopy(2), startsWith('2 hidden for known conflicts.'));
    expect(
      wordingOnlyCopy(const ['nightshades']),
      'Checked by wording only: nightshades.',
    );
  });
}
