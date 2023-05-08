import 'package:flutter_test/flutter_test.dart';
import 'package:meal_mate/models/recipe.dart';


void main()
{
  group('Recipe', () {
    test('should create a Recipe object with correct data', () {
      final recipe = Recipe(
        title: 'Test Recipe',
        ingredients: ['Ingredient 1', 'Ingredient 2'],
        measurements: ['1 cup', '2 cups'],
        servingSize: 4,
        cookingInstructions: 'Cook until done.',
      );

      expect(recipe.title, 'Test Recipe');
      expect(recipe.ingredients, ['Ingredient 1', 'Ingredient 2']);
      expect(recipe.measurements, ['1 cup', '2 cups']);
      expect(recipe.servingSize, 4);
      expect(recipe.cookingInstructions, 'Cook until done.');
    });
  });
}