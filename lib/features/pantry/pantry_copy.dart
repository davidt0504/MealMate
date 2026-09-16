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

/// My Shelves' default view (nothing marked yet) — distinct from [pantryEmptyCopy], which is
/// about the catalog itself being empty.
const pantryNothingMarkedCopy =
    'Nothing marked yet. Search above for what you have.';

/// The accessible name of one row's control, stating the meaning *before* the platform's own
/// toggle state, which no label can suppress. One function so the screen, the recipe detail
/// and their tests cannot drift.
String pantryRowLabel(String name, bool marked) =>
    marked ? '$name, marked as in your pantry' : '$name, not marked';

/// The "Low" chip: flags restock without touching the have/none mark (OPT-006 gate 1) — an
/// optional earlier heads-up, independent of "Used up".
const pantryLowChipLabel = 'Low';

/// The "Used up" chip: clears the have-mark and flags restock together (OPT-006 gate 4).
const pantryUsedUpChipLabel = 'Used up';

/// The accessible label for the restock-flag chip, stating what tapping it does rather than
/// only its current state — the chip itself already shows "Low"/state via its selected style.
String pantryRestockChipLabel(String name, bool flagged) => flagged
    ? '$name, flagged for restock. Tap to clear.'
    : '$name, flag for restock';

/// The accessible label for the "Used up" chip.
String pantryUsedUpLabel(String name) => '$name, used up — clear and restock';

/// One or more custom ingredients still need a shelf before they can appear on My Shelves
/// (OPT-006 gate 5 remediation) — never a silent drop, never a grouped "Other" bucket.
String pantryNeedsCategoryCopy(int count) => count == 1
    ? '1 item needs a shelf — tap to categorize'
    : '$count items need a shelf — tap to categorize';

/// Heading for the categorization sheet/dialog the remediation banner opens.
const pantryCategorizeHeading = 'Give these a shelf';

/// The accessible label for the required category dropdown, shared by the categorization flow
/// and custom-ingredient creation.
const pantryCategoryFieldLabel = 'Store category';
