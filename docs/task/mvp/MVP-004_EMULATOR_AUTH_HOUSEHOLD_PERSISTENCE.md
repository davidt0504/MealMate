# MVP-004 — Anonymous-first entry and household identity

> Planning input, not an approved execution plan. Recommended workflows do not invoke or authorize themselves.

> Retitled 2026-08-26 under D-034; formerly "Emulator auth, household, and persistence". The ID and filename are kept because the register and four other cards cite `MVP-004`.

| Field | Value |
|---|---|
| Status | See `docs/ROADMAP.md` task register |
| Type | Implementation |
| Workstream | Identity/entry |
| Depends on | MVP-002, MVP-003 |
| Complexity | Complex |
| Assurance | Elevated |
| Sequential batching | No |
| Recommended workflow | `plan-task` |
| External actions | None; on-device evidence uses the Android emulator per D-022 |

> **Re-derived for PRD v3 on 2026-08-26 (D-029, D-034).** The dated 2026-08-24 banner is folded into the body below. `MVP-003` was added to `Depends on` in the same change (register row and Outcome cell updated together, per D-029): the 2026-08-24 banner narrowed this card to household identity **UI** over the Rust kernel, and that UI needs the shell.

## Workflow gate

Before planning, read `docs/ROADMAP.md` and apply the mandatory planning gate in `docs/task/README.md` for `MVP-004`. Before implementation, apply the mandatory execution gate and repeat it as the approved plan's first execution step.

## Outcome and user value

Let a new user start using Kimatta immediately, with no account and no network, and give the household a durable identity in Rust-owned SQLite that every later food card hangs off. Household ownership is the core data boundary (invariant 1); MVP-002 made Rust its owner, and this card is where a real household first exists.

## Authoritative sources

- `docs/PRD_v3.md` §6.4 (SQLite owned by Rust), §6.5 (cloud is optional; core MVP planning has no cloud dependency), §7.1 (household identity), §12 (schema/migration discipline), §15 (First run)
- `docs/ROADMAP.md` D-008, D-015, D-022, D-028, D-030, D-034; `docs/task/MVP_INVARIANTS.md` 1, 8, 15, 17
- Historical (D-028): `docs/PRD_v2.md` §§7.2, 14.2, 14.4, 14.6

## Load-bearing constraints

- Household scoping is enforced in Rust-owned SQLite, not by cloud security rules. A household-scoped read or write never reaches another household's rows (invariants 1, 17).
- First launch creates a local anonymous identity with no network call and no durable account (invariant 8, PRD §15 "No mandatory durable account").
- Solo use is a one-person household; IDs are stable and ownership is explicit.
- Durable-account upgrade, cloud auth, and any sync are deferred to MVP-018; nothing here may depend on them (PRD §6.5).
- Bootstrap is idempotent: a restart must not produce a second household or a duplicate member.

## Scope

- Create and persist the anonymous local identity, the owning household, and its first member through coarse bridge commands over explicit DTOs (invariant 21).
- Household identity UI on the MVP-003 shell: show the household, allow renaming, and make solo-versus-household framing visible without demanding setup.
- Seed and reset tooling against the local database for development.
- Rust unit tests and bridge integration tests; on-device evidence per D-022.

## Non-goals

- Firebase in any form, real or emulated; durable accounts; invitations; multi-user collaboration; production auth providers; rich member profiles.

## Decision gates

- Resolve any schema-shape conflict with MVP-002's `household` / `household_member` tables before writing a migration; a change there is a migration, not an edit (PRD §12).

## Acceptance criteria

- **AC-1:** First launch creates exactly one anonymous identity and one owned household; a second launch reuses both.
- **AC-2:** Household-scoped reads and writes succeed for the owning household and never return or mutate another household's rows.
- **AC-3:** Restart retains household state in SQLite with no duplicate bootstrap records.
- **AC-4:** The core entry path makes no network call on first run or on restart.
- **AC-5:** Identity UI states are accessible and honest when a household has no name set.

## Evidence plan

| Criterion | Required evidence |
|---|---|
| AC-1 | Rust unit test plus a bridge integration test asserting reuse on second open |
| AC-2 | Rust tests with a second household fixture, including negative cases |
| AC-3 | Restart integration test and on-device database inspection over the D-022 bridge |
| AC-4 | Network log or airplane-mode emulator run showing no outbound request |
| AC-5 | Widget test and accessibility inspection record |

## Stop/failure conditions

- Stop on any network dependency in the entry path, any cloud credential request, ambiguous ownership, or a schema change made without a migration. Two cycles then re-plan.

## Handoff

In one `docs/ROADMAP.md` handoff edit, record PASS/FAIL/NOT VERIFIED evidence, the resulting status, EMULATOR-PERSISTENCE-READY progress, and **Next implementation task**.

## Verification record — 2026-08-28

Executed under the approved plan (`~/.claude/plans/read-docs-prd-v3-md-docs-roadmap-md-docs-optimized-hickey.md`) on `orch/6`. The `docs/ROADMAP.md` evidence-log row is the handoff record; the reproducible evidence for the AC-3 persistence claim is transcribed verbatim below. Artifacts under `~/mvp004_evidence/`.

| Criterion | Result | Carrier |
|---|---|---|
| AC-1 | PASS | Rust `ensure_creates_once_then_reuses`, `ensure_persists_across_reopen`; bridge bootstrap-reuse and reopen tests; on-device 1 household / 1 member on first run |
| AC-2 | PASS | Rust `load_household_scopes_members`, `rename_touches_only_the_named_household`, `rename_unknown_household_is_an_error`, `mismatched_member_rejected`; bridge foreign-id rename rejected, different file → different household |
| AC-3 | PASS | `pm clear` → first launch created `kimatta.db` (post-clear mtime); force-stop relaunch keeps identical ids, counts 1/1 |
| AC-4 | PASS | Airplane mode on for both launches; no `INTERNET` in the main manifest; no network crate/package |
| AC-5 | PASS | Five widget tests incl. three a11y guidelines × light/dark; on-device uiautomator dump + screenshots |

### Raw evidence — AC-3 persistence across relaunch

Re-run 2026-08-28 from the pulled artifacts, so the record below is the evidence rather than a
summary of it. Both databases were pulled off the emulator with
`adb exec-out run-as dev.mealmate.temp cat files/kimatta.db > ~/mvp004_evidence/<name>.db` —
`kimatta_first.db` after the first launch following `tools/emulator.sh reset` (`pm clear`), and
`kimatta_relaunch.db` after `am force-stop` and a second launch.

```
$ sqlite3 ~/mvp004_evidence/kimatta_first.db 'PRAGMA user_version; SELECT id, ifnull(name,"(NULL)") FROM household; SELECT id, household_id, display_name FROM household_member;'
1
bdfe3373-f5c8-4aa0-90c1-4ec56e186bfe|(NULL)
ed853676-dd2b-4107-9024-8c5a509d3107|bdfe3373-f5c8-4aa0-90c1-4ec56e186bfe|Me

$ sqlite3 ~/mvp004_evidence/kimatta_relaunch.db 'PRAGMA user_version; SELECT id, ifnull(name,"(NULL)") FROM household; SELECT id, household_id, display_name FROM household_member;'
1
bdfe3373-f5c8-4aa0-90c1-4ec56e186bfe|(NULL)
ed853676-dd2b-4107-9024-8c5a509d3107|bdfe3373-f5c8-4aa0-90c1-4ec56e186bfe|Me

$ ls -l ~/mvp004_evidence/
-rw-r--r-- 1 davidlinux davidlinux 111298 Aug 28 23:07 household_first.png
-rw-r--r-- 1 davidlinux davidlinux 110962 Aug 28 23:07 household_relaunch.png
-rw-r--r-- 1 davidlinux davidlinux  20480 Aug 28 23:06 kimatta_first.db
-rw-r--r-- 1 davidlinux davidlinux  20480 Aug 28 23:07 kimatta_relaunch.db
-rw-r--r-- 1 davidlinux davidlinux 115920 Aug 28 23:06 settings_first.png
```

That is the AC-3 claim in full: schema v1 both times, one household and one member both times,
identical ids across the force-stop, and the household still unnamed. Anyone with the two `.db`
files can re-run those two commands and get these bytes back.

Screenshots stay outside the tree (binary): `settings_first.png`, `household_first.png`,
`household_relaunch.png`, each captured with
`adb exec-out screencap -p > ~/mvp004_evidence/<name>.png`.

**Transcribed, not reproducible from artifacts.** These two reads were live device state that no
pulled artifact carries, so they rest on transcription and are marked as such rather than being
presented as raw output:

- `run-as dev.mealmate.temp ls -l files/kimatta.db` → mtime `2026-08-28 23:05` device-local,
  i.e. after the `04:05:54Z` `pm clear` — the first on-device first-create observed on any card.
- `settings get global airplane_mode_on` → `1` for both launches (AC-4).

Reproducing either requires another on-device run; the AC-3 rows above do not.

Status: `Verify` — Elevated tier; awaiting fresh-context verifier and owner approval.

## Dependencies added by this card

- `uuid` 1.26.0 (`features = ["v4"]`), bridge crate only. Rationale: household/member ids must be globally unique and stable if MVP-018 later links identities; PRD §6.6 lists it as a likely choice. Rejected: `rowid`-derived ids (not stable across export/merge). Test implications: none — ids are opaque strings to every test; the storage crate stays id-agnostic. Reversal cost: one `Cargo.toml` line and two `Uuid::new_v4()` calls in `api/household.rs`. D-031: in-scope evidence record, not a Draft event.
