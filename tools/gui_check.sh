#!/usr/bin/env bash
# Pre-release GUI checks on the emulator: scripted user journeys, then random-tap monkey runs.
#
#   bash tools/gui_check.sh journeys [serial]   # integration_test/journeys_test.dart, ~3-5 min
#   bash tools/gui_check.sh monkey   [serial]   # adb monkey on the debug app, ~3 min
#
# Env: GUI_ONLY=J4 runs one journey; GUI_TIMEOUT (s, default 2100) bounds the journeys run;
# GUI_SEEDS (default "101 202 303") and GUI_EVENTS (default 1500) shape the monkey run.
# Logs land in build/gui_check/.
#
# Emulator only, and refused anywhere else: the journeys delete the app's database and monkey
# can tap Restore or Start fresh. The verdict is read from logcat for the same reason as
# tools/ac1_migration_check.sh -- WSL cannot reach the port `flutter test integration_test` needs.
# ponytail: install/logcat/poll duplicate ~20 lines of ac1_migration_check.sh; that script pushes
# a fixture between install and launch, so fold it in here only if a third device check appears.

set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
APP_ID=dev.mealmate.temp
TEST_FILE=integration_test/journeys_test.dart
APK=build/app/outputs/flutter-apk/app-debug.apk
OUT="$ROOT/build/gui_check"
CMD=${1:-}
SERIAL=${2:-}
# The release gate is per commit, so logs and PASS lines name the tree they ran on.
REV=$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)$(
  [ -z "$(git -C "$ROOT" status --porcelain 2>/dev/null)" ] || echo +dirty)
STAMP=$(date +%Y%m%dT%H%M%S)-$REV

say()  { printf '\n== %s\n' "$*"; }
info() { printf '  %s\n' "$*"; }
fail() { printf '  FAIL  %s\n' "$*" >&2; }

adb_() {
  if [ -n "$SERIAL" ]; then
    bash "$ROOT/tools/emulator.sh" adb -s "$SERIAL" "$@"
  else
    bash "$ROOT/tools/emulator.sh" adb "$@"
  fi
}

case "$CMD" in
  journeys|monkey) ;;
  *) printf 'usage: bash tools/gui_check.sh journeys|monkey [serial]\n' >&2; exit 2 ;;
esac

# Only stops this script running twice; the pgrep below covers the other users of the AVD and
# of build/app, which a second session may be driving.
exec 9>/tmp/kimatta-gui-check.lock
flock -n 9 || { fail "another gui_check.sh run holds /tmp/kimatta-gui-check.lock"; exit 1; }
busy=$(pgrep -af 'flutter(_tools\.[a-z]+)? build apk|ac1_migration_check|emulator\.sh seed' || true)
if [ -n "$busy" ]; then
  fail "something else is using the emulator or the build directory:"
  printf '%s\n' "$busy" >&2
  exit 1
fi

# The guard is the reason this script may wipe data: both checks must hold, and SERIAL is pinned
# afterwards so a phone plugged in mid-run never receives a command.
if [ -z "$SERIAL" ]; then
  SERIAL=$(adb_ devices | tr -d '\r' | awk 'NF && $1 ~ /^emulator-/ && $2 == "device" {print $1}' \
           | head -1) || true
fi
qemu=$(adb_ shell 'getprop ro.boot.qemu; getprop ro.kernel.qemu' 2>/dev/null | tr -d '\r') || true
if [[ ! "$SERIAL" =~ ^emulator- ]] || ! grep -qx 1 <<<"$qemu"; then
  fail "no emulator to run on (serial '${SERIAL:-none}'); these checks wipe app data, so a phone is refused"
  info "start one first: bash tools/emulator.sh up"
  exit 1
fi
info "target: $SERIAL"

export PATH="$HOME/development/flutter/bin:$HOME/.cargo/bin:$PATH"
mkdir -p "$OUT"

# A release build from device.sh or CI carries another signature; only the emulator gets here.
install_apk() {
  local out
  if ! out=$(adb_ install -r "$APK" 2>&1); then
    if grep -q INSTALL_FAILED_UPDATE_INCOMPATIBLE <<<"$out"; then
      info "installed app has another signature; uninstalling it on the emulator"
      adb_ uninstall "$APP_ID" >/dev/null
      adb_ install "$APK"
    else
      printf '%s\n' "$out" >&2
      return 1
    fi
  fi
  adb_ shell pm clear "$APP_ID" >/dev/null
  adb_ logcat -G 16M >/dev/null 2>&1 || info "logcat -G refused; default buffer"
  adb_ logcat -c
}

run_journeys() {
  local args=() log verdict="" n s want ran
  [ -n "${GUI_ONLY:-}" ] && args+=("--dart-define=JOURNEY_ONLY=$GUI_ONLY")
  local timeout=${GUI_TIMEOUT:-2100}
  log="$OUT/journeys-$STAMP.log"

  say "Building the journeys as the app's entrypoint"
  cd "$ROOT"
  flutter build apk --debug -t "$TEST_FILE" "${args[@]}"
  say "Installing"
  install_apk
  say "Running on $SERIAL (up to ${timeout}s)"
  adb_ shell am start -W -n "$APP_ID/.MainActivity" >/dev/null 2>&1 || true
  for _ in $(seq 1 $((timeout / 2))); do
    sleep 2
    adb_ logcat -d -s flutter:I > "$log" 2>/dev/null || true
    if grep -q 'All tests passed' "$log"; then verdict=pass; break; fi
    if grep -q 'Some tests failed' "$log"; then verdict=fail; break; fi
  done
  sed -n 's/^.*flutter *: //p' "$log"

  printf '\n  note: %s and the emulator now hold the test harness -- opening the app runs\n' "$APK"
  info "the suite and wipes its data. Run 'flutter build apk --debug' before using either."
  info "full log: $log"

  case "$verdict" in
    fail) fail "journeys failed -- see the output above"; return 1 ;;
    "")   fail "no verdict after ${timeout}s; last started: $(grep -o 'journey start: .*' "$log" \
              | tail -1 || echo none)"; return 1 ;;
  esac
  # `+N: All tests passed!` or `+N ~S: ...`; N counts setUpAll and tearDownAll as passes.
  n=$(grep -o '+[0-9]*\( ~[0-9]*\)\?: All tests passed' "$log" | tail -1)
  s=$(sed -n 's/.*~\([0-9]*\).*/\1/p' <<<"$n"); s=${s:-0}
  n=$(sed 's/^+\([0-9]*\).*/\1/' <<<"$n")
  ran=$((n - 2))
  want=$(grep -c '^  journey(' "$TEST_FILE")
  if [ $((ran + s)) -ne "$want" ]; then
    fail "$ran ran + $s skipped, but the file declares $want journeys"; return 1
  fi
  if [ -n "${GUI_ONLY:-}" ] && [ "$ran" -lt 1 ]; then
    fail "GUI_ONLY=$GUI_ONLY matched no journey"; return 1
  fi
  printf '\n  PASS  %s journeys ran, %s skipped at %s\n' "$ran" "$s" "$REV"
}

run_monkey() {
  local seeds=${GUI_SEEDS:-101 202 303} events=${GUI_EVENTS:-1500} seed txt log hits bad=0

  say "Building the debug app (also replaces a journeys harness left on the emulator)"
  cd "$ROOT"
  flutter build apk --debug
  say "Installing"
  install_apk
  for seed in $seeds; do
    txt="$OUT/monkey-$seed-$STAMP.txt"
    say "Seed $seed: $events events"
    # Monkey exits non-zero on a crash; the checks below read its output instead.
    adb_ shell monkey -p "$APP_ID" -s "$seed" --throttle 150 --pct-syskeys 0 \
      --pct-appswitch 0 -v "$events" > "$txt" 2>&1 || true
    if grep -q '// CRASH\|// NOT RESPONDING' "$txt"; then
      fail "seed $seed: monkey reports a crash or ANR (GUI_SEEDS=$seed reproduces it)"
      grep -A12 '// CRASH\|// NOT RESPONDING' "$txt" | head -40 >&2
      bad=1
    elif ! grep -q "Events injected: $events\$" <<<"$(tr -d '\r' < "$txt")"; then
      fail "seed $seed: no 'Events injected: $events' -- the run stopped early; see $txt"
      bad=1
    else
      info "seed $seed: $events events injected"
    fi
  done

  log="$OUT/monkey-$STAMP.log"
  adb_ logcat -d -v brief > "$log" 2>/dev/null || true
  info "flutter lines in the log: $(grep -c '/flutter' "$log" || true)"
  # Other apps on the AVD (gms, systemui) crash often; only this app's fatal counts.
  # ponytail: Rust panics go to stderr (dropped on Android) and screens turn caught errors into
  # snackbars, so this sees crashes, ANRs and uncaught Dart errors only; a debug-only logcat line
  # in the panic hook / describeFailure is the upgrade if monkey must catch those.
  hits=$(grep -n 'EXCEPTION CAUGHT BY\|E/flutter\|Unhandled Exception' "$log" || true)
  if grep -A2 'FATAL EXCEPTION' "$log" | grep -q "Process: $APP_ID"; then
    hits+=$'\n'$(grep -n -A2 'FATAL EXCEPTION' "$log")
  fi
  if [ -n "$hits" ]; then
    fail "app errors in the log (seeds: $seeds):"
    head -40 <<<"$hits" >&2
    bad=1
  fi
  if [ "$(adb_ shell settings get global airplane_mode_on | tr -d '\r')" = 1 ]; then
    info "note: monkey left airplane mode on; turn it off before other emulator work"
  fi
  info "logs: $OUT/monkey-*-$STAMP.txt and $log"

  [ "$bad" = 0 ] || return 1
  printf '\n  PASS  %s seeds x %s events, no crash or app error at %s\n' \
    "$(wc -w <<<"$seeds")" "$events" "$REV"
}

case "$CMD" in
  journeys) run_journeys ;;
  monkey)   run_monkey ;;
esac
