import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/src/rust/api/starter.dart';
import 'package:meal_mate/src/rust/api/starter.dart' as bridge;

typedef StarterInstall = Future<StarterInstallReportDto> Function(
  String householdId,
);

/// The one starter-content seam (MVP-011). A plain function provider rather than an
/// `AsyncNotifier`: `App` calls it once per launch and nothing watches its result, so
/// there is no state to publish and nothing to invalidate.
///
/// It is overridden unconditionally in the widget harness for the same reason every other
/// bridge provider is — `flutter test` never calls `RustLib.init()`, so an un-overridden
/// provider throws on the first frame.
final starterInstallProvider = Provider<StarterInstall>(
  (_) =>
      (householdId) => bridge.installStarterContent(householdId: householdId),
);
