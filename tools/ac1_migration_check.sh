#!/usr/bin/env bash
# FIX-001 AC-1: exercise the v12 -> v13 migration inside the real app on a real Android runtime.
#
#   bash tools/ac1_migration_check.sh [serial]
#
# The Rust fixture tests already prove the SQL. What this adds is the part only a device can
# show: the shipped app opening a genuine pre-FIX-001 database and coming back with the diced
# tomato lines remapped.
#
# It needs the emulator (or a phone) up first:  bash tools/emulator.sh up
# Every adb call goes through tools/emulator.sh, which owns the WSL->Windows bridge (D-033).

set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
APP_ID=dev.mealmate.temp
FIXTURE_NAME=v12_pre_fix001.db
FIXTURE="$ROOT/build/ac1/$FIXTURE_NAME"
REMOTE_DIR="/sdcard/Android/data/$APP_ID/files"
SERIAL=${1:-}

say()  { printf '\n== %s\n' "$*"; }
fail() { printf '  FAIL  %s\n' "$*" >&2; }

adb_() {
  if [ -n "$SERIAL" ]; then
    bash "$ROOT/tools/emulator.sh" adb -s "$SERIAL" "$@"
  else
    bash "$ROOT/tools/emulator.sh" adb "$@"
  fi
}

say "Building the pre-FIX-001 v12 fixture"
# Regenerated every run rather than committed: it is derived from the starter manifest, and a
# stale checked-in copy would quietly stop matching the content it is supposed to predate.
mkdir -p "$ROOT/build/ac1"
export PATH="$HOME/.cargo/bin:$PATH"
(cd "$ROOT/rust" && cargo run --quiet -p kimatta-storage --example make_v12_fixture -- "$FIXTURE")

say "Pushing it where the app can read it"
# The app's external files directory: readable by the app without root, and outside the APK, so
# no test data ships to users. Created here because it exists only after a first launch.
adb_ shell mkdir -p "$REMOTE_DIR"
adb_ push "$FIXTURE" "$REMOTE_DIR/$FIXTURE_NAME"

say "Running the on-device migration test"
cd "$ROOT"
export PATH="$HOME/development/flutter/bin:$PATH"
if [ -n "$SERIAL" ]; then
  flutter test integration_test/migration_v12_test.dart -d "$SERIAL"
else
  flutter test integration_test/migration_v12_test.dart
fi
rc=$?

say "Cleaning up"
adb_ shell rm -f "$REMOTE_DIR/$FIXTURE_NAME" || true

if [ $rc -ne 0 ]; then
  fail "AC-1 migration check did not pass"
  exit $rc
fi
printf '\n  PASS  v12 -> v13 migrated in the app on this device\n'
