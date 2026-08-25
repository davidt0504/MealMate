# PRE-003 — iOS toolchain readiness

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Readiness |
| Workstream | Developer environment (post-launch) |
| Depends on | PRE-002, DEC-005 |
| Complexity | TBD |
| Assurance | TBD |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None (derive after DEC-005) |

## Outcome and user value

Prove iOS build and signing of the Rust-bridged app on a macOS/CI environment. Post-launch by D-015; this card is the discharge point for the iOS `NOT VERIFIED` item PRE-002 AC-8 carries (DEC-005 records the Android-only scope of its commitment), and it re-opens the PRD v3 §22 kill criterion for iOS packaging.

## Authoritative sources

- `docs/PRD_v3.md` §6.3 Phase-0 requirement, §17 Phase 1, §22
- `docs/ROADMAP.md` D-015, D-027, D-028
- `docs/task/MVP_INVARIANTS.md` 14, 15

Body to be derived after DEC-005 Done (D-029).
