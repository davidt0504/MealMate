import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/app/share_channel.dart';

void main() {
  test('the first http(s) link in shared text is the one imported', () {
    expect(
      firstUrl('Try this https://example.com/shawarma tonight'),
      'https://example.com/shawarma',
    );
    expect(
      firstUrl('http://a.example/1 and https://b.example/2'),
      'http://a.example/1',
    );
    expect(firstUrl('HTTPS://Example.com/Pie'), 'HTTPS://Example.com/Pie');
  });

  test('sentence punctuation after a link is not part of it', () {
    expect(
      firstUrl('Look: https://example.com/pie.'),
      'https://example.com/pie',
    );
    expect(
      firstUrl('(see https://example.com/pie?id=3)!'),
      'https://example.com/pie?id=3',
    );
    expect(firstUrl('"https://example.com/pie"'), 'https://example.com/pie');
  });

  test('text without a web link has nothing to import', () {
    expect(firstUrl(''), isNull);
    expect(firstUrl('chicken, rice, lemons'), isNull);
    expect(firstUrl('javascript:alert(1)'), isNull);
    expect(firstUrl('file:///etc/passwd'), isNull);
    expect(firstUrl('ftp://example.com/pie'), isNull);
  });
}
