import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/placeholder_screen.dart';
import 'package:meal_mate/app/shell.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/onboarding/welcome_screen.dart';
import 'package:meal_mate/features/pantry/pantry_screen.dart';
import 'package:meal_mate/features/planning/cover_screen.dart';
import 'package:meal_mate/features/planning/cycle_editor_screen.dart';
import 'package:meal_mate/features/planning/planner_screen.dart';
import 'package:meal_mate/features/recipes/recipe_detail_screen.dart';
import 'package:meal_mate/features/recipes/recipe_form_screen.dart';
import 'package:meal_mate/features/recipes/recipe_list_screen.dart';
import 'package:meal_mate/features/restrictions/restrictions_screen.dart';
import 'package:meal_mate/features/settings/settings_screen.dart';
import 'package:meal_mate/features/shopping/shopping_screen.dart';

/// Plan is home: the product's default state is "this week is covered" (PRD v3 §15).
const homeLocation = '/plan';

/// First run. Outside the shell, so no navigation bar is shown while it is up.
const welcomeLocation = '/welcome';

/// The `?offset=` query value as a cycle offset. `offset` crosses to Rust as an i32 and
/// `sse_encode_i_32` narrows with `putInt32`, which keeps the low 32 bits and does not throw —
/// so an unsaturated 64-bit value arrives as an unrelated in-range window instead of being
/// refused, and `?offset=4294967297` would quietly preview next cycle. Saturating here hands an
/// out-of-range *intent* to the ±520 bound, which answers in prose; the bound is deliberately
/// not applied here, so an in-range value past it is still refused rather than silently moved.
/// An unreadable value is the active cycle, as it already was.
int coverOffset(String? raw) {
  final parsed = int.tryParse(raw ?? '');
  if (parsed == null) return 0;
  return parsed.clamp(-2147483648, 2147483647);
}

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
              builder: (_, _) => const RecipeListScreen(),
              // Literal segments before `:id`, so `new` and `archived` are never read as
              // recipe ids.
              routes: [
                GoRoute(
                  path: 'new',
                  builder: (_, _) => const RecipeFormScreen(),
                ),
                GoRoute(
                  path: 'archived',
                  builder: (_, _) => const ArchivedRecipesScreen(),
                ),
                GoRoute(
                  path: ':id',
                  builder: (_, state) =>
                      RecipeDetailScreen(recipeId: state.pathParameters['id']!),
                  routes: [
                    GoRoute(
                      path: 'edit',
                      builder: (_, state) => RecipeFormScreen(
                        recipeId: state.pathParameters['id'],
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ],
        ),
        StatefulShellBranch(
          restorationScopeId: 'plan',
          routes: [
            GoRoute(
              path: '/plan',
              builder: (_, _) => const PlannerScreen(),
              routes: [
                GoRoute(
                  path: 'cover',
                  builder: (context, state) => CoverScreen(
                    offset: coverOffset(state.uri.queryParameters['offset']),
                  ),
                ),
              ],
            ),
          ],
        ),
        StatefulShellBranch(
          restorationScopeId: 'pantry',
          routes: [
            GoRoute(path: '/pantry', builder: (_, _) => const PantryScreen()),
          ],
        ),
        StatefulShellBranch(
          restorationScopeId: 'shopping',
          routes: [
            GoRoute(
              path: '/shopping',
              builder: (_, _) => const ShoppingScreen(),
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
