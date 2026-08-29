//! Rust-owned SQLite (PRD v3 §12): foreign keys on, explicit migrations, transactions.
#![forbid(unsafe_code)]

use std::path::Path;

pub use food_domain::{
    format_civil_date, parse_civil_date, CivilDate, CustomIngredient, CustomIngredientId,
    HouseholdRestrictions, Ingredient, IngredientId, IngredientLine, IngredientRef, MealScope,
    MealSlot, MemberPreference, MemberPreferences, PlanningCycle, PlanningError, PreferenceError,
    ProvenanceKind, Quantity, QuantityRange, Rational, Recipe, RecipeError, RecipeId,
    RecipeProvenance, Restriction, RestrictionError, RestrictionKind, Sentiment, Unit, UnitKind,
    DEFAULT_CYCLE_DAYS, MAX_CYCLE_DAYS, MIN_CYCLE_DAYS,
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
    #[error(transparent)]
    Recipe(#[from] RecipeError),
    #[error("custom ingredient {ingredient} belongs to household {actual}, not {expected}")]
    CustomIngredientHouseholdMismatch {
        ingredient: String,
        expected: String,
        actual: String,
    },
    #[error("no recipe {recipe} in household {household}")]
    NoSuchRecipe { recipe: String, household: String },
    #[error("recipe {0} has a line with both a catalog and a custom ingredient")]
    CorruptIngredientRef(String),
    #[error("recipe {0} has no provenance row")]
    CorruptProvenance(String),
    #[error("no ingredient {0}")]
    NoSuchIngredient(String),
    #[error(
        "recipe {recipe} line {position} has quantity_kind {kind:?} \
         with min ({min_numer:?}, {min_denom:?}) and max ({max_numer:?}, {max_denom:?})"
    )]
    CorruptQuantity {
        recipe: String,
        position: usize,
        kind: String,
        min_numer: Option<u32>,
        min_denom: Option<u32>,
        max_numer: Option<u32>,
        max_denom: Option<u32>,
    },
    #[error("recipe {recipe} line {position} has unit_kind {kind:?} with unit_text {text:?}")]
    CorruptUnit {
        recipe: String,
        position: usize,
        kind: String,
        text: Option<String>,
    },
    #[error("no custom ingredient {ingredient} in household {household}")]
    NoSuchCustomIngredient {
        ingredient: String,
        household: String,
    },
    #[error("no member with id {0}")]
    NoSuchMember(String),
    #[error(transparent)]
    Preference(#[from] PreferenceError),
    #[error(transparent)]
    Restriction(#[from] RestrictionError),
    #[error("household {household} restriction {position} has kind {kind:?} with text {text:?}")]
    CorruptRestriction {
        household: String,
        position: usize,
        kind: String,
        text: Option<String>,
    },
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
    // CHECKs only on stable invariants (servings ≥ 1, boolean, ref exclusivity); the
    // `kind`/`quantity_kind`/`unit_kind` vocabularies grow and are validated by parse on
    // read and write. The two ingredient FKs deliberately carry no `ON DELETE CASCADE`:
    // deleting an ingredient must not silently drop recipe lines (PRD §12), so the default
    // `NO ACTION` makes such a delete fail — and both are indexed so proving no child row
    // references the parent is a lookup, not a scan of every line of every recipe.
    // `ingredient` has no `household_id`: it is the global catalog; only `custom_ingredient`
    // is household-owned.
    M::up(
        "CREATE TABLE ingredient (
        id TEXT PRIMARY KEY NOT NULL,
        canonical_name TEXT NOT NULL,
        store_category TEXT
    ) STRICT;
    CREATE TABLE ingredient_alias (
        ingredient_id TEXT NOT NULL REFERENCES ingredient(id) ON DELETE CASCADE,
        alias TEXT NOT NULL,
        PRIMARY KEY (ingredient_id, alias)
    ) STRICT;
    CREATE TABLE custom_ingredient (
        id TEXT PRIMARY KEY NOT NULL,
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        name TEXT NOT NULL,
        store_category TEXT
    ) STRICT;
    CREATE INDEX custom_ingredient_household ON custom_ingredient(household_id);
    CREATE TABLE recipe (
        id TEXT PRIMARY KEY NOT NULL,
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        title TEXT NOT NULL,
        servings INTEGER CHECK (servings IS NULL OR servings >= 1),
        instructions TEXT NOT NULL
    ) STRICT;
    CREATE INDEX recipe_household ON recipe(household_id);
    CREATE TABLE recipe_provenance (
        recipe_id TEXT PRIMARY KEY NOT NULL REFERENCES recipe(id) ON DELETE CASCADE,
        kind TEXT NOT NULL,
        source_url TEXT,
        source_name TEXT,
        source_author TEXT
    ) STRICT;
    CREATE TABLE recipe_ingredient_line (
        recipe_id TEXT NOT NULL REFERENCES recipe(id) ON DELETE CASCADE,
        position INTEGER NOT NULL,
        original_text TEXT NOT NULL,
        name TEXT NOT NULL,
        ingredient_id TEXT REFERENCES ingredient(id),
        custom_ingredient_id TEXT REFERENCES custom_ingredient(id),
        quantity_kind TEXT NOT NULL,
        min_numer INTEGER,
        min_denom INTEGER,
        max_numer INTEGER,
        max_denom INTEGER,
        unit_kind TEXT NOT NULL,
        unit_text TEXT,
        preparation TEXT,
        optional INTEGER NOT NULL CHECK (optional IN (0, 1)),
        PRIMARY KEY (recipe_id, position),
        CHECK (ingredient_id IS NULL OR custom_ingredient_id IS NULL)
    ) STRICT;
    CREATE INDEX recipe_ingredient_line_ingredient
        ON recipe_ingredient_line(ingredient_id);
    CREATE INDEX recipe_ingredient_line_custom_ingredient
        ON recipe_ingredient_line(custom_ingredient_id);",
    ),
    // `position` exists for the same reason it does on `recipe_ingredient_line`: both sets are
    // replaced whole, so a key of `(parent_id, position)` is what makes a replacement
    // order-preserving and what gives a corrupt row a stable coordinate to report. No CHECK on
    // `kind` or `sentiment`: those vocabularies grow (MVP-009 may extend the restriction list),
    // so they are validated by parse on read and write, as `unit_kind` is. That leaves
    // `kind = 'other'` with a NULL `text` storable — hence the reject-on-read below rather than
    // coercing such a row into something displayable.
    M::up(
        "ALTER TABLE household ADD COLUMN onboarded INTEGER NOT NULL DEFAULT 0
        CHECK (onboarded IN (0, 1));
    CREATE TABLE household_restriction (
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        position INTEGER NOT NULL,
        kind TEXT NOT NULL,
        text TEXT,
        PRIMARY KEY (household_id, position)
    ) STRICT;
    CREATE TABLE member_food_preference (
        member_id TEXT NOT NULL REFERENCES household_member(id) ON DELETE CASCADE,
        position INTEGER NOT NULL,
        sentiment TEXT NOT NULL,
        subject TEXT NOT NULL,
        PRIMARY KEY (member_id, position)
    ) STRICT;",
    ),
    // Archive marker (MVP-008 decision gate, owner 2026-08-28): "delete" sets this, never
    // removes the row, so every planner/occurrence reference stays resolvable (PRD §12).
    // Same civil-date GLOB as `planning_cycle.anchor_date`; NULL = active.
    M::up(
        "ALTER TABLE recipe ADD COLUMN archived_at TEXT
        CHECK (archived_at IS NULL
            OR archived_at GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]');",
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
    /// Whether first run has been completed. App/UX state, deliberately not part of
    /// `household_core::Household`, which carries kernel identity only.
    pub onboarded: bool,
}

/// The single local household, or `None` before bootstrap. Takes the oldest row so a
/// second household — which nothing should create — can never displace the first.
pub fn load_household(conn: &Connection) -> Result<Option<HouseholdRecord>, StorageError> {
    let row = conn
        .query_row(
            "SELECT id, name, onboarded FROM household ORDER BY rowid LIMIT 1",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, bool>(2)?,
                ))
            },
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
            "SELECT id, name, onboarded FROM household WHERE id = ?1",
            params![id.as_str()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, bool>(2)?,
                ))
            },
        )
        .optional()?;
    with_members(conn, row)
}

/// Attaches the member rows to a household row, so both loaders scope members identically.
fn with_members(
    conn: &Connection,
    row: Option<(String, Option<String>, bool)>,
) -> Result<Option<HouseholdRecord>, StorageError> {
    let Some((id, name, onboarded)) = row else {
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
    Ok(Some(HouseholdRecord {
        household,
        members,
        onboarded,
    }))
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
                // Matches the column default: a household that was just created has not
                // been through first run.
                onboarded: false,
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

/// Marks exactly this household onboarded. Idempotent: a second call is a no-op write, so a
/// user who taps the button twice does not get an error. An absent id is an error, as in
/// [`rename_household`].
pub fn mark_onboarded(conn: &Connection, id: &HouseholdId) -> Result<(), StorageError> {
    let changed = conn.execute(
        "UPDATE household SET onboarded = 1 WHERE id = ?1",
        params![id.as_str()],
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

/// Replaces the household's whole restriction set in one IMMEDIATE transaction, so a partial
/// set is never observable — the same shape as [`save_planning_cycle`]. IMMEDIATE because it
/// reads (`require_household`) before it writes. An empty set is legal and clears the set.
pub fn save_restrictions(
    conn: &mut Connection,
    id: &HouseholdId,
    set: &HouseholdRestrictions,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, id)?;
    tx.execute(
        "DELETE FROM household_restriction WHERE household_id = ?1",
        params![id.as_str()],
    )?;
    for (position, restriction) in set.restrictions().iter().enumerate() {
        let (kind, text) = match restriction {
            Restriction::Known(kind) => (kind.as_str(), None),
            Restriction::Other(text) => ("other", Some(text.as_str())),
        };
        tx.execute(
            "INSERT INTO household_restriction (household_id, position, kind, text)
             VALUES (?1, ?2, ?3, ?4)",
            // `u32`, as `length_days` is: rusqlite binds no `usize`, and the set is bounded
            // by what a person will type into a checkbox list.
            params![id.as_str(), position as u32, kind, text],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// The household's restrictions in stored order, or an empty set — including for a household
/// that does not exist, which has no restrictions in exactly the same sense.
pub fn load_restrictions(
    conn: &Connection,
    id: &HouseholdId,
) -> Result<HouseholdRestrictions, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT position, kind, text FROM household_restriction
         WHERE household_id = ?1 ORDER BY position",
    )?;
    let rows = stmt
        .query_map(params![id.as_str()], |r| {
            Ok((
                r.get::<_, u32>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let restrictions = rows
        .into_iter()
        .map(|(position, kind, text)| restriction_from_row(id, position as usize, kind, text))
        .collect::<Result<Vec<_>, StorageError>>()?;
    Ok(HouseholdRestrictions::new(restrictions))
}

/// One stored restriction row, back through the domain constructors. A row whose `kind`/`text`
/// pair does not describe a restriction is reported with its columns and never coerced into
/// something displayable (invariant 10): a warning surface that quietly drops what it could
/// not read is under-warning, the one direction this feature must not fail in. Unlike
/// `CorruptUnit`, the vocabulary failure lands here too — `kind` is both the discriminant and
/// the token, so an unrecognised value is a fault in the same column either way. A blank
/// `other` text stays the domain's own error: that row is shaped correctly and it is the value
/// that cannot be a restriction.
fn restriction_from_row(
    household: &HouseholdId,
    position: usize,
    kind: String,
    text: Option<String>,
) -> Result<Restriction, StorageError> {
    let parsed = match (kind.as_str(), text.as_deref()) {
        ("other", Some(text)) => Some(Restriction::other(text)?),
        (known, None) => RestrictionKind::parse(known).map(Restriction::Known).ok(),
        _ => None,
    };
    parsed.ok_or(StorageError::CorruptRestriction {
        household: household.as_str().to_owned(),
        position,
        kind,
        text,
    })
}

/// Errors when `member` names no member, or names one belonging to another household. Both
/// loaders and the writer share it, so a caller holding a foreign member id is rejected the
/// same way whichever way it arrives — and, in the writer, before any row is touched.
fn require_member(
    conn: &Connection,
    household: &HouseholdId,
    member: &MemberId,
) -> Result<(), StorageError> {
    let owner: Option<String> = conn
        .query_row(
            "SELECT household_id FROM household_member WHERE id = ?1",
            params![member.as_str()],
            |r| r.get(0),
        )
        .optional()?;
    let Some(owner) = owner else {
        return Err(StorageError::NoSuchMember(member.as_str().to_owned()));
    };
    if owner != household.as_str() {
        return Err(StorageError::MemberHouseholdMismatch {
            member: member.as_str().to_owned(),
            expected: household.as_str().to_owned(),
            actual: owner,
        });
    }
    Ok(())
}

/// Replaces this member's whole preference set in one IMMEDIATE transaction, so a partial set
/// is never observable — the same shape as [`save_planning_cycle`] and [`save_restrictions`].
/// IMMEDIATE because it reads (`require_member`) before it writes: the write lock is taken at
/// `BEGIN` rather than upgraded from a shared one part-way through. `household` is checked
/// against the member's own household first, so a caller holding a foreign member id is
/// rejected before any write with the same `MemberHouseholdMismatch` the insert path raises.
///
/// This card writes no preferences from the app: `MVP-009`/`MVP-023` add the consumer, and
/// AC-4 asks only that the storage layer key them by member.
pub fn save_member_preferences(
    conn: &mut Connection,
    household: &HouseholdId,
    member: &MemberId,
    set: &MemberPreferences,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_member(&tx, household, member)?;
    tx.execute(
        "DELETE FROM member_food_preference WHERE member_id = ?1",
        params![member.as_str()],
    )?;
    for (position, preference) in set.preferences().iter().enumerate() {
        tx.execute(
            "INSERT INTO member_food_preference (member_id, position, sentiment, subject)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                member.as_str(),
                position as u32,
                preference.sentiment().as_str(),
                preference.subject(),
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// This member's preferences in stored order, or an empty set. Every stored value goes back
/// through the domain constructors, so a row that does not describe a preference is reported
/// rather than coerced.
pub fn load_member_preferences(
    conn: &Connection,
    household: &HouseholdId,
    member: &MemberId,
) -> Result<MemberPreferences, StorageError> {
    require_member(conn, household, member)?;
    let mut stmt = conn.prepare(
        "SELECT sentiment, subject FROM member_food_preference
         WHERE member_id = ?1 ORDER BY position",
    )?;
    let rows = stmt
        .query_map(params![member.as_str()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let preferences = rows
        .into_iter()
        .map(|(sentiment, subject)| {
            Ok(MemberPreference::new(
                Sentiment::parse(&sentiment)?,
                subject,
            )?)
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    Ok(MemberPreferences::new(preferences))
}

/// Inserts or replaces a catalog ingredient and its whole alias set (MVP-011 seeding path).
/// Aliases arrive sorted and deduplicated from `Ingredient::new`, so the alias PK cannot trip.
pub fn upsert_ingredient(
    conn: &mut Connection,
    ingredient: &Ingredient,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let id = ingredient.id().as_str();
    tx.execute(
        "INSERT INTO ingredient (id, canonical_name, store_category) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET
             canonical_name = excluded.canonical_name,
             store_category = excluded.store_category",
        params![id, ingredient.canonical_name(), ingredient.store_category()],
    )?;
    tx.execute(
        "DELETE FROM ingredient_alias WHERE ingredient_id = ?1",
        params![id],
    )?;
    for alias in ingredient.aliases() {
        tx.execute(
            "INSERT INTO ingredient_alias (ingredient_id, alias) VALUES (?1, ?2)",
            params![id, alias],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// The catalog ingredient `id`, or `None`. Rows go back through `Ingredient::new`, so a
/// corrupt row surfaces as `StorageError::Recipe`.
pub fn load_ingredient(
    conn: &Connection,
    id: &IngredientId,
) -> Result<Option<Ingredient>, StorageError> {
    let row = conn
        .query_row(
            "SELECT canonical_name, store_category FROM ingredient WHERE id = ?1",
            params![id.as_str()],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
        )
        .optional()?;
    let Some((canonical_name, store_category)) = row else {
        return Ok(None);
    };
    let mut stmt =
        conn.prepare("SELECT alias FROM ingredient_alias WHERE ingredient_id = ?1 ORDER BY alias")?;
    let aliases = stmt
        .query_map(params![id.as_str()], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(Ingredient::new(
        id.clone(),
        canonical_name,
        aliases,
        store_category,
    )?))
}

/// Inserts or replaces a household's custom ingredient. `NoSuchHousehold` for an absent
/// parent. As `save_recipe` does for a recipe id, the current owner is probed inside the
/// transaction: an id already owned by another household is `NoSuchCustomIngredient` naming
/// only the *requesting* household, never a raw UNIQUE-constraint error confirming the id
/// exists somewhere.
pub fn upsert_custom_ingredient(
    conn: &mut Connection,
    item: &CustomIngredient,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, item.household_id())?;
    let id = item.id().as_str();
    let owner: Option<String> = tx
        .query_row(
            "SELECT household_id FROM custom_ingredient WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()?;
    if owner.is_some_and(|o| o != item.household_id().as_str()) {
        return Err(StorageError::NoSuchCustomIngredient {
            ingredient: id.to_owned(),
            household: item.household_id().as_str().to_owned(),
        });
    }
    // `household_id` is not in the DO UPDATE list: the probe above already established the
    // row is this household's, so an upsert can never move a row between households.
    tx.execute(
        "INSERT INTO custom_ingredient (id, household_id, name, store_category)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             store_category = excluded.store_category",
        params![
            id,
            item.household_id().as_str(),
            item.name(),
            item.store_category(),
        ],
    )?;
    tx.commit()?;
    Ok(())
}

/// Every custom ingredient of exactly `household`, ordered by name then id.
pub fn list_custom_ingredients(
    conn: &Connection,
    household: &HouseholdId,
) -> Result<Vec<CustomIngredient>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, store_category FROM custom_ingredient
         WHERE household_id = ?1 ORDER BY name, id",
    )?;
    let items = stmt
        .query_map(params![household.as_str()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?
        .map(|row| {
            let (id, name, store_category) = row?;
            Ok(CustomIngredient::new(
                CustomIngredientId::new(id)?,
                household.clone(),
                name,
                store_category,
            )?)
        })
        .collect();
    items
}

/// `(id, title)` of a recipe, for listing without loading its lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeSummary {
    pub id: RecipeId,
    pub title: String,
}

/// A recipe as stored, with its archive marker. `archived_at` is library/UX lifecycle state,
/// deliberately not part of `food_domain::Recipe` — as `HouseholdRecord::onboarded` is kept
/// out of `Household` — so `Recipe::new` and `save_recipe` never see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeRecord {
    pub recipe: Recipe,
    /// The civil date the household archived it, or `None` while it is in the library.
    pub archived_at: Option<CivilDate>,
}

/// Which side of the archive marker `list_recipes` returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeListing {
    Active,
    Archived,
}

/// Every referenced ingredient must exist, and every custom one must belong to the recipe's
/// household. All probes run before any write, so a rejected save leaves no partial row.
fn check_line_refs(conn: &Connection, recipe: &Recipe) -> Result<(), StorageError> {
    for line in recipe.lines() {
        match line.ingredient() {
            None => {}
            Some(IngredientRef::Catalog(id)) => {
                let exists: bool = conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM ingredient WHERE id = ?1)",
                    params![id.as_str()],
                    |r| r.get(0),
                )?;
                if !exists {
                    return Err(StorageError::NoSuchIngredient(id.as_str().to_owned()));
                }
            }
            Some(IngredientRef::Custom(id)) => {
                let owner: Option<String> = conn
                    .query_row(
                        "SELECT household_id FROM custom_ingredient WHERE id = ?1",
                        params![id.as_str()],
                        |r| r.get(0),
                    )
                    .optional()?;
                match owner {
                    None => return Err(StorageError::NoSuchIngredient(id.as_str().to_owned())),
                    Some(actual) if actual != recipe.household_id().as_str() => {
                        return Err(StorageError::CustomIngredientHouseholdMismatch {
                            ingredient: id.as_str().to_owned(),
                            expected: recipe.household_id().as_str().to_owned(),
                            actual,
                        });
                    }
                    Some(_) => {}
                }
            }
        }
    }
    Ok(())
}

/// Inserts or wholly replaces the recipe, its provenance row and its whole line set in one
/// IMMEDIATE transaction. Every custom-ingredient reference must belong to
/// `recipe.household_id()`; a mismatch is rejected before any row is written. An existing
/// recipe id owned by another household is `NoSuchRecipe`, never hijacked.
pub fn save_recipe(conn: &mut Connection, recipe: &Recipe) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, recipe.household_id())?;
    check_line_refs(&tx, recipe)?;
    let id = recipe.id().as_str();
    let owner: Option<String> = tx
        .query_row(
            "SELECT household_id FROM recipe WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()?;
    if owner.is_some_and(|o| o != recipe.household_id().as_str()) {
        return Err(StorageError::NoSuchRecipe {
            recipe: id.to_owned(),
            household: recipe.household_id().as_str().to_owned(),
        });
    }
    tx.execute(
        "INSERT INTO recipe (id, household_id, title, servings, instructions)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
             title = excluded.title,
             servings = excluded.servings,
             instructions = excluded.instructions",
        params![
            id,
            recipe.household_id().as_str(),
            recipe.title(),
            recipe.servings(),
            recipe.instructions(),
        ],
    )?;
    let p = recipe.provenance();
    tx.execute(
        "INSERT INTO recipe_provenance (recipe_id, kind, source_url, source_name, source_author)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(recipe_id) DO UPDATE SET
             kind = excluded.kind,
             source_url = excluded.source_url,
             source_name = excluded.source_name,
             source_author = excluded.source_author",
        params![
            id,
            p.kind().as_str(),
            p.source_url(),
            p.source_name(),
            p.source_author(),
        ],
    )?;
    tx.execute(
        "DELETE FROM recipe_ingredient_line WHERE recipe_id = ?1",
        params![id],
    )?;
    for (position, line) in recipe.lines().iter().enumerate() {
        let (ingredient_id, custom_ingredient_id) = match line.ingredient() {
            None => (None, None),
            Some(IngredientRef::Catalog(i)) => (Some(i.as_str()), None),
            Some(IngredientRef::Custom(c)) => (None, Some(c.as_str())),
        };
        // `Unknown` → all NULL; `Exact` → min only; `Range` → both bounds.
        let (min, max) = match line.quantity() {
            Quantity::Unknown => (None, None),
            Quantity::Exact(r) => (Some(r), None),
            Quantity::Range(range) => (Some(range.min()), Some(range.max())),
        };
        let (unit_kind, unit_text) = match line.unit() {
            Unit::None => ("none", None),
            Unit::Known(kind) => ("known", Some(kind.as_str())),
            Unit::Other(text) => ("other", Some(text.as_str())),
        };
        tx.execute(
            "INSERT INTO recipe_ingredient_line
             (recipe_id, position, original_text, name, ingredient_id, custom_ingredient_id,
              quantity_kind, min_numer, min_denom, max_numer, max_denom,
              unit_kind, unit_text, preparation, optional)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                id,
                position as u32,
                line.original_text(),
                line.name(),
                ingredient_id,
                custom_ingredient_id,
                line.quantity().kind_str(),
                min.map(Rational::numer),
                min.map(Rational::denom),
                max.map(Rational::numer),
                max.map(Rational::denom),
                unit_kind,
                unit_text,
                line.preparation(),
                line.optional(),
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// One stored line, before it goes back through the domain constructors.
struct LineRow {
    original_text: String,
    name: String,
    ingredient_id: Option<String>,
    custom_ingredient_id: Option<String>,
    quantity_kind: String,
    min: (Option<u32>, Option<u32>),
    max: (Option<u32>, Option<u32>),
    unit_kind: String,
    unit_text: Option<String>,
    preparation: Option<String>,
    optional: bool,
}

/// A bound is present only when both of its columns are; a half-present bound is treated as
/// absent and then caught by the kind check below as `CorruptQuantity`, which names the
/// columns — the kind is not the fault when a valid kind meets inconsistent bounds.
fn bound(numer: Option<u32>, denom: Option<u32>) -> Result<Option<Rational>, RecipeError> {
    match (numer, denom) {
        (Some(n), Some(d)) => Ok(Some(Rational::new(n, d)?)),
        _ => Ok(None),
    }
}

/// `position` is the line's index within the recipe. `load_recipe` reads lines `ORDER BY
/// position` over a set `save_recipe` rewrites whole with gap-free `enumerate()` positions,
/// so the enumeration index equals the stored column; a future change that can leave gaps
/// must select `position` instead of counting.
fn line_from_row(
    recipe_id: &str,
    position: usize,
    row: LineRow,
) -> Result<IngredientLine, StorageError> {
    let ingredient = match (row.ingredient_id, row.custom_ingredient_id) {
        (None, None) => None,
        (Some(i), None) => Some(IngredientRef::Catalog(IngredientId::new(i)?)),
        (None, Some(c)) => Some(IngredientRef::Custom(CustomIngredientId::new(c)?)),
        // Unreachable under the CHECK, but never resolved by picking one.
        (Some(_), Some(_)) => {
            return Err(StorageError::CorruptIngredientRef(recipe_id.to_owned()));
        }
    };
    let min = bound(row.min.0, row.min.1)?;
    let max = bound(row.max.0, row.max.1)?;
    // Split, so each error names the column actually at fault: a kind from the known
    // vocabulary meeting bounds that do not match it is a bounds fault, and reporting it as
    // `unknown quantity kind "exact"` sends a maintainer to grep a vocabulary that is fine.
    let quantity = match (row.quantity_kind.as_str(), min, max) {
        ("unknown", None, None) => Quantity::Unknown,
        ("exact", Some(r), None) => Quantity::Exact(r),
        ("range", Some(lo), Some(hi)) => Quantity::Range(QuantityRange::new(lo, hi)?),
        ("unknown" | "exact" | "range", _, _) => {
            return Err(StorageError::CorruptQuantity {
                recipe: recipe_id.to_owned(),
                position,
                kind: row.quantity_kind,
                min_numer: row.min.0,
                min_denom: row.min.1,
                max_numer: row.max.0,
                max_denom: row.max.1,
            });
        }
        (kind, _, _) => return Err(RecipeError::UnknownQuantityKind(kind.to_owned()).into()),
    };
    // Not split, unlike quantity above: `RecipeError::UnknownUnit` is `UnitKind::parse`'s
    // error and carries a unit string, so feeding it a `unit_kind` discriminant would put two
    // vocabularies in one variant. Both faults report the columns instead.
    let unit = match (row.unit_kind.as_str(), row.unit_text.as_deref()) {
        ("none", None) => Unit::None,
        ("known", Some(text)) => Unit::Known(UnitKind::parse(text)?),
        ("other", Some(text)) => Unit::Other(text.to_owned()),
        (kind, text) => {
            return Err(StorageError::CorruptUnit {
                recipe: recipe_id.to_owned(),
                position,
                kind: kind.to_owned(),
                text: text.map(str::to_owned),
            });
        }
    };
    Ok(IngredientLine::new(
        row.original_text,
        row.name,
        ingredient,
        quantity,
        unit,
        row.preparation,
        row.optional,
    )?)
}

/// The recipe `id` **in `household`**, or `None` — another household's recipe is `None`,
/// never the row. Every stored value goes back through the domain constructors, so a row
/// that violates an invariant surfaces as `StorageError::Recipe`, never as an invalid value.
/// An archived recipe still loads (its `archived_at` says so): references to it must keep
/// resolving.
pub fn load_recipe(
    conn: &Connection,
    household: &HouseholdId,
    id: &RecipeId,
) -> Result<Option<RecipeRecord>, StorageError> {
    let row = conn
        .query_row(
            "SELECT title, servings, instructions, archived_at FROM recipe
             WHERE id = ?1 AND household_id = ?2",
            params![id.as_str(), household.as_str()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<u32>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()?;
    let Some((title, servings, instructions, archived_at)) = row else {
        return Ok(None);
    };
    let archived_at = archived_at.as_deref().map(parse_civil_date).transpose()?;
    let provenance = conn
        .query_row(
            "SELECT kind, source_url, source_name, source_author FROM recipe_provenance
             WHERE recipe_id = ?1",
            params![id.as_str()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .optional()?;
    let Some((kind, source_url, source_name, source_author)) = provenance else {
        return Err(StorageError::CorruptProvenance(id.as_str().to_owned()));
    };
    let provenance = RecipeProvenance::new(
        ProvenanceKind::parse(&kind)?,
        source_url,
        source_name,
        source_author,
    )?;
    let mut stmt = conn.prepare(
        "SELECT original_text, name, ingredient_id, custom_ingredient_id, quantity_kind,
                min_numer, min_denom, max_numer, max_denom, unit_kind, unit_text,
                preparation, optional
         FROM recipe_ingredient_line WHERE recipe_id = ?1 ORDER BY position",
    )?;
    let lines = stmt
        .query_map(params![id.as_str()], |r| {
            Ok(LineRow {
                original_text: r.get(0)?,
                name: r.get(1)?,
                ingredient_id: r.get(2)?,
                custom_ingredient_id: r.get(3)?,
                quantity_kind: r.get(4)?,
                min: (r.get(5)?, r.get(6)?),
                max: (r.get(7)?, r.get(8)?),
                unit_kind: r.get(9)?,
                unit_text: r.get(10)?,
                preparation: r.get(11)?,
                optional: r.get(12)?,
            })
        })?
        .enumerate()
        .map(|(position, row)| line_from_row(id.as_str(), position, row?))
        .collect::<Result<Vec<_>, StorageError>>()?;
    Ok(Some(RecipeRecord {
        recipe: Recipe::new(
            id.clone(),
            household.clone(),
            title,
            servings,
            instructions,
            lines,
            provenance,
        )?,
        archived_at,
    }))
}

/// `(id, title)` of every recipe in `household` on the requested side of the archive marker,
/// ordered by title then id.
pub fn list_recipes(
    conn: &Connection,
    household: &HouseholdId,
    listing: RecipeListing,
) -> Result<Vec<RecipeSummary>, StorageError> {
    let sql = match listing {
        RecipeListing::Active => {
            "SELECT id, title FROM recipe
             WHERE household_id = ?1 AND archived_at IS NULL ORDER BY title, id"
        }
        RecipeListing::Archived => {
            "SELECT id, title FROM recipe
             WHERE household_id = ?1 AND archived_at IS NOT NULL ORDER BY title, id"
        }
    };
    let mut stmt = conn.prepare(sql)?;
    let summaries = stmt
        .query_map(params![household.as_str()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .map(|row| {
            let (id, title) = row?;
            Ok(RecipeSummary {
                id: RecipeId::new(id)?,
                title,
            })
        })
        .collect();
    summaries
}

/// Sets `archived_at` to `at` on an active recipe. Idempotent: an already-archived recipe
/// keeps its first date. The row is never deleted (PRD §12), so `load_recipe` keeps
/// resolving it. Absent, or owned by another household, is `NoSuchRecipe`.
pub fn archive_recipe(
    conn: &mut Connection,
    household: &HouseholdId,
    id: &RecipeId,
    at: CivilDate,
) -> Result<(), StorageError> {
    set_archive_marker(
        conn,
        household,
        id,
        "UPDATE recipe SET archived_at = ?1
         WHERE id = ?2 AND household_id = ?3 AND archived_at IS NULL",
        Some(format_civil_date(at)),
    )
}

/// Clears `archived_at`. Idempotent on an active recipe; scoping as `archive_recipe`.
pub fn restore_recipe(
    conn: &mut Connection,
    household: &HouseholdId,
    id: &RecipeId,
) -> Result<(), StorageError> {
    set_archive_marker(
        conn,
        household,
        id,
        "UPDATE recipe SET archived_at = ?1
         WHERE id = ?2 AND household_id = ?3 AND archived_at IS NOT NULL",
        None,
    )
}

/// The guarded UPDATE matches only rows on the other side of the marker, so zero rows means
/// either "already there" (a no-op) or "no such recipe in this household" — the existence
/// probe tells them apart, inside the same IMMEDIATE transaction so nothing moves between.
fn set_archive_marker(
    conn: &mut Connection,
    household: &HouseholdId,
    id: &RecipeId,
    update: &str,
    marker: Option<String>,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let changed = tx.execute(update, params![marker, id.as_str(), household.as_str()])?;
    if changed == 0 {
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM recipe WHERE id = ?1 AND household_id = ?2)",
            params![id.as_str(), household.as_str()],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(StorageError::NoSuchRecipe {
                recipe: id.as_str().to_owned(),
                household: household.as_str().to_owned(),
            });
        }
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
    /// on-device v1→v2 migration (PRD §12: tested from representative prior versions); since
    /// v4 it proves v1→latest end to end.
    #[test]
    fn an_existing_v1_database_migrates_to_v5_without_losing_data() {
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
        assert_eq!(schema_version(&conn).unwrap(), 5);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "household_member"), 2);
        assert_eq!(count(&conn, "planning_cycle"), 0);
        assert_eq!(count(&conn, "recipe"), 0);
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
    fn empty_db_migrates_to_v5() {
        let conn = open(":memory:").unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 5);
    }

    // --- Step 4: schema v3 -----------------------------------------------------------------

    /// Test-level stand-in for the on-device v2→v3 migration, as the v1 test is for v1→v2.
    #[test]
    fn an_existing_v2_database_migrates_to_v5_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 2).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 2);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            save_planning_cycle(
                &mut raw,
                &cycle(
                    "h",
                    "2026-08-29",
                    7,
                    &[MealSlot::Breakfast, MealSlot::Dinner],
                ),
            )
            .unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 5);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "planning_cycle"), 1);
        assert_eq!(count(&conn, "planning_meal_slot"), 2);
        assert_eq!(count(&conn, "recipe"), 0);
    }

    // --- schema v4 -------------------------------------------------------------------------

    /// Test-level stand-in for the on-device v3→v4 migration, as the v1 and v2 tests are for
    /// theirs. A recipe is saved at v3 so the migration is proved not to disturb the tables
    /// migration 3 introduced, not merely the household one it alters.
    #[test]
    fn an_existing_v3_database_migrates_to_v5_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 3).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 3);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            save_recipe(&mut raw, &recipe("h", "r", vec![])).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 5);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "household_member"), 1);
        assert_eq!(count(&conn, "recipe"), 1);
        assert_eq!(count(&conn, "household_restriction"), 0);
        assert_eq!(count(&conn, "member_food_preference"), 0);
    }

    // --- schema v5 -------------------------------------------------------------------------

    /// Test-level stand-in for the on-device v4→v5 migration, in the pattern of the v3→v4
    /// test: a recipe saved at v4 must survive the `ALTER TABLE` that adds `archived_at`.
    #[test]
    fn an_existing_v4_database_migrates_to_v5_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 4).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 4);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            save_recipe(&mut raw, &recipe("h", "r", vec![])).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 5);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "recipe"), 1);
        assert_eq!(count(&conn, "household_restriction"), 0);
        assert_eq!(count(&conn, "member_food_preference"), 0);
    }

    /// No backfill: a recipe that existed before v5 is active, which is what NULL means.
    #[test]
    fn a_migrated_recipe_is_not_archived() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 4).unwrap();
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            save_recipe(&mut raw, &recipe("h", "r", vec![])).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(archived_at_raw(&conn, "r"), None);
        assert_eq!(active_ids(&conn, "h"), vec!["r"]);
    }

    /// The column default is what an already-shipped household reads after the upgrade: the
    /// migration adds no backfill `UPDATE`, so a v3 install reads `onboarded = 0` and is
    /// shown Welcome once — it has genuinely never been welcomed, and Welcome is skippable.
    /// This pins that default, not an exemption from it.
    #[test]
    fn a_migrated_household_is_not_onboarded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 3).unwrap();
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
        }
        let conn = open(&path).unwrap();
        let onboarded: bool = conn
            .query_row("SELECT onboarded FROM household WHERE id = 'h'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(!onboarded);
    }

    fn assert_constraint_violation(err: rusqlite::Error) {
        assert!(
            matches!(err, rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation),
            "expected a constraint violation, got {err:?}"
        );
    }

    /// A recipe row plus its provenance, inserted raw so the line-level constraints below
    /// are the only thing under test.
    fn raw_recipe(conn: &Connection, household: &str, id: &str) {
        conn.execute(
            "INSERT INTO recipe (id, household_id, title, servings, instructions)
             VALUES (?1, ?2, 'T', 2, '')",
            params![id, household],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO recipe_provenance (recipe_id, kind) VALUES (?1, 'authored')",
            params![id],
        )
        .unwrap();
    }

    const RAW_LINE: &str = "INSERT INTO recipe_ingredient_line
        (recipe_id, position, original_text, name, ingredient_id, custom_ingredient_id,
         quantity_kind, unit_kind, optional)
        VALUES (?1, ?2, 'x', 'x', ?3, ?4, 'unknown', 'none', 0)";

    #[test]
    fn foreign_keys_on_lines_reject_absent_ingredients() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_recipe(&conn, "h", "r");
        let err = conn
            .execute(RAW_LINE, params!["r", 0, "ghost", Option::<String>::None])
            .unwrap_err();
        assert_constraint_violation(err);
        let err = conn
            .execute(RAW_LINE, params!["r", 0, Option::<String>::None, "ghost"])
            .unwrap_err();
        assert_constraint_violation(err);
        assert_eq!(count(&conn, "recipe_ingredient_line"), 0);
    }

    #[test]
    fn a_line_with_both_refs_is_rejected_by_the_check() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_recipe(&conn, "h", "r");
        conn.execute(
            "INSERT INTO ingredient (id, canonical_name) VALUES ('i', 'flour')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO custom_ingredient (id, household_id, name) VALUES ('c', 'h', 'mix')",
            [],
        )
        .unwrap();
        // Both parents exist, so only the exclusivity CHECK can be what rejects this.
        let err = conn
            .execute(RAW_LINE, params!["r", 0, "i", "c"])
            .unwrap_err();
        assert_constraint_violation(err);
        conn.execute(RAW_LINE, params!["r", 0, "i", Option::<String>::None])
            .unwrap();
        conn.execute(RAW_LINE, params!["r", 1, Option::<String>::None, "c"])
            .unwrap();
        assert_eq!(count(&conn, "recipe_ingredient_line"), 2);
    }

    #[test]
    fn deleting_a_household_cascades_recipes_lines_provenance_and_custom_ingredients() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_recipe(&conn, "h", "r");
        conn.execute(
            "INSERT INTO custom_ingredient (id, household_id, name) VALUES ('c', 'h', 'mix')",
            [],
        )
        .unwrap();
        conn.execute(RAW_LINE, params!["r", 0, Option::<String>::None, "c"])
            .unwrap();
        raw_restriction(&conn, "h", "peanuts", None);
        for table in [
            "recipe",
            "recipe_provenance",
            "recipe_ingredient_line",
            "custom_ingredient",
            "household_restriction",
        ] {
            assert_eq!(count(&conn, table), 1, "{table} must be seeded");
        }
        conn.execute("DELETE FROM household WHERE id = 'h'", [])
            .unwrap();
        for table in [
            "household",
            "recipe",
            "recipe_provenance",
            "recipe_ingredient_line",
            "custom_ingredient",
            "household_restriction",
        ] {
            assert_eq!(count(&conn, table), 0, "{table} must be empty");
        }
    }

    /// The deliberate `NO ACTION`: deleting an ingredient must never silently drop recipe
    /// lines (PRD §12).
    #[test]
    fn deleting_a_catalog_ingredient_referenced_by_a_line_is_refused() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_recipe(&conn, "h", "r");
        conn.execute(
            "INSERT INTO ingredient (id, canonical_name) VALUES ('i', 'flour')",
            [],
        )
        .unwrap();
        conn.execute(RAW_LINE, params!["r", 0, "i", Option::<String>::None])
            .unwrap();
        let err = conn
            .execute("DELETE FROM ingredient WHERE id = 'i'", [])
            .unwrap_err();
        assert_constraint_violation(err);
        assert_eq!(count(&conn, "ingredient"), 1);
        assert_eq!(count(&conn, "recipe_ingredient_line"), 1);
    }

    #[test]
    fn zero_servings_is_rejected_by_the_check() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = conn
            .execute(
                "INSERT INTO recipe (id, household_id, title, servings, instructions)
                 VALUES ('r', 'h', 'T', 0, '')",
                [],
            )
            .unwrap_err();
        assert_constraint_violation(err);
        // NULL servings is the honest "unknown" and must still insert.
        conn.execute(
            "INSERT INTO recipe (id, household_id, title, servings, instructions)
             VALUES ('r', 'h', 'T', NULL, '')",
            [],
        )
        .unwrap();
        assert_eq!(count(&conn, "recipe"), 1);
    }

    #[test]
    fn migrations_validate() {
        MIGRATIONS.validate().unwrap();
    }

    // --- Step 5: ingredient and custom-ingredient repository -----------------------------

    fn ingredient(id: &str, name: &str, aliases: &[&str]) -> Ingredient {
        Ingredient::new(
            IngredientId::new(id).unwrap(),
            name,
            aliases.iter().map(|a| (*a).to_owned()).collect(),
            Some("aisle".to_owned()),
        )
        .unwrap()
    }

    fn custom(id: &str, household: &str, name: &str) -> CustomIngredient {
        CustomIngredient::new(
            CustomIngredientId::new(id).unwrap(),
            HouseholdId::new(household).unwrap(),
            name,
            None,
        )
        .unwrap()
    }

    #[test]
    fn ingredient_round_trips_with_aliases() {
        let mut conn = open(":memory:").unwrap();
        let i = ingredient("flour", "flour", &["plain flour", "AP flour"]);
        upsert_ingredient(&mut conn, &i).unwrap();
        let id = IngredientId::new("flour").unwrap();
        assert_eq!(load_ingredient(&conn, &id).unwrap(), Some(i));
        assert_eq!(count(&conn, "ingredient_alias"), 2);
        assert_eq!(
            load_ingredient(&conn, &IngredientId::new("absent").unwrap()).unwrap(),
            None
        );
    }

    #[test]
    fn upsert_replaces_the_alias_set() {
        let mut conn = open(":memory:").unwrap();
        upsert_ingredient(&mut conn, &ingredient("flour", "flour", &["a", "b", "c"])).unwrap();
        let replacement = ingredient("flour", "wheat flour", &["b"]);
        upsert_ingredient(&mut conn, &replacement).unwrap();
        let id = IngredientId::new("flour").unwrap();
        assert_eq!(load_ingredient(&conn, &id).unwrap(), Some(replacement));
        assert_eq!(count(&conn, "ingredient"), 1);
        assert_eq!(count(&conn, "ingredient_alias"), 1);
    }

    #[test]
    fn custom_ingredient_round_trips() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let c = custom("c", "h", "nana's mix");
        upsert_custom_ingredient(&mut conn, &c).unwrap();
        let id = HouseholdId::new("h").unwrap();
        assert_eq!(list_custom_ingredients(&conn, &id).unwrap(), vec![c]);
    }

    #[test]
    fn custom_ingredient_for_an_absent_household_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        let err = upsert_custom_ingredient(&mut conn, &custom("c", "ghost", "x")).unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(h) if h == "ghost"));
        assert_eq!(count(&conn, "custom_ingredient"), 0);
    }

    #[test]
    fn custom_ingredients_are_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let c1 = custom("c1", "h1", "zed");
        let c1b = custom("c1b", "h1", "alpha");
        let c2 = custom("c2", "h2", "beta");
        for c in [&c1, &c1b, &c2] {
            upsert_custom_ingredient(&mut conn, c).unwrap();
        }
        let h1 = HouseholdId::new("h1").unwrap();
        let h2 = HouseholdId::new("h2").unwrap();
        assert_eq!(list_custom_ingredients(&conn, &h1).unwrap(), vec![c1b, c1]);
        assert_eq!(list_custom_ingredients(&conn, &h2).unwrap(), vec![c2]);
        let h3 = HouseholdId::new("h3").unwrap();
        assert_eq!(list_custom_ingredients(&conn, &h3).unwrap(), vec![]);
    }

    // --- Step 6: recipe repository -------------------------------------------------------

    fn rat(numer: u32, denom: u32) -> Rational {
        Rational::new(numer, denom).unwrap()
    }

    fn line(original: &str, name: &str, quantity: Quantity, unit: Unit) -> IngredientLine {
        IngredientLine::new(original, name, None, quantity, unit, None, false).unwrap()
    }

    fn full_line(
        original: &str,
        name: &str,
        ingredient: Option<IngredientRef>,
        quantity: Quantity,
        unit: Unit,
        preparation: Option<&str>,
        optional: bool,
    ) -> IngredientLine {
        IngredientLine::new(
            original,
            name,
            ingredient,
            quantity,
            unit,
            preparation.map(str::to_owned),
            optional,
        )
        .unwrap()
    }

    fn authored() -> RecipeProvenance {
        RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap()
    }

    fn recipe(household: &str, id: &str, lines: Vec<IngredientLine>) -> Recipe {
        Recipe::new(
            RecipeId::new(id).unwrap(),
            HouseholdId::new(household).unwrap(),
            format!("Recipe {id}"),
            Some(4),
            "Cook it.",
            lines,
            authored(),
        )
        .unwrap()
    }

    /// AC-1's three-line fixture: a resolved catalog line with an exact fraction and a
    /// preparation, a resolved custom line with a range that is optional, and an unresolved
    /// line that knows nothing but its text.
    fn three_lines() -> Vec<IngredientLine> {
        vec![
            full_line(
                "1/2 cup flour, sifted",
                "flour",
                Some(IngredientRef::Catalog(IngredientId::new("flour").unwrap())),
                Quantity::Exact(rat(1, 2)),
                Unit::Known(UnitKind::Cup),
                Some("sifted"),
                false,
            ),
            full_line(
                "2-3 pieces nana's mix (optional)",
                "nana's mix",
                Some(IngredientRef::Custom(CustomIngredientId::new("c").unwrap())),
                Quantity::Range(QuantityRange::new(rat(2, 1), rat(3, 1)).unwrap()),
                Unit::Known(UnitKind::Piece),
                None,
                true,
            ),
            line(
                "a splash of something",
                "something",
                Quantity::Unknown,
                Unit::None,
            ),
        ]
    }

    /// Household `h` with the catalog and custom ingredients the fixture lines reference.
    fn seed_for_recipes(conn: &mut Connection, household: &str) {
        seed(conn, household);
        upsert_ingredient(conn, &ingredient("flour", "flour", &[])).unwrap();
        upsert_custom_ingredient(conn, &custom("c", household, "nana's mix")).unwrap();
    }

    fn load_record(conn: &Connection, household: &str, id: &str) -> Option<RecipeRecord> {
        load_recipe(
            conn,
            &HouseholdId::new(household).unwrap(),
            &RecipeId::new(id).unwrap(),
        )
        .unwrap()
    }

    fn load(conn: &Connection, household: &str, id: &str) -> Option<Recipe> {
        load_record(conn, household, id).map(|r| r.recipe)
    }

    fn ids(conn: &Connection, household: &str, listing: RecipeListing) -> Vec<String> {
        list_recipes(conn, &HouseholdId::new(household).unwrap(), listing)
            .unwrap()
            .into_iter()
            .map(|s| s.id.as_str().to_owned())
            .collect()
    }

    fn active_ids(conn: &Connection, household: &str) -> Vec<String> {
        ids(conn, household, RecipeListing::Active)
    }

    fn archived_ids(conn: &Connection, household: &str) -> Vec<String> {
        ids(conn, household, RecipeListing::Archived)
    }

    /// The column as stored, bypassing `load_recipe`, so the archive tests do not depend on
    /// the read path they are also proving.
    fn archived_at_raw(conn: &Connection, id: &str) -> Option<String> {
        conn.query_row(
            "SELECT archived_at FROM recipe WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn archive(
        conn: &mut Connection,
        household: &str,
        id: &str,
        at: &str,
    ) -> Result<(), StorageError> {
        archive_recipe(
            conn,
            &HouseholdId::new(household).unwrap(),
            &RecipeId::new(id).unwrap(),
            parse_civil_date(at).unwrap(),
        )
    }

    fn restore(conn: &mut Connection, household: &str, id: &str) -> Result<(), StorageError> {
        restore_recipe(
            conn,
            &HouseholdId::new(household).unwrap(),
            &RecipeId::new(id).unwrap(),
        )
    }

    #[test]
    fn recipe_round_trips_with_structured_and_original_lines() {
        let mut conn = open(":memory:").unwrap();
        seed_for_recipes(&mut conn, "h");
        let r = recipe("h", "r", three_lines());
        save_recipe(&mut conn, &r).unwrap();
        assert_eq!(load(&conn, "h", "r"), Some(r));
        assert_eq!(count(&conn, "recipe_ingredient_line"), 3);
    }

    #[test]
    fn save_replaces_the_whole_line_set() {
        let mut conn = open(":memory:").unwrap();
        seed_for_recipes(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", three_lines())).unwrap();
        assert_eq!(count(&conn, "recipe_ingredient_line"), 3);
        let one = recipe(
            "h",
            "r",
            vec![line("1 egg", "egg", Quantity::Exact(rat(1, 1)), Unit::None)],
        );
        save_recipe(&mut conn, &one).unwrap();
        assert_eq!(load(&conn, "h", "r"), Some(one));
        assert_eq!(count(&conn, "recipe_ingredient_line"), 1);
        assert_eq!(count(&conn, "recipe"), 1);
        assert_eq!(count(&conn, "recipe_provenance"), 1);
    }

    #[test]
    fn list_returns_the_household_summaries_in_title_order() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        for (id, title) in [
            ("r1", "Zucchini bake"),
            ("r2", "Apple pie"),
            ("r3", "Apple pie"),
        ] {
            let r = Recipe::new(
                RecipeId::new(id).unwrap(),
                HouseholdId::new("h").unwrap(),
                title,
                None,
                "",
                vec![],
                authored(),
            )
            .unwrap();
            save_recipe(&mut conn, &r).unwrap();
        }
        let summaries = list_recipes(
            &conn,
            &HouseholdId::new("h").unwrap(),
            RecipeListing::Active,
        )
        .unwrap();
        assert_eq!(
            summaries,
            vec![
                RecipeSummary {
                    id: RecipeId::new("r2").unwrap(),
                    title: "Apple pie".to_owned(),
                },
                RecipeSummary {
                    id: RecipeId::new("r3").unwrap(),
                    title: "Apple pie".to_owned(),
                },
                RecipeSummary {
                    id: RecipeId::new("r1").unwrap(),
                    title: "Zucchini bake".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn recipe_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        let r = recipe("h", "r", three_lines());
        {
            let mut conn = open(&path).unwrap();
            seed_for_recipes(&mut conn, "h");
            save_recipe(&mut conn, &r).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(load(&conn, "h", "r"), Some(r));
    }

    #[test]
    fn unknown_quantity_and_unit_survive_a_round_trip_as_unknown() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let r = recipe(
            "h",
            "r",
            vec![line("some salt", "salt", Quantity::Unknown, Unit::None)],
        );
        save_recipe(&mut conn, &r).unwrap();
        let loaded = load(&conn, "h", "r").unwrap();
        assert_eq!(loaded.lines()[0].quantity(), Quantity::Unknown);
        assert_eq!(loaded.lines()[0].unit(), &Unit::None);
        // The columns really are NULL, not zero or empty string.
        let (kind, numer, unit_kind, unit_text): (String, Option<u32>, String, Option<String>) =
            conn.query_row(
                "SELECT quantity_kind, min_numer, unit_kind, unit_text
                 FROM recipe_ingredient_line WHERE recipe_id = 'r'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!((kind.as_str(), numer), ("unknown", None));
        assert_eq!((unit_kind.as_str(), unit_text), ("none", None));
    }

    #[test]
    fn other_unit_text_survives_verbatim() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let r = recipe(
            "h",
            "r",
            vec![line(
                "1 handful  Spinach",
                "Spinach",
                Quantity::Exact(rat(1, 1)),
                Unit::Other(" handful ".to_owned()),
            )],
        );
        save_recipe(&mut conn, &r).unwrap();
        let loaded = load(&conn, "h", "r").unwrap();
        assert_eq!(
            loaded.lines()[0].unit(),
            &Unit::Other(" handful ".to_owned())
        );
        assert_eq!(loaded.lines()[0].original_text(), "1 handful  Spinach");
    }

    #[test]
    fn range_and_mixed_fraction_survive_exactly() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let range = QuantityRange::new(rat(2, 1), rat(3, 1)).unwrap();
        let r = recipe(
            "h",
            "r",
            vec![
                line(
                    "1 1/2 cups milk",
                    "milk",
                    Quantity::Exact(rat(3, 2)),
                    Unit::Known(UnitKind::Cup),
                ),
                line(
                    "2-3 cloves",
                    "garlic",
                    Quantity::Range(range),
                    Unit::Known(UnitKind::Piece),
                ),
            ],
        );
        save_recipe(&mut conn, &r).unwrap();
        let loaded = load(&conn, "h", "r").unwrap();
        assert_eq!(loaded.lines()[0].quantity(), Quantity::Exact(rat(3, 2)));
        assert_eq!(loaded.lines()[1].quantity(), Quantity::Range(range));
        assert_eq!(loaded, r);
    }

    #[test]
    fn optional_flag_and_preparation_survive() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let r = recipe(
            "h",
            "r",
            vec![
                full_line(
                    "1 onion, diced (optional)",
                    "onion",
                    None,
                    Quantity::Exact(rat(1, 1)),
                    Unit::None,
                    Some(" diced "),
                    true,
                ),
                full_line(
                    "1 onion",
                    "onion",
                    None,
                    Quantity::Exact(rat(1, 1)),
                    Unit::None,
                    None,
                    false,
                ),
            ],
        );
        save_recipe(&mut conn, &r).unwrap();
        let loaded = load(&conn, "h", "r").unwrap();
        assert_eq!(loaded.lines()[0].preparation(), Some(" diced "));
        assert!(loaded.lines()[0].optional());
        assert_eq!(loaded.lines()[1].preparation(), None);
        assert!(!loaded.lines()[1].optional());
    }

    #[test]
    fn a_stub_recipe_with_no_lines_round_trips() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let r = Recipe::new(
            RecipeId::new("r").unwrap(),
            HouseholdId::new("h").unwrap(),
            "Tuesday thing",
            None,
            "",
            vec![],
            authored(),
        )
        .unwrap();
        save_recipe(&mut conn, &r).unwrap();
        assert_eq!(load(&conn, "h", "r"), Some(r));
    }

    #[test]
    fn provenance_round_trips_for_every_kind() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        assert!(!ProvenanceKind::ALL.is_empty());
        for (i, kind) in ProvenanceKind::ALL.into_iter().enumerate() {
            let id = format!("r{i}");
            let p = RecipeProvenance::new(
                kind,
                Some(format!("https://example.com/{i}")),
                Some("Example".to_owned()),
                Some("A. Cook".to_owned()),
            )
            .unwrap();
            let r = Recipe::new(
                RecipeId::new(id.as_str()).unwrap(),
                HouseholdId::new("h").unwrap(),
                "T",
                None,
                "",
                vec![],
                p.clone(),
            )
            .unwrap();
            save_recipe(&mut conn, &r).unwrap();
            assert_eq!(load(&conn, "h", &id).unwrap().provenance(), &p);
        }
    }

    #[test]
    fn load_of_another_households_recipe_is_none() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        save_recipe(&mut conn, &recipe("h1", "r", vec![])).unwrap();
        assert_eq!(load(&conn, "h2", "r"), None);
        assert!(load(&conn, "h1", "r").is_some());
    }

    #[test]
    fn list_never_returns_another_households_recipes() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        save_recipe(&mut conn, &recipe("h1", "r1", vec![])).unwrap();
        save_recipe(&mut conn, &recipe("h2", "r2", vec![])).unwrap();
        assert_eq!(active_ids(&conn, "h1"), vec!["r1"]);
        assert_eq!(active_ids(&conn, "h2"), vec!["r2"]);
    }

    // --- MVP-008: archive, never hard-delete (owner decision, 2026-08-28) ------------------

    #[test]
    fn archive_hides_a_recipe_from_the_active_list_and_lists_it_as_archived() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r1", vec![])).unwrap();
        save_recipe(&mut conn, &recipe("h", "r2", vec![])).unwrap();
        assert_eq!(archived_ids(&conn, "h"), Vec::<String>::new());
        archive(&mut conn, "h", "r1", "2026-08-29").unwrap();
        assert_eq!(active_ids(&conn, "h"), vec!["r2"]);
        assert_eq!(archived_ids(&conn, "h"), vec!["r1"]);
        assert_eq!(archived_at_raw(&conn, "r1"), Some("2026-08-29".to_owned()));
        // The row is still there: nothing was deleted (PRD §12).
        assert_eq!(count(&conn, "recipe"), 2);
    }

    #[test]
    fn archive_is_idempotent_and_keeps_the_first_date() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        archive(&mut conn, "h", "r", "2026-08-29").unwrap();
        archive(&mut conn, "h", "r", "2026-09-01").unwrap();
        assert_eq!(archived_at_raw(&conn, "r"), Some("2026-08-29".to_owned()));
        assert_eq!(archived_ids(&conn, "h"), vec!["r"]);
    }

    #[test]
    fn restore_returns_a_recipe_to_the_active_list() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        archive(&mut conn, "h", "r", "2026-08-29").unwrap();
        restore(&mut conn, "h", "r").unwrap();
        assert_eq!(archived_at_raw(&conn, "r"), None);
        assert_eq!(active_ids(&conn, "h"), vec!["r"]);
        assert_eq!(archived_ids(&conn, "h"), Vec::<String>::new());
        // Restoring an active recipe is a no-op, not an error.
        restore(&mut conn, "h", "r").unwrap();
        assert_eq!(active_ids(&conn, "h"), vec!["r"]);
    }

    #[test]
    fn archive_and_restore_are_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        save_recipe(&mut conn, &recipe("h1", "r", vec![])).unwrap();
        let err = archive(&mut conn, "h2", "r", "2026-08-29").unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::NoSuchRecipe { recipe, household } if recipe == "r" && household == "h2"
            ),
            "got {err:?}"
        );
        assert_eq!(archived_at_raw(&conn, "r"), None);
        archive(&mut conn, "h1", "r", "2026-08-29").unwrap();
        let err = restore(&mut conn, "h2", "r").unwrap_err();
        assert!(
            matches!(err, StorageError::NoSuchRecipe { .. }),
            "got {err:?}"
        );
        assert_eq!(archived_at_raw(&conn, "r"), Some("2026-08-29".to_owned()));
        assert_eq!(archived_ids(&conn, "h2"), Vec::<String>::new());
    }

    #[test]
    fn archive_of_an_absent_recipe_is_no_such_recipe() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = archive(&mut conn, "h", "ghost", "2026-08-29").unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::NoSuchRecipe { recipe, household } if recipe == "ghost" && household == "h"
            ),
            "got {err:?}"
        );
        let err = restore(&mut conn, "h", "ghost").unwrap_err();
        assert!(
            matches!(err, StorageError::NoSuchRecipe { .. }),
            "got {err:?}"
        );
    }

    /// `save_recipe`'s UPSERT names neither `archived_at` nor a default for it, so editing an
    /// archived recipe neither restores it nor moves its date — restore is the only way back.
    #[test]
    fn save_never_changes_archive_state() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        archive(&mut conn, "h", "r", "2026-08-29").unwrap();
        let edited = recipe(
            "h",
            "r",
            vec![line("1 egg", "egg", Quantity::Unknown, Unit::None)],
        );
        save_recipe(&mut conn, &edited).unwrap();
        assert_eq!(archived_at_raw(&conn, "r"), Some("2026-08-29".to_owned()));
        assert_eq!(load(&conn, "h", "r"), Some(edited));
        assert_eq!(active_ids(&conn, "h"), Vec::<String>::new());
    }

    /// An archived id still resolves by `load_recipe`: this is the reference-stability half
    /// of the archive policy that MVP-012's occurrences will rely on.
    #[test]
    fn load_reports_archived_at() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        assert_eq!(load_record(&conn, "h", "r").unwrap().archived_at, None);
        archive(&mut conn, "h", "r", "2026-08-29").unwrap();
        let record = load_record(&conn, "h", "r").unwrap();
        assert_eq!(
            record.archived_at,
            Some(parse_civil_date("2026-08-29").unwrap())
        );
        assert_eq!(record.recipe, recipe("h", "r", vec![]));
    }

    #[test]
    fn a_line_referencing_another_households_custom_ingredient_is_rejected_before_any_write() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed_for_recipes(&mut conn, "h2"); // owns custom ingredient `c`
        let r = recipe("h1", "r", three_lines());
        let err = save_recipe(&mut conn, &r).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::CustomIngredientHouseholdMismatch { ingredient, expected, actual }
                    if ingredient == "c" && expected == "h1" && actual == "h2"
            ),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "recipe"), 0);
        assert_eq!(count(&conn, "recipe_provenance"), 0);
        assert_eq!(count(&conn, "recipe_ingredient_line"), 0);
    }

    #[test]
    fn saving_over_another_households_recipe_id_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let original = recipe(
            "h1",
            "r",
            vec![line("1 egg", "egg", Quantity::Unknown, Unit::None)],
        );
        save_recipe(&mut conn, &original).unwrap();
        let hijack = recipe("h2", "r", vec![]);
        let err = save_recipe(&mut conn, &hijack).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::NoSuchRecipe { recipe, household } if recipe == "r" && household == "h2"
            ),
            "got {err:?}"
        );
        assert_eq!(load(&conn, "h1", "r"), Some(original));
        assert_eq!(load(&conn, "h2", "r"), None);
        assert_eq!(count(&conn, "recipe"), 1);
        assert_eq!(count(&conn, "recipe_ingredient_line"), 1);
    }

    #[test]
    fn recipe_for_an_absent_household_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        let err = save_recipe(&mut conn, &recipe("ghost", "r", vec![])).unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(h) if h == "ghost"));
        assert_eq!(count(&conn, "recipe"), 0);
    }

    #[test]
    fn a_line_naming_an_absent_ingredient_is_rejected_typed() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let refs = [
            IngredientRef::Catalog(IngredientId::new("ghost-i").unwrap()),
            IngredientRef::Custom(CustomIngredientId::new("ghost-c").unwrap()),
        ];
        for (r, expected) in refs.into_iter().zip(["ghost-i", "ghost-c"]) {
            let l = full_line(
                "x",
                "x",
                Some(r),
                Quantity::Unknown,
                Unit::None,
                None,
                false,
            );
            let err = save_recipe(&mut conn, &recipe("h", "r", vec![l])).unwrap_err();
            assert!(
                matches!(&err, StorageError::NoSuchIngredient(id) if id == expected),
                "got {err:?}"
            );
        }
        assert_eq!(count(&conn, "recipe"), 0);
        assert_eq!(count(&conn, "recipe_provenance"), 0);
        assert_eq!(count(&conn, "recipe_ingredient_line"), 0);
    }

    #[test]
    fn a_corrupt_quantity_row_is_rejected_on_read() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let r = recipe(
            "h",
            "r",
            vec![line("1/2 cup", "x", Quantity::Exact(rat(1, 2)), Unit::None)],
        );
        save_recipe(&mut conn, &r).unwrap();
        conn.execute("UPDATE recipe_ingredient_line SET min_denom = 0", [])
            .unwrap();
        let err = load_recipe(
            &conn,
            &HouseholdId::new("h").unwrap(),
            &RecipeId::new("r").unwrap(),
        )
        .unwrap_err();
        assert!(
            matches!(err, StorageError::Recipe(RecipeError::ZeroDenominator)),
            "got {err:?}"
        );
    }

    #[test]
    fn an_unknown_unit_text_row_is_rejected_on_read() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let r = recipe(
            "h",
            "r",
            vec![line(
                "1 cup",
                "x",
                Quantity::Exact(rat(1, 1)),
                Unit::Known(UnitKind::Cup),
            )],
        );
        save_recipe(&mut conn, &r).unwrap();
        conn.execute("UPDATE recipe_ingredient_line SET unit_text = 'cups'", [])
            .unwrap();
        let err = load_recipe(
            &conn,
            &HouseholdId::new("h").unwrap(),
            &RecipeId::new("r").unwrap(),
        )
        .unwrap_err();
        assert!(
            matches!(&err, StorageError::Recipe(RecipeError::UnknownUnit(u)) if u == "cups"),
            "got {err:?}"
        );
    }

    /// Saves a one-line recipe, applies `mutation` directly to the stored row, and returns
    /// the error `load_recipe` then raises. The line is `Exact(1/2)` in `cup` so either the
    /// quantity or the unit columns can be corrupted from the same fixture.
    fn corrupt_line_and_load(mutation: &str) -> StorageError {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let r = recipe(
            "h",
            "r",
            vec![line(
                "1/2 cup",
                "x",
                Quantity::Exact(rat(1, 2)),
                Unit::Known(UnitKind::Cup),
            )],
        );
        save_recipe(&mut conn, &r).unwrap();
        conn.execute(mutation, []).unwrap();
        load_recipe(
            &conn,
            &HouseholdId::new("h").unwrap(),
            &RecipeId::new("r").unwrap(),
        )
        .unwrap_err()
    }

    #[test]
    fn a_valid_quantity_kind_with_a_half_present_bound_is_reported_with_its_columns() {
        let err = corrupt_line_and_load("UPDATE recipe_ingredient_line SET min_denom = NULL");
        assert!(
            matches!(
                &err,
                StorageError::CorruptQuantity {
                    recipe,
                    position: 0,
                    kind,
                    min_numer: Some(1),
                    min_denom: None,
                    max_numer: None,
                    max_denom: None,
                } if recipe == "r" && kind == "exact"
            ),
            "got {err:?}"
        );
        // The message must send a maintainer to the bound columns, not to the kind
        // vocabulary — `exact` is valid everywhere and is not the fault here.
        let message = err.to_string();
        assert!(message.contains("line 0"), "{message}");
        assert!(message.contains(r#"quantity_kind "exact""#), "{message}");
        assert!(message.contains("min (Some(1), None)"), "{message}");
    }

    #[test]
    fn an_unknown_quantity_kind_row_is_reported_as_unknown_kind() {
        let err = corrupt_line_and_load(
            "UPDATE recipe_ingredient_line SET quantity_kind = 'approximately'",
        );
        assert!(
            matches!(
                &err,
                StorageError::Recipe(RecipeError::UnknownQuantityKind(k)) if k == "approximately"
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn a_corrupt_unit_kind_row_is_reported_with_its_columns() {
        let err = corrupt_line_and_load("UPDATE recipe_ingredient_line SET unit_kind = 'metric'");
        assert!(
            matches!(
                &err,
                StorageError::CorruptUnit { recipe, position: 0, kind, text }
                    if recipe == "r" && kind == "metric" && text.as_deref() == Some("cup")
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn a_known_unit_kind_with_no_unit_text_is_reported_as_corrupt() {
        let err = corrupt_line_and_load("UPDATE recipe_ingredient_line SET unit_text = NULL");
        assert!(
            matches!(
                &err,
                StorageError::CorruptUnit { position: 0, kind, text: None, .. }
                    if kind == "known"
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn the_line_ingredient_foreign_keys_are_indexed() {
        // Pinned to v3: the indexes must ship inside migration 3, not a later one. Expressed
        // as absent-at-2 then present-at-3, because that is the claim — asserting
        // `schema_version == 3` right after `to_version(.., 3)` would be a tautology, and
        // bumping the number to the current latest would delete the claim entirely.
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_version(&mut conn, 2).unwrap();
        assert!(
            line_index_names(&conn).is_empty(),
            "no recipe_ingredient_line index can exist before migration 3 creates the table"
        );
        MIGRATIONS.to_version(&mut conn, 3).unwrap();
        let names = line_index_names(&conn);
        for expected in [
            "recipe_ingredient_line_ingredient",
            "recipe_ingredient_line_custom_ingredient",
        ] {
            assert!(
                names.iter().any(|n| n == expected),
                "{expected} missing from {names:?}"
            );
        }
    }

    fn line_index_names(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master
                 WHERE type = 'index' AND tbl_name = 'recipe_ingredient_line'",
            )
            .unwrap();
        let names = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        names
    }

    #[test]
    fn re_adding_a_custom_ingredient_id_in_the_same_household_updates_it() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_custom_ingredient(&mut conn, &custom("c", "h", "mix")).unwrap();
        let renamed = CustomIngredient::new(
            CustomIngredientId::new("c").unwrap(),
            HouseholdId::new("h").unwrap(),
            "nana's mix",
            Some("spices".to_owned()),
        )
        .unwrap();
        upsert_custom_ingredient(&mut conn, &renamed).unwrap();
        let stored = list_custom_ingredients(&conn, &HouseholdId::new("h").unwrap()).unwrap();
        assert_eq!(stored, vec![renamed]);
        assert_eq!(count(&conn, "custom_ingredient"), 1);
    }

    #[test]
    fn a_custom_ingredient_id_owned_by_another_household_is_rejected_typed() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let mine = custom("c", "h1", "mix");
        upsert_custom_ingredient(&mut conn, &mine).unwrap();
        let err = upsert_custom_ingredient(&mut conn, &custom("c", "h2", "theirs")).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::NoSuchCustomIngredient { ingredient, household }
                    if ingredient == "c" && household == "h2"
            ),
            "got {err:?}"
        );
        // The message names only the requesting household, and never SQLite's own text —
        // the same non-disclosure `save_recipe`'s owner probe exists to provide.
        let message = err.to_string();
        assert!(!message.contains("h1"), "{message}");
        assert!(!message.contains("UNIQUE"), "{message}");
        assert_eq!(
            list_custom_ingredients(&conn, &HouseholdId::new("h1").unwrap()).unwrap(),
            vec![mine]
        );
        assert!(
            list_custom_ingredients(&conn, &HouseholdId::new("h2").unwrap())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_recipe_with_no_provenance_row_is_reported_as_corrupt() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        conn.execute("DELETE FROM recipe_provenance", []).unwrap();
        let err = load_recipe(
            &conn,
            &HouseholdId::new("h").unwrap(),
            &RecipeId::new("r").unwrap(),
        )
        .unwrap_err();
        assert!(
            matches!(&err, StorageError::CorruptProvenance(id) if id == "r"),
            "got {err:?}"
        );
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
        // Two levels: household → household_member → member_food_preference.
        save_member_preferences(
            &mut conn,
            &HouseholdId::new("h").unwrap(),
            &MemberId::new("m1").unwrap(),
            &MemberPreferences::new([MemberPreference::new(Sentiment::Like, "tofu").unwrap()]),
        )
        .unwrap();
        assert_eq!(count(&conn, "member_food_preference"), 1);
        conn.execute("DELETE FROM household WHERE id = ?1", params!["h"])
            .unwrap();
        assert_eq!(count(&conn, "household"), 0);
        assert_eq!(count(&conn, "household_member"), 0);
        assert_eq!(count(&conn, "member_food_preference"), 0);
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
        assert_eq!(schema_version(&conn).unwrap(), 5);
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

    // --- MVP-006: the onboarding flag ------------------------------------------------------

    fn onboarded_of(conn: &Connection, id: &str) -> bool {
        conn.query_row("SELECT onboarded FROM household WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap()
    }

    /// A household that has just been created has not been through first run, so the welcome
    /// screen is what its next launch must show.
    #[test]
    fn a_new_household_is_not_onboarded() {
        let mut conn = open(":memory:").unwrap();
        let record = ensure_household(&mut conn, &household("h1"), &member("m1", "h1")).unwrap();
        assert!(!record.onboarded);
        assert!(!load_household(&conn).unwrap().unwrap().onboarded);
    }

    #[test]
    fn marking_onboarded_is_visible_to_both_loaders() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[member("m1", "h1")]).unwrap();
        let id = HouseholdId::new("h1").unwrap();
        mark_onboarded(&conn, &id).unwrap();
        assert!(load_household(&conn).unwrap().unwrap().onboarded);
        assert!(load_household_by_id(&conn, &id).unwrap().unwrap().onboarded);
    }

    /// Idempotent: the button can be tapped twice, and the second call must not be an error.
    #[test]
    fn marking_onboarded_twice_is_not_an_error() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[]).unwrap();
        let id = HouseholdId::new("h1").unwrap();
        mark_onboarded(&conn, &id).unwrap();
        mark_onboarded(&conn, &id).unwrap();
        assert!(onboarded_of(&conn, "h1"));
    }

    /// The whole point of the flag: a second launch must not show first run again.
    #[test]
    fn the_onboarded_flag_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut conn = open(&path).unwrap();
            ensure_household(&mut conn, &household("h1"), &member("m1", "h1")).unwrap();
            mark_onboarded(&conn, &HouseholdId::new("h1").unwrap()).unwrap();
        }
        let conn = open(&path).unwrap();
        assert!(load_household(&conn).unwrap().unwrap().onboarded);
    }

    #[test]
    fn marking_an_absent_household_is_an_error_and_changes_nothing() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[]).unwrap();
        let err = mark_onboarded(&conn, &HouseholdId::new("nope").unwrap()).unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)));
        assert!(!onboarded_of(&conn, "h1"));
    }

    #[test]
    fn marking_onboarded_touches_only_the_named_household() {
        let mut conn = open(":memory:").unwrap();
        insert_household(&mut conn, &household("h1"), &[]).unwrap();
        insert_household(&mut conn, &household("h2"), &[]).unwrap();
        mark_onboarded(&conn, &HouseholdId::new("h2").unwrap()).unwrap();
        assert!(onboarded_of(&conn, "h2"));
        assert!(!onboarded_of(&conn, "h1"));
    }

    // --- MVP-006: household restrictions ---------------------------------------------------

    fn hid(id: &str) -> HouseholdId {
        HouseholdId::new(id).unwrap()
    }

    fn mixed_set() -> HouseholdRestrictions {
        HouseholdRestrictions::new([
            Restriction::Known(RestrictionKind::Peanuts),
            Restriction::other("nightshades").unwrap(),
            Restriction::Known(RestrictionKind::TreeNuts),
        ])
    }

    /// Inserts a restriction row the domain could not have produced, so the read path's own
    /// guards are what is under test.
    fn raw_restriction(conn: &Connection, household: &str, kind: &str, text: Option<&str>) {
        conn.execute(
            "INSERT INTO household_restriction (household_id, position, kind, text)
             VALUES (?1, 0, ?2, ?3)",
            params![household, kind, text],
        )
        .unwrap();
    }

    #[test]
    fn restrictions_round_trip_in_stored_order() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let set = mixed_set();
        save_restrictions(&mut conn, &hid("h"), &set).unwrap();
        assert_eq!(load_restrictions(&conn, &hid("h")).unwrap(), set);
    }

    #[test]
    fn saving_replaces_the_whole_restriction_set() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_restrictions(&mut conn, &hid("h"), &mixed_set()).unwrap();
        assert_eq!(count(&conn, "household_restriction"), 3);
        let shorter = HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Vegan)]);
        save_restrictions(&mut conn, &hid("h"), &shorter).unwrap();
        assert_eq!(load_restrictions(&conn, &hid("h")).unwrap(), shorter);
        // The rows of the longer set are gone, not merely unread.
        assert_eq!(count(&conn, "household_restriction"), 1);
    }

    #[test]
    fn saving_an_empty_set_clears_the_restrictions() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_restrictions(&mut conn, &hid("h"), &mixed_set()).unwrap();
        save_restrictions(&mut conn, &hid("h"), &HouseholdRestrictions::default()).unwrap();
        assert_eq!(
            load_restrictions(&conn, &hid("h")).unwrap(),
            HouseholdRestrictions::default()
        );
        assert_eq!(count(&conn, "household_restriction"), 0);
    }

    /// The owner's 2026-08-28 scope resolution, pinned: restrictions belong to a household,
    /// and one household's set never leaks into another's.
    #[test]
    fn restrictions_are_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let a = mixed_set();
        let b = HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Gluten)]);
        save_restrictions(&mut conn, &hid("h1"), &a).unwrap();
        save_restrictions(&mut conn, &hid("h2"), &b).unwrap();
        assert_eq!(load_restrictions(&conn, &hid("h1")).unwrap(), a);
        assert_eq!(load_restrictions(&conn, &hid("h2")).unwrap(), b);
        // A household with no set of its own reads empty, not someone else's.
        assert_eq!(
            load_restrictions(&conn, &hid("absent")).unwrap(),
            HouseholdRestrictions::default()
        );
    }

    #[test]
    fn restrictions_for_an_absent_household_are_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = save_restrictions(&mut conn, &hid("nope"), &mixed_set()).unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)), "{err:?}");
        assert_eq!(count(&conn, "household_restriction"), 0);
    }

    /// Invariant 10: a row the vocabulary does not cover is reported with its columns, never
    /// coerced into something displayable. Under-warning is the one direction this must not
    /// fail in.
    #[test]
    fn a_row_with_an_unknown_kind_is_reported_as_corrupt() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_restriction(&conn, "h", "nightshades", None);
        let err = load_restrictions(&conn, &hid("h")).unwrap_err();
        assert!(
            matches!(&err, StorageError::CorruptRestriction { household, position, kind, text }
                if household == "h" && *position == 0 && kind == "nightshades" && text.is_none()),
            "got {err:?}"
        );
    }

    /// SQLite will happily store `kind='other'` with a NULL `text`; the read path must not.
    #[test]
    fn an_other_row_with_no_text_is_reported_as_corrupt() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_restriction(&conn, "h", "other", None);
        let err = load_restrictions(&conn, &hid("h")).unwrap_err();
        assert!(
            matches!(&err, StorageError::CorruptRestriction { kind, text, .. }
                if kind == "other" && text.is_none()),
            "got {err:?}"
        );
    }

    /// A known kind carrying text is a shape fault too: nothing writes it, so a row that has
    /// it did not come from this code.
    #[test]
    fn a_known_kind_row_carrying_text_is_reported_as_corrupt() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_restriction(&conn, "h", "peanuts", Some("peanuts"));
        let err = load_restrictions(&conn, &hid("h")).unwrap_err();
        assert!(
            matches!(&err, StorageError::CorruptRestriction { kind, .. } if kind == "peanuts"),
            "got {err:?}"
        );
    }

    /// Blank text is the domain's own error rather than a shape fault: the row is shaped
    /// correctly and it is the value that cannot be a restriction.
    #[test]
    fn an_other_row_with_blank_text_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_restriction(&conn, "h", "other", Some("   "));
        let err = load_restrictions(&conn, &hid("h")).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::Restriction(RestrictionError::Empty { .. })
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn restrictions_persist_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut conn = open(&path).unwrap();
            seed(&mut conn, "h");
            save_restrictions(&mut conn, &hid("h"), &mixed_set()).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(load_restrictions(&conn, &hid("h")).unwrap(), mixed_set());
    }

    // --- MVP-006: member-scoped preferences (AC-4) -----------------------------------------

    fn mid(id: &str) -> MemberId {
        MemberId::new(id).unwrap()
    }

    fn prefs(items: &[(Sentiment, &str)]) -> MemberPreferences {
        MemberPreferences::new(
            items
                .iter()
                .map(|(s, subject)| MemberPreference::new(*s, *subject).unwrap()),
        )
    }

    /// Two members of the same household, so the household is held constant and only the
    /// member varies.
    fn seed_two_members(conn: &mut Connection) {
        insert_household(
            conn,
            &household("h"),
            &[member("m1", "h"), member("m2", "h")],
        )
        .unwrap();
    }

    /// **AC-4's named evidence.** Preferences are keyed by member, not by household: two
    /// members of the *same* household hold different sets and read back distinctly. The
    /// like/dislike round-trip and stored order are asserted here too — a separate test for
    /// them would be strictly subsumed by this equality.
    #[test]
    fn preferences_are_keyed_by_member_not_household() {
        let mut conn = open(":memory:").unwrap();
        seed_two_members(&mut conn);
        let a = prefs(&[(Sentiment::Like, "tofu"), (Sentiment::Dislike, "olives")]);
        let b = prefs(&[(Sentiment::Dislike, "tofu")]);
        save_member_preferences(&mut conn, &hid("h"), &mid("m1"), &a).unwrap();
        save_member_preferences(&mut conn, &hid("h"), &mid("m2"), &b).unwrap();
        assert_eq!(
            load_member_preferences(&conn, &hid("h"), &mid("m1")).unwrap(),
            a
        );
        assert_eq!(
            load_member_preferences(&conn, &hid("h"), &mid("m2")).unwrap(),
            b
        );
    }

    #[test]
    fn saving_replaces_the_whole_preference_set() {
        let mut conn = open(":memory:").unwrap();
        seed_two_members(&mut conn);
        save_member_preferences(
            &mut conn,
            &hid("h"),
            &mid("m1"),
            &prefs(&[(Sentiment::Like, "tofu"), (Sentiment::Dislike, "olives")]),
        )
        .unwrap();
        assert_eq!(count(&conn, "member_food_preference"), 2);
        let shorter = prefs(&[(Sentiment::Like, "rice")]);
        save_member_preferences(&mut conn, &hid("h"), &mid("m1"), &shorter).unwrap();
        assert_eq!(
            load_member_preferences(&conn, &hid("h"), &mid("m1")).unwrap(),
            shorter
        );
        assert_eq!(count(&conn, "member_food_preference"), 1);
        // The empty set is legal and clears the member's preferences.
        save_member_preferences(
            &mut conn,
            &hid("h"),
            &mid("m1"),
            &MemberPreferences::default(),
        )
        .unwrap();
        assert_eq!(count(&conn, "member_food_preference"), 0);
    }

    #[test]
    fn preferences_for_a_member_of_another_household_are_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed_two_members(&mut conn);
        seed(&mut conn, "other");
        let set = prefs(&[(Sentiment::Like, "tofu")]);
        let err = save_member_preferences(&mut conn, &hid("other"), &mid("m1"), &set).unwrap_err();
        assert!(
            matches!(&err, StorageError::MemberHouseholdMismatch { member, expected, actual }
                if member == "m1" && expected == "other" && actual == "h"),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "member_food_preference"), 0);
        assert!(matches!(
            load_member_preferences(&conn, &hid("other"), &mid("m1")).unwrap_err(),
            StorageError::MemberHouseholdMismatch { .. }
        ));
    }

    #[test]
    fn preferences_for_an_absent_member_are_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed_two_members(&mut conn);
        let set = prefs(&[(Sentiment::Like, "tofu")]);
        let err = save_member_preferences(&mut conn, &hid("h"), &mid("nope"), &set).unwrap_err();
        assert!(
            matches!(&err, StorageError::NoSuchMember(m) if m == "nope"),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "member_food_preference"), 0);
        assert!(matches!(
            load_member_preferences(&conn, &hid("h"), &mid("nope")).unwrap_err(),
            StorageError::NoSuchMember(_)
        ));
    }

    #[test]
    fn a_row_with_an_unknown_sentiment_is_rejected_on_read() {
        let mut conn = open(":memory:").unwrap();
        seed_two_members(&mut conn);
        conn.execute(
            "INSERT INTO member_food_preference (member_id, position, sentiment, subject)
             VALUES ('m1', 0, 'loathes', 'olives')",
            [],
        )
        .unwrap();
        let err = load_member_preferences(&conn, &hid("h"), &mid("m1")).unwrap_err();
        assert!(
            matches!(&err, StorageError::Preference(PreferenceError::UnknownSentiment(s))
                if s == "loathes"),
            "got {err:?}"
        );
    }

    #[test]
    fn preferences_persist_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        let set = prefs(&[(Sentiment::Dislike, "olives")]);
        {
            let mut conn = open(&path).unwrap();
            seed_two_members(&mut conn);
            save_member_preferences(&mut conn, &hid("h"), &mid("m1"), &set).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(
            load_member_preferences(&conn, &hid("h"), &mid("m1")).unwrap(),
            set
        );
    }
}
