import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
import 'package:meal_mate/features/pantry/pantry_category_field.dart';
import 'package:meal_mate/features/pantry/pantry_provider.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';

/// Creates a household's own ingredient when the catalog has no match (OPT-006 gate 5) —
/// `store_category` is required so every custom item lands on a real shelf; there is no
/// "Other"/uncategorised fallback. Does not auto-mark the new item "have" — invariant 6's
/// default is no record, not a record of possession.
class CustomIngredientFormScreen extends ConsumerStatefulWidget {
  const CustomIngredientFormScreen({
    super.key,
    required this.householdId,
    this.initialName = '',
  });

  final String householdId;
  final String initialName;

  @override
  ConsumerState<CustomIngredientFormScreen> createState() =>
      _CustomIngredientFormScreenState();
}

class _CustomIngredientFormScreenState
    extends ConsumerState<CustomIngredientFormScreen> {
  late final _name = TextEditingController(text: widget.initialName);
  String? _category;
  String? _nameError;
  bool _saving = false;

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    final name = _name.text.trim();
    if (name.isEmpty) {
      setState(() => _nameError = 'Name is required');
      return;
    }
    final category = _category;
    if (category == null) return;
    setState(() {
      _nameError = null;
      _saving = true;
    });
    try {
      await addCustomIngredient(
        item: CustomIngredientDto(
          id: '',
          householdId: widget.householdId,
          name: name,
          storeCategory: category,
        ),
      );
      await ref.read(pantryProvider.notifier).refresh();
      if (mounted) Navigator.of(context).pop();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Pantry'))),
        );
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Add ingredient')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            TextField(
              controller: _name,
              autofocus: true,
              decoration: InputDecoration(
                labelText: 'Name',
                errorText: _nameError,
              ),
            ),
            const SizedBox(height: 16),
            CategoryDropdownField(
              value: _category,
              onChanged: (value) => setState(() => _category = value),
            ),
            const SizedBox(height: 24),
            FilledButton(
              onPressed: _saving ? null : _save,
              child: const Text('Save'),
            ),
          ],
        ),
      ),
    );
  }
}
