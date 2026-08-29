use kimatta_storage::{
    Connection, Household, HouseholdId, HouseholdMember, HouseholdRecord, MemberId,
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
    kimatta_storage::rename_household(conn, id, name)?;
    // By id rather than `load_household`'s oldest-row read: this record is what the UI
    // stores and shows after a save, so it has to be the household that was renamed.
    kimatta_storage::load_household_by_id(conn, id)?.ok_or_else(|| KimattaError::Storage {
        // Unreachable today: `rename_household` already returns `NoSuchHousehold` when the
        // UPDATE changes no rows, so an absent id never gets here. The window is open only
        // because no transaction spans the UPDATE and this read — kept as an error rather
        // than an unwrap so a second connection could not turn it into a panic.
        message: "household vanished after rename".into(),
    })
}

fn to_dto(record: HouseholdRecord) -> HouseholdDto {
    HouseholdDto {
        id: record.household.id.as_str().to_owned(),
        name: record.household.name,
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
