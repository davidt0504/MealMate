// The pantry as the user reads it. Copy only — no widget, so the recipe detail can offer a
// pantry control without depending on the pantry screen.

/// Shown above the list and never softened: a mark may suppress a purchase, and the absence
/// of a mark is unknown, never a record that the household is out (invariants 6 and 19).
const pantryDisclaimer =
    'Optional. Mark what you already have so your shopping list can skip it. '
    'Anything unmarked is simply unknown — it is not a record that you are out.';

/// An empty catalog is honest about why, rather than reading as "you have nothing" — and
/// names the affordance that fills it, since nothing else within a session will. It does not
/// promise that recipes add ingredients: no path in this build creates one.
const pantryEmptyCopy =
    'No ingredients to mark yet. Your starter content adds them the first time '
    'it installs — pull down to refresh if this stays empty.';

/// The search found nothing, which says nothing about the pantry itself.
String pantryNoMatchCopy(String query) =>
    'Nothing matches "$query". Try another name, or clear the search.';

/// The accessible name of one row's control, stating the meaning *before* the platform's own
/// toggle state, which no label can suppress. One function so the screen, the recipe detail
/// and their tests cannot drift.
String pantryRowLabel(String name, bool marked) =>
    marked ? '$name, marked as in your pantry' : '$name, not marked';
