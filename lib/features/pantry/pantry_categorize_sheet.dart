import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
import 'package:meal_mate/features/pantry/pantry_category_field.dart';
import 'package:meal_mate/features/pantry/pantry_copy.dart';
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/src/rust/api/pantry.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

/// OPT-006 gate 5 remediation: lets a household assign a shelf to each pre-existing,
/// have-marked custom ingredient whose `store_category` is still `NULL`. Opened from the "My
/// Shelves" screen's categorize banner.
///
/// Scoped to have-marked rows only, matching the banner's own count exactly — `entries` is the
/// same `needsCategory` list the banner counted, passed straight through rather than re-queried,
/// so the two can never disagree about how many items there are. An unmarked custom ingredient
/// missing a category is not blocking anything visible yet, so it waits until the household
/// marks it "have" (at which point it enters this same list).
class CategorizeMissingIngredientsSheet extends ConsumerStatefulWidget {
  const CategorizeMissingIngredientsSheet({
    super.key,
    required this.householdId,
    required this.entries,
  });

  final String householdId;
  final List<PantryEntryDto> entries;

  @override
  ConsumerState<CategorizeMissingIngredientsSheet> createState() =>
      _CategorizeMissingIngredientsSheetState();
}

class _CategorizeMissingIngredientsSheetState
    extends ConsumerState<CategorizeMissingIngredientsSheet> {
  late List<PantryEntryDto> _remaining = widget.entries;
  final Set<String> _writing = {};

  static String _customId(PantryEntryDto entry) =>
      (entry.ingredient as IngredientRefDto_Custom).id;

  Future<void> _categorize(PantryEntryDto entry, String category) async {
    final id = _customId(entry);
    setState(() => _writing.add(id));
    try {
      await ref
          .read(pantryProvider.notifier)
          .categorize(widget.householdId, id, category);
      if (mounted) {
        setState(
          () => _remaining = [
            for (final e in _remaining)
              if (e.ingredient != entry.ingredient) e,
          ],
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Pantry'))),
        );
      }
    } finally {
      if (mounted) setState(() => _writing.remove(id));
    }
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              pantryCategorizeHeading,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            if (_remaining.isEmpty)
              const Text('All set — nothing left to categorize.')
            else
              for (final entry in _remaining) _Row(
                entry: entry,
                busy: _writing.contains(_customId(entry)),
                onSave: (category) => _categorize(entry, category),
              ),
          ],
        ),
      ),
    );
  }
}

class _Row extends StatefulWidget {
  const _Row({required this.entry, required this.busy, required this.onSave});

  final PantryEntryDto entry;
  final bool busy;
  final ValueChanged<String> onSave;

  @override
  State<_Row> createState() => _RowState();
}

class _RowState extends State<_Row> {
  String? _category;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          Expanded(child: Text(widget.entry.name)),
          const SizedBox(width: 8),
          SizedBox(
            width: 170,
            child: CategoryDropdownField(
              value: _category,
              onChanged: (value) => setState(() => _category = value),
            ),
          ),
          const SizedBox(width: 8),
          FilledButton(
            onPressed: widget.busy || _category == null
                ? null
                : () => widget.onSave(_category!),
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }
}
