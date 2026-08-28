# Orchestration System — Build Brief
Purpose: build re-executable machinery — a GLOBALLY INSTALLED slash command entry
point plus the scripts and automations it invokes — that executes a set of task docs
listed in a TASK MANIFEST (a file passed as a required path argument) via subagents
with adversarial review, in ANY git repository, not only this one. This repo and
docs/SEQUENCE.txt are the reference instance for building and testing, not the target
of the design. Three tiers below — treat them differently.

## Tier 1 — Non-negotiable requirements (outcomes; do not revisit)
- GLOBAL INSTALLATION: the slash command, scripts, prompt templates, and RUNBOOK
  install at user level (under ~/.claude/), never into any project. Nothing
  repo-specific is hardcoded anywhere in the system.
- PER-REPO CONFIGURATION: anything that varies by repository (build/test commands,
  integration branch, dependency install steps) comes from explicit per-repo
  configuration, never from assumptions carried over from the reference repo.
- RUNTIME PRECONDITIONS CHECK: before doing anything in a target repo, the command
  verifies the repo satisfies all requirements (git repo, clean state, required
  commands resolvable, per-repo config present and valid) and fails fast with
  specific guidance rather than starting a partial run.
- No commit from this system ever lands on a branch human developers use. All work
  happens in isolated checkouts, merged to the repo's configured integration branch
  only after a task fully passes.
- A task is COMPLETE only when: implementation done, build and tests green (per the
  repo's configured commands), review clean (zero high/critical findings), and
  independent verification passed.
- Never --dangerously-skip-permissions. Unattended sessions run under least-privilege
  tool allowlists appropriate to their phase.
- Never edit my existing commands/skills/hooks/automations in place. Wrap or create
  variants. Task docs are spec: propose improvements, apply only with my approval.
- Before wiring any command into the system, verify which definition resolves for its
  name (project vs. user level). /handoff in particular must resolve to MY custom
  version, not any similarly-named installed skill.
- Orchestration sessions must load my commands and the enforcement hooks (beware
  headless flags that skip ambient config discovery).
- Hard stop: any task failure, non-clean review exit, or budget breach halts ALL work
  and notifies me. Cost is tracked per task and bounded by a configurable ceiling.
- Every question surfaced to me goes through /deep-options first, and reaches my phone.
- Low/medium findings not fixed are logged as deferred with justification — never
  silently dropped.
- HOOK SCOPING IS LOAD-BEARING: orchestration hooks are necessarily user-level and
  therefore present in EVERY repo and session. They must be provably inert in all
  interactive sessions and all repos outside an active orchestration run.
- The system is manifest-agnostic: the manifest path is a required argument.
  Orchestration state is keyed per repo AND per manifest so runs in different repos
  or against different manifests never collide, and resume targets the right state.
- The system is resumable after interruption without redoing completed work, and the
  slash command is a THIN entry point — logic lives in scripts and state, not prose.
- Phase 4 includes an interactive /review-skill pass, with me present, over every new
  prompt-bearing artifact (the slash command, per-phase prompt templates, any command
  variants/wrappers). /review-skill must never be wired into the unattended loop.
- Deliverables include a dry-run mode; smoke tests demonstrating each mechanism fired
  at least once (including one phone notification) — INCLUDING a portability smoke
  test in a second, minimal scratch repo, covering preconditions check, per-repo
  config, and a dry run; and RUNBOOK.md covering start, resume, intervention,
  re-execution, and ONBOARDING A NEW REPO from zero.
- Nothing runs against a real task sequence until I approve, after reviewing smoke
  test results.

## Tier 2 — Decisions with rationale (revisit ONLY if discovery contradicts the
## rationale; if so, stop and tell me rather than silently substituting)
- Fresh context per task via headless invocations driven by a thin script, not one
  long session. Rationale: context exhaustion and drifting loop discipline.
- Review-loop decisions read structured, schema-conforming output (severity counts +
  recommendation), not prose. Rationale: prose interpretation is the least reliable
  link in the loop.
- Loop semantics: severity counts decide continuation (any high/critical open = not
  done). The redteam's recommendation only selects between "fix and re-review" and
  "escalate to me". Bounded rounds; ceiling breach escalates. Rationale: closes an
  undefined state (recommendation=accept with high findings open) found in review.
- Per-task flow: implement → verify build/tests → /handoff → review loop → independent
  verification → merge.
- Orchestration state has exactly one writer (the driver layer), living outside the
  per-task checkouts and outside any repo's tracked files. Sessions report state
  upward; they never write the state file. Rationale: fragmentation and races.
- Independent verification of completion claims runs with fresh context, separate from
  the implementing/reviewing sessions.
- Model tier is selected per task (opus default; fable for high-complexity tasks)
  based on a complexity assessment made during planning.
- Pre-flight before any implementation: audit all task docs referenced by the given
  manifest; only material ambiguities (would cause meaningfully different
  implementations) block; proposed doc optimizations go to me for approval; RUNBOOK
  documents how the gate reopens after I edit docs.
- After all tasks complete: a whole-system adversarial review focused on cross-task
  seams, then /code-report.

## Tier 3 — Uncertainties: investigate before deciding. Each lists the problem and
## what "resolved" means — not the answer.
- My commands' actual contracts AND levels: what /handoff, /redteam-code,
  /fix-findings, /deep-options, /remote-control, /code-report, /review-skill really
  consume and emit; whether /redteam-code has severity levels; and which commands are
  user-level vs project-level. Project-level dependencies are a portability problem —
  propose whether to promote them, and how the preconditions check handles a repo
  where a required command is absent. NOTE: all custom; the only source of truth is
  the command files themselves, never similarly-named public skills. Resolved when:
  each contract and level is documented, with any adapter or promotion it needs.
- Per-repo configuration mechanism: how build/test commands, integration branch,
  dependency install, and any repo-specific settings are declared (config file in
  repo? central registry? detection with confirmation?). Resolved when: a second repo
  can be onboarded via documented steps without editing system code.
- Per-repo state location and keying: where state lives so multiple repos and
  manifests coexist and resume targets the right run. Resolved when: two repos run
  without collision and resume works in both.
- Severity-scale alignment: /review-skill uses CRITICAL/HIGH/MEDIUM/LOW and a shared
  ~/.claude/reference/review-axes.md. Check whether /redteam-code follows the same
  conventions; if so, reuse that taxonomy in the findings schema. A lead, not an
  assumption — verify in the command files.
- Manifest format contract: what structure a manifest must have (task ordering, doc
  references), using docs/SEQUENCE.txt as the reference instance. Resolved when:
  documented in RUNBOOK.md and the parser tolerates it.
- Hook scoping mechanism: how user-level orchestration hooks stay inert everywhere
  outside an active run. Resolved when: demonstrated — interactive sessions in BOTH
  the reference repo and an unrelated repo trigger none of them.
- Unattended question flow: headless runs can't wait mid-run for a phone reply.
  Resolved when: a working blocker → notify → my answer → resume path is tested once
  end to end. Investigate whether /remote-control already provides part of this.
- Worktree lifecycle: creation timing vs dependency order, per-worktree dependency
  install (per the repo's config), cleanup after headless runs, integration branch
  creation if absent. Resolved when: task N+1 sees task N's merged output, tests run
  in a fresh worktree, and no stale worktrees remain after a run.
- Where independent verification hooks in: event-triggered vs an explicit driver
  step. (Caution: TaskCompleted fires on internal todo items, not manifest tasks.)
  Resolved when: it runs exactly once per task, between review-clean and merge.
- Parallelism: whether manifest tasks are provably independent, and whether it is
  worth its coordination cost. Sequential is the default; parallelism must be earned
  with evidence.
- Anything else discovery surfaces that contradicts Tier 2: stop and raise it.

## Process
1. DISCOVERY: resolve every Tier 3 item answerable by reading this repo and my user
   config (~/.claude). Inventory existing hooks/settings/agents/automations at BOTH
   levels; audit the reference manifest + its task docs. Output DISCOVERY.md + open
   judgment questions (via /deep-options). STOP for my review.
2. ALIGNMENT: I settle remaining judgment calls (possibly via a grilling session).
3. BUILD: honor all tiers; install deliverables at user level.
4. VERIFY: dry-run + smoke tests per Tier 1 — including the second-repo portability
   test — then the interactive /review-skill pass over new prompt artifacts. STOP
   for my approval before any real run.
