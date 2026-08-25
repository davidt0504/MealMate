import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/main.dart';
import 'package:meal_mate/src/rust/api/health.dart';

void main() {
  testWidgets('HealthScreen renders fake-fed calls and a typed error', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        home: HealthScreen(
          version: '9.9.9',
          report: Future.value(
            const HealthReport(dbPath: '/x.db', schemaVersion: 7),
          ),
          probeError: () => Future.error(const KimattaError.invalidPath()),
        ),
      ),
    );
    await tester.pump();
    expect(find.text('core_version: 9.9.9'), findsOneWidget);
    expect(find.textContaining('schema v7'), findsOneWidget);
    await tester.tap(find.text('Probe typed error'));
    await tester.pump();
    expect(find.text('probe: KimattaError.InvalidPath'), findsOneWidget);
  });
}
