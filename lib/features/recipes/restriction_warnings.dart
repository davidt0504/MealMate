import 'package:meal_mate/features/restrictions/restriction_copy.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

/// Every user-visible string this feature renders lives in this module, so the AC-4 copy test
/// — which samples all of them, from `test/restriction_warnings_test.dart` — can prove none of
/// them claims safety. A restriction warning names what was matched and never what was cleared
/// (invariant 10).
const noRestrictionsCopy = 'No restrictions set — nothing was checked.';
const noKnownConflictCopy =
    'No known conflict found. This is not a safety check: only the '
    'ingredient names were matched against known terms.';
const hideConflictsLabel = 'Hide known conflicts';
const warningsHeading = 'Restriction warnings';

/// The engaged filter's line when it hid nothing. It exists because the alternative — an
/// engaged filter with no line at all — is the state that reads most like "these were checked
/// and cleared", which AC-4 and invariant 10 forbid.
const nothingHiddenCopy =
    'Nothing hidden — no recipe matched a known term. This is not a safety check.';

String hiddenCountCopy(int n) =>
    '$n hidden for known conflicts. The rest matched no known term — '
    'this is not a safety check.';

String wordingOnlyCopy(List<String> texts) =>
    'Checked by wording only: ${texts.join(', ')}.';

/// "restriction ← line ← term", so every warning is explainable from the screen alone.
String describeConflict(ConflictDto c) =>
    '${describeRestriction(c.restriction)} — "${c.lineName}" matched "${c.term}"';

/// One line for a list tile: the distinct restrictions hit, in assessment order.
String summariseConflicts(RestrictionAssessmentDto a) {
  final labels = <String>[];
  for (final c in a.conflicts) {
    final label = describeRestriction(c.restriction);
    if (!labels.contains(label)) labels.add(label);
  }
  return 'May conflict: ${labels.join(', ')}';
}
