import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/features/settings/backup_copy.dart';

void main() {
  test('the stamp is compact, colon-free and zero-padded', () {
    final stamp = backupStamp(DateTime(2026, 9, 2, 1, 2, 3));
    expect(stamp, '20260902-010203');
    expect(stamp.contains(':'), isFalse);
  });

  test('export filenames sort chronologically', () {
    final names = [
      exportFileName(DateTime(2026, 9, 2, 10, 0, 0)),
      exportFileName(DateTime(2026, 9, 2, 9, 59, 59)),
      exportFileName(DateTime(2025, 12, 31, 23, 59, 59)),
    ]..sort();
    expect(names.last, 'kimatta-export-20260902-100000.db');
    expect(names.first, 'kimatta-export-20251231-235959.db');
  });

  test('destructive copy states the loss and the kept file', () {
    expect(restoreConfirmBody('x.db'), contains('replaced by "x.db"'));
    expect(restoreConfirmBody('x.db'), contains('kept on this device'));
    expect(startFreshConfirmBody, contains('lost'));
    expect(startFreshConfirmBody, contains('kept on this device'));
    // The dialog actions name the destructive half, not the feature.
    expect(restoreConfirmAction, 'Replace my data');
    expect(startFreshConfirmAction, 'Set data aside and start empty');
  });

  test('the import confirm names the file, what it replaces and the undo', () {
    final body = importConfirmBody('old.db');
    expect(body, contains('replaced by "old.db"'));
    expect(body, contains('recipes'));
    expect(body, contains('"Restore latest export"'));
    expect(importConfirmAction, 'Save a copy and replace my data');
  });

  test('import outcomes name the safety export and the refused file', () {
    expect(
      importedCopy('kimatta-export-20260916-101500.db'),
      contains('"kimatta-export-20260916-101500.db"'),
    );
    expect(
      importStoppedCopy('Export unavailable: disk full'),
      contains('nothing was replaced'),
    );
    expect(
      importStoppedCopy('Export unavailable: disk full'),
      endsWith('disk full'),
    );
    // The chosen file is what is damaged, never the live database.
    expect(damagedFileCopy, contains('file is damaged'));
    expect(damagedFileCopy, isNot(contains('local database')));
  });
}
