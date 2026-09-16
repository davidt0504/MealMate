// FIX-001 AC-1 on a device: the shipped app opens a genuine pre-FIX-001 database (schema 12,
// diced tomato lines still pointed at `ing-canned-tomatoes`) and comes back with those lines
// remapped to `ing-diced-tomatoes`.
//
// The Rust fixture tests already prove the SQL. What only a device shows is the same migration
// running inside the real app, against a real file, on an Android runtime.
//
// The fixture is pushed by `tools/ac1_migration_check.sh` rather than bundled as a Flutter
// asset: test data must not ship inside the release APK. Absent, this fails loudly rather than
// passing quietly, because a skipped migration check reads exactly like a passing one.

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:path_provider/path_provider.dart';

import 'package:meal_mate/src/rust/api/health.dart';
import 'package:meal_mate/src/rust/api/recipe.dart';
import 'package:meal_mate/src/rust/frb_generated.dart';

/// Written by `make_v12_fixture`; the household every fixture recipe belongs to.
const _householdId = 'fixture-household';
const _fixtureName = 'v12_pre_fix001.db';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async => RustLib.init());

  testWidgets(
    'a schema 12 database migrates in place and repoints the diced tomato lines',
    (tester) async {
      final external = await getExternalStorageDirectory();
      expect(
        external,
        isNotNull,
        reason: 'no external files directory on this device',
      );
      final fixture = File(
        '${external!.path}${Platform.pathSeparator}$_fixtureName',
      );
      if (!fixture.existsSync()) {
        fail(
          'fixture missing at ${fixture.path} -- run tools/ac1_migration_check.sh, '
          'which builds and pushes it',
        );
      }

      final dir = await Directory.systemTemp.createTemp('ac1-migration');
      addTearDown(() => dir.delete(recursive: true));
      final dbPath = '${dir.path}${Platform.pathSeparator}kimatta.db';

      // A fresh database first, as a reinstalled app holds, then the restore that brings the old
      // file in. `validate_export` passes an older schema deliberately: restore migrates the
      // staged copy forward, which is the upgrade path this check exists to exercise.
      // Latest is read from a fresh database rather than hard-coded, so a later migration does
      // not break a check that is only about getting past 12.
      final fresh = await openDatabase(dbPath: dbPath);
      final latest = fresh.schemaVersion;
      expect(
        latest,
        greaterThanOrEqualTo(13),
        reason: 'this app predates the v13 remap it is meant to check',
      );

      final restored = await restoreDatabase(
        exportPath: fixture.path,
        dbPath: dbPath,
      );
      expect(
        restored.schemaVersion,
        latest,
        reason: 'the schema 12 export must be migrated forward to latest, not restored as-is',
      );

      final summaries = await listRecipes(householdId: _householdId);
      expect(
        summaries,
        isNotEmpty,
        reason: 'the restored database should carry the fixture household\'s starter recipes',
      );
      // Every diced line in every recipe, not one recipe picked by title: the fixture holds two
      // chili recipes, and only a full sweep proves no line was left behind on the old entry.
      final diced = <String, IngredientLineDto>{};
      for (final summary in summaries) {
        final recipe = await loadRecipe(
          householdId: _householdId,
          recipeId: summary.id,
        );
        expect(
          recipe,
          isNotNull,
          reason: '"${summary.title}" is listed but will not load',
        );
        for (final line in recipe!.lines) {
          if (line.name.toLowerCase().contains('diced')) {
            diced['${summary.title}: ${line.name}'] = line;
          }
        }
      }

      // The five FIX-001 names: David's Chili, Taco Soup, Taco Pasta, Stuffed Pepper Casserole,
      // and Black Bean and Lentil Soup. `make_v12_fixture` refuses to write a fixture with none.
      expect(
        diced.keys,
        hasLength(5),
        reason:
            'expected the five FIX-001 diced tomato lines, found ${diced.keys}',
      );

      diced.forEach((where, line) {
        final ingredient = line.ingredient;
        expect(
          ingredient,
          isA<IngredientRefDto_Catalog>(),
          reason: '$where resolved to no catalog ingredient',
        );
        expect(
          (ingredient! as IngredientRefDto_Catalog).id,
          'ing-diced-tomatoes',
          reason: '$where still points at the pre-FIX-001 catalog entry',
        );
      });
    },
  );
}
