import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/pantry/pantry_copy.dart';
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/src/rust/api/pantry.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

class PantryScreen extends ConsumerStatefulWidget {
  const PantryScreen({super.key});

  @override
  ConsumerState<PantryScreen> createState() => _PantryScreenState();
}

class _PantryScreenState extends ConsumerState<PantryScreen> {
  final _search = TextEditingController();
  String _query = '';
  // Per-row rather than screen-wide: the pantry is edited one tap at a time, and a screen-wide
  // gate would freeze 37 rows while one write lands.
  final Set<IngredientRefDto> _writing = {};

  @override
  void dispose() {
    _search.dispose();
    super.dispose();
  }

  /// Name *or* alias, case-folded: `ingredient_alias` ships names like `garbanzo beans` that
  /// are not substrings of their canonical name, so a name-only filter would find nothing for
  /// the word the user actually knows. No bridge call — the whole list is already in hand.
  bool _matches(PantryEntryDto entry, String folded) {
    if (folded.isEmpty) return true;
    if (entry.name.toLowerCase().contains(folded)) return true;
    return entry.aliases.any((a) => a.toLowerCase().contains(folded));
  }

  Future<void> _toggle(
    String householdId,
    PantryEntryDto entry,
    bool marked,
  ) async {
    setState(() => _writing.add(entry.ingredient));
    try {
      await ref
          .read(pantryProvider.notifier)
          .setMark(householdId, entry.ingredient, marked);
    } catch (e) {
      // The switch is rendered from provider state, which the failed write never changed, so
      // the row is already back at its stored value; the snackbar says why.
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Pantry'))),
        );
      }
    } finally {
      if (mounted) setState(() => _writing.remove(entry.ingredient));
    }
  }

  /// Both refresh affordances. `PantryNotifier.refresh` returns the failure rather than
  /// publishing it over a list the user is reading, so the report lands in the same snackbar
  /// every other pantry failure on this screen uses.
  Future<void> _refresh() async {
    final error = await ref.read(pantryProvider.notifier).refresh();
    if (error != null && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(describeFailure(error, subject: 'Pantry'))),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final pantry = ref.watch(pantryProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Pantry')),
      body: switch (pantry) {
        AsyncData(value: final entries) => _body(entries),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(describeFailure(error, subject: 'Pantry')),
              const SizedBox(height: 16),
              // The list is read once per launch, so without this a failed read is permanent
              // until the app is killed.
              FilledButton(onPressed: _refresh, child: const Text('Try again')),
            ],
          ),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Widget _body(List<PantryEntryDto> entries) {
    final householdId = ref.watch(householdProvider).valueOrNull?.id;
    // Decided once here rather than per row. Unreachable today — `PantryNotifier.build` awaits
    // the household id, so pantry data implies household data — but the per-row arm it
    // replaces turned every switch on the screen inert with nothing saying why.
    if (householdId == null) {
      return const Center(child: CircularProgressIndicator());
    }
    final folded = _query.trim().toLowerCase();
    final shown = [
      for (final entry in entries)
        if (_matches(entry, folded)) entry,
    ];
    return RefreshIndicator(
      onRefresh: _refresh,
      child: ListView(
        padding: const EdgeInsets.all(16),
        // Without this the gesture is inert in exactly the state `pantryEmptyCopy` tells the
        // user to perform it: default physics give a non-overflowing list zero scroll extent,
        // so `RefreshIndicator` never fires — and the list is shortest when it is empty.
        physics: const AlwaysScrollableScrollPhysics(),
        children: [
          const Text(pantryDisclaimer),
          const SizedBox(height: 8),
          TextField(
            controller: _search,
            decoration: const InputDecoration(labelText: 'Search'),
            onChanged: (value) => setState(() => _query = value),
          ),
          const SizedBox(height: 8),
          if (entries.isEmpty)
            const Text(pantryEmptyCopy)
          else if (shown.isEmpty)
            Text(pantryNoMatchCopy(_query.trim()))
          else
            for (final entry in shown) _row(entry, householdId),
        ],
      ),
    );
  }

  Widget _row(PantryEntryDto entry, String householdId) {
    final busy = _writing.contains(entry.ingredient);
    // The label goes *inside* the tile, not around it: `SwitchListTile` returns a
    // `MergeSemantics` whose merged node is the tappable one, so a wrapper outside it would
    // become a separate parent node and leave the tappable node with no name at all
    // (`labeledTapTargetGuideline` catches exactly that). `ExcludeSemantics` on the title and
    // subtitle keeps the merged name exactly `pantryRowLabel` rather than concatenating the
    // visible strings after it — the meaning stated before the platform's own on/off, which
    // no label can suppress.
    return SwitchListTile(
      title: Semantics(
        label: pantryRowLabel(entry.name, entry.marked),
        child: ExcludeSemantics(child: Text(entry.name)),
      ),
      subtitle: entry.ingredient is IngredientRefDto_Custom
          ? const ExcludeSemantics(child: Text('Your ingredient'))
          : null,
      value: entry.marked,
      onChanged: busy ? null : (marked) => _toggle(householdId, entry, marked),
    );
  }
}
