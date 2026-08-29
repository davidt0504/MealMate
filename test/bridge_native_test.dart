import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/api/household.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/restrictions.dart';
import 'package:meal_mate/src/rust/api/starter.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';
import 'package:meal_mate/features/recipes/recipe_fields.dart';
import 'package:meal_mate/features/restrictions/restriction_copy.dart';

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

  test('open_database migrates a real database to schema v7', () async {
    final report = await openDatabase(dbPath: await tempDb());
    expect(report.schemaVersion, 7);
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

  test('onboarding completes once and survives reopening the file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    expect(h.onboarded, isFalse);
    expect((await completeOnboarding(householdId: h.id)).onboarded, isTrue);
    // Idempotent: a second tap is a no-op write, not an error.
    expect((await completeOnboarding(householdId: h.id)).onboarded, isTrue);
    await openDatabase(dbPath: path);
    expect((await bootstrapHousehold()).onboarded, isTrue);
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

  test('starter content installs once and is idempotent', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final first = await installStarterContent(householdId: h.id);
    // Empty by design, and the report says so rather than leaving `installed: 0`
    // indistinguishable from a swallowed failure (MVP-011 AC-3).
    expect(first.catalogInstalled, greaterThan(0));
    expect(first.installed, first.available);
    expect(first.pendingCookReview, greaterThan(0));

    final second = await installStarterContent(householdId: h.id);
    expect(second.catalogInstalled, 0);
    expect(second.installed, 0);
    expect(second.available, first.available);
    expect(second.pendingCookReview, first.pendingCookReview);
  });

  test('installed starter content survives reopening the same file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    final first = await installStarterContent(householdId: h.id);

    await openDatabase(dbPath: path);
    final again = await installStarterContent(householdId: h.id);
    // Nothing is re-seeded, which is only observable because the catalog persisted.
    expect(again.catalogInstalled, 0);
    expect(again.installed, 0);
    expect(first.catalogInstalled, greaterThan(0));
  });

  test(
    'installing for an unknown household is a typed Storage error',
    () async {
      await openDatabase(dbPath: await tempDb());
      await bootstrapHousehold();
      await expectLater(
        () => installStarterContent(householdId: 'not-a-household'),
        throwsA(isA<KimattaError_Storage>()),
      );
    },
  );

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

  test('a saved restriction set survives reopening the same file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    expect(await loadRestrictions(householdId: h.id), isEmpty);
    const sent = [
      RestrictionDto.known(kind: 'peanuts'),
      RestrictionDto.other(text: 'nightshades'),
    ];
    expect(await saveRestrictions(householdId: h.id, restrictions: sent), sent);
    await openDatabase(dbPath: path);
    expect(await loadRestrictions(householdId: h.id), sent);
  });

  test('an empty save clears the restriction set', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await saveRestrictions(
      householdId: h.id,
      restrictions: const [RestrictionDto.known(kind: 'dairy')],
    );
    expect(
      await saveRestrictions(householdId: h.id, restrictions: const []),
      isEmpty,
    );
    expect(await loadRestrictions(householdId: h.id), isEmpty);
  });

  test('an unknown restriction kind is a typed Restriction error', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await expectLater(
      () => saveRestrictions(
        householdId: h.id,
        restrictions: const [RestrictionDto.known(kind: 'nightshades')],
      ),
      throwsA(isA<KimattaError_Restriction>()),
    );
    expect(await loadRestrictions(householdId: h.id), isEmpty);
  });

  // Buys back the compile-time exhaustiveness the token-string DTO costs (the `UnitDto`
  // convention `rust/src/api/recipe.rs:26-27` records). A kind added in Rust without a Dart
  // label fails here rather than rendering `tree_nuts` to a user.
  test('every known restriction kind has a Dart label', () async {
    final kinds = await knownRestrictionKinds();
    expect(kinds, isNotEmpty);
    for (final kind in kinds) {
      expect(
        restrictionLabel(kind),
        isNot(kind),
        reason: '$kind has no label in restrictionLabel',
      );
      expect(
        describeRestriction(RestrictionDto.known(kind: kind)),
        restrictionLabel(kind),
      );
    }
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

  test('archiving hides a recipe and restoring brings it back', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final saved = await saveRecipe(recipe: recipeFor(h.id, const []));
    expect(saved.archivedAt, isNull);
    final archived = await archiveRecipe(
      householdId: h.id,
      recipeId: saved.id,
      archivedOn: '2026-08-29',
    );
    expect(archived.archivedAt, '2026-08-29');
    expect(await listRecipes(householdId: h.id), isEmpty);
    final gone = await listArchivedRecipes(householdId: h.id);
    expect(gone.single.id, saved.id);
    // Still resolvable by id: the reference-stability half of the archive policy.
    final loaded = await loadRecipe(householdId: h.id, recipeId: saved.id);
    expect(loaded?.archivedAt, '2026-08-29');
    final restored = await restoreRecipe(householdId: h.id, recipeId: saved.id);
    expect(restored.archivedAt, isNull);
    expect((await listRecipes(householdId: h.id)).single.id, saved.id);
    expect(await listArchivedRecipes(householdId: h.id), isEmpty);
  });

  test('an archived recipe survives reopening the same file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    final saved = await saveRecipe(recipe: recipeFor(h.id, const []));
    await archiveRecipe(
      householdId: h.id,
      recipeId: saved.id,
      archivedOn: '2026-08-29',
    );
    await openDatabase(dbPath: path);
    expect(await listRecipes(householdId: h.id), isEmpty);
    expect((await listArchivedRecipes(householdId: h.id)).single.id, saved.id);
    final again = await loadRecipe(householdId: h.id, recipeId: saved.id);
    expect(again?.archivedAt, '2026-08-29');
  });

  // MVP-009 AC-3, across the bridge: the assessment follows the stored restriction set.
  test('a saved recipe reports its restriction assessment and it follows the restriction set', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await saveRestrictions(
      householdId: h.id,
      restrictions: const [RestrictionDto.known(kind: 'dairy')],
    );
    final saved = await saveRecipe(
      recipe: recipeFor(h.id, const [
        IngredientLineDto(
          originalText: '1 tbsp butter',
          name: 'butter',
          quantity: QuantityDto.unknown(),
          unit: UnitDto.none(),
          optional: false,
        ),
      ]),
    );
    final a = saved.assessment!;
    expect(a.restrictionsChecked, 1);
    expect(a.linesChecked, 1);
    expect(a.conflicts.single.term, 'butter');
    expect(a.conflicts.single.lineName, 'butter');
    expect(a.conflicts.single.linePosition, 0);
    expect(
      a.conflicts.single.restriction,
      const RestrictionDto.known(kind: 'dairy'),
    );
    final listed = await listRecipes(householdId: h.id);
    expect(listed.single.assessment.conflicts.single.term, 'butter');
    await saveRestrictions(householdId: h.id, restrictions: const []);
    final again = await loadRecipe(householdId: h.id, recipeId: saved.id);
    expect(again!.assessment!.conflicts, isEmpty);
    expect(again.assessment!.restrictionsChecked, 0);
    expect(
      (await listRecipes(householdId: h.id)).single.assessment.conflicts,
      isEmpty,
    );
  });

  test('a non-civil archive date is a typed Planning error', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final saved = await saveRecipe(recipe: recipeFor(h.id, const []));
    await expectLater(
      () => archiveRecipe(
        householdId: h.id,
        recipeId: saved.id,
        archivedOn: '2026-08-29T23:00:00Z',
      ),
      throwsA(isA<KimattaError_Planning>()),
    );
    expect((await listRecipes(householdId: h.id)).single.id, saved.id);
  });

  // The `1.5` the Dart parser sends as 15/10 is stored canonically as 3/2.
  test('a decimal quantity is stored as its lowest-terms rational', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final saved = await saveRecipe(
      recipe: recipeFor(h.id, const [
        IngredientLineDto(
          originalText: '1.5 cups milk',
          name: 'milk',
          quantity: QuantityDto.exact(numer: 15, denom: 10),
          unit: UnitDto.known(unit: 'cup'),
          optional: false,
        ),
      ]),
    );
    expect(
      saved.lines.single.quantity,
      const QuantityDto.exact(numer: 3, denom: 2),
    );
  });

  // The unit counterpart of the restriction cross-check above. Labels may equal their token
  // (`cup`), so the assertion is on the hard-coded vocabulary and on no raw underscore
  // reaching a user, not on label ≠ token.
  test('every known unit kind has a Dart label', () async {
    final kinds = await knownUnitKinds();
    expect(kinds, [
      'tsp',
      'tbsp',
      'cup',
      'fl_oz',
      'ml',
      'l',
      'g',
      'kg',
      'oz',
      'lb',
      'piece',
    ]);
    for (final kind in kinds) {
      expect(unitLabel(kind), isNotEmpty, reason: '$kind has no label');
      expect(unitLabel(kind), isNot(contains('_')), reason: '$kind is raw');
    }
  });

  // MVP-012: expected-to-pass pins of the planned-meal bridge on a real file. The generated
  // `PlannedMealDto.==` compares `components` by list identity, so assertions go field by
  // field with a deep-list matcher, as the recipe tests do.
  Future<PlannedMealDto> plannedDinner(String householdId, String date) async {
    await ensurePlanningCycle(
      householdId: householdId,
      defaultAnchorDate: date,
    );
    final recipe = await saveRecipe(recipe: recipeFor(householdId, const []));
    return savePlannedMeal(
      meal: PlannedMealDto(
        id: '',
        householdId: householdId,
        date: date,
        slot: MealSlotDto.dinner,
        components: [
          MealComponentDto(
            kind: 'recipe',
            recipeId: recipe.id,
            scale: const ScaleDto(numer: 3, denom: 2),
          ),
          const MealComponentDto(kind: 'leftovers', note: ' chili '),
        ],
        locked: false,
      ),
    );
  }

  test(
    'a planned meal with two components survives reopening the same file',
    () async {
      final path = await tempDb();
      await openDatabase(dbPath: path);
      final h = await bootstrapHousehold();
      final saved = await plannedDinner(h.id, '2026-08-30');
      expect(saved.id, isNotEmpty);
      expect(saved.locked, isFalse);
      await openDatabase(dbPath: path);
      final again = await loadPlannedMeal(householdId: h.id, mealId: saved.id);
      expect(again, isNotNull);
      expect(again!.date, '2026-08-30');
      expect(again.slot, MealSlotDto.dinner);
      expect(again.components, hasLength(2));
      expect(again.components[0].kind, 'recipe');
      expect(again.components[0].recipeId, saved.components[0].recipeId);
      expect(again.components[0].scale, const ScaleDto(numer: 3, denom: 2));
      expect(again.components[1].kind, 'leftovers');
      expect(again.components[1].note, ' chili ');
      expect(again.components[1].recipeId, isNull);
      final listed = await listPlannedMeals(
        householdId: h.id,
        fromDate: '2026-08-30',
        toDate: '2026-08-30',
      );
      expect(listed.single.id, saved.id);
    },
  );

  test('a locked planned meal stays locked across save', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final saved = await plannedDinner(h.id, '2026-08-30');
    final locked = await setPlannedMealLock(
      householdId: h.id,
      mealId: saved.id,
      locked: true,
    );
    expect(locked.locked, isTrue);
    final resaved = await savePlannedMeal(
      meal: PlannedMealDto(
        id: saved.id,
        householdId: h.id,
        date: '2026-08-30',
        slot: MealSlotDto.dinner,
        components: const [MealComponentDto(kind: 'dining_out')],
        locked: false,
      ),
    );
    expect(resaved.locked, isTrue);
    expect(resaved.components.single.kind, 'dining_out');
  });

  test('a planned meal id from a different household is not found', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    final saved = await plannedDinner(h.id, '2026-08-30');
    expect(
      await loadPlannedMeal(householdId: 'not-${h.id}', mealId: saved.id),
      isNull,
    );
    expect(
      await listPlannedMeals(
        householdId: 'not-${h.id}',
        fromDate: '2026-01-01',
        toDate: '2026-12-31',
      ),
      isEmpty,
    );
    await expectLater(
      () => deletePlannedMeal(householdId: 'not-${h.id}', mealId: saved.id),
      throwsA(isA<KimattaError_Storage>()),
    );
    expect(
      await loadPlannedMeal(householdId: h.id, mealId: saved.id),
      isNotNull,
    );
  });

  test('a PlannedMeal error is a typed KimattaError_PlannedMeal', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await ensurePlanningCycle(
      householdId: h.id,
      defaultAnchorDate: '2026-08-30',
    );
    await expectLater(
      () => savePlannedMeal(
        meal: PlannedMealDto(
          id: '',
          householdId: h.id,
          date: '2026-08-30',
          slot: MealSlotDto.dinner,
          components: const [
            MealComponentDto(kind: 'open'),
            MealComponentDto(kind: 'leftovers'),
          ],
          locked: false,
        ),
      ),
      throwsA(isA<KimattaError_PlannedMeal>()),
    );
    expect(
      await listPlannedMeals(
        householdId: h.id,
        fromDate: '2026-08-30',
        toDate: '2026-08-30',
      ),
      isEmpty,
    );
    expect(await knownMealComponentKinds(), [
      'recipe',
      'leftovers',
      'dining_out',
      'frozen_quick',
      'freeform',
      'open',
    ]);
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
