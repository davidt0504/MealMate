import 'dart:io';

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
  });

  test('health_check migrates a real database to schema v1', () async {
    final dir = await Directory.systemTemp.createTemp('kimatta-test');
    addTearDown(() => dir.delete(recursive: true));
    final report = await healthCheck(
      dbPath: '${dir.path}${Platform.pathSeparator}k.db',
    );
    expect(report.schemaVersion, 1);
  });

  test('storage failure surfaces as KimattaError_Storage', () async {
    final missing =
        '${Directory.systemTemp.path}${Platform.pathSeparator}'
        'kimatta-absent-${DateTime.now().microsecondsSinceEpoch}'
        '${Platform.pathSeparator}k.db';
    await expectLater(
      () => healthCheck(dbPath: missing),
      throwsA(isA<KimattaError_Storage>()),
    );
  });
}
