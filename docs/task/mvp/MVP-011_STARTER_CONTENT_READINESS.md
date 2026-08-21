# MVP-011 — Starter content readiness

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Implementation/content |
| Workstream | Recipes/content |
| Depends on | MVP-007, MVP-008, MVP-009 |
| Complexity | Focused |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | Read-only source research allowed; no scraped/licensed purchase content |

## Outcome and user value

Ship a small set of genuinely useful, tested recipes so first-time planning works before users build a library.

## Authoritative sources

- `docs/PRD_v2.md` §§7.5, 13.2, 21.3, 24.3; `docs/ROADMAP.md` D-014; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Use original recipes plus verified US federal/public-domain/CC0 material; CC BY only if attribution survives every relevant surface.
- Exclude scraped material and BY-SA, NC, or ND licenses for MVP.
- Maintain a provenance manifest with source, rights basis, author, attribution, modifications, and verification date.
- Every shipped recipe is cooked at least once; target the best 30, not a quota. Use no unverified photos.

## Scope

- Define selection rubric, author/source candidates, normalize to the structured model, validate restrictions/units, conduct cook review, and package seed content/provenance.

## Non-goals

- Large catalog, automated scraping, nutritional certification, copied editorial prose, or public content marketplace.

## Decision gates

- Any uncertain rights case is excluded or separately reviewed; quantity targets may shrink rather than lower quality.

## Acceptance criteria

- **AC-1:** Every included recipe has complete provenance and an allowed rights basis.
- **AC-2:** Every included recipe round-trips in the app model and passes restriction/unit checks.
- **AC-3:** Every shipped recipe has a recorded cook review and actionable corrections resolved.
- **AC-4:** No unverified image or prohibited license is packaged.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Independently reviewed provenance manifest |
| AC-2 | Automated fixture/schema tests |
| AC-3 | Human cook log; NOT VERIFIED cannot ship |
| AC-4 | Asset/license inventory review |

## Stop/failure conditions

- Stop on unclear rights, unverifiable attribution, unsafe ambiguity, or pressure to hit count over quality. Two cycles then re-plan.

## Handoff

Record recipe count, evidence, exclusions, and status in `docs/ROADMAP.md`.
