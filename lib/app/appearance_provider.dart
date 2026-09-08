import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/app/theme.dart';

class AppearanceSelection {
  const AppearanceSelection({required this.palette, required this.type});

  const AppearanceSelection.defaults()
    : palette = PaletteChoice.aizomeLinen,
      type = TypeChoice.shipporiAtkinson;

  final PaletteChoice palette;
  final TypeChoice type;

  AppearanceSelection copyWith({PaletteChoice? palette, TypeChoice? type}) =>
      AppearanceSelection(
        palette: palette ?? this.palette,
        type: type ?? this.type,
      );
}

class AppearanceNotifier extends Notifier<AppearanceSelection> {
  @override
  AppearanceSelection build() => const AppearanceSelection.defaults();

  void selectPalette(PaletteChoice palette) {
    state = state.copyWith(palette: palette);
  }

  void selectType(TypeChoice type) {
    state = state.copyWith(type: type);
  }
}

final appearanceProvider =
    NotifierProvider<AppearanceNotifier, AppearanceSelection>(
      AppearanceNotifier.new,
    );
