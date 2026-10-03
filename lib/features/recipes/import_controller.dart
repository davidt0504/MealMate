import 'dart:typed_data';

import 'package:flutter/foundation.dart' show protected;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/recipe_import.dart';

/// Where an import from a link stands (OPT-001). Paste and Android Share both enter through
/// [ImportController.start]; nothing is saved until the review's Save.
sealed class ImportState {
  const ImportState();
}

final class ImportIdle extends ImportState {
  const ImportIdle();
}

final class ImportFetching extends ImportState {
  const ImportFetching(this.url);
  final String url;
}

/// Fetched: the user may still pick a recipe or answer the duplicate question.
final class ImportFetched extends ImportState {
  const ImportFetched(this.url, this.result);
  final String url;
  final ImportResultDto result;
}

/// The draft open in the review form.
final class ImportReviewing extends ImportState {
  const ImportReviewing(this.draft);
  final ImportDraftDto draft;
}

final class ImportFailed extends ImportState {
  const ImportFailed(this.url, this.kind, {this.sourceUrl, this.error});
  final String url;

  /// `null` when the failure was not an import refusal (the bridge or database failed).
  final ImportErrorKind? kind;

  /// The page's normalized URL, set for [ImportErrorKind.noRecipe] only.
  final String? sourceUrl;
  final Object? error;
}

/// The message design §8 gives each failure.
String importFailureMessage(ImportErrorKind? kind) => switch (kind) {
  ImportErrorKind.offline => "You're offline or the site can't be reached.",
  ImportErrorKind.timeout => 'The site took too long.',
  ImportErrorKind.refused => "This site didn't let Kimatta read the page.",
  ImportErrorKind.blocked => "That link can't be imported.",
  ImportErrorKind.unreadable ||
  ImportErrorKind.noRecipe => "This page isn't a recipe Kimatta can read.",
  null => "Something went wrong reading that link.",
};

/// Only offline and slow sites are worth retrying; a refusal never is (design §3).
bool importCanRetry(ImportErrorKind? kind) =>
    kind == ImportErrorKind.offline || kind == ImportErrorKind.timeout;

class ImportController extends Notifier<ImportState> {
  /// Bumped by every start and cancel, so a fetch that finishes after the user moved on is
  /// dropped instead of replacing what they are now looking at.
  ///
  /// ponytail: the superseded Rust fetch is not cancelled; it ends by its own deadline.
  int _generation = 0;

  @override
  ImportState build() => const ImportIdle();

  /// The bridge call behind an overridable seam, as `RecipeLibraryNotifier`'s are.
  @protected
  Future<ImportResultDto> fetch(String householdId, String url) =>
      importRecipeFromUrl(householdId: householdId, url: url);

  Future<void> start(String url) async {
    final generation = ++_generation;
    state = ImportFetching(url);
    ImportState next;
    try {
      final household = await ref.read(householdProvider.future);
      next = ImportFetched(url, await fetch(household.id, url));
    } on KimattaError_Import catch (e) {
      next = ImportFailed(url, e.kind, sourceUrl: e.url);
    } catch (e) {
      next = ImportFailed(url, null, error: e);
    }
    if (generation == _generation) state = next;
  }

  /// Opens [draft] in the review.
  void review(ImportDraftDto draft) {
    _generation++;
    state = ImportReviewing(draft);
  }

  /// The "add it by hand" path for a page with no recipe data: an empty review that keeps
  /// where the recipe came from.
  void reviewByHand(String householdId, String sourceUrl) => review(
    ImportDraftDto(
      recipe: RecipeDto(
        id: '',
        householdId: householdId,
        title: '',
        instructions: '',
        lines: const [],
        provenance: RecipeProvenanceDto(kind: 'imported', sourceUrl: sourceUrl),
        components: const [],
      ),
      inlineLines: Uint32List(0),
    ),
  );

  /// Abandons whatever is in flight or in review; nothing was saved.
  void reset() {
    _generation++;
    state = const ImportIdle();
  }
}

final importControllerProvider =
    NotifierProvider<ImportController, ImportState>(ImportController.new);
