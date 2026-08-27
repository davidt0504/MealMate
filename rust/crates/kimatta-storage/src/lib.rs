//! Rust-owned SQLite (PRD v3 §12): foreign keys on, explicit migrations, transactions.
#![forbid(unsafe_code)]

use std::path::Path;

use household_core::{Household, HouseholdMember};
use rusqlite::{params, Connection};
use rusqlite_migration::{Migrations, M};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Migration(#[from] rusqlite_migration::Error),
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

/// Inserts the household and the given member rows atomically; any failure rolls back everything.
/// Callers own the parent/child pairing — a member whose `household_id` names a *different*
/// existing household is inserted successfully, because the foreign key only tests existence.
pub fn insert_household(
    conn: &mut Connection,
    household: &Household,
    members: &[HouseholdMember],
) -> Result<(), StorageError> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO household (id, name) VALUES (?1, ?2)",
        params![household.id.as_str(), household.name],
    )?;
    for m in members {
        tx.execute(
            "INSERT INTO household_member (id, household_id, display_name) VALUES (?1, ?2, ?3)",
            params![m.id.as_str(), m.household_id.as_str(), m.display_name],
        )?;
    }
    tx.commit()?;
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
        let err = insert_household(&mut conn, &household("h"), &[member("m", "other")]);
        assert!(err.is_err());
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
}
