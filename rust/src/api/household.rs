use kimatta_storage::rusqlite::TransactionBehavior;
use kimatta_storage::{
    Connection, Household, HouseholdId, HouseholdMember, HouseholdRecord, MemberId, StorageError,
};
use uuid::Uuid;

use crate::api::error::KimattaError;

pub struct MemberDto {
    pub id: String,
    pub display_name: String,
}

pub struct HouseholdDto {
    pub id: String,
    pub name: Option<String>,
    pub members: Vec<MemberDto>,
    /// Whether first run has been completed. Flutter's first-run gate reads this and nothing
    /// else, so "have we welcomed this user" is a Rust-owned durable fact (invariant 17).
    pub onboarded: bool,
}

/// Returns the local household, creating an anonymous one-person household on the first
/// call (invariants 1 and 8). Later calls and later launches return the same ids.
pub fn bootstrap_household() -> Result<HouseholdDto, KimattaError> {
    let candidate = Household {
        id: HouseholdId::new(Uuid::new_v4().to_string())?,
        name: None,
    };
    let member = HouseholdMember {
        id: MemberId::new(Uuid::new_v4().to_string())?,
        household_id: candidate.id.clone(),
        // Fixed default; member profiles are MVP-006's.
        display_name: "Me".to_owned(),
    };
    let record = crate::db::with(|conn| {
        Ok(kimatta_storage::ensure_household(
            conn, &candidate, &member,
        )?)
    })?;
    Ok(to_dto(record))
}

/// Renames exactly `household_id` (blank clears the name) and returns the updated household.
pub fn rename_household(
    household_id: String,
    name: Option<String>,
) -> Result<HouseholdDto, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    let record = crate::db::with(|conn| rename_in(conn, &id, name.as_deref()))?;
    Ok(to_dto(record))
}

/// The rename itself, against a caller-supplied connection. Split out so it can be tested
/// with more than one household present without installing the process-wide connection,
/// which `db.rs`'s `with_no_connection_is_not_open` needs to stay uninstalled.
fn rename_in(
    conn: &mut Connection,
    id: &HouseholdId,
    name: Option<&str>,
) -> Result<HouseholdRecord, KimattaError> {
    // One IMMEDIATE transaction around the UPDATE and the read-back, as `save_recipe_in`
    // uses (AC-2): without it a read-back that failed after the write had committed would
    // report a rename that actually happened as a failure.
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StorageError::from)?;
    kimatta_storage::rename_household(&tx, id, name)?;
    // By id rather than `load_household`'s oldest-row read: this record is what the UI
    // stores and shows after a save, so it has to be the household that was renamed.
    let record =
        kimatta_storage::load_household_by_id(&tx, id)?.ok_or_else(|| KimattaError::Storage {
            // Unreachable: `rename_household` already returns `NoSuchHousehold` when the
            // UPDATE changes no rows, so an absent id never gets here, and the transaction
            // closes the window a second connection could have used. Kept as an error
            // rather than an unwrap so it could never become a panic.
            message: "household vanished after rename".into(),
        })?;
    tx.commit().map_err(StorageError::from)?;
    Ok(record)
}

/// Marks the local household onboarded, and returns it. Idempotent, so the button cannot fail
/// merely because it was already pressed.
pub fn complete_onboarding(household_id: String) -> Result<HouseholdDto, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    let record = crate::db::with(|conn| complete_in(conn, &id))?;
    Ok(to_dto(record))
}

/// Split out from the command for the same reason `rename_in` is: it can be tested with more
/// than one household present without installing the process-wide connection, which
/// `db.rs`'s `with_no_connection_is_not_open` needs to stay uninstalled.
fn complete_in(conn: &mut Connection, id: &HouseholdId) -> Result<HouseholdRecord, KimattaError> {
    // Wrapped for the same reason `rename_in` is (AC-2).
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StorageError::from)?;
    kimatta_storage::mark_onboarded(&tx, id)?;
    let record =
        kimatta_storage::load_household_by_id(&tx, id)?.ok_or_else(|| KimattaError::Storage {
            // Unreachable for the same reason `rename_in`'s twin is: `mark_onboarded`
            // already returns `NoSuchHousehold` when the UPDATE changes no rows.
            message: "household vanished after completing onboarding".into(),
        })?;
    tx.commit().map_err(StorageError::from)?;
    Ok(record)
}

fn to_dto(record: HouseholdRecord) -> HouseholdDto {
    HouseholdDto {
        id: record.household.id.as_str().to_owned(),
        name: record.household.name,
        onboarded: record.onboarded,
        members: record
            .members
            .into_iter()
            .map(|m| MemberDto {
                id: m.id.as_str().to_owned(),
                display_name: m.display_name,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed(conn: &mut Connection, id: &str) {
        let h = Household {
            id: HouseholdId::new(id).unwrap(),
            name: None,
        };
        let m = HouseholdMember {
            id: MemberId::new(format!("m-{id}")).unwrap(),
            household_id: h.id.clone(),
            display_name: "Me".to_owned(),
        };
        kimatta_storage::insert_household(conn, &h, std::slice::from_ref(&m)).unwrap();
    }

    /// Fail-first for the L4 defect: with two households present, the pre-fix body read back
    /// whichever was oldest, so renaming `h2` returned `h1` under `h2`'s new name.
    #[test]
    fn rename_reads_back_the_household_it_named() {
        let mut conn = kimatta_storage::open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let id = HouseholdId::new("h2").unwrap();
        let record = rename_in(&mut conn, &id, Some(" Casa ")).unwrap();
        assert_eq!(record.household.id.as_str(), "h2");
        assert_eq!(record.household.name.as_deref(), Some("Casa"));
        assert_eq!(record.members.len(), 1);
        assert_eq!(record.members[0].id.as_str(), "m-h2");
    }

    /// Idempotent and scoped: the button cannot fail merely because it was pressed twice, and
    /// completing one household's first run must not complete another's.
    #[test]
    fn complete_onboarding_is_idempotent_and_scoped_to_the_named_id() {
        let mut conn = kimatta_storage::open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let id = HouseholdId::new("h2").unwrap();
        assert!(!onboarded_of(&conn, &id));
        let record = complete_in(&mut conn, &id).unwrap();
        assert_eq!(record.household.id.as_str(), "h2");
        assert!(record.onboarded);
        // A second tap is a no-op write, not an error.
        assert!(complete_in(&mut conn, &id).unwrap().onboarded);
        let other = HouseholdId::new("h1").unwrap();
        assert!(!onboarded_of(&conn, &other));
    }

    fn onboarded_of(conn: &Connection, id: &HouseholdId) -> bool {
        kimatta_storage::load_household_by_id(conn, id)
            .unwrap()
            .unwrap()
            .onboarded
    }

    /// The transaction has to close, not just open: a body that forgot `tx.commit()` would
    /// roll back on drop and lose the rename silently. `:memory:` cannot see that, so this
    /// one writes a file, drops the connection and reads it back.
    #[test]
    fn a_rename_is_committed_not_left_open() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("household.db");
        let mut conn = kimatta_storage::open(&path).unwrap();
        seed(&mut conn, "h1");
        let id = HouseholdId::new("h1").unwrap();
        rename_in(&mut conn, &id, Some("Casa")).unwrap();
        drop(conn);

        let reopened = kimatta_storage::open(&path).unwrap();
        let record = kimatta_storage::load_household_by_id(&reopened, &id)
            .unwrap()
            .unwrap();
        assert_eq!(record.household.name.as_deref(), Some("Casa"));
    }

    /// The same durability check for `complete_in`, with a second household present so a
    /// lost `WHERE` clause fails it too.
    #[test]
    fn an_onboarding_commit_survives_a_reopen_and_stays_scoped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("household.db");
        let mut conn = kimatta_storage::open(&path).unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let id = HouseholdId::new("h2").unwrap();
        complete_in(&mut conn, &id).unwrap();
        drop(conn);

        let reopened = kimatta_storage::open(&path).unwrap();
        assert!(onboarded_of(&reopened, &id));
        assert!(!onboarded_of(&reopened, &HouseholdId::new("h1").unwrap()));
    }

    #[test]
    fn rename_of_an_absent_id_is_no_such_household() {
        let mut conn = kimatta_storage::open(":memory:").unwrap();
        seed(&mut conn, "h1");
        let id = HouseholdId::new("absent").unwrap();
        // Short-circuits in storage, which is why the "household vanished" arm is unreachable.
        assert!(matches!(
            rename_in(&mut conn, &id, Some("x")).unwrap_err(),
            KimattaError::Storage { .. }
        ));
    }
}
