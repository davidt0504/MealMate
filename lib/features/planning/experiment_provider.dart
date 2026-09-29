import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';

import 'package:meal_mate/src/rust/api/experiment.dart';

/// The tester build (`--dart-define=KIMATTA_TESTER=true`): the only build that shows the
/// experiment controls (OPT-007 §10). Behind a provider so a test can flip it.
final testerModeProvider = Provider<bool>(
  (_) => const bool.fromEnvironment('KIMATTA_TESTER'),
);

/// Where the experiment's own small file lives: the app-support directory, beside — never
/// inside — the household database.
final experimentDirProvider = FutureProvider<String>(
  (_) async => (await getApplicationSupportDirectory()).path,
);

/// The bridge calls behind one overridable seam, as the backup actions are.
class ExperimentBackend {
  const ExperimentBackend();

  Future<ExperimentSessionDto> startSession(String dir) =>
      experimentStartSession(dir: dir);

  Future<bool> record(String dir, ExperimentEventDto event) =>
      experimentRecord(dir: dir, event: event);

  Future<ExperimentStatusDto> status(String dir) => experimentStatus(dir: dir);

  Future<void> setEnabled(String dir, bool enabled) =>
      experimentSetEnabled(dir: dir, enabled: enabled);

  Future<void> setOverride(String dir, String? labelId) =>
      experimentSetOverride(dir: dir, labelId: labelId);

  Future<void> reset(String dir) => experimentReset(dir: dir);

  Future<int> export(String dir, String destPath) =>
      experimentExport(dir: dir, destPath: destPath);
}

final experimentBackendProvider = Provider<ExperimentBackend>(
  (_) => const ExperimentBackend(),
);

/// Active foreground time for one session: a monotonic stopwatch, paused while the app is in
/// the background. Behind a provider so a test can drive it.
final activeClockProvider = Provider.autoDispose<Stopwatch>((_) => Stopwatch());

/// One Cover visit is one experiment session: its label is fixed when it starts, so tester
/// changes wait for the next visit and never swap a label mid-review. Recording is best
/// effort — a failure to write tester telemetry must never disturb planning — and does
/// nothing unless the experiment is enabled for this session.
class WeekActionExperiment
    extends AutoDisposeAsyncNotifier<ExperimentSessionDto> {
  late Stopwatch _clock;
  bool _accepted = false;

  @override
  Future<ExperimentSessionDto> build() async {
    // The experiment file outlives a reinstall, so the build is the gate: a release over a
    // tester build must neither show a variant nor record, and it has no screen to stop it.
    if (!ref.watch(testerModeProvider)) {
      return const ExperimentSessionDto(
        sessionId: '',
        labelId: 'new_mix',
        label: 'New mix',
        active: false,
        overridden: false,
      );
    }
    _clock = ref.watch(activeClockProvider);
    final dir = await ref.watch(experimentDirProvider.future);
    final backend = ref.read(experimentBackendProvider);
    final session = await backend.startSession(dir);
    _clock
      ..reset()
      ..start();
    final lifecycle = AppLifecycleListener(
      onHide: () => _pause(),
      onShow: () => _resume(),
    );
    ref.onDispose(() {
      lifecycle.dispose();
      _clock.stop();
      _send(
        backend,
        dir,
        session,
        ExperimentEventKindDto.end,
        completed: _accepted,
      );
    });
    return session;
  }

  void _pause() {
    if (!_clock.isRunning) return;
    _clock.stop();
    record(ExperimentEventKindDto.background);
  }

  void _resume() {
    if (_clock.isRunning) return;
    _clock.start();
    record(ExperimentEventKindDto.resume);
  }

  void _send(
    ExperimentBackend backend,
    String dir,
    ExperimentSessionDto session,
    ExperimentEventKindDto kind, {
    int changed = 0,
    int exhausted = 0,
    bool completed = false,
  }) {
    if (!session.active) return;
    unawaited(
      backend
          .record(
            dir,
            ExperimentEventDto(
              sessionId: session.sessionId,
              kind: kind,
              elapsedMs: BigInt.from(_clock.elapsedMilliseconds),
              changed: changed,
              exhausted: exhausted,
              completed: completed,
            ),
          )
          .then<void>((_) {}, onError: (Object _) {}),
    );
  }

  /// Records one event of this session, if it is recording at all.
  void record(
    ExperimentEventKindDto kind, {
    int changed = 0,
    int exhausted = 0,
    bool completed = false,
  }) {
    final session = state.valueOrNull;
    final dir = ref.read(experimentDirProvider).valueOrNull;
    if (session == null || dir == null) return;
    if (kind == ExperimentEventKindDto.acceptSuccess) _accepted = true;
    _send(
      ref.read(experimentBackendProvider),
      dir,
      session,
      kind,
      changed: changed,
      exhausted: exhausted,
      completed: completed,
    );
  }
}

final weekActionExperimentProvider =
    AsyncNotifierProvider.autoDispose<
      WeekActionExperiment,
      ExperimentSessionDto
    >(WeekActionExperiment.new);
