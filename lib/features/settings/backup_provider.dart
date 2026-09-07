import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/features/planning/cover_provider.dart';
import 'package:meal_mate/features/planning/planner_provider.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/restrictions/restrictions_provider.dart';
import 'package:meal_mate/features/settings/backup_copy.dart';
import 'package:meal_mate/features/settings/health_provider.dart';
import 'package:meal_mate/features/shopping/shopping_provider.dart';
import 'package:meal_mate/src/rust/api/health.dart';

/// The bridge/filesystem seam for backup, restore and start-fresh, injectable so widget
/// tests never reach the real bridge.
final backupActionsProvider = Provider<BackupActions>(
  (_) => const BackupActions(),
);

class BackupActions {
  const BackupActions();

  Future<Directory> _exportsDir() async {
    final dir = await getApplicationSupportDirectory();
    return Directory('${dir.path}${Platform.pathSeparator}exports');
  }

  Future<ExportReport> export() async {
    final exports = await _exportsDir();
    return exportDatabase(
      destPath:
          '${exports.path}${Platform.pathSeparator}${exportFileName(DateTime.now())}',
    );
  }

  /// Newest export by filename — the stamp format makes lexicographic order
  /// chronological. `null` when none exist.
  Future<String?> latestExport() async {
    final exports = await _exportsDir();
    if (!exports.existsSync()) {
      return null;
    }
    final names =
        exports
            .listSync()
            .whereType<File>()
            .map((f) => f.path)
            .where((p) => p.endsWith('.db'))
            .toList()
          ..sort();
    return names.isEmpty ? null : names.last;
  }

  /// Opens the platform share sheet for a finished export — the sanctioned way to move
  /// data off the device (the manifest excludes the database from auto-backup).
  Future<void> shareExport(String path) async {
    await SharePlus.instance.share(
      ShareParams(files: [XFile(path)], subject: 'Kimatta export'),
    );
  }

  Future<HealthReport> restore(String exportPath) async => restoreDatabase(
    exportPath: exportPath,
    dbPath: await localDatabasePath(),
  );

  Future<HealthReport> startFresh() async => resetDatabase(
    dbPath: await localDatabasePath(),
    stamp: backupStamp(DateTime.now()),
  );
}

/// True while a destructive backup operation is in flight, so both destructive buttons can
/// be disabled meanwhile. The app keeps exactly one `.pre-restore` generation: a second
/// confirmed restore moves the *first* restore's result into that slot and destroys the only
/// copy of the database the user started with (invariant 8). The confirmation dialog is not
/// that guard — it closes as soon as it is answered, long before the bridge call returns, and
/// `restore_database` runs on an FRB worker thread.
class BackupBusy extends Notifier<bool> {
  @override
  bool build() => false;

  /// `false` when an operation is already running and the caller must not start another —
  /// where a tap that raced the rebuild disabling the button lands.
  bool begin() {
    if (state) {
      return false;
    }
    state = true;
    return true;
  }

  void end() => state = false;
}

final backupBusyProvider = NotifierProvider<BackupBusy, bool>(BackupBusy.new);

/// The in-memory identity of the database all cached providers describe. A restore can put back
/// an older snapshot with the same household id, so DTO equality cannot identify this boundary.
/// Invalidate the old snapshot before publishing the generation: rebuilding an app listener
/// between those operations could otherwise consume the old household as this generation's data.
class DatabaseGeneration extends Notifier<int> {
  @override
  int build() => 0;

  /// Every provider that caches database-derived state, enumerated. The household id is often
  /// unchanged across a restore, so `selectAsync((h) => h.id)` does not provide this cascade.
  /// Family/autoDispose providers are covered whole by invalidating the family.
  void advanceAfterSwap() {
    ref.invalidate(healthReportProvider);
    ref.invalidate(householdProvider);
    ref.invalidate(recipeLibraryProvider);
    ref.invalidate(archivedRecipesProvider);
    ref.invalidate(recipeDetailProvider);
    ref.invalidate(planningCycleProvider);
    ref.invalidate(restrictionsProvider);
    ref.invalidate(pantryProvider);
    ref.invalidate(shoppingProvider);
    ref.invalidate(plannerProvider);
    ref.invalidate(coverProvider);
    state++;
  }
}

final databaseGenerationProvider = NotifierProvider<DatabaseGeneration, int>(
  DatabaseGeneration.new,
);
