# MealMate (dev)

A household meal-planning app: recipes, meal plans, a pantry-aware shopping list, and
the loop between them. Android is the MVP and initial-launch platform; iOS is the first
post-launch priority.

## Architecture direction (PRD v3)

Adopted 2026-08-24 (`docs/ROADMAP.md` D-028). The app is a focused consumer food product built
as the first controller on a deliberately small Household Control Kernel:

- **Flutter/Dart** owns screens, navigation, accessibility, animations, transient view state, and
  platform adapters.
- A **generated `flutter_rust_bridge` boundary**, kept coarse (service-level DTO calls, never
  per-field access or SQLite handles), connects the UI to the core.
- **Rust** owns household/member identity, durable food state, policies/constraints, the
  deterministic planner (Cover My Week), shopping-list derivation, coverage assessment, and the
  authoritative local **SQLite** store. Crate direction: `household-core <- food-domain <-
  kimatta-application <- kimatta-bridge`; `kimatta-storage` depends on `household-core` (and on `food-domain` once it exists) and is called by `kimatta-application`, or directly by the bridge until that crate exists; the kernel never imports food types.

Current state: **Rust foundation landed (MVP-002)** — `household-core` (typed household/member identity) and `kimatta-storage` (SQLite, migration v1) are the first domain crates; the bridge `health_check` opens the real database. See
`docs/V3_IMPLEMENTATION_STATUS.md` for what is complete, deferred, and blocked;
`docs/V3_MIGRATION_PLAN.md` for the Phase-0 inventory and phase → card map; `docs/PRD_v3.md` and
`docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` for the product and engineering constitution. Rust
toolchain and build commands (PRE-002 command contract in `docs/ROADMAP.md`; native-assets backend,
Rust 1.98.0, `flutter_rust_bridge` 2.13.0):

```bash
export PATH="$HOME/.cargo/bin:$PATH"          # once per shell; rustup lives in ~/.cargo, ~/.rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain 1.98.0 --profile minimal
rustup target add --toolchain 1.98.0 aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
rustup component add --toolchain 1.98.0 rustfmt clippy
cargo install flutter_rust_bridge_codegen --version 2.13.0 --locked

flutter_rust_bridge_codegen generate       # after any change under rust/src/api
(cd rust && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace)
(cd rust && cargo build --release) && flutter test   # stale .so = false pass; never run alone
flutter build apk --debug
flutter build apk --release                # ships libkimatta_bridge.so for arm64-v8a, armeabi-v7a, x86_64
```

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
