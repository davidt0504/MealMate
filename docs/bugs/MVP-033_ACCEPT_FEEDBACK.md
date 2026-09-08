# BUG — Accept does not acknowledge an accepted covered week

| Field | Value |
|---|---|
| Status | Open |
| Severity | Low — misleading completion feedback; no data loss or blocked route |
| Found in | MVP-033 release `v1.0.1+2` |
| Device | Samsung Galaxy S20 FE, Android 13 |
| Owner observation | 2026-09-07 |
| Affected surface | `Cover My Week` after **Accept** |

## Reproduction

1. Launch a fresh household into **Cover My Week**.
2. Tap **Accept**.

## Actual behavior

The plan is accepted and the shopping-list route is available, but the **Accept** button remains visible and appears actionable. There is no clear in-place acknowledgement that the action succeeded, so the person cannot tell whether their tap worked without discovering the next route.

## Expected behavior

After a successful acceptance, the screen should make the settled state unmistakable: replace or disable the action with calm confirmation and a clear next action toward the shopping list. It must not add celebration, urgency, or a modal interruption.

## Scope and route

This is a presentation-feedback correction, not a planner, shopping, or onboarding-state defect. The current `CoverScreen` already holds `_accepted` and renders `Shopping list is ready`; the defect is that it leaves the original primary **Accept** control in place. A bounded follow-up may use that state to replace the control after success, with regression coverage for failure and repeated taps.

Do not use this bug to reopen MVP-033's first-run routing decision. It may be addressed by the next selected UX-maintenance work or a dedicated card; any implementation must preserve MVP-024's one-tap acceptance and MVP-033's no-trap completion behavior.
