# MVP-024 AC-8 — Attention measurement protocol and record

Carries PRD §22's required MVP proof: a dated attention measurement over at least four
household scenarios, both arms per scenario, with the **median of the four per-scenario
active-seconds ratios** as the stated statistic (AC-8's wording is operative; the
`KNOWN_ISSUES-low.md` ambiguity entry resolves as *median of per-scenario ratios*, not the
ratio of medians). An adverse result — no material attention reduction — is a hard stop per
the card: it returns `MVP-023` to Draft under D-031 with this record as the trigger, not a
remediation loop here.

## Why this record can start unmeasured

The measurement requires a human operator: active seconds are wall-clock human attention on
an emulator or device, timed with a stopwatch, and an orchestrated session cannot
self-measure them honestly. Until an operator fills the record, the AC-8 evidence row is
`NOT VERIFIED` under the D-027 route (resolving action is outside the card's own work;
discharge point: owner measurement before `MVP-022`'s §22 proof), and the card holds at
`Verify`.

## Protocol

Four scenarios, each constructible through the app UI available at this card's point in
`docs/task/SEQUENCE.txt` (`MVP-006` preferences, `MVP-008` recipes, `MVP-012`/`MVP-013`
slots):

1. **Fresh start** — starter household: onboarding done, starter catalog only, no
   preferences beyond onboarding, empty week.
2. **Many-locked week** — ≥5 of 7 dinner slots planned and locked by hand before the run.
3. **Sparse-pantry typical week** — a normal recipe library (~10 recipes), no pantry marks,
   2–3 slots pre-filled unlocked.
4. **Restrictions + preference-rich** — ≥2 restrictions (one Known, one free-text) and ≥3
   preferences per member.

Two arms per scenario, to the same end state (an accepted, ready plan with the shopping
list reachable):

- **Arm A (Cover)**: enter Cover My Week, resolve every surfaced question, Accept.
- **Arm B (Manual)**: fill the same week through the `MVP-013` grid by hand, then open
  Shopping. Observed, not gated — no target applies to this arm.

Arm order counterbalanced across scenarios: **A/B, B/A, A/B, B/A** (scenarios 1–4 in that
order). Reset the household state between arms of the same scenario (fresh DB seeded to the
scenario description) so the second arm never benefits from the first arm's writes.

Per arm, record one row:

- operator, date
- **active seconds** (stopwatch; pause during app launch/build waits — attention, not
  wall-clock elapsed)
- **explicit decisions** (count of taps that choose content: accept, swap choice, veto,
  reviewed marker, per-slot picks in the manual arm; navigation taps excluded)
- **filled enabled slots**, counted from the `MVP-013` grid after the run
- **restriction warnings**, counted from `MVP-009`'s surface after the run

Statistic: per scenario, ratio = (Arm A active seconds) / (Arm B active seconds); the
record states the **median of the four ratios**.

## Record

| Scenario | Arm | Order | Operator | Date | Active seconds | Explicit decisions | Filled slots | Restriction warnings |
|---|---|---|---|---|---|---|---|---|
| 1 fresh-start | A Cover | 1st | — | — | — | — | — | — |
| 1 fresh-start | B Manual | 2nd | — | — | — | — | — | — |
| 2 many-locked | A Cover | 2nd | — | — | — | — | — | — |
| 2 many-locked | B Manual | 1st | — | — | — | — | — | — |
| 3 sparse-pantry | A Cover | 1st | — | — | — | — | — | — |
| 3 sparse-pantry | B Manual | 2nd | — | — | — | — | — | — |
| 4 restriction-rich | A Cover | 2nd | — | — | — | — | — | — |
| 4 restriction-rich | B Manual | 1st | — | — | — | — | — | — |

**Per-scenario ratios:** — , — , — , —
**Median of per-scenario ratios:** —

**Status: NOT VERIFIED (2026-09-02)** — no operator measurement taken in-session; see the
D-027 route above.
