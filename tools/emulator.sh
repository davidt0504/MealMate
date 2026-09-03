#!/usr/bin/env bash
# Bring up the Windows-host Android emulator + WSL adb bridge (D-022 shape). See D-033.
#
# bash, not sh: /dev/tcp is a bash builtin; nc is not guaranteed present.
#
#   human:  S=$(tools/emulator.sh up) && [ -n "$S" ] && export ADB_SERVER_SOCKET=$S
#   agent:  tools/emulator.sh adb devices
#   reset:  tools/emulator.sh reset      # pm clear — next launch is a first run
#
# Never `adb reboot`: the AVD restores a Quick Boot snapshot and installed APKs vanish.
# Process names depend on how the AVD was started (measured): via emulator.exe you get BOTH
# `emulator` and `qemu-system-x86_64`; via Android Studio, only `qemu-system-x86_64`. Probe with
# `emulator|qemu`, and prefer `taskkill /IM qemu-system-x86_64.exe /F` — it works in both cases.
# Assumes WSL2 NAT networking — see the gateway-derivation entry in KNOWN_ISSUES.md.
# One-time elevated host setup is setup-adb-bridge.ps1 (host copy:
# C:\Users\David\meal_mate_tools\, source of truth: tools/windows/). Run it from an elevated
# PowerShell on the host: invoked unelevated it returns rc 0 as soon as the elevated child is
# *launched*, so its status says nothing about what the child did (a declined UAC gives rc 1).

set -euo pipefail

AVD=${EMU_AVD:-pre001_avd}
BOOT_TIMEOUT=${EMU_BOOT_TIMEOUT:-180}
EMU_EXE='C:\Android\sdk\emulator\emulator.exe'
PORT=5037                                   # constant: baked into the PS1's firewall rule
APP_ID=dev.mealmate.temp                    # temporary applicationId pending DEC-004
LOCK=/tmp/kimatta-emulator-up.lock
LOCK_WAIT=300
ERRLOG=""                                   # created lazily; `exec` in cmd_adb skips EXIT traps
trap '[ -n "$ERRLOG" ] && rm -f "$ERRLOG" || true' EXIT

exec 9>&1 1>&2                  # fd 9 = real stdout; stray stdout is forced to stderr

say()  { printf '%s\n'         "$*" >&2; }
pass() { printf '  PASS  %s\n' "$*" >&2; }
fail() { printf '  FAIL  %s\n' "$*" >&2; }
info() { printf '  ..    %s\n' "$*" >&2; }
emit() { printf '%s\n' "$1" >&9; }

# Every $( ) assignment carries `|| true` or sits in a condition: a bare one exits the script
# under `set -euo pipefail` BEFORE its own guard runs (measured).
gateway() { ip route | awk '/default/ {print $3; exit}'; }

socket() {                      # value function: the ONLY helper that writes stdout
  local gw; gw=$(gateway) || return 1
  [ -n "$gw" ] || return 1
  printf 'tcp:%s:%d' "$gw" "$PORT"
}

port_open() {
  local gw; gw=$(gateway) || return 1
  [ -n "$gw" ] || return 1
  timeout 2 bash -c "exec 3<>/dev/tcp/$gw/$PORT" 2>/dev/null
}

adb_out() {                     # bounded; stderr kept for diagnosis rather than discarded
  local s; s=$(socket) || return 0        # no gateway -> never fall back to a LOCAL adb server
  [ -n "$ERRLOG" ] || ERRLOG=$(mktemp)
  ADB_SERVER_SOCKET=$s timeout 10 adb "$@" 2>>"$ERRLOG" || true
}
adb_errors()     { [ -n "$ERRLOG" ] && [ -s "$ERRLOG" ] && tail -3 "$ERRLOG" | tr '\n' ' ' || true; }
devices_raw()    { adb_out devices | tr -d '\r' | grep -vE '^List of devices attached' | sed '/^[[:space:]]*$/d'; }
device_ready()   { devices_raw | grep -qE '[[:space:]]device$'; }
device_present() { [ -n "$(devices_raw || true)" ]; }   # ANY state: offline/unauthorized/authorizing
boot_done()      { [ "$(adb_out shell getprop sys.boot_completed | tr -d '\r')" = "1" ]; }
ready()          { device_ready && boot_done; }  # the ONE readiness predicate, used by all sites

ps_out() { powershell.exe -NoProfile -Command "$1" 2>/dev/null | tr -d '\r'; }

# Both probes return non-zero when the probe ITSELF failed, distinct from a negative result.
qemu_count() {                  # 'emulator|qemu' per the measured rule in the memory file
  local n; n=$(ps_out "(Get-Process | Where-Object { \$_.ProcessName -match 'emulator|qemu' } | Measure-Object).Count") || true
  case "$n" in ''|*[!0-9]*) return 1 ;; *) printf '%s' "$n" ;; esac
}
avd_locked() {
  local r; r=$(ps_out "Test-Path \"\$env:USERPROFILE\.android\avd\\$AVD.avd\hardware-qemu.ini.lock\"") || true
  case "$r" in True) return 0 ;; False) return 1 ;; *) return 2 ;; esac   # 2 = probe failed
}

host_adb() {
  if [ -n "${EMU_ADB_EXE:-}" ]; then printf '%s' "$EMU_ADB_EXE"; return 0; fi
  local hits pick
  hits=$(ps_out "where.exe adb.exe" | sed '/^[[:space:]]*$/d') || true
  [ -n "$hits" ] || return 1
  pick=$(printf '%s\n' "$hits" | grep -i 'WinGet' | head -1) || true
  [ -n "$pick" ] || pick=$(printf '%s\n' "$hits" | head -1) || true
  printf '%s' "$pick"
}

accel_ok() {                    # $LASTEXITCODE must reach PowerShell literally
  local ps; ps=$(printf "& '%s' -accel-check; exit \$LASTEXITCODE" "$EMU_EXE") || true
  powershell.exe -NoProfile -Command "$ps" >/dev/null 2>&1
}

cmd_up() {
  local sock i deadline qc
  sock=$(socket) || { fail "no default route; cannot derive the adb socket"; return 1; }

  if port_open && ready; then pass "bridge already up ($sock)"; emit "$sock"; return 0; fi

  exec 8>"$LOCK" || { fail "cannot open $LOCK"; return 1; }
  if ! flock -n 8; then
    info "another 'up' is in progress; waiting up to ${LOCK_WAIT}s"
    flock -w "$LOCK_WAIT" 8 || { fail "lock held longer than ${LOCK_WAIT}s"; return 1; }
  fi
  if port_open && ready; then pass "brought up concurrently ($sock)"; emit "$sock"; return 0; fi

  if ! port_open; then
    local exe; exe=$(host_adb) || { fail "no adb.exe on the host (set EMU_ADB_EXE)"; return 1; }
    info "starting Windows adb server: $exe"
    powershell.exe -NoProfile -Command \
      "Start-Process -WindowStyle Hidden '$exe' -ArgumentList '-a','-P','$PORT','nodaemon','server'" || true
    for i in $(seq 1 10); do port_open && break; sleep 1; done   # sleep: pacing must not rely on SYN drops
    if ! port_open; then
      fail "TCP $PORT on $(gateway) unreachable after starting the server"
      info "observed binds: $(ps_out "netstat -ano | Select-String 'LISTENING' | Select-String ':$PORT '" | tr '\n' ' ')"
      info "a 127.0.0.1-only bind means something local squats the port (wslrelay.exe has done this) — kill that PID on the host"
      info "otherwise run setup-adb-bridge.ps1 -VerifyOnly from an ELEVATED PowerShell on the host (C:\\Users\\David\\meal_mate_tools\\)"
      return 1
    fi
    pass "Windows adb server listening on $PORT"
  fi

  if device_present; then
    info "a device is present but not ready; waiting rather than launching another"
  elif ! qc=$(qemu_count); then
    fail "could not determine whether an emulator process is running; refusing to launch blind"; return 1
  elif [ "$qc" != "0" ]; then
    info "an emulator process is running but has not registered with adb yet; waiting"
  else
    local lrc=0; avd_locked || lrc=$?      # bare `avd_locked` would trip set -e on rc 1 and 2
    case $lrc in
      # A lock with no process is stale by construction — `taskkill` always leaves one, and that
      # is the documented teardown. The emulator clears it on launch, so warn rather than block.
      0) info "AVD $AVD has a stale lock (no emulator process); the emulator clears it on launch" ;;
      2) fail "AVD lock probe failed; refusing to launch blind"; return 1 ;;
    esac
    accel_ok || { fail "emulator.exe -accel-check failed; no hardware acceleration"; return 1; }
    info "launching AVD $AVD"
    powershell.exe -NoProfile -Command "Start-Process '$EMU_EXE' -ArgumentList '-avd','$AVD'" || true
  fi

  deadline=$((SECONDS + BOOT_TIMEOUT))     # wall clock: the loop body contains two 10s timeouts
  while [ "$SECONDS" -lt "$deadline" ]; do
    ready && { pass "device booted"; emit "$sock"; return 0; }
    port_open || { fail "the adb bridge went down while waiting for boot"; return 1; }
    sleep 3
  done
  fail "device did not reach 'device' + boot_completed within ${BOOT_TIMEOUT}s"
  info "emulator processes: $(qemu_count || echo '?'); devices: $(devices_raw | tr '\n' ' ' || true); adb stderr: $(adb_errors)"
  info "an emulator that exited immediately looks identical to a slow boot — check the host"
  return 1
}

cmd_adb() {
  local sock; sock=$(socket) || { fail "no default route"; return 1; }
  port_open || { fail "bridge is down — run 'tools/emulator.sh up' first"; return 1; }
  [ -n "$ERRLOG" ] && rm -f "$ERRLOG" || true   # exec skips the EXIT trap; clean up first
  exec 1>&9 9>&-                # restore real stdout before exec, or adb's output goes to stderr
  ADB_SERVER_SOCKET=$sock exec adb "$@"
}

cmd_status() {
  local gw rc=0 present qc
  gw=$(gateway) || true
  [ -n "$gw" ] && pass "default route: $gw" || { fail "no default route"; rc=1; }
  if port_open; then
    pass "TCP $PORT reachable"
    present=$(devices_raw | tr '\n' ' ') || true
    if ready; then pass "device ready (booted): $present"
    elif [ -n "$present" ]; then fail "device attached but not booted: $present"; rc=1
    else fail "no devices attached"; rc=1
    fi
  else
    fail "TCP $PORT unreachable — skipping adb probes (they would cost ~20s)"; rc=1
  fi
  if qc=$(qemu_count); then
    info "emulator processes: $qc"
    if [ "$qc" = "0" ] && avd_locked; then
      info "stale AVD lock: $AVD.avd/hardware-qemu.ini.lock with no emulator process (normal after taskkill)"
    fi
  else
    fail "emulator process probe failed (PowerShell interop?)"; rc=1
  fi
  [ -n "$(adb_errors)" ] && info "adb stderr: $(adb_errors)" || true
  return $rc
}

cmd_down() {
  local rc=0 i
  if port_open; then
    adb_out kill-server
    for i in 1 2 3; do port_open || break; sleep 1; done   # the socket closes after the ack
    if port_open; then fail "adb server still listening on $PORT"; rc=1
    else pass "remote adb server killed"; fi
  else
    info "no adb server on $PORT; nothing to kill"
  fi
  say "close the emulator on the host with:  taskkill /IM qemu-system-x86_64.exe /F"
  return $rc
}

# Wipes the app's private data (kimatta.db included) so the next launch is a true first run.
# The AVD's Quick Boot snapshot can restore a state where the package is absent (header note);
# `pm clear` on an absent package exits non-zero and the script is `set -euo pipefail`, so
# probe first and treat "not installed" as already clean.
cmd_reset() {
  local sock; sock=$(socket) || { fail "no default route"; return 1; }
  port_open || { fail "bridge is down — run 'tools/emulator.sh up' first"; return 1; }
  if ADB_SERVER_SOCKET=$sock adb shell pm path "$APP_ID" >/dev/null 2>&1; then
    ADB_SERVER_SOCKET=$sock adb shell pm clear "$APP_ID" >&2
  else
    info "reset: $APP_ID not installed; nothing to clear"
  fi
}

# Development seed: today the only durable data is the household, which the app bootstraps
# on first launch — so seeding is install + reset + launch. Grows real fixtures with MVP-007/011.
cmd_seed() {
  local sock; sock=$(socket) || { fail "no default route"; return 1; }
  port_open || { fail "bridge is down — run 'tools/emulator.sh up' first"; return 1; }
  # cwd-independent like every other command here: the APK is located from the script,
  # not from wherever the caller happens to be standing.
  local root; root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd) \
    || { fail "cannot resolve the repo root from ${BASH_SOURCE[0]}"; return 1; }
  local apk="$root/build/app/outputs/flutter-apk/app-debug.apk"
  [ -f "$apk" ] || { fail "no debug APK at $apk — run 'flutter build apk --debug' first"; return 1; }
  ADB_SERVER_SOCKET=$sock adb install -r "$apk" >&2
  cmd_reset
  # `am start` exits 0 even when it prints `Error: Activity not started`, so the output is
  # what is checked, not the status. -W waits for the launch and reports it as `Status:`.
  local out; out=$(ADB_SERVER_SOCKET=$sock adb shell am start -W -n "$APP_ID/.MainActivity" 2>&1) || true
  printf '%s\n' "$out" >&2
  if printf '%s' "$out" | grep -qi '^Error'; then
    fail "launch failed: $APP_ID/.MainActivity did not start"
    return 1
  fi
  # A launch that resolves but never comes up prints `Status: timeout` and no Error line.
  if ! printf '%s' "$out" | grep -qi '^Status: ok'; then
    fail "launch did not report 'Status: ok' — the activity may not be up"
    return 1
  fi
  pass "launched $APP_ID/.MainActivity"
}

case "${1:-}" in
  up)     shift; cmd_up "$@" ;;
  adb)    shift; cmd_adb "$@" ;;
  status) shift; cmd_status "$@" ;;
  down)   shift; cmd_down "$@" ;;
  # Neither takes an argument. Rejecting extras rather than passing them to a function that
  # ignores them: `emulator.sh reset --wipe` used to silently do the plain default instead.
  reset)  shift; [ $# -eq 0 ] || { say "reset takes no arguments (got: $*)"; exit 2; }; cmd_reset ;;
  seed)   shift; [ $# -eq 0 ] || { say "seed takes no arguments (got: $*)"; exit 2; }; cmd_seed ;;
  -h|--help|help) say "usage: tools/emulator.sh {up|adb <args>|status|down|reset|seed}"; exit 0 ;;
  *) say "usage: tools/emulator.sh {up|adb <args>|status|down|reset|seed}"; exit 2 ;;
esac
