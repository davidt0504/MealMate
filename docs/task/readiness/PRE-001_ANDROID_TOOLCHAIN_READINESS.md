# PRE-001 — Android toolchain readiness

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | Ready |
| Type | Readiness |
| Workstream | Developer environment |
| Depends on | — |
| Complexity | Focused |
| Assurance | Standard |
| Sequential batching | Complete before MVP-001 |
| Recommended workflow | `plan-task` |
| External actions | SDK downloads may require network/disk; no credentials, signing, or cloud resources |

## Outcome and user value

Prove this Linux/WSL2 environment can create, analyze, test, build, and run a current Android Flutter app before the legacy tree is replaced.

## Authoritative sources

- `docs/PRD_v2.md` §§14.1, 19.1
- `docs/ROADMAP.md` D-004, D-015

## Load-bearing constraints

- Do not modify or delete the current app during readiness checks.
- Use a temporary scaffold outside the repository for proof.
- Pin/document compatible Flutter, Dart, Java, Gradle, Android SDK, and emulator/device versions.
- iOS tooling is out of scope.

## Scope

- Inventory installed prerequisites and storage/virtualization constraints.
- Install or identify missing supported tooling with reproducible commands.
- Create a temporary Flutter app; run format/analyze/test, Android debug build, and emulator launch when virtualization permits.
- Document any host-only step and the exact project command contract for MVP-001.

## Non-goals

- Meal Mate scaffold changes, Firebase, CI, production credentials, release signing, or physical-device certification.

## Decision gates

- If emulator acceleration is unavailable, decide whether a slower emulator or later physical Android device evidence is an acceptable temporary boundary; do not claim runtime PASS without it.

## Acceptance criteria

- **AC-1:** Compatible tool versions and installation locations are documented.
- **AC-2:** A temporary current Flutter template passes formatting, analysis, and tests.
- **AC-3:** An Android debug artifact builds successfully.
- **AC-4:** The app launches in an Android emulator, or the blocker is recorded as NOT VERIFIED with a bounded remediation plan.
- **AC-5:** No Meal Mate application file changed.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Version/doctor output and written compatibility record |
| AC-2 | Command output from the temporary project |
| AC-3 | Build command output and artifact path |
| AC-4 | Emulator launch/screenshot/log, or explicit NOT VERIFIED blocker |
| AC-5 | Before/after Git diff excluding approved docs and line-ending noise |

## Stop/failure conditions

- Stop before destructive package cleanup, system-wide version replacement, credentials, or unauthorized external resources.
- Two failed toolchain remediation cycles return the card to planning.

## Handoff

Record evidence and exact Flutter/Dart commands in `docs/ROADMAP.md`. Delete only the known temporary scaffold. Promote MVP-001 only when PRE-001 and DEC-001 are Done.
