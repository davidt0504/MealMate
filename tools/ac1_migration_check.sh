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
#
# Why not `flutter test integration_test/...`: that installs the app and then connects to it
# through a port adb forwards. The adb server runs on Windows (D-022), so the port opens on the
# Windows loopback and WSL is refused. Instead the test is built as the app's entrypoint,
# launched on the device, and its verdict read back from logcat.

set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
APP_ID=dev.mealmate.temp
FIXTURE_NAME=v12_pre_fix001.db
FIXTURE="$ROOT/build/ac1/$FIXTURE_NAME"
REMOTE_DIR="/sdcard/Android/data/$APP_ID/files"
SERIAL=${1:-}
TIMEOUT_S=120

say()  { printf '\n== %s\n' "$*"; }
fail() { printf '  FAIL  %s\n' "$*" >&2; }

adb_() {
  if [ -n "$SERIAL" ]; then
    bash "$ROOT/tools/emulator.sh" adb -s "$SERIAL" "$@"
  else
    bash "$ROOT/tools/emulator.sh" adb "$@"
  fi
}

# Runs on every exit, including a failed test: `set -e` would otherwise stop the script before
# the cleanup and leave the fixture sitting on the device.
cleanup() {
  adb_ shell rm -f "$REMOTE_DIR/$FIXTURE_NAME" >/dev/null 2>&1 || true
}
trap cleanup EXIT

export PATH="$HOME/development/flutter/bin:$HOME/.cargo/bin:$PATH"

say "Building the pre-FIX-001 v12 fixture"
# Regenerated every run rather than committed: it is derived from the starter manifest, and a
# stale checked-in copy would quietly stop matching the content it is supposed to predate.
mkdir -p "$ROOT/build/ac1"
(cd "$ROOT/rust" && cargo run --quiet -p kimatta-storage --example make_v12_fixture -- "$FIXTURE")

say "Building the test as the app's entrypoint"
cd "$ROOT"
flutter build apk --debug -t integration_test/migration_v12_test.dart

say "Installing and pushing the fixture"
adb_ install -r build/app/outputs/flutter-apk/app-debug.apk
# The app's external files directory: readable by the app without root, and outside the APK, so
# no test data ships to users.
adb_ shell mkdir -p "$REMOTE_DIR"
adb_ push "$FIXTURE" "$REMOTE_DIR/$FIXTURE_NAME"

say "Running on the device"
adb_ logcat -c
# `am start -W` can report `Status: timeout` while the activity still starts, so the verdict
# comes from the test's own output, never from this command's status.
adb_ shell am start -W -n "$APP_ID/.MainActivity" >/dev/null 2>&1 || true

log=$(mktemp)
verdict=""
for _ in $(seq 1 $((TIMEOUT_S / 2))); do
  sleep 2
  adb_ logcat -d -s flutter:I > "$log" 2>/dev/null || true
  if grep -q 'All tests passed' "$log"; then verdict=pass; break; fi
  if grep -q 'Some tests failed' "$log"; then verdict=fail; break; fi
done

sed -n 's/^.*flutter *: //p' "$log"
rm -f "$log"

case "$verdict" in
  pass) printf '\n  PASS  schema 12 migrated to latest in the app on this device\n' ;;
  fail) fail "AC-1 migration check failed -- see the test output above"; exit 1 ;;
  *)    fail "no test verdict in logcat after ${TIMEOUT_S}s"; exit 1 ;;
esac
