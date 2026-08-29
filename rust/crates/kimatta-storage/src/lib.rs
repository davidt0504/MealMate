//! Rust-owned SQLite (PRD v3 §12): foreign keys on, explicit migrations, transactions.
#![forbid(unsafe_code)]

use std::path::Path;

pub use food_domain::{
    format_civil_date, parse_civil_date, CivilDate, MealScope, MealSlot, PlanningCycle,
    PlanningError, DEFAULT_CYCLE_DAYS, MAX_CYCLE_DAYS, MIN_CYCLE_DAYS,
};
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
    #[error(transparent)]
    Planning(#[from] PlanningError),
    #[error("planning cycle for household {0} has no meal slots")]
    CorruptMealScope(String),
}

// Two consts: `M` has drop glue, so an inline `&[M::up(..)]` argument is not promoted
// (E0716); a `&[..]` tail in a const initializer is lifetime-extended.
const MIGRATION_ARRAY: &[M] = &[
    M::up(
        "CREATE TABLE household (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT
    ) STRICT;
    CREATE TABLE household_member (
        id TEXT PRIMARY KEY NOT NULL,
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        display_name TEXT NOT NULL
    ) STRICT;",
    ),
    M::up(
        "CREATE TABLE planning_cycle (
        household_id TEXT PRIMARY KEY NOT NULL
            REFERENCES household(id) ON DELETE CASCADE,
        anchor_date TEXT NOT NULL
            CHECK (anchor_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
        length_days INTEGER NOT NULL CHECK (length_days BETWEEN 1 AND 31)
    ) STRICT;
    CREATE TABLE planning_meal_slot (
        household_id TEXT NOT NULL
            REFERENCES planning_cycle(household_id) ON DELETE CASCADE,
        slot TEXT NOT NULL,
        PRIMARY KEY (household_id, slot)
    ) STRICT;",
    ),
];
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

/// Errors with [`StorageError::NoSuchHousehold`] when `id` names no household, so an absent
/// parent is reported the way `rename_household` already reports it rather than surfacing as
/// a raw foreign-key failure.
fn require_household(conn: &Connection, id: &HouseholdId) -> Result<(), StorageError> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM household WHERE id = ?1)",
        params![id.as_str()],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(StorageError::NoSuchHousehold(id.as_str().to_owned()));
    }
    Ok(())
}

/// Writes the cycle row and replaces its whole slot set. The caller supplies the
/// transaction, so a partial scope is never observable.
fn write_planning_cycle(conn: &Connection, cycle: &PlanningCycle) -> Result<(), StorageError> {
    let id = cycle.household_id().as_str();
    conn.execute(
        "INSERT INTO planning_cycle (household_id, anchor_date, length_days)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(household_id) DO UPDATE SET
             anchor_date = excluded.anchor_date,
             length_days = excluded.length_days",
        params![id, format_civil_date(cycle.anchor()), cycle.length_days()],
    )?;
    conn.execute(
        "DELETE FROM planning_meal_slot WHERE household_id = ?1",
        params![id],
    )?;
    for slot in cycle.scope().slots() {
        conn.execute(
            "INSERT INTO planning_meal_slot (household_id, slot) VALUES (?1, ?2)",
            params![id, slot.as_str()],
        )?;
    }
    Ok(())
}

/// Returns the household's planning cycle, creating `default_cycle` if none exists.
/// Select and insert share one IMMEDIATE transaction, mirroring `ensure_household`, so
/// concurrent callers cannot both create. `default_cycle` is ignored when one exists.
pub fn ensure_planning_cycle(
    conn: &mut Connection,
    default_cycle: &PlanningCycle,
) -> Result<PlanningCycle, StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let id = default_cycle.household_id();
    require_household(&tx, id)?;
    let cycle = match load_planning_cycle(&tx, id)? {
        Some(existing) => existing,
        None => {
            write_planning_cycle(&tx, default_cycle)?;
            default_cycle.clone()
        }
    };
    tx.commit()?;
    Ok(cycle)
}

/// Replaces the household's cycle and its whole slot set in one transaction, so a partial
/// scope can never be observed. IMMEDIATE because it reads (`require_household`) before it
/// writes, as its two read-then-write siblings do: the write lock is taken at `BEGIN` rather
/// than upgraded from a shared one part-way through.
pub fn save_planning_cycle(
    conn: &mut Connection,
    cycle: &PlanningCycle,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, cycle.household_id())?;
    write_planning_cycle(&tx, cycle)?;
    tx.commit()?;
    Ok(())
}

/// The cycle for exactly `id`, or `None`. Slots are returned in `MealSlot` canonical order,
/// not insertion order, so a round-trip is order-stable. Every stored value is put back
/// through `PlanningCycle::new`, so a row set that violates an invariant surfaces as an
/// error rather than producing an invalid value.
pub fn load_planning_cycle(
    conn: &Connection,
    id: &HouseholdId,
) -> Result<Option<PlanningCycle>, StorageError> {
    let row = conn
        .query_row(
            "SELECT anchor_date, length_days FROM planning_cycle WHERE household_id = ?1",
            params![id.as_str()],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)),
        )
        .optional()?;
    let Some((anchor, length_days)) = row else {
        return Ok(None);
    };
    let mut stmt = conn.prepare("SELECT slot FROM planning_meal_slot WHERE household_id = ?1")?;
    let raw = stmt
        .query_map(params![id.as_str()], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    // Checked before `MealScope::new`, which also rejects an empty set: the guard order is
    // the only thing that keeps "no rows persisted" (corruption) distinct from "the caller
    // passed nothing" (bad input), and so the only thing that makes this variant reachable.
    if raw.is_empty() {
        return Err(StorageError::CorruptMealScope(id.as_str().to_owned()));
    }
    let slots = raw
        .iter()
        .map(|s| MealSlot::parse(s))
        .collect::<Result<Vec<_>, PlanningError>>()?;
    Ok(Some(PlanningCycle::new(
        id.clone(),
        parse_civil_date(&anchor)?,
        length_days,
        MealScope::new(slots)?,
    )?))
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

    fn cycle(household_id: &str, anchor: &str, days: u32, scope: &[MealSlot]) -> PlanningCycle {
        PlanningCycle::new(
            HouseholdId::new(household_id).unwrap(),
            parse_civil_date(anchor).unwrap(),
            days,
            MealScope::new(scope.iter().copied()).unwrap(),
        )
        .unwrap()
    }

    /// A household to hang a planning cycle on, so cycle tests never trip the household FK
    /// when that is not what they are testing.
    fn seed(conn: &mut Connection, id: &str) {
        insert_household(conn, &household(id), &[member(&format!("m-{id}"), id)]).unwrap();
    }

    #[test]
    fn planning_cycle_round_trips() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let c = cycle(
            "h",
            "2026-08-29",
            14,
            &[MealSlot::Breakfast, MealSlot::Dinner],
        );
        save_planning_cycle(&mut conn, &c).unwrap();
        let id = HouseholdId::new("h").unwrap();
        assert_eq!(load_planning_cycle(&conn, &id).unwrap(), Some(c));
    }

    #[test]
    fn ensure_creates_the_default_cycle_once_then_reuses_it() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let anchor = parse_civil_date("2026-08-29").unwrap();
        let default = PlanningCycle::default_for(HouseholdId::new("h").unwrap(), anchor).unwrap();
        let first = ensure_planning_cycle(&mut conn, &default).unwrap();
        let other = cycle("h", "2020-01-01", 3, &[MealSlot::Lunch]);
        let second = ensure_planning_cycle(&mut conn, &other).unwrap();
        assert_eq!(first, default);
        assert_eq!(second, first);
        assert_eq!(count(&conn, "planning_cycle"), 1);
        assert_eq!(count(&conn, "planning_meal_slot"), 1);
    }

    #[test]
    fn ensure_cycle_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        let anchor = parse_civil_date("2026-08-29").unwrap();
        let default = PlanningCycle::default_for(HouseholdId::new("h").unwrap(), anchor).unwrap();
        let first = {
            let mut conn = open(&path).unwrap();
            seed(&mut conn, "h");
            ensure_planning_cycle(&mut conn, &default).unwrap()
        };
        let mut conn = open(&path).unwrap();
        let other = cycle("h", "2020-01-01", 3, &[MealSlot::Lunch]);
        assert_eq!(ensure_planning_cycle(&mut conn, &other).unwrap(), first);
        assert_eq!(count(&conn, "planning_cycle"), 1);
    }

    /// Pins that a save replaces the whole slot set rather than merging into it.
    #[test]
    fn saving_replaces_the_whole_slot_set() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_planning_cycle(
            &mut conn,
            &cycle(
                "h",
                "2026-08-29",
                7,
                &[MealSlot::Breakfast, MealSlot::Lunch, MealSlot::Dinner],
            ),
        )
        .unwrap();
        assert_eq!(count(&conn, "planning_meal_slot"), 3);
        let dinner = cycle("h", "2026-08-29", 7, &[MealSlot::Dinner]);
        save_planning_cycle(&mut conn, &dinner).unwrap();
        let id = HouseholdId::new("h").unwrap();
        assert_eq!(
            load_planning_cycle(&conn, &id).unwrap().unwrap().scope(),
            &MealScope::dinner_only()
        );
        assert_eq!(count(&conn, "planning_meal_slot"), 1);
    }

    #[test]
    fn slots_load_in_canonical_order_not_insertion_order() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let c = cycle(
            "h",
            "2026-08-29",
            7,
            &[MealSlot::Dinner, MealSlot::Breakfast],
        );
        save_planning_cycle(&mut conn, &c).unwrap();
        // Re-insert the rows in reverse so the read cannot be passing by luck of rowid order.
        conn.execute("DELETE FROM planning_meal_slot", []).unwrap();
        for slot in ["dinner", "breakfast"] {
            conn.execute(
                "INSERT INTO planning_meal_slot (household_id, slot) VALUES (?1, ?2)",
                params!["h", slot],
            )
            .unwrap();
        }
        let id = HouseholdId::new("h").unwrap();
        assert_eq!(
            load_planning_cycle(&conn, &id)
                .unwrap()
                .unwrap()
                .scope()
                .slots(),
            [MealSlot::Breakfast, MealSlot::Dinner]
        );
    }

    #[test]
    fn cycle_for_an_absent_household_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        let c = cycle("ghost", "2026-08-29", 7, &[MealSlot::Dinner]);
        assert!(matches!(
            save_planning_cycle(&mut conn, &c).unwrap_err(),
            StorageError::NoSuchHousehold(_)
        ));
        assert!(matches!(
            ensure_planning_cycle(&mut conn, &c).unwrap_err(),
            StorageError::NoSuchHousehold(_)
        ));
        assert_eq!(count(&conn, "planning_cycle"), 0);
        assert_eq!(count(&conn, "planning_meal_slot"), 0);
    }

    #[test]
    fn load_of_an_absent_cycle_is_none() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let id = HouseholdId::new("h").unwrap();
        assert_eq!(load_planning_cycle(&conn, &id).unwrap(), None);
    }

    /// `open()` always migrates to latest, so v1 is unreachable through the public API once v2
    /// exists — hence the raw connection and `to_version`. Test-level stand-in for the
    /// on-device v1→v2 migration (PRD §12: tested from representative prior versions).
    #[test]
    fn an_existing_v1_database_migrates_to_v2_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 1).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 1);
            insert_household(
                &mut raw,
                &household("h"),
                &[member("m1", "h"), member("m2", "h")],
            )
            .unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 2);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "household_member"), 2);
        assert_eq!(count(&conn, "planning_cycle"), 0);
    }

    /// Pins the two-level cascade: household → planning_cycle → planning_meal_slot.
    #[test]
    fn deleting_a_household_cascades_to_its_cycle_and_slots() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_planning_cycle(
            &mut conn,
            &cycle(
                "h",
                "2026-08-29",
                7,
                &[MealSlot::Breakfast, MealSlot::Lunch, MealSlot::Dinner],
            ),
        )
        .unwrap();
        conn.execute("DELETE FROM household WHERE id = ?1", params!["h"])
            .unwrap();
        for table in [
            "household",
            "household_member",
            "planning_cycle",
            "planning_meal_slot",
        ] {
            assert_eq!(count(&conn, table), 0, "{table} must be empty");
        }
    }

    #[test]
    fn a_cycle_with_no_slot_rows_is_reported_as_corrupt() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let c = cycle("h", "2026-08-29", 7, &[MealSlot::Dinner]);
        save_planning_cycle(&mut conn, &c).unwrap();
        conn.execute("DELETE FROM planning_meal_slot", []).unwrap();
        let id = HouseholdId::new("h").unwrap();
        let err = load_planning_cycle(&conn, &id).unwrap_err();
        assert!(
            matches!(&err, StorageError::CorruptMealScope(h) if h == "h"),
            "got {err:?}"
        );
    }

    /// Reaches the SQL guard the type system short-circuits, as
    /// `foreign_key_rejects_absent_parent` does for the household FK. Written against
    /// `MIN_CYCLE_DAYS`/`MAX_CYCLE_DAYS` rather than literals: migration 2's CHECK is immutable
    /// on shipped databases, so widening the constants has to go red here — at the layer that
    /// would have to migrate — and not in `food-domain` alone.
    #[test]
    fn the_length_check_constraint_matches_the_domain_bounds() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        // Saturating: a plain `- 1` would wrap to `u32::MAX` if the floor ever moved to 0, and
        // that still violates the CHECK, so this half would stay green. The good half below is
        // what detects that case — it would try to insert 0 and be rejected.
        for bad in [MIN_CYCLE_DAYS.saturating_sub(1), MAX_CYCLE_DAYS + 1] {
            let err = conn
                .execute(
                    "INSERT INTO planning_cycle (household_id, anchor_date, length_days)
                     VALUES (?1, ?2, ?3)",
                    params!["h", "2026-08-29", bad],
                )
                .unwrap_err();
            assert!(
                matches!(err, rusqlite::Error::SqliteFailure(e, _)
                    if e.code == rusqlite::ErrorCode::ConstraintViolation),
                "length_days = {bad} must violate the CHECK, got {err:?}"
            );
        }
        assert_eq!(count(&conn, "planning_cycle"), 0);
        // The half that detects divergence: widening the constants without migrating leaves the
        // CHECK rejecting a length the domain has started accepting.
        for good in [MIN_CYCLE_DAYS, MAX_CYCLE_DAYS] {
            conn.execute(
                "INSERT INTO planning_cycle (household_id, anchor_date, length_days)
                 VALUES (?1, ?2, ?3)",
                params!["h", "2026-08-29", good],
            )
            .unwrap_or_else(|e| panic!("length_days = {good} must insert cleanly, got {e:?}"));
            // `household_id` is the primary key, so the next iteration needs the row gone.
            conn.execute("DELETE FROM planning_cycle", []).unwrap();
        }
    }

    #[test]
    fn the_anchor_glob_rejects_a_direct_non_iso_write() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = conn
            .execute(
                "INSERT INTO planning_cycle (household_id, anchor_date, length_days)
                 VALUES (?1, ?2, ?3)",
                params!["h", "29/08/2026", 7],
            )
            .unwrap_err();
        assert!(
            matches!(err, rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "planning_cycle"), 0);
    }

    /// The GLOB is a *shape* check, so `2026-13-45` passes it. This is the one read-side
    /// `PlanningCycle::new` failure the schema cannot prevent.
    #[test]
    fn a_glob_passing_but_impossible_anchor_is_rejected_on_read() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        conn.execute(
            "INSERT INTO planning_cycle (household_id, anchor_date, length_days)
             VALUES (?1, ?2, ?3)",
            params!["h", "2026-13-45", 7],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO planning_meal_slot (household_id, slot) VALUES (?1, ?2)",
            params!["h", "dinner"],
        )
        .unwrap();
        let id = HouseholdId::new("h").unwrap();
        let err = load_planning_cycle(&conn, &id).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::Planning(PlanningError::InvalidDate(raw)) if raw == "2026-13-45"
            ),
            "got {err:?}"
        );
    }

    /// The third read-side guard, alongside `a_cycle_with_no_slot_rows_is_reported_as_corrupt`
    /// and `a_glob_passing_but_impossible_anchor_is_rejected_on_read`. `slot` carries no CHECK,
    /// so unlike both columns beside it nothing stops the row landing in the first place.
    #[test]
    fn an_unknown_slot_row_is_rejected_on_read() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let c = cycle("h", "2026-08-29", 7, &[MealSlot::Dinner]);
        save_planning_cycle(&mut conn, &c).unwrap();
        conn.execute(
            "INSERT INTO planning_meal_slot (household_id, slot) VALUES (?1, ?2)",
            params!["h", "supper"],
        )
        .unwrap();
        let id = HouseholdId::new("h").unwrap();
        let err = load_planning_cycle(&conn, &id).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::Planning(PlanningError::UnknownMealSlot(raw)) if raw == "supper"
            ),
            "got {err:?}"
        );
    }

    /// Pins the lock ordering, not just the outcome. With another connection holding RESERVED
    /// and no busy handler, an IMMEDIATE transaction is refused at `BEGIN`, before
    /// `require_household` runs; a DEFERRED one begins, reads, and reports the absent household
    /// instead — so the household named here is deliberately one that was never seeded.
    #[test]
    fn save_takes_the_write_lock_before_it_reads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        let mut conn = open(&path).unwrap();
        // rusqlite sets a 5s busy timeout on every connection; without this the contended
        // BEGIN would stall for it rather than answering.
        conn.busy_timeout(std::time::Duration::ZERO).unwrap();
        // Opened directly rather than through `open`, so the second handle does not re-run
        // `to_latest` while this test is about locking.
        let other = Connection::open(&path).unwrap();
        other.execute_batch("BEGIN IMMEDIATE").unwrap();

        let c = cycle("absent", "2026-08-29", 7, &[MealSlot::Dinner]);
        let err = save_planning_cycle(&mut conn, &c).unwrap_err();
        assert!(
            matches!(err, StorageError::Sqlite(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::DatabaseBusy),
            "the BEGIN must be refused before the household read, got {err:?}"
        );
    }

    #[test]
    fn planning_reads_and_writes_are_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let c1 = cycle("h1", "2026-08-29", 7, &[MealSlot::Dinner]);
        let c2 = cycle(
            "h2",
            "2026-01-05",
            14,
            &[MealSlot::Breakfast, MealSlot::Lunch],
        );
        save_planning_cycle(&mut conn, &c1).unwrap();
        save_planning_cycle(&mut conn, &c2).unwrap();
        let id1 = HouseholdId::new("h1").unwrap();
        let id2 = HouseholdId::new("h2").unwrap();
        assert_eq!(load_planning_cycle(&conn, &id1).unwrap(), Some(c1.clone()));
        assert_eq!(load_planning_cycle(&conn, &id2).unwrap(), Some(c2.clone()));
        // Saving one leaves the other byte-for-byte alone.
        let c1b = cycle("h1", "2027-03-01", 3, &[MealSlot::Lunch]);
        save_planning_cycle(&mut conn, &c1b).unwrap();
        assert_eq!(load_planning_cycle(&conn, &id2).unwrap(), Some(c2));
        assert_eq!(load_planning_cycle(&conn, &id1).unwrap(), Some(c1b));
    }

    #[test]
    fn empty_db_migrates_to_v2() {
        let conn = open(":memory:").unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 2);
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
        assert_eq!(schema_version(&conn).unwrap(), 2);
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
