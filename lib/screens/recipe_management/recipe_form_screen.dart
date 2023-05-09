import 'package:flutter/material.dart';
import 'package:meal_mate/models/recipe.dart';

class RecipeFormScreen extends StatefulWidget {
  @override
  _RecipeFormScreenState createState() => _RecipeFormScreenState();
}

class _RecipeFormScreenState extends State<RecipeFormScreen> {
  final _formKey = GlobalKey<FormState>();

  String _title = '';
  List<String> _ingredients = [];
  List<String> _measurements = [];
  int _servingSize = 1;
  String _cookingInstructions = '';

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Add Recipe')),
      body: Padding(
        padding: EdgeInsets.all(16.0),
        child: Form(
          key: _formKey,
          child: ListView(
            children: [
              // Title field
              TextFormField(
                decoration: InputDecoration(labelText: 'Title'),
                validator: (value) {
                  if (value == null || value.isEmpty) {
                    return 'Please enter a title';
                  }
                  return null;
                },
                onSaved: (value) {
                  _title = value!;
                },
              ),

              // Ingredients field
              // You can create a custom widget for adding and removing ingredients and measurements, or use a TextField for now and let the user input them as comma-separated values

              // Serving size field
              TextFormField(
                decoration: InputDecoration(labelText: 'Serving Size'),
                keyboardType: TextInputType.number,
                validator: (value) {
                  if (value == null || value.isEmpty || int.tryParse(value) == null) {
                    return 'Please enter a valid serving size';
                  }
                  return null;
                },
                onSaved: (value) {
                  _servingSize = int.parse(value!);
                },
              ),

              // Cooking instructions field
              TextFormField(
                decoration: InputDecoration(labelText: 'Cooking Instructions'),
                keyboardType: TextInputType.multiline,
                maxLines: null,
                onSaved: (value) {
                  _cookingInstructions = value!;
                },
              ),

              // Save button
              Padding(
                padding: EdgeInsets.symmetric(vertical: 16.0),
                child: ElevatedButton(
                  onPressed: () {
                    if (_formKey.currentState!.validate()) {
                      _formKey.currentState!.save();
                      // Save the recipe data and navigate back to the recipe list screen
                    }
                  },
                  child: Text('Save Recipe'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
