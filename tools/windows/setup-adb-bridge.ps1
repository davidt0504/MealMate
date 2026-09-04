<#
.SYNOPSIS
    Prepares the Windows host for the WSL -> Android emulator adb bridge (ROADMAP D-022).

.DESCRIPTION
    Implements the three host prerequisites recorded in docs/ROADMAP.md under
    "Prerequisites discovered during PRE-001, required for MVP-003/004/005":

      1. Remove the inbound Block rules named 'adb.exe'. Windows Firewall applies
         Block over Allow, so while they exist no port-based Allow rule can work
         and the bridge times out silently.
      2. Add a scoped inbound Allow rule for TCP 5037 limited to the WSL subnet.
      3. Confirm Windows platform-tools exists under the Android SDK root. The
         emulator validates the SDK root and aborts with "Broken AVD system path"
         without it, even though the bridge uses a different adb.exe.

    Self-elevates. Idempotent - safe to re-run. Refuses to delete an 'adb.exe'
    rule whose action is Allow unless -Force is given.

.PARAMETER WslSubnet
    CIDR allowed to reach TCP 5037. Default matches the subnet recorded in
    ROADMAP D-022. If WSL is reassigned a different subnet (Windows update, or a
    switch to mirrored networking), run `ip route` in WSL and pass the new value.

.PARAMETER SdkRoot
    Windows Android SDK root that must contain platform-tools\adb.exe.

.PARAMETER VerifyOnly
    Report current state and change nothing. Use this to check the host before
    MVP-003/004/005, or to diagnose a bridge that stopped working.

.PARAMETER Force
    Permit removal of an 'adb.exe' rule whose action is Allow. Not needed for the
    documented Block-rule case.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File C:\Users\David\meal_mate_tools\setup-adb-bridge.ps1 -VerifyOnly

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File C:\Users\David\meal_mate_tools\setup-adb-bridge.ps1
#>

[CmdletBinding()]
param(
    [string] $WslSubnet = '172.21.80.0/20',
    [string] $SdkRoot   = 'C:\Android\sdk',
    [switch] $VerifyOnly,
    [switch] $Force
)

$ErrorActionPreference = 'Stop'
$AllowRuleName = 'WSL adb server'
$BlockRuleName = 'adb.exe'
$AdbPort       = 5037

function Write-Section { param([string] $Text) Write-Host "`n=== $Text ===" -ForegroundColor Cyan }
function Write-Pass    { param([string] $Text) Write-Host "  PASS  $Text" -ForegroundColor Green }
function Write-Fail    { param([string] $Text) Write-Host "  FAIL  $Text" -ForegroundColor Red }
function Write-Info    { param([string] $Text) Write-Host "  ..    $Text" -ForegroundColor Gray }

# --- Self-elevate -----------------------------------------------------------
$identity  = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
$isAdmin   = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host 'Not elevated - relaunching as Administrator...' -ForegroundColor Yellow
    $argList = @(
        '-NoProfile'
        '-ExecutionPolicy', 'Bypass'
        '-File', "`"$PSCommandPath`""
        '-WslSubnet', $WslSubnet
        '-SdkRoot', "`"$SdkRoot`""
    )
    if ($VerifyOnly) { $argList += '-VerifyOnly' }
    if ($Force)      { $argList += '-Force' }

    try {
        Start-Process -FilePath 'powershell.exe' -Verb RunAs -ArgumentList $argList
    } catch {
        Write-Fail 'Elevation was declined or failed. Re-run from an elevated PowerShell.'
        exit 1
    }
    exit 0
}

Write-Host 'Kimatta - WSL adb bridge host setup' -ForegroundColor White
Write-Host "Mode: $(if ($VerifyOnly) { 'VERIFY ONLY (no changes)' } else { 'APPLY' })"
Write-Host "WSL subnet: $WslSubnet"
Write-Host "SDK root:   $SdkRoot"

$failures = New-Object System.Collections.Generic.List[string]

# --- 1. Inspect and remove the Block rules ----------------------------------
Write-Section "1. Inbound Block rules named '$BlockRuleName'"

$blockRules = @(Get-NetFirewallRule -DisplayName $BlockRuleName -ErrorAction SilentlyContinue)

if ($blockRules.Count -eq 0) {
    Write-Pass "No '$BlockRuleName' rules present - nothing blocking."
} else {
    $blockRules |
        Select-Object DisplayName, Direction, Action, Enabled, Profile |
        Format-Table -AutoSize | Out-String | Write-Host

    $allowRules = @($blockRules | Where-Object { $_.Action -eq 'Allow' })
    if ($allowRules.Count -gt 0 -and -not $Force) {
        Write-Fail "$($allowRules.Count) rule(s) named '$BlockRuleName' have Action=Allow."
        Write-Info 'Removing them could revoke access you rely on. Review the table above.'
        Write-Info 'Re-run with -Force only if you are sure they should go.'
        $failures.Add('adb.exe rules not removed (Allow rule present, -Force not given)')
    } elseif ($VerifyOnly) {
        Write-Info "$($blockRules.Count) rule(s) would be removed (VerifyOnly - no change made)."
        $failures.Add('adb.exe Block rules still present')
    } else {
        Remove-NetFirewallRule -DisplayName $BlockRuleName
        Write-Pass "Removed $($blockRules.Count) rule(s) named '$BlockRuleName'."
    }
}

# --- 2. Scoped Allow rule for TCP 5037 --------------------------------------
Write-Section "2. Inbound Allow rule '$AllowRuleName' (TCP $AdbPort)"

$existingAllow = @(Get-NetFirewallRule -DisplayName $AllowRuleName -ErrorAction SilentlyContinue)

if ($VerifyOnly) {
    if ($existingAllow.Count -eq 0) {
        Write-Fail 'Allow rule is missing.'
        $failures.Add('WSL adb server allow rule missing')
    } else {
        $scope = ($existingAllow | Get-NetFirewallAddressFilter).RemoteAddress -join ', '
        Write-Pass "Allow rule present. Remote scope: $scope"
        if ($scope -notmatch [regex]::Escape($WslSubnet)) {
            Write-Fail "Scope does not include $WslSubnet - the bridge will time out."
            $failures.Add("allow rule scope mismatch (has: $scope)")
        }
    }
} else {
    if ($existingAllow.Count -gt 0) {
        Remove-NetFirewallRule -DisplayName $AllowRuleName
        Write-Info 'Removed prior copy so the scope is rewritten cleanly.'
    }
    New-NetFirewallRule -DisplayName $AllowRuleName `
        -Direction Inbound -Protocol TCP -LocalPort $AdbPort `
        -Action Allow -Profile Any -RemoteAddress $WslSubnet | Out-Null
    Write-Pass "Created '$AllowRuleName' allowing TCP $AdbPort from $WslSubnet only."
}

# --- 3. Windows platform-tools ----------------------------------------------
Write-Section '3. Windows platform-tools under the SDK root'

$adbPath = Join-Path $SdkRoot 'platform-tools\adb.exe'
if (Test-Path $adbPath) {
    Write-Pass "Found $adbPath"
} else {
    Write-Fail "Missing $adbPath"
    Write-Info 'The emulator validates the SDK root and aborts with "Broken AVD system path".'
    Write-Info "Install platform-tools into $SdkRoot via Android Studio SDK Manager or sdkmanager."
    $failures.Add('Windows platform-tools missing')
}

# --- Final verification -----------------------------------------------------
Write-Section 'Final state'

$final = @(Get-NetFirewallRule -DisplayName $BlockRuleName, $AllowRuleName -ErrorAction SilentlyContinue)
if ($final.Count -gt 0) {
    $final |
        Select-Object DisplayName, Direction, Action, Enabled |
        Format-Table -AutoSize | Out-String | Write-Host
} else {
    Write-Host '  (no matching firewall rules)'
}

Write-Host ''
if ($failures.Count -eq 0) {
    Write-Host 'RESULT: PASS - host is ready for the WSL adb bridge.' -ForegroundColor Green
    Write-Host 'Next: start the Windows-side adb server, then from WSL run' -ForegroundColor Gray
    Write-Host '  export ADB_SERVER_SOCKET=tcp:$(ip route | awk ''/default/ {print $3}'' | head -1):5037' -ForegroundColor Gray
    Write-Host '  adb devices' -ForegroundColor Gray
    $exitCode = 0
} else {
    Write-Host "RESULT: $($failures.Count) item(s) outstanding" -ForegroundColor Red
    foreach ($f in $failures) { Write-Host "  - $f" -ForegroundColor Red }
    $exitCode = 1
}

Write-Host ''
Write-Host 'Copy the output above back to the session if you want it verified.' -ForegroundColor Gray
Write-Host 'Press Enter to close...' -ForegroundColor Gray
try { Read-Host | Out-Null } catch { }
exit $exitCode
