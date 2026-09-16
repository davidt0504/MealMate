import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/pantry/custom_ingredient_form.dart';
import 'package:meal_mate/features/pantry/pantry_categorize_sheet.dart';
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
  // gate would freeze every row while one write lands.
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

  Future<void> _write(
    PantryEntryDto entry,
    Future<PantryEntryDto> Function() write,
  ) async {
    setState(() => _writing.add(entry.ingredient));
    try {
      await write();
    } catch (e) {
      // The row is rendered from provider state, which a failed write never changed, so the
      // row already reflects its stored value; the snackbar says why.
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Pantry'))),
        );
      }
    } finally {
      if (mounted) setState(() => _writing.remove(entry.ingredient));
    }
  }

  /// Marking "have" is a plain mark. Unmarking routes through the same combined
  /// unmark+restock-flag transaction "Used up" uses — gate 4's safety net ("no household
  /// discipline required to avoid silent drift") must hold on *every* path to "not marked",
  /// not only when the household happens to tap the smaller "Used up" chip instead of the
  /// larger, more obvious row toggle.
  Future<void> _toggleMark(String householdId, PantryEntryDto entry) => _write(
    entry,
    () => entry.marked
        ? ref.read(pantryProvider.notifier).setUsedUp(
            householdId,
            entry.ingredient,
          )
        : ref
              .read(pantryProvider.notifier)
              .setMark(householdId, entry.ingredient, true),
  );

  Future<void> _toggleRestock(String householdId, PantryEntryDto entry) =>
      _write(
        entry,
        () => ref
            .read(pantryProvider.notifier)
            .setRestockFlag(
              householdId,
              entry.ingredient,
              !entry.restockRequested,
            ),
      );

  Future<void> _usedUp(String householdId, PantryEntryDto entry) => _write(
    entry,
    () => ref.read(pantryProvider.notifier).setUsedUp(
      householdId,
      entry.ingredient,
    ),
  );

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

  Future<void> _openCategorizeSheet(
    String householdId,
    List<PantryEntryDto> needsCategory,
  ) => showModalBottomSheet<void>(
    context: context,
    isScrollControlled: true,
    builder: (_) => CategorizeMissingIngredientsSheet(
      householdId: householdId,
      entries: needsCategory,
    ),
  );

  Future<void> _openCreateForm(String householdId, String prefillName) =>
      Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => CustomIngredientFormScreen(
            householdId: householdId,
            initialName: prefillName,
          ),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final pantry = ref.watch(pantryProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('My Shelves')),
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
    // replaces turned every control on the screen inert with nothing saying why.
    if (householdId == null) {
      return const Center(child: CircularProgressIndicator());
    }
    final folded = _query.trim().toLowerCase();
    final searching = folded.isNotEmpty;

    return RefreshIndicator(
      onRefresh: _refresh,
      child: ListView(
        padding: const EdgeInsets.all(16),
        // Without this the gesture is inert in exactly the state the empty copy tells the
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
          else if (searching)
            ..._searchResults(entries, folded, householdId)
          else
            ..._myShelves(entries, householdId),
        ],
      ),
    );
  }

  /// Gate 2's "expand in place": the same screen, filtered to the full catalog (marked and
  /// unmarked) with no grouping — grouping is a "My Shelves" property, not a browse property.
  List<Widget> _searchResults(
    List<PantryEntryDto> entries,
    String folded,
    String householdId,
  ) {
    final shown = [
      for (final entry in entries)
        if (_matches(entry, folded)) entry,
    ];
    final hasExactMatch = entries.any(
      (e) => e.name.toLowerCase() == folded,
    );
    return [
      if (shown.isEmpty)
        Text(pantryNoMatchCopy(_query.trim()))
      else
        for (final entry in shown) _row(entry, householdId),
      if (!hasExactMatch)
        Align(
          alignment: Alignment.centerLeft,
          child: TextButton.icon(
            onPressed: () => _openCreateForm(householdId, _query.trim()),
            icon: const Icon(Icons.add),
            label: Text("Add '${_query.trim()}' as a new ingredient"),
          ),
        ),
    ];
  }

  /// Gate 2's default view: only have-marked rows, grouped by `store_category`. A marked row
  /// with no category (a pre-existing custom ingredient predating OPT-006's required-category
  /// rule) is excluded from the grouped list and surfaced via the categorize banner instead —
  /// never a grouped "uncategorised" bucket (gate 5's closed decision).
  List<Widget> _myShelves(List<PantryEntryDto> entries, String householdId) {
    final marked = [
      for (final entry in entries)
        if (entry.marked) entry,
    ];
    final needsCategory = [
      for (final entry in marked)
        if (entry.storeCategory == null) entry,
    ];
    final grouped = <String, List<PantryEntryDto>>{};
    for (final entry in marked) {
      final category = entry.storeCategory;
      if (category != null) grouped.putIfAbsent(category, () => []).add(entry);
    }
    final categories = grouped.keys.toList()..sort();
    final theme = Theme.of(context).textTheme;
    return [
      if (needsCategory.isNotEmpty)
        _CategorizeBanner(
          count: needsCategory.length,
          onTap: () => _openCategorizeSheet(householdId, needsCategory),
        ),
      if (marked.isEmpty)
        const Text(pantryNothingMarkedCopy)
      else
        for (final category in categories) ...[
          Text(category, style: theme.titleSmall),
          for (final entry in grouped[category]!) _row(entry, householdId),
        ],
    ];
  }

  Widget _row(PantryEntryDto entry, String householdId) {
    final busy = _writing.contains(entry.ingredient);
    // The have-mark's label goes *inside* the tappable node, not around it, for the reason
    // `SwitchListTile` used to enforce here via its own `MergeSemantics`: a wrapper outside the
    // tappable node becomes a separate parent with no name of its own
    // (`labeledTapTargetGuideline` catches exactly that).
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Semantics(
            label: pantryRowLabel(entry.name, entry.marked),
            button: true,
            child: ExcludeSemantics(
              child: InkWell(
                onTap: busy ? null : () => _toggleMark(householdId, entry),
                child: Row(
                  children: [
                    Icon(
                      entry.marked
                          ? Icons.check_box
                          : Icons.check_box_outline_blank,
                    ),
                    const SizedBox(width: 4),
                    const Text('Have'),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Text(
                        entry.name,
                        style: Theme.of(context).textTheme.bodyLarge,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
          if (entry.ingredient is IngredientRefDto_Custom)
            const Padding(
              padding: EdgeInsets.only(left: 4),
              child: Text('Your ingredient', style: TextStyle(fontSize: 12)),
            ),
          const SizedBox(height: 4),
          Padding(
            padding: const EdgeInsets.only(left: 4),
            child: Wrap(
              spacing: 8,
              children: [
                _ShelfChip(
                  label: pantryLowChipLabel,
                  semanticLabel: pantryRestockChipLabel(
                    entry.name,
                    entry.restockRequested,
                  ),
                  selected: entry.restockRequested,
                  onPressed: busy
                      ? null
                      : () => _toggleRestock(householdId, entry),
                ),
                _ShelfChip(
                  label: pantryUsedUpChipLabel,
                  semanticLabel: pantryUsedUpLabel(entry.name),
                  selected: false,
                  onPressed: busy ? null : () => _usedUp(householdId, entry),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// A small shelf-sticker/tag-styled action chip (OPT-006 gate 3) — scoped to this screen, since
/// no other screen needs this look; a shared `ChipThemeData` would be speculative machinery for
/// one caller.
class _ShelfChip extends StatelessWidget {
  const _ShelfChip({
    required this.label,
    required this.semanticLabel,
    required this.selected,
    required this.onPressed,
  });

  final String label;
  final String semanticLabel;
  final bool selected;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Semantics(
      label: semanticLabel,
      button: true,
      child: ExcludeSemantics(
        child: ActionChip(
          label: Text(label),
          onPressed: onPressed,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(6),
            side: BorderSide(
              color: selected ? scheme.primary : scheme.outlineVariant,
            ),
          ),
          backgroundColor: selected ? scheme.primaryContainer : null,
          labelStyle: selected
              ? TextStyle(
                  color: scheme.onPrimaryContainer,
                  fontWeight: FontWeight.w600,
                )
              : null,
        ),
      ),
    );
  }
}

class _CategorizeBanner extends StatelessWidget {
  const _CategorizeBanner({required this.count, required this.onTap});

  final int count;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Material(
        color: Theme.of(context).colorScheme.secondaryContainer,
        borderRadius: BorderRadius.circular(8),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(8),
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                const Icon(Icons.shelves),
                const SizedBox(width: 8),
                Expanded(child: Text(pantryNeedsCategoryCopy(count))),
                const Icon(Icons.chevron_right),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
