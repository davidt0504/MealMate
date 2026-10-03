import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:meal_mate/app/share_channel.dart';
import 'package:meal_mate/features/household/household_provider.dart';
import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/recipes/recipe_fields.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/recipe_import.dart';

/// Create (`recipeId == null`) or edit. Ephemeral form state lives here (PRD §13); the saved
/// recipe is Rust's. Nothing is trimmed or normalised on the way out — the user's text goes
/// verbatim and Rust reports what it rejects — because silent normalisation is this card's
/// stop condition. The one string nobody types is a line's `original_text`:
/// `_LineDraft.savesAs` composes it from the row's fields unless a loaded row is left as it was
/// (D-045). Ingredient identity (`ingredient`) is not *chosen* by this card; MVP-009/011 add
/// matching on top. A seeded line's existing ref rides through the edit
/// untouched — a field the form does not render is a field it has no business erasing — right
/// up until the user rewrites the row's name, at which point it is dropped rather than left to
/// speak for a food the line no longer names. See `_LineDraft.emittedIngredient`.
///
/// With a [draft] (OPT-001) it is the import review: seeded from the unsaved draft, saved as
/// a new recipe, its sub-recipe components grouped under their own headings.
class RecipeFormScreen extends ConsumerStatefulWidget {
  const RecipeFormScreen({super.key, this.recipeId, this.draft});

  final String? recipeId;
  final ImportDraftDto? draft;

  @override
  ConsumerState<RecipeFormScreen> createState() => _RecipeFormScreenState();
}

/// One ingredient row's controllers. Owned by the screen state so a rebuild never resets a
/// half-typed row.
class _LineDraft {
  _LineDraft([IngredientLineDto? from, bool inline = false])
    : name = TextEditingController(text: from?.name ?? ''),
      amount = TextEditingController(text: _amountText(from?.quantity)),
      preparation = TextEditingController(text: from?.preparation ?? ''),
      otherUnit = TextEditingController(
        text: switch (from?.unit) {
          UnitDto_Other(:final text) => text,
          _ => '',
        },
      ),
      unitKey = switch (from?.unit) {
        UnitDto_Known(:final unit) => unit,
        UnitDto_Other() => unitOtherKey,
        _ => unitNoneKey,
      },
      optional = from?.optional ?? false,
      ingredient = from?.ingredient,
      component = from?.component,
      inlineHint = inline,
      seededName = from?.name,
      seededOriginal = from?.originalText {
    _seededFields = from == null ? null : _fields;
  }

  final TextEditingController name;
  final TextEditingController amount;
  final TextEditingController preparation;
  final TextEditingController otherUnit;
  String unitKey;
  bool optional;

  /// The line's catalog/custom identity as it was loaded. No control renders or changes it
  /// (MVP-009/011 own matching), and that is exactly the hazard: FRB emits it as an *optional*
  /// named parameter, so an emit that omits it compiles and silently re-saves the line with no
  /// identity — the storage write replaces every line row, so the null is committed. `null` on
  /// a row the user added, which is the truth for a new line. Read through
  /// [emittedIngredient], never directly.
  final IngredientRefDto? ingredient;

  /// The imported sub-recipe this row belongs to (OPT-001), `null` for the main recipe. Like
  /// [ingredient], FRB emits it as an optional parameter, so an emit that forgot it would
  /// silently strip every component boundary on save. No control changes it.
  final int? component;

  /// The imported line names a sub-recipe whose ingredients are listed inline below it.
  final bool inlineHint;

  /// Where the review's notice scrolls to.
  final GlobalKey rowKey = GlobalKey();

  /// The `name` this row arrived with, so [emittedIngredient] can tell an untouched row from a
  /// renamed one. `null` on a row the user added.
  final String? seededName;

  /// The ref to save for this row: the loaded one while the name it was matched under still
  /// stands, and `null` once the user has rewritten that name.
  ///
  /// Nothing here re-matches — the ref is only ever kept or dropped, because deciding what a
  /// renamed line now *is* belongs to MVP-009/011's explicit mapping, not to a form with no
  /// control for it. The asymmetry is deliberate, and it is about which way each choice fails.
  /// Keeping a stale ref is the unsafe direction: `Identities::status` matches on the ref
  /// alone and never consults the name, so a line renamed away from a pantry-marked ingredient
  /// would inherit that mark and vanish from the shopping list, and it could merge quantities
  /// with a food it no longer names. Dropping it costs only precision — the line goes back to
  /// `Unresolved`, so it lists separately, uncategorised, and always as `Needed`. That
  /// over-lists; it never silently omits.
  ///
  /// Compared verbatim, with no `trim`, for the reason the whole form sends text verbatim: a
  /// name the user altered at all is a name this card cannot vouch for.
  IngredientRefDto? get emittedIngredient =>
      name.text == seededName ? ingredient : null;

  /// The `original_text` this row arrived with. `null` on a row the user added.
  final String? seededOriginal;

  /// Every field [composeLine] reads, as loaded — `null` on a row the user added. Captured in
  /// the constructor, not lazily, so it can never pick up an edit.
  late final (String, String, String, String, String)? _seededFields;

  (String, String, String, String, String) get _fields =>
      (name.text, amount.text, unitKey, otherUnit.text, preparation.text);

  /// What this row saves as `original_text` (D-045). A loaded row whose fields all still equal
  /// what was loaded keeps its stored text verbatim — starter and imported wording included.
  /// Any other row is a new entry and composes it, so recipe detail never states an amount the
  /// structured line contradicts. Compared verbatim, like [emittedIngredient].
  String get savesAs {
    final original = seededOriginal;
    return original != null && _fields == _seededFields
        ? original
        : composeLine(
            amount: amount.text,
            unitKey: unitKey,
            otherUnit: otherUnit.text,
            name: name.text,
            preparation: preparation.text,
          );
  }

  /// What is wrong with this row, *without* the `Ingredient N:` prefix — `_row` applies the
  /// number at paint time from the live index. Baking it in here would outlive its own truth:
  /// removing a row renumbers every row below it, so the surviving card would head
  /// `Ingredient 1` and read `Ingredient 2: …`.
  String? error;

  /// Anchors `_revealFirstError`'s scroll on the error text itself rather than on the row's
  /// `Card`: at text scale 2.0 the card is taller than the viewport, so scrolling its top
  /// into view leaves the error — the card's last child — below the fold.
  final GlobalKey errorKey = GlobalKey();

  void dispose() {
    for (final c in [name, amount, preparation, otherUnit]) {
      c.dispose();
    }
  }

  /// The stored rational, re-rendered so an edit shows what is stored (`1/2`, `2-3`), not
  /// what was typed — the amount is structured data, unlike `original_text`.
  static String _amountText(QuantityDto? q) => switch (q) {
    QuantityDto_Exact(:final numer, :final denom) => _ratio(numer, denom),
    QuantityDto_Range(
      :final minNumer,
      :final minDenom,
      :final maxNumer,
      :final maxDenom,
    ) =>
      '${_ratio(minNumer, minDenom)}-${_ratio(maxNumer, maxDenom)}',
    _ => '',
  };

  static String _ratio(int numer, int denom) =>
      denom == 1 ? '$numer' : '$numer/$denom';
}

class _RecipeFormScreenState extends ConsumerState<RecipeFormScreen> {
  final _title = TextEditingController();
  final _servings = TextEditingController();
  final _prepMinutes = TextEditingController();
  final _instructions = TextEditingController();
  final List<_LineDraft> _lines = [];
  final _titleKey = GlobalKey();
  final _servingsKey = GlobalKey();
  final _prepMinutesKey = GlobalKey();
  String? _titleError;
  String? _servingsError;
  String? _prepMinutesError;
  bool _saving = false;
  bool _seeded = false;
  RecipeDto? _existing;

  /// The recipe's sub-recipe components as edited here ("Edit section" replaces entries).
  /// Saved are only those a saved line still belongs to.
  List<RecipeComponentDto> _components = [];

  bool get _editing => widget.recipeId != null;

  @override
  void dispose() {
    _title.dispose();
    _servings.dispose();
    _prepMinutes.dispose();
    _instructions.dispose();
    for (final l in _lines) {
      l.dispose();
    }
    super.dispose();
  }

  /// Seeds from the loaded recipe, else from the import draft. Only a loaded recipe becomes
  /// `_existing`: a draft is a create, so the household guard in `_save` uses the live id.
  void _seedOnce(RecipeDto? existing) {
    if (_seeded) return;
    _seeded = true;
    _existing = existing;
    final seed = existing ?? widget.draft?.recipe;
    if (seed == null) {
      _lines.add(_LineDraft());
      return;
    }
    _title.text = seed.title;
    _servings.text = seed.servings?.toString() ?? '';
    _prepMinutes.text = seed.prepMinutes?.toString() ?? '';
    _instructions.text = seed.instructions;
    final inline = widget.draft?.inlineLines ?? const <int>[];
    _lines.addAll([
      for (final (i, line) in seed.lines.indexed)
        _LineDraft(line, inline.contains(i)),
    ]);
    if (_lines.isEmpty && existing == null) _lines.add(_LineDraft());
    _components = [...seed.components];
  }

  /// Builds the DTO, or `null` after setting the inline errors. Typed strings go verbatim;
  /// `original_text` is the row's [_LineDraft.savesAs].
  RecipeDto? _validate(String householdId) {
    var ok = true;
    _titleError = null;
    _servingsError = null;
    _prepMinutesError = null;
    if (_title.text.trim().isEmpty) {
      _titleError = 'Give the recipe a title.';
      ok = false;
    }
    int? servings;
    if (_servings.text.trim().isNotEmpty) {
      servings = int.tryParse(_servings.text.trim());
      // Upper bound for the reason `maxQuantity` records: `servings` is a bridge `u32`, so a
      // larger count would be masked to a different number and read back as `Serves 1`.
      if (servings == null || servings < 1 || servings > maxQuantity) {
        _servingsError =
            'Servings must be a whole number from 1 to $maxQuantity.';
        ok = false;
      }
    }
    int? prepMinutes;
    if (_prepMinutes.text.trim().isNotEmpty) {
      prepMinutes = int.tryParse(_prepMinutes.text.trim());
      // Same bound and same reason as `servings`: a bridge `u32`.
      if (prepMinutes == null || prepMinutes < 1 || prepMinutes > maxQuantity) {
        _prepMinutesError =
            'Prep time must be a whole number of minutes from 1 to $maxQuantity.';
        ok = false;
      }
    }
    final lines = <IngredientLineDto>[];
    for (final l in _lines) {
      l.error = null;
      // An untouched row (the one a new form starts with) is not a line; only a row the
      // user started filling in is validated.
      if ([
        l.name,
        l.amount,
        l.preparation,
        l.otherUnit,
      ].every((c) => c.text.isEmpty)) {
        continue;
      }
      final quantity = parseQuantity(l.amount.text);
      // A required name is also what keeps a composed `original_text` non-blank.
      if (l.name.text.trim().isEmpty) {
        l.error = 'name the ingredient.';
      } else if (quantity == null) {
        l.error =
            'cannot read "${l.amount.text}" — '
            'try 2, 1/2, 1 1/2 or 2-3, or leave it blank.';
      } else if (l.unitKey == unitOtherKey && l.otherUnit.text.trim().isEmpty) {
        l.error = 'name the unit.';
      } else if (l.preparation.text.isNotEmpty &&
          l.preparation.text.trim().isEmpty) {
        // The domain applies the same non-blank rule to `preparation` as to the fields
        // above, so a lone space would fail the whole save in Rust with no row named. The
        // emit below still sends the text verbatim — normalising it here to `null` would
        // silently discard what the user typed, which is this card's stop condition.
        l.error = 'write a preparation or leave it blank.';
      }
      if (l.error != null) {
        ok = false;
        continue;
      }
      lines.add(
        IngredientLineDto(
          originalText: l.savesAs,
          name: l.name.text,
          ingredient: l.emittedIngredient,
          quantity: quantity!,
          unit: switch (l.unitKey) {
            unitNoneKey => const UnitDto.none(),
            unitOtherKey => UnitDto.other(text: l.otherUnit.text),
            final kind => UnitDto.known(unit: kind),
          },
          preparation: l.preparation.text.isEmpty ? null : l.preparation.text,
          optional: l.optional,
          component: l.component,
        ),
      );
    }
    if (!ok) return null;
    // Only components a saved line still belongs to: a component whose rows were all
    // removed or blanked goes with them, rather than failing the save in Rust.
    final owned = {for (final l in lines) l.component};
    return RecipeDto(
      id: widget.recipeId ?? '',
      householdId: householdId,
      title: _title.text,
      servings: servings,
      prepMinutes: prepMinutes,
      instructions: _instructions.text,
      lines: lines,
      provenance:
          _existing?.provenance ??
          widget.draft?.recipe.provenance ??
          const RecipeProvenanceDto(kind: 'authored'),
      components: [
        for (final c in _components)
          if (owned.contains(c.position)) c,
      ],
    );
  }

  /// The field `_validate` rejected first, in layout order, or `null` if it rejected none.
  GlobalKey? _firstErrorKey() {
    if (_titleError != null) return _titleKey;
    if (_servingsError != null) return _servingsKey;
    if (_prepMinutesError != null) return _prepMinutesKey;
    for (final l in _lines) {
      if (l.error != null) return l.errorKey;
    }
    return null;
  }

  /// A rejected save's only signal is the inline error, and the user is by then scrolled to
  /// Save at the foot of the form — so the offending field is above the fold and nothing on
  /// screen changes, which reads as Save being broken. Scroll the first one back into view.
  ///
  /// Post-frame because the errors are not painted until the `finally`'s `setState` rebuilds:
  /// a row's error `Text` does not exist in the tree until then, and its arrival changes the
  /// layout the scroll is computed against.
  void _revealFirstError() {
    final target = _firstErrorKey();
    if (target == null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final context = target.currentContext;
      if (context != null) {
        Scrollable.ensureVisible(
          context,
          duration: MediaQuery.disableAnimationsOf(context)
              ? Duration.zero
              : const Duration(milliseconds: 200),
        );
      }
    });
  }

  /// An edit's owning household comes from the recipe that was loaded, never from the live
  /// provider: `_seedOnce` latches `_existing`, so a household that changes under a form
  /// already on screen would otherwise re-stamp the recipe with whichever id is current
  /// (invariant 1). The two ids agreeing is a precondition of saving at all — a form whose
  /// recipe and household disagree was seeded from an entry that is no longer this
  /// household's, and there is no field the user could correct to resolve it.
  ///
  /// Defence in depth rather than a reachable bug today: `write_recipe`'s owner probe rejects
  /// the reassignment in Rust as well, and one household is all `bootstrapHousehold` ever
  /// produces. It reports the refusal as `NoSuchRecipe` though, which reads as "recipe not
  /// found" over a recipe plainly on screen — so the refusal is worth stating here, in the one
  /// place that knows *why* the ids differ.
  ///
  /// Ahead of `_saving` because it neither throws nor awaits: entering the saving state only
  /// to leave it in the same frame would flicker every control on the form.
  ///
  /// `_validate` runs *inside* the `try`: this `Future` is discarded by the Save button's
  /// `VoidCallback`, so a synchronous throw out of validation would complete it with an error
  /// nothing reads — no inline error, no snackbar, no write, no sign the tap registered.
  /// The `finally`'s `setState` is what publishes the inline errors on the reject path.
  Future<void> _save(String householdId) async {
    // `_existing` is null on a create, which makes `owner` the live id and this branch dead —
    // so the mismatch is the whole condition and no separate `_editing` test is needed.
    final owner = _existing?.householdId ?? householdId;
    if (owner != householdId) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text(
            'This recipe belongs to another household and cannot be saved here.',
          ),
        ),
      );
      return;
    }
    setState(() => _saving = true);
    try {
      final dto = _validate(owner);
      if (dto == null) {
        _revealFirstError();
        return;
      }
      final stored = await ref.read(recipeLibraryProvider.notifier).save(dto);
      if (widget.draft != null) {
        // Imported: the new recipe is what the user wants to see. Leaving the review route
        // clears the import (its `onExit`), so there is no draft left to re-offer.
        if (mounted) context.go('/recipes/${stored.id}');
        return;
      }
      if (mounted) {
        context.go(_editing ? '/recipes/${stored.id}' : '/recipes');
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(describeFailure(e, subject: 'Recipes'))),
        );
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  /// The row's fields can still hold focus — tapping the ✕ does not unfocus a `TextField` on
  /// Android — and `EditableText` touches its controller while tearing down, so the dispose
  /// waits for the frame that unmounts it rather than running inside `setState`.
  Future<void> _removeLine(int i) async {
    final component = _componentOf(_lines[i]);
    final last =
        component != null &&
        component.status == 'expanded' &&
        _lines.where((l) => l.component == component.position).length == 1;
    if (last) {
      final remove = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          content: Text('Remove the ${component.title} section and its steps?'),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Keep'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('Remove'),
            ),
          ],
        ),
      );
      if (remove != true || !mounted) return;
    }
    final gone = _lines.removeAt(i);
    setState(() {});
    WidgetsBinding.instance.addPostFrameCallback((_) => gone.dispose());
  }

  @override
  Widget build(BuildContext context) {
    final kinds = ref.watch(knownUnitKindsProvider);
    // A create has nothing to load; `AsyncData(null)` keeps the two arms symmetrical.
    final existing = _editing
        ? ref.watch(recipeDetailProvider(widget.recipeId!))
        : const AsyncData<RecipeDto?>(null);
    return Scaffold(
      appBar: AppBar(
        title: Text(
          _editing
              ? 'Edit recipe'
              : widget.draft != null
              ? 'Review recipe'
              : 'New recipe',
        ),
      ),
      // Form only under AsyncData for both, as the restrictions editor does: an edit form
      // seeded from nothing would save an empty recipe over a real one.
      body: switch ((existing, kinds)) {
        (AsyncData(value: final r), AsyncData(value: final kinds)) => _body(
          r,
          kinds,
        ),
        (AsyncError(:final error), _) ||
        (_, AsyncError(:final error)) => Padding(
          padding: const EdgeInsets.all(24),
          child: Text(describeFailure(error, subject: 'Recipes')),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Widget _body(RecipeDto? existing, List<String> kinds) {
    if (_editing && existing == null) {
      return const Padding(
        padding: EdgeInsets.all(24),
        child: Text('Recipe not found.'),
      );
    }
    _seedOnce(existing);
    final householdId = ref.watch(householdProvider).valueOrNull?.id;
    // A `Column`, not a lazy `ListView`: every row's controls exist whether or not they are
    // on screen, so focus traversal and `Scrollable.ensureVisible` reach Save below the fold.
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (widget.draft?.notice case final notice?) _notice(notice),
          TextField(
            key: _titleKey,
            controller: _title,
            enabled: !_saving,
            decoration: InputDecoration(
              labelText: 'Title',
              errorText: _titleError,
            ),
          ),
          TextField(
            key: _servingsKey,
            controller: _servings,
            enabled: !_saving,
            keyboardType: TextInputType.number,
            decoration: InputDecoration(
              labelText: 'Servings',
              helperText: 'Optional.',
              errorText: _servingsError,
            ),
          ),
          TextField(
            key: _prepMinutesKey,
            controller: _prepMinutes,
            enabled: !_saving,
            keyboardType: TextInputType.number,
            decoration: InputDecoration(
              labelText: 'Prep time (minutes)',
              helperText: 'Optional.',
              errorText: _prepMinutesError,
            ),
          ),
          TextField(
            controller: _instructions,
            enabled: !_saving,
            maxLines: null,
            decoration: const InputDecoration(labelText: 'Instructions'),
          ),
          for (final c in _components)
            if (c.status == 'expanded' &&
                _lines.any((l) => l.component == c.position))
              _section(c),
          const SizedBox(height: 16),
          Text('Ingredients', style: Theme.of(context).textTheme.titleMedium),
          for (final (i, l) in _lines.indexed) ...[
            if (_startsGroup(i)) _groupHeading(l),
            _row(i, l, kinds),
          ],
          OutlinedButton(
            onPressed: _saving
                ? null
                : () => setState(() => _lines.add(_LineDraft())),
            child: const Text('Add ingredient'),
          ),
          const SizedBox(height: 16),
          FilledButton(
            onPressed: _saving || householdId == null
                ? null
                : () => _save(householdId),
            child: const Text('Save recipe'),
          ),
        ],
      ),
    );
  }

  RecipeComponentDto? _componentOf(_LineDraft l) {
    final position = l.component;
    if (position == null) return null;
    for (final c in _components) {
      if (c.position == position) return c;
    }
    return null;
  }

  /// A heading wherever the run of rows changes component, so a sub-recipe's lines sit
  /// together under its title and the main recipe's lines after it read as main again.
  bool _startsGroup(int i) => i == 0
      ? _lines[0].component != null
      : _lines[i].component != _lines[i - 1].component;

  Widget _groupHeading(_LineDraft l) {
    final c = _componentOf(l);
    final text = Theme.of(context).textTheme;
    return Padding(
      padding: const EdgeInsets.only(top: 12),
      child: c == null
          ? Text('Main recipe', style: text.titleSmall)
          : Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(c.title, style: text.titleSmall),
                Text(switch (c.status) {
                  'unresolved' =>
                    "Couldn't be added automatically — kept as written",
                  'alternative' => 'Has a linked homemade alternative',
                  _ => 'Sub-recipe',
                }, style: text.bodySmall),
                for (final link in c.links)
                  TextButton.icon(
                    icon: const Icon(Icons.open_in_new),
                    label: Text(
                      'Open link: ${Uri.tryParse(link)?.host ?? link}',
                    ),
                    onPressed: () =>
                        ref.read(shareChannelProvider).openUrl(link),
                  ),
              ],
            ),
    );
  }

  /// The one consolidated notice (design §7). Tapping it scrolls to the first row the
  /// import could not complete.
  Widget _notice(String notice) => Card(
    color: Theme.of(context).colorScheme.secondaryContainer,
    child: InkWell(
      onTap: () {
        final first = _lines.where(
          (l) => _componentOf(l)?.status == 'unresolved',
        );
        final target = first.isEmpty ? null : first.first.rowKey.currentContext;
        if (target != null) Scrollable.ensureVisible(target);
      },
      child: Padding(padding: const EdgeInsets.all(12), child: Text(notice)),
    ),
  );

  /// An expanded sub-recipe's steps: read-only here, edited through "Edit section".
  Widget _section(RecipeComponentDto c) => Padding(
    padding: const EdgeInsets.only(top: 12),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(c.title, style: Theme.of(context).textTheme.titleSmall),
        if (c.instructions.isNotEmpty) Text(c.instructions),
        TextButton(
          onPressed: _saving ? null : () => _editSection(c),
          child: Text('Edit section: ${c.title}'),
        ),
      ],
    ),
  );

  Future<void> _editSection(RecipeComponentDto c) async {
    final title = TextEditingController(text: c.title);
    final steps = TextEditingController(text: c.instructions);
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Edit section'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: title,
              decoration: const InputDecoration(labelText: 'Section title'),
            ),
            TextField(
              controller: steps,
              maxLines: null,
              decoration: const InputDecoration(labelText: 'Steps'),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          // Rust rejects a blank title, so the dialog never offers one.
          ListenableBuilder(
            listenable: title,
            builder: (context, _) => TextButton(
              onPressed: title.text.trim().isEmpty
                  ? null
                  : () => Navigator.pop(context, true),
              child: const Text('Save section'),
            ),
          ),
        ],
      ),
    );
    if (saved == true && mounted) {
      setState(() {
        _components = [
          for (final x in _components)
            x.position == c.position
                ? RecipeComponentDto(
                    position: x.position,
                    title: title.text,
                    sourceUrl: x.sourceUrl,
                    scaleNumer: x.scaleNumer,
                    scaleDenom: x.scaleDenom,
                    replacedText: x.replacedText,
                    status: x.status,
                    links: x.links,
                    instructions: steps.text,
                  )
                : x,
        ];
      });
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      title.dispose();
      steps.dispose();
    });
  }

  Widget _row(int i, _LineDraft l, List<String> kinds) => Card(
    key: ValueKey(l),
    margin: const EdgeInsets.symmetric(vertical: 8),
    child: Padding(
      padding: const EdgeInsets.all(12),
      child: Column(
        key: l.rowKey,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(child: Text('Ingredient ${i + 1}')),
              IconButton(
                icon: const Icon(Icons.close),
                tooltip: 'Remove ingredient ${i + 1}',
                onPressed: _saving ? null : () => _removeLine(i),
              ),
            ],
          ),
          if (l.inlineHint)
            const Text(
              'This line names a sauce whose ingredients are listed below; '
              'you can delete it.',
            ),
          // Rebuilt on each keystroke in the fields it reads; the unit dropdown already
          // rebuilds the form.
          ListenableBuilder(
            listenable: Listenable.merge([
              l.name,
              l.amount,
              l.otherUnit,
              l.preparation,
            ]),
            builder: (context, _) {
              final line = l.savesAs;
              return line.isEmpty
                  ? const SizedBox.shrink()
                  : Text(savesAsCopy(line));
            },
          ),
          TextField(
            controller: l.name,
            enabled: !_saving,
            decoration: const InputDecoration(labelText: 'Name'),
          ),
          TextField(
            controller: l.amount,
            enabled: !_saving,
            decoration: const InputDecoration(
              labelText: 'Amount',
              helperText: 'Optional: 2, 1/2, 1 1/2 or 2-3.',
            ),
          ),
          DropdownButtonFormField<String>(
            initialValue: l.unitKey,
            decoration: const InputDecoration(labelText: 'Unit'),
            items: [
              const DropdownMenuItem(
                value: unitNoneKey,
                child: Text('No unit'),
              ),
              for (final kind in kinds)
                DropdownMenuItem(value: kind, child: Text(unitLabel(kind))),
              const DropdownMenuItem(
                value: unitOtherKey,
                child: Text('Other…'),
              ),
            ],
            onChanged: _saving
                ? null
                : (key) => setState(() => l.unitKey = key ?? unitNoneKey),
          ),
          if (l.unitKey == unitOtherKey)
            TextField(
              controller: l.otherUnit,
              enabled: !_saving,
              decoration: const InputDecoration(labelText: 'Unit name'),
            ),
          TextField(
            controller: l.preparation,
            enabled: !_saving,
            decoration: const InputDecoration(
              labelText: 'Preparation',
              helperText: 'Optional, e.g. sifted.',
            ),
          ),
          SwitchListTile(
            title: const Text('Optional'),
            value: l.optional,
            onChanged: _saving ? null : (on) => setState(() => l.optional = on),
          ),
          if (l.error case final error?)
            Text(
              'Ingredient ${i + 1}: $error',
              key: l.errorKey,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
        ],
      ),
    ),
  );
}
