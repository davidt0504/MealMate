import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/router.dart';
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/planning/planning_cycle.dart';
import 'package:meal_mate/features/planning/planning_provider.dart';

/// First run: one screen, no input fields, nothing to answer. It says what the app has
/// already assumed and offers two ways forward, both of which mark onboarding complete
/// (PRD v3 §15, AC-1/AC-3). Stateful rather than a `ConsumerWidget` because `_finish` awaits
/// the bridge and then touches `context`, which `use_build_context_synchronously` — part of
/// `flutter_lints` and so part of the `flutter analyze` gate — requires a `mounted` guard for.
class WelcomeScreen extends ConsumerStatefulWidget {
  const WelcomeScreen({super.key});

  @override
  ConsumerState<WelcomeScreen> createState() => _WelcomeScreenState();
}

class _WelcomeScreenState extends ConsumerState<WelcomeScreen> {
  bool _finishing = false;

  Future<void> _finish(String destination) async {
    setState(() => _finishing = true);
    // Read the notifier and the id before the await, as `HouseholdScreen._save` does: the
    // last `ref` use on this widget, so a disposal mid-write cannot reach a disposed ref.
    final notifier = ref.read(householdProvider.notifier);
    final id = ref.read(householdProvider).valueOrNull?.id;
    String? failure;
    try {
      if (id != null) {
        await notifier.completeOnboarding(id);
      } else {
        // Both buttons wait for the household to resolve, so this is the error arm. Saying
        // so is the point: skipping the call silently would navigate on with `onboarded`
        // still 0, and Welcome would simply return next launch with nothing to explain it.
        failure = 'Setup unavailable: the household did not load';
      }
    } catch (e) {
      failure = describeFailure(e, subject: 'Setup');
    }
    if (!mounted) return;
    // Navigate either way. A database failure must not trap the user on a screen they
    // cannot leave — AC-1 says skip is always available, and Settings is where the failure
    // is explained.
    if (failure != null) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(failure)));
    }
    context.go(destination);
  }

  @override
  Widget build(BuildContext context) {
    final cycle = ref.watch(planningCycleProvider);
    // `_finish` needs the household's id, so both buttons wait for it — the same gate
    // `restrictions_screen.dart` and `cycle_editor_screen.dart` put on Save. An error counts
    // as resolved: Welcome sits outside the shell, so buttons left dead on the error arm
    // would strand the user on a screen with no navigation bar, and AC-1 keeps the skip path
    // always available. The error arm is where `_finish`'s null-id branch reports.
    final household = ref.watch(householdProvider);
    final ready = household.hasValue || household.hasError;
    return Scaffold(
      appBar: AppBar(title: const Text('Welcome')),
      body: ListView(
        padding: const EdgeInsets.all(24),
        children: [
          const Text(
            'Meal Mate plans for a household. Yours is a household of one until '
            'you say otherwise.',
          ),
          const SizedBox(height: 16),
          Text(switch (cycle) {
            AsyncData(:final value) =>
              'Already set up for you: ${describePlanningCycle(value)}.',
            AsyncError(:final error) => describeFailure(
              error,
              subject: 'Planning cycle',
            ),
            _ => 'Loading planning cycle…',
          }),
          const SizedBox(height: 16),
          const Text(
            'Everything here is optional, and everything can be changed later in '
            'Settings.',
          ),
          const SizedBox(height: 24),
          FilledButton(
            onPressed: _finishing || !ready
                ? null
                : () => _finish(homeLocation),
            child: const Text('Get started'),
          ),
          const SizedBox(height: 8),
          TextButton(
            onPressed: _finishing || !ready ? null : () => _finish('/settings'),
            child: const Text('Set up first'),
          ),
        ],
      ),
    );
  }
}
