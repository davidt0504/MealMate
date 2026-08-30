import 'package:meal_mate/features/planning/planner_copy.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/shopping.dart';

// The shopping list as the user reads it. Copy only — no widget — so
// `test/shopping_copy_test.dart` can sample every string and prove none claims safety
// (invariant 10). Dates are the ISO strings Rust shipped; nothing here does arithmetic.

const shoppingTitle = 'Shopping';

/// First and last date of the window verbatim, as `windowHeading` does for the planner.
String shoppingHeading(String from, String to) => '$from to $to';

/// Said only of a derivation that resolved to nothing: no recipe line in this window and
/// nothing typed by hand.
const shoppingEmptyCopy =
    'Nothing to buy this cycle yet. Plan a recipe, or add your own item.';

/// The accessible name of a row's checkbox, stating the meaning *before* the platform's own
/// checked state, which no label can suppress (the `pantryRowLabel` precedent).
String lineLabel(String name, bool checked) =>
    checked ? '$name, checked off' : '$name, not checked';

String _rational(int numer, int denom) =>
    denom == 1 ? '$numer' : '$numer/$denom';

String _amount(QuantityDto q) => switch (q) {
  QuantityDto_Unknown() => '',
  QuantityDto_Exact(:final numer, :final denom) => _rational(numer, denom),
  QuantityDto_Range(
    :final minNumer,
    :final minDenom,
    :final maxNumer,
    :final maxDenom,
  ) =>
    '${_rational(minNumer, minDenom)}-${_rational(maxNumer, maxDenom)}',
};

/// `1/2 cup`, `2-3 tbsp`, `2 piece`, `3 handful`; an unknown amount is said to be unknown
/// rather than rendered as nothing, since "flour" alone reads as "any amount".
String describeQuantity(QuantityDto quantity, UnitDto unit) {
  if (quantity is QuantityDto_Unknown) return 'amount not known';
  final amount = _amount(quantity);
  return switch (unit) {
    UnitDto_None() => amount,
    UnitDto_Known(:final unit) => '$amount $unit',
    UnitDto_Other(:final text) => '$amount $text',
  };
}

/// Renders a stored `checked_against` token (`exact:3/2|known:cup`, `range:1/1-2/1|none`,
/// `unknown|other=clove`) the way [describeQuantity] renders the live line, so "was X" and the
/// current amount read alike. A token this cannot parse is shown verbatim rather than
/// guessed at.
String describeCheckedAgainst(String token) {
  final bar = token.indexOf('|');
  if (bar < 0) return token;
  final amount = token.substring(0, bar);
  final unit = token.substring(bar + 1);
  final String? amountText;
  if (amount == 'unknown') {
    return 'amount not known';
  } else if (amount.startsWith('exact:')) {
    amountText = _fraction(amount.substring(6));
  } else if (amount.startsWith('range:')) {
    final parts = amount.substring(6).split('-');
    if (parts.length == 2) {
      final min = _fraction(parts[0]);
      final max = _fraction(parts[1]);
      amountText = min == null || max == null ? null : '$min-$max';
    } else {
      amountText = null;
    }
  } else {
    amountText = null;
  }
  if (amountText == null) return token;
  if (unit == 'none') return amountText;
  if (unit.startsWith('known:')) return '$amountText ${unit.substring(6)}';
  if (unit.startsWith('other=')) {
    return '$amountText ${_unescape(unit.substring(6))}';
  }
  return token;
}

/// `null` for anything that is not `numer/denom`, so an unreadable body reaches
/// [describeCheckedAgainst]'s verbatim fallback instead of being relabelled with a unit.
String? _fraction(String raw) {
  final parts = raw.split('/');
  if (parts.length != 2) return null;
  final numer = int.tryParse(parts[0]);
  final denom = int.tryParse(parts[1]);
  if (numer == null || denom == null) return null;
  return denom == 1 ? '$numer' : raw;
}

/// Undoes the key-segment escape (`\\` → `\`, `\:` → `:`).
String _unescape(String raw) {
  final out = StringBuffer();
  var escaping = false;
  for (final ch in raw.split('')) {
    if (escaping) {
      out.write(ch);
      escaping = false;
    } else if (ch == r'\') {
      escaping = true;
    } else {
      out.write(ch);
    }
  }
  return out.toString();
}

/// Shown under a line whose amount moved since it was checked; the check is *not* kept.
String changedCopy(String was) =>
    'Amount changed since you checked it — was $was';

const optionalCopy = 'optional';

String separateReasonCopy(SeparateReasonDto reason) => switch (reason) {
  SeparateReasonDto.unresolved =>
    'Not matched to a known ingredient, so it is listed on its own.',
  SeparateReasonDto.unitNotCombinable =>
    'Listed separately: its unit does not convert exactly to the others.',
  SeparateReasonDto.unknownQuantity =>
    'Listed separately: this recipe gives no amount.',
  SeparateReasonDto.arithmeticOverflow =>
    'Listed separately: the amounts were too large to add exactly.',
};

/// Says what a pantry mark means and no more: it suppressed a purchase, it proved nothing.
const omittedCopy =
    'Skipped because you marked it as in your pantry. A mark does not say how '
    'much you have.';

const neededHeading = 'To buy';
const alreadyHaveHeading = 'Already have';
const removedHeading = 'Removed';
const yourItemsHeading = 'Your items';
const uncategorisedHeading = 'Other';

/// `title · date Slot · original text`, plus the scale when one was applied — the same
/// vocabulary the planner uses for the same occurrence.
String explainContribution(ContributionDto c) {
  final scale = c.scale == null ? '' : ' (${describeScale(c.scale)})';
  return '${c.recipeTitle} · ${c.date} ${slotLabel(c.slot)} · '
      '${c.originalText}$scale';
}

String addedToPantryCopy(int n) => n == 1
    ? '1 item marked as in your pantry'
    : '$n items marked as in your pantry';

const resetTitle = 'Start over?';

/// Names both counts, since both are cleared: every edit stored against this cycle's list —
/// checks, removals and restores alike, including the rows the current derivation no longer
/// shows — and the manual items already checked off. Unchecked manual items stay. Said as
/// "edits" rather than naming the shapes, because the count spans all of them.
String resetBody(int lineStates, int checkedItems) {
  String n(int count, String one, String many) =>
      count == 1 ? '1 $one' : '$count $many';
  return 'This clears ${n(lineStates, 'edit you made', 'edits you made')} to '
      "this cycle's list and "
      '${n(checkedItems, 'checked-off item', 'checked-off items')} of your own. '
      'Unchecked items of your own stay.';
}

/// Stated up front rather than discovered: generated lines follow the plan, so a change to
/// the plan changes them, and only manual items are edited in place.
const manualEditPolicyCopy =
    'Recipe lines follow your plan and update with it; your checks and removals '
    'are kept. Items you add yourself are edited here.';

const manualNameRequiredCopy = 'A name is required.';

/// Said when stored edits were made against lines this cycle's derivation no longer contains
/// — a plan change or a new cycle length moves the window. The rows are untouched in storage,
/// and deriving the original window shows them again, so nothing here may read as a deletion.
String orphanedStatesCopy(int n) => n == 1
    ? '1 earlier check or removal was made on a line this cycle no longer has, '
          'so it is not shown here.'
    : '$n earlier checks or removals were made on lines this cycle no longer '
          'has, so they are not shown here.';

String countsCopy(int needed, int alreadyHave, int removed) =>
    '$needed to buy · $alreadyHave already have · $removed removed';

enum Section { needed, alreadyHave, removed }

/// One rule for where a derived line renders (MVP-016 decision 6): hidden wins over
/// everything; a pantry-omitted line sits under Already have until restored; the rest is
/// needed. `null` state is "no record", which is needed unless the derivation omitted it.
Section sectionFor(ShoppingLineStatusDto status, ShoppingLineStateDto? state) {
  if (state?.hidden ?? false) return Section.removed;
  if (status == ShoppingLineStatusDto.omittedPantryMarked &&
      !(state?.restored ?? false)) {
    return Section.alreadyHave;
  }
  return Section.needed;
}
