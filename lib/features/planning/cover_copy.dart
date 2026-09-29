// Cover My Week as the user reads it. Copy only — no widget — so
// `test/cover_copy_test.dart` can sample every string and prove none of them claims safety
// (invariant 19, PRD §10). No control-systems vocabulary anywhere (PRD §2.3). Control labels
// (Accept, Another, Lock in, Choose, Cover My Week) stay in the widget, per the planner's
// convention. Dates render as the ISO strings Rust ships (MVP-013).

const coverTitle = 'Cover My Week';
const firstRunCopy =
    'Kimatta gets dinner planning out of your head; change anything that doesn’t fit.';

/// The banner, from the exact proposal's status.
const coveredCopy = 'This week is covered.';
const tentativeCopy =
    'This week is planned, with a few things we could not check.';
const unresolvedCopy = 'This week is not fully planned yet.';
const needsAttentionCopy = 'Some meals need attention before you accept.';

const assumptionsHeading = "What we couldn't check";

/// Draft edits are not the plan yet: Accept alone writes them (OPT-007 §4).
const pendingCopy = "Changes aren't applied yet.";

/// Shown when commitments exist: the week action never moves them.
const lockedStayCopy = 'Locked-in meals stay.';

/// Why the week action is unavailable, most specific first.
const allLockedCopy = 'All meals are locked in.';
const noSuggestionsToChangeCopy =
    'No suggested meals can change: the rest are your own picks, open, or '
    'past.';

/// Exhaustion keeps the meal and says so; the choices below it are Choose and Reconsider.
String exhaustedCopy(String date) => 'No more alternatives for $date.';

/// Another on a slot with no dish to compare: a fallback, an open night or a free note.
const blockedAnotherCopy =
    'There is no recipe here to find another for. Choose a meal instead.';

/// Per-slot problems the exact proposal carries, keyed by reason code; null renders nothing.
String? slotProblemCopy(List<String> codes) {
  if (codes.contains('DRAFT_RECIPE_UNAVAILABLE')) {
    return 'This recipe is no longer in your library. Choose another.';
  }
  if (codes.contains('PLAN_INFEASIBLE')) {
    return 'Nothing we know of fits this meal. Choose something for it.';
  }
  if (codes.any(
    (c) =>
        c.startsWith('HARD_VETO') ||
        c.startsWith('RESTRICTION_CONFLICT') ||
        c.startsWith('PREP_WINDOW_IMPOSSIBLE'),
  )) {
    return 'This meal conflicts with a rule, restriction or time limit.';
  }
  if (codes.contains('HARD_CONSTRAINT_UNRESOLVED')) {
    return 'A restriction in your own words was matched by wording alone — check '
        'this meal yourself.';
  }
  return null;
}

/// Choose writes nothing yet, and the pick is locked in within the draft.
const chooseLocksCopy = 'A meal you choose is locked in for this plan.';
const chooseSheetTitle = 'Choose a meal';
const replaceSheetTitle = 'Replace a locked-in meal';
const replaceWarningCopy =
    'This replaces a meal you locked in. Your choice stays locked in.';
const chooseLibraryUnavailableCopy = 'Your recipe library could not be read.';

const neverSuggestTitle = 'Never suggest this meal?';

/// Names the rule's reach both ways: it starts now and outlives this draft, and it touches
/// nothing already decided. It is reversible, and says where.
String neverSuggestBody(List<String> dishes) =>
    '${dishes.join(', ')} won’t be suggested from now on, even if you discard '
    'this plan. Saved and locked-in meals aren’t changed. You can remove this '
    'in Settings › Meal exclusions.';

/// A multi-dish meal: the rule applies to the dishes left ticked.
const neverSuggestPickCopy = 'Choose which dishes to stop suggesting.';

/// Review (§5): meals or facts changed elsewhere since this plan started.
const reviewIntroCopy =
    'Meals changed elsewhere since you started. Choose what to keep for each '
    'day.';
const reviewFactsOnlyCopy =
    'Recipes or rules changed since you started. Review the plan before you '
    'accept it.';
const savedMealHeading = 'Saved meal';
const yourChoiceHeading = 'Your choice';
const nothingSavedCopy = 'Nothing saved';

String pastDroppedCopy(int n) => n == 1
    ? 'A change to a day that has passed was dropped.'
    : '$n changes to days that have passed were dropped.';

String expiredDraftCopy(String anchor) =>
    'An unfinished plan for the week of $anchor was set aside.';

/// The Accept button's accessible meaning: what pressing it writes.
const acceptSemanticsCopy = 'Write this plan to your week';

const acceptedCopy = 'Plan written.';
const shoppingReadyCopy = 'Shopping list is ready';
const finishSetupCopy = 'Your plan is saved, but setup did not finish.';

/// The reviewed-restrictions row, shown only while the assessment says no restrictions are
/// configured: reviewed absence is confirmed absence (PRD §10), so answering "we don't have
/// any" is a real answer, not a dismissal.
const reviewedRowCopy = 'No dietary restrictions are set. Is that right?';
const reviewedRowSetLabel = 'Set restrictions';
const reviewedRowNoneLabel = "We don't have any";

/// The meal-exclusions settings screen (OPT-007 §8).
const exclusionsTitle = 'Meal exclusions';
const exclusionsEmptyCopy = 'No meals are excluded.';
const exclusionsDishHeading = 'Meals never suggested';
const exclusionsPhraseHeading = 'Older word rules';
const exclusionsPhraseScopeCopy =
    'These older rules skip any meal whose name or ingredients contain the '
    'words.';
const exclusionsUnavailableCopy = 'No longer in your recipes';
String describeExclusionCount(int n) =>
    n == 1 ? '1 meal is never suggested.' : '$n meals are never suggested.';
const exclusionsRemovedCopy =
    'Removed. It can be suggested again; nothing already planned changed.';

/// User-language for the cycle assumptions disclosure, keyed by the code list Rust ships.
/// Deliberately unmapped, so they render nothing here:
/// - `PANTRY_INCOMPLETE`: always present and non-blocking; pantry stays quiet by having no
///   entry (invariant 6).
/// - `HARD_CONSTRAINT_UNRESOLVED`: already surfaced on the slot it concerns.
/// Unknown codes return null and are omitted — raw tokens never render.
String? assumptionCopy(String code) => switch (code) {
  'LOCK_HELD_OVER' =>
    'A meal you locked was kept even though a check would have flagged it.',
  'NO_PREP_TIME_ESTIMATES' =>
    'Schedule fit could not be checked for every meal.',
  'RESTRICTIONS_NOT_CONFIGURED' =>
    'No dietary restrictions are set, so no restriction check was applied.',
  'PREFERENCES_SPARSE' =>
    'Few preferences are recorded, so this plan is a conservative default.',
  'CONSERVATIVE_DEFAULT' =>
    'This plan leans on defaults rather than tuned choices.',
  'UNKNOWN_POLICY_TYPE' => "A household setting couldn't be applied.",
  _ => null,
};
