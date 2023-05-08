class Recipe
{
  String title;
  List<String> ingredients;
  List<String> measurements;
  int servingSize;
  String cookingInstructions;

  Recipe({
    required this.title,
    required this.ingredients,
    required this.measurements,
    required this.servingSize,
    required this.cookingInstructions,
  });
}