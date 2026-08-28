import 'package:flutter/material.dart';

/// Single seed until a design card introduces real tokens.
const seedColor = Color(0xFF3F6B3A);

final lightTheme = ThemeData(
  colorSchemeSeed: seedColor,
  brightness: Brightness.light,
);
final darkTheme = ThemeData(
  colorSchemeSeed: seedColor,
  brightness: Brightness.dark,
);
