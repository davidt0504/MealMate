import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/api/household.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';

void main() {
  setUpAll(() async => RustLib.init());

  Future<String> tempDb() async {
    final dir = await Directory.systemTemp.createTemp('kimatta-test');
    addTearDown(() => dir.delete(recursive: true));
    return '${dir.path}${Platform.pathSeparator}k.db';
  }

  // Declared first on purpose: the connection is process-wide, so this must run
  // before any openDatabase call in this file.
  test('bootstrap_household before open_database is NotOpen', () async {
    await expectLater(bootstrapHousehold, throwsA(isA<KimattaError_NotOpen>()));
  });

  test('core_version crosses the bridge', () {
    expect(coreVersion(), '0.1.0');
  });

  test('typed Rust error is matchable in Dart', () async {
    await expectLater(
      () => openDatabase(dbPath: '   '),
      throwsA(isA<KimattaError_InvalidPath>()),
    );
  });

  test('open_database migrates a real database to schema v2', () async {
    final report = await openDatabase(dbPath: await tempDb());
    expect(report.schemaVersion, 2);
  });

  test('storage failure surfaces as KimattaError_Storage', () async {
    final missing =
        '${Directory.systemTemp.path}${Platform.pathSeparator}'
        'kimatta-absent-${DateTime.now().microsecondsSinceEpoch}'
        '${Platform.pathSeparator}k.db';
    await expectLater(
      () => openDatabase(dbPath: missing),
      throwsA(isA<KimattaError_Storage>()),
    );
  });

  test('bootstrap creates one household once and reuses it', () async {
    await openDatabase(dbPath: await tempDb());
    final first = await bootstrapHousehold();
    final second = await bootstrapHousehold();
    expect(first.id, isNotEmpty);
    expect(first.name, isNull);
    expect(first.members.single.displayName, 'Me');
    expect(second.id, first.id);
    expect(second.members.single.id, first.members.single.id);
  });

  test('reopening the same file keeps the household (restart)', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final first = await bootstrapHousehold();
    await openDatabase(dbPath: path);
    final again = await bootstrapHousehold();
    expect(again.id, first.id);
    expect(again.members.length, 1);
  });

  test('rename persists and blank clears', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final named = await renameHousehold(householdId: h.id, name: ' Casa ');
    expect(named.name, 'Casa');
    expect((await bootstrapHousehold()).name, 'Casa');
    final cleared = await renameHousehold(householdId: h.id, name: '  ');
    expect(cleared.name, isNull);
  });

  test('rename of a foreign id is rejected and changes nothing', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await expectLater(
      () => renameHousehold(householdId: 'not-${h.id}', name: 'x'),
      throwsA(isA<KimattaError_Storage>()),
    );
    expect((await bootstrapHousehold()).name, isNull);
  });

  test(
    'ensure_planning_cycle returns the dinner-only default across the bridge',
    () async {
      await openDatabase(dbPath: await tempDb());
      final h = await bootstrapHousehold();
      final c = await ensurePlanningCycle(
        householdId: h.id,
        defaultAnchorDate: '2026-08-29',
      );
      expect(c.lengthDays, 7);
      expect(c.mealSlots, [MealSlotDto.dinner]);
      expect(c.dates.length, 7);
      expect(c.dates.first, '2026-08-29');
      // The month-boundary case, end to end.
      expect(c.dates.last, '2026-09-04');
    },
  );

  test('a saved cycle survives reopening the same file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    await savePlanningCycle(
      householdId: h.id,
      anchorDate: '2026-01-05',
      lengthDays: 14,
      mealSlots: [MealSlotDto.breakfast, MealSlotDto.dinner],
    );
    await openDatabase(dbPath: path);
    final again = await ensurePlanningCycle(
      householdId: h.id,
      defaultAnchorDate: '2099-12-01',
    );
    expect(again.anchorDate, '2026-01-05');
    expect(again.lengthDays, 14);
    expect(again.mealSlots, [MealSlotDto.breakfast, MealSlotDto.dinner]);
  });

  test('an invalid anchor date is a typed Planning error in Dart', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await expectLater(
      () => ensurePlanningCycle(
        householdId: h.id,
        defaultAnchorDate: '2026-08-29T23:00:00Z',
      ),
      throwsA(isA<KimattaError_Planning>()),
    );
  });

  test('a different file is a different household', () async {
    await openDatabase(dbPath: await tempDb());
    final a = await bootstrapHousehold();
    await openDatabase(dbPath: await tempDb());
    final b = await bootstrapHousehold();
    expect(b.id, isNot(a.id));
  });
}
