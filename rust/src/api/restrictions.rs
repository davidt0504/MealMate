use kimatta_storage::{
    Connection, HouseholdId, HouseholdRestrictions, Restriction, RestrictionKind,
};

use crate::api::error::KimattaError;

/// `kind` is a `RestrictionKind` token (`peanuts`, `tree_nuts`, …) rather than an
/// enum-of-enum, for the reason `UnitDto` records: the vocabulary MVP-009 may extend must not
/// double the bridge surface. The price is that Dart's label switch has no compile-time
/// exhaustiveness; the cross-check test in `bridge_native_test.dart` buys it back, so a kind
/// added here without a Dart label fails a test rather than rendering `tree_nuts` to a user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestrictionDto {
    Known { kind: String },
    Other { text: String },
}

/// The known vocabulary, so the UI's checkbox list has one source and cannot drift from the
/// domain.
pub fn known_restriction_kinds() -> Vec<String> {
    RestrictionKind::ALL
        .iter()
        .map(|kind| kind.as_str().to_owned())
        .collect()
}

pub fn load_restrictions(household_id: String) -> Result<Vec<RestrictionDto>, KimattaError> {
    crate::db::with(|conn| load_in(conn, &household_id))
}

/// Replaces the household's whole restriction set and returns what was stored — trimmed and
/// de-duplicated — mirroring `save_planning_cycle`'s return-what-was-stored contract, so the
/// UI shows persisted truth rather than typed input. An empty list is legal and clears the set.
pub fn save_restrictions(
    household_id: String,
    restrictions: Vec<RestrictionDto>,
) -> Result<Vec<RestrictionDto>, KimattaError> {
    crate::db::with(|conn| save_in(conn, &household_id, &restrictions))
}

fn to_domain(dto: &RestrictionDto) -> Result<Restriction, KimattaError> {
    match dto {
        RestrictionDto::Known { kind } => Ok(Restriction::Known(RestrictionKind::parse(kind)?)),
        RestrictionDto::Other { text } => Ok(Restriction::other(text.as_str())?),
    }
}

/// Shared with `recipe.rs`, where a conflict names the restriction it was matched against.
pub(crate) fn from_domain(restriction: &Restriction) -> RestrictionDto {
    match restriction {
        Restriction::Known(kind) => RestrictionDto::Known {
            kind: kind.as_str().to_owned(),
        },
        Restriction::Other(text) => RestrictionDto::Other { text: text.clone() },
    }
}

fn to_dtos(set: &HouseholdRestrictions) -> Vec<RestrictionDto> {
    set.restrictions().iter().map(from_domain).collect()
}

/// Split out from the commands for the same reason `rename_in` is: they can be tested with
/// more than one household present without installing the process-wide connection.
fn load_in(conn: &Connection, household_id: &str) -> Result<Vec<RestrictionDto>, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    Ok(to_dtos(&kimatta_storage::load_restrictions(conn, &id)?))
}

pub(crate) fn save_in(
    conn: &mut Connection,
    household_id: &str,
    restrictions: &[RestrictionDto],
) -> Result<Vec<RestrictionDto>, KimattaError> {
    let id = HouseholdId::new(household_id)?;
    // Validated before storage is touched, as `planning.rs`'s `save_in` is: an invalid request
    // cannot reach a write, so a rejected save leaves the stored set exactly as it was.
    let set = HouseholdRestrictions::new(
        restrictions
            .iter()
            .map(to_domain)
            .collect::<Result<Vec<_>, KimattaError>>()?,
    );
    kimatta_storage::save_restrictions(conn, &id, &set)?;
    Ok(to_dtos(&set))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kimatta_storage::{Household, HouseholdMember, MemberId};

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

    fn open_seeded(ids: &[&str]) -> Connection {
        let mut conn = kimatta_storage::open(":memory:").unwrap();
        for id in ids {
            seed(&mut conn, id);
        }
        conn
    }

    fn known(kind: &str) -> RestrictionDto {
        RestrictionDto::Known {
            kind: kind.to_owned(),
        }
    }

    fn other(text: &str) -> RestrictionDto {
        RestrictionDto::Other {
            text: text.to_owned(),
        }
    }

    fn rows(conn: &Connection) -> u32 {
        conn.query_row("SELECT COUNT(*) FROM household_restriction", [], |r| {
            r.get(0)
        })
        .unwrap()
    }

    #[test]
    fn a_mixed_set_round_trips_in_order() {
        let mut conn = open_seeded(&["h"]);
        let sent = vec![known("peanuts"), other("nightshades"), known("tree_nuts")];
        assert_eq!(save_in(&mut conn, "h", &sent).unwrap(), sent);
        assert_eq!(load_in(&conn, "h").unwrap(), sent);
    }

    /// The return-what-was-stored contract `save_planning_cycle` set: the UI shows persisted
    /// truth rather than typed input, so trimming and de-duplication are visible immediately.
    #[test]
    fn saving_returns_the_trimmed_deduplicated_set() {
        let mut conn = open_seeded(&["h"]);
        let stored = save_in(
            &mut conn,
            "h",
            &[
                known("dairy"),
                other("  nightshades  "),
                known("dairy"),
                other("nightshades"),
            ],
        )
        .unwrap();
        assert_eq!(stored, vec![known("dairy"), other("nightshades")]);
        assert_eq!(load_in(&conn, "h").unwrap(), stored);
    }

    #[test]
    fn an_unknown_kind_token_is_rejected_and_writes_nothing() {
        let mut conn = open_seeded(&["h"]);
        save_in(&mut conn, "h", &[known("peanuts")]).unwrap();
        let err = save_in(&mut conn, "h", &[known("nightshades")]).unwrap_err();
        assert!(matches!(err, KimattaError::Restriction { .. }), "{err:?}");
        // Rejected before the delete-and-replace, so the stored set is untouched.
        assert_eq!(load_in(&conn, "h").unwrap(), vec![known("peanuts")]);
    }

    #[test]
    fn blank_other_text_is_rejected_and_writes_nothing() {
        let mut conn = open_seeded(&["h"]);
        for blank in ["", "   "] {
            let err = save_in(&mut conn, "h", &[known("soy"), other(blank)]).unwrap_err();
            assert!(
                matches!(err, KimattaError::Restriction { .. }),
                "{blank:?} gave {err:?}"
            );
        }
        assert_eq!(rows(&conn), 0);
    }

    /// Hard-coded rather than mapped from `RestrictionKind::ALL`: a test that mirrors the
    /// production constant cannot catch an addition to it, and this list is what the UI's
    /// checkbox list is built from.
    #[test]
    fn known_restriction_kinds_are_the_eleven_domain_tokens() {
        assert_eq!(
            known_restriction_kinds(),
            vec![
                "peanuts",
                "tree_nuts",
                "dairy",
                "eggs",
                "gluten",
                "soy",
                "fish",
                "shellfish",
                "sesame",
                "vegetarian",
                "vegan",
            ]
        );
    }

    #[test]
    fn commands_are_scoped_to_the_named_household() {
        let mut conn = open_seeded(&["h1", "h2"]);
        save_in(&mut conn, "h1", &[known("peanuts")]).unwrap();
        save_in(&mut conn, "h2", &[other("nightshades")]).unwrap();
        assert_eq!(load_in(&conn, "h1").unwrap(), vec![known("peanuts")]);
        assert_eq!(load_in(&conn, "h2").unwrap(), vec![other("nightshades")]);
    }

    #[test]
    fn an_absent_household_is_rejected() {
        let mut conn = open_seeded(&["h"]);
        let err = save_in(&mut conn, "nope", &[known("eggs")]).unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
        assert_eq!(rows(&conn), 0);
    }

    /// The read arm of the test above, at the layer the UI actually calls: an absent household
    /// must reach the caller as an error, not as the empty list that the warning surface would
    /// render as "no restrictions". Storage is what rejects it; this pins that the rejection is
    /// not swallowed on the way out.
    #[test]
    fn loading_for_an_absent_household_is_rejected() {
        let mut conn = open_seeded(&["h"]);
        save_in(&mut conn, "h", &[known("peanuts")]).unwrap();
        let err = load_in(&conn, "nope").unwrap_err();
        assert!(matches!(err, KimattaError::Storage { .. }), "{err:?}");
    }
}
