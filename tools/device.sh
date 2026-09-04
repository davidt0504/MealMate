#!/usr/bin/env bash
# Install the release APK on a physical Android device, or hand it to Windows for a phone
# with no developer options. Emulator bring-up stays in tools/emulator.sh (D-033); this
# script knows nothing about AVDs and never starts one. Every adb call goes through
# `tools/emulator.sh adb` so the WSL -> Windows socket derivation keeps one authority.
#
#   bash tools/emulator.sh adb devices      # if this works, the Windows adb server is up
#   bash tools/emulator.sh up               # only if it does not. Attach and unlock the phone
#                                           # FIRST: `up` checks for a device once, immediately,
#                                           # and launches the AVD if none has enumerated yet
#   bash tools/device.sh install [serial]   # adb install -r the release APK
#   bash tools/device.sh export  [dir]      # copy it under the Windows user profile
#
# This script never builds. Build first:  flutter build apk --release
#
# `bash tools/…`, never `./`: both scripts are tracked 100644 and this repo sets
# core.fileMode=false, so a direct call exits 126. Every invocation this file PRINTS carries
# the prefix too -- those hints are pasted by an operator whose command just failed.
#
# Two value functions, `pick_serial` and `win_downloads`, return their result on stdout and are
# read through $( ). Everything else writes to stderr via say/pass/fail/info. There is no fd-9
# dance as in emulator.sh, so nothing on those two call paths may print to stdout -- a stray
# line there is silently captured into the returned value.
#
# emulator.sh:40-41's measured rule -- every $( ) assignment carries `|| true` or sits in a
# condition, because a bare one exits the script BEFORE its own guard runs -- is followed below.
# `dest=` is the one exemption: it runs `date -r "$APK"` after `[ -f "$APK" ]` has passed, and a
# `|| true` there would build a filename out of an empty string instead of failing.

set -euo pipefail

# Guarded exactly as emulator.sh:224-225 guards the byte-identical expression.
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd) \
  || { printf '  FAIL  cannot resolve the repo root from %s\n' "${BASH_SOURCE[0]}" >&2; exit 1; }
APK="$ROOT/build/app/outputs/flutter-apk/app-release.apk"
APP_ID=dev.mealmate.temp                    # temporary applicationId pending DEC-004

say()  { printf '%s\n'         "$*" >&2; }
pass() { printf '  PASS  %s\n' "$*" >&2; }
fail() { printf '  FAIL  %s\n' "$*" >&2; }
info() { printf '  ..    %s\n' "$*" >&2; }

adb_() { bash "$ROOT/tools/emulator.sh" adb "$@"; }

# The release APK, never the debug one sitting beside it in the same directory: a debug build
# runs the Dart VM in JIT and ships the Vulkan validation layers, so anything timed on it is
# meaningless.
require_apk() {
  [ -f "$APK" ] || {
    fail "no release APK at $APK"
    info "build it first: flutter build apk --release"
    return 1
  }
  say "release APK built $(date -r "$APK" '+%Y-%m-%d %H:%M')"
  # A printed date is a guard only if someone reads it, and a days-old app-release.apk sat in
  # this exact path for a week. `-type f` matters: without it the start directories are
  # themselves candidates, `-quit` fires on the first, and the message names `lib` -- something
  # the operator cannot act on -- while a `git checkout` that changed nothing semantically moves
  # directory mtimes and fires it spuriously.
  #
  # The android/ entries are named individually rather than as `android`: that directory also
  # holds .gradle/ and .kotlin/, whose state files Gradle flushes AFTER the APK task finishes
  # (measured: local.properties 08:32:53 at build start, .gradle/file-system.probe 08:33:02 and
  # fileHashes.bin 08:33:03 at build end, against an 08:33 APK). Including it wholesale would
  # print STALE on every single invocation, naming a path the operator cannot act on -- the
  # exact defect `-type f` was added to remove, made permanent. rust/target and build/ are
  # excluded for the same reason: they churn on every command.
  #
  # Warned, not refused: installing an older build on purpose is legitimate, and a hard gate
  # with no override is the kind of thing that gets worked around.
  local newer
  newer=$(find "$ROOT/lib" "$ROOT/rust/src" "$ROOT/rust/crates" "$ROOT/rust/Cargo.toml" \
               "$ROOT/rust/Cargo.lock" "$ROOT/rust/rust-toolchain.toml" "$ROOT/hook" \
               "$ROOT/android/app/src" "$ROOT/android/app/build.gradle.kts" \
               "$ROOT/android/build.gradle.kts" "$ROOT/android/settings.gradle.kts" \
               "$ROOT/android/gradle.properties" \
               "$ROOT/android/gradle/wrapper/gradle-wrapper.properties" \
               "$ROOT/pubspec.yaml" "$ROOT/pubspec.lock" \
               -type f -newer "$APK" -print -quit 2>/dev/null) || true
  [ -z "$newer" ] || fail "STALE -- $newer is newer than this APK; rebuild with 'flutter build apk --release'"
}

# States other than `device` are the common first failure -- a phone whose RSA prompt has not
# been accepted reads `unauthorized` -- so they are reported rather than filtered away into a
# bare "no device found", which sends the operator looking at the cable.
#
# `NF &&` is load-bearing: `adb devices` ends with a blank line, so without it awk sees one
# empty record, both conditions hold vacuously, and the zero-device case reports a phantom
# " ()" as "attached but not usable" -- the exact misdiagnosis this function exists to avoid.
pick_serial() {
  local raw rows ready other
  # adb's own status is checked here, separately from the filter below: folding them into one
  # pipeline under `pipefail` reports "adb is unavailable" when adb worked fine and simply
  # nothing was plugged in -- `grep -v` legitimately exits 1 when the header is the only line.
  raw=$(adb_ devices) || {
    fail "adb is unavailable -- the Windows adb server is not listening"
    info "see this script's header; usually: bash tools/emulator.sh up"
    return 1
  }
  rows=$(printf '%s\n' "$raw" | tr -d '\r' | grep -vE '^List of devices attached' \
         | sed '/^[[:space:]]*$/d') || true
  ready=$(printf '%s\n' "$rows" | awk 'NF && $1 !~ /^emulator-/ && $2 == "device" {print $1}') || true
  # The whole remainder of the record, not $2: adb emits multi-word states such as
  # `no permissions (user in plugdev group…)`, and printing $2 alone yields "SERIAL (no)".
  other=$(printf '%s\n' "$rows" \
          | awk 'NF && $1 !~ /^emulator-/ && $2 != "device" {
                   s=$0; sub(/^[^ \t]+[ \t]+/,"",s); print $1" ("s")" }') || true
  case "$(printf '%s' "$ready" | grep -c . || true)" in
    1) printf '%s' "$ready" ;;
    0) fail "no physical device is ready"
       if [ -n "$other" ]; then
         info "attached but not usable: $(printf '%s' "$other" | tr '\n' ' ')"
         info "'unauthorized' means the RSA prompt on the phone has not been accepted yet"
       else
         info "only the emulator is attached, or nothing is"
         info "unlock the phone, set its USB mode to File Transfer, and replug"
       fi
       return 1 ;;
    *) fail "more than one physical device: $(printf '%s' "$ready" | tr '\n' ' ')"
       info "name one: bash tools/device.sh install <serial>"
       return 1 ;;
  esac
}

cmd_install() {
  local serial=${1:-}
  require_apk || return 1
  [ -n "$serial" ] || { serial=$(pick_serial) || return 1; }
  if ! adb_ -s "$serial" install -r "$APK" >&2; then
    fail "install failed on $serial"
    info "INSTALL_FAILED_VERIFICATION_FAILURE (or a 'Blocked by Play Protect' dialog) is"
    info "Play Protect's USB verifier, on by default. Turn off 'Verify apps over USB' in"
    info "Developer options, or:"
    info "  bash tools/emulator.sh adb -s $serial shell settings put global verifier_verify_adb_installs 0"
    info "INSTALL_FAILED_UPDATE_INCOMPATIBLE means an older build signed with a different"
    info "debug key is present. ~/.android/debug.keystore is per-machine (and valid to 2056),"
    info "so this means the installed copy came from a different machine or a lost keystore."
    info "Removing it also erases its data: allowBackup is false and there is no cloud backup"
    info "yet, so export from Settings first if that data matters, then:"
    info "  bash tools/emulator.sh adb -s $serial uninstall $APP_ID"
    return 1
  fi
  pass "installed on $serial -- open 'Kimatta (dev)' on the phone"
}

# Derived, not hardcoded: the WSL account (davidlinux) and the Windows account (David) differ.
win_downloads() {
  local up unix
  up=$(powershell.exe -NoProfile -Command '$env:USERPROFILE' 2>/dev/null | tr -d '\r') || true
  [ -n "$up" ] || {
    fail "could not read %USERPROFILE% from the host"
    info "pass one explicitly: bash tools/device.sh export <dir>"
    return 1
  }
  # Assigned and checked rather than interpolated: a failing wslpath inside the printf argument
  # would silently yield the path "/Downloads", and the caller would report "no such directory:
  # /Downloads" -- a path the operator never named and cannot place.
  unix=$(wslpath -u "$up") || true
  [ -n "$unix" ] || {
    fail "could not translate %USERPROFILE% ($up) to a WSL path"
    info "pass one explicitly: bash tools/device.sh export <dir>"
    return 1
  }
  printf '%s/Downloads' "$unix"
}

cmd_export() {
  local dir=${1:-} dest
  require_apk || return 1
  [ -n "$dir" ] || { dir=$(win_downloads) || return 1; }
  [ -d "$dir" ] || {
    fail "no such directory: $dir"
    info "a OneDrive-redirected Downloads folder lands elsewhere -- pass it explicitly:"
    info "  bash tools/device.sh export <dir>"
    return 1
  }
  # Stamped with the build time so a second build never lands as a mystery duplicate and the
  # phone shows which one it is being asked to install.
  dest="$dir/mealmate-dev-$(date -r "$APK" '+%Y%m%d-%H%M').apk"
  cp -- "$APK" "$dest"
  pass "copied to $dest"
  say  "on Windows: $(wslpath -w "$dest")"
}

case "${1:-}" in
  install) shift; [ $# -le 1 ] || { say "install takes at most one serial (got: $*)"; exit 2; }
           cmd_install "$@" ;;
  export)  shift; [ $# -le 1 ] || { say "export takes at most one directory (got: $*)"; exit 2; }
           cmd_export "$@" ;;
  -h|--help|help) say "usage: bash tools/device.sh {install [serial]|export [dir]}"; exit 0 ;;
  *) say "usage: bash tools/device.sh {install [serial]|export [dir]}"; exit 2 ;;
esac
