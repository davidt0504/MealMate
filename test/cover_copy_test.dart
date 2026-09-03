import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/features/planning/cover_copy.dart';
import 'package:meal_mate/src/rust/api/planning.dart';

/// Every exported string, explicitly — the safety regex must see the whole surface.
final coverCopySamples = <String>[
  coverTitle,
  coveredCopy,
  tentativeCopy,
  unresolvedCopy,
  needsYouCopy(1),
  needsYouCopy(2),
  questionsHeading,
  quietHeading,
  assumptionsHeading,
  infeasibleQuestionCopy,
  wordingQuestionCopy,
  fallbackQuestionCopy,
  swapLocksCopy,
  swapSheetTitle,
  swapLibraryUnavailableCopy,
  questionLockedCopy,
  vetoConfirmTitle('Peanut noodles'),
  vetoConfirmBody('Peanut noodles'),
  acceptSemanticsCopy,
  acceptedCopy,
  shoppingReadyCopy,
  reviewedRowCopy,
  reviewedRowSetLabel,
  reviewedRowNoneLabel,
  swapToLabel('Rice bowl'),
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
    expect(coverCopySamples.length, 31);
    for (final sample in coverCopySamples) {
      expect(assurance.hasMatch(sample), isFalse, reason: sample);
    }
  });

  test('assumption map keeps pantry and wording-only quiet', () {
    // Hard-coded set: mirroring the production map would hide additions to it.
    expect(assumptionCopy('PANTRY_INCOMPLETE'), isNull);
    expect(assumptionCopy('HARD_CONSTRAINT_UNRESOLVED'), isNull);
    expect(assumptionCopy('SOME_FUTURE_CODE'), isNull);
    expect(assumptionCopy('RESTRICTIONS_NOT_CONFIGURED'), isNotNull);
  });

  test('optionRecipeId maps only a single full recipe token', () {
    expect(optionRecipeId('recipe:r-1:1/1'), 'r-1');
    expect(optionRecipeId('recipe:r-tofu:3/2'), 'r-tofu');
    // An id containing a colon parses greedily and deterministically.
    expect(optionRecipeId('recipe:odd:id:1/1'), 'odd:id');
    // Note-bearing kinds, joins, and hostile ids all fall to the picker.
    expect(optionRecipeId('leftovers:from tuesday'), isNull);
    expect(optionRecipeId('frozen_quick:'), isNull);
    expect(optionRecipeId('recipe:r-1:1/1,recipe:r-2:1/1'), isNull);
    expect(optionRecipeId('recipe:a,recipe:b:1/1'), isNull);
    expect(optionRecipeId('recipe:r-1'), isNull);
    expect(optionRecipeId(''), isNull);
  });

  test('questionSlot parses the engine id shape and nothing else', () {
    expect(questionSlot('food:infeasible:2026-08-29:dinner'), (
      date: '2026-08-29',
      slot: MealSlotDto.dinner,
    ));
    expect(questionSlot('food:preferences:2026-08-29'), isNull);
    expect(questionSlot('food:x:2026-08-29:supper'), isNull);
    expect(questionSlot(''), isNull);
  });

  // The veto is permanent: no command reads, edits or deletes a `food.hard_veto` row, so the
  // one dialog whose job is informed consent must not point at a reversal surface.
  test('the veto dialog states permanence and promises no reversal', () {
    expect(
      vetoConfirmBody('Rice bowl'),
      contains('Rice bowl will never be suggested again.'),
    );
    expect(vetoConfirmBody('Rice bowl'), isNot(contains('household settings')));
  });

  // The engine matches a veto subject as a whole-token phrase against each candidate's title
  // *and* every ingredient line name (`veto_hit`, `food-domain/src/planner/tier0.rs`), so the
  // rule reaches further than the dish that was tapped. The write is irreversible and the UI
  // is its only minting surface, so the breadth is stated before consent is taken.
  test('the veto dialog states the breadth the consent actually buys', () {
    final body = vetoConfirmBody('Chili');
    expect(body, contains('ingredient list'));
    expect(body, contains('those exact words'));
  });

  // "those exact words" is a back-reference: it only reads as the subject if it follows the
  // subject and precedes the permanence claim. A reorder would leave it dangling.
  test(
    'the breadth sentence sits between the subject and the no-undo claim',
    () {
      final body = vetoConfirmBody('Peanut noodles');
      expect(body, startsWith('Peanut noodles will never be suggested again.'));
      // Both bounds asserted: `indexOf` returns -1 for an absent phrase, which would satisfy
      // `lessThan` on its own and make this pass against copy that never states the breadth.
      expect(body.indexOf('those exact words'), greaterThan(0));
      expect(
        body.indexOf('those exact words'),
        lessThan(body.indexOf('no way to undo')),
      );
    },
  );

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

  // Expected-to-pass: the subject is interpolated, never escaped or truncated, so a title
  // carrying punctuation reads back verbatim in both strings.
  test('veto copy embeds any subject verbatim', () {
    const subject = "Rice & 'bowl' (50%)";
    expect(vetoConfirmTitle(subject), 'Never suggest $subject?');
    expect(vetoConfirmBody(subject), startsWith('$subject will never'));
  });

  test('needsYouCopy counts in user language', () {
    expect(needsYouCopy(1), '1 thing needs you');
    expect(needsYouCopy(3), '3 things need you');
  });
}
