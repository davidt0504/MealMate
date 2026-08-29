import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';
import 'package:meal_mate/src/rust/api/planning.dart';

/// Bounds mirror `food-domain`'s `MIN_CYCLE_DAYS`/`MAX_CYCLE_DAYS`; Rust remains the
/// authority and rejects anything outside them, this only stops the UI sending a request it
/// already knows is invalid.
const minCycleDays = 1;
const maxCycleDays = 31;

class CycleEditorScreen extends ConsumerStatefulWidget {
  const CycleEditorScreen({super.key});

  @override
  ConsumerState<CycleEditorScreen> createState() => _CycleEditorScreenState();
}

class _CycleEditorScreenState extends ConsumerState<CycleEditorScreen> {
  final Set<MealSlotDto> _slots = {};
  int _length = 0;
  String _anchor = '';
  bool _saving = false;
  // Seeded once, as `HouseholdScreen` does: the provider republishes on save, so an
  // unguarded reseed in `build` would overwrite what the user is editing.
  bool _seeded = false;

  void _seedOnce(PlanningCycleDto c) {
    if (_seeded) return;
    _seeded = true;
    _slots.addAll(c.mealSlots);
    _length = c.lengthDays;
    _anchor = c.anchorDate;
  }

  Future<void> _save(String householdId) async {
    setState(() => _saving = true);
    try {
      // Sorted by index before sending, so what is stored and returned already matches the
      // canonical order Rust uses; without it the round-trip reorders under the user.
      final slots = _slots.toList()..sort((a, b) => a.index.compareTo(b.index));
      await ref
          .read(planningCycleProvider.notifier)
          .save(
            householdId,
            anchorDate: _anchor,
            lengthDays: _length,
            mealSlots: slots,
          );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(describeFailure(e, subject: 'Planning cycle')),
          ),
        );
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final cycle = ref.watch(planningCycleProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Planning cycle')),
      // The form is built only under AsyncData: `_seedOnce` is only sound when the form
      // cannot exist before its data does, and a stepper over an unseeded length and an
      // empty anchor is exactly the failure that guard is there to prevent.
      body: switch (cycle) {
        AsyncData(:final value) => _body(value),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Text(describeFailure(error, subject: 'Planning cycle')),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Widget _body(PlanningCycleDto c) {
    _seedOnce(c);
    final householdId = ref.watch(householdProvider).valueOrNull?.id;
    final canSave = !_saving && _slots.isNotEmpty && householdId != null;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        for (final slot in MealSlotDto.values)
          CheckboxListTile(
            title: Text(slot.name[0].toUpperCase() + slot.name.substring(1)),
            value: _slots.contains(slot),
            onChanged: (on) => setState(() {
              if (on ?? false) {
                _slots.add(slot);
              } else {
                _slots.remove(slot);
              }
            }),
          ),
        if (_slots.isEmpty) const Text('Pick at least one meal.'),
        const SizedBox(height: 8),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            IconButton(
              icon: const Icon(Icons.remove),
              tooltip: 'Fewer days',
              onPressed: _length > minCycleDays
                  ? () => setState(() => _length -= 1)
                  : null,
            ),
            Text('$_length days'),
            IconButton(
              icon: const Icon(Icons.add),
              tooltip: 'More days',
              onPressed: _length < maxCycleDays
                  ? () => setState(() => _length += 1)
                  : null,
            ),
          ],
        ),
        const SizedBox(height: 8),
        Text('Starts $_anchor'),
        TextButton(
          onPressed: () => setState(() => _anchor = todayCivilDate()),
          child: const Text('Start from today'),
        ),
        const SizedBox(height: 8),
        FilledButton(
          onPressed: canSave ? () => _save(householdId) : null,
          child: const Text('Save cycle'),
        ),
      ],
    );
  }
}
