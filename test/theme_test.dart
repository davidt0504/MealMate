import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/app/theme.dart';
import 'package:meal_mate/app/font_licenses.dart';

typedef _ColorPair = ({
  String name,
  Color foreground,
  Color background,
  double minimum,
});

double _contrast(Color a, Color b) {
  final lighter = a.computeLuminance() > b.computeLuminance() ? a : b;
  final darker = identical(lighter, a) ? b : a;
  return (lighter.computeLuminance() + 0.05) /
      (darker.computeLuminance() + 0.05);
}

List<_ColorPair> _pairs(ColorScheme scheme) => [
  for (final surface in <(String, Color)>[
    ('surface', scheme.surface),
    ('surfaceDim', scheme.surfaceDim),
    ('surfaceBright', scheme.surfaceBright),
    ('surfaceContainerLowest', scheme.surfaceContainerLowest),
    ('surfaceContainerLow', scheme.surfaceContainerLow),
    ('surfaceContainer', scheme.surfaceContainer),
    ('surfaceContainerHigh', scheme.surfaceContainerHigh),
    ('surfaceContainerHighest', scheme.surfaceContainerHighest),
  ]) ...[
    (
      name: 'onSurface/${surface.$1}',
      foreground: scheme.onSurface,
      background: surface.$2,
      minimum: 4.5,
    ),
    (
      name: 'onSurfaceVariant/${surface.$1}',
      foreground: scheme.onSurfaceVariant,
      background: surface.$2,
      minimum: 4.5,
    ),
  ],
  (
    name: 'primary/surface',
    foreground: scheme.primary,
    background: scheme.surface,
    minimum: 4.5,
  ),
  (
    name: 'secondary/surface',
    foreground: scheme.secondary,
    background: scheme.surface,
    minimum: 4.5,
  ),
  (
    name: 'tertiary/surface',
    foreground: scheme.tertiary,
    background: scheme.surface,
    minimum: 4.5,
  ),
  (
    name: 'error/surface',
    foreground: scheme.error,
    background: scheme.surface,
    minimum: 4.5,
  ),
  for (final pair in <(String, Color, Color)>[
    ('onPrimary/primary', scheme.onPrimary, scheme.primary),
    (
      'onPrimaryContainer/primaryContainer',
      scheme.onPrimaryContainer,
      scheme.primaryContainer,
    ),
    ('onPrimaryFixed/primaryFixed', scheme.onPrimaryFixed, scheme.primaryFixed),
    (
      'onPrimaryFixedVariant/primaryFixedDim',
      scheme.onPrimaryFixedVariant,
      scheme.primaryFixedDim,
    ),
    ('onSecondary/secondary', scheme.onSecondary, scheme.secondary),
    (
      'onSecondaryContainer/secondaryContainer',
      scheme.onSecondaryContainer,
      scheme.secondaryContainer,
    ),
    (
      'onSecondaryFixed/secondaryFixed',
      scheme.onSecondaryFixed,
      scheme.secondaryFixed,
    ),
    (
      'onSecondaryFixedVariant/secondaryFixedDim',
      scheme.onSecondaryFixedVariant,
      scheme.secondaryFixedDim,
    ),
    ('onTertiary/tertiary', scheme.onTertiary, scheme.tertiary),
    (
      'onTertiaryContainer/tertiaryContainer',
      scheme.onTertiaryContainer,
      scheme.tertiaryContainer,
    ),
    (
      'onTertiaryFixed/tertiaryFixed',
      scheme.onTertiaryFixed,
      scheme.tertiaryFixed,
    ),
    (
      'onTertiaryFixedVariant/tertiaryFixedDim',
      scheme.onTertiaryFixedVariant,
      scheme.tertiaryFixedDim,
    ),
    ('onError/error', scheme.onError, scheme.error),
    (
      'onErrorContainer/errorContainer',
      scheme.onErrorContainer,
      scheme.errorContainer,
    ),
    (
      'onInverseSurface/inverseSurface',
      scheme.onInverseSurface,
      scheme.inverseSurface,
    ),
  ])
    (name: pair.$1, foreground: pair.$2, background: pair.$3, minimum: 4.5),
  (
    name: 'outline/surface',
    foreground: scheme.outline,
    background: scheme.surface,
    minimum: 3.0,
  ),
];

List<Color> _allTokens(ColorScheme scheme) => [
  scheme.primary,
  scheme.onPrimary,
  scheme.primaryContainer,
  scheme.onPrimaryContainer,
  scheme.primaryFixed,
  scheme.primaryFixedDim,
  scheme.onPrimaryFixed,
  scheme.onPrimaryFixedVariant,
  scheme.secondary,
  scheme.onSecondary,
  scheme.secondaryContainer,
  scheme.onSecondaryContainer,
  scheme.secondaryFixed,
  scheme.secondaryFixedDim,
  scheme.onSecondaryFixed,
  scheme.onSecondaryFixedVariant,
  scheme.tertiary,
  scheme.onTertiary,
  scheme.tertiaryContainer,
  scheme.onTertiaryContainer,
  scheme.tertiaryFixed,
  scheme.tertiaryFixedDim,
  scheme.onTertiaryFixed,
  scheme.onTertiaryFixedVariant,
  scheme.error,
  scheme.onError,
  scheme.errorContainer,
  scheme.onErrorContainer,
  scheme.surface,
  scheme.onSurface,
  scheme.surfaceDim,
  scheme.surfaceBright,
  scheme.surfaceContainerLowest,
  scheme.surfaceContainerLow,
  scheme.surfaceContainer,
  scheme.surfaceContainerHigh,
  scheme.surfaceContainerHighest,
  scheme.onSurfaceVariant,
  scheme.outline,
  scheme.outlineVariant,
  scheme.shadow,
  scheme.scrim,
  scheme.inverseSurface,
  scheme.onInverseSurface,
  scheme.inversePrimary,
  scheme.surfaceTint,
];

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  for (final entry in candidateColorSchemes.entries) {
    final palette = entry.key.$1;
    final brightness = entry.key.$2;
    final scheme = entry.value;
    test('${palette.name} ${brightness.name} tokens are opaque and tinted', () {
      for (final color in _allTokens(scheme)) {
        expect(color.a, 1.0, reason: color.toString());
        expect(color, isNot(const Color(0xFF000000)));
        expect(color, isNot(const Color(0xFFFFFFFF)));
      }
    });

    test('${palette.name} ${brightness.name} contrast ratios', () {
      for (final pair in _pairs(scheme)) {
        final ratio = _contrast(pair.foreground, pair.background);
        debugPrint(
          '${palette.name}/${brightness.name} ${pair.name}: '
          '${ratio.toStringAsFixed(2)}:1',
        );
        expect(
          ratio,
          greaterThanOrEqualTo(pair.minimum),
          reason:
              '${pair.name} is ${ratio.toStringAsFixed(2)}:1; '
              'requires ${pair.minimum}:1',
        );
      }
    });
  }

  testWidgets('all four font licenses are registered', (tester) async {
    registerFontLicenses();
    final entries = await LicenseRegistry.licenses.toList();
    final text = entries
        .expand(
          (entry) => <String>[
            ...entry.packages,
            ...entry.paragraphs.map((paragraph) => paragraph.text),
          ],
        )
        .join('\n');
    expect(text, contains('Atkinson Hyperlegible'));
    expect(text, contains('Braille Institute of America'));
    expect(text, contains('The Shippori Mincho Project Authors'));
    expect(text, contains('The Zen Old Mincho Project Authors'));
    expect(text, contains('The Zen Maru Gothic Project Authors'));
  });

  test('type pairings use only bundled 400 and 700 weights', () {
    for (final type in TypeChoice.values) {
      final theme = buildTheme(
        PaletteChoice.aizomeLinen,
        type,
        Brightness.light,
      );
      expect(theme.textTheme.bodyMedium?.fontFamily, 'Atkinson Hyperlegible');
      expect(theme.textTheme.bodyMedium?.fontWeight, FontWeight.w400);
      expect(theme.textTheme.labelLarge?.fontWeight, FontWeight.w700);
      expect(theme.textTheme.titleLarge?.fontWeight, FontWeight.w700);
      expect(theme.textTheme.titleLarge?.fontFamily, switch (type) {
        TypeChoice.shipporiAtkinson => 'Shippori Mincho',
        TypeChoice.zenOldAtkinson => 'Zen Old Mincho',
        TypeChoice.zenMaruAtkinson => 'Zen Maru Gothic',
        TypeChoice.atkinsonOnly => 'Atkinson Hyperlegible',
      });
    }
  });
}
