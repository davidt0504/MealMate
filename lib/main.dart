import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import 'package:path_provider/path_provider.dart';

import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  // ponytail: spike-only — publishes the semantics tree so `uiautomator dump`
  // can locate the probe button for the AC-5 screenshot. Remove with this screen.
  SemanticsBinding.instance.ensureSemantics();
  await RustLib.init();
  final dir = await getApplicationSupportDirectory();
  final dbPath = '${dir.path}${Platform.pathSeparator}kimatta.db';
  runApp(
    MaterialApp(
      home: HealthScreen(
        version: coreVersion(),
        report: healthCheck(dbPath: dbPath),
        probeError: () => healthCheck(dbPath: ' '),
      ),
    ),
  );
}

/// PRE-002 spike screen: renders the two bridge calls and a typed-error probe.
class HealthScreen extends StatefulWidget {
  const HealthScreen({
    super.key,
    required this.version,
    required this.report,
    required this.probeError,
  });

  final String version;
  final Future<HealthReport> report;
  final Future<HealthReport> Function() probeError;

  @override
  State<HealthScreen> createState() => _HealthScreenState();
}

class _HealthScreenState extends State<HealthScreen> {
  String _probeResult = 'not probed';

  Future<void> _probe() async {
    String result;
    try {
      await widget.probeError();
      result = 'no error';
    } on KimattaError catch (e) {
      result = switch (e) {
        KimattaError_InvalidPath() => 'KimattaError.InvalidPath',
        KimattaError_Storage(:final message) =>
          'KimattaError.Storage($message)',
      };
    }
    setState(() => _probeResult = result);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Bridge health (PRE-002)')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('core_version: ${widget.version}'),
            FutureBuilder<HealthReport>(
              future: widget.report,
              builder: (context, snap) => Text(
                snap.hasData
                    ? 'health: schema v${snap.data!.schemaVersion} '
                          'at ${snap.data!.dbPath}'
                    : snap.hasError
                    ? 'health error: ${snap.error}'
                    : 'health: …',
              ),
            ),
            const SizedBox(height: 16),
            FilledButton(
              onPressed: _probe,
              child: const Text('Probe typed error'),
            ),
            Text('probe: $_probeResult'),
          ],
        ),
      ),
    );
  }
}
