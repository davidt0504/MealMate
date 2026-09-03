import 'package:meal_mate/src/rust/api/recipe.dart';

/// Dropdown keys for the two non-`UnitKind` choices; neither can collide with a token from
/// `known_unit_kinds`, which are all unit names.
const unitNoneKey = 'none';
const unitOtherKey = 'other';

/// The `_ => kind` arm renders the raw token, which is honest; the cross-check test in
/// `bridge_native_test.dart` is what makes a kind added in Rust without a label here a test
/// failure rather than `fl_oz` reaching a user.
String unitLabel(String kind) => switch (kind) {
  'tsp' => 'tsp',
  'tbsp' => 'tbsp',
  'cup' => 'cup',
  'fl_oz' => 'fl oz',
  'ml' => 'ml',
  'l' => 'l',
  'g' => 'g',
  'kg' => 'kg',
  'oz' => 'oz',
  'lb' => 'lb',
  'piece' => 'piece',
  _ => kind,
};

/// What a line looks like on the detail screen: the text as the user wrote it, untouched
/// (invariant 2). The structured fields are for matching and shopping, never for display.
String describeLine(IngredientLineDto line) => line.originalText;

final _mixed = RegExp(r'^(\d+) (\d+)/(\d+)$');
final _fraction = RegExp(r'^(\d+)/(\d+)$');
final _decimal = RegExp(r'^(\d+)\.(\d+)$');
final _whole = RegExp(r'^\d+$');
final _rangeDash = RegExp(r'\s*[-–]\s*');

/// Both parts of an amount cross the bridge as `u32` (`rust/src/api/recipe.rs:15-22`) and the
/// generated encoder masks rather than range-checks (`putUint32`), so a larger number would be
/// stored as a different one. Bounded here, where the form can say so inline. This is also
/// exactly the range `_LineDraft._amountText` renders back into the field, so a stored amount
/// always re-parses on edit.
const maxQuantity = 0xFFFFFFFF;

/// `10^9` is the largest power of ten inside [maxQuantity], so a decimal carries at most nine
/// places. Checked before the accumulator runs: `denom *= 10` wraps past 64 bits at nineteen
/// places and lands on a negative denominator, which the zero guard below does not catch.
const _maxDecimalPlaces = 9;

/// Parses an amount as a cook would type it. Blank is `Unknown` (the honest absent amount);
/// `2`, `1/2`, `1 1/2`, `1.5` are exact; `2-3` (hyphen or en dash) is a range. Anything
/// else — including the `2+` and `0` forms MVP-007 left unrepresented — is `null`, so the form
/// can say so inline rather than silently storing `Unknown`. Decimals are sent unreduced
/// (`1.5` → 15/10); Rust normalises. Zero and inverted ranges are rejected here for the
/// message, and by Rust again for the invariant.
QuantityDto? parseQuantity(String raw) {
  final text = raw.trim();
  if (text.isEmpty) return const QuantityDto.unknown();
  final parts = text.split(_rangeDash);
  QuantityDto? result;
  if (parts.length == 1) {
    final r = _rational(parts[0]);
    if (r != null) result = QuantityDto.exact(numer: r.$1, denom: r.$2);
  } else if (parts.length == 2) {
    final min = _rational(parts[0]);
    final max = _rational(parts[1]);
    // Cross-multiplied as `BigInt`: both products reach [maxQuantity] squared, which is
    // outside Dart's 64-bit `int` and wraps negative, inverting the comparison so an
    // inverted range would be accepted. `4294967295-1/4294967295` is such a range.
    if (min != null &&
        max != null &&
        BigInt.from(min.$1) * BigInt.from(max.$2) <=
            BigInt.from(max.$1) * BigInt.from(min.$2)) {
      result = QuantityDto.range(
        minNumer: min.$1,
        minDenom: min.$2,
        maxNumer: max.$1,
        maxDenom: max.$2,
      );
    }
  }
  return result;
}

/// `(numer, denom)` of one positive amount inside [maxQuantity], or `null`.
(int, int)? _rational(String text) {
  final s = text.trim();
  (int, int)? pair;
  if (_whole.hasMatch(s)) {
    final n = _bounded(s);
    if (n != null) pair = (n, 1);
  } else if (_fraction.firstMatch(s) case final m?) {
    final n = _bounded(m[1]!);
    final d = _bounded(m[2]!);
    if (n != null && d != null) pair = (n, d);
  } else if (_mixed.firstMatch(s) case final m?) {
    final whole = _bounded(m[1]!);
    final n = _bounded(m[2]!);
    final d = _bounded(m[3]!);
    // Divided rather than `whole * d <= maxQuantity - n`: the product is what would overflow.
    if (whole != null &&
        n != null &&
        d != null &&
        d != 0 &&
        whole <= (maxQuantity - n) ~/ d) {
      pair = (whole * d + n, d);
    }
  } else if (_decimal.firstMatch(s) case final m?) {
    final places = m[2]!.length;
    if (places <= _maxDecimalPlaces) {
      var denom = 1;
      for (var i = 0; i < places; i++) {
        denom *= 10;
      }
      final n = _bounded(m[1]! + m[2]!);
      if (n != null) pair = (n, denom);
    }
  }
  if (pair != null && (pair.$1 == 0 || pair.$2 == 0)) pair = null;
  return pair;
}

/// The digits as an `int` inside [maxQuantity], or `null`. `tryParse` rather than `parse`:
/// above 64 bits `int.parse` throws, and that throw escapes `_validate` — which `_save` calls
/// — as an unhandled asynchronous error instead of the inline message the parser promises.
int? _bounded(String digits) {
  final value = int.tryParse(digits);
  return value != null && value <= maxQuantity ? value : null;
}
