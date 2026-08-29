import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';

import 'package:meal_mate/src/rust/api/health.dart';

/// Opens/migrates the app-private database. Read once at startup by [App]
/// (PRD v3 §6.3: one coarse call); Settings displays the result.
/// Never `Directory.systemTemp`: it resolves to the unwritable /data/local/tmp
/// on Android.
final healthReportProvider = FutureProvider<HealthReport>((ref) async {
  final dir = await getApplicationSupportDirectory();
  return openDatabase(dbPath: '${dir.path}${Platform.pathSeparator}kimatta.db');
});
