# OPT-001 — Structured URL recipe import

> Optional planning input, not an MVP dependency or approved execution plan. Recommended workflows do not invoke or authorize themselves.

| Field | Value |
|---|---|
| Status | Draft |
| Type | Optional stretch implementation |
| Workstream | Recipe import |
| Depends on | MVP-008 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | Only when it cannot delay a required MVP card |
| Recommended workflow | `deep-options` if extraction boundary is unclear, then `plan-task` |
| External actions | Network access to public URLs only; obey site access/rights boundaries |

> **v3 amendment (2026-08-24, D-028/D-029):** LLM/OCR import stays at the messy-input boundary (principles §12); accepted output becomes ordinary structured Rust-owned state after deterministic validation. Authoritative source adds `docs/PRD_v3.md` §9.1, §20; `PRD_v2` citations are historical. Re-derive this card after DEC-005 is Done; its status stays Draft until then.

## Outcome and user value

Let users prefill a new private recipe from deterministic structured metadata at a URL, then confirm and correct it before saving.

## Authoritative sources

- `docs/PRD_v2.md` §§7.6, 13.2, 21.3, 23; `docs/ROADMAP.md` D-016; `docs/task/MVP_INVARIANTS.md`

## Load-bearing constraints

- Parse schema.org/JSON-LD and other explicitly approved deterministic metadata only; no LLM dependency.
- Preserve source URL, attribution/provenance, original text, uncertainty, and user edits.
- Never promise support for arbitrary sites, paywalls, blocked pages, hostile markup, or copyright-sensitive republishing.
- Import is private by default and always requires review/confirmation before save.

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
