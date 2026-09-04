# DEC-006 — Monetization model, packaging and price

> Planning input, not an approved execution plan. Workflow recommendations are not invocations.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Decision |
| Workstream | Monetization |
| Depends on | DEC-003 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No implementation until Done |
| Recommended workflow | `deep-options`; then `grill-me` for owner pricing and packaging choices |
| External actions | Research is read-only; opening a merchant account, publishing a price, or committing to a billing vendor requires separate owner authorization |

## Workflow gate

Before resolving `DEC-006`, read `docs/ROADMAP.md`, confirm every declared dependency is Done with evidence, and confirm the roadmap identifies this card as the current required decision or explicit owner-paced work. This decision does not need to be the Next implementation task and does not occupy the implementation lane. Its result and Done transition require explicit owner approval.

## Outcome and user value

Decide what the household pays for, at what boundary, and on what terms — so that `MVP-027` can model entitlement and `MVP-028` can gate a surface without either card inventing product policy.

## Authoritative sources

- `docs/PRD_v3.md` §20 (monetization philosophy; "No ads"; the Remember/Prepare/Act ladder is labelled **potential future**), §16 "Not MVP", and `:1004` ("core deterministic intelligence should be useful without artificial scarcity")
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §21 (permitted and forbidden monetization axes)
- `docs/ROADMAP.md` D-037, D-040, and the post-launch candidate list at `:337`
- `docs/task/MVP_INVARIANTS.md` 15, 16
- Historical, and itself decision 8 below: `docs/PRD_v2.md` §16 — lines 390-395 (household entitlement; billing ownership stored separately from effective entitlement), `:788-793` (fairness constraints on packaging experiments), `:847` ("Do not lock a final subscription price into this PRD"), `:862` (`foundingUser`; `complimentary` grants)

## Locked constraints

- **The deterministic planner may not be metered.** `HOUSEHOLD_CONTROL_PRINCIPLES.md` §21 excludes "artificially metered deterministic intelligence" from the permitted axes, and `PRD_v3.md:1004` requires core deterministic intelligence to be useful "without artificial scarcity". Cover My Week, the planner, restriction warnings and the shopping-list derivation are therefore outside any paid boundary this card may draw. `MVP-028` carries the same constraint as an explicit non-gate list.
- Permitted axes are only those §21 names: responsibility or capability transferred to software; household reliability and sync; variable-cost external or generative services. **Not** notifications, feed content, or metering of deterministic work.
- No ads, in any tier, ever (`PRD_v3.md` §20).
- Price must never be personalised on inferred wealth, vulnerability or willingness to pay (`PRD_v2.md:788-793`). Packaging and presentation may be cohort-experimented; the price itself may not be individually derived.
- Whatever is decided here becomes a `docs/ROADMAP.md` D-row, not a PRD edit — `PRD_v2.md:847` forbids locking a price into the PRD.
- The paywall ships **live at launch** (owner decision, D-040). This card's result therefore gates `MVP-027`, `MVP-028` and `MVP-029`, all of which are on the launch path.

## Decisions to resolve

1. **The free/paid boundary.** Which capabilities are free permanently, given the §21 constraint removes the planner from consideration. State the boundary as a rule, not a feature list, so later features inherit it.
2. **Entitlement scope.** `PRD_v2.md:390-395` puts entitlement at the household and stores billing ownership separately from effective entitlement. Confirm or replace that model, and say what "household-level" means when members are on different devices.
3. **Packaging.** One paid tier or several; whether the Remember/Prepare/Act ladder from `PRD_v3.md` §20 becomes the packaging or stays a roadmap.
4. **The price**, currency, billing period, and whether a trial or introductory offer exists. Records as a D-row.
5. **`foundingUser`.** `PRD_v2.md:862` says the benefit "must be explicitly defined before monetization launches" and that the tag does **not** automatically mean lifetime access. Define it, and define who qualifies — anyone with a durable account before the paywall ships is already accruing an expectation.
6. **Metered LLM conveniences.** `PRD_v3.md` §20 allows variable-cost generative work to be premium. Decide whether it is metered, bundled, or absent at launch. `OPT-001:73` stops if it "introduces recurring LLM cost", so a metered path must not be smuggled in there.
7. **Member departure.** What happens to household entitlement when the purchasing member leaves the household, and separately when they delete their account (`MVP-026`). `PRD_v2.md:813` says the benefit is household-level and purchased once; it decides nothing about departure. `MVP-027` implements the answer and `MVP-026` must honour it.
8. **Is `PRD_v2.md` §16 live background or superseded?** `PRD_v3.md:6` supersedes v2 "where this document changes product or architecture direction". v3 §20 compresses §16 rather than contradicting it, so §16 is arguably still live — but no recorded decision says so, and every mechanic in this card currently rests on that inference. Settle it explicitly.

## Options and tradeoffs

- **Boundary at reliability/sync** (§21's second axis) — sells household sync and durable backup. Fits the local-first architecture, since the free tier stays fully functional offline and paid buys multi-device coherence. Risk: `MVP-018`'s cloud adapter is explicitly optional and its failure "degrades only the optional feature", so the paid surface is the one most likely to be degraded.
- **Boundary at delegated responsibility** (§21's first axis) — sells proactive plan refresh and schedule-derived constraints, the `Prepare` rung. Closest to the product thesis and to §21's "high-value successful customer who barely opens the product". Risk: none of it is built, so it cannot ship at launch.
- **Boundary at variable-cost services** (§21's third axis) — sells only what costs money to run, which is the easiest to justify and the easiest to meter honestly. Risk: at launch there is no LLM path at all, so this tier would be empty.
- **Single tier versus ladder** — a single paid tier is far cheaper to implement, price, and explain, and `MVP-027`'s state model stays binary. A ladder needs upgrade/downgrade/proration handling in `MVP-029` from day one.
- **Trial versus no trial** — a trial requires the grace/hold state machine to be exercised before any revenue arrives, adding risk to a launch-blocking card.

## Completion criteria

- Decisions 1–8 each resolved with a rationale, or explicitly deferred with the deferral recorded and its consequence for `MVP-027`/`MVP-028`/`MVP-029` named.
- The free/paid boundary is stated as a rule and checked against `HOUSEHOLD_CONTROL_PRINCIPLES.md` §21 item by item, showing that nothing gated is deterministic planning.
- Price, currency, billing period and any trial recorded as a `docs/ROADMAP.md` D-row, not in a PRD.
- `foundingUser` defined, including who qualifies and what the benefit survives.
- The member-departure rule stated precisely enough for `MVP-027` to implement and `MVP-026` to honour without further interpretation.
- Decision 8 answered, so later cards know whether `PRD_v2.md` §16 may be cited as authority.

## Stop/failure conditions

- Stop if a proposed boundary requires metering deterministic planning — that is a §21 violation and no packaging argument overrides it.
- Stop if resolving this needs a merchant account, a published price, or a signed vendor agreement; those are owner actions outside this card.
- Stop if the answer would require `MVP-018`'s cloud adapter to become mandatory — `MVP-018:35` makes it optional by construction and that is an architecture change, not a pricing choice.
- After two failed remediation cycles, return to planning rather than weakening the constraints.

## Resolution

_Unresolved._
