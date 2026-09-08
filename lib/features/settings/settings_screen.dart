import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/appearance_provider.dart';
import 'package:meal_mate/app/theme.dart';
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';
import 'package:meal_mate/features/restrictions/restrictions_provider.dart';
import 'package:meal_mate/features/restrictions/restriction_copy.dart';
import 'package:meal_mate/features/settings/backup_copy.dart';
import 'package:meal_mate/features/settings/backup_provider.dart';
import 'package:meal_mate/features/settings/health_provider.dart';
import 'package:meal_mate/src/rust/api/error.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  void _showSnack(BuildContext context, String message) {
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(message)));
  }

  Future<void> _export(BuildContext context, WidgetRef ref) async {
    final actions = ref.read(backupActionsProvider);
    try {
      final report = await actions.export();
      if (context.mounted) {
        _showSnack(context, exportedCopy(report.path));
      }
      try {
        await actions.shareExport(report.path);
      } catch (_) {
        // The export itself succeeded and was reported; a dismissed or
        // unavailable share sheet is not a data failure.
      }
    } catch (e) {
      if (context.mounted) {
        _showSnack(context, describeFailure(e, subject: 'Export'));
      }
    }
  }

  Future<void> _restore(BuildContext context, WidgetRef ref) async {
    final actions = ref.read(backupActionsProvider);
    // Read before the first await and held for the duration: the notifier outlives this
    // element, so releasing the flag cannot depend on the screen still being mounted.
    final busy = ref.read(backupBusyProvider.notifier);
    if (!busy.begin()) {
      return;
    }
    try {
      final latest = await actions.latestExport();
      if (latest == null) {
        if (context.mounted) {
          _showSnack(context, noExportsCopy);
        }
        return;
      }
      if (!context.mounted) {
        return;
      }
      final name = latest.split(RegExp(r'[/\\]')).last;
      final confirmed = await _confirm(
        context,
        title: restoreConfirmTitle,
        body: restoreConfirmBody(name),
        action: restoreConfirmAction,
      );
      if (!confirmed) {
        return;
      }
      try {
        await actions.restore(latest);
      } finally {
        // On both outcomes, and around the call alone rather than the whole method: a
        // refused or cancelled restore swapped nothing, but a *failed* one still did.
        // `restore_database` empties the connection slot before the steps that can fail, and
        // two of `recover_original`'s three exits return without reinstalling one, so a
        // cached health report would keep reporting a database no bridge call can reach —
        // and the recovery row, gated on `health is AsyncError`, would never appear.
        if (context.mounted) {
          ref.read(databaseGenerationProvider.notifier).advanceAfterSwap();
        }
      }
      if (context.mounted) {
        _showSnack(context, restoredCopy);
      }
    } catch (e) {
      if (context.mounted) {
        _showSnack(context, describeFailure(e, subject: 'Restore'));
      }
    } finally {
      busy.end();
    }
  }

  Future<void> _startFresh(BuildContext context, WidgetRef ref) async {
    final actions = ref.read(backupActionsProvider);
    final busy = ref.read(backupBusyProvider.notifier);
    if (!busy.begin()) {
      return;
    }
    try {
      final confirmed = await _confirm(
        context,
        title: startFreshConfirmTitle,
        body: startFreshConfirmBody,
        action: startFreshConfirmAction,
      );
      if (!confirmed) {
        return;
      }
      try {
        await actions.startFresh();
      } finally {
        // As in `_restore`: `reset_database` leaves the slot empty on the way through and
        // returns without refilling it when the reopen fails.
        if (context.mounted) {
          ref.read(databaseGenerationProvider.notifier).advanceAfterSwap();
        }
      }
      if (context.mounted) {
        _showSnack(context, startedFreshCopy);
      }
    } catch (e) {
      if (context.mounted) {
        _showSnack(context, describeFailure(e, subject: 'Start fresh'));
      }
    } finally {
      busy.end();
    }
  }

  Future<bool> _confirm(
    BuildContext context, {
    required String title,
    required String body,
    required String action,
  }) async {
    final answer = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(body),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: Text(action),
          ),
        ],
      ),
    );
    return answer ?? false;
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final health = ref.watch(healthReportProvider);
    final household = ref.watch(householdProvider);
    final appearance = ref.watch(appearanceProvider);
    // Only the destructive pair is gated: an export in flight overwrites nothing.
    final busy = ref.watch(backupBusyProvider);
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
          ListTile(
            title: const Text('Backup'),
            subtitle: const Text(
              'Export a copy of your data, or restore the latest export.',
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Wrap(
              spacing: 8,
              children: [
                FilledButton.tonal(
                  onPressed: () => _export(context, ref),
                  child: const Text(exportButtonLabel),
                ),
                FilledButton.tonal(
                  onPressed: busy ? null : () => _restore(context, ref),
                  child: const Text(restoreButtonLabel),
                ),
              ],
            ),
          ),
          ListTile(title: const Text('Diagnostics'), subtitle: Text(line)),
          if (health is AsyncError)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Wrap(
                spacing: 8,
                children: [
                  FilledButton.tonal(
                    onPressed: () => ref.invalidate(healthReportProvider),
                    child: const Text(tryAgainLabel),
                  ),
                  // Only corruption earns start-fresh: offering it for, say, a
                  // disk-full failure would trade recoverable data for nothing.
                  if (health.error is KimattaError_Corrupt)
                    FilledButton.tonal(
                      onPressed: busy ? null : () => _startFresh(context, ref),
                      child: const Text(startFreshLabel),
                    ),
                ],
              ),
            ),
          // ponytail: MVP-034 Phase B removes this preview section and its provider after
          // the owner's dated palette/type pick. The licenses entry below remains.
          const ListTile(
            key: ValueKey('appearance-preview'),
            title: Text('Appearance preview'),
            subtitle: Text(
              'Temporary choices for evaluating the Kimatta visual identity.',
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
            child: Column(
              children: [
                _AppearancePicker<PaletteChoice>(
                  label: 'Palette',
                  value: appearance.palette,
                  values: PaletteChoice.values,
                  describe: (choice) => choice.label,
                  onChanged: (choice) => ref
                      .read(appearanceProvider.notifier)
                      .selectPalette(choice),
                ),
                const SizedBox(height: 8),
                _AppearancePicker<TypeChoice>(
                  label: 'Type pairing',
                  value: appearance.type,
                  values: TypeChoice.values,
                  describe: (choice) => choice.label,
                  onChanged: (choice) =>
                      ref.read(appearanceProvider.notifier).selectType(choice),
                ),
              ],
            ),
          ),
          ListTile(
            key: const ValueKey('open-source-licenses'),
            title: const Text('Open source licenses'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => showLicensePage(
              context: context,
              applicationName: 'Kimatta (dev)',
            ),
          ),
        ],
      ),
    );
  }
}

class _AppearancePicker<T> extends StatelessWidget {
  const _AppearancePicker({
    required this.label,
    required this.value,
    required this.values,
    required this.describe,
    required this.onChanged,
  });

  final String label;
  final T value;
  final List<T> values;
  final String Function(T value) describe;
  final ValueChanged<T> onChanged;

  @override
  Widget build(BuildContext context) => Semantics(
    label: label,
    child: InputDecorator(
      decoration: InputDecoration(
        labelText: label,
        border: const OutlineInputBorder(),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<T>(
          key: ValueKey('appearance-${label.toLowerCase()}'),
          value: value,
          isExpanded: true,
          items: [
            for (final choice in values)
              DropdownMenuItem<T>(
                value: choice,
                child: Text(
                  describe(choice),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
          ],
          onChanged: (choice) {
            if (choice != null) onChanged(choice);
          },
        ),
      ),
    ),
  );
}
