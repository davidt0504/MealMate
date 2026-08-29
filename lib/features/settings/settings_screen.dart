import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/settings/health_provider.dart';
import 'package:meal_mate/src/rust/api/error.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final health = ref.watch(healthReportProvider);
    final household = ref.watch(householdProvider);
    final line = switch (health) {
      AsyncData(:final value) =>
        'Local database: schema v${value.schemaVersion} at ${value.dbPath}',
      AsyncError(:final error) => switch (error) {
        KimattaError_NotOpen() =>
          'Local database unavailable: the database is not open',
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
          ListTile(
            title: const Text('Household'),
            subtitle: Text(switch (household) {
              AsyncData(:final value) =>
                '${value.name ?? unnamedHousehold} · ${describeHousehold(value)}',
              AsyncError(:final error) => describeFailure(error),
              _ => 'Loading household…',
            }),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => context.go('/settings/household'),
          ),
          ListTile(title: const Text('Diagnostics'), subtitle: Text(line)),
        ],
      ),
    );
  }
}
