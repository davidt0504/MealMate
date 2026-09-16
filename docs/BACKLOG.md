# Backlog — beta feedback intake

Loose notes from anyone using the app land here first, verbatim, dated, with a source. Each row
is triaged into exactly one disposition; the row stays as the audit trail. This file is intake
only: it never carries status (read `docs/ROADMAP.md`) and never replaces a card.

Dispositions: `bug dossier` (`docs/bugs/`, symptom not yet root-caused) · `FIX card` (bounded,
root-caused, no open design question; batched) · `OPT card` (design-sized; `grill-me` gate) ·
`folded into <ID>` · `won't do` (say why).

| Date | Source | Note (verbatim) | Disposition | Link |
|---|---|---|---|---|
| 2026-09-08 | owner | Why did crushed tomatoes get added? An ingredient was added to the shopping list that wasn't in any of the recipes for the week | FIX card — root-caused 2026-09-08: catalog `ing-canned-tomatoes` is named "crushed tomatoes" and five diced-tomato lines map to it; the list renders the catalog name | `docs/task/fix/FIX-001_BETA_FEEDBACK_FIXES_I.md` |
| 2026-09-08 | owner | I should be able to delete things from the shopping list without compromising the recipes for the planned week | FIX card — removal exists but is hidden inside the "Why is this here?" sheet; recipes were never affected | `docs/task/fix/FIX-001_BETA_FEEDBACK_FIXES_I.md` |
| 2026-09-08 | owner | "As written" and "Name" is redundant. | FIX card — form-only change; domain keeps both fields | `docs/task/fix/FIX-001_BETA_FEEDBACK_FIXES_I.md` |
| 2026-09-08 | owner | Pantry UI is terrible. There's no indication of what the toggle is for. There's no way to set how many of a staple to keep. The long list of all possible ingredients is also poor. Let's figure out a way to design the pantry to feel more like navigating a pantry at home and not like a raw list | OPT card — absorbs OPT-003; counts stay closed, staples model reopened as a decision gate | `docs/task/optional/OPT-006_PANTRY_REDESIGN.md` |
| 2026-09-08 | owner | Plan needs to 1) anticipate leftovers better (maybe have a way to turn off leftovers) instead of a whole meal everyday | OPT card | `docs/task/optional/OPT-008_LEFTOVERS_PLANNING.md` |
| 2026-09-08 | owner | 2) swap needs to say something else more intuitive or something and we need a button along side that and Never suggest for another suggestion from the app. Probably also a button for a completely reshuffle week if nothing sounds immediately appealing. | OPT card | `docs/task/optional/OPT-007_PLANNER_ACTIONS.md` |
| 2026-09-08 | owner | Accept button still doesn't go away after clicking | FIX card — dossier `docs/bugs/MVP-033_ACCEPT_FEEDBACK.md` already names cause and expected behaviour | `docs/task/fix/FIX-001_BETA_FEEDBACK_FIXES_I.md` |
| 2026-09-15 | owner | An export can leave the phone but cannot come back: Restore reads only the app's own internal exports directory, so moving to the dedicated release signing key forces an uninstall that loses the data with nothing able to read the saved file back in | OPT card — Rust already accepts an older-schema export and migrates it forward (`validate_export`), and the confirm copy exists; the gap is a file picker, so the design questions are where a file may come from and what the replace warning says | `docs/task/optional/OPT-009_DATABASE_IMPORT.md` |
