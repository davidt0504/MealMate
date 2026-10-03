//! Cover draft rows and their request receipts (OPT-007 §4–§5). Storage keeps the payload as
//! opaque text and enforces only what a table can: household ownership on every read and write,
//! one open draft per window, compare-and-swap on the revision, and one receipt per request id.
//! Every transition rule lives in `kimatta-application`.

use rusqlite::{params, OptionalExtension, Transaction};

use crate::{format_civil_date, parse_civil_date, CivilDate, HouseholdId, StorageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftState {
    Active,
    NeedsReview,
    Accepted,
    Discarded,
    Expired,
}

impl DraftState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::NeedsReview => "needs_review",
            Self::Accepted => "accepted",
            Self::Discarded => "discarded",
            Self::Expired => "expired",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        [
            Self::Active,
            Self::NeedsReview,
            Self::Accepted,
            Self::Discarded,
            Self::Expired,
        ]
        .into_iter()
        .find(|s| s.as_str() == raw)
    }

    /// Open drafts can still change; the rest are history.
    pub fn is_open(self) -> bool {
        matches!(self, Self::Active | Self::NeedsReview)
    }
}

/// The window a draft plans: the unique key an open draft is found by. `scope` is the enabled
/// slots joined by `,` in canonical order, so a changed scope is a different window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftWindow {
    pub anchor: CivilDate,
    pub length_days: u32,
    pub scope: String,
    pub end: CivilDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftRow {
    pub id: String,
    pub household_id: HouseholdId,
    pub window: DraftWindow,
    pub revision: i64,
    pub state: DraftState,
    pub planning_identity: String,
    pub review_identity: String,
    pub format_version: u32,
    pub algorithm_version: u32,
    pub similarity_version: u32,
    pub payload: Option<String>,
    pub undo_payload: Option<String>,
    pub created_on: CivilDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftReceipt {
    pub household_id: HouseholdId,
    pub request_id: String,
    pub draft_id: String,
    pub operation: String,
    pub request_digest: String,
    pub revision: i64,
    pub state: DraftState,
    /// The operation's own result, small and typed by the application layer.
    pub outcome: String,
    pub ledger_entry_id: Option<String>,
}

const COLUMNS: &str = "id, household_id, anchor_date, length_days, scope, window_end, revision, \
     state, planning_identity, review_identity, format_version, algorithm_version, \
     similarity_version, payload, undo_payload, created_on";

fn corrupt(id: &str, column: &'static str, value: &str) -> StorageError {
    StorageError::CorruptRow {
        table: "planning_draft",
        id: id.to_owned(),
        column,
        value: value.to_owned(),
    }
}

type RawRow = (
    String,
    String,
    String,
    u32,
    String,
    String,
    i64,
    String,
    String,
    String,
    u32,
    u32,
    u32,
    Option<String>,
    Option<String>,
    String,
);

fn raw(r: &rusqlite::Row<'_>) -> rusqlite::Result<RawRow> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
        r.get(7)?,
        r.get(8)?,
        r.get(9)?,
        r.get(10)?,
        r.get(11)?,
        r.get(12)?,
        r.get(13)?,
        r.get(14)?,
        r.get(15)?,
    ))
}

fn row_from(raw: RawRow) -> Result<DraftRow, StorageError> {
    let (
        id,
        household,
        anchor,
        length_days,
        scope,
        end,
        revision,
        state,
        planning_identity,
        review_identity,
        format_version,
        algorithm_version,
        similarity_version,
        payload,
        undo_payload,
        created_on,
    ) = raw;
    let date = |column: &'static str, value: &str| {
        parse_civil_date(value).map_err(|_| corrupt(&id, column, value))
    };
    Ok(DraftRow {
        household_id: HouseholdId::new(household)?,
        window: DraftWindow {
            anchor: date("anchor_date", &anchor)?,
            length_days,
            scope,
            end: date("window_end", &end)?,
        },
        revision,
        state: DraftState::parse(&state).ok_or_else(|| corrupt(&id, "state", &state))?,
        planning_identity,
        review_identity,
        format_version,
        algorithm_version,
        similarity_version,
        payload,
        undo_payload,
        created_on: date("created_on", &created_on)?,
        id,
    })
}

/// Inserts a new draft. The partial unique index refuses a second open draft for one window.
pub fn insert_draft_in(tx: &Transaction<'_>, row: &DraftRow) -> Result<(), StorageError> {
    crate::require_household(tx, &row.household_id)?;
    tx.execute(
        &format!(
            "INSERT INTO planning_draft ({COLUMNS})
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"
        ),
        params![
            row.id,
            row.household_id.as_str(),
            format_civil_date(row.window.anchor),
            row.window.length_days,
            row.window.scope,
            format_civil_date(row.window.end),
            row.revision,
            row.state.as_str(),
            row.planning_identity,
            row.review_identity,
            row.format_version,
            row.algorithm_version,
            row.similarity_version,
            row.payload,
            row.undo_payload,
            format_civil_date(row.created_on),
        ],
    )?;
    Ok(())
}

/// A draft by id, only within its own household: another household's id reads as absent.
pub fn load_draft_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
    id: &str,
) -> Result<Option<DraftRow>, StorageError> {
    tx.query_row(
        &format!("SELECT {COLUMNS} FROM planning_draft WHERE id = ?1 AND household_id = ?2"),
        params![id, household.as_str()],
        raw,
    )
    .optional()?
    .map(row_from)
    .transpose()
}

/// The open draft for exactly this window, if any.
pub fn load_open_draft_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
    window: &DraftWindow,
) -> Result<Option<DraftRow>, StorageError> {
    tx.query_row(
        &format!(
            "SELECT {COLUMNS} FROM planning_draft
             WHERE household_id = ?1 AND anchor_date = ?2 AND length_days = ?3 AND scope = ?4
               AND state IN ('active', 'needs_review')"
        ),
        params![
            household.as_str(),
            format_civil_date(window.anchor),
            window.length_days,
            window.scope,
        ],
        raw,
    )
    .optional()?
    .map(row_from)
    .transpose()
}

/// Every draft of the household that is open or expired, oldest first: what a caller must
/// consider before starting another window.
pub fn list_unclosed_drafts_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
) -> Result<Vec<DraftRow>, StorageError> {
    let mut stmt = tx.prepare(&format!(
        "SELECT {COLUMNS} FROM planning_draft
         WHERE household_id = ?1 AND state IN ('active', 'needs_review', 'expired')
         ORDER BY created_on, id"
    ))?;
    let rows = stmt
        .query_map(params![household.as_str()], raw)?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter().map(row_from).collect()
}

/// Writes `row` over the stored draft if and only if the stored revision is still `expected`.
/// Everything but the id, household and window is replaced; the caller sets the new revision.
pub fn update_draft_in(
    tx: &Transaction<'_>,
    row: &DraftRow,
    expected: i64,
) -> Result<(), StorageError> {
    let changed = tx.execute(
        "UPDATE planning_draft SET revision = ?1, state = ?2, planning_identity = ?3,
             review_identity = ?4, format_version = ?5, algorithm_version = ?6,
             similarity_version = ?7, payload = ?8, undo_payload = ?9
         WHERE id = ?10 AND household_id = ?11 AND revision = ?12",
        params![
            row.revision,
            row.state.as_str(),
            row.planning_identity,
            row.review_identity,
            row.format_version,
            row.algorithm_version,
            row.similarity_version,
            row.payload,
            row.undo_payload,
            row.id,
            row.household_id.as_str(),
            expected,
        ],
    )?;
    if changed == 0 {
        if load_draft_in(tx, &row.household_id, &row.id)?.is_none() {
            return Err(StorageError::NoSuchDraft {
                draft: row.id.clone(),
                household: row.household_id.as_str().to_owned(),
            });
        }
        return Err(StorageError::DraftRevisionMismatch {
            draft: row.id.clone(),
            expected,
        });
    }
    Ok(())
}

/// Closes every open draft whose window ended before `today`. One the household acted on (it
/// has a receipt) becomes expired and keeps its last display; one it never touched is
/// discarded outright, so opening Cover without planning leaves no tile behind. Expired drafts
/// whose window ended before `purge_before` are then discarded with their payload dropped.
/// Dates come from the caller (Rust reads no clock). Returns `(closed, purged)`. Ledger rows
/// and receipts are never touched.
pub fn expire_drafts_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
    today: CivilDate,
    purge_before: CivilDate,
) -> Result<(usize, usize), StorageError> {
    let untouched = tx.execute(
        "UPDATE planning_draft
         SET state = 'discarded', payload = NULL, undo_payload = NULL, revision = revision + 1
         WHERE household_id = ?1 AND state IN ('active', 'needs_review') AND window_end < ?2
           AND NOT EXISTS (
               SELECT 1 FROM planning_draft_receipt r WHERE r.draft_id = planning_draft.id
           )",
        params![household.as_str(), format_civil_date(today)],
    )?;
    let expired = tx.execute(
        "UPDATE planning_draft SET state = 'expired', revision = revision + 1
         WHERE household_id = ?1 AND state IN ('active', 'needs_review') AND window_end < ?2",
        params![household.as_str(), format_civil_date(today)],
    )?;
    let purged = tx.execute(
        "UPDATE planning_draft SET state = 'discarded', payload = NULL, undo_payload = NULL
         WHERE household_id = ?1 AND state = 'expired' AND window_end < ?2",
        params![household.as_str(), format_civil_date(purge_before)],
    )?;
    Ok((untouched + expired, purged))
}

pub fn load_receipt_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
    request_id: &str,
) -> Result<Option<DraftReceipt>, StorageError> {
    let found = tx
        .query_row(
            "SELECT draft_id, operation, request_digest, revision, state, outcome,
                    ledger_entry_id
             FROM planning_draft_receipt WHERE household_id = ?1 AND request_id = ?2",
            params![household.as_str(), request_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .optional()?;
    found
        .map(
            |(draft_id, operation, request_digest, revision, state, outcome, ledger)| {
                Ok(DraftReceipt {
                    household_id: household.clone(),
                    request_id: request_id.to_owned(),
                    state: DraftState::parse(&state).ok_or_else(|| StorageError::CorruptRow {
                        table: "planning_draft_receipt",
                        id: request_id.to_owned(),
                        column: "state",
                        value: state.clone(),
                    })?,
                    draft_id,
                    operation,
                    request_digest,
                    revision,
                    outcome,
                    ledger_entry_id: ledger,
                })
            },
        )
        .transpose()
}

/// Saves a receipt. The same request id with the same digest is already recorded and is a
/// no-op; with a different digest it is [`StorageError::ReceiptConflict`].
pub fn save_receipt_in(tx: &Transaction<'_>, receipt: &DraftReceipt) -> Result<(), StorageError> {
    if let Some(existing) = load_receipt_in(tx, &receipt.household_id, &receipt.request_id)? {
        if existing.request_digest == receipt.request_digest {
            return Ok(());
        }
        return Err(StorageError::ReceiptConflict {
            request: receipt.request_id.clone(),
        });
    }
    tx.execute(
        "INSERT INTO planning_draft_receipt (household_id, request_id, draft_id, operation,
             request_digest, revision, state, outcome, ledger_entry_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            receipt.household_id.as_str(),
            receipt.request_id,
            receipt.draft_id,
            receipt.operation,
            receipt.request_digest,
            receipt.revision,
            receipt.state.as_str(),
            receipt.outcome,
            receipt.ledger_entry_id,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, TransactionBehavior};

    use super::*;
    use crate::{insert_household, open, Household, HouseholdMember, MemberId};

    fn hid(id: &str) -> HouseholdId {
        HouseholdId::new(id).unwrap()
    }

    fn date(s: &str) -> CivilDate {
        parse_civil_date(s).unwrap()
    }

    fn seeded() -> Connection {
        let mut conn = open(":memory:").unwrap();
        for id in ["h", "other"] {
            insert_household(
                &mut conn,
                &Household {
                    id: hid(id),
                    name: None,
                },
                &[HouseholdMember {
                    id: MemberId::new(format!("m-{id}")).unwrap(),
                    household_id: hid(id),
                    display_name: "A".to_owned(),
                }],
            )
            .unwrap();
        }
        conn
    }

    fn window() -> DraftWindow {
        DraftWindow {
            anchor: date("2026-09-28"),
            length_days: 7,
            scope: "dinner".to_owned(),
            end: date("2026-10-04"),
        }
    }

    fn draft(id: &str, household: &str) -> DraftRow {
        DraftRow {
            id: id.to_owned(),
            household_id: hid(household),
            window: window(),
            revision: 0,
            state: DraftState::Active,
            planning_identity: "p".to_owned(),
            review_identity: "review".to_owned(),
            format_version: 1,
            algorithm_version: 3,
            similarity_version: 1,
            payload: Some("{}".to_owned()),
            undo_payload: None,
            created_on: date("2026-09-28"),
        }
    }

    fn with_tx<T>(conn: &mut Connection, f: impl FnOnce(&Transaction<'_>) -> T) -> T {
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let out = f(&tx);
        tx.commit().unwrap();
        out
    }

    fn receipt(request: &str, digest: &str) -> DraftReceipt {
        DraftReceipt {
            household_id: hid("h"),
            request_id: request.to_owned(),
            draft_id: "d1".to_owned(),
            operation: "another".to_owned(),
            request_digest: digest.to_owned(),
            revision: 1,
            state: DraftState::Active,
            outcome: "changed".to_owned(),
            ledger_entry_id: None,
        }
    }

    #[test]
    fn a_draft_round_trips_and_is_invisible_to_another_household() {
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            insert_draft_in(tx, &draft("d1", "h")).unwrap();
            assert_eq!(
                load_draft_in(tx, &hid("h"), "d1").unwrap(),
                Some(draft("d1", "h"))
            );
            assert_eq!(load_draft_in(tx, &hid("other"), "d1").unwrap(), None);
            assert_eq!(
                load_open_draft_in(tx, &hid("h"), &window()).unwrap(),
                Some(draft("d1", "h"))
            );
            assert_eq!(
                load_open_draft_in(tx, &hid("other"), &window()).unwrap(),
                None
            );
        });
    }

    #[test]
    fn one_open_draft_per_window_but_closed_ones_do_not_block_a_new_one() {
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            insert_draft_in(tx, &draft("d1", "h")).unwrap();
            assert!(insert_draft_in(tx, &draft("d2", "h")).is_err());
            // Another household's draft for the same dates is its own.
            insert_draft_in(tx, &draft("d3", "other")).unwrap();
            let mut closed = draft("d1", "h");
            closed.revision = 1;
            closed.state = DraftState::Accepted;
            closed.payload = None;
            update_draft_in(tx, &closed, 0).unwrap();
            insert_draft_in(tx, &draft("d2", "h")).unwrap();
            // A different scope is a different window.
            let mut lunch = draft("d4", "h");
            lunch.window.scope = "lunch,dinner".to_owned();
            insert_draft_in(tx, &lunch).unwrap();
        });
    }

    #[test]
    fn a_stale_revision_or_foreign_household_cannot_update() {
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            insert_draft_in(tx, &draft("d1", "h")).unwrap();
            let mut next = draft("d1", "h");
            next.revision = 1;
            next.payload = Some("{\"slots\":[]}".to_owned());
            update_draft_in(tx, &next, 0).unwrap();
            let mut stale = next.clone();
            stale.revision = 2;
            let err = update_draft_in(tx, &stale, 0).unwrap_err();
            assert!(matches!(
                err,
                StorageError::DraftRevisionMismatch { expected: 0, .. }
            ));
            let mut foreign = next.clone();
            foreign.household_id = hid("other");
            let err = update_draft_in(tx, &foreign, 1).unwrap_err();
            assert!(matches!(err, StorageError::NoSuchDraft { .. }));
            assert_eq!(load_draft_in(tx, &hid("h"), "d1").unwrap(), Some(next));
        });
    }

    #[test]
    fn inserting_for_a_missing_household_is_named_not_a_raw_fk_error() {
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            let err = insert_draft_in(tx, &draft("d1", "ghost")).unwrap_err();
            assert!(matches!(err, StorageError::NoSuchHousehold(_)));
        });
    }

    #[test]
    fn expiry_keeps_a_touched_draft_until_the_purge_date_and_spares_future_drafts() {
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            let mut old = draft("d1", "h");
            old.payload = Some("{\"slots\":[]}".to_owned());
            insert_draft_in(tx, &old).unwrap();
            save_receipt_in(tx, &receipt("req-1", "digest-a")).unwrap();
            let mut future = draft("future", "h");
            future.window.anchor = date("2026-10-05");
            future.window.end = date("2026-10-11");
            insert_draft_in(tx, &future).unwrap();
            // The window ends 10-04: on 10-04 nothing expires.
            assert_eq!(
                expire_drafts_in(tx, &hid("h"), date("2026-10-04"), date("2026-09-04")).unwrap(),
                (0, 0)
            );
            assert_eq!(
                expire_drafts_in(tx, &hid("h"), date("2026-10-05"), date("2026-09-05")).unwrap(),
                (1, 0)
            );
            let old = load_draft_in(tx, &hid("h"), "d1").unwrap().unwrap();
            assert_eq!(old.state, DraftState::Expired);
            assert!(old.payload.is_some(), "a touched draft keeps its display");
            // 30 days after the window end it closes; identity stays.
            assert_eq!(
                expire_drafts_in(tx, &hid("h"), date("2026-11-04"), date("2026-10-05")).unwrap(),
                (1, 1)
            );
            let old = load_draft_in(tx, &hid("h"), "d1").unwrap().unwrap();
            assert_eq!(old.state, DraftState::Discarded);
            assert_eq!(old.payload, None);
            assert_eq!(old.planning_identity, "p");
            // The untouched future draft closed on the same pass, leaving nothing unclosed.
            assert!(list_unclosed_drafts_in(tx, &hid("h")).unwrap().is_empty());
            assert!(list_unclosed_drafts_in(tx, &hid("other"))
                .unwrap()
                .is_empty());
        });
    }

    /// A draft with no receipt was never acted on by the household — a revision bumped only by
    /// review marking does not count — so it closes when its window ends and leaves no tile.
    #[test]
    fn an_untouched_draft_closes_at_expiry() {
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            let mut bumped = draft("d1", "h");
            bumped.revision = 1;
            bumped.state = DraftState::NeedsReview;
            insert_draft_in(tx, &bumped).unwrap();
            assert_eq!(
                expire_drafts_in(tx, &hid("h"), date("2026-10-05"), date("2026-09-05")).unwrap(),
                (1, 0)
            );
            let row = load_draft_in(tx, &hid("h"), "d1").unwrap().unwrap();
            assert_eq!(row.state, DraftState::Discarded);
            assert_eq!(row.payload, None);
            assert!(list_unclosed_drafts_in(tx, &hid("h")).unwrap().is_empty());
        });
    }

    /// An expired row whose payload is already gone (set aside as unreadable, or purged by an
    /// older build) still closes at the purge date.
    #[test]
    fn an_expired_row_without_a_payload_still_closes_at_the_purge_date() {
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            let mut gone = draft("d1", "h");
            gone.state = DraftState::Expired;
            gone.payload = None;
            insert_draft_in(tx, &gone).unwrap();
            assert_eq!(
                expire_drafts_in(tx, &hid("h"), date("2026-11-04"), date("2026-10-05")).unwrap(),
                (0, 1)
            );
            assert!(list_unclosed_drafts_in(tx, &hid("h")).unwrap().is_empty());
        });
    }

    #[test]
    fn a_receipt_is_replay_safe_and_refuses_reuse_with_other_content() {
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            insert_draft_in(tx, &draft("d1", "h")).unwrap();
            save_receipt_in(tx, &receipt("req-1", "digest-a")).unwrap();
            save_receipt_in(tx, &receipt("req-1", "digest-a")).unwrap();
            let err = save_receipt_in(tx, &receipt("req-1", "digest-b")).unwrap_err();
            assert!(matches!(err, StorageError::ReceiptConflict { .. }));
            assert_eq!(
                load_receipt_in(tx, &hid("h"), "req-1").unwrap(),
                Some(receipt("req-1", "digest-a"))
            );
            // Receipts are household-scoped.
            assert_eq!(load_receipt_in(tx, &hid("other"), "req-1").unwrap(), None);
        });
    }

    #[test]
    fn a_failed_transaction_leaves_neither_draft_nor_receipt() {
        let mut conn = seeded();
        {
            let tx = conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .unwrap();
            insert_draft_in(&tx, &draft("d1", "h")).unwrap();
            save_receipt_in(&tx, &receipt("req-1", "a")).unwrap();
            // Dropped without commit: the whole command rolls back.
        }
        with_tx(&mut conn, |tx| {
            assert_eq!(load_draft_in(tx, &hid("h"), "d1").unwrap(), None);
            assert_eq!(load_receipt_in(tx, &hid("h"), "req-1").unwrap(), None);
        });
    }

    /// Migration 16 is additive: a v15 database's meals, locks, policies and ledger read back
    /// identically after it runs, and the new tables start empty.
    #[test]
    fn migrating_a_v15_database_leaves_existing_rows_unchanged() {
        use crate::{
            append_ledger_entry, list_ledger_entries, list_planned_meals, list_policies,
            save_planned_meal, save_planning_cycle, save_policy, set_planned_meal_lock,
            EvidenceSource, LedgerEntry, LedgerEntryId, MealComponent, MealScope, MealSlot,
            OutcomeStatus, PlannedMeal, PlannedMealId, PlanningCycle, Policy, PolicyId,
            WriteSource, MIGRATIONS,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        let from = date("2026-09-28");
        let to = date("2026-10-04");
        let (meals, policies, ledger) = {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 15).unwrap();
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            insert_household(
                &mut raw,
                &Household {
                    id: hid("h"),
                    name: None,
                },
                &[HouseholdMember {
                    id: MemberId::new("m").unwrap(),
                    household_id: hid("h"),
                    display_name: "A".to_owned(),
                }],
            )
            .unwrap();
            save_planning_cycle(
                &mut raw,
                &PlanningCycle::new(hid("h"), from, 7, MealScope::dinner_only()).unwrap(),
            )
            .unwrap();
            let id = PlannedMealId::new("pm").unwrap();
            let meal = PlannedMeal::new(
                id.clone(),
                hid("h"),
                from,
                MealSlot::Dinner,
                vec![MealComponent::FrozenQuick { note: None }],
                false,
            )
            .unwrap();
            save_planned_meal(&mut raw, &meal, WriteSource::User).unwrap();
            set_planned_meal_lock(&mut raw, &hid("h"), &id, true, WriteSource::User).unwrap();
            save_policy(
                &mut raw,
                &Policy::new(
                    PolicyId::new("p").unwrap(),
                    hid("h"),
                    "food",
                    "food.hard_veto",
                    std::collections::BTreeMap::from([("subject".to_owned(), "liver".to_owned())]),
                    true,
                    EvidenceSource::ExplicitUser,
                )
                .unwrap(),
            )
            .unwrap();
            append_ledger_entry(
                &mut raw,
                &LedgerEntry {
                    id: LedgerEntryId::new("l").unwrap(),
                    household_id: hid("h"),
                    controller_id: "food".to_owned(),
                    algorithm_version: 2,
                    snapshot_hash: "0".to_owned(),
                    reason_codes: Vec::new(),
                    selected_action: "propose".to_owned(),
                    prior_status: OutcomeStatus::Unresolved,
                    resulting_status: OutcomeStatus::Unresolved,
                    payload: "x".to_owned(),
                },
            )
            .unwrap();
            (
                list_planned_meals(&raw, &hid("h"), from, to).unwrap(),
                list_policies(&raw, &hid("h"), "food").unwrap(),
                list_ledger_entries(&raw, &hid("h")).unwrap(),
            )
        };
        assert!(meals[0].locked());
        let conn = open(&path).unwrap();
        assert_eq!(crate::schema_version(&conn).unwrap(), 17);
        assert_eq!(
            list_planned_meals(&conn, &hid("h"), from, to).unwrap(),
            meals
        );
        assert_eq!(list_policies(&conn, &hid("h"), "food").unwrap(), policies);
        assert_eq!(list_ledger_entries(&conn, &hid("h")).unwrap(), ledger);
        let drafts: i64 = conn
            .query_row("SELECT count(*) FROM planning_draft", [], |r| r.get(0))
            .unwrap();
        assert_eq!(drafts, 0);
    }

    #[test]
    fn drafts_travel_in_an_export() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        let mut conn = seeded();
        with_tx(&mut conn, |tx| {
            insert_draft_in(tx, &draft("d1", "h")).unwrap();
            save_receipt_in(tx, &receipt("req-1", "a")).unwrap();
        });
        crate::export_database(&conn, &path).unwrap();
        let mut copy = open(&path).unwrap();
        with_tx(&mut copy, |tx| {
            assert_eq!(
                load_draft_in(tx, &hid("h"), "d1").unwrap(),
                Some(draft("d1", "h"))
            );
            assert!(load_receipt_in(tx, &hid("h"), "req-1").unwrap().is_some());
        });
    }
}
