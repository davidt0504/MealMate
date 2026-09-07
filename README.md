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

## Installing on a phone

The app is local-only — no backend, and the release manifest carries no `INTERNET`
permission — so a sideloaded build is a complete, usable app with no setup beyond
installing it. Two phones are two independent households: there is no sync until `MVP-018`.

## Downloading an APK from GitHub

Every pushed version tag such as `v1.0.0` triggers the **Publish Android APK** GitHub
Actions workflow. It builds the universal release APK and attaches it to that tag's GitHub
Release, where it can be downloaded directly from the repository's **Releases** page.

To publish a version, first update `version:` in `pubspec.yaml` (the build number after `+`
must be higher than every previously published Android build), commit the release-ready
source, then run:

```bash
git tag v1.0.0
git push origin v1.0.0
```

When the workflow succeeds, download `MealMate-v1.0.0.apk` from the newly created release.
You can also re-run it from **Actions → Publish Android APK → Run workflow**, providing an
existing tag. The current project signs release builds with its debug key, so these downloads
are appropriate for internal testing and sideloading—not public production distribution. Use a
dedicated release keystore stored as GitHub Actions secrets before sharing broadly.

**Build once.** One universal APK, all three ABIs, so it installs on any Android phone:

```bash
flutter build apk --release
```

### Path A — your own phone, over USB debugging

Fastest to iterate on, and the only path that gives you `logcat` and `pm clear`.

1. Enable **Developer options**: Settings → About phone → Software information → tap
   *Build number* seven times. Then turn on **USB debugging**, and turn **off**
   *Verify apps over USB* — Play Protect's USB verifier otherwise fails the install with
   `INSTALL_FAILED_VERIFICATION_FAILURE`, which reads like a corrupt APK.
2. **Unlock the phone, set its USB mode to File Transfer**, then plug it into the **Windows
   host** (not WSL) and accept the RSA prompt on the phone's screen.
3. `bash tools/emulator.sh adb devices` — if the phone is listed, the adb server is already
   running and you do not need `up` at all.
4. `bash tools/device.sh install`

**Never run `up` before the phone is listed.** `cmd_up` (`tools/emulator.sh:124-141`) checks
for a device exactly once, immediately after the adb server comes up. A phone that has not
registered yet — cable just inserted, driver still loading, or simply not plugged in — leaves
the emulator-process count at zero and `up` **launches the AVD**. Accept the RSA prompt before
running `up` too, or it polls for a full 180 s and then reports something about emulator
processes, which is not what went wrong.

`emulator.sh` prints its own hints **without** the `bash` prefix (`tools/emulator.sh:157`,
`:209`, `:221`, `:255-256`, and its header). Unless the executable bit has been repaired, add
`bash ` when pasting one — on a bridge-down failure you will see its unprefixed suggestion
first and this script's correct one second.

**With the emulator also running,** `device.sh install` is unaffected — it always passes
`-s` — but `emulator.sh`'s `up`, `status` and `reset` call `adb` without a serial and break
(`boot_done` at `:65` and `cmd_reset` at `:210-211`). Prefix them:

```bash
ANDROID_SERIAL=<serial> bash tools/emulator.sh reset
bash tools/emulator.sh adb -s <serial> shell pm clear dev.mealmate.temp   # or reset one phone directly
```

### Path B — a phone with no developer options

Nothing has to be enabled on the phone for this. Use it for a second household member.

```bash
bash tools/device.sh export          # copies the APK under the Windows user profile
```

Then transfer the file (USB file transfer, Quick Share, or Drive) and tap it. Three
prerequisites, each of which fails with a message that reads like a corrupt APK:

1. **Auto Blocker** off — Settings → Security and privacy → Auto Blocker.
2. **Install unknown apps** granted to whichever app is doing the transferring.
3. **Play Protect's on-device scan** — separate from Path A's USB verifier, and not
   disableable from Developer options, which this path does not have. One UI shows
   "Unsafe app blocked" / "App not installed"; the escape is *More details → Install
   anyway*, or turn app scanning off in Play Store → profile → Play Protect → settings.

### Fallback — wireless debugging

`adb pair` against the Windows adb server. Removes the cable and any USB-driver problem,
still needs Developer options, and needs both devices on the same non-isolated network.

### Caveats

- The release build is signed with WSL's `~/.android/debug.keystore` (valid to 2056, so
  expiry is not a concern) — but that key is **per-machine**. Building on a different machine,
  or losing that file to a WSL reset, produces a different signature, and Android then refuses
  the update: reinstalling means uninstalling first, which **erases the app's data**
  (`allowBackup=false`, no cloud backup yet). **Back up `~/.android/debug.keystore`** if
  anyone but you is running these builds. The release build type is not `debuggable` — this is
  a real release build that merely carries a debug *signature*.
- The Settings export is the only sanctioned way data leaves the phone.
- A phone plugged in *after* the host adb server started may not enumerate until
  `bash tools/emulator.sh down` and a fresh `up`.

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
