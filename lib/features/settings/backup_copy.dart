/// Copy and filename rules for the Settings backup and recovery section (MVP-017).
///
/// Pure Dart on purpose, like the other `*_copy.dart` files: the wording is honest about
/// destructive outcomes (invariant 8) and is pinned by unit tests.
library;

const exportButtonLabel = 'Export data';
const restoreButtonLabel = 'Restore latest export';
const noExportsCopy = 'No exports yet — export your data first.';
const tryAgainLabel = 'Try again';
const startFreshLabel = 'Start fresh';

const restoreConfirmTitle = 'Restore this export?';

/// The dialog's action names the destructive half of the trade, not the feature.
const restoreConfirmAction = 'Replace my data';
const startFreshConfirmAction = 'Set data aside and start empty';

/// Names the overwrite and where the replaced data goes — the user is trading current
/// data for the export's, and that must be said, not implied.
String restoreConfirmBody(String exportName) =>
    'Your current data will be replaced by "$exportName". '
    'The replaced database is kept on this device in case you need it back.';

const startFreshConfirmTitle = 'Start fresh with no data?';

/// States the data loss plainly and that the damaged file is kept, not destroyed.
const startFreshConfirmBody =
    'The damaged database will be set aside and the app will start empty. '
    'Anything the damaged file still held is lost to the app unless you '
    'restore an export. The damaged file itself is kept on this device.';

String exportedCopy(String path) => 'Exported to $path';
const restoredCopy = 'Restore complete.';
const startedFreshCopy =
    'Started fresh. The damaged file was kept on this device.';

/// Compact sortable stamp: `yyyyMMdd-HHmmss`. No colons — raw ISO breaks on FAT/exFAT
/// and Windows, where shared exports can land — and lexicographic order stays
/// chronological, so "latest export" is a plain sort.
String backupStamp(DateTime t) {
  String two(int v) => v.toString().padLeft(2, '0');
  return '${t.year.toString().padLeft(4, '0')}${two(t.month)}${two(t.day)}'
      '-${two(t.hour)}${two(t.minute)}${two(t.second)}';
}

String exportFileName(DateTime t) => 'kimatta-export-${backupStamp(t)}.db';
