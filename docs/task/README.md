# Meal Mate Task System

These cards are planning inputs, not approved execution plans. A workflow recommendation is not an invocation or authorization. Read the PRD, roadmap, invariants, and the selected card; `docs/PRD_v3.md` is authoritative where it conflicts with `docs/PRD_v2.md` (D-028); do not load the whole task directory by default.

## Lifecycle

`Draft → Ready → In Progress → Verify → Done`

- `DEC-001`, `PRE-001`, and `MVP-001` are Done; `DEC-002` is Done (2026-08-24) and `DEC-004` is `In Progress`. `PRE-002` is `Ready`; `MVP-002` returned to `Draft` under D-028 (2026-08-24). `DEC-004` (naming clearance) is owner-paced: it directly gates the cards that bind a production identifier (`MVP-018`, `MVP-021`, `MVP-022`) and transitively gates `MVP-019` and `MVP-020`.
- Promote a card only when every dependency is Done and its inputs are current.
- Return it to Draft after a material product, architecture, or dependency change.
- Record evidence and status in `docs/ROADMAP.md` at handoff.
- Approvals are nontransitive: approving a decision, card, or plan approves only that artifact.
- Recheck plan freshness immediately before execution.

## Recommended routing

Use decision workflows for consequential open choices, planning for implementation cards, and execution only for an already approved plan.

Codex examples:

```text
$personal-workflows:deep-options Read docs/task/decision/DEC-001_PRODUCT_PLATFORM_IDENTITY_DECISION.md and resolve its decisions.
$personal-workflows:grill-me Read <card-or-draft-plan> and challenge unresolved owner tradeoffs.
$personal-workflows:plan-task Read docs/task/mvp/MVP-001_CLEAN_ROOM_SCAFFOLD.md and plan its implementation.
$personal-workflows:redteam-plan Read <proposed-plan-path> and identify correctness, safety, and verification gaps before approval.
$personal-workflows:execute-plan Read <approved-plan-path> and execute it.
```

Claude equivalents use `/deep-options`, `/grill-me`, `/plan-task`, `/redteam-plan`, and `/execute-plan`. The installed execution workflow is `execute-plan`. For Elevated cards, red-team the proposed plan before approval. Never automatically chain commands. Do not use `--auto` for the first three implementation cards or for security, data-loss, destructive, credential, production, DNS, signing, or store-release work.

## Complexity and assurance

Complexity selects the implementation capability: `focused`, `complex`, or `mechanical`. Assurance selects evidence rigor: `standard` or `elevated`. They are intentionally independent.

As of 2026-08-20, suggested routing is:

| Work | Codex | Claude |
|---|---|---|
| Focused | Terra | Sonnet |
| Complex | Sol | Opus |
| Mechanical, low risk | Luna | Haiku |

Use Fable only if locally available and proven on comparable repository work. Model labels are advisory and may age; cards remain model-neutral.

## Risk-tiered verification

- Standard: implementer runs targeted tests, static analysis, and the card's required manual evidence.
- Elevated: tests should be derived from acceptance criteria before or independently from implementation when practical; a fresh-context verifier reruns them, investigates failures, and checks material coverage gaps.
- The verifier begins read-only. Test changes require explicit justification and review; never weaken a test merely to make it pass.
- Report each required item as `PASS`, `FAIL`, or `NOT VERIFIED`. Only all required `PASS` permits Done, with one exception: an item may stand at `NOT VERIFIED` and still permit Done when all three hold — (1) the card records why the resolving action is not the card's own work to do, naming the constraint or stop condition that puts it outside scope; (2) the card names where the item is discharged downstream, or states that it is permanently unresolvable and names the residual risk; and (3) the owner accepts that residual risk, recorded in the `docs/ROADMAP.md` Evidence-log row as a dated, attributed clause — `owner-accepted YYYY-MM-DD`, mirroring the form of the `owner-approved 2026-08-22` clause in `MVP-001`'s row. An item whose resolving action is simply unfinished work inside the card's own scope is not covered and still blocks Done, as does any `NOT VERIFIED` missing one of the three. This exception applies to cards reaching Done on or after 2026-08-24; cards already Done are unaffected, whether or not they would have met it.
- Allow at most two implement/fix/reverify cycles before escalating the underlying design or task boundary.

Subagents may be used when the active environment supports them, but independence comes from separate context and evidence—not from agent count. Never run concurrent writers in the same checkout. Sequential batching is allowed only after the current card is Done.

## Authority boundary

No task implicitly authorizes production/paid resources, destructive replacement, credentials, DNS, signing, store submissions, commit, or push. Cards must call out external actions. Prefer emulators and non-production resources until an owner separately authorizes promotion.

A small read-only metadata/dependency validator may be added later if repeated maintenance friction justifies it. Do not add orchestration machinery preemptively.
