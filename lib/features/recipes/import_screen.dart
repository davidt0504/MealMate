import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/share_channel.dart';
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/recipes/import_controller.dart';
import 'package:meal_mate/features/recipes/recipe_form_screen.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/recipe_import.dart';

/// "Paste link" (OPT-001): one URL field, then the fetch, then — when the page needs an
/// answer first — the recipe picker or the already-saved question. A share lands here too.
class ImportScreen extends ConsumerStatefulWidget {
  const ImportScreen({super.key});

  @override
  ConsumerState<ImportScreen> createState() => _ImportScreenState();
}

class _ImportScreenState extends ConsumerState<ImportScreen> {
  final _url = TextEditingController();

  /// The picked recipe on a page with several; `null` until the user picks.
  int? _chosen;

  @override
  void initState() {
    super.initState();
    _prefillFromClipboard();
  }

  /// Only a clipboard that holds an http(s) link prefills the field (design §2).
  Future<void> _prefillFromClipboard() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    final link = firstUrl(data?.text ?? '');
    if (mounted && link != null && _url.text.isEmpty) {
      setState(() => _url.text = link);
    }
  }

  @override
  void dispose() {
    _url.dispose();
    super.dispose();
  }

  ImportController get _controller =>
      ref.read(importControllerProvider.notifier);

  void _start(String url) {
    setState(() => _chosen = null);
    _controller.start(url.trim());
  }

  @override
  Widget build(BuildContext context) {
    ref.listen(importControllerProvider, (_, next) {
      if (next is ImportReviewing) context.go('/recipes/import/review');
    });
    final state = ref.watch(importControllerProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Import a recipe')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: switch (state) {
          ImportIdle() || ImportReviewing() => _entry(),
          ImportFetching() => [
            const Center(child: CircularProgressIndicator()),
            const SizedBox(height: 16),
            const Text('Reading the page…', textAlign: TextAlign.center),
            TextButton(
              onPressed: _controller.reset,
              child: const Text('Cancel'),
            ),
          ],
          ImportFetched(:final result) => _fetched(result),
          ImportFailed() => _failed(state),
        },
      ),
    );
  }

  List<Widget> _entry() => [
    TextField(
      controller: _url,
      keyboardType: TextInputType.url,
      decoration: const InputDecoration(labelText: 'Recipe link'),
    ),
    const SizedBox(height: 16),
    ListenableBuilder(
      listenable: _url,
      builder: (context, _) => FilledButton(
        onPressed: _url.text.trim().isEmpty ? null : () => _start(_url.text),
        child: const Text('Import'),
      ),
    ),
  ];

  List<Widget> _fetched(ImportResultDto result) {
    if (result.drafts.length > 1 && _chosen == null) {
      return [
        Text(
          'This page has ${result.drafts.length} recipes. Which one?',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        for (final (i, d) in result.drafts.indexed)
          ListTile(
            title: Text(d.recipe.title),
            onTap: () => setState(() => _chosen = i),
          ),
      ];
    }
    final draft = result.drafts[_chosen ?? 0];
    final existing = result.existing;
    if (existing == null) {
      // Nothing to ask: straight into the review.
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted && ref.read(importControllerProvider) is ImportFetched) {
          _controller.review(draft);
        }
      });
      return [const Center(child: CircularProgressIndicator())];
    }
    return [
      Text(
        'You already saved this recipe',
        style: Theme.of(context).textTheme.titleMedium,
      ),
      const SizedBox(height: 16),
      FilledButton(
        onPressed: () => _openExisting(existing),
        child: Text(existing.archived ? 'Restore it' : 'Open'),
      ),
      TextButton(
        onPressed: () => _controller.review(draft),
        child: const Text('Import another copy'),
      ),
    ];
  }

  Future<void> _openExisting(ExistingRecipeDto existing) async {
    try {
      if (existing.archived) {
        final household = await ref.read(householdProvider.future);
        await ref
            .read(recipeLibraryProvider.notifier)
            .restore(household.id, existing.id);
      }
      _controller.reset();
      if (mounted) context.go('/recipes/${existing.id}');
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Recipes'))),
        );
      }
    }
  }

  List<Widget> _failed(ImportFailed failed) => [
    Text(importFailureMessage(failed.kind)),
    const SizedBox(height: 16),
    if (importCanRetry(failed.kind))
      FilledButton(
        onPressed: () => _start(failed.url),
        child: const Text('Retry'),
      ),
    if (failed.kind == ImportErrorKind.noRecipe)
      FilledButton(
        onPressed: () async {
          final household = await ref.read(householdProvider.future);
          _controller.reviewByHand(
            household.id,
            failed.sourceUrl ?? failed.url,
          );
        },
        child: const Text('You can still add it by hand'),
      ),
    TextButton(
      onPressed: _controller.reset,
      child: const Text('Try another link'),
    ),
  ];
}

/// The review: the recipe form on the draft that was in [importControllerProvider] when it
/// opened. Captured once, so a later change to the import (a new share being fetched) never
/// swaps the form out from under the user. With no draft — the route restored after the
/// process died, which loses the draft by design — it goes back to the link field rather
/// than showing an empty form.
class ImportReviewScreen extends ConsumerStatefulWidget {
  const ImportReviewScreen({super.key});

  @override
  ConsumerState<ImportReviewScreen> createState() => _ImportReviewScreenState();
}

class _ImportReviewScreenState extends ConsumerState<ImportReviewScreen> {
  late final ImportDraftDto? _draft = switch (ref.read(
    importControllerProvider,
  )) {
    ImportReviewing(:final draft) => draft,
    _ => null,
  };

  @override
  void initState() {
    super.initState();
    if (_draft == null) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) context.go('/recipes/import');
      });
    }
  }

  @override
  Widget build(BuildContext context) => switch (_draft) {
    final draft? => RecipeFormScreen(draft: draft),
    null => const Scaffold(body: Center(child: CircularProgressIndicator())),
  };
}
