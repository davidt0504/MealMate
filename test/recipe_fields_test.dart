import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/features/recipes/recipe_fields.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

void main() {
  group('parseQuantity', () {
    test('blank is the honest Unknown, never an error', () {
      expect(parseQuantity(''), const QuantityDto.unknown());
      expect(parseQuantity('  '), const QuantityDto.unknown());
    });

    test('whole numbers, fractions, mixed numbers and decimals are exact', () {
      expect(parseQuantity('2'), const QuantityDto.exact(numer: 2, denom: 1));
      expect(parseQuantity('1/2'), const QuantityDto.exact(numer: 1, denom: 2));
      expect(
        parseQuantity('1 1/2'),
        const QuantityDto.exact(numer: 3, denom: 2),
      );
      // Sent as 15/10: Rust normalises to 3/2 (pinned in bridge_native_test).
      expect(
        parseQuantity('1.5'),
        const QuantityDto.exact(numer: 15, denom: 10),
      );
      expect(parseQuantity(' 3 '), const QuantityDto.exact(numer: 3, denom: 1));
    });

    test('a dash between two amounts is a range', () {
      const expected = QuantityDto.range(
        minNumer: 2,
        minDenom: 1,
        maxNumer: 3,
        maxDenom: 1,
      );
      expect(parseQuantity('2-3'), expected);
      expect(parseQuantity('2 - 3'), expected);
      expect(parseQuantity('2–3'), expected);
      expect(
        parseQuantity('1/2-3/4'),
        const QuantityDto.range(
          minNumer: 1,
          minDenom: 2,
          maxNumer: 3,
          maxDenom: 4,
        ),
      );
    });

    // Rejected with a message, never silently `Unknown`: `2+` and `0` are the cases MVP-007
    // deferred, and `3-2`/`1/0` are what Rust would reject anyway — better here, inline.
    test('unreadable amounts are null so the form can say so', () {
      for (final raw in [
        '0',
        '1/0',
        '3-2',
        'abc',
        '2+',
        '1-2-3',
        '0-2',
        '-1',
      ]) {
        expect(parseQuantity(raw), isNull, reason: raw);
      }
    });

    // The bridge carries every part of an amount as `u32` and the generated encoder masks
    // rather than range-checks, so anything above it would be stored as a different number.
    test('the u32 range the bridge carries is the accepted range', () {
      expect(
        parseQuantity('4294967295'),
        const QuantityDto.exact(numer: 4294967295, denom: 1),
      );
      expect(parseQuantity('4294967296'), isNull);
      expect(
        parseQuantity('1/4294967295'),
        const QuantityDto.exact(numer: 1, denom: 4294967295),
      );
      expect(parseQuantity('1/4294967296'), isNull);
    });

    // A per-group bound alone would miss this: both groups are in range, the product is not.
    test('a mixed number is bounded by its product, not by its groups', () {
      expect(
        parseQuantity('999999999 1/2'),
        const QuantityDto.exact(numer: 1999999999, denom: 2),
      );
      expect(parseQuantity('999999999 1/9'), isNull);
    });

    // Each of these used to throw `FormatException` out of `_validate` — which `_save` calls
    // outside its `try` — or wrap silently past 64 bits. All are `null` now, so the form says
    // so inline. `4294967295-1/4294967295` is the range whose cross-multiply overflows:
    // 4294967295 x 4294967295 wraps to -8589934591, which would compare as `<=` and be
    // accepted as an inverted range.
    test('amounts outside 64 bits are null, never a throw or a wrap', () {
      for (final raw in [
        '99999999999999999999',
        '1.0000000000000000000',
        '0.0000000000000000000',
        '2-99999999999999999999',
        '4294967295-1/4294967295',
      ]) {
        expect(parseQuantity(raw), isNull, reason: raw);
      }
    });
  });

  group('unitLabel', () {
    test('every known token has a label and the fallback is the token', () {
      const labels = {
        'tsp': 'tsp',
        'tbsp': 'tbsp',
        'cup': 'cup',
        'fl_oz': 'fl oz',
        'ml': 'ml',
        'l': 'l',
        'g': 'g',
        'kg': 'kg',
        'oz': 'oz',
        'lb': 'lb',
        'piece': 'piece',
      };
      expect(labels, isNotEmpty);
      labels.forEach((kind, label) => expect(unitLabel(kind), label));
      expect(unitLabel('furlong'), 'furlong');
    });
  });

  test('describeLine renders the original text verbatim', () {
    const line = IngredientLineDto(
      originalText: '  1/2 cup Flour, sifted ',
      name: 'Flour',
      quantity: QuantityDto.exact(numer: 1, denom: 2),
      unit: UnitDto.known(unit: 'cup'),
      optional: false,
    );
    expect(describeLine(line), '  1/2 cup Flour, sifted ');
  });
}
