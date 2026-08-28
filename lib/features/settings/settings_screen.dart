import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/settings/health_provider.dart';
import 'package:meal_mate/src/rust/api/health.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final health = ref.watch(healthReportProvider);
    final line = switch (health) {
      AsyncData(:final value) =>
        'Local database: schema v${value.schemaVersion} at ${value.dbPath}',
      AsyncError(:final error) => switch (error) {
        KimattaError_InvalidPath() =>
          'Local database unavailable: the database path is invalid',
        KimattaError_Storage(:final message) =>
          'Local database unavailable: $message',
        // `error` is statically `Object`, so this arm stays mandatory even
        // though the two above exhaust the sealed union. With both variants
        // matched it now means "not one of ours".
        _ => 'Local database unavailable: $error',
      },
      _ => 'Local database: checking…',
    };
    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        children: [
          const ListTile(
            title: Text('Preferences'),
            subtitle: Text(
              'Household and restriction settings arrive with onboarding.',
            ),
          ),
          ListTile(title: const Text('Diagnostics'), subtitle: Text(line)),
        ],
      ),
    );
  }
}
