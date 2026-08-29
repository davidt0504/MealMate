import 'package:meal_mate/src/rust/api/restrictions.dart';

/// The restriction vocabulary as the user reads it. Copy only — no widget, so the recipe
/// warnings module can name a restriction without depending on the restrictions screen.

/// The honesty line, shown first and never softened (invariant 10, and the card's
/// "restrictions are preferences/warnings, not medical assurance").
const restrictionsDisclaimer =
    'Warnings only. Meal Mate flags what it knows about; it can never tell you '
    'a recipe is safe.';

/// The `_ => kind` arm renders the raw token, which is honest; Step 8's bridge cross-check
/// test is what makes a kind added in Rust without a label here a test failure rather than a
/// silent user-visible regression.
String restrictionLabel(String kind) => switch (kind) {
  'peanuts' => 'Peanuts',
  'tree_nuts' => 'Tree nuts',
  'dairy' => 'Dairy',
  'eggs' => 'Eggs',
  'gluten' => 'Gluten',
  'soy' => 'Soy',
  'fish' => 'Fish',
  'shellfish' => 'Shellfish',
  'sesame' => 'Sesame',
  'vegetarian' => 'Vegetarian',
  'vegan' => 'Vegan',
  _ => kind,
};

/// Both the chip list and the Settings subtitle go through this, so no rendering path is
/// left without an arm.
String describeRestriction(RestrictionDto r) => switch (r) {
  RestrictionDto_Known(:final kind) => restrictionLabel(kind),
  RestrictionDto_Other(:final text) => text,
};
