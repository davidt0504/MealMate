import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:meal_mate/features/household/household_screen.dart';
import 'package:meal_mate/features/shopping/shopping_copy.dart';
import 'package:meal_mate/features/shopping/shopping_provider.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/shopping.dart';

class ShoppingScreen extends ConsumerStatefulWidget {
  const ShoppingScreen({super.key});

  @override
  ConsumerState<ShoppingScreen> createState() => _ShoppingScreenState();
}

sealed class _ManualItemResult {
  const _ManualItemResult();
}

class _ManualItemSave extends _ManualItemResult {
  const _ManualItemSave(this.name, this.note);
  final String name;
  final String? note;
}

class _ManualItemDelete extends _ManualItemResult {
  const _ManualItemDelete(this.id);
  final String id;
}

/// Add/edit form for one manual item. Pops a [_ManualItemResult] (or `null` on cancel);
/// the screen does the writing so the dialog holds no notifier.
class _ManualItemDialog extends StatefulWidget {
  const _ManualItemDialog({required this.existing});

  final ShoppingManualItemDto? existing;

  @override
  State<_ManualItemDialog> createState() => _ManualItemDialogState();
}

class _ManualItemDialogState extends State<_ManualItemDialog> {
  late final _name = TextEditingController(text: widget.existing?.name ?? '');
  late final _note = TextEditingController(text: widget.existing?.note ?? '');
  String? _error;

  @override
  void dispose() {
    _name.dispose();
    _note.dispose();
    super.dispose();
  }

  void _save() {
    if (_name.text.trim().isEmpty) {
      setState(() => _error = manualNameRequiredCopy);
      return;
    }
    final note = _note.text.trim();
    Navigator.pop(
      context,
      _ManualItemSave(_name.text, note.isEmpty ? null : note),
    );
  }

  @override
  Widget build(BuildContext context) {
    final existing = widget.existing;
    return AlertDialog(
      title: Text(existing == null ? 'Add item' : 'Edit item'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          TextField(
            controller: _name,
            autofocus: true,
            decoration: InputDecoration(labelText: 'Name', errorText: _error),
          ),
          TextField(
            controller: _note,
            decoration: const InputDecoration(labelText: 'Note'),
          ),
        ],
      ),
      actions: [
        if (existing != null)
          TextButton(
            onPressed: () =>
                Navigator.pop(context, _ManualItemDelete(existing.id)),
            child: const Text('Delete'),
          ),
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(onPressed: _save, child: const Text('Save')),
      ],
    );
  }
}

class _ShoppingScreenState extends ConsumerState<ShoppingScreen> {
  int _offset = 0;
  // Per-key, as the pantry's per-row set: one line's write must not freeze the others.
  final Set<String> _writing = {};

  ShoppingNotifier get _notifier =>
      ref.read(shoppingProvider(_offset).notifier);

  void _report(Object error) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(describeFailure(error, subject: 'Shopping'))),
    );
  }

  Future<void> _refresh() async {
    final error = await _notifier.refresh();
    if (error != null) _report(error);
  }

  /// Runs one keyed write; failures land in the snackbar and the row, rendered from
  /// provider state, stays at its stored value.
  Future<void> _write(String key, Future<void> Function() action) async {
    final startedAt = _offset;
    setState(() => _writing.add(key));
    try {
      await action();
    } catch (e) {
      _report(e);
    } finally {
      // A cycle change already cleared the set. Line keys carry no window bound, so removing
      // the key here would unfreeze a row the *new* cycle is itself writing.
      if (mounted && _offset == startedAt) {
        setState(() => _writing.remove(key));
      }
    }
  }

  /// The check exactly as a row renders it: a check made against an amount that has since
  /// moved is reported rather than kept, so it is not carried across a section change either.
  static bool _renderedCheck(ShoppingLineStateDto? state) =>
      (state?.checked ?? false) && !(state?.changed ?? false);

  Future<void> _setLine(
    ShoppingLineDto line, {
    required bool checked,
    required bool hidden,
    required bool restored,
  }) => _write(
    line.key,
    () => _notifier.setLineState(
      ShoppingLineStateDto(
        key: line.key,
        checked: checked,
        hidden: hidden,
        // `restored` qualifies a pantry-omitted line and nothing else. Carried onto a line
        // the derivation now calls Needed it would sit in storage meaning nothing, then
        // override the next pantry mark for the same key.
        restored:
            restored &&
            line.status == ShoppingLineStatusDto.omittedPantryMarked,
        changed: false,
      ),
    ),
  );

  /// Decision 7: checked, still needed, resolved, not removed, and not checked against an
  /// amount that has since moved.
  List<IngredientRefDto> _eligible(ShoppingView view) => [
    for (final group in view.view.list.groups)
      for (final line in group.lines)
        if (line.ingredient != null &&
            line.status == ShoppingLineStatusDto.needed)
          if (view.stateFor(line.key) case final s?
              when s.checked && !s.changed && !s.hidden)
            line.ingredient!,
  ];

  Future<void> _addToPantry(ShoppingView view) async {
    final refs = _eligible(view);
    if (refs.isEmpty) return;
    try {
      final changed = await _notifier.addCheckedToPantry(refs);
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(addedToPantryCopy(changed.length)),
          action: SnackBarAction(
            label: 'Undo',
            // Exactly the returned set, so a mark that predated this action survives.
            onPressed: () =>
                _notifier.undoPantryMarks(changed).catchError((Object e) {
                  _report(e);
                  return <IngredientRefDto>[];
                }),
          ),
        ),
      );
    } catch (e) {
      _report(e);
    }
  }

  Future<void> _startOver(ShoppingView view) async {
    final checkedItems = view.view.manualItems.where((i) => i.checked).length;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text(resetTitle),
        // The orphaned rows are counted too: the bridge filtered them out of `line_states`,
        // but `reset_shopping_list` deletes every row for the window regardless.
        content: Text(
          resetBody(
            view.view.lineStates.length + view.view.orphanedLineStateCount,
            checkedItems,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Start over'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    try {
      await _notifier.reset();
    } catch (e) {
      _report(e);
    }
  }

  /// The dialog owns its controllers (a `StatefulWidget`, disposed with its `State`), because
  /// the route's future completes on pop while the exit animation is still rendering the
  /// fields — disposing then trips "used after being disposed" mid-frame.
  Future<void> _editItem(
    ShoppingManualItemDto? existing,
    String householdId,
  ) async {
    final result = await showDialog<_ManualItemResult>(
      context: context,
      builder: (_) => _ManualItemDialog(existing: existing),
    );
    switch (result) {
      case null:
        return;
      case _ManualItemDelete(:final id):
        await _write(id, () => _notifier.deleteManualItem(id));
      case _ManualItemSave(:final name, :final note):
        await _write(
          existing?.id ?? '',
          () => _notifier.saveManualItem(
            ShoppingManualItemDto(
              id: existing?.id ?? '',
              householdId: householdId,
              name: name,
              note: note,
              checked: existing?.checked ?? false,
            ),
          ),
        );
    }
  }

  @override
  Widget build(BuildContext context) {
    final view = ref.watch(shoppingProvider(_offset));
    final ready = view.valueOrNull;
    return Scaffold(
      appBar: AppBar(
        title: const Text(shoppingTitle),
        actions: [
          IconButton(
            tooltip: 'Previous cycle',
            icon: const Icon(Icons.chevron_left),
            // The busy set is per-key and keys carry no window bound, so it would otherwise
            // freeze the arriving cycle's rows on the outgoing cycle's writes.
            onPressed: () => setState(() {
              _offset--;
              _writing.clear();
            }),
          ),
          IconButton(
            tooltip: 'Next cycle',
            icon: const Icon(Icons.chevron_right),
            onPressed: () => setState(() {
              _offset++;
              _writing.clear();
            }),
          ),
          PopupMenuButton<String>(
            tooltip: 'More',
            onSelected: (choice) {
              if (ready == null) return;
              switch (choice) {
                case 'pantry':
                  _addToPantry(ready);
                case 'reset':
                  _startOver(ready);
              }
            },
            itemBuilder: (_) => [
              PopupMenuItem(
                value: 'pantry',
                enabled: ready != null && _eligible(ready).isNotEmpty,
                child: const Text('Add checked to pantry'),
              ),
              PopupMenuItem(
                value: 'reset',
                enabled: ready != null,
                child: const Text('Start over'),
              ),
            ],
          ),
        ],
      ),
      // A dependency-driven reload (a meal save, a pantry toggle, the pantry invalidation
      // after a bulk mark) must not blank a list the user is reading: a loading state that
      // still carries a value renders it, and the spinner is reserved for a first load.
      body: switch (view) {
        AsyncData(:final value) => _body(value),
        AsyncLoading(valueOrNull: final value?) => _body(value),
        AsyncError(:final error) => Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(describeFailure(error, subject: 'Shopping')),
              const SizedBox(height: 16),
              FilledButton(onPressed: _refresh, child: const Text('Try again')),
            ],
          ),
        ),
        _ => const Center(child: CircularProgressIndicator()),
      },
    );
  }

  Widget _body(ShoppingView view) {
    final list = view.view.list;
    final householdId = view.window.householdId;
    final needed = <ShoppingGroupDto, List<ShoppingLineDto>>{};
    final alreadyHave = <ShoppingLineDto>[];
    final removed = <ShoppingLineDto>[];
    for (final group in list.groups) {
      for (final line in group.lines) {
        switch (sectionFor(line.status, view.stateFor(line.key))) {
          case Section.needed:
            needed.putIfAbsent(group, () => []).add(line);
          case Section.alreadyHave:
            alreadyHave.add(line);
          case Section.removed:
            removed.add(line);
        }
      }
    }
    final neededCount = needed.values.fold<int>(0, (n, l) => n + l.length);
    final items = view.view.manualItems;
    final theme = Theme.of(context).textTheme;
    return RefreshIndicator(
      onRefresh: _refresh,
      child: ListView(
        padding: const EdgeInsets.all(16),
        // Without this the gesture is inert on a short list — the empty state exactly.
        physics: const AlwaysScrollableScrollPhysics(),
        children: [
          Text(shoppingHeading(view.from, view.to), style: theme.titleMedium),
          Text(countsCopy(neededCount, alreadyHave.length, removed.length)),
          // States the derivation no longer has a line for: said, not silently dropped, so
          // the user is not left reading a list their earlier edits are simply absent from.
          if (view.view.orphanedLineStateCount > 0)
            Text(orphanedStatesCopy(view.view.orphanedLineStateCount)),
          const SizedBox(height: 8),
          const Text(manualEditPolicyCopy),
          const SizedBox(height: 16),
          if (list.groups.isEmpty && items.isEmpty)
            const Text(shoppingEmptyCopy)
          else ...[
            if (neededCount > 0) Text(neededHeading, style: theme.titleLarge),
            for (final entry in needed.entries) ...[
              Text(
                entry.key.category ?? uncategorisedHeading,
                style: theme.titleSmall,
              ),
              for (final line in entry.value) _neededRow(line, view),
            ],
          ],
          const SizedBox(height: 16),
          Text(yourItemsHeading, style: theme.titleLarge),
          for (final item in items) _manualRow(item, householdId),
          Align(
            alignment: Alignment.centerLeft,
            // `_editItem` writes a new item under the empty-string key; guarding on it stops
            // a second dialog opening over an in-flight save, which Rust would mint a second
            // id for.
            child: TextButton.icon(
              onPressed: _writing.contains('')
                  ? null
                  : () => _editItem(null, householdId),
              icon: const Icon(Icons.add),
              label: const Text('Add item'),
            ),
          ),
          if (alreadyHave.isNotEmpty)
            ExpansionTile(
              title: Text('$alreadyHaveHeading (${alreadyHave.length})'),
              children: [
                for (final line in alreadyHave)
                  ListTile(
                    title: Text(line.name),
                    subtitle: Text(
                      '${describeQuantity(line.quantity, line.unit)} · '
                      '$omittedCopy',
                    ),
                    trailing: TextButton(
                      onPressed: _writing.contains(line.key)
                          ? null
                          : () => _setLine(
                              line,
                              checked: _renderedCheck(view.stateFor(line.key)),
                              hidden: false,
                              restored: true,
                            ),
                      child: const Text('Add anyway'),
                    ),
                  ),
              ],
            ),
          if (removed.isNotEmpty)
            ExpansionTile(
              title: Text('$removedHeading (${removed.length})'),
              children: [
                for (final line in removed)
                  ListTile(
                    title: Text(line.name),
                    subtitle: Text(describeQuantity(line.quantity, line.unit)),
                    trailing: TextButton(
                      onPressed: _writing.contains(line.key)
                          ? null
                          : () => _setLine(
                              line,
                              checked: _renderedCheck(view.stateFor(line.key)),
                              hidden: false,
                              restored:
                                  view.stateFor(line.key)?.restored ?? false,
                            ),
                      child: const Text('Put back'),
                    ),
                  ),
              ],
            ),
        ],
      ),
    );
  }

  Widget _neededRow(ShoppingLineDto line, ShoppingView view) {
    final state = view.stateFor(line.key);
    final changed = state?.changed ?? false;
    // A check made against a different amount renders unchecked: it is reported, not kept.
    final checked = _renderedCheck(state);
    final busy = _writing.contains(line.key);
    final subtitle = [
      describeQuantity(line.quantity, line.unit),
      if (line.optional) optionalCopy,
      if (changed)
        changedCopy(describeCheckedAgainst(state?.checkedAgainst ?? '')),
    ].join(' · ');
    // The label goes *inside* the tile, for the reason the pantry row gives: the tile's
    // merged node is the tappable one. The explain button sits outside the tile so it is
    // its own labelled node rather than being merged into the checkbox's.
    return Row(
      children: [
        Expanded(
          child: CheckboxListTile(
            title: Semantics(
              label: lineLabel(line.name, checked),
              child: ExcludeSemantics(child: Text(line.name)),
            ),
            subtitle: ExcludeSemantics(child: Text(subtitle)),
            value: checked,
            onChanged: busy
                ? null
                : (value) => _setLine(
                    line,
                    checked: value ?? false,
                    hidden: false,
                    restored: state?.restored ?? false,
                  ),
          ),
        ),
        IconButton(
          tooltip: 'Why is this here?',
          icon: const Icon(Icons.info_outline),
          onPressed: () => _explain(line, state),
        ),
      ],
    );
  }

  void _explain(ShoppingLineDto line, ShoppingLineStateDto? state) {
    final canReturnToPantry =
        line.status == ShoppingLineStatusDto.omittedPantryMarked &&
        (state?.restored ?? false);
    showModalBottomSheet<void>(
      context: context,
      builder: (context) => SafeArea(
        child: ListView(
          shrinkWrap: true,
          padding: const EdgeInsets.all(16),
          children: [
            Text(line.name, style: Theme.of(context).textTheme.titleLarge),
            for (final c in line.contributions) Text(explainContribution(c)),
            if (line.separateReason case final reason?)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Text(separateReasonCopy(reason)),
              ),
            if (line.status == ShoppingLineStatusDto.omittedPantryMarked)
              const Padding(
                padding: EdgeInsets.only(top: 8),
                child: Text(omittedCopy),
              ),
            const SizedBox(height: 8),
            TextButton(
              onPressed: () {
                Navigator.pop(context);
                _setLine(
                  line,
                  // Removing is a move, not an edit: the check comes back with "Put back".
                  checked: _renderedCheck(state),
                  hidden: true,
                  restored: state?.restored ?? false,
                );
              },
              child: const Text('Remove from list'),
            ),
            if (canReturnToPantry)
              TextButton(
                onPressed: () {
                  Navigator.pop(context);
                  _setLine(
                    line,
                    // The fourth arm of the same move: sending a line back under Already
                    // have does not edit it, so the check comes with it.
                    checked: _renderedCheck(state),
                    hidden: false,
                    restored: false,
                  );
                },
                child: const Text('Back to pantry'),
              ),
          ],
        ),
      ),
    );
  }

  Widget _manualRow(ShoppingManualItemDto item, String householdId) {
    final busy = _writing.contains(item.id);
    return Row(
      children: [
        Expanded(
          child: CheckboxListTile(
            title: Semantics(
              label: lineLabel(item.name, item.checked),
              child: ExcludeSemantics(child: Text(item.name)),
            ),
            subtitle: item.note == null
                ? null
                : ExcludeSemantics(child: Text(item.note!)),
            value: item.checked,
            onChanged: busy
                ? null
                : (value) => _write(
                    item.id,
                    () => _notifier.saveManualItem(
                      ShoppingManualItemDto(
                        id: item.id,
                        householdId: item.householdId,
                        name: item.name,
                        note: item.note,
                        checked: value ?? false,
                      ),
                    ),
                  ),
          ),
        ),
        IconButton(
          tooltip: 'Edit item',
          icon: const Icon(Icons.edit_outlined),
          onPressed: busy ? null : () => _editItem(item, householdId),
        ),
      ],
    );
  }
}
