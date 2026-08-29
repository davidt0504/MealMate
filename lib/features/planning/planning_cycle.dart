import 'package:meal_mate/src/rust/api/planning.dart';

/// The local civil date, formatted ISO. Deliberately built from the *local* Y/M/D of a
/// `DateTime` and never from `toUtc()` or `toIso8601String()`: at 23:30 local in a
/// UTC-negative zone the UTC instant is already the next day, and sending that would move
/// "Tuesday dinner" a day (PRD v3 §8, MVP-005 stop condition).
String todayCivilDate([DateTime? now]) {
  final d = now ?? DateTime.now();
  final month = d.month.toString().padLeft(2, '0');
  final day = d.day.toString().padLeft(2, '0');
  return '${d.year}-$month-$day';
}

/// One honest line about the cycle for Settings — no planner language, since the planner
/// arrives with MVP-013/MVP-024. Reads `anchorDate` verbatim (already ISO from Rust) and
/// does no date arithmetic.
String describePlanningCycle(PlanningCycleDto c) {
  final names = c.mealSlots.map((s) => s.name).toList();
  final first = names.first[0].toUpperCase() + names.first.substring(1);
  final scope = names.length == 1
      ? '$first only'
      : [first, ...names.skip(1)].join(', ');
  final unit = c.lengthDays == 1 ? 'day' : 'days';
  return '$scope · ${c.lengthDays} $unit from ${c.anchorDate}';
}
