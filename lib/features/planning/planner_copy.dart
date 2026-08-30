import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planning.dart';

// The planner as the user reads it. Copy only — no widget — so `test/planner_copy_test.dart`
// can sample every string and prove none of them claims safety (invariant 10). No weekday
// names anywhere: dates are rendered as the ISO strings Rust shipped (MVP-013 stop condition).

const plannerTitle = 'Plan';

String slotLabel(MealSlotDto slot) => switch (slot) {
  MealSlotDto.breakfast => 'Breakfast',
  MealSlotDto.lunch => 'Lunch',
  MealSlotDto.dinner => 'Dinner',
};

/// First and last date of the window verbatim; no arithmetic, no locale format.
String windowHeading(PlanningCycleDto window) =>
    '${window.dates.first} to ${window.dates.last}';

const emptyCellCopy = 'Nothing planned.';

/// Names the manual affordance first; the automated one is "later", not a promise of when.
const plannerEmptyCopy =
    'Nothing planned this cycle yet. Add a meal to any day, or use Cover My '
    'Week later.';

/// The accessible name of the lock switch. It states what a lock binds — automation, not the
/// user (invariant 18) — before the platform's own on/off, which no label can suppress.
String lockLabel(bool locked) =>
    locked ? 'Locked — automation will not change this meal' : 'Unlocked';

/// `null` is the bridge's "as written". The two half presets get their glyph; anything else
/// is rendered as the fraction it is rather than rounded.
String describeScale(ScaleDto? scale) => switch (scale) {
  null => '1×',
  ScaleDto(numer: 1, denom: 2) => '½×',
  ScaleDto(numer: 3, denom: 2) => '1½×',
  ScaleDto(:final numer, denom: 1) => '$numer×',
  ScaleDto(:final numer, :final denom) => '$numer/$denom×',
};

/// The `_ => kind` arm renders the raw token, which is honest, as `restrictionLabel` does.
String kindLabel(String kind) => switch (kind) {
  'recipe' => 'Recipe',
  'leftovers' => 'Leftovers',
  'dining_out' => 'Dining out',
  'frozen_quick' => 'Frozen / quick',
  'freeform' => 'Something else',
  'open' => 'Leave open',
  _ => kind,
};

/// An archived recipe stays on the meal it was planned for (MVP-012 decision gate); the
/// marker says why it is not in the picker.
String archivedRecipeCopy(String title) => '$title (archived)';

/// A resolved "in neither library". Only the two providers together can say this, and only
/// once both have produced a list — see [pendingRecipeCopy] and [unreadRecipeCopy].
const unknownRecipeCopy = 'Recipe unavailable';

/// A library read is still in flight, so the recipe's name and its restriction assessment are
/// both unknown. Distinct from [unknownRecipeCopy]: claiming a recipe is not there while the
/// read that would find it has not finished is false, and the conflict line missing alongside
/// it reads as cleared (invariant 10).
const pendingRecipeCopy = 'Loading recipe…';

/// A library read failed. The same silence as [pendingRecipeCopy] — no name, no assessment —
/// but it does not promise the name is on its way, because nothing on this screen retries it.
const unreadRecipeCopy = 'Recipe could not be read';

const noteRequiredCopy = 'A note is required for this kind.';
const removeLastComponentTitle = 'Remove the whole meal?';

/// Names both consequences, because removing the last component removes the occurrence — the
/// domain refuses an empty one — and the lock goes with it.
const removeLastComponentBody =
    'This is the only thing planned here; removing it clears the meal and its '
    'lock.';

const noFreeSlotCopy = 'No free slot in this cycle.';

/// Said only of a library that has resolved to nothing; a read still in flight gets a spinner
/// instead, since "you have no recipes" is a claim only a finished read can make.
const emptyLibraryCopy = 'No recipes in your library yet.';

const libraryUnreadCopy = 'Your recipe library could not be read.';
const kindsUnreadCopy = 'The other kinds could not be read.';

/// The picker's and the two sheets' section headings.
const recipesHeading = 'Recipes';
const otherHeading = 'Other';
const moveToHeading = 'Move to';
const scaleHeading = 'Scale';
