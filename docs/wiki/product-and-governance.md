# Product and governance

## Authority and intent

`docs/PRD_v3.md` is the current product/architecture statement: Kimatta is a focused food outcome controller on a small household-control kernel. `HOUSEHOLD_CONTROL_PRINCIPLES.md` supplies cross-product principles such as human authority, explicit uncertainty, local/private defaults, reproducibility, and restraint against premature generalization. Older PRDs remain historical context.

`README.md` gives the developer-facing current architecture and build/use instructions, while `V3_IMPLEMENTATION_STATUS.md` and `V3_MIGRATION_PLAN.md` explain the pivot's implemented and deferred boundaries. The roadmap is the live sequencing and decision register, not runtime source.

## Task system

`docs/task/README.md`, `TASK_TEMPLATE.md`, `SEQUENCE.txt`, and `MVP_INVARIANTS.md` define how work is shaped and ordered. Invariants pin household ownership, structured ingredients, conservative aggregation, real planning dates, optional pantry, offline authority, anonymous first launch, warning-only restriction language, local-first privacy, Rust ownership, hard Tier-0 constraints, deterministic planning, and a coarse generated bridge.

Decision cards DEC-001 through DEC-008 govern product identity, engineering foundation, cloud/sharing, naming, Rust/bridge commitment, monetization, legal posture, and production activation. Readiness cards isolate Android, Rust/FRB, and eventual iOS toolchain proof.

MVP cards 001 through 034 provide acceptance criteria and evidence for scaffold through production and post-core growth. The implemented repository currently covers the offline recipe-plan-pantry-shopping loop, controller/planner, backup/recovery, starter content, and associated Android UI. Cards for backend/auth, observability, sharing, billing, legal publication, and production submission describe future gates; their presence is not implementation evidence. Optional cards describe recipe import, staples, and additional meal scopes.

## Research, reviews, and evidence

Research files cover initial product work, naming/trademark screening, starter candidates, and a verifier prompt. `docs/reviews` contains dated workflow/architecture reviews; findings must be read in their historical commit context because the code has advanced substantially since the August architecture review.

Measurement artifacts include two binary screenshots for Cover My Week and an accessibility/attention note. Brand assets contain the source icon and dark/light lockups. These are asset trees and are rolled up here in prose while remaining individually fingerprinted in the coverage manifest.

`refs/owner-recipe-transcriptions.md` and `refs/recipes-pending.json` are source/reference inputs for owner-provided recipe content. They are not runtime configuration. Known-issue ledgers separate current, low-priority, archived, and audit findings. `.impeccable.md` is a design-review instruction artifact rather than application code.

## Working-tree boundary

At report start, `docs/ROADMAP.md` had an uncommitted modification. This page acknowledges that WIP but derives architectural claims from committed code/configuration and authoritative document roles, not from the edit's uncommitted contents. Future incremental reports should describe that change in their brief unless it becomes committed.

## Coverage evidence

This page owns 81 tracked files: governance/root issue ledgers, 74 files under `docs`, and two reference-data files. Fifteen PNGs across brand, measurement, and Android asset trees are classified as binary in the global manifest; the product/governance page owns five of them. No document or reference path is omitted; exact assignments and fingerprints are in `../coverage-manifest.json`.

