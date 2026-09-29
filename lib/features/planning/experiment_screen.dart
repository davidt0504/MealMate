import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
import 'package:meal_mate/features/planning/experiment_provider.dart';
import 'package:meal_mate/features/settings/backup_provider.dart';
import 'package:meal_mate/src/rust/api/experiment.dart';

const experimentTitle = 'Label experiment';
const experimentNextSessionCopy =
    'Changes here start with the next visit to Cover My Week.';

/// Tester-only (OPT-007 §10): run, steer, reset or export the week-action wording experiment.
/// Nothing here reaches the household's data; the export is its own small file of coarse
/// events, written only when the tester asks.
class ExperimentScreen extends ConsumerStatefulWidget {
  const ExperimentScreen({super.key});

  @override
  ConsumerState<ExperimentScreen> createState() => _ExperimentScreenState();
}

class _ExperimentScreenState extends ConsumerState<ExperimentScreen> {
  ExperimentStatusDto? _status;
  Object? _error;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  ExperimentBackend get _backend => ref.read(experimentBackendProvider);

  Future<void> _refresh() async {
    try {
      final dir = await ref.read(experimentDirProvider.future);
      final status = await _backend.status(dir);
      if (mounted) setState(() => _status = status);
    } catch (e) {
      if (mounted) setState(() => _error = e);
    }
  }

  Future<void> _act(Future<String?> Function(String dir) action) async {
    setState(() => _busy = true);
    final messenger = ScaffoldMessenger.of(context);
    try {
      final dir = await ref.read(experimentDirProvider.future);
      final message = await action(dir);
      if (message != null) {
        messenger.showSnackBar(SnackBar(content: Text(message)));
      }
      await _refresh();
    } catch (e) {
      messenger.showSnackBar(
        SnackBar(content: Text(describeFailure(e, subject: experimentTitle))),
      );
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final status = _status;
    return Scaffold(
      appBar: AppBar(title: const Text(experimentTitle)),
      body: status == null
          ? Center(
              child: _error == null
                  ? const CircularProgressIndicator()
                  : Text(describeFailure(_error!, subject: experimentTitle)),
            )
          : ListView(
              padding: const EdgeInsets.symmetric(vertical: 8),
              children: [
                const ListTile(title: Text(experimentNextSessionCopy)),
                SwitchListTile(
                  key: const ValueKey('experiment:enabled'),
                  title: const Text('Run the experiment'),
                  value: status.nextEnabled ?? status.enabled,
                  onChanged: _busy
                      ? null
                      : (on) => _act((dir) async {
                          await _backend.setEnabled(dir, on);
                          return null;
                        }),
                ),
                ListTile(
                  title: const Text('Assigned label'),
                  subtitle: Text(_labelName(status, status.assigned)),
                ),
                ListTile(
                  title: const Text('Show instead'),
                  subtitle: const Text(
                    'Sessions with a chosen label are marked and left out of the comparison.',
                  ),
                  trailing: DropdownButton<String?>(
                    key: const ValueKey('experiment:override'),
                    value: status.overridePending
                        ? status.nextOverride
                        : status.overrideLabel,
                    onChanged: _busy
                        ? null
                        : (id) => _act((dir) async {
                            await _backend.setOverride(dir, id);
                            return null;
                          }),
                    items: [
                      const DropdownMenuItem(child: Text('Assigned')),
                      for (final l in status.labels)
                        DropdownMenuItem(value: l.id, child: Text(l.label)),
                    ],
                  ),
                ),
                ListTile(
                  title: Text('${status.events} events recorded'),
                  subtitle: status.dropped > BigInt.zero
                      ? Text('${status.dropped} oldest events were dropped')
                      : null,
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Wrap(
                    spacing: 8,
                    children: [
                      FilledButton.tonal(
                        key: const ValueKey('experiment:export'),
                        onPressed: _busy
                            ? null
                            : () => _act((dir) async {
                                final dest = [
                                  dir,
                                  'exports',
                                  'wording-experiment.json',
                                ].join(Platform.pathSeparator);
                                final n = await _backend.export(dir, dest);
                                // App support is private on a release APK: the share sheet
                                // is the only way the tester can get the file off.
                                try {
                                  await ref
                                      .read(backupActionsProvider)
                                      .shareExport(dest);
                                } catch (_) {
                                  return 'Exported $n events; sharing failed';
                                }
                                return 'Exported $n events';
                              }),
                        child: const Text('Export'),
                      ),
                      TextButton(
                        key: const ValueKey('experiment:reset'),
                        onPressed: _busy
                            ? null
                            : () => _act((dir) async {
                                await _backend.reset(dir);
                                return 'Experiment reset';
                              }),
                        child: const Text('Reset'),
                      ),
                    ],
                  ),
                ),
              ],
            ),
    );
  }

  String _labelName(ExperimentStatusDto status, String? id) =>
      status.labels.where((l) => l.id == id).map((l) => l.label).firstOrNull ??
      'Not assigned yet';
}
