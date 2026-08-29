//! Rust-owned SQLite (PRD v3 §12): foreign keys on, explicit migrations, transactions.
#![forbid(unsafe_code)]

use std::path::Path;

pub use household_core::{Household, HouseholdId, HouseholdMember, IdError, MemberId};
pub use rusqlite::Connection;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use rusqlite_migration::{Migrations, M};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Migration(#[from] rusqlite_migration::Error),
    #[error(transparent)]
    Id(#[from] IdError),
    #[error("member {member} belongs to household {actual}, not {expected}")]
    MemberHouseholdMismatch {
        member: String,
        expected: String,
        actual: String,
    },
    #[error("no household with id {0}")]
    NoSuchHousehold(String),
}

// Two consts: `M` has drop glue, so an inline `&[M::up(..)]` argument is not promoted
// (E0716); a `&[..]` tail in a const initializer is lifetime-extended.
const MIGRATION_ARRAY: &[M] = &[M::up(
    "CREATE TABLE household (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT
    ) STRICT;
    CREATE TABLE household_member (
        id TEXT PRIMARY KEY NOT NULL,
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        display_name TEXT NOT NULL
    ) STRICT;",
)];
pub const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_ARRAY);

/// Opens (creating if absent) the database at `path`, migrates to latest, enables foreign keys.
/// Foreign keys are off across `to_latest` because migrations run inside one transaction, so a
/// migration cannot disable them itself; a migration that rebuilds a table should therefore carry
/// `.foreign_key_check()`.
pub fn open(path: impl AsRef<Path>) -> Result<Connection, StorageError> {
    let mut conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    MIGRATIONS.to_latest(&mut conn)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

pub fn schema_version(conn: &Connection) -> Result<u32, StorageError> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Inserts the household and its member rows atomically; any failure rolls back everything.
/// Every member must carry `household.id` — a member naming another household is rejected
/// before any row is written (the foreign key alone only tests existence).
pub fn insert_household(
    conn: &mut Connection,
    household: &Household,
    members: &[HouseholdMember],
) -> Result<(), StorageError> {
    let tx = conn.transaction()?;
    insert_rows(&tx, household, members)?;
    tx.commit()?;
    Ok(())
}

fn insert_rows(
    conn: &Connection,
    household: &Household,
    members: &[HouseholdMember],
) -> Result<(), StorageError> {
    if let Some(m) = members.iter().find(|m| m.household_id != household.id) {
        return Err(StorageError::MemberHouseholdMismatch {
            member: m.id.as_str().to_owned(),
            expected: household.id.as_str().to_owned(),
            actual: m.household_id.as_str().to_owned(),
        });
    }
    conn.execute(
        "INSERT INTO household (id, name) VALUES (?1, ?2)",
        params![household.id.as_str(), household.name],
    )?;
    for m in members {
        conn.execute(
            "INSERT INTO household_member (id, household_id, display_name) VALUES (?1, ?2, ?3)",
            params![m.id.as_str(), m.household_id.as_str(), m.display_name],
        )?;
    }
    Ok(())
}

/// A household with its members, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HouseholdRecord {
    pub household: Household,
    pub members: Vec<HouseholdMember>,
}

/// The single local household, or `None` before bootstrap. Takes the oldest row so a
/// second household — which nothing should create — can never displace the first.
pub fn load_household(conn: &Connection) -> Result<Option<HouseholdRecord>, StorageError> {
    let row = conn
        .query_row(
            "SELECT id, name FROM household ORDER BY rowid LIMIT 1",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
        )
        .optional()?;
    with_members(conn, row)
}

/// The household with exactly `id`, or `None`. Unlike [`load_household`], does not assume
/// a single household exists, so a caller holding an id gets that household back rather
/// than whichever one is oldest.
pub fn load_household_by_id(
    conn: &Connection,
    id: &HouseholdId,
) -> Result<Option<HouseholdRecord>, StorageError> {
    let row = conn
        .query_row(
            "SELECT id, name FROM household WHERE id = ?1",
            params![id.as_str()],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
        )
        .optional()?;
    with_members(conn, row)
}

/// Attaches the member rows to a household row, so both loaders scope members identically.
fn with_members(
    conn: &Connection,
    row: Option<(String, Option<String>)>,
) -> Result<Option<HouseholdRecord>, StorageError> {
    let Some((id, name)) = row else {
        return Ok(None);
    };
    let household = Household {
        id: HouseholdId::new(id)?,
        name,
    };
    let mut stmt = conn.prepare(
        "SELECT id, display_name FROM household_member WHERE household_id = ?1 ORDER BY rowid",
    )?;
    let members = stmt
        .query_map(params![household.id.as_str()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .map(|row| {
            let (id, display_name) = row?;
            Ok(HouseholdMember {
                id: MemberId::new(id)?,
                household_id: household.id.clone(),
                display_name,
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    Ok(Some(HouseholdRecord { household, members }))
}

/// Returns the local household, creating `candidate` with `member` if none exists yet.
/// Select and insert share one IMMEDIATE transaction, so concurrent callers cannot both
/// create; `candidate` is ignored when a household already exists.
pub fn ensure_household(
    conn: &mut Connection,
    candidate: &Household,
    member: &HouseholdMember,
) -> Result<HouseholdRecord, StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let record = match load_household(&tx)? {
        Some(existing) => existing,
        None => {
            insert_rows(&tx, candidate, std::slice::from_ref(member))?;
            HouseholdRecord {
                household: candidate.clone(),
                members: vec![member.clone()],
            }
        }
    };
    tx.commit()?;
    Ok(record)
}

/// Sets the name of exactly the household `id`; trimmed, and blank clears it to `NULL`
/// (an unnamed household is a valid state — the UI frames it honestly rather than inventing
/// a name). Never touches another household's row.
pub fn rename_household(
    conn: &Connection,
    id: &HouseholdId,
    name: Option<&str>,
) -> Result<(), StorageError> {
    let name = name.map(str::trim).filter(|n| !n.is_empty());
    let changed = conn.execute(
        "UPDATE household SET name = ?1 WHERE id = ?2",
        params![name, id.as_str()],
    )?;
    if changed == 0 {
        return Err(StorageError::NoSuchHousehold(id.as_str().to_owned()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use household_core::{HouseholdId, MemberId};

    use super::*;

    fn household(id: &str) -> Household {
        Household {
            id: HouseholdId::new(id).unwrap(),
            name: Some("Home".into()),
        }
    }

    fn member(id: &str, household_id: &str) -> HouseholdMember {
        HouseholdMember {
            id: MemberId::new(id).unwrap(),
            household_id: HouseholdId::new(household_id).unwrap(),
            display_name: id.to_uppercase(),
        }
    }

    fn count(conn: &Connection, table: &str) -> u32 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn empty_db_migrates_to_v1() {
        let conn = open(":memory:").unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 1);
    }

    #[test]
    fn migrations_validate() {
        MIGRATIONS.validate().unwrap();
    }

    #[test]
    fn open_leaves_foreign_keys_on() {
        let conn = open(":memory:").unwrap();
        let on: i32 = conn
            .pragma_query_value(None, "foreign_keys", |r| r.get(0))
            .unwrap();
        assert_eq!(on, 1);
    }

    #[test]
    fn orphan_member_rejected() {
        let mut conn = open(":memory:").unwrap();
        let err =
            insert_household(&mut conn, &household("h"), &[member("m", "other")]).unwrap_err();
        // The mis-parent guard, not the foreign key: `insert_rows` rejects a member naming
        // another household before either INSERT runs, so SQLite never sees these rows.
        // `foreign_key_rejects_absent_parent` is what covers the FK itself.
        assert!(matches!(err, StorageError::MemberHouseholdMismatch { .. }));
        assert_eq!(count(&conn, "household"), 0);
        assert_eq!(count(&conn, "household_member"), 0);
    }

    /// Reaches the SQL layer that `orphan_member_rejected` no longer gets to. Goes red if a
    /// regression leaves `foreign_keys` OFF on the connection `open` hands out, or if a
    /// future insert path bypasses `insert_rows`.
    #[test]
    fn foreign_key_rejects_absent_parent() {
        let conn = open(":memory:").unwrap();
        let err = conn
            .execute(
                "INSERT INTO household_member (id, household_id, display_name) VALUES (?1, ?2, ?3)",
                params!["m", "absent", "M"],
            )
            .unwrap_err();
        assert!(matches!(
            err,
            rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation
        ));
        assert_eq!(count(&conn, "household_member"), 0);
    }

    /// Pins the schema's `ON DELETE CASCADE`, which no test reached before.
    #[test]
    fn deleting_a_household_cascades_to_its_members() {
        let mut conn = open(":memory:").unwrap();
        let members = [member("m1", "h"), member("m2", "h")];
        insert_household(&mut conn, &household("h"), &members).unwrap();
        conn.execute("DELETE FROM household WHERE id = ?1", params!["h"])
            .unwrap();
        assert_eq!(count(&conn, "household"), 0);
        assert_eq!(count(&conn, "household_member"), 0);
    }

    #[test]
    fn rollback_on_later_row_failure() {
        let mut conn = open(":memory:").unwrap();
        let dup = [member("m1", "h"), member("m1", "h")];
        assert!(insert_household(&mut conn, &household("h"), &dup).is_err());
        assert_eq!(count(&conn, "household"), 0);
        assert_eq!(count(&conn, "household_member"), 0);
    }

    #[test]
    fn insert_round_trip() {
        let mut conn = open(":memory:").unwrap();
        let members = [member("m1", "h"), member("m2", "h")];
        insert_household(&mut conn, &household("h"), &members).unwrap();
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "household_member"), 2);
    }

    #[test]
    fn reopen_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut conn = open(&path).unwrap();
            insert_household(&mut conn, &household("h"), &[member("m", "h")]).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 1);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "household_member"), 1);
        let on: i32 = conn
            .pragma_query_value(None, "foreign_keys", |r| r.get(0))
            .unwrap();
        assert_eq!(on, 1);
    }

    fn name_of(conn: &Connection, id: &str) -> Option<String> {
        conn.query_row("SELECT name FROM household WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap()
    }

    #[test]
    fn mismatched_member_rejected() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[]).unwrap();
        let err = insert_household(&mut conn, &household("h2"), &[member("m", "h1")]).unwrap_err();
        assert!(matches!(err, StorageError::MemberHouseholdMismatch { .. }));
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "household_member"), 0);
    }

    #[test]
    fn load_household_empty_is_none() {
        let conn = open(":memory:").unwrap();
        assert_eq!(load_household(&conn).unwrap(), None);
    }

    /// Expected-to-pass: a unit test for the new loader, not a falsifiable pin on the bug it
    /// was added for. What regressed is the bridge command in `rust/src/api/household.rs`,
    /// which is pinned by `rename_reads_back_the_household_it_named` in that crate.
    #[test]
    fn load_by_id_returns_that_household_not_the_oldest() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[member("m1", "h1")]).unwrap();
        insert_household(&mut conn, &household("h2"), &[member("m2", "h2")]).unwrap();
        let rec = load_household_by_id(&conn, &HouseholdId::new("h2").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(rec.household.id.as_str(), "h2");
        assert_eq!(rec.members, vec![member("m2", "h2")]);
        // `load_household` would have answered h1 for the same database.
        assert_eq!(
            load_household(&conn)
                .unwrap()
                .unwrap()
                .household
                .id
                .as_str(),
            "h1"
        );
        assert_eq!(
            load_household_by_id(&conn, &HouseholdId::new("absent").unwrap()).unwrap(),
            None
        );
    }

    #[test]
    fn load_household_scopes_members() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[member("m1", "h1")]).unwrap();
        insert_household(&mut conn, &household("h2"), &[member("m2", "h2")]).unwrap();
        let rec = load_household(&conn).unwrap().unwrap();
        assert_eq!(rec.household.id.as_str(), "h1");
        assert_eq!(rec.members, vec![member("m1", "h1")]);
    }

    #[test]
    fn ensure_creates_once_then_reuses() {
        let mut conn = open(":memory:").unwrap();
        let first = ensure_household(&mut conn, &household("h1"), &member("m1", "h1")).unwrap();
        let second = ensure_household(&mut conn, &household("h2"), &member("m2", "h2")).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.household.id.as_str(), "h1");
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "household_member"), 1);
    }

    #[test]
    fn ensure_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        let first = {
            let mut conn = open(&path).unwrap();
            ensure_household(&mut conn, &household("h1"), &member("m1", "h1")).unwrap()
        };
        let mut conn = open(&path).unwrap();
        let again = ensure_household(&mut conn, &household("h2"), &member("m2", "h2")).unwrap();
        assert_eq!(first, again);
        assert_eq!(count(&conn, "household"), 1);
    }

    #[test]
    fn rename_touches_only_the_named_household() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[]).unwrap();
        insert_household(&mut conn, &household("h2"), &[]).unwrap();
        rename_household(&conn, &HouseholdId::new("h1").unwrap(), Some("  Casa  ")).unwrap();
        assert_eq!(name_of(&conn, "h1"), Some("Casa".into()));
        assert_eq!(name_of(&conn, "h2"), Some("Home".into()));
        rename_household(&conn, &HouseholdId::new("h1").unwrap(), Some("   ")).unwrap();
        assert_eq!(name_of(&conn, "h1"), None);
        rename_household(&conn, &HouseholdId::new("h1").unwrap(), None).unwrap();
        assert_eq!(name_of(&conn, "h1"), None);
    }

    #[test]
    fn rename_unknown_household_is_an_error() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[]).unwrap();
        let err =
            rename_household(&conn, &HouseholdId::new("nope").unwrap(), Some("x")).unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)));
        assert_eq!(name_of(&conn, "h1"), Some("Home".into()));
    }
}
