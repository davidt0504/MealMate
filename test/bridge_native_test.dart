import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

import 'package:meal_mate/src/rust/api/decisions.dart';
import 'package:meal_mate/src/rust/api/error.dart';
import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/api/household.dart';
import 'package:meal_mate/src/rust/api/pantry.dart';
import 'package:meal_mate/src/rust/api/planned_meals.dart';
import 'package:meal_mate/src/rust/api/planner.dart';
import 'package:meal_mate/src/rust/api/planning.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/api/restrictions.dart';
import 'package:meal_mate/src/rust/api/shopping.dart';
import 'package:meal_mate/src/rust/api/starter.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';
import 'package:meal_mate/features/household/household_screen.dart'
    show describeFailure;
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

  test('open_database migrates a real database to schema v10', () async {
    final report = await openDatabase(dbPath: await tempDb());
    expect(report.schemaVersion, 10);
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

  test('an export is an openable, versioned copy of the live data', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    final dest =
        '${File(path).parent.path}${Platform.pathSeparator}exports'
        '${Platform.pathSeparator}kimatta-export.db';
    final report = await exportDatabase(destPath: dest);
    expect(report.path, dest);
    expect(report.schemaVersion, 10);
    // Opening the export as the live database proves it is the database's own
    // format, not a write-only artifact.
    final opened = await openDatabase(dbPath: dest);
    expect(opened.schemaVersion, 10);
    expect((await bootstrapHousehold()).id, h.id);
  });

  test('a restore returns the exported content (AC-4)', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    await renameHousehold(householdId: h.id, name: 'Casa');
    final dest = '${File(path).parent.path}${Platform.pathSeparator}export.db';
    final report = await exportDatabase(destPath: dest);
    await renameHousehold(householdId: h.id, name: 'Mutated');

    final restored = await restoreDatabase(exportPath: dest, dbPath: path);
    expect(restored.schemaVersion, report.schemaVersion);
    expect((await bootstrapHousehold()).name, 'Casa');
    // The overwritten database is kept aside, not destroyed.
    expect(File('$path.pre-restore').existsSync(), isTrue);
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
    // The report distinguishes an install that wrote nothing from a swallowed failure. Since
    // MVP-032 a first install writes the federal entries, so `installed == available` is the
    // shipped count rather than 0 == 0, and `pendingCookReview` is the `original` remainder.
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

  /// AC-1 at the real bridge: the mark is durable state in the file, not process state.
  test('a marked pantry item survives reopening the same file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    await installStarterContent(householdId: h.id);
    final first = (await listPantry(householdId: h.id)).first;
    expect(first.marked, isFalse);
    final stored = await setPantryMark(
      householdId: h.id,
      ingredient: first.ingredient,
      marked: true,
    );
    expect(stored.marked, isTrue);

    await openDatabase(dbPath: path);
    final again = (await listPantry(householdId: h.id))
        .where((e) => e.ingredient == first.ingredient)
        .single;
    expect(again.marked, isTrue);
    expect(again.name, first.name);
  });

  /// MVP-015 at the real bridge, and the only place the generated shopping decoders are ever
  /// executed: `flutter_rust_bridge_codegen generate` proves codegen ran, not that
  /// `Option<SeparateReasonDto>` and `Option<ScaleDto>` round-trip. Expected-to-pass — the
  /// command already exists, so this cannot fail first; it is non-vacuous because it is the
  /// only Dart caller of `deriveShoppingList` in the tree.
  test(
    'a derived shopping list groups a resolved line and omits a marked one',
    () async {
      await openDatabase(dbPath: await tempDb());
      final h = await bootstrapHousehold();
      await installStarterContent(householdId: h.id);
      final catalog = await listPantry(householdId: h.id);
      final a = catalog[0]; // stays Needed
      final b = catalog[1]; // marked below, so omitted
      await ensurePlanningCycle(
        householdId: h.id,
        defaultAnchorDate: '2026-08-30',
      );
      final saved = await saveRecipe(
        recipe: recipeFor(h.id, [
          IngredientLineDto(
            originalText: '2 cup ${a.name}',
            name: a.name,
            ingredient: a.ingredient,
            quantity: const QuantityDto.exact(numer: 2, denom: 1),
            unit: const UnitDto.known(unit: 'cup'),
            optional: false,
          ),
          IngredientLineDto(
            originalText: '1 piece ${b.name}',
            name: b.name,
            ingredient: b.ingredient,
            quantity: const QuantityDto.exact(numer: 1, denom: 1),
            unit: const UnitDto.known(unit: 'piece'),
            optional: false,
          ),
          const IngredientLineDto(
            originalText: '1 piece mystery',
            name: 'mystery',
            quantity: QuantityDto.exact(numer: 1, denom: 1),
            unit: UnitDto.known(unit: 'piece'),
            optional: false,
          ),
        ]),
      );
      // Two recipe components of the same recipe, one scaled and one as-written, so both
      // arms of the `Option<ScaleDto>` decoder carry a value on the way back.
      await savePlannedMeal(
        meal: PlannedMealDto(
          id: '',
          householdId: h.id,
          date: '2026-08-30',
          slot: MealSlotDto.dinner,
          locked: false,
          components: [
            MealComponentDto(
              kind: 'recipe',
              recipeId: saved.id,
              scale: const ScaleDto(numer: 2, denom: 1),
            ),
            MealComponentDto(kind: 'recipe', recipeId: saved.id),
          ],
        ),
      );
      await setPantryMark(
        householdId: h.id,
        ingredient: b.ingredient,
        marked: true,
      );

      final list = await deriveShoppingList(
        householdId: h.id,
        fromDate: '2026-08-30',
        toDate: '2026-08-30',
      );
      // Lines are nested inside store-category groups, and the unresolved ones are
      // uncategorised, so they are guaranteed to sit in a different group from a and b.
      final lines = [for (final g in list.groups) ...g.lines];
      expect(list.algorithmVersion, greaterThan(0));
      expect(list.contributionCount, 6);
      expect(lines, hasLength(4));
      expect(
        lines.fold<int>(0, (n, l) => n + l.contributions.length),
        6,
        reason: 'every contribution is accounted for',
      );

      final merged = lines.firstWhere((l) => l.ingredient == a.ingredient);
      expect(merged.key, startsWith('m:catalog:'));
      expect(merged.quantity, const QuantityDto.exact(numer: 6, denom: 1));
      expect(merged.unit, const UnitDto.known(unit: 'cup'));
      expect(merged.status, ShoppingLineStatusDto.needed);
      // Decode branch 1: a null `Option<SeparateReasonDto>`, the common case.
      expect(merged.separateReason, isNull);
      final scales = merged.contributions.map((c) => c.scale).toList();
      expect(scales, hasLength(2));
      // Decode branches 2 and 3: `Option<ScaleDto>` both ways, inside a nested list.
      expect(scales.where((s) => s == null), hasLength(1));
      expect(
        scales.where((s) => s == const ScaleDto(numer: 2, denom: 1)),
        hasLength(1),
      );

      final omitted = lines.firstWhere((l) => l.ingredient == b.ingredient);
      expect(omitted.status, ShoppingLineStatusDto.omittedPantryMarked);
      expect(
        omitted.quantity,
        const QuantityDto.exact(numer: 3, denom: 1),
        reason: 'a mark suppresses a purchase without discarding the amount',
      );

      final unresolved = lines.where((l) => l.ingredient == null).toList();
      expect(unresolved, hasLength(2));
      for (final l in unresolved) {
        expect(l.key, startsWith('s:'));
        expect(l.name, 'mystery');
        // Decode branch 4: a present `Option<SeparateReasonDto>`.
        expect(l.separateReason, SeparateReasonDto.unresolved);
      }
    },
  );

  /// MVP-023 AC-8: one coarse command carries the whole cover-cycle operation — snapshot,
  /// assessment, plan, ledger row and apply — and its nested outcome decodes in Dart.
  test('a cover cycle outcome crosses the bridge', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await ensurePlanningCycle(
      householdId: h.id,
      defaultAnchorDate: '2026-08-30',
    );
    final saved = await saveRecipe(recipe: recipeFor(h.id, const []));
    final out = await coverCycle(
      request: CoverCycleRequestDto(
        householdId: h.id,
        today: '2026-08-30',
        offsetCycles: 0,
        apply: true,
      ),
    );
    expect(out.ledgerEntryId, isNotEmpty);
    expect(out.applied, isTrue);
    expect(out.changedSlots, 7);
    expect(out.result.algorithmVersion, 2);
    expect(out.result.snapshotHash.length, 16);
    expect(out.result.horizonFrom, '2026-08-30');
    expect(out.result.slots.length, 7, reason: 'seven dinner slots');
    // No restrictions were configured, so the plan may not claim `covered`.
    expect(out.result.status, OutcomeStatusDto.tentativelyCovered);
    expect(out.result.assumptions, contains('RESTRICTIONS_NOT_CONFIGURED'));
    expect(out.result.unresolvedIssues, isNotEmpty);
    expect(
      out.result.proposed.first.components.first.recipeId,
      saved.id,
      reason: 'the one recipe wins every slot over the fallbacks',
    );
    expect(out.result.proposals.single.actionType, 'apply_plan');
    expect(out.result.search.beamWidth, 8);
    expect(out.result.search.slotOrder, 'canonical');
    expect(out.result.scoreTiers.length, 6);
    final planned = await listPlannedMeals(
      householdId: h.id,
      fromDate: '2026-08-30',
      toDate: '2026-09-05',
    );
    expect(planned.length, 7);
    // A malformed today is the same typed error every other command raises.
    await expectLater(
      () => coverCycle(
        request: CoverCycleRequestDto(
          householdId: h.id,
          today: '2026-08-30T00:00:00Z',
          offsetCycles: 0,
          apply: false,
        ),
      ),
      throwsA(isA<KimattaError_Planning>()),
    );
  });

  /// MVP-024 AC-1/2/4 at the real bridge: preview → swap → apply preserves the swapped,
  /// locked slot; a veto is evidence the next run obeys; the reviewed marker clears the
  /// restrictions assumption. Every outcome's ledger entry id is the AC-4 evidence assert.
  test('plan decisions cross the bridge and steer the next cover', () async {
    await openDatabase(dbPath: await tempDb());
    final h = await bootstrapHousehold();
    await ensurePlanningCycle(
      householdId: h.id,
      defaultAnchorDate: '2026-08-30',
    );
    final pancakes = await saveRecipe(recipe: recipeFor(h.id, const []));
    final waffles = await saveRecipe(
      recipe: RecipeDto(
        id: '',
        householdId: h.id,
        title: 'Waffles',
        servings: 4,
        instructions: 'Mix. Bake.',
        lines: const [],
        provenance: const RecipeProvenanceDto(kind: 'authored'),
      ),
    );

    final preview = await coverCycle(
      request: CoverCycleRequestDto(
        householdId: h.id,
        today: '2026-08-30',
        offsetCycles: 0,
        apply: false,
      ),
    );
    expect(preview.ledgerEntryId, isNotEmpty);
    expect(preview.applied, isFalse);

    final swapped = await recordPlanDecision(
      request: PlanDecisionRequestDto(
        householdId: h.id,
        today: '2026-08-30',
        offsetCycles: 0,
        decision: PlanDecisionDto.swap(
          date: '2026-08-30',
          slot: MealSlotDto.dinner,
          components: [MealComponentDto(kind: 'recipe', recipeId: waffles.id)],
        ),
      ),
    );
    expect(swapped.ledgerEntryId, isNotEmpty);

    final applied = await coverCycle(
      request: CoverCycleRequestDto(
        householdId: h.id,
        today: '2026-08-30',
        offsetCycles: 0,
        apply: true,
      ),
    );
    expect(applied.applied, isTrue);
    final swappedSlot = applied.result.slots.firstWhere(
      (s) => s.date == '2026-08-30',
    );
    // Invariant 18 at the real bridge: the swapped slot is locked and unmoved.
    expect(swappedSlot.state, CoverageStateDto.lockedByUser);
    expect(swappedSlot.components.single.recipeId, waffles.id);

    final vetoed = await recordPlanDecision(
      request: PlanDecisionRequestDto(
        householdId: h.id,
        today: '2026-08-30',
        offsetCycles: 0,
        decision: const PlanDecisionDto.veto(
          subject: 'Pancakes',
          date: '2026-08-31',
          slot: MealSlotDto.dinner,
        ),
      ),
    );
    expect(vetoed.ledgerEntryId, isNotEmpty);
    final afterVeto = await coverCycle(
      request: CoverCycleRequestDto(
        householdId: h.id,
        today: '2026-08-30',
        offsetCycles: 0,
        apply: false,
      ),
    );
    expect(
      afterVeto.result.proposed.every(
        (p) => p.components.every((c) => c.recipeId != pancakes.id),
      ),
      isTrue,
      reason: 'the vetoed dish is Tier-0-rejected on the next run',
    );

    final reviewed = await recordPlanDecision(
      request: PlanDecisionRequestDto(
        householdId: h.id,
        today: '2026-08-30',
        offsetCycles: 0,
        decision: const PlanDecisionDto.restrictionsReviewed(),
      ),
    );
    expect(reviewed.ledgerEntryId, isNotEmpty);
    final afterReviewed = await coverCycle(
      request: CoverCycleRequestDto(
        householdId: h.id,
        today: '2026-08-30',
        offsetCycles: 0,
        apply: false,
      ),
    );
    expect(
      afterReviewed.result.assumptions,
      isNot(contains('RESTRICTIONS_NOT_CONFIGURED')),
      reason: 'reviewed absence is confirmed absence',
    );

    // The step-4 mirror the KNOWN_ISSUES entry asks for: an out-of-range offset is the
    // typed Planning error, and `describeFailure` renders it as planning prose.
    try {
      await coverCycle(
        request: CoverCycleRequestDto(
          householdId: h.id,
          today: '2026-08-30',
          offsetCycles: 521,
          apply: false,
        ),
      );
      fail('offset 521 must be refused');
    } on KimattaError catch (e) {
      expect(e, isA<KimattaError_Planning>());
      // The bound's own vocabulary — `offset_cycles`, `520` — is what `describeFailure` must
      // never reach the user with, so the assertion is on the prose, not on the number.
      final shown = describeFailure(e, subject: 'Cover My Week');
      expect(shown, contains('too far away to plan'));
      expect(shown, isNot(contains('offset_cycles')));
    }
  });

  /// MVP-016 at the real bridge: the overlay is durable (AC-2), the view carries it back
  /// with the derivation, and a bulk pantry mark flips a line to omitted and the returned
  /// refs flip it back (AC-3). Expected-to-pass — the commands already exist; it is the
  /// only Dart execution of the generated view/state/manual-item decoders.
  test('a shopping view survives reopening the same file', () async {
    final path = await tempDb();
    await openDatabase(dbPath: path);
    final h = await bootstrapHousehold();
    await installStarterContent(householdId: h.id);
    final catalog = await listPantry(householdId: h.id);
    final a = catalog[0];
    await ensurePlanningCycle(
      householdId: h.id,
      defaultAnchorDate: '2026-08-30',
    );
    final saved = await saveRecipe(
      recipe: recipeFor(h.id, [
        IngredientLineDto(
          originalText: '2 cup ${a.name}',
          name: a.name,
          ingredient: a.ingredient,
          quantity: const QuantityDto.exact(numer: 2, denom: 1),
          unit: const UnitDto.known(unit: 'cup'),
          optional: false,
        ),
      ]),
    );
    await savePlannedMeal(
      meal: PlannedMealDto(
        id: '',
        householdId: h.id,
        date: '2026-08-30',
        slot: MealSlotDto.dinner,
        locked: false,
        components: [MealComponentDto(kind: 'recipe', recipeId: saved.id)],
      ),
    );
    final first = await loadShoppingView(
      householdId: h.id,
      fromDate: '2026-08-30',
      toDate: '2026-08-30',
    );
    final line = first.list.groups.single.lines.single;
    expect(first.lineStates, isEmpty);
    expect(first.manualItems, isEmpty);

    final stored = await setShoppingLineState(
      householdId: h.id,
      fromDate: '2026-08-30',
      toDate: '2026-08-30',
      state: ShoppingLineStateDto(
        key: line.key,
        checked: true,
        hidden: false,
        restored: false,
        changed: false,
      ),
    );
    expect(stored.checked, isTrue);
    expect(stored.checkedAgainst, 'exact:2/1|known:cup');
    final item = await saveShoppingManualItem(
      item: ShoppingManualItemDto(
        id: '',
        householdId: h.id,
        name: ' batteries ',
        note: null,
        checked: false,
      ),
    );
    expect(item.id, isNotEmpty);
    expect(item.name, 'batteries');

    await openDatabase(dbPath: path);
    final reopened = await loadShoppingView(
      householdId: h.id,
      fromDate: '2026-08-30',
      toDate: '2026-08-30',
    );
    // Field by field: the generated `==` compares `List` members by identity, so two
    // separately decoded lists never compare equal as wholes.
    final reopenedLine = reopened.list.groups.single.lines.single;
    expect(reopened.list.contributionCount, first.list.contributionCount);
    expect(reopenedLine.key, line.key);
    expect(reopenedLine.quantity, line.quantity);
    expect(reopenedLine.contributions, line.contributions);
    expect(reopened.lineStates, [stored]);
    expect(reopened.manualItems, [item]);

    final changed = await setPantryMarks(
      householdId: h.id,
      ingredients: [a.ingredient],
      marked: true,
    );
    expect(changed, [a.ingredient]);
    final omitted = await loadShoppingView(
      householdId: h.id,
      fromDate: '2026-08-30',
      toDate: '2026-08-30',
    );
    expect(
      omitted.list.groups.single.lines.single.status,
      ShoppingLineStatusDto.omittedPantryMarked,
    );
    final undone = await setPantryMarks(
      householdId: h.id,
      ingredients: changed,
      marked: false,
    );
    expect(undone, [a.ingredient]);
    final restored = await loadShoppingView(
      householdId: h.id,
      fromDate: '2026-08-30',
      toDate: '2026-08-30',
    );
    expect(
      restored.list.groups.single.lines.single.status,
      ShoppingLineStatusDto.needed,
    );

    await resetShoppingList(
      householdId: h.id,
      fromDate: '2026-08-30',
      toDate: '2026-08-30',
    );
    final cleared = await loadShoppingView(
      householdId: h.id,
      fromDate: '2026-08-30',
      toDate: '2026-08-30',
    );
    expect(cleared.lineStates, isEmpty);
    expect(cleared.manualItems, [item], reason: 'unchecked items carry');
    await deleteShoppingManualItem(householdId: h.id, itemId: item.id);
    await expectLater(
      () => deleteShoppingManualItem(householdId: h.id, itemId: item.id),
      throwsA(isA<KimattaError_Storage>()),
    );
    await expectLater(
      () => saveShoppingManualItem(
        item: ShoppingManualItemDto(
          id: '',
          householdId: h.id,
          name: '  ',
          note: null,
          checked: false,
        ),
      ),
      throwsA(isA<KimattaError_Shopping>()),
    );
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

  // MVP-013: expected-to-pass pin. The window read at offset 0 on the anchor day is the
  // ensured cycle itself; the calendar cases are proven in `food-domain`.
  test(
    'the active cycle window on a fresh household is the ensured cycle',
    () async {
      await openDatabase(dbPath: await tempDb());
      final h = await bootstrapHousehold();
      final ensured = await ensurePlanningCycle(
        householdId: h.id,
        defaultAnchorDate: '2026-08-29',
      );
      final window = await planningCycleWindow(
        householdId: h.id,
        today: '2026-08-29',
        offsetCycles: 0,
      );
      expect(window.anchorDate, ensured.anchorDate);
      expect(window.lengthDays, ensured.lengthDays);
      expect(window.mealSlots, ensured.mealSlots);
      expect(window.dates, ensured.dates);
      final next = await planningCycleWindow(
        householdId: h.id,
        today: '2026-08-29',
        offsetCycles: 1,
      );
      expect(next.anchorDate, '2026-09-05');
    },
  );

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
