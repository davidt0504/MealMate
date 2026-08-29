import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/placeholder_screen.dart';
import 'package:meal_mate/app/shell.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/onboarding/welcome_screen.dart';
import 'package:meal_mate/features/planning/cycle_editor_screen.dart';
import 'package:meal_mate/features/restrictions/restrictions_screen.dart';
import 'package:meal_mate/features/settings/settings_screen.dart';

/// Plan is home: the product's default state is "this week is covered" (PRD v3 §15).
const homeLocation = '/plan';

/// First run. Outside the shell, so no navigation bar is shown while it is up.
const welcomeLocation = '/welcome';

GoRouter buildRouter({String initialLocation = homeLocation}) => GoRouter(
  initialLocation: initialLocation,
  restorationScopeId: 'router',
  errorBuilder: (context, state) => _NotFoundScreen(uri: state.uri),
  routes: [
    GoRoute(path: welcomeLocation, builder: (_, _) => const WelcomeScreen()),
    StatefulShellRoute.indexedStack(
      restorationScopeId: 'shell',
      builder: (context, state, shell) => AppShell(navigationShell: shell),
      branches: [
        StatefulShellBranch(
          restorationScopeId: 'recipes',
          routes: [
            GoRoute(
              path: '/recipes',
              builder: (_, _) => const PlaceholderScreen(
                title: 'Recipes',
                message: 'Your recipe and meal library arrives with MVP-008.',
              ),
            ),
          ],
        ),
        StatefulShellBranch(
          restorationScopeId: 'plan',
          routes: [
            GoRoute(
              path: '/plan',
              builder: (context, _) => PlaceholderScreen(
                title: 'Plan',
                message: 'Manual planning arrives with MVP-013.',
                action: FilledButton(
                  onPressed: () => context.go('/plan/cover'),
                  child: const Text('Cover My Week'),
                ),
              ),
              routes: [
                GoRoute(
                  path: 'cover',
                  builder: (_, _) => const PlaceholderScreen(
                    title: 'Cover My Week',
                    message:
                        'The planner that fills your week arrives with MVP-024. '
                        'Nothing is planned yet.',
                  ),
                ),
              ],
            ),
          ],
        ),
        StatefulShellBranch(
          restorationScopeId: 'pantry',
          routes: [
            GoRoute(
              path: '/pantry',
              builder: (_, _) => const PlaceholderScreen(
                title: 'Pantry',
                message:
                    "Optional have/don't-have pantry arrives with MVP-014.",
              ),
            ),
          ],
        ),
        StatefulShellBranch(
          restorationScopeId: 'shopping',
          routes: [
            GoRoute(
              path: '/shopping',
              builder: (_, _) => const PlaceholderScreen(
                title: 'Shopping',
                message: 'Your derived shopping list arrives with MVP-016.',
              ),
            ),
          ],
        ),
        StatefulShellBranch(
          restorationScopeId: 'settings',
          routes: [
            GoRoute(
              path: '/settings',
              builder: (_, _) => const SettingsScreen(),
              routes: [
                GoRoute(
                  path: 'household',
                  builder: (_, _) => const HouseholdScreen(),
                ),
                GoRoute(
                  path: 'cycle',
                  builder: (_, _) => const CycleEditorScreen(),
                ),
                GoRoute(
                  path: 'restrictions',
                  builder: (_, _) => const RestrictionsScreen(),
                ),
              ],
            ),
          ],
        ),
      ],
    ),
  ],
);

class _NotFoundScreen extends StatelessWidget {
  const _NotFoundScreen({required this.uri});

  final Uri uri;

  @override
  Widget build(BuildContext context) {
    return PlaceholderScreen(
      title: 'Page not found',
      message: 'No screen at $uri.',
      action: FilledButton(
        onPressed: () => context.go(homeLocation),
        child: const Text('Go to Plan'),
      ),
    );
  }
}
