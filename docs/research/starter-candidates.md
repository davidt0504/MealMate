# MVP-032 Phase 0 — federal public-domain starter-recipe candidates

> Research output and **owner decision document**. Nothing under
> `rust/crates/food-domain/content/` changes until every row below carries a dated
> `select` / `reject` / `hold` mark and both decision items are answered. The gate covers the
> rows put to the owner: a row excluded on authorship never enters it, because that rubric
> exclusion is not the owner's to lower (see the `federal author` legend and §8). The five `no`
> rows are therefore unmarked by design, not pending.

- **Researched:** 2026-09-05, by read-only page reads of `.gov` recipe collections.
- **Card:** `docs/task/mvp/MVP-032_STARTER_CONTENT_II_FEDERAL_SOURCES.md`
- **Plan:** `~/.claude/plans/curious-purring-blum.md` (revision 5)
- **Rows:** 31 candidates read in full. 26 classified `trace-only`, 5 classified `no`
  (rejected by this research pass — see decision item 1).
- **Roster:** the counts above classify *federal authorship*, not selection. On selection: 26
  marked `select` on 2026-09-05; **2 dropped on 2026-09-06** at the owner's roster review (rows 18
  and 31, above); **24 federal entries ship**, alongside the 25 owner household entries, for a
  shipped roster of 49. Recorded 2026-09-06 — the shipped file had carried 24 while these two rows
  still read `select`.

**Layout note.** Every rubric column the card requires is present. The scannable columns
and the `owner decision` column are in **§1**; the long-text columns for the same row —
source line verbatim, planned authored names, catalog additions, unit conversions, notes —
are in **§2**, keyed by the same row number. This is presentation only; no column is
omitted.

---

## 1. Candidate table

`federal author` — `literal-yes` = the recipe page's own source line names a federal
agency; `trace-only` = federal only via the authorship-trace URL; `no` = traces to a
non-federal body. The `no` rows were rejected by this research pass under the card's
authorship rule rather than being put to the owner as a choice — authorship is the one
threshold the card says is never lowered — and the owner's 2026-09-05 review in §8 records
those rejections standing. So they carry no owner `select`/`reject` mark of their own, and
`docs/ROADMAP.md`'s AC-1 evidence sentence should not be read as reporting one.

`restriction kinds` — **the real matcher's output**, not a prediction. Produced by running
`food_domain::assess` over each row's planned authored ingredient names against all 11
`RestrictionKind` values, the same call `every_recipe_matches_its_declared_conflicts`
makes. A dish is *vegan-suitable* when `vegan` is absent, *gluten-free* when `gluten` is
absent, *omnivore* when `vegetarian` is present.

| # | title | source URL (all `nhlbi.nih.gov/health/heart-healthy-living/healthy-foods/healthy-eating-recipes/…`) | federal author | servings | prep min | cook min | ingredient lines | common pantry? | restriction kinds (matcher) | owner decision |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 20-Minute Chicken Creole | `…/20-minute-chicken-creole` | trace-only | 4 | 15 | 20 | 12 | mostly (chili sauce) | vegetarian, vegan | select |
| 2 | Asian-Style Steamed Salmon | `…/asian-style-steamed-salmon` | trace-only | 4 | 15 | 10 | 7 | mostly (shiitake) | soy, fish, sesame, vegetarian, vegan | select |
| 3 | Baked Salmon Dijon | `…/baked-salmon-dijon` | trace-only | 6 | 10 | 20 | 9 | yes | dairy, fish, vegetarian, vegan | select |
| 4 | Baked Tilapia With Tomatoes | `…/baked-tilapia-tomatoes` | trace-only | 4 | 10 | 25 | 11 | yes | fish, vegetarian, vegan | select |
| 5 | Braised Cod With Leeks | `…/braised-cod-leeks` | trace-only | 4 | 15 | 25 | 9 | yes | dairy, fish, vegetarian, vegan | select |
| 6 | Caribbean Pink Beans | `…/caribbean-pink-beans` | trace-only | 16 | 10 | 60 | 7 | mostly (plantain) | *(none — vegan, gluten-free)* | select |
| 7 | Chicken and Mushroom Fricassee | `…/chicken-and-mushroom-fricassee` | trace-only | 4 | 10 | 30 | 14 | yes | dairy, vegetarian, vegan | select |
| 8 | Chicken Chile Stew | `…/chicken-chile-stew` | **no** | 10 | 20 | 105 | 7 | mostly | vegetarian, vegan | *rejected — see item 1* |
| 9 | Chicken Picadillo | `…/chicken-picadillo` | trace-only | 6 | 15 | 25 | 16 | mostly (capers, olives) | vegetarian, vegan | select |
| 10 | Cold Fusilli Pasta with Summer Vegetables | `…/cold-fusilli-pasta-summer` | trace-only | 4 | 20 | 10 | 12 | yes | dairy, gluten, vegan | select |
| 11 | Grilled Tuna With Chickpea and Spinach Salad | `…/grilled-tuna-chickpea-and-spinach` | trace-only | 4 | 25 | 20 | 11 | yes | fish, vegetarian, vegan | select |
| 12 | Hawaiian Huli Huli Chicken | `…/hawaiian-huli-huli-chicken` | trace-only | 4 | 10 | 30 | 8 | yes | soy, vegetarian, vegan | select |
| 13 | Heavenly Chicken With Angel Hair Pasta | `…/heavenly-chicken-angel-hair-pasta` | trace-only | 4 | 15 | 15 | 10 | yes | gluten, vegetarian, vegan | select |
| 14 | Lentil Soup | `…/lentil-soup` | trace-only | 11 | 15 | 90 | 12 | yes | *(none — vegan, gluten-free)* | select |
| 15 | Marinated Chicken (Adobong Manok) | `…/marinated-chicken-adobong-manok` | **no** | 4 | 10 | 90 | 10 | yes | soy, vegetarian, vegan | *rejected — see item 1* |
| 16 | Minestrone Soup | `…/minestrone-soup` | trace-only | 16 | 15 | 60 | 15 | yes | gluten | select |
| 17 | Mushroom Penne | `…/mushroom-penne` | trace-only | 4 | 15 | 15 | 11 | mostly (red wine) | dairy, gluten, vegetarian, vegan | select |
| 18 | New Orleans Red Beans | `…/new-orleans-red-beans` | trace-only | 8 | 10 | 140 | 11 | yes | *(none — vegan, gluten-free)* | `select` 2026-09-05, **`drop` 2026-09-06** — cut at the owner's roster review when the twenty-five household entries landed; see the **Roster** line in the header |
| 19 | Pinto Beans | `…/pinto-beans` | **no** | 16 | 20 | 45 | 9 | yes | *(none — vegan, gluten-free)* | *rejected — see item 1* |
| 20 | Pita Pizzas | `…/pita-pizzas` | trace-only | 4 | 10 | 8 | 6 | yes | dairy, gluten, vegetarian, vegan | select |
| 21 | Red Beans and Rice | `…/red-beans-and-rice` | trace-only | 4 | 5 | 25 | 9 | yes | *(none — vegan, gluten-free)* | select |
| 22 | Spicy Southern Barbecued Chicken | `…/spicy-southern-barbecued-chicken` | trace-only | 6 | 10 | 65 | 12 | mostly (molasses) | vegetarian, vegan | select |
| 23 | Turkey and Beef Meatballs with Whole-Wheat Spaghetti | `…/turkey-and-beef-meatballs-whole` | trace-only | 4 | 20 | 20 | 16 | mostly (evaporated milk) | dairy, gluten, vegetarian, vegan | select |
| 24 | Turkey Bolognese With Shell Pasta | `…/turkey-bolognese-shell-pasta` | trace-only | 4 | 5 | 35 | 13 | mostly (porcini, wine) | dairy, gluten, vegetarian, vegan | select |
| 25 | Turkey Club Burger | `…/turkey-club-burger` | trace-only | 4 | 20 | 20 | 10 | yes | eggs, gluten, vegetarian, vegan | select |
| 26 | Tuscan Beans With Tomatoes and Oregano | `…/tuscan-beans-tomatoes-and-oregano` | trace-only | 4 | 15 | 0 | 8 | yes | *(none — vegan, gluten-free)* | select |
| 27 | Vegetable Stew | `…/vegetable-stew` | trace-only | 8 | 20 | 45 | 13 | yes | *(none — vegan, gluten-free)* | select |
| 28 | Zucchini Medley | `…/zucchini-medley` | **no** | 4 | 15 | 10 | 6 | yes | dairy, vegan | *rejected — see item 1* |
| 29 | Alaska Salmon Salad | `…/alaska-salmon-salad` | **no** | 6 | 10 | 30 | 6 | yes | dairy, fish, vegetarian, vegan | *rejected — see item 1* |
| 30 | Baked Fish | `…/baked-fish` | trace-only | 6 | 10 | 25 | 11 | yes | dairy, gluten, fish, vegetarian, vegan | select |
| 31 | Homemade Turkey Soup | `…/homemade-turkey-soup` | trace-only | 16 | 15 | 180 | 12 | no (turkey carcass) | gluten, vegetarian, vegan | `select` 2026-09-05, **`drop` 2026-09-06** — cut at the owner's roster review when the twenty-five household entries landed; see the **Roster** line in the header |

---

## 2. Row detail

Source lines are quoted **verbatim** as the recipe page prints them. Italics in the
original are shown as they appeared. Every source line names a *cookbook*, never an
agency — see decision item 1.

**Authorship-trace URLs**, one per collection:

| collection (as printed in the source line) | authorship trace URL | what that page says |
|---|---|---|
| *Deliciously Healthy Dinners* | `nhlbi.nih.gov/resources/keep-beat-recipes-deliciously-healthy-dinners` | NHLBI publication, January 2010; "a brand new version of the popular Keep the Beat classic cookbook" with "75 new deliciously healthy recipes". No developer named, no testing statement **on this landing page** — **corrected 2026-09-07: the publication itself names both.** NIH Pub 10-2921's acknowledgments read "Recipes were developed by David Kamen … Chef/Instructor at the Culinary Institute of America, and Colleen Pierre … consultant", with testing by Northern Illinois University. Reading the landing page instead of the publication is what let this through; see the AC-2 amendment on the `MVP-032` `Done` row in `docs/ROADMAP.md` and the HIGH entry in `KNOWN_ISSUES.md`. |
| *Deliciously Healthy Family Meals* | `nhlbi.nih.gov/resources/keep-beat-recipes-deliciously-healthy-family-meals` | NHLBI publication; "This Keep the Beat cookbook contains 40 recipes **developed just for the NHLBI**". No testing statement. |
| *Delicious Heart Healthy Latino Recipes* | `nhlbi.nih.gov/resources/delicious-heart-healthy-latino-recipes-book-platillos-latinos-sabrosos-y-saludables` | NHLBI publication, April 2024. No developer credit, no testing statement, no copyright statement **on this landing page** — **corrected 2026-09-07:** the publication (NIH Pub 24-HL-4049S) states NHLBI developed the cookbook, and credits recipe *testing* to Wahida Karmally and colleagues at Columbia University's Irving Center, with review by promotores from the National Association of Community Health Workers. Development is federal; a named non-federal party tested. |
| *Stay Young At Heart* | `nhlbi.nih.gov/health/educational/wecan/eat-right/fun-family-recipes.htm` | NHLBI collection, part of a collaboration between NHLBI, NIDDK, NICHD and NCI — all federal institutes. |
| *Heart Healthy Home Cooking African American Style* | `nhlbi.nih.gov/resources/heart-healthy-home-cooking-african-american-style` | NHLBI/NIH, February 2021, NIH Publication No. 21-HL-3792. **"26 tested and tasty favorite African American dishes"** — the only explicit testing claim found anywhere in this research. |
| *Honoring the Gift of Heart Health Manual for American Indians and Native Alaskans* | `nhlbi.nih.gov/education/heart-truth/CHW/HTGHH` | Developed by NHLBI, the Center on Minority Health and Health Disparities and the IHS **"in partnership with Laguna Pueblo in New Mexico, Bristol Bay Area Health Corporation in Alaska, and the Ponca Tribe in Oklahoma."** Named co-developers are tribal organizations, which are not federal agencies. |
| *Healthy Heart, Healthy Family Manual for the Filipino Community* | `nhlbi.nih.gov/resources/healthy-heart-healthy-family-community-health-workers-manual-filipino-community` | NHLBI/HRSA collaborative (both federal) delivered through community health workers. No statement of who authored the recipes. |

Row-by-row:

1. **20-Minute Chicken Creole** — source line: `Recipe Source: _Deliciously Healthy Dinners_`. Planned authored names: chicken breast, crushed tomatoes, chili sauce, green bell pepper, celery, onion, garlic, basil, parsley, crushed red pepper, salt. Catalog additions: chicken breast, chili sauce, green bell pepper, celery, basil, crushed red pepper. Unit conversions: `12 oz` chicken → g. Notes: 12 ingredient lines, above the ≤10 rubric.
2. **Asian-Style Steamed Salmon** — `Recipe Source: _Deliciously Healthy Dinners_`. Names: chicken broth, shiitake mushrooms, ginger, scallion, soy sauce, toasted sesame oil, salmon fillet. Catalog additions: chicken broth, shiitake mushrooms. Conversions: `12 oz` salmon → g. Notes: **clears every rubric threshold** — 4 servings, 15 min prep, 7 lines.
3. **Baked Salmon Dijon** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: sour cream, dill, scallion, dijon mustard, lemon juice, salmon fillet, garlic powder, black pepper. Catalog additions: sour cream, dijon mustard, garlic powder. Conversions: `1½ lb` salmon → g. Notes: clears every threshold (6 servings, 10 min, 9 lines).
4. **Baked Tilapia With Tomatoes** — `Recipe Source: "Delicious Heart Healthy Latino Recipes"`. Names: tilapia fillet, tomato, olive oil, thyme, black olives, red pepper flakes, garlic, red onion, lime juice. Catalog additions: tilapia fillet, tomato, thyme, black olives, red pepper flakes, red onion. Conversions: none (all counts/spoons). Notes: 11 lines, one over.
5. **Braised Cod With Leeks** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: butter, leek, carrot, potato, chicken broth, parsley, cod fillet, salt, black pepper. Catalog additions: leek, chicken broth, cod fillet. Conversions: `12 oz` cod → g. Notes: clears every threshold.
6. **Caribbean Pink Beans** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: pink beans, plantain, tomato, red bell pepper, onion, garlic, salt. Catalog additions: pink beans, plantain, tomato, red bell pepper. Conversions: `1 lb` beans → g. Notes: **vegan and gluten-free**; 16 servings and a 60-minute cook are both large for a weeknight.
7. **Chicken and Mushroom Fricassee** — `Recipe Source: _Deliciously Healthy Dinners_`. Names: olive oil, white mushrooms, leek, potato, celery, pearl onions, chicken broth, chicken thighs, parsley, lemon juice, cornstarch, sour cream, salt, black pepper. Catalog additions: white mushrooms, leek, celery, pearl onions, chicken broth, cornstarch, sour cream. Conversions: `10 oz` mushrooms, `1 lb` chicken → g. Notes: 14 lines, well over the rubric.
8. **Chicken Chile Stew** — `Recipe Source: "_Honoring the Gift of Heart Health Manual for American Indians and Native Alaskans_"`. **Rejected on authorship** (item 1). Names would have been: chicken breast, celery, tomato, green chiles, garlic, black pepper.
9. **Chicken Picadillo** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: olive oil, onion, green bell pepper, red bell pepper, garlic, chicken breast, tomato sauce, chicken broth, lemon juice, ground cumin, bay leaf, raisins, cilantro, capers, green olives. Catalog additions: green/red bell pepper, chicken breast, tomato sauce, chicken broth, bay leaf, raisins, cilantro, capers, green olives. Conversions: `12 oz` chicken → g. Notes: 16 lines, the longest in the set.
10. **Cold Fusilli Pasta with Summer Vegetables** — `Recipe Source: "Recipe Source: _Deliciously Healthy Dinners_"`. Names: whole-wheat fusilli, cherry tomatoes, green bell pepper, red onion, zucchini, chickpeas, basil, salt, black pepper, olive oil, balsamic vinegar, parmesan cheese. Catalog additions: whole-wheat fusilli, cherry tomatoes, green bell pepper, red onion, zucchini, basil, balsamic vinegar, parmesan cheese. Conversions: `8 oz` pasta, `15½ oz` chickpeas → g. Notes: **vegetarian but not vegan**; 12 lines.
11. **Grilled Tuna With Chickpea and Spinach Salad** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: olive oil, garlic, lemon juice, oregano, tuna steak, chickpeas, spinach, tomato, salt, black pepper. Catalog additions: oregano, tuna steak, spinach, tomato. Conversions: `12 oz` tuna, `15½ oz` chickpeas, `10 oz` spinach → g. Notes: prep 25 min, inside the ≤30 threshold; 11 lines.
12. **Hawaiian Huli Huli Chicken** — `Recipe Source: "Deliciously Healthy Family Meals"`. Names: chicken breast, pineapple, ketchup, soy sauce, honey, orange juice, garlic, ginger. Catalog additions: chicken breast, pineapple, ketchup, honey, orange juice. Conversions: `12 oz` chicken → g. Notes: clears every threshold; skewers need no equipment the app models.
13. **Heavenly Chicken With Angel Hair Pasta** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: onion, garlic, broccoli, olive oil, chicken breast, pasta sauce, cayenne pepper, salt, angel hair pasta. Catalog additions: broccoli, chicken breast, pasta sauce, cayenne pepper, angel hair pasta. Conversions: `8 oz` chicken, `26 oz` sauce, `8 oz` pasta → g. Notes: clears every threshold (4 servings, 15 min, 10 lines).
14. **Lentil Soup** — `Recipe Source: "Delicious Heart Healthy Latino Recipes"`. Names: olive oil, carrot, celery, onion, garlic, oregano, basil, black pepper, lentils, crushed tomatoes, vegetable broth, water. Catalog additions: celery, oregano, basil, lentils, water. Conversions: `14½ oz` tomatoes → g. Notes: **vegan and gluten-free**; 90-minute cook.
15. **Marinated Chicken (Adobong Manok)** — `Recipe Source: "_Healthy Heart, Healthy Family Manual for the Filipino Community_"`. **Rejected on authorship** (item 1).
16. **Minestrone Soup** — `Recipe Source: "_Stay Young At Heart_"`. Names: olive oil, garlic, onion, celery, tomato paste, parsley, carrot, cabbage, tomato, kidney beans, frozen peas, green beans, hot sauce, water, spaghetti. Catalog additions: celery, tomato paste, cabbage, tomato, kidney beans, green beans, hot sauce, water. Conversions: `6 oz` paste, `1 lb` tomatoes, `15½ oz` beans → g. Notes: **vegan**; gluten only (the spaghetti). 15 lines, 16 servings.
17. **Mushroom Penne** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: whole-wheat penne, olive oil, white mushrooms, onion, garlic, red wine, chicken broth, salt, black pepper, thyme, parmesan cheese. Catalog additions: whole-wheat penne, white mushrooms, red wine, chicken broth, thyme, parmesan cheese. Conversions: `8 oz` penne, `8 oz` mushrooms → g. Notes: 11 lines; red wine is not a common pantry item for every household.
18. **New Orleans Red Beans** — `Recipe Source: "_Stay Young At Heart_"`. Names: red beans, water, onion, celery, bay leaf, green bell pepper, garlic, parsley, thyme, salt, black pepper. Catalog additions: red beans, water, celery, bay leaf, green bell pepper, thyme. Conversions: `1 lb` beans → g. Notes: **vegan and gluten-free**; 140-minute cook.
19. **Pinto Beans** — `Recipe Source: "Honoring the Gift of Heart Health Manual for American Indians and Native Alaskans"`. **Rejected on authorship** (item 1). Would otherwise have been a vegan, gluten-free candidate.
20. **Pita Pizzas** — `Recipe Source: "_Deliciously Healthy Family Meals_"`. Names: tomato sauce, chicken breast, broccoli, parmesan cheese, basil, whole-wheat pita. Catalog additions: tomato sauce, chicken breast, broccoli, parmesan cheese, basil, whole-wheat pita. Conversions: none. Notes: **clears every threshold and is the shortest main in the set** — 4 servings, 10 min prep, 6 lines.
21. **Red Beans and Rice** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: olive oil, onion, green bell pepper, garlic, ground cumin, oregano, vegetable broth, brown rice, kidney beans. Catalog additions: green bell pepper, oregano, brown rice, kidney beans. Conversions: `14½ oz` broth → ml (the `ing-vegetable-broth` id is already `VolumeMetric`), `15 oz` beans → g. Notes: **vegan and gluten-free, 5 min prep, 4 servings, 9 lines — the strongest vegan candidate.**
22. **Spicy Southern Barbecued Chicken** — `Recipe Source: "Heart Healthy Home Cooking African American Style"`. Names: tomato paste, ketchup, honey, molasses, worcestershire sauce, vinegar, cayenne pepper, black pepper, onion powder, ginger, garlic, chicken. Catalog additions: tomato paste, ketchup, honey, molasses, worcestershire sauce, vinegar, cayenne pepper, onion powder, chicken. Conversions: none. Notes: 12 lines; **from the one collection carrying an explicit "tested" claim.**
23. **Turkey and Beef Meatballs with Whole-Wheat Spaghetti** — `Recipe Source: _Deliciously Healthy Family Meals_`. Names: whole-wheat spaghetti, tomato sauce, basil, parmesan cheese, ground turkey, whole-wheat breadcrumbs, evaporated milk, chives, parsley, ground beef. Catalog additions: whole-wheat spaghetti, tomato sauce, basil, parmesan cheese, ground turkey, whole-wheat breadcrumbs, evaporated milk, chives, ground beef. Conversions: `8 oz` pasta, `6 oz` turkey, `6 oz` beef → g. Notes: 16 lines once both meatball sub-lists are flattened.
24. **Turkey Bolognese With Shell Pasta** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: red wine, porcini mushrooms, onion, celery, carrot, garlic, olive oil, ground turkey, anise seed, salt, shell pasta, tomato paste, parmesan cheese. Catalog additions: red wine, porcini mushrooms, celery, ground turkey, anise seed, shell pasta, tomato paste, parmesan cheese. Conversions: `½ oz` porcini, `12 oz` turkey, `8 oz` pasta → g. Notes: 13 lines; two optional ingredients.
25. **Turkey Club Burger** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: ground turkey, scallion, black pepper, eggs, olive oil, mayonnaise, dijon mustard, spinach, portabella mushroom, whole-wheat hamburger buns. Catalog additions: ground turkey, mayonnaise, dijon mustard, spinach, portabella mushroom, whole-wheat hamburger buns. Conversions: `12 oz` turkey, `4 oz` spinach, `4 oz` mushroom → g. Notes: clears the line and prep thresholds (10 lines, 20 min).
26. **Tuscan Beans With Tomatoes and Oregano** — `Recipe Source: "_Deliciously Healthy Dinners_"`. Names: chickpeas, cherry tomatoes, olive oil, balsamic vinegar, oregano, black pepper, seasoning blend, romaine lettuce. Catalog additions: cherry tomatoes, balsamic vinegar, oregano, seasoning blend, romaine lettuce. Conversions: `15½ oz` chickpeas → g. Notes: **vegan, gluten-free, zero cook time, 8 lines** — a salad rather than a dinner; the owner decides whether that covers a slot.
27. **Vegetable Stew** — `Recipe Source: "_Heart Healthy Home Cooking African American Style_"`. Names: water, vegetable bouillon, potato, carrot, summer squash, sweet corn, thyme, garlic, scallion, hot pepper, onion, tomato. Catalog additions: water, vegetable bouillon, summer squash, sweet corn, thyme, hot pepper, tomato. Conversions: `15 oz` corn → g. Notes: **vegan and gluten-free**; 13 lines; from the collection with the explicit "tested" claim.
28. **Zucchini Medley** — `Recipe Source: "Honoring the Gift of Heart Health Manual for American Indians and Native Alaskans"`. **Rejected on authorship** (item 1).
29. **Alaska Salmon Salad** — `Recipe Source: "Honoring the Gift of Heart Health Manual for American Indians and Native Alaskans"`. **Rejected on authorship** (item 1). Its ¼-cup serving is a spread, not a dinner, so it would likely have been rejected on the rubric too.
30. **Baked Fish** — `Recipe Source: "_Heart Healthy Home Cooking African American Style_"`. Names: fish fillet, lemon juice, buttermilk, hot sauce, garlic, white pepper, salt, onion powder, breadcrumbs, vegetable oil, lemon. Catalog additions: fish fillet, buttermilk, hot sauce, white pepper, onion powder, breadcrumbs, vegetable oil. Conversions: `2 lb` fish → g. Notes: 11 lines; "fish fillets" is unspecified as to species; from the "tested" collection.
31. **Homemade Turkey Soup** — `Recipe Source: "_Stay Young At Heart_"`. Names: turkey, onion, celery, thyme, rosemary, sage, basil, marjoram, tarragon, salt, black pepper, pasta. Catalog additions: turkey, celery, thyme, rosemary, sage, basil, marjoram, tarragon, pasta. Conversions: `½ lb` pasta → g. Notes: **fails the common-pantry column** — it starts from a leftover turkey carcass; 180-minute cook; 12 lines.

---

## 3. Coverage summary

Computed from §1's matcher column, over the **26 `trace-only` rows only** (the 5 `no` rows
are already rejected):

| coverage | definition | count | rows |
|---|---|---|---|
| vegan | no `vegan` conflict | **7** | 6, 14, 16, 18, 21, 26, 27 |
| vegetarian | no `vegetarian` conflict | **8** | 6, 10, 14, 16, 18, 21, 26, 27 |
| gluten-free | no `gluten` conflict | **17** | 1, 2, 3, 4, 5, 6, 7, 9, 11, 12, 14, 18, 21, 22, 26, 27, 29†|
| omnivore | has a `vegetarian` conflict | **18** | 1, 2, 3, 4, 5, 7, 9, 11, 12, 13, 17, 22, 23, 24, 25, 30, 31, + others |

† row 29 is rejected on authorship; excluded from the usable count, which is 16.

**All four AC-4 arms are covered with margin.** No threshold needs lowering to reach ten.

---

## 4. What your selection has to clear

Step 6 of the plan enforces both of these mechanically before any content is written, so
they are worth seeing while marking:

1. **At least ten rows marked `select`.**
2. **The selected rows must cover all four:** at least one vegan (no `vegan` conflict), one
   vegetarian (no `vegetarian`), one gluten-free (no `gluten`), and one omnivore (has
   `vegetarian`).

If a selection misses either bar, execution stops and comes back to you rather than
proceeding.

Mark each row's `owner decision` cell `select`, `reject` or `hold`, and date the review at
the bottom of this document. Own-household recipes may be appended as extra rows; they
enter as `original` entries needing a truthful `cook_review`, and are optional.

---

## 5. Owner decision item 1 — what counts as proven federal authorship

**The card's literal wording rejects the entire set.** It requires that "its source line
names a federal agency". Not one of the 31 recipe pages does: every source line names a
**cookbook** (*Deliciously Healthy Dinners*, *Stay Young At Heart*, …). There are **zero
`literal-yes` rows**. Under a literal reading this card stops here with nothing shippable.

**What the pages do show.** Every one of the 26 `trace-only` rows sits on an
`nhlbi.nih.gov` URL, and each named cookbook has an NHLBI publication page (the trace
table in §2) identifying it as NHLBI's own publication — one with an NIH publication
number (21-HL-3792), one stating its recipes were "developed just for the NHLBI".

**What this research pass already rejected on your behalf,** because the card makes
authorship the one threshold that is never lowered:

- Rows 8, 19, 28, 29 — *Honoring the Gift of Heart Health*. NHLBI's own page says it was
  developed "in partnership with Laguna Pueblo in New Mexico, Bristol Bay Area Health
  Corporation in Alaska, and the Ponca Tribe in Oklahoma." Those named co-developers are
  tribal organizations, not federal agencies. This is exactly the partner-content hazard
  D-041 names, and it is why the column exists.
- Row 15 — *Healthy Heart, Healthy Family Manual for the Filipino Community*. An
  NHLBI/HRSA publication (both federal), but the manual is delivered through community
  health workers and no page states who authored the recipes. Federal authorship of the
  *recipe* cannot be shown from the page, so it is rejected rather than assumed.

**The question for you.** Does `trace-only` — a federal-domain recipe page plus a federal
publication page for the named cookbook — satisfy the card's authorship bar?

- **If yes:** the card's Load-bearing-constraints sentence is amended to the three-field
  form (recipe page URL, source line verbatim, authorship-trace URL), with a matching D-041
  amendment clause, both dated and attributed to you. That happens at plan step 7, after
  this stop, with your authority — not before it.
- **If no:** no candidate qualifies, and the card stops for a different source or a
  different approach.

---

## 6. Owner decision item 2 — contractor authorship, and what the federal arm is claiming

D-041's federal arm rests on two things: that a federal publication is public domain, and
that "published under that kitchen's standards" substitutes for the cook review this
project otherwise requires. Both need a look before ten recipes ship on them.

**The facts found.**

- NHLBI's page for *Deliciously Healthy Family Meals* says its 40 recipes were "developed
  just for the NHLBI". Published NHLBI material describes the *Keep the Beat* recipes as
  created by a Culinary Institute of America-trained chef and a James Beard
  Foundation-award-winning registered dietitian — i.e. **commissioned contractors, not
  NHLBI staff**.
- 17 U.S.C. §105 removes copyright from works of federal **employees**. A contractor
  work-for-hire is a different posture. So "federal publication ⇒ public domain" is not
  automatic for these collections.
- **No collection states that its recipes were tested, with one exception:**
  *Heart-Healthy Home Cooking African American Style* is described as "26 **tested** and
  tasty favorite African American dishes" (rows 22, 27, 30). Every other page is silent.
  Silence is not a "not tested" statement, so the card's decision gate does not fire on its
  own — which is why this is put to you here.

**Which claim is this card making?** The two are not interchangeable:

- **(a) "The text is US federal public domain."** Then `rights.basis` is
  `us_federal_public_domain`, the entries ship on D-041's federal arm, and AC-2 and
  invariant 12 are graded against that claim. The contractor question above is a real, if
  small, exposure to it.
- **(b) "We take only the uncopyrightable layer."** US copyright protects neither an
  ingredient list nor a functional instruction, and this card rewrites every instruction in
  the project's own words regardless. On this reading nothing of anyone's is being
  reproduced — but then the entries are *our* prose over public facts, which is the
  `original` basis the existing ten entries carry, and **`original` does not ship without a
  cook review**. The library stays empty.

**Recommended default, for you to accept or overturn:** claim (a), with the risk stated in
the roadmap handoff — the practical exposure is near zero because (b) is simultaneously
true of everything actually copied, and the alternative is that the card cannot deliver its
own outcome. If you prefer (b), execution stops and the card's premise needs re-deciding
rather than papering over.

**Second half of the question — testing.** Does the federal arm apply to collections that
say nothing about testing? A stricter answer is available and costs coverage: restricting
to *Heart-Healthy Home Cooking African American Style* leaves rows 22, 27 and 30 — three
recipes, below AC-4's ten.

---

## 7. Sources attempted

| source | outcome |
|---|---|
| `nhlbi.nih.gov/health/heart-healthy-living/healthy-foods/healthy-eating-recipes` | **Reachable.** 54 recipes listed; 31 read in full. The remaining 23 are desserts, drinks, fruit salads and single-vegetable sides (Grapesicles, Mango Shake, Apple Coffee Cake, Rainbow Fruit Salad, …), none of which covers a dinner slot. |
| `myplate.gov/recipes` | **HTTP 403** to page reads. Not retrievable read-only. MyPlate Kitchen is also an aggregator that republishes state and university partner recipes, which the card excludes, so it was not pursued further. |
| `snaped.fns.usda.gov/recipes` | **HTTP 404** at that path. SNAP-Ed Connection is likewise an aggregator. |
| `healthyeating.nhlbi.nih.gov/about.aspx` and `…/pdfs/Dinners_Cookbook_508-compliant.pdf` | **HTTP 301** — both now redirect to the main recipe listing; the standalone cookbook PDFs and their acknowledgements pages are gone. This is why the contractor-authorship evidence in item 2 comes from NHLBI's publication pages and published descriptions rather than from cookbook front matter. |
| CDC recipe collections | No `.gov` CDC recipe collection surfaced in search that was not a republication of the NHLBI or USDA material above. |

No scraping tool, bulk download, or authenticated request was used. Every retrieval was a
single read-only page fetch.

---

## 8. Owner review

**Reviewed by:** David (owner)  **Date:** 2026-09-05

**Row marks:** all 26 `trace-only` rows marked `select`. The 5 `no` rows stay rejected on
authorship, as this research pass classified them.

**Decision item 1 — authorship standard: `trace-only` qualifies.** Answered by the owner's
selection of the full 26-row set, every member of which is `trace-only`; there are no
`literal-yes` rows, so no other reading of that selection exists. The card's
Load-bearing-constraints authorship sentence is amended to the three-field form (federal-domain
recipe page URL, source line verbatim, authorship-trace URL), with a matching D-041 amendment
clause, both dated and attributed to the owner. The rejections above stand: authorship remains
the one threshold never lowered.

**Decision item 2 — rights claim and testing: claim (a), with the caveat recorded.**
`rights.basis` stays `us_federal_public_domain` for all 26. The doc comment on
`RightsBasis::UsFederalPublicDomain` (`rust/crates/food-domain/src/recipe.rs:284`) and the
JSON `policy` block are corrected to state the actual claim — *published by a US federal
agency and asserted by that agency to be free of use restrictions; for the contractor-developed
Keep the Beat collections this rests on NHLBI's own statement rather than on 17 U.S.C. §105's
automatic operation*. NHLBI's requested citation goes in `rights.attribution`, the contractor
and rewrite caveat in `rights.modifications`, and an endorsement-prohibition note in `policy`
beside the existing `cc_by_note` for `MVP-020` to read. `MVP-020` renders `attribution`, never
the raw basis string.

Legality does not rest on that label: 37 CFR 202.1(a), Copyright Office Circular 33 and
*Publications Int'l, Ltd. v. Meredith Corp.*, 88 F.3d 473 (7th Cir. 1996) put ingredient lists
and functional steps outside copyright entirely, and every instruction is rewritten in the
project's own words.

**Two authoring rules carried from this review:** attribution is phrased as a source citation,
never a badge — NHLBI prohibits use "in any direct or indirect product endorsement or
advertising", and the paid model (DEC-006) makes that live; and the "heart-healthy" framing is
**not** carried into titles or copy, so the app makes no health claim it cannot stand behind.

**Roster size — 26, not the AC-4 floor of 10.** The `≤10 ingredient lines` rubric threshold is
**relaxed** (it was a planning guess; nothing in the architecture reads ingredient-line count
except shopping-list length). Authorship is not relaxed. Rationale, from the architecture:
`RECENTLY_EATEN` covers a 14-day history window (`controller.rs:568`) against 7 dinner slots a
week, so at N ≤ 14 every candidate carries the same −1 by week 2 and the variety term stops
discriminating; 26 − 14 = 12 leaves exactly `candidates_per_slot` dishes penalty-free. Cost is
+24% planner work with `states_scored` flat at 588 for any N ≥ 12, ~31 KB of APK, and a catalog
growing toward ~100 entries — which lengthens the Pantry screen's eager `ListView`, so
candidates that reuse existing catalog ids are preferred during conversion.

---

## Owner household recipes — added 2026-09-06 (D-031 second sentence: in-scope evidence addition)

Thirty index-card recipes photographed by the owner (`refs/images/`, gitignored) were
transcribed in full. Twenty-five are mains and ship as `original` entries on the **cook arm**
of D-041's rule; five are held back and parked in `refs/recipes-pending.json`.

**Rights.** `basis: original`, `source_author: David`, no `source_url`. These are the
household's own recipes; nothing is reproduced from a third party. Where a card named a brand
(chili beans, chili seasoning) the generic product is authored instead and the substitution is
recorded in `rights.modifications`. "David's Chili" keeps its name by owner decision.

**Cook review.** Every entry carries `cooked_on: 2026-09-06`, `by: David`. The date is an
**attestation** date, not a record of a particular cooking — the owner confirmed that he or his
wife has cooked each dish, but no card records when. Each entry's `cook_review.corrections`
says exactly that, so the field is not read later as a claim it cannot support.

**Servings.** No card records servings. Authored as **6 for soups and slow-cooker dishes, 4 for
everything else**, by owner decision. The bar that matters is `feeds_leftovers`
(`candidates.rs:75`): `servings × scale ≥ members + 1`, so 4 clears the leftovers bar for a
two-person household at 3 and 6 clears it comfortably; both stay honest for a family of four,
where 4 fails at 5 and 6 passes. `servings: null` was rejected — it makes `feeds_leftovers`
return false permanently and invisibly.

**prep_minutes.** Derived from each card's stated cook time under the rule the owner set on
2026-09-06 and recorded in `policy.prep_minutes`: *the time the meal costs you*. Slow cooker →
active work only; oven or stovetop → prep plus cook. Effect on the shipped roster: 36 of 49
dishes fit a 40-minute weeknight window, and the seven slow-cooker dishes stay viable at 10–20
minutes rather than being hard-rejected at 180–540. The owner accepted these values as
first-pass estimates to be tuned in-app (see below).

**Tuning.** Values are tweakable without a rebuild: `recipe_form_screen.dart:127,163-195` binds
both `prepMinutes` and `servings`, and `editing_a_starter_recipe_keeps_its_rights_and_slug`
(`kimatta-storage/src/lib.rs:6899`) pins that an edited starter keeps its slug and rights.
Note the reach limit: `install_starter_content` (`lib.rs:1825-1834`) installs only slugs a
household does not already hold, so a later edit to this JSON does **not** reach an
already-installed device — it reaches fresh installs. The two lanes are complementary and the
table above is the written baseline to diff a tweaked value against.

**Matcher misses — one closed at the source, one recorded** (`policy.expected_conflicts_note`).
`meatball-casserole` initially declared no `vegetarian` conflict: `MEAT_TERMS` has no "meatball"
or "meat" term, and the line was named only "frozen meatballs", so a vegetarian household would
have been offered it with no warning. The owner confirmed 2026-09-06 that the meatballs are beef
or turkey; the ingredient is now named **"beef or turkey meatballs"**, both words are
`MEAT_TERMS`, and the entry declares `vegetarian` and `vegan` correctly. That is an accuracy fix
that happens to match — the opposite of renaming to dodge a match, since the name got *more*
true. Still open, because no accurate name fixes it: `crockpot-barbecue-chicken` and
`crockpot-chicken-tacos` declare no `dairy` conflict for their buttermilk-based ranch dressing
and ranch seasoning lines, which `DAIRY`'s terms do not match. Adding "ranch" is an MVP-009 rule
change with a `RULE_VERSION` bump, not a content edit.

**Held back for OPT-004** (`refs/recipes-pending.json`, transcribed and import-ready): three
sides — honey glazed carrots, roasted green beans, roasted vegetables — and two breakfast or
baking dishes — banana bread, apple cinnamon roll casserole. The planner covers dinner only and
`MealScope::dinner_only()` is still the default, and a side is not a meal, so shipping them
today would make banana bread a candidate for Tuesday dinner.

**Roster after this addition:** 59 authored, **49 shipping** (24 federal + 25 owner), 10
pending — the ten Kimatta `original` entries, which still need a cook. Catalog 164.
Coverage across the shipped set: vegan-safe 7, vegetarian-safe 10, gluten-free 34, omnivore 39.
