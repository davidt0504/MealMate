import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/pantry/pantry_copy.dart';
import 'package:meal_mate/features/pantry/pantry_provider.dart';

/// The store-category dropdown shared by custom-ingredient creation
/// (`CustomIngredientFormScreen`) and the categorization-remediation flow
/// (`CategorizeMissingIngredientsSheet`) — one grouping code path, per OPT-006 gate 5 ("no
/// Other/uncategorised fallback bucket anywhere on My Shelves").
class CategoryDropdownField extends ConsumerWidget {
  const CategoryDropdownField({
    super.key,
    required this.value,
    required this.onChanged,
    this.errorText,
  });

  final String? value;
  final ValueChanged<String?> onChanged;
  final String? errorText;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final categories = ref.watch(knownStoreCategoriesProvider);
    return switch (categories) {
      AsyncData(value: final known) => DropdownButtonFormField<String>(
        initialValue: value,
        decoration: InputDecoration(
          labelText: pantryCategoryFieldLabel,
          errorText: errorText,
        ),
        items: [
          for (final category in known)
            DropdownMenuItem(value: category, child: Text(category)),
        ],
        onChanged: onChanged,
      ),
      AsyncError() => InputDecorator(
        decoration: InputDecoration(
          labelText: pantryCategoryFieldLabel,
          errorText: 'Categories unavailable — try again',
        ),
        child: const SizedBox.shrink(),
      ),
      _ => const LinearProgressIndicator(),
    };
  }
}
