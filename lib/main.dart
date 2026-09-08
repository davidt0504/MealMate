import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/app/app.dart';
import 'package:meal_mate/app/font_licenses.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  registerFontLicenses();
  await RustLib.init();
  // The resolved Riverpod major is 2.x, which has no provider-retry behaviour
  // to disable — `ProviderScope` gains a `retry` argument only in 3.0. A failed
  // DB open is shown on Settings, not re-run with backoff.
  runApp(const ProviderScope(child: App()));
}
