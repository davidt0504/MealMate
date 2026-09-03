import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';

import 'package:meal_mate/src/rust/api/health.dart';

/// Opens/migrates the app-private database. Read once at startup by [App]
/// (PRD v3 §6.3: one coarse call); Settings displays the result.
/// Never `Directory.systemTemp`: it resolves to the unwritable /data/local/tmp
/// on Android.
final healthReportProvider = FutureProvider<HealthReport>((ref) async {
  return openDatabase(dbPath: await localDatabasePath());
});

/// The one place the live database's location is spelled. Restore and start-fresh call this
/// too, so a later move of the database cannot leave the destructive operations pointed at a
/// path the app never opens — which would report success while nothing visible changed.
Future<String> localDatabasePath() async {
  final dir = await getApplicationSupportDirectory();
  return '${dir.path}${Platform.pathSeparator}kimatta.db';
}
