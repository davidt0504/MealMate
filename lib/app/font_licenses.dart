import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

bool _fontLicensesRegistered = false;

/// Registers the bundled font licenses with Flutter's standard license page.
void registerFontLicenses() {
  if (_fontLicensesRegistered) return;
  _fontLicensesRegistered = true;

  const licenses = <String, String>{
    'Atkinson Hyperlegible': 'assets/fonts/atkinsonhyperlegible/OFL.txt',
    'Shippori Mincho': 'assets/fonts/shipporimincho/OFL.txt',
    'Zen Maru Gothic': 'assets/fonts/zenmarugothic/OFL.txt',
    'Zen Old Mincho': 'assets/fonts/zenoldmincho/OFL.txt',
  };

  LicenseRegistry.addLicense(() async* {
    for (final entry in licenses.entries) {
      final text = await rootBundle.loadString(entry.value);
      yield LicenseEntryWithLineBreaks(<String>[entry.key], text);
    }
  });
}
