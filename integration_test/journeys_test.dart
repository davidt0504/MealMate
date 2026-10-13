// The pre-release GUI journeys: real Android, the real Rust core, a real SQLite file, and data
// carried across screens the way a household uses the app. `test/app_test.dart` pins each
// screen against fakes; nothing there proves an accepted Cover reaches Shopping and Pantry.
//
// Run through `tools/gui_check.sh journeys`, never `flutter test`: WSL cannot reach the port
// that needs, so the script builds this file as the app's entrypoint and reads the verdict
// from logcat. It refuses anything but the emulator -- every journey deletes the app's database.
//
// Each journey starts from a fresh database: the scratch file is opened first so the live
// connection lets go of `kimatta.db`, which is then deleted, and a keyed `ProviderScope` builds
// a new container. No `pumpAndSettle`: the app's progress indicators never settle, so every
// wait polls a finder under the live binding instead.

import 'dart:async';
import 'dart:io';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:path_provider/path_provider.dart';

import 'package:meal_mate/app/app.dart';
import 'package:meal_mate/features/planning/cover_provider.dart';
import 'package:meal_mate/features/planning/cover_screen.dart';
import 'package:meal_mate/features/planning/planner_copy.dart';
import 'package:meal_mate/features/recipes/recipes_provider.dart';
import 'package:meal_mate/features/settings/backup_provider.dart';
import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';

/// `--dart-define=JOURNEY_ONLY=J4` (`GUI_ONLY` in the script) runs one journey.
const _only = String.fromEnvironment('JOURNEY_ONLY');

/// A journey that found an app defect carries `ki:` -- the KNOWN_ISSUES.md title -- and is
/// skipped until the owner decides on the fix. Names start with their id and a space, so `J1`
/// matches J1 and not J10.
void journey(
  String name,
  Future<void> Function(WidgetTester tester) body, {
  String? ki,
}) {
  if (ki != null) debugPrint('journey skip: $name -- KI: $ki');
  testWidgets(
    name,
    (tester) async {
      debugPrint('journey start: $name');
      await body(tester);
    },
    skip: ki != null || (_only.isNotEmpty && !name.startsWith('$_only ')),
    timeout: const Timeout(Duration(minutes: 3)),
  );
}

late String _support;

Future<void> freshApp(WidgetTester tester) async {
  _support = (await getApplicationSupportDirectory()).path;
  await openDatabase(
    dbPath: '$_support${Platform.pathSeparator}journeys-scratch.db',
  );
  for (final entry in Directory(_support).listSync()) {
    final name = entry.uri.pathSegments.where((s) => s.isNotEmpty).last;
    if (name.startsWith('kimatta.db') || name == 'exports') {
      await entry.delete(recursive: true);
    }
  }
  final started = DateTime.now();
  await tester.pumpWidget(
    ProviderScope(
      key: UniqueKey(),
      child: App(key: UniqueKey()),
    ),
  );
  await waitFor(
    tester,
    title('Cover My Week'),
    timeout: const Duration(seconds: 45),
  );
  await waitFor(tester, find.byKey(const ValueKey('cover:week')));
  debugPrint(
    'journey startup: first-run Cover in ${DateTime.now().difference(started).inMilliseconds} ms',
  );
}

Finder tab(String label) =>
    find.descendant(of: find.byType(NavigationBar), matching: find.text(label));
Finder title(String text) =>
    find.descendant(of: find.byType(AppBar), matching: find.text(text));
Finder field(String label, [int i = 0]) =>
    find.widgetWithText(TextField, label).at(i);
Finder keyPrefix(String prefix) => find.byWidgetPredicate(
  (w) =>
      w.key is ValueKey<String> &&
      (w.key! as ValueKey<String>).value.startsWith(prefix),
);

ProviderContainer container(WidgetTester tester) =>
    ProviderScope.containerOf(tester.element(find.byType(App)));

/// Every visible `Text`, for a timeout message that says where the app actually was.
String _visibleTexts() => find
    .byType(Text)
    .evaluate()
    .map((e) => (e.widget as Text).data ?? '')
    .where((t) => t.isNotEmpty)
    .join(' | ');

/// `Finder.at(i)` throws rather than matching nothing while fewer than `i + 1` exist.
bool _present(Finder finder) {
  try {
    return finder.evaluate().isNotEmpty;
  } on RangeError {
    return false;
  }
}

Future<void> waitFor(
  WidgetTester tester,
  Finder finder, {
  Duration timeout = const Duration(seconds: 15),
}) async {
  final end = DateTime.now().add(timeout);
  while (!_present(finder)) {
    if (DateTime.now().isAfter(end)) {
      fail('timed out waiting for $finder; visible: ${_visibleTexts()}');
    }
    await tester.pump(const Duration(milliseconds: 100));
  }
}

Future<void> waitGone(
  WidgetTester tester,
  Finder finder, {
  Duration timeout = const Duration(seconds: 15),
}) async {
  final end = DateTime.now().add(timeout);
  while (_present(finder)) {
    if (DateTime.now().isAfter(end)) {
      fail('timed out waiting for $finder to go; visible: ${_visibleTexts()}');
    }
    await tester.pump(const Duration(milliseconds: 100));
  }
}

Future<void> tapAndWait(
  WidgetTester tester,
  Finder target,
  Finder expected,
) async {
  FocusManager.instance.primaryFocus?.unfocus();
  await waitFor(tester, target);
  await _center(tester, target);
  await tester.tap(target.first);
  await waitFor(tester, expected);
}

/// Drags the first (or given) scrollable until [target] is built; lazy lists build only
/// what is on screen.
Future<void> scrollUntilVisible(
  WidgetTester tester,
  Finder target, {
  Finder? scrollable,
  double delta = 300,
}) async {
  final view = scrollable ?? find.byType(Scrollable).first;
  for (var i = 0; i < 40 && !_present(target); i++) {
    await tester.drag(view, Offset(0, -delta));
    await tester.pump(const Duration(milliseconds: 200));
  }
  await waitFor(tester, target);
  await _center(tester, target);
}

/// Scrolls [target] to mid-viewport: `ensureVisible`'s edge alignment can leave it under the
/// NavigationBar, where a tap lands on the bar instead.
Future<void> _center(WidgetTester tester, Finder target) async {
  unawaited(
    Scrollable.ensureVisible(tester.element(target.first), alignment: 0.5),
  );
  await tester.pump(const Duration(milliseconds: 400));
  await _waitHittable(tester, target);
}

/// A layout still settling after the scroll (seen once on the recipe form's Save) leaves the
/// target's centre over empty scroll space, and the tap is lost. Polls the same hit test
/// `tap`'s own miss warning runs; if it never passes, the tap's warning and the caller's
/// `waitFor` timeout still report it.
Future<void> _waitHittable(WidgetTester tester, Finder target) async {
  final end = DateTime.now().add(const Duration(seconds: 5));
  bool hit() {
    final box = tester.renderObject(target.first);
    final result = HitTestResult();
    tester.binding.hitTestInView(
      result,
      tester.getCenter(target.first),
      tester.view.viewId,
    );
    return result.path.any((entry) => entry.target == box);
  }

  while (!hit() && DateTime.now().isBefore(end)) {
    await tester.pump(const Duration(milliseconds: 100));
  }
}

/// Waits out Cover's in-flight write.
Future<void> idle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 200));
  await waitGone(
    tester,
    find.bySemanticsLabel('Updating plan'),
    timeout: const Duration(seconds: 30),
  );
}

/// A snackbar with an action persists on this SDK and sits over the NavigationBar.
Future<void> clearSnackBars(WidgetTester tester) async {
  for (final m in find.byType(ScaffoldMessenger).evaluate()) {
    ((m as StatefulElement).state as ScaffoldMessengerState).clearSnackBars();
  }
  await tester.pump(const Duration(milliseconds: 500));
}

/// Delivers [text] the way `MainActivity` forwards a share into a running app.
Future<void> share(WidgetTester tester, String text) async {
  await tester.binding.defaultBinaryMessenger.handlePlatformMessage(
    'kimatta/share',
    const StandardMethodCodec().encodeMethodCall(
      MethodCall('sharedText', text),
    ),
    (_) {},
  );
  await tester.pump(const Duration(milliseconds: 300));
}

/// `+n to buy · ...` -> (to buy, already have).
(int, int) _counts() {
  final line = find
      .textContaining(' to buy · ')
      .evaluate()
      .map((e) => (e.widget as Text).data!)
      .first;
  final n = RegExp(r'\d+')
      .allMatches(line)
      .map((m) => int.parse(m[0]!))
      .toList();
  return (n[0], n[1]);
}

/// String keys matching [pattern], in tree order (top to bottom).
List<String> _keys(String pattern) => find
    .byWidgetPredicate(
      (w) =>
          w.key is ValueKey<String> &&
          RegExp(pattern).hasMatch((w.key! as ValueKey<String>).value),
    )
    .evaluate()
    .map((e) => (e.widget.key! as ValueKey<String>).value)
    .toList();

/// Cover's day cards, top to bottom: `cover:<date>:<slot>`.
List<String> _coverCards() => _keys(r'^cover:\d{4}-');

/// The Plan window heading, `<first date> to <last date>`. Built but offstage once the list
/// scrolls past it, so offstage widgets are searched too.
String _windowHeading() => find
    .byType(Text, skipOffstage: false)
    .evaluate()
    .map((e) => (e.widget as Text).data ?? '')
    .firstWhere((t) => RegExp(r'^\d{4}-\d\d-\d\d to ').hasMatch(t));

/// The dish a Cover card shows: its second `Text`, after the date line.
String _dish(String card) =>
    (find
                .descendant(
                  of: find.byKey(ValueKey(card)),
                  matching: find.byType(Text),
                )
                .evaluate()
                .elementAt(1)
                .widget
            as Text)
        .data!;

Future<void> _until(
  WidgetTester tester,
  bool Function() done,
  String what,
) async {
  final end = DateTime.now().add(const Duration(seconds: 15));
  while (!done()) {
    if (DateTime.now().isAfter(end)) {
      fail('timed out waiting for $what; visible: ${_visibleTexts()}');
    }
    await tester.pump(const Duration(milliseconds: 100));
  }
}

const stew = 'AAA Journey Stew';

/// Authors [stew] -- sorted first in the library -- through the form, ending on Recipes.
Future<void> authorStew(WidgetTester tester) async {
  await clearSnackBars(tester);
  await tapAndWait(tester, tab('Recipes'), title('Recipes'));
  await tapAndWait(tester, find.text('New recipe'), field('Title'));
  await tester.enterText(field('Title'), stew);
  await tester.enterText(field('Servings'), '4');
  await tester.enterText(field('Prep time (minutes)'), '25');
  await tester.enterText(field('Instructions'), 'Simmer everything.');
  await scrollUntilVisible(tester, field('Name'));
  await tester.enterText(field('Name'), 'journey saffron');
  await tester.enterText(field('Amount'), '2');
  await tapAndWait(tester, find.text('Add ingredient'), field('Name', 1));
  await scrollUntilVisible(tester, field('Name', 1));
  await tester.enterText(field('Name', 1), 'journey barley');
  final save = find.text('Save recipe');
  await scrollUntilVisible(tester, save);
  await tapAndWait(tester, save, title('Recipes'));
  await waitFor(tester, find.text(stew));
}

/// The Plan grid. The Plan tab returns to the screen it was left on (Cover, during first run);
/// a second tap on the selected tab resets it to the grid.
Future<void> toPlan(WidgetTester tester) async {
  await clearSnackBars(tester);
  await tester.tap(tab('Plan'));
  await _until(
    tester,
    () => _present(title('Plan')) || _present(title('Cover My Week')),
    'the Plan tab',
  );
  if (_present(title('Cover My Week'))) {
    await tapAndWait(tester, tab('Plan'), title('Plan'));
  }
  await waitFor(tester, keyPrefix('cell:'));
}

/// Archives [recipe] from its detail screen, ending on Recipes.
Future<void> archive(WidgetTester tester, String recipe) async {
  await clearSnackBars(tester);
  await tapAndWait(tester, tab('Recipes'), title('Recipes'));
  await tapAndWait(tester, find.text(recipe), title(recipe));
  final delete = find.text('Delete');
  await scrollUntilVisible(tester, delete);
  await tapAndWait(tester, delete, find.text('Archive this recipe?'));
  await tapAndWait(
    tester,
    find.widgetWithText(FilledButton, 'Archive'),
    title('Recipes'),
  );
  await waitGone(tester, find.text(recipe));
}

/// Renames the household from Settings, ending on Settings.
Future<void> rename(WidgetTester tester, String name) async {
  await clearSnackBars(tester);
  await tapAndWait(tester, tab('Settings'), title('Settings'));
  await tapAndWait(
    tester,
    find.widgetWithText(ListTile, 'Household'),
    title('Household'),
  );
  await waitFor(tester, field('Household name'));
  await tester.enterText(field('Household name'), name);
  await tapAndWait(
    tester,
    find.text('Save name'),
    find.widgetWithText(ListTile, name),
  );
  await tapAndWait(tester, find.byTooltip('Back'), title('Settings'));
  await waitFor(tester, find.textContaining('$name · '));
}

/// Adds an item of the user's own on Shopping.
Future<void> addItem(WidgetTester tester, String name) async {
  await scrollUntilVisible(tester, find.text('Add item'));
  await tapAndWait(tester, find.text('Add item'), field('Name'));
  await tester.enterText(field('Name'), name);
  await tapAndWait(
    tester,
    find.widgetWithText(FilledButton, 'Save'),
    // `find.text` would also match the dialog's own field.
    find.widgetWithText(CheckboxListTile, name),
  );
  await waitGone(tester, find.byType(AlertDialog));
}

/// The options menu on a shopping line.
Finder lineOptions(String name) => find.descendant(
  of: find.ancestor(of: find.text(name), matching: find.byType(Row)).first,
  matching: find.byTooltip('Line options'),
);

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async => RustLib.init());

  journey('J1 first run accept, shop, mark into pantry', (tester) async {
    await freshApp(tester);
    await tapAndWait(
      tester,
      find.byKey(const ValueKey('cover:accept')),
      find.text('Plan written.'),
    );
    await idle(tester);

    await tapAndWait(
      tester,
      find.text('Shopping list is ready'),
      title('Shopping'),
    );
    await waitFor(tester, find.textContaining(' to buy · '));
    final (toBuy, have) = _counts();
    expect(toBuy, greaterThan(0));

    final first = find.byType(CheckboxListTile).first;
    final name =
        (tester.widget<CheckboxListTile>(first).title! as Semantics).child!;
    final itemName = ((name as ExcludeSemantics).child! as Text).data!;
    await tester.tap(first);
    await waitFor(
      tester,
      find.byWidgetPredicate((w) => w is CheckboxListTile && w.value == true),
    );
    await tapAndWait(
      tester,
      find.byTooltip('More'),
      find.text('Add checked to pantry'),
    );
    await tapAndWait(
      tester,
      find.text('Add checked to pantry'),
      find.text('1 item marked as in your pantry'),
    );
    // The mark is per ingredient: every line of it moves, not only the one checked.
    await _until(
      tester,
      () => _counts().$2 != have,
      'the pantry mark to move lines',
    );
    final (toBuyAfter, haveAfter) = _counts();
    expect(haveAfter, greaterThan(have));
    expect(toBuyAfter + haveAfter, toBuy + have);
    final haveHeading = find.text('Already have ($haveAfter)');
    await clearSnackBars(tester);
    await scrollUntilVisible(tester, haveHeading);
    await tester.tap(haveHeading);
    await scrollUntilVisible(
      tester,
      find.descendant(
        of: find.byType(ExpansionTile),
        matching: find.text(itemName),
      ),
    );
    await clearSnackBars(tester);

    await tapAndWait(tester, tab('Pantry'), title('My Shelves'));
    await scrollUntilVisible(tester, find.text(itemName));
    final row = find
        .ancestor(of: find.text(itemName), matching: find.byType(Row))
        .first;
    expect(
      find.descendant(of: row, matching: find.byIcon(Icons.check_box)),
      findsOneWidget,
    );

    // Every cell of the accepted week is filled: none reads "Nothing planned." top to bottom.
    // The Plan tab returns to the Cover it was left on; a second tap resets it to the grid.
    await tapAndWait(tester, tab('Plan'), title('Cover My Week'));
    await tapAndWait(tester, tab('Plan'), find.textContaining(' · 1×'));
    for (var i = 0; i < 15; i++) {
      expect(find.text('Nothing planned.'), findsNothing);
      await tester.drag(find.byType(Scrollable).first, const Offset(0, -400));
      await tester.pump(const Duration(milliseconds: 200));
    }
  });

  journey('J2 cover another, undo, choose, never suggest, exclusions', (
    tester,
  ) async {
    await freshApp(tester);
    final card = _coverCards()[0];
    final slot = card.substring('cover:'.length);
    final original = _dish(card);

    await tester.tap(find.byKey(ValueKey('cover:another:$slot')));
    await idle(tester);
    await _until(
      tester,
      () => _dish(card) != original,
      'Another to change $card',
    );
    await tapAndWait(
      tester,
      find.byKey(const ValueKey('cover:undo')),
      find.text(original),
    );
    await idle(tester);
    expect(_dish(card), original);

    final pick = original == 'Taco Pasta' ? 'Taco Soup' : 'Taco Pasta';
    await tapAndWait(
      tester,
      find.byKey(ValueKey('cover:more:$slot')),
      find.text('Choose'),
    );
    await tapAndWait(tester, find.text('Choose'), find.text('Choose a meal'));
    final sheetItem = find.descendant(
      of: find.byType(BottomSheet),
      matching: find.text(pick),
    );
    await scrollUntilVisible(
      tester,
      sheetItem,
      scrollable: find
          .descendant(
            of: find.byType(BottomSheet),
            matching: find.byType(Scrollable),
          )
          .first,
    );
    await tester.tap(sheetItem);
    await idle(tester);
    await _until(
      tester,
      () => _dish(card) == pick,
      'the chosen $pick on $card',
    );
    expect(
      find.descendant(
        of: find.byKey(ValueKey('cover:lock:$slot')),
        matching: find.text('Locked in'),
      ),
      findsOneWidget,
    );

    final second = _coverCards()[1];
    final slot2 = second.substring('cover:'.length);
    final excluded = _dish(second);
    await tapAndWait(
      tester,
      find.byKey(ValueKey('cover:more:$slot2')),
      find.text('Never suggest'),
    );
    await tapAndWait(
      tester,
      find.text('Never suggest'),
      find.text('Never suggest this meal?'),
    );
    await tester.tap(find.widgetWithText(FilledButton, 'Never suggest'));
    await idle(tester);
    await waitFor(
      tester,
      find.descendant(
        of: find.byKey(ValueKey(second)),
        matching: find.text(
          'This meal conflicts with a rule, restriction or time limit.',
        ),
      ),
    );

    // Leaving Cover through the shell ends first run; the rule outlives the draft.
    await tapAndWait(tester, tab('Plan'), find.textContaining(' · Dinner'));
    await clearSnackBars(tester);
    await tapAndWait(tester, tab('Settings'), title('Settings'));
    final exclusions = find.byKey(const ValueKey('settings:exclusions'));
    await scrollUntilVisible(tester, exclusions);
    expect(
      find.descendant(
        of: exclusions,
        matching: find.text('1 meal is never suggested.'),
      ),
      findsOneWidget,
    );
    await tapAndWait(tester, exclusions, title('Meal exclusions'));
    await waitFor(tester, find.text(excluded));
    await tapAndWait(
      tester,
      find.text('Remove'),
      find.text(
        'Removed. It can be suggested again; nothing already planned changed.',
      ),
    );
    await waitFor(tester, find.text('No meals are excluded.'));
  });

  journey('J3 recipe create, edit, archive, restore', (tester) async {
    await freshApp(tester);
    await authorStew(tester);

    await tapAndWait(tester, find.text(stew), title(stew));
    await waitFor(tester, find.text('Serves 4'));
    expect(find.text('Prep 25 min'), findsOneWidget);
    expect(find.textContaining('journey saffron'), findsOneWidget);
    expect(find.textContaining('journey barley'), findsOneWidget);

    await tapAndWait(tester, find.byTooltip('Edit'), field('Title'));
    await tester.enterText(field('Title'), '$stew Two');
    final save = find.text('Save recipe');
    await scrollUntilVisible(tester, save);
    await tapAndWait(tester, save, title('$stew Two'));

    final delete = find.text('Delete');
    await scrollUntilVisible(tester, delete);
    await tapAndWait(tester, delete, find.text('Archive this recipe?'));
    await tapAndWait(
      tester,
      find.widgetWithText(FilledButton, 'Archive'),
      title('Recipes'),
    );
    await waitGone(tester, find.text('$stew Two'));

    await tapAndWait(
      tester,
      find.byTooltip('Archived recipes'),
      title('Archived recipes'),
    );
    await waitFor(tester, find.text('$stew Two'));
    await tapAndWait(
      tester,
      find.text('Restore'),
      find.text('Nothing archived.'),
    );
    await tapAndWait(tester, tab('Recipes'), title('Recipes'));
    await waitFor(tester, find.text('$stew Two'));
  });

  journey('J4 authored recipe through plan, shopping, explain, archive', (
    tester,
  ) async {
    await freshApp(tester);
    await authorStew(tester);

    await toPlan(tester);
    final cell = keyPrefix('cell:').first;
    await tapAndWait(
      tester,
      find.descendant(of: cell, matching: find.text('Add')),
      find.byType(BottomSheet),
    );
    await tapAndWait(
      tester,
      find.descendant(of: find.byType(BottomSheet), matching: find.text(stew)),
      find.descendant(of: cell, matching: find.text('$stew · 1×')),
    );

    await clearSnackBars(tester);
    await tapAndWait(tester, tab('Shopping'), title('Shopping'));
    await waitFor(tester, find.textContaining(' to buy · '));
    await scrollUntilVisible(tester, find.text('Other'));
    await scrollUntilVisible(tester, find.text('journey barley'));
    await scrollUntilVisible(tester, find.text('journey saffron'));

    await tapAndWait(
      tester,
      lineOptions('journey saffron'),
      find.text('Why is this here?'),
    );
    await tapAndWait(
      tester,
      find.text('Why is this here?'),
      find.text(
        'Not matched to a known ingredient, so it is listed on its own.',
      ),
    );
    expect(find.textContaining('$stew · '), findsOneWidget);
    await tapAndWait(
      tester,
      find.descendant(
        of: find.byType(BottomSheet),
        matching: find.text('Skip this time'),
      ),
      find.text('Removed (1)'),
    );

    await archive(tester, stew);
    await toPlan(tester);
    await waitFor(tester, find.text('$stew (archived) · 1×'));
  });

  journey('J5 planner add, scale, move, remove, cycle', (tester) async {
    await freshApp(tester);
    await toPlan(tester);
    await waitFor(tester, find.text(plannerEmptyCopy));
    const meal = 'Baked Turkey Meatballs';
    final cells = _keys(r'^cell:');
    final from = find.byKey(ValueKey(cells[0]));
    final to = find.byKey(ValueKey(cells[1]));
    Finder inCell(Finder cell, String text) =>
        find.descendant(of: cell, matching: find.text(text));
    // The menu's items render in an overlay, outside the cell.
    Future<void> menu(Finder cell, String item, Finder expected) async {
      await tapAndWait(
        tester,
        find.descendant(of: cell, matching: find.byTooltip('Show menu')),
        find.text(item),
      );
      await tapAndWait(tester, find.text(item).last, expected);
    }

    await tapAndWait(tester, inCell(from, 'Add'), find.byType(BottomSheet));
    await tapAndWait(
      tester,
      find.descendant(of: find.byType(BottomSheet), matching: find.text(meal)),
      inCell(from, '$meal · 1×'),
    );
    expect(find.text(plannerEmptyCopy), findsNothing);

    await menu(from, 'Scale…', find.text(scaleHeading));
    await tapAndWait(tester, find.text('2×'), inCell(from, '$meal · 2×'));

    final target = '${cells[1].split(':')[1]} · Dinner';
    await menu(from, 'Move…', find.text(moveToHeading));
    await tapAndWait(tester, find.text(target), inCell(to, '$meal · 2×'));
    // The move scrolls the target into view; the source may no longer be built.
    await scrollUntilVisible(tester, inCell(from, emptyCellCopy), delta: -300);

    await menu(to, 'Remove', find.text(removeLastComponentTitle));
    await tapAndWait(
      tester,
      find.widgetWithText(FilledButton, 'Remove'),
      inCell(to, emptyCellCopy),
    );
    expect(find.text('$meal · 2×'), findsNothing);
    await scrollUntilVisible(tester, find.text(plannerEmptyCopy), delta: -300);

    final heading = _windowHeading();
    await tester.tap(find.byTooltip('Next cycle'));
    await _until(tester, () => _windowHeading() != heading, 'the next cycle');
    await tester.tap(find.byTooltip('Previous cycle'));
    await _until(tester, () => _windowHeading() == heading, 'the cycle back');

    await clearSnackBars(tester);
    await tapAndWait(tester, tab('Settings'), title('Settings'));
    final cycle = find.text('Planning cycle');
    await scrollUntilVisible(tester, cycle);
    await tapAndWait(tester, cycle, find.text('7 days'));
    final lunch = find.widgetWithText(CheckboxListTile, 'Lunch');
    await tester.tap(lunch);
    await _until(
      tester,
      () => tester.widget<CheckboxListTile>(lunch).value ?? false,
      'Lunch ticked',
    );
    await tapAndWait(tester, find.byTooltip('More days'), find.text('8 days'));
    await tester.tap(find.text('Save cycle'));
    await tester.pump(const Duration(seconds: 1));
    await tapAndWait(tester, find.byTooltip('Back'), title('Settings'));
    await waitFor(tester, find.textContaining('Lunch, dinner · 8 days from '));

    await toPlan(tester);
    expect(_keys(r'^cell:.*:lunch$'), isNotEmpty);
    expect(_windowHeading(), isNot(heading));
  });

  journey('J6 vegetarian restriction warns, hides and steers Cover', (
    tester,
  ) async {
    const meat = 'Crockpot Barbecue Chicken';
    const veg = 'Black Bean and Lentil Soup';
    await freshApp(tester);
    final set = find.text('Set restrictions');
    await scrollUntilVisible(tester, set);
    await tapAndWait(tester, set, title('Restrictions'));
    final vegetarian = find.widgetWithText(CheckboxListTile, 'Vegetarian');
    await waitFor(tester, vegetarian);
    await tester.tap(vegetarian);
    await _until(
      tester,
      () => tester.widget<CheckboxListTile>(vegetarian).value ?? false,
      'Vegetarian ticked',
    );
    final save = find.text('Save restrictions');
    await scrollUntilVisible(tester, save);
    await tapAndWait(tester, save, find.text('Restrictions saved.'));

    await clearSnackBars(tester);
    await tapAndWait(tester, tab('Recipes'), title('Recipes'));
    await scrollUntilVisible(tester, find.text(meat));
    final meatTile = find.ancestor(
      of: find.text(meat),
      matching: find.byType(ListTile),
    );
    await waitFor(
      tester,
      find.descendant(
        of: meatTile,
        matching: find.text('May conflict: Vegetarian'),
      ),
    );
    await tapAndWait(tester, find.text(meat), title(meat));
    await scrollUntilVisible(tester, find.text('Restriction warnings'));
    await tapAndWait(tester, find.byTooltip('Back'), title('Recipes'));

    final hide = find.widgetWithText(FilterChip, 'Hide known conflicts');
    await scrollUntilVisible(tester, hide, delta: -300);
    await tapAndWait(tester, hide, find.text(veg));
    await waitGone(tester, find.text(meat));
    await tapAndWait(tester, hide, find.text(meat));

    final conflicting = {
      for (final r in container(tester).read(recipeLibraryProvider).value!)
        if (r.assessment.conflicts.isNotEmpty) r.title,
    };
    expect(conflicting, contains(meat));
    expect(conflicting, isNot(contains(veg)));

    await toPlan(tester);
    final heading = _windowHeading();
    await tester.tap(find.byTooltip('Next cycle'));
    await _until(tester, () => _windowHeading() != heading, 'the next cycle');
    await tapAndWait(
      tester,
      find.text('Cover My Week'),
      title('Cover My Week'),
    );
    await waitFor(tester, find.byKey(const ValueKey('cover:week')));
    await _until(tester, () => _coverCards().isNotEmpty, 'Cover cards');
    final offset = tester.widget<CoverScreen>(find.byType(CoverScreen)).offset;
    final slots = container(tester)
        .read(coverProvider(offset))
        .value!
        .draft
        .slots
        .length;
    // Cover's list builds only the cards on screen, so they are collected while scrolling.
    final checked = <String>{};
    for (var i = 0; i < 40 && checked.length < slots; i++) {
      for (final card in _coverCards()) {
        if (checked.add(card)) {
          expect(conflicting, isNot(contains(_dish(card))), reason: card);
        }
      }
      expect(
        find.text(
          'This meal conflicts with a rule, restriction or time limit.',
        ),
        findsNothing,
      );
      await tester.drag(find.byType(Scrollable).first, const Offset(0, -300));
      await tester.pump(const Duration(milliseconds: 200));
    }
    expect(checked, hasLength(slots));
  });

  journey('J7 pantry mark, add anyway, own items, start over', (tester) async {
    const soup = 'Black Bean and Lentil Soup';
    Finder inHave(String name) => find.descendant(
      of: find.byType(ExpansionTile),
      matching: find.text(name),
    );
    Future<void> expandHave() async {
      final heading = find.textContaining('Already have (');
      await scrollUntilVisible(tester, heading);
      await tapAndWait(tester, heading, inHave('garlic'));
      // The expansion animates; positions read before it ends are stale.
      await tester.pump(const Duration(milliseconds: 600));
    }

    await freshApp(tester);
    await toPlan(tester);

    await clearSnackBars(tester);
    await tapAndWait(tester, tab('Pantry'), title('My Shelves'));
    await tester.enterText(field('Search'), 'garlic');
    // `find.text` also matches the search field's own text, so the row is found by its InkWell.
    final garlic = find.widgetWithText(InkWell, 'garlic');
    await tapAndWait(
      tester,
      garlic,
      find.descendant(of: garlic, matching: find.byIcon(Icons.check_box)),
    );

    await toPlan(tester);
    final cell = keyPrefix('cell:').first;
    await tapAndWait(
      tester,
      find.descendant(of: cell, matching: find.text('Add')),
      find.byType(BottomSheet),
    );
    await tapAndWait(
      tester,
      find.descendant(of: find.byType(BottomSheet), matching: find.text(soup)),
      find.descendant(of: cell, matching: find.text('$soup · 1×')),
    );

    await clearSnackBars(tester);
    await tapAndWait(tester, tab('Shopping'), title('Shopping'));
    await waitFor(tester, find.textContaining(' to buy · '));
    await scrollUntilVisible(tester, find.text('onion'));
    expect(inHave('onion'), findsNothing);
    await expandHave();
    await tapAndWait(
      tester,
      find.descendant(
        of: find.ancestor(
          of: inHave('garlic'),
          matching: find.byType(ListTile),
        ),
        matching: find.text('Add anyway'),
      ),
      find.text('onion'),
    );
    await waitGone(tester, inHave('garlic'));
    await scrollUntilVisible(tester, find.text('garlic'), delta: -300);

    await addItem(tester, 'Journey batteries');
    await addItem(tester, 'Journey foil');
    final batteries = find.widgetWithText(
      CheckboxListTile,
      'Journey batteries',
    );
    await tester.tap(batteries);
    await _until(
      tester,
      () => tester.widget<CheckboxListTile>(batteries).value ?? false,
      'batteries checked',
    );

    await clearSnackBars(tester);
    await tapAndWait(tester, find.byTooltip('More'), find.text('Start over'));
    await tapAndWait(
      tester,
      find.text('Start over').last,
      find.text('Start over?'),
    );
    await tapAndWait(
      tester,
      find.widgetWithText(FilledButton, 'Start over'),
      find.text('Journey foil'),
    );
    await waitGone(tester, find.text('Journey batteries'));
    // Foil is on screen behind the dialog before the tap, so only this proves it survived.
    expect(
      find.widgetWithText(CheckboxListTile, 'Journey foil'),
      findsOneWidget,
      reason: 'unchecked own items survive Start over',
    );
    if (!_present(inHave('garlic'))) await expandHave();
    expect(inHave('garlic'), findsOneWidget);
  });

  journey('J8 rename and authored recipe survive a cold restart', (
    tester,
  ) async {
    await freshApp(tester);
    await tapAndWait(
      tester,
      find.byKey(const ValueKey('cover:accept')),
      find.text('Plan written.'),
    );
    await idle(tester);

    await rename(tester, 'Journey House');
    await authorStew(tester);
    final count = container(tester).read(recipeLibraryProvider).value!.length;
    await toPlan(tester);
    final firstMeal = find.descendant(
      of: keyPrefix('cell:').first,
      matching: find.textContaining(' · 1×'),
    );
    await waitFor(tester, firstMeal);
    final meal = tester.widget<Text>(firstMeal).data;

    // A process restart, as far as the app can tell: the connection is dropped and a new
    // provider tree opens kimatta.db again.
    await openDatabase(
      dbPath: '$_support${Platform.pathSeparator}journeys-scratch.db',
    );
    await tester.pumpWidget(
      ProviderScope(
        key: UniqueKey(),
        child: App(key: UniqueKey()),
      ),
    );
    await waitFor(tester, title('Plan'), timeout: const Duration(seconds: 45));
    await waitFor(
      tester,
      find.descendant(of: keyPrefix('cell:').first, matching: find.text(meal!)),
    );
    expect(title('Cover My Week'), findsNothing);
    expect(find.text(plannerEmptyCopy), findsNothing);

    await tapAndWait(tester, tab('Settings'), title('Settings'));
    await waitFor(tester, find.textContaining('Journey House · '));
    await tapAndWait(tester, tab('Recipes'), title('Recipes'));
    await waitFor(tester, find.text(stew));
    await _until(
      tester,
      () =>
          container(tester).read(recipeLibraryProvider).valueOrNull?.length ==
          count,
      'the library at $count recipes',
    );
  });

  journey('J9 restore the latest export over later changes', (tester) async {
    await freshApp(tester);
    await tapAndWait(
      tester,
      find.byKey(const ValueKey('cover:accept')),
      find.text('Plan written.'),
    );
    await idle(tester);
    await rename(tester, 'Before');
    // The app's own export path and name; the button itself only adds the share sheet.
    await container(tester).read(backupActionsProvider).export();
    await rename(tester, 'After');
    await authorStew(tester);

    await clearSnackBars(tester);
    await tapAndWait(tester, tab('Settings'), title('Settings'));
    final restore = find.text('Restore latest export');
    await scrollUntilVisible(tester, restore);
    await tapAndWait(tester, restore, find.text('Replace my data'));
    await tapAndWait(
      tester,
      find.text('Replace my data').last,
      find.text('Restore complete.'),
    );
    await scrollUntilVisible(
      tester,
      find.textContaining('Before · '),
      delta: -300,
    );
    expect(find.textContaining('After · '), findsNothing);
    expect(
      File('$_support${Platform.pathSeparator}kimatta.db.pre-restore')
          .existsSync(),
      isTrue,
    );

    await clearSnackBars(tester);
    await tapAndWait(tester, tab('Recipes'), title('Recipes'));
    await waitFor(tester, find.text('Baked Turkey Meatballs'));
    expect(find.text(stew), findsNothing);
  });

  journey('J10 shares without a link or to a refused host', (tester) async {
    await freshApp(tester);
    await toPlan(tester);

    await share(tester, 'nothing to see here');
    await waitFor(tester, find.text('No recipe link found in what was shared'));
    expect(title('Plan'), findsOneWidget);
    await clearSnackBars(tester);

    // Refused by `guard_url` before any I/O, so the emulator's network does not matter.
    await share(tester, 'Try this: https://localhost/recipe.');
    await waitFor(tester, title('Import a recipe'));
    await waitFor(tester, find.text("That link can't be imported."));
    expect(find.text('Retry'), findsNothing);
    await tapAndWait(
      tester,
      find.text('Try another link'),
      field('Recipe link'),
    );
  });

  // Never from a tab root: there `handlePopRoute` closes the app, and the run with it.
  journey('J11 system back from Cover, a detail, a dialog and a sheet', (
    tester,
  ) async {
    await freshApp(tester);
    await tester.binding.handlePopRoute();
    await waitFor(tester, title('Plan'));
    await waitFor(tester, keyPrefix('cell:'));

    await tapAndWait(tester, tab('Recipes'), title('Recipes'));
    const meal = 'Baked Turkey Meatballs';
    await tapAndWait(tester, find.text(meal), title(meal));
    await tester.binding.handlePopRoute();
    await waitFor(tester, title('Recipes'));

    await tapAndWait(tester, tab('Shopping'), title('Shopping'));
    await addItem(tester, 'Journey batteries');
    final batteries = find.widgetWithText(
      CheckboxListTile,
      'Journey batteries',
    );
    await tester.tap(batteries);
    await _until(
      tester,
      () => tester.widget<CheckboxListTile>(batteries).value ?? false,
      'batteries checked',
    );
    await clearSnackBars(tester);
    await tapAndWait(tester, find.byTooltip('More'), find.text('Start over'));
    await tapAndWait(
      tester,
      find.text('Start over').last,
      find.text('Start over?'),
    );
    await tester.binding.handlePopRoute();
    await waitGone(tester, find.text('Start over?'));
    await tester.pump(const Duration(seconds: 1));
    expect(title('Shopping'), findsOneWidget);
    expect(tester.widget<CheckboxListTile>(batteries).value, isTrue);

    await toPlan(tester);
    await tapAndWait(
      tester,
      find.descendant(of: keyPrefix('cell:').first, matching: find.text('Add')),
      find.byType(BottomSheet),
    );
    await tester.binding.handlePopRoute();
    await waitGone(tester, find.byType(BottomSheet));
    expect(title('Plan'), findsOneWidget);
    expect(find.text(plannerEmptyCopy), findsOneWidget);
  });
}
