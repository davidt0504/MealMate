import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/features/planning/cover_copy.dart';

/// Every exported string, explicitly — the safety regex must see the whole surface.
final coverCopySamples = <String>[
  coverTitle,
  firstRunCopy,
  coveredCopy,
  tentativeCopy,
  unresolvedCopy,
  needsAttentionCopy,
  assumptionsHeading,
  pendingCopy,
  lockedStayCopy,
  allLockedCopy,
  noSuggestionsToChangeCopy,
  exhaustedCopy('2026-09-29'),
  blockedAnotherCopy,
  for (final codes in const [
    ['DRAFT_RECIPE_UNAVAILABLE'],
    ['PLAN_INFEASIBLE'],
    ['HARD_VETO'],
    ['HARD_CONSTRAINT_UNRESOLVED'],
  ])
    slotProblemCopy(codes)!,
  chooseLocksCopy,
  chooseSheetTitle,
  replaceSheetTitle,
  replaceWarningCopy,
  chooseLibraryUnavailableCopy,
  neverSuggestTitle,
  neverSuggestBody(['Peanut noodles', 'Rice']),
  neverSuggestPickCopy,
  reviewIntroCopy,
  reviewFactsOnlyCopy,
  savedMealHeading,
  yourChoiceHeading,
  nothingSavedCopy,
  pastDroppedCopy(1),
  pastDroppedCopy(2),
  expiredDraftCopy('2026-09-21'),
  acceptSemanticsCopy,
  acceptedCopy,
  shoppingReadyCopy,
  finishSetupCopy,
  reviewedRowCopy,
  reviewedRowSetLabel,
  reviewedRowNoneLabel,
  exclusionsTitle,
  exclusionsEmptyCopy,
  exclusionsDishHeading,
  exclusionsPhraseHeading,
  exclusionsPhraseScopeCopy,
  exclusionsUnavailableCopy,
  describeExclusionCount(1),
  describeExclusionCount(3),
  exclusionsRemovedCopy,
  for (final code in const [
    'LOCK_HELD_OVER',
    'NO_PREP_TIME_ESTIMATES',
    'RESTRICTIONS_NOT_CONFIGURED',
    'PREFERENCES_SPARSE',
    'CONSERVATIVE_DEFAULT',
    'UNKNOWN_POLICY_TYPE',
  ])
    assumptionCopy(code)!,
];

void main() {
  // Expected-to-pass pin (invariant 19), the same regex `planner_copy_test.dart` uses,
  // copied verbatim: no cover string may read as a safety claim.
  test('no cover copy claims safety', () {
    final assurance = RegExp(
      r'\b(safe|safely|free-from|free of|allergen-free|suitable for|friendly|'
      r'contains no|ok for|okay for|cleared)\b',
      caseSensitive: false,
    );
    // The sample list is hand-maintained and every regex in this file iterates only it, so its
    // size is pinned: a sample dropped in a merge or a refactor silently shrinks the surface
    // all three checks see. This does not catch a *new* constant added to `cover_copy.dart` and
    // never sampled — Dart has no reflection over a library's top-level constants, and the only
    // construction that would is deriving the samples from one exported map the widgets also
    // read. That residual is tracked in `KNOWN_ISSUES-low.md`.
    expect(coverCopySamples, isNotEmpty);
    expect(coverCopySamples.length, 55);
    for (final sample in coverCopySamples) {
      expect(assurance.hasMatch(sample), isFalse, reason: sample);
    }
  });

  test('the first-run sentence uses product language', () {
    final machinery = RegExp(
      r'\b(controller|optimizer|algorithm|feedback loop|control system)\b',
      caseSensitive: false,
    );
    expect(machinery.hasMatch(firstRunCopy), isFalse);
    expect(firstRunCopy, isNot(contains('safe')));
  });

  test('assumption map keeps pantry and wording-only quiet', () {
    // Hard-coded set: mirroring the production map would hide additions to it.
    expect(assumptionCopy('PANTRY_INCOMPLETE'), isNull);
    expect(assumptionCopy('HARD_CONSTRAINT_UNRESOLVED'), isNull);
    expect(assumptionCopy('SOME_FUTURE_CODE'), isNull);
    expect(assumptionCopy('RESTRICTIONS_NOT_CONFIGURED'), isNotNull);
  });

  // OPT-007 §8: the rule is on the dish's identity, reversible, and outlives the draft. The
  // dialog says all three before consent — and, unlike the old word rule, claims no reach into
  // ingredient lists it does not have.
  test('the never-suggest dialog states reach, permanence of the rule, and reversal', () {
    final body = neverSuggestBody(['Chili']);
    expect(body, startsWith('Chili won’t be suggested from now on'));
    expect(body, contains('even if you discard this plan'));
    expect(body, contains('Saved and locked-in meals aren’t changed'));
    expect(body, contains('Settings › Meal exclusions'));
    expect(body, isNot(contains('ingredient')));
    expect(body, isNot(contains('no way to undo')));
  });

  test('a multi-dish rule names every dish it covers', () {
    expect(
      neverSuggestBody(['Rice', 'Beans']),
      startsWith('Rice, Beans won’t be suggested'),
    );
  });

  test('slot problems map known codes and never render a raw one', () {
    expect(slotProblemCopy(const ['SOME_FUTURE_CODE']), isNull);
    expect(slotProblemCopy(const []), isNull);
    // Parameterised Tier-0 codes reach their family's copy.
    expect(
      slotProblemCopy(const ['RESTRICTION_CONFLICT:rules_v1']),
      slotProblemCopy(const ['HARD_VETO']),
    );
    // The most specific problem wins.
    expect(
      slotProblemCopy(const ['HARD_VETO', 'DRAFT_RECIPE_UNAVAILABLE']),
      contains('no longer in your library'),
    );
  });

  // Expected-to-pass pin, the third whole-surface regex. `contains_phrase`
  // (`food-domain/src/restriction.rs:355`) is `windows(n).any(|w| w == phrase)` over lowercased
  // whole tokens with no stemming: a `Rice bowl` veto does not match `Rice bowls`. Copy that
  // reads as substring or inflection matching overstates the rule in the other direction, which
  // on an irreversible write is the same defect invariant 19 exists to prevent.
  test('no cover copy reads as substring or inflection matching', () {
    final fuzzy = RegExp(
      r'anything containing|similar to|or similar|related to|anything like|'
      r'mentions?\b',
      caseSensitive: false,
    );
    for (final sample in coverCopySamples) {
      expect(fuzzy.hasMatch(sample), isFalse, reason: sample);
    }
  });

  test('no cover copy promises a change the app cannot make', () {
    // The whole surface, not just `vetoConfirmBody`: a reversal promise re-added anywhere
    // trips this.
    final reversal = RegExp(
      r'change this later|household settings',
      caseSensitive: false,
    );
    for (final sample in coverCopySamples) {
      expect(reversal.hasMatch(sample), isFalse, reason: sample);
    }
  });

  // Expected-to-pass: dish names are interpolated, never escaped or truncated.
  test('never-suggest copy embeds any dish name verbatim', () {
    const dish = "Rice & 'bowl' (50%)";
    expect(neverSuggestBody([dish]), startsWith('$dish won’t'));
  });

  test('counts read in user language', () {
    expect(describeExclusionCount(1), '1 meal is never suggested.');
    expect(describeExclusionCount(3), '3 meals are never suggested.');
    expect(pastDroppedCopy(1), startsWith('A change'));
    expect(pastDroppedCopy(2), startsWith('2 changes'));
  });
}
