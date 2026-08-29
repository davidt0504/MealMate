import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/household.dart';

/// Shown when no name is set. The framing is honest about the state rather
/// than inventing a name (PRD v3 §15: optional household name).
const unnamedHousehold = 'Unnamed household';

String describeHousehold(HouseholdDto h) => switch (h.members.length) {
  1 => 'Just you for now — a household of one.',
  final n => '$n members.',
};

/// `subject` defaults to `'Household'` so every existing call site — and the strings they
/// pin — is unchanged; the planning tile passes `'Planning cycle'` rather than rendering a
/// planning failure as a household one.
String describeFailure(
  Object error, {
  String subject = 'Household',
}) => switch (error) {
  KimattaError_NotOpen() => '$subject unavailable: the database is not open',
  KimattaError_InvalidPath() =>
    '$subject unavailable: the database path is invalid',
  KimattaError_Storage(:final message) => '$subject unavailable: $message',
  // Added with the variant itself, so the new error has prose rather than a raw
  // freezed `toString()` the first time MVP-006's editor can produce it.
  KimattaError_Planning(:final message) => '$subject unavailable: $message',
  // Added with the variant itself, as the Planning arm was, so MVP-006's restriction
  // editor cannot surface a raw freezed `toString()`.
  KimattaError_Restriction(:final message) => '$subject unavailable: $message',
  // Added with MVP-008's editor, as the Planning and Restriction arms were, so a recipe
  // validation error reaches the user as prose rather than a raw freezed `toString()`.
  KimattaError_Recipe(:final message) => '$subject unavailable: $message',
  // Added with the variant itself (MVP-012), as every arm above was, so MVP-013's planner
  // cannot surface a raw freezed `toString()`.
  KimattaError_PlannedMeal(:final message) => '$subject unavailable: $message',
  _ => '$subject unavailable: $error',
};

class HouseholdScreen extends ConsumerStatefulWidget {
  const HouseholdScreen({super.key});

  @override
  ConsumerState<HouseholdScreen> createState() => _HouseholdScreenState();
}

class _HouseholdScreenState extends ConsumerState<HouseholdScreen> {
  final _name = TextEditingController();
  bool _saving = false;
  // Seed the field once, when the household first arrives; a `.text =` in
  // `build` would collapse the selection on every rebuild and re-fill a field
  // the user had just emptied. A successful save is the one other point where
  // overwriting the field is correct — see `_resync`.
  bool _seeded = false;

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  Future<void> _save(HouseholdDto h) async {
    setState(() => _saving = true);
    try {
      // Read the notifier before the await — the last `ref` use on this widget,
      // so a disposal mid-save cannot reach Riverpod's disposed-ref StateError.
      // The notifier outlives the screen, so it publishes the new household
      // whether or not this screen is still around to see it.
      final updated = await ref
          .read(householdProvider.notifier)
          .rename(h.id, _name.text);
      if (mounted) _resync(updated.name ?? '');
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text(describeFailure(e))));
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  /// Storage trims the name and clears a blank one, so after a save the field
  /// has to show what was actually stored rather than what was typed. Caret goes
  /// to the end: `onSubmitted` saves while the field still holds focus.
  void _resync(String stored) {
    if (_name.text == stored) return;
    _name.value = TextEditingValue(
      text: stored,
      selection: TextSelection.collapsed(offset: stored.length),
    );
  }

  @override
  Widget build(BuildContext context) {
    final household = ref.watch(householdProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Household')),
      body: switch (household) {
        AsyncData(:final value) => _body(_seedOnce(value)),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Text(describeFailure(error)),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  HouseholdDto _seedOnce(HouseholdDto h) {
    if (!_seeded) {
      _seeded = true;
      _name.text = h.name ?? '';
    }
    return h;
  }

  Widget _body(HouseholdDto h) => ListView(
    padding: const EdgeInsets.all(16),
    children: [
      ListTile(
        leading: const Icon(Icons.home_outlined),
        title: Text(h.name ?? unnamedHousehold),
        subtitle: Text(describeHousehold(h)),
      ),
      for (final m in h.members)
        ListTile(
          leading: const Icon(Icons.person_outline),
          title: Text(m.displayName),
        ),
      const SizedBox(height: 16),
      TextField(
        controller: _name,
        decoration: const InputDecoration(
          labelText: 'Household name',
          helperText: 'Optional. Leave blank to keep it unnamed.',
        ),
        textInputAction: TextInputAction.done,
        onSubmitted: (_) => _saving ? null : _save(h),
      ),
      const SizedBox(height: 8),
      FilledButton(
        onPressed: _saving ? null : () => _save(h),
        child: const Text('Save name'),
      ),
    ],
  );
}
