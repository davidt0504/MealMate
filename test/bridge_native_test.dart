import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/api/household.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';

void main() {
  setUpAll(() async => RustLib.init());

  Future<String> tempDb() async {
    final dir = await Directory.systemTemp.createTemp('kimatta-test');
    addTearDown(() => dir.delete(recursive: true));
    return '${dir.path}${Platform.pathSeparator}k.db';
  }

  // Declared first on purpose: the connection is process-wide, so this must run
  // before any openDatabase call in this file.
  test('bootstrap_household before open_database is NotOpen', () async {
    await expectLater(bootstrapHousehold, throwsA(isA<KimattaError_NotOpen>()));
  });

  test('core_version crosses the bridge', () {
    expect(coreVersion(), '0.1.0');
  });

  test('typed Rust error is matchable in Dart', () async {
    await expectLater(
      () => openDatabase(dbPath: '   '),
      throwsA(isA<KimattaError_InvalidPath>()),
    );
  });

  test('open_database migrates a real database to schema v3', () async {
    final report = await openDatabase(dbPath: await tempDb());
    expect(report.schemaVersion, 3);
  });

  test('storage failure surfaces as KimattaError_Storage', () async {
    final missing =
        '${Directory.systemTemp.path}${Platform.pathSeparator}'
        'kimatta-absent-${DateTime.now().microsecondsSinceEpoch}'
        '${Platform.pathSeparator}k.db';
    await expectLater(
      () => openDatabase(dbPath: missing),
      throwsA(isA<KimattaError_Storage>()),
    );
  });

  test('bootstrap creates one household once and reuses it', () async {
    await openDatabase(dbPath: await tempDb());
    final first = await bootstrapHousehold();
    final second = await bootstrapHousehold();
    expect(first.id, isNotEmpty);
    expect(first.name, isNull);
    expect(first.members.single.displayName, 'Me');
    expect(second.id, first.id);
    expect(second.members.single.id, first.members.single.id);
  });

  test('reopening the same file keeps the household (restart)', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final first = await bootstrapHousehold();
    await openDatabase(dbPath: path);
    final again = await bootstrapHousehold();
    expect(again.id, first.id);
    expect(again.members.length, 1);
  });

  test('rename persists and blank clears', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final named = await renameHousehold(householdId: h.id, name: ' Casa ');
    expect(named.name, 'Casa');
    expect((await bootstrapHousehold()).name, 'Casa');
    final cleared = await renameHousehold(householdId: h.id, name: '  ');
    expect(cleared.name, isNull);
  });

  test('rename of a foreign id is rejected and changes nothing', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await expectLater(
      () => renameHousehold(householdId: 'not-${h.id}', name: 'x'),
      throwsA(isA<KimattaError_Storage>()),
    );
    expect((await bootstrapHousehold()).name, isNull);
  });

  test(
    'ensure_planning_cycle returns the dinner-only default across the bridge',
    () async {
      await openDatabase(dbPath: await tempDb());
      final h = await bootstrapHousehold();
      final c = await ensurePlanningCycle(
        householdId: h.id,
        defaultAnchorDate: '2026-08-29',
      );
      expect(c.lengthDays, 7);
      expect(c.mealSlots, [MealSlotDto.dinner]);
      expect(c.dates.length, 7);
      expect(c.dates.first, '2026-08-29');
      // The month-boundary case, end to end.
      expect(c.dates.last, '2026-09-04');
    },
  );

  test('a saved cycle survives reopening the same file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    await savePlanningCycle(
      householdId: h.id,
      anchorDate: '2026-01-05',
      lengthDays: 14,
      mealSlots: [MealSlotDto.breakfast, MealSlotDto.dinner],
    );
    await openDatabase(dbPath: path);
    final again = await ensurePlanningCycle(
      householdId: h.id,
      defaultAnchorDate: '2099-12-01',
    );
    expect(again.anchorDate, '2026-01-05');
    expect(again.lengthDays, 14);
    expect(again.mealSlots, [MealSlotDto.breakfast, MealSlotDto.dinner]);
  });

  test('an invalid anchor date is a typed Planning error in Dart', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await expectLater(
      () => ensurePlanningCycle(
        householdId: h.id,
        defaultAnchorDate: '2026-08-29T23:00:00Z',
      ),
      throwsA(isA<KimattaError_Planning>()),
    );
  });

  test('a different file is a different household', () async {
    await openDatabase(dbPath: await tempDb());
    final a = await bootstrapHousehold();
    await openDatabase(dbPath: await tempDb());
    final b = await bootstrapHousehold();
    expect(b.id, isNot(a.id));
  });

  RecipeDto recipeFor(String householdId, List<IngredientLineDto> lines) {
    return RecipeDto(
      id: '',
      householdId: householdId,
      title: 'Pancakes',
      servings: 4,
      instructions: 'Mix. Fry.',
      lines: lines,
      provenance: const RecipeProvenanceDto(kind: 'authored'),
    );
  }

  test(
    'a recipe with structured and original lines round-trips across the bridge',
    () async {
      await openDatabase(dbPath: await tempDb());
      final h = await bootstrapHousehold();
      final custom = await addCustomIngredient(
        item: CustomIngredientDto(
          id: '',
          householdId: h.id,
          name: "nana's mix",
        ),
      );
      final saved = await saveRecipe(
        recipe: recipeFor(h.id, [
          const IngredientLineDto(
            originalText: '  1/2 cup Flour, sifted ',
            name: 'Flour',
            quantity: QuantityDto.exact(numer: 1, denom: 2),
            unit: UnitDto.known(unit: 'cup'),
            preparation: 'sifted',
            optional: false,
          ),
          IngredientLineDto(
            originalText: "2-3 handfuls nana's mix (optional)",
            name: "nana's mix",
            ingredient: IngredientRefDto.custom(id: custom.id),
            quantity: const QuantityDto.range(
              minNumer: 2,
              minDenom: 1,
              maxNumer: 3,
              maxDenom: 1,
            ),
            unit: const UnitDto.other(text: 'handful'),
            optional: true,
          ),
          const IngredientLineDto(
            originalText: 'a splash of something',
            name: 'something',
            quantity: QuantityDto.unknown(),
            unit: UnitDto.none(),
            optional: false,
          ),
        ]),
      );
      expect(saved.id, isNotEmpty);
      final loaded = await loadRecipe(householdId: h.id, recipeId: saved.id);
      expect(loaded, isNotNull);
      expect(loaded!.title, 'Pancakes');
      expect(loaded.servings, 4);
      expect(loaded.instructions, 'Mix. Fry.');
      expect(loaded.provenance.kind, 'authored');
      expect(loaded.lines.length, 3);
      expect(loaded.lines[0].originalText, '  1/2 cup Flour, sifted ');
      expect(
        loaded.lines[0].quantity,
        const QuantityDto.exact(numer: 1, denom: 2),
      );
      expect(loaded.lines[0].unit, const UnitDto.known(unit: 'cup'));
      expect(loaded.lines[0].preparation, 'sifted');
      expect(loaded.lines[0].optional, isFalse);
      expect(
        loaded.lines[1].ingredient,
        IngredientRefDto.custom(id: custom.id),
      );
      expect(
        loaded.lines[1].quantity,
        const QuantityDto.range(
          minNumer: 2,
          minDenom: 1,
          maxNumer: 3,
          maxDenom: 1,
        ),
      );
      expect(loaded.lines[1].unit, const UnitDto.other(text: 'handful'));
      expect(loaded.lines[1].optional, isTrue);
      expect(loaded.lines[2].quantity, const QuantityDto.unknown());
      expect(loaded.lines[2].unit, const UnitDto.none());
      expect(loaded.lines[2].ingredient, isNull);
      final list = await listRecipes(householdId: h.id);
      expect(list.single.id, saved.id);
      expect(list.single.title, 'Pancakes');
    },
  );

  test('a saved recipe survives reopening the same file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    final saved = await saveRecipe(
      recipe: recipeFor(h.id, const [
        IngredientLineDto(
          originalText: '2 eggs',
          name: 'eggs',
          quantity: QuantityDto.exact(numer: 2, denom: 1),
          unit: UnitDto.known(unit: 'piece'),
          optional: false,
        ),
      ]),
    );
    await openDatabase(dbPath: path);
    final again = await loadRecipe(householdId: h.id, recipeId: saved.id);
    expect(again, isNotNull);
    expect(again!.lines.single.originalText, '2 eggs');
    expect(again.lines.single.unit, const UnitDto.known(unit: 'piece'));
  });

  test('a blank title is a typed Recipe error in Dart', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await expectLater(
      () => saveRecipe(
        recipe: RecipeDto(
          id: '',
          householdId: h.id,
          title: '   ',
          instructions: '',
          lines: const [],
          provenance: const RecipeProvenanceDto(kind: 'authored'),
        ),
      ),
      throwsA(isA<KimattaError_Recipe>()),
    );
    expect(await listRecipes(householdId: h.id), isEmpty);
  });

  test('a recipe id from a different household is not found', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final saved = await saveRecipe(recipe: recipeFor(h.id, const []));
    expect(
      await loadRecipe(householdId: 'not-${h.id}', recipeId: saved.id),
      isNull,
    );
    expect(await listRecipes(householdId: 'not-${h.id}'), isEmpty);
    expect(await loadRecipe(householdId: h.id, recipeId: saved.id), isNotNull);
  });

  test('custom ingredients list only for their household', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await addCustomIngredient(
      item: CustomIngredientDto(id: '', householdId: h.id, name: 'mix'),
    );
    expect((await listCustomIngredients(householdId: h.id)).single.name, 'mix');
    expect(await listCustomIngredients(householdId: 'not-${h.id}'), isEmpty);
  });
}
