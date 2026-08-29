import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';
import 'package:meal_mate/features/restrictions/restrictions_provider.dart';
import 'package:meal_mate/features/restrictions/restriction_copy.dart';
import 'package:meal_mate/features/settings/health_provider.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final health = ref.watch(healthReportProvider);
    final household = ref.watch(householdProvider);
    final line = switch (health) {
      AsyncData(:final value) =>
        'Local database: schema v${value.schemaVersion} at ${value.dbPath}',
      AsyncError(:final error) => describeFailure(
        error,
        subject: 'Local database',
      ),
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
          ListTile(
            title: const Text('Planning cycle'),
            subtitle: Text(switch (ref.watch(planningCycleProvider)) {
              AsyncData(:final value) => describePlanningCycle(value),
              AsyncError(:final error) => describeFailure(
                error,
                subject: 'Planning cycle',
              ),
              _ => 'Loading planning cycle…',
            }),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => context.go('/settings/cycle'),
          ),
          ListTile(
            title: const Text('Restrictions'),
            subtitle: Text(switch (ref.watch(restrictionsProvider)) {
              AsyncData(value: final list) when list.isEmpty =>
                'None set — nothing is filtered out.',
              AsyncData(:final value) =>
                value.map(describeRestriction).join(', '),
              AsyncError(:final error) => describeFailure(
                error,
                subject: 'Restrictions',
              ),
              _ => 'Loading restrictions…',
            }),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => context.go('/settings/restrictions'),
          ),
          ListTile(title: const Text('Diagnostics'), subtitle: Text(line)),
        ],
      ),
    );
  }
}
