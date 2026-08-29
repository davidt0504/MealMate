import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/restrictions/restrictions_provider.dart';
import 'package:meal_mate/src/rust/api/restrictions.dart';

/// The honesty line, shown first and never softened (invariant 10, and the card's
/// "restrictions are preferences/warnings, not medical assurance").
const restrictionsDisclaimer =
    'Warnings only. Meal Mate flags what it knows about; it can never tell you '
    'a recipe is safe.';

/// The `_ => kind` arm renders the raw token, which is honest; Step 8's bridge cross-check
/// test is what makes a kind added in Rust without a label here a test failure rather than a
/// silent user-visible regression.
String restrictionLabel(String kind) => switch (kind) {
  'peanuts' => 'Peanuts',
  'tree_nuts' => 'Tree nuts',
  'dairy' => 'Dairy',
  'eggs' => 'Eggs',
  'gluten' => 'Gluten',
  'soy' => 'Soy',
  'fish' => 'Fish',
  'shellfish' => 'Shellfish',
  'sesame' => 'Sesame',
  'vegetarian' => 'Vegetarian',
  'vegan' => 'Vegan',
  _ => kind,
};

/// Both the chip list and the Settings subtitle go through this, so no rendering path is
/// left without an arm.
String describeRestriction(RestrictionDto r) => switch (r) {
  RestrictionDto_Known(:final kind) => restrictionLabel(kind),
  RestrictionDto_Other(:final text) => text,
};

class RestrictionsScreen extends ConsumerStatefulWidget {
  const RestrictionsScreen({super.key});

  @override
  ConsumerState<RestrictionsScreen> createState() => _RestrictionsScreenState();
}

class _RestrictionsScreenState extends ConsumerState<RestrictionsScreen> {
  final _entry = TextEditingController();
  final Set<String> _known = {};
  final List<String> _other = [];
  bool _saving = false;
  // Seeded once: `save` republishes the provider, so an unguarded reseed in `build` would
  // re-add a chip the user had just deleted.
  bool _seeded = false;

  @override
  void dispose() {
    _entry.dispose();
    super.dispose();
  }

  void _seedOnce(List<RestrictionDto> stored) {
    if (_seeded) return;
    _seeded = true;
    for (final r in stored) {
      switch (r) {
        case RestrictionDto_Known(:final kind):
          _known.add(kind);
        case RestrictionDto_Other(:final text):
          _other.add(text);
      }
    }
  }

  List<RestrictionDto> _pending(List<String> kinds) => [
    for (final kind in kinds)
      if (_known.contains(kind)) RestrictionDto.known(kind: kind),
    for (final text in _other) RestrictionDto.other(text: text),
  ];

  Future<void> _save(String householdId, List<String> kinds) async {
    setState(() => _saving = true);
    try {
      final stored = await ref
          .read(restrictionsProvider.notifier)
          .save(householdId, _pending(kinds));
      if (mounted) {
        setState(() {
          _known
            ..clear()
            ..addAll([
              for (final r in stored)
                if (r case RestrictionDto_Known(:final kind)) kind,
            ]);
          _other
            ..clear()
            ..addAll([
              for (final r in stored)
                if (r case RestrictionDto_Other(:final text)) text,
            ]);
        });
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Restrictions'))),
        );
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final restrictions = ref.watch(restrictionsProvider);
    final kinds = ref.watch(knownRestrictionKindsProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Restrictions')),
      // The form exists only under AsyncData, and only once the vocabulary has arrived. This
      // is load-bearing rather than symmetry: `_known`/`_other` start empty and Save writes
      // the whole set, so a Save tapped before the load resolved — or on the error arm —
      // would send an empty list and silently clear every stored restriction. For a screen
      // whose whole purpose is not under-warning, that is the worst failure direction.
      body: switch ((restrictions, kinds)) {
        (AsyncData(value: final stored), AsyncData(value: final kinds)) =>
          _body(stored, kinds),
        (AsyncError(:final error), _) ||
        (_, AsyncError(:final error)) => Padding(
          padding: const EdgeInsets.all(24),
          child: Text(describeFailure(error, subject: 'Restrictions')),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Widget _body(List<RestrictionDto> stored, List<String> kinds) {
    _seedOnce(stored);
    final householdId = ref.watch(householdProvider).valueOrNull?.id;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        const Text(restrictionsDisclaimer),
        const SizedBox(height: 8),
        // The checkbox list, `Add` and chip delete are all dead while a save is in flight,
        // not just Save: `_save` re-seeds `_known`/`_other` from what came back, so a box
        // ticked during the await would be dropped on completion with nothing on screen to
        // say so. The text field stays live because typing alone changes nothing — `_add`
        // is what commits it, and that is gated with the rest.
        for (final kind in kinds)
          CheckboxListTile(
            title: Text(restrictionLabel(kind)),
            value: _known.contains(kind),
            onChanged: _saving
                ? null
                : (on) => setState(() {
                    if (on ?? false) {
                      _known.add(kind);
                    } else {
                      _known.remove(kind);
                    }
                  }),
          ),
        if (_other.isNotEmpty)
          Wrap(
            spacing: 8,
            children: [
              for (final text in _other)
                Chip(
                  label: Text(text),
                  onDeleted: _saving
                      ? null
                      : () => setState(() => _other.remove(text)),
                ),
            ],
          ),
        TextField(
          controller: _entry,
          decoration: const InputDecoration(
            labelText: 'Something else',
            helperText: 'Anything the list above does not cover.',
          ),
        ),
        const SizedBox(height: 8),
        OutlinedButton(
          onPressed: _saving ? null : () => _add(kinds),
          child: const Text('Add'),
        ),
        const SizedBox(height: 8),
        FilledButton(
          onPressed: _saving || householdId == null
              ? null
              : () => _save(householdId, kinds),
          child: const Text('Save restrictions'),
        ),
      ],
    );
  }

  /// A blank entry warns about nothing, so it is not added — the same rule Rust enforces.
  /// Text naming a kind the checkbox list already covers ticks that box instead of adding a
  /// second entry: `is_same` (`food-domain/src/restriction.rs`) never collapses a
  /// `Known`/`Other` pair, so both would round-trip intact and the same allergen would be
  /// shown — and matched — twice. Ticking an already-ticked box changes nothing, which is
  /// the honest answer: what the user asked to be warned about is already stored.
  void _add(List<String> kinds) {
    final text = _entry.text.trim();
    if (text.isEmpty) return;
    final folded = text.toLowerCase();
    String? matched;
    for (final kind in kinds) {
      if (kind.toLowerCase() == folded ||
          restrictionLabel(kind).toLowerCase() == folded) {
        matched = kind;
        break;
      }
    }
    setState(() {
      if (matched != null) {
        _known.add(matched);
      } else if (!_other.contains(text)) {
        _other.add(text);
      }
      _entry.clear();
    });
  }
}
