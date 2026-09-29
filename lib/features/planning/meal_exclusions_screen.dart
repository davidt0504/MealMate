import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
import 'package:meal_mate/features/planning/cover_copy.dart';
import 'package:meal_mate/features/planning/cover_provider.dart';
import 'package:meal_mate/src/rust/api/planning_drafts.dart';

/// Settings › Meal exclusions (OPT-007 §8): every standing "never suggest" rule, with Remove.
/// Identity rules and the older word rules are listed apart because they reach differently:
/// the older ones skip any meal whose name or ingredients contain the words, and say so.
/// Removing a rule makes the meal eligible again; nothing already planned changes.
class MealExclusionsScreen extends ConsumerStatefulWidget {
  const MealExclusionsScreen({super.key});

  @override
  ConsumerState<MealExclusionsScreen> createState() =>
      _MealExclusionsScreenState();
}

class _MealExclusionsScreenState extends ConsumerState<MealExclusionsScreen> {
  String? _removing;

  @override
  Widget build(BuildContext context) {
    final rules = ref.watch(mealExclusionsProvider);
    return Scaffold(
      appBar: AppBar(title: const Text(exclusionsTitle)),
      body: switch (rules) {
        AsyncData(value: final list) when list.isEmpty => const Padding(
          padding: EdgeInsets.all(16),
          child: Text(exclusionsEmptyCopy),
        ),
        AsyncData(:final value) => _list(value),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(describeFailure(error, subject: exclusionsTitle)),
              TextButton(
                onPressed: () => ref.invalidate(mealExclusionsProvider),
                child: const Text('Try again'),
              ),
            ],
          ),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Widget _list(List<MealExclusionDto> rules) {
    final dishes = [
      for (final r in rules)
        if (r.kind is MealExclusionKindDto_Dish) r,
    ];
    final phrases = [
      for (final r in rules)
        if (r.kind is MealExclusionKindDto_Phrase) r,
    ];
    final heading = Theme.of(context).textTheme.titleSmall;
    return ListView(
      padding: const EdgeInsets.symmetric(vertical: 8),
      children: [
        if (dishes.isNotEmpty)
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
            child: Text(exclusionsDishHeading, style: heading),
          ),
        for (final r in dishes) _tile(r),
        if (phrases.isNotEmpty) ...[
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
            child: Text(exclusionsPhraseHeading, style: heading),
          ),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16),
            child: Text(exclusionsPhraseScopeCopy),
          ),
        ],
        for (final r in phrases) _tile(r),
      ],
    );
  }

  Widget _tile(MealExclusionDto rule) {
    final (title, subtitle) = switch (rule.kind) {
      MealExclusionKindDto_Dish(:final title, :final available) => (
        title ?? 'A meal',
        available ? null : exclusionsUnavailableCopy,
      ),
      MealExclusionKindDto_Phrase(:final subject) => ('“$subject”', null),
    };
    return ListTile(
      key: ValueKey('exclusion:${rule.policyId}'),
      title: Text(title),
      subtitle: subtitle == null ? null : Text(subtitle),
      trailing: TextButton(
        onPressed: _removing != null ? null : () => _remove(rule.policyId),
        child: Text('Remove', semanticsLabel: 'Remove $title'),
      ),
    );
  }

  Future<void> _remove(String policyId) async {
    setState(() => _removing = policyId);
    final messenger = ScaffoldMessenger.of(context);
    try {
      await ref.read(mealExclusionsProvider.notifier).remove(policyId);
      messenger.showSnackBar(
        const SnackBar(content: Text(exclusionsRemovedCopy)),
      );
    } catch (e) {
      messenger.showSnackBar(
        SnackBar(content: Text(describeFailure(e, subject: exclusionsTitle))),
      );
    } finally {
      if (mounted) setState(() => _removing = null);
    }
  }
}
