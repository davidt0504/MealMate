# MealMate (dev)

A household meal-planning app: recipes, meal plans, a pantry-aware shopping list, and
the loop between them. Android is the MVP and initial-launch platform; iOS is the first
post-launch priority.

**`MealMate` is a temporary internal codename, not the public brand.** It was retired as
the intended public name by `docs/ROADMAP.md` D-019 after a naming-clearance check. The
scaffold therefore carries a deliberately temporary development identity — Android
`applicationId dev.mealmate.temp`, display name `MealMate (dev)`, Dart package
`meal_mate` — chosen so it is unmistakably disposable. `DEC-004` decides the real name and
production application ID, and must replace this identity before `MVP-018`, `MVP-021`, or
`MVP-022` binds an identifier to Firebase, App Links, signing, or Play.

## Optional repository cleanup hook

Windows download metadata can appear in WSL as literal `*:Zone.Identifier`
files. Git ignores these files, and contributors may optionally enable the
repository's visible pre-commit cleanup hook:

```bash
git config --local core.hooksPath .githooks
```

When enabled, the hook prints and removes matching metadata files before each
commit. The cleanup sweeps the entire worktree, including unstaged and
git-ignored directories, not only the paths being committed. It refuses the
commit if one has already been staged. Disable the hook
at any time with:

```bash
git config --local --unset core.hooksPath
```
