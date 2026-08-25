import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';

void main() {
  setUpAll(() async => RustLib.init());

  test('core_version crosses the bridge', () {
    expect(coreVersion(), '0.1.0');
  });

  test('typed Rust error is matchable in Dart', () async {
    await expectLater(
      () => healthCheck(dbPath: '   '),
      throwsA(isA<KimattaError_InvalidPath>()),
    );
    final report = await healthCheck(dbPath: '/tmp/k.db');
    expect(report.schemaVersion, 0);
  });
}
