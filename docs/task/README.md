# Meal Mate Task System

These cards are planning inputs, not approved execution plans. A workflow recommendation is not an invocation or authorization. Read the PRD, `docs/ROADMAP.md`, the invariants, and the selected card; `docs/PRD_v3.md` is authoritative where it conflicts with `docs/PRD_v2.md` (D-028); do not load the whole task directory by default. `docs/ROADMAP.md` is the only live source for task status, the current implementation task, and the next implementation task.

## Lifecycle

`Draft → Ready → In Progress → Verify → Done`

- Never copy live status into this README or a task card; read the roadmap register.
- Promote a card to Ready only when every direct dependency is Done and it is the selected next implementation task. Ready means eligible to plan, not approved to execute.
- Return it to Draft per the Status contract in `docs/ROADMAP.md`, which carries the single Draft-trigger rule (D-031).
- Record evidence, status, and the next implementation task together in `docs/ROADMAP.md` at handoff.
- Approvals are nontransitive: approving a decision, card, or plan approves only that artifact.
- Recheck plan freshness immediately before execution.

## Mandatory planning, execution, and handoff gates

Before `plan-task` plans an implementation, readiness, or optional card:

1. Confirm the card is the roadmap's **Next implementation task** and its register status is `Ready`.
2. Confirm every ID in its `Depends on` cell is `Done` in the roadmap register and its evidence row satisfies the current PASS/D-027 contract. The six cards completed before D-035 are covered by the one-time migration audit in the roadmap. For later cards, direct dependencies are sufficient because no new card reaches Done without the same recursive evidence and owner-approval check.
3. Confirm no other implementation card is `In Progress` or `Verify`. An explicitly owner-paced decision such as `DEC-004` may proceed concurrently and does not occupy the implementation lane.
4. Review the card against its current sources and decisions. Planning may proceed when card prose needs refresh, but the plan must schedule that refresh before implementation rather than creating a separate process-only task.

An optional or post-launch card must also be explicitly selected as the roadmap's Next implementation task and must not displace required MVP work without an owner decision recorded in the roadmap.

Before a decision workflow resolves a decision card, confirm every declared dependency is Done with an evidence row and confirm the roadmap identifies the decision as either the current required decision or explicitly owner-paced work. A decision does not need to be the Next implementation task and does not occupy the implementation lane. Its result and Done transition still require explicit owner approval.

If any check fails, report the exact conflicting row or missing evidence and stop without changing implementation state. Do not infer a different next task.

Every produced implementation plan must make the execution gate its first execution step. Immediately before code or other scoped implementation changes, `execute-plan` re-reads the roadmap, selected card, and approved plan; repeats checks 1–3; confirms that the plan was produced or revalidated against the current sources and explicitly refreshes any stale card prose before implementation; then moves the card `Ready → In Progress` in the roadmap. A missing, inaccessible, or stale plan stops execution and routes back to `plan-task`; it does not make the card ineligible to plan. At handoff, update the card's status, evidence row, delivery-gate progress, and **Next implementation task** together. Use `Verify` while required evidence or explicit owner status approval remains unresolved. Use `Done` only after the evidence contract and explicit owner approval are both satisfied. Owner-delegated approval (recorded 2026-08-28): under a `# orchestrate: auto-approve` directive in `docs/task/SEQUENCE.txt`, the orchestrator's review, fix loop, independent verifier `PASS`, and merge to `integration` together constitute the owner's approval for a card whose `External actions` row is `None` or emulator-only — the orchestrator then writes the `Done` evidence row and promotes the next card itself, and the `integration → master` merge is the owner's ratification. Any other `External actions` wording keeps the card at `Verify` for explicit owner approval. The **Next implementation task** is the unique `Ready` row in the register; the Current-milestone lines restate it and are rewritten by whoever changes the register.

## Recommended routing

Use decision workflows for consequential open choices, planning for implementation cards, and execution only for an already approved plan.

Codex examples:

```text
$personal-workflows:deep-options Read docs/task/decision/DEC-001_PRODUCT_PLATFORM_IDENTITY_DECISION.md and resolve its decisions.
$personal-workflows:grill-me Read <card-or-draft-plan> and challenge unresolved owner tradeoffs.
$personal-workflows:plan-task Read docs/task/mvp/MVP-001_CLEAN_ROOM_SCAFFOLD.md and plan its implementation.
$personal-workflows:redteam-plan Read <proposed-plan-path> and identify correctness, safety, and verification gaps before approval.
$personal-workflows:execute-plan Read <approved-plan-path>, docs/ROADMAP.md, and the selected card; repeat the mandatory execution gate, then execute it.
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

## Owner-build delivery

Owner directive recorded 2026-09-07: a new owner-installable Android build is not handed off as
a local filesystem path alone. Its corresponding source must be intentionally committed and pushed,
a new `v*` tag must run `.github/workflows/release-apk.yml` successfully, and the handoff must include
the direct GitHub Release page or APK link. Verify the release asset exists and record its size and
SHA-256 digest before calling delivery complete.

A local APK may still be built for verification before the source is ready to publish. Do not attach
an APK made from an uncommitted or mismatched tree to an older tag, and do not sweep unrelated or
unreviewed working-tree changes into a release commit merely to satisfy this rule. If the source is
not yet safe to commit and tag, report the GitHub build as pending rather than presenting the local
APK as the owner handoff.

A small read-only metadata/dependency validator may be added later if repeated maintenance friction justifies it. Do not add orchestration machinery preemptively.
