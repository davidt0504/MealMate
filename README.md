# MealMate

MealMate is a meal planning app with a smart pantry, recipe storage, shopping lists, and social features. The open-source repo includes the app's code, docs, and resources.

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
