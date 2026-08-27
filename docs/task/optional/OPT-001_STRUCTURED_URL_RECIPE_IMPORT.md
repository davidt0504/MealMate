# OPT-001 — Structured URL recipe import

> Optional planning input, not an MVP dependency or approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Optional stretch implementation |
| Workstream | Recipe import |
| Depends on | MVP-008 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | Only when it cannot delay a required MVP card |
| Recommended workflow | `deep-options` if extraction boundary is unclear, then `plan-task` |
| External actions | Network access to public URLs only; obey site access/rights boundaries |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below.

## Workflow gate

Before planning or execution, read `docs/ROADMAP.md` and apply the mandatory planning or execution gate in `docs/task/README.md` for `OPT-001`. Do not begin implementation unless it passes. Any implementation plan must repeat the execution gate as its first execution step. Because this card is optional, the owner must explicitly select it without displacing required MVP work.

## Outcome and user value

Let users prefill a new private recipe from deterministic structured metadata at a URL, then confirm and correct it before saving.

## Authoritative sources

- `docs/PRD_v3.md` §9.1 (LLMs at the messy boundary, never the recurring controller), §9.3 (an LLM may not silently invent a trusted action), §20 (variable-cost LLM convenience), §14 items 4–5 (external and LLM output are untrusted)
- `docs/HOUSEHOLD_CONTROL_PRINCIPLES.md` §7 (candidate-action completeness), §12 (algorithm hierarchy)
- `docs/ROADMAP.md` D-016, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 9, 12, 17
- Historical (D-028): `docs/PRD_v2.md` §§7.6, 13.2, 21.3, 23

## Load-bearing constraints

- Parse schema.org/JSON-LD and other explicitly approved deterministic metadata only; no LLM dependency on this path.
- Preserve source URL, attribution/provenance, original text, uncertainty, and user edits.
- Never promise support for arbitrary sites, paywalls, blocked pages, hostile markup, or copyright-sensitive republishing.
- Import is private by default and always requires review/confirmation before save.
- Imported content is untrusted input. It becomes ordinary structured Rust-owned state only after deterministic validation, and it can never modify policy, restrictions, or authority (PRD §14 item 4, invariant 17).
- If a later card adds an LLM or OCR lane, it belongs at the messy-input boundary producing *candidates*, never in the deterministic path and never as a source of authoritative state (PRD §9.1, principles §12).

## Scope

- URL validation/fetch boundary, structured-data extraction, mapping/uncertainty, preview/edit/confirm UI, provenance, failure messaging, security limits, and adversarial fixtures.

## Non-goals

- General scraping, browser automation, OCR, LLM fallback, bypassing access controls, automatic public sharing, or MVP completion dependency.

## Decision gates

- If useful coverage requires general scraping or generative extraction, stop and return the option to post-MVP research.

## Acceptance criteria

- **AC-1:** Supported structured recipes prefill reviewable fields while preserving source/provenance.
- **AC-2:** Unsupported, malformed, oversized, redirected, blocked, or hostile inputs fail safely and clearly.
- **AC-3:** Nothing is persisted before confirmation; edits round-trip through the normal recipe model.
- **AC-4:** Rights/privacy/security review confirms no access-control bypass or automatic public republication.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Licensed/local fixture integration tests |
| AC-2 | Adversarial fetch/parser tests |
| AC-3 | Widget/repository tests |
| AC-4 | Fresh-context rights/security review |

## Stop/failure conditions

- Stop if this delays MVP, needs site-specific scraping, bypasses restrictions, loses provenance, or introduces recurring LLM cost. Two cycles then defer.

## Handoff

Record evidence/status separately in `docs/ROADMAP.md`; it never blocks MVP-022.
