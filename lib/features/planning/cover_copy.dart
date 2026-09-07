import 'package:meal_mate/src/rust/api/planning.dart';

// Cover My Week as the user reads it. Copy only — no widget — so
// `test/cover_copy_test.dart` can sample every string and prove none of them claims safety
// (invariant 19, PRD §10). No control-systems vocabulary anywhere (PRD §2.3). Control labels
// (Accept, Swap, Cover My Week) stay in the widget, per the planner's convention.

const coverTitle = 'Cover My Week';
const firstRunCopy =
    'Kimatta gets dinner planning out of your head; change anything that doesn’t fit.';

/// The three quiet banner states; the fourth renders [needsYouCopy] instead.
const coveredCopy = 'This week is covered.';
const tentativeCopy =
    'This week is planned, with a few things we could not check.';
const unresolvedCopy = 'This week is not fully planned yet.';

/// The exception-console headline: material questions only, counted by high urgency.
String needsYouCopy(int n) =>
    n == 1 ? '1 thing needs you' : '$n things need you';

const questionsHeading = 'Needs you';

/// The quiet section for low-urgency requests — outside the "needs you" count by design
/// (PRD §11: low-value questions stay silent until the user is already here).
const quietHeading = 'When you have a moment';

const assumptionsHeading = "What we couldn't check";

/// A slot the planner could not fill; the options below it are ways to resolve it.
const infeasibleQuestionCopy =
    'Nothing we know of fits this slot. Pick something for it.';

/// A restriction written in the household's own words matched by wording only.
const wordingQuestionCopy =
    'A restriction in your own words was matched by wording alone — check this '
    'meal yourself.';

const fallbackQuestionCopy = 'This needs your decision.';

/// Swap writes the slot locked: the user explicitly decided it, so the next apply must not
/// move it (invariant 18). Stated wherever a swap is offered.
const swapLocksCopy =
    'A swapped meal is locked; automation will not change it.';

const swapSheetTitle = 'Swap this meal';

/// The swap sheet's read-failure line. Prose, not a control label, so it belongs on the
/// sampled surface rather than in the widget.
const swapLibraryUnavailableCopy = 'Your recipe library could not be read.';

/// What a question card says in place of its Swap affordance when the slot it addresses is
/// locked. Deliberately not [lockLabel], which states what a lock binds — automation — and so
/// answers a question the household did not ask: here it is *their own* swap that is
/// unavailable, and the honest answer names their own earlier word and the surface that can
/// take it back (the plan screen's lock switch).
const questionLockedCopy =
    'You locked this meal, so it cannot be swapped here. Unlock it on your '
    'plan to change it.';

String vetoConfirmTitle(String subject) => 'Never suggest $subject?';

/// Names the veto's effect in plain words: it is a standing rule, not a one-off skip. Both
/// following sentences are load-bearing. The engine matches the subject as a whole-token
/// phrase against every candidate's title *and* every ingredient line name (`veto_hit`,
/// `food-domain/src/planner/tier0.rs`), so vetoing a dish named `Chili` also drops anything
/// listing `chili powder` — wider than the tile that was tapped, and the dialog is where that
/// is disclosed, because the write is irreversible. "Those exact words" is deliberate: the
/// match is a contiguous whole-token window with no stemming, so `Rice bowl` does not veto
/// `Rice bowls`, and copy implying otherwise would overstate the rule the other way. And
/// nothing in the app can remove a `food.hard_veto` row, so the dialog must not point anywhere
/// for a reversal that does not exist.
String vetoConfirmBody(String subject) =>
    '$subject will never be suggested again. Any meal whose name or '
    'ingredient list contains those exact words is dropped too. There is no '
    'way to undo this in the app.';

/// The Accept button's accessible meaning: what pressing it writes.
const acceptSemanticsCopy = 'Write this plan to your week';

const acceptedCopy = 'Plan written.';
const shoppingReadyCopy = 'Shopping list is ready';

/// The reviewed-restrictions row, shown only while the assessment says no restrictions are
/// configured: reviewed absence is confirmed absence (PRD §10), so answering "we don't have
/// any" is a real answer, not a dismissal.
const reviewedRowCopy = 'No dietary restrictions are set. Is that right?';
const reviewedRowSetLabel = 'Set restrictions';
const reviewedRowNoneLabel = "We don't have any";

/// User-language for the cycle assumptions disclosure, keyed by the code list Rust ships
/// (`result.assumptions`) — never by `unresolved_issues`, whose sentences carry no codes.
/// Deliberately unmapped, so they render nothing here:
/// - `PANTRY_INCOMPLETE`: always present and non-blocking; pantry stays quiet by having no
///   entry (invariant 6).
/// - `HARD_CONSTRAINT_UNRESOLVED`: already surfaced as a high-urgency question.
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

/// Anchored parse of one candidate option token. Only a single, full-string
/// `recipe:<id>:<numer>/<denom>` token maps to a swap target — recipe ids are
/// UUID-by-convention only (`RecipeId::new` accepts colons and commas), so anything
/// containing a comma (a multi-component join, or a hostile id) and every note-bearing kind
/// returns null, which sends the tap to the tile's Swap picker instead.
String? optionRecipeId(String text) {
  if (text.contains(',')) return null;
  final match = RegExp(r'^recipe:(.+):(\d+)/(\d+)$').firstMatch(text);
  return match?.group(1);
}

String swapToLabel(String title) => 'Swap to $title';

/// The `(date, slot)` a question addresses, parsed from the request id the engine builds as
/// `food:<kind>:<date>:<slot>`. The cycle-level preferences request has no slot and returns
/// null, as does anything else that does not parse — the card then renders without slot
/// affordances rather than guessing.
({String date, MealSlotDto slot})? questionSlot(String id) {
  final parts = id.split(':');
  if (parts.length != 4) return null;
  final slot = switch (parts[3]) {
    'breakfast' => MealSlotDto.breakfast,
    'lunch' => MealSlotDto.lunch,
    'dinner' => MealSlotDto.dinner,
    _ => null,
  };
  if (slot == null) return null;
  return (date: parts[2], slot: slot);
}
