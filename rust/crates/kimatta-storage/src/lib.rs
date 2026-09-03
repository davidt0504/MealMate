//! Rust-owned SQLite (PRD v3 §12): foreign keys on, explicit migrations, transactions.
#![forbid(unsafe_code)]

pub mod controller;

use std::collections::{HashMap, HashSet};
use std::path::Path;

pub use controller::*;
pub use food_domain::planner;

pub use food_domain::starter::{
    all_starter_content, shipped_starter_content, CookReview, StarterContent, StarterError,
    StarterRecipe,
};
pub use food_domain::{
    assess, base_factor, derive_shopping_list, format_civil_date, parse_civil_date, quantity_token,
    CivilDate, Conflict, Contribution, CustomIngredient, CustomIngredientId, HouseholdRestrictions,
    IdentityInfo, Ingredient, IngredientId, IngredientLine, IngredientRef, LineStatus,
    MealComponent, MealScope, MealSlot, MemberPreference, MemberPreferences, PlannedMeal,
    PlannedMealError, PlannedMealId, PlanningCycle, PlanningError, PreferenceError, ProvenanceKind,
    Quantity, QuantityRange, Rational, Recipe, RecipeError, RecipeId, RecipeProvenance,
    RecipeRights, Restriction, RestrictionAssessment, RestrictionError, RestrictionKind,
    RightsBasis, Sentiment, SeparateReason, ShoppingError, ShoppingGroup, ShoppingInput,
    ShoppingLine, ShoppingList, ShoppingManualItemId, Unit, UnitFamily, UnitKind, WriteSource,
    DEFAULT_CYCLE_DAYS, MAX_CYCLE_DAYS, MIN_CYCLE_DAYS, RULE_VERSION, SHOPPING_ALGORITHM_VERSION,
};
pub use household_core::{
    ActionProposal, AttentionRequest, Band, Confidence, EvidenceSource, Horizon, Household,
    HouseholdController, HouseholdId, HouseholdMember, IdError, KernelError, LedgerEntry,
    LedgerEntryId, MemberId, OutcomeAssessment, OutcomeStatus, Policy, PolicyId, ReasonCode,
    RequiredAuthority, Reversibility, Urgency,
};
pub use rusqlite;
pub use rusqlite::Connection;
use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};
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
    #[error("recipe {recipe} has an unreadable rights record: {detail}")]
    CorruptRights { recipe: String, detail: String },
    #[error("starter recipe {0:?} carries no starter slug")]
    MissingStarterSlug(String),
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
    #[error(transparent)]
    PlannedMeal(#[from] PlannedMealError),
    #[error("household {0} has no planning cycle yet; read its cycle before planning a meal")]
    NoPlanningCycle(String),
    #[error("no planned meal {meal} in household {household}")]
    NoSuchPlannedMeal { meal: String, household: String },
    #[error("planned meal {0} is locked against automation")]
    LockedPlannedMeal(String),
    #[error("household {household} does not plan the {slot} slot")]
    SlotNotEnabled { household: String, slot: String },
    #[error("recipe {recipe} belongs to household {actual}, not {expected}")]
    ComponentRecipeMismatch {
        recipe: String,
        expected: String,
        actual: String,
    },
    #[error("household {household} already plans {slot} on {date}")]
    OccupiedSlot {
        household: String,
        date: String,
        slot: String,
    },
    #[error(
        "planned meal {meal} component {position} has kind {kind:?} with recipe {recipe_id:?} \
         and scale ({scale_numer:?}, {scale_denom:?})"
    )]
    CorruptComponent {
        meal: String,
        position: usize,
        kind: String,
        recipe_id: Option<String>,
        scale_numer: Option<u32>,
        scale_denom: Option<u32>,
    },
    #[error("no shopping item {0} in this household")]
    NoSuchShoppingItem(String),
    #[error(transparent)]
    Shopping(#[from] ShoppingError),
    #[error(transparent)]
    Kernel(#[from] KernelError),
    /// `table` names the row's own table: `list_policies` and `list_ledger_entries` both report
    /// through this variant, and an operator sent to `controller_ledger` for a `policy` row has
    /// no pointer to the row that is actually corrupt.
    #[error("{table} row {id} has {column} {value:?}, which is not a stored token")]
    CorruptRow {
        table: &'static str,
        id: String,
        column: &'static str,
        value: String,
    },
    #[error("apply was given {got} planned-meal ids for {expected} proposed slots")]
    IdCountMismatch { expected: usize, got: usize },
    #[error(
        "the plan was built from snapshot {expected} but the household is now at {found}; \
         re-run the planner"
    )]
    StalePlan { expected: String, found: String },
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
    // Prep time and the rights columns MVP-007 AC-4 deferred here. All nullable and not
    // backfilled: an authored recipe must not acquire a fabricated rights row, and a recipe
    // written before this migration genuinely has no prep estimate. The `rights_basis`/
    // `verified_on` pairing is not expressible through `ALTER TABLE ADD COLUMN`, so it is
    // enforced on the read path, where migration 5 already puts date correctness.
    M::up(
        "ALTER TABLE recipe ADD COLUMN prep_minutes INTEGER
            CHECK (prep_minutes IS NULL OR prep_minutes >= 1);
        ALTER TABLE recipe_provenance ADD COLUMN rights_basis TEXT;
        ALTER TABLE recipe_provenance ADD COLUMN attribution TEXT;
        ALTER TABLE recipe_provenance ADD COLUMN modifications TEXT;
        ALTER TABLE recipe_provenance ADD COLUMN verified_on TEXT
            CHECK (verified_on IS NULL
                OR verified_on GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]');
        ALTER TABLE recipe_provenance ADD COLUMN starter_slug TEXT;
        CREATE INDEX recipe_provenance_starter_slug ON recipe_provenance(starter_slug);",
    ),
    // Planned meal occurrences (MVP-012). A `planned_meal` row asserts *planned* only, never
    // cooked (invariant 19): confirmation is the outcome ledger's, so no status column lives
    // here. One occurrence per `(household, date, slot)`; `locked` is written only by its own
    // command, never by a save. Components are keyed `(parent, position)` and replaced whole,
    // as every other child set is. No CHECK on `slot`/`kind`: vocabularies validated by parse,
    // as `unit_kind` is. The CHECKs that *are* here pin the kind/column pairing — a non-recipe
    // meal is a `kind` of its own, never a NULL `recipe_id` (card stop condition) — and, for
    // `freeform` alone, that the note the kind exists to carry is actually there, so no write
    // can land a row the reader will refuse. That last one trims the ASCII whitespace set
    // rather than SQLite's `trim` default of spaces only; it is still coarser than the
    // `str::trim` the reader applies, which is why `check_freeform_notes` guards in Rust too.
    // `recipe_id` carries no cascade and is indexed, for the reason the line FKs give; recipes
    // are archived, never deleted, so no component dangles.
    M::up(
        "CREATE TABLE planned_meal (
        id TEXT PRIMARY KEY NOT NULL,
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        date TEXT NOT NULL CHECK (date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
        slot TEXT NOT NULL,
        locked INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
        UNIQUE (household_id, date, slot)
    ) STRICT;
    CREATE INDEX planned_meal_household_date ON planned_meal(household_id, date);
    CREATE TABLE meal_component (
        planned_meal_id TEXT NOT NULL REFERENCES planned_meal(id) ON DELETE CASCADE,
        position INTEGER NOT NULL,
        kind TEXT NOT NULL,
        recipe_id TEXT REFERENCES recipe(id),
        note TEXT,
        scale_numer INTEGER,
        scale_denom INTEGER,
        PRIMARY KEY (planned_meal_id, position),
        CHECK ((kind = 'recipe') = (recipe_id IS NOT NULL)),
        CHECK (kind = 'recipe' OR (scale_numer IS NULL AND scale_denom IS NULL)),
        CHECK (kind <> 'recipe' OR note IS NULL),
        CHECK (kind <> 'freeform'
            OR (note IS NOT NULL AND trim(note, char(32, 9, 10, 11, 12, 13)) <> '')),
        CHECK ((scale_numer IS NULL) = (scale_denom IS NULL)),
        CHECK (scale_numer IS NULL OR (scale_numer >= 1 AND scale_denom >= 1))
    ) STRICT;
    CREATE INDEX meal_component_recipe ON meal_component(recipe_id);",
    ),
    // Optional binary pantry (MVP-014). A row means *this household marked this identity as one
    // it has*; no row means no record, which is unknown — never "they do not have it"
    // (invariant 6, card constraint, PRD §10). There is deliberately no `present` column: a
    // stored `0` would be exactly the absence claim the model may not make, and every consumer
    // (`MVP-015` subtraction, `MVP-023` pantry fit) acts identically on "no record". Identity is
    // the existing `IngredientRef`: exactly one of the two columns, as `recipe_ingredient_line`
    // allows at most one. The pair cannot be a PRIMARY KEY because either column is NULL half
    // the time, so the two partial unique indexes are the real key. Both FKs carry no cascade,
    // for the reason the line FKs give: deleting an ingredient must not silently drop a
    // household's records. Each FK is indexed on its own because the household-leading unique
    // indexes cannot serve the FK-parent-delete lookup.
    M::up(
        "CREATE TABLE pantry_item (
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        ingredient_id TEXT REFERENCES ingredient(id),
        custom_ingredient_id TEXT REFERENCES custom_ingredient(id),
        CHECK ((ingredient_id IS NULL) <> (custom_ingredient_id IS NULL))
    ) STRICT;
    CREATE UNIQUE INDEX pantry_item_catalog
        ON pantry_item(household_id, ingredient_id) WHERE ingredient_id IS NOT NULL;
    CREATE UNIQUE INDEX pantry_item_custom
        ON pantry_item(household_id, custom_ingredient_id)
        WHERE custom_ingredient_id IS NOT NULL;
    CREATE INDEX pantry_item_ingredient ON pantry_item(ingredient_id);
    CREATE INDEX pantry_item_custom_ingredient
        ON pantry_item(custom_ingredient_id);",
    ),
    // Shopping-list overlay (MVP-016). The list itself is never stored — it is re-derived on
    // every read — so these rows are the *only* durable shopping state, and each one is a
    // user action on one derived line. `shopping_line_state` is keyed by the cycle window as
    // well as the line key: a check belongs to this trip, not to the identity forever, so a
    // cycle-settings edit (length/anchor) orphans every window's rows — an accepted
    // limitation, and the rows are kept rather than deleted. `checked` is a plain boolean
    // despite the pantry's no-`present` rule: it records that the user ticked this list, not
    // an inventory claim. `checked_against` is the quantity token the tick was made against,
    // present iff `checked`, so a later read can report a changed amount instead of keeping a
    // check against a different number. A row with every flag off says nothing and is refused;
    // storage deletes instead. `shopping_manual_item` is household-scoped with no window, so
    // an unbought item carries into the next cycle.
    M::up(
        "CREATE TABLE shopping_line_state (
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        from_date TEXT NOT NULL,
        to_date TEXT NOT NULL,
        line_key TEXT NOT NULL CHECK (line_key <> ''),
        checked INTEGER NOT NULL CHECK (checked IN (0, 1)),
        checked_against TEXT,
        hidden INTEGER NOT NULL CHECK (hidden IN (0, 1)),
        restored INTEGER NOT NULL CHECK (restored IN (0, 1)),
        CHECK (checked + hidden + restored > 0),
        CHECK ((checked = 1) = (checked_against IS NOT NULL)),
        PRIMARY KEY (household_id, from_date, to_date, line_key)
    ) STRICT;
    CREATE TABLE shopping_manual_item (
        id TEXT PRIMARY KEY,
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        name TEXT NOT NULL CHECK (trim(name, char(32, 9, 10, 11, 12, 13)) <> ''),
        note TEXT,
        checked INTEGER NOT NULL CHECK (checked IN (0, 1))
    ) STRICT;
    CREATE INDEX shopping_manual_item_household ON shopping_manual_item(household_id);",
    ),
    // Kernel policy envelope and the controller ledger (MVP-023, PRD §7.3, §7.7). `policy`
    // is household-owned and cascades like every other household child; `parameters` is
    // `key<TAB>value` lines, which `Policy::new` keeps unforgeable by refusing tabs and
    // newlines in either half. `controller_ledger` is append-only by trigger, not by
    // convention (invariant 20): UPDATE and DELETE are refused at the engine, and so is an
    // INSERT whose `id` or `seq` already exists — that third trigger is what closes
    // `INSERT OR REPLACE`, whose conflict eviction skips the BEFORE DELETE trigger unless
    // `PRAGMA recursive_triggers` is on (it is off by default and set nowhere here), and
    // which on a `seq` conflict would evict an unrelated older row. `DROP TRIGGER` and
    // `PRAGMA writable_schema` remain bypasses; those take deliberately hostile SQL rather
    // than an ordinary-looking statement. Its household FK
    // deliberately carries **no cascade** — deleting a household while ledger rows exist is
    // refused, because a planner record must outlive the state it describes. `seq` is the
    // total order across households; `reason_codes` is space-joined, which `ReasonCode`
    // keeps splittable by refusing whitespace.
    M::up(
        "CREATE TABLE policy (
        id TEXT PRIMARY KEY NOT NULL,
        household_id TEXT NOT NULL REFERENCES household(id) ON DELETE CASCADE,
        domain TEXT NOT NULL CHECK (trim(domain, char(32, 9, 10, 11, 12, 13)) <> ''),
        policy_type TEXT NOT NULL
            CHECK (trim(policy_type, char(32, 9, 10, 11, 12, 13)) <> ''),
        parameters TEXT NOT NULL,
        enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
        source TEXT NOT NULL
    ) STRICT;
    CREATE INDEX policy_household_domain ON policy(household_id, domain);
    CREATE TABLE controller_ledger (
        id TEXT PRIMARY KEY NOT NULL,
        seq INTEGER NOT NULL UNIQUE,
        household_id TEXT NOT NULL REFERENCES household(id),
        controller_id TEXT NOT NULL,
        algorithm_version INTEGER NOT NULL,
        snapshot_hash TEXT NOT NULL,
        reason_codes TEXT NOT NULL,
        selected_action TEXT NOT NULL,
        prior_status TEXT NOT NULL,
        resulting_status TEXT NOT NULL,
        payload TEXT NOT NULL
    ) STRICT;
    CREATE INDEX controller_ledger_household ON controller_ledger(household_id);
    CREATE TRIGGER controller_ledger_no_update BEFORE UPDATE ON controller_ledger
    BEGIN
        SELECT RAISE(ABORT, 'controller_ledger is append-only');
    END;
    CREATE TRIGGER controller_ledger_no_delete BEFORE DELETE ON controller_ledger
    BEGIN
        SELECT RAISE(ABORT, 'controller_ledger is append-only');
    END;
    CREATE TRIGGER controller_ledger_no_replace BEFORE INSERT ON controller_ledger
    WHEN EXISTS (SELECT 1 FROM controller_ledger
                 WHERE id = NEW.id OR seq = NEW.seq)
    BEGIN
        SELECT RAISE(ABORT, 'controller_ledger is append-only');
    END;",
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
/// A set that actually changes also retires the household's `food.restrictions_reviewed`
/// marker, in this same transaction.
pub fn save_restrictions(
    conn: &mut Connection,
    id: &HouseholdId,
    set: &HouseholdRestrictions,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, id)?;
    // A stored review answers "is this set complete?" about the set that was reviewed. Any
    // change to the set retires that answer, so the marker is disabled in the same
    // transaction that changes the set — otherwise a set emptied again later would read as
    // reviewed and `RESTRICTIONS_NOT_CONFIGURED` would stay suppressed for a state nobody
    // confirmed. Unchanged saves leave the marker alone: re-saving the same set is not a
    // change, and clearing it there would re-ask a question the household just answered.
    //
    // The guard asks only *whether* the set changed, so it compares raw rows rather than
    // calling `load_restrictions`: a row this build cannot parse must not abort the save.
    // Replacing the whole set is the only way to clear such a row — `restriction_from_row`
    // rejects a `kind` a newer build wrote — and routing the guard through it would have
    // closed that repair path. Raw equality is identical to the old domain comparison for any
    // row set this build could have written; it additionally reads as *changed* for an
    // unparseable row and for stored rows `HouseholdRestrictions::new`'s dedup would collapse.
    // Both retire the marker, which is the safe direction: re-asking a question, never
    // suppressing one.
    let writing = restriction_rows_for(set);
    if restriction_rows_in(&tx, id)? != writing {
        disable_policies_of_type_in(
            &tx,
            id,
            "food",
            planner::FoodPolicies::RESTRICTIONS_REVIEWED,
        )?;
    }
    tx.execute(
        "DELETE FROM household_restriction WHERE household_id = ?1",
        params![id.as_str()],
    )?;
    for (position, kind, text) in &writing {
        tx.execute(
            "INSERT INTO household_restriction (household_id, position, kind, text)
             VALUES (?1, ?2, ?3, ?4)",
            params![id.as_str(), position, kind, text],
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
    let restrictions = restriction_rows_in(conn, id)?
        .into_iter()
        .map(|(position, kind, text)| restriction_from_row(id, position as usize, kind, text))
        .collect::<Result<Vec<_>, StorageError>>()?;
    Ok(HouseholdRestrictions::new(restrictions))
}

/// The stored `(position, kind, text)` triples, unparsed and in stored order. Split out of
/// [`load_restrictions`] so [`save_restrictions`] can ask whether the set changed without
/// going through `restriction_from_row`, whose refusal would otherwise abort the one
/// operation that can clear a row this build cannot read.
fn restriction_rows_in(
    conn: &Connection,
    id: &HouseholdId,
) -> Result<Vec<(u32, String, Option<String>)>, StorageError> {
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
    Ok(rows)
}

/// The rows [`save_restrictions`] writes for `set`, in the order it writes them: the one place
/// the domain-to-column mapping lives, so the change guard and the INSERT can never disagree
/// about what "the same set" means. Positions are `u32`, as `length_days` is — rusqlite binds
/// no `usize`, and the set is bounded by what a person will type into a checkbox list.
fn restriction_rows_for(set: &HouseholdRestrictions) -> Vec<(u32, String, Option<String>)> {
    set.restrictions()
        .iter()
        .enumerate()
        .map(|(position, restriction)| match restriction {
            Restriction::Known(kind) => (position as u32, kind.as_str().to_owned(), None),
            Restriction::Other(text) => (position as u32, "other".to_owned(), Some(text.clone())),
        })
        .collect()
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
    upsert_ingredient_rows(&tx, ingredient)?;
    tx.commit()?;
    Ok(())
}

/// The row half of `upsert_ingredient`, for callers that already hold a transaction:
/// `rusqlite` rejects a nested `transaction_with_behavior`, so `install_starter_content`
/// cannot call `upsert_ingredient` itself.
fn upsert_ingredient_rows(
    tx: &Transaction<'_>,
    ingredient: &Ingredient,
) -> Result<(), StorageError> {
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
    Ok(Some(Ingredient::new(
        id.clone(),
        canonical_name,
        load_aliases(conn, id)?,
        store_category,
    )?))
}

/// The catalog ingredient's alternative names in stored order. Shared by `load_ingredient`
/// and `set_pantry_mark`, which reads the same names back for the entry it returns.
fn load_aliases(conn: &Connection, id: &IngredientId) -> Result<Vec<String>, StorageError> {
    let mut stmt =
        conn.prepare("SELECT alias FROM ingredient_alias WHERE ingredient_id = ?1 ORDER BY alias")?;
    let aliases = stmt
        .query_map(params![id.as_str()], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(aliases)
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

/// One browsable identity with this household's mark. `marked` means *the household marked
/// this as one it has*; `false` means **no record**, which is unknown — never a claim that the
/// household lacks it (invariant 6, PRD §10). Named `marked` rather than `present` so no
/// consumer reads `false` as evidence of absence. `aliases` are the catalog's own alternative
/// names, so a search for "garbanzo beans" finds "chickpeas"; a custom ingredient has none.
/// Lives here rather than in `food-domain` because it is a read projection over two tables, as
/// `RecipeSummary` is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PantryEntry {
    pub ingredient: IngredientRef,
    pub name: String,
    pub aliases: Vec<String>,
    pub marked: bool,
}

/// Every catalog alias, keyed by ingredient id, in one query rather than one per row — the
/// "one restriction read per listing, not per recipe" shape the recipe listing uses.
fn alias_index(conn: &Connection) -> Result<HashMap<String, Vec<String>>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT ingredient_id, alias FROM ingredient_alias ORDER BY ingredient_id, alias",
    )?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    for (id, alias) in rows {
        index.entry(id).or_default().push(alias);
    }
    Ok(index)
}

/// Everything this household can mark: the whole catalog plus exactly its own custom
/// ingredients, each carrying its current mark. An absent household reads as the catalog with
/// nothing marked, the same "reads as empty" contract [`load_restrictions`] states — a
/// household with no records has no records in exactly the same sense.
///
/// `COLLATE NOCASE` is load-bearing: BINARY collation would sort every capitalised custom
/// name ahead of the entire lowercase catalog rather than interleaving them.
pub fn list_pantry_entries(
    conn: &Connection,
    household: &HouseholdId,
) -> Result<Vec<PantryEntry>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT 'catalog' AS kind, i.id AS id, i.canonical_name AS name,
                EXISTS(SELECT 1 FROM pantry_item p
                       WHERE p.household_id = ?1 AND p.ingredient_id = i.id) AS marked
         FROM ingredient i
         UNION ALL
         SELECT 'custom', c.id, c.name,
                EXISTS(SELECT 1 FROM pantry_item p
                       WHERE p.household_id = ?1 AND p.custom_ingredient_id = c.id)
         FROM custom_ingredient c WHERE c.household_id = ?1
         ORDER BY name COLLATE NOCASE, kind, id",
    )?;
    let rows = stmt
        .query_map(params![household.as_str()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, bool>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut aliases = alias_index(conn)?;
    rows.into_iter()
        .map(|(kind, id, name, marked)| {
            // Keyed on the kind, not the bare id: a custom ingredient whose id string happens
            // to match a catalog one must not inherit that catalog entry's aliases.
            let (ingredient, aliases) = if kind == "catalog" {
                (
                    IngredientRef::Catalog(IngredientId::new(&id)?),
                    aliases.remove(&id).unwrap_or_default(),
                )
            } else {
                (
                    IngredientRef::Custom(CustomIngredientId::new(&id)?),
                    Vec::new(),
                )
            };
            Ok(PantryEntry {
                ingredient,
                name,
                aliases,
                marked,
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()
}

/// Just the identities this household has marked — the derivation's only pantry need, where
/// [`list_pantry_entries`] is the *browse* read that materialises the whole catalog and every
/// alias to answer the same question.
///
/// One statement per kind rather than one row-shape match: each `WHERE` repeats the predicate
/// of the partial unique index that covers it (`pantry_item_catalog` and `pantry_item_custom`
/// are both household-leading), and neither has to decide what a row violating the table's
/// CHECK would mean. That keeps the read exactly as forgiving as the browse query it replaces —
/// a both-NULL row satisfies neither `EXISTS` there and neither predicate here.
pub fn list_marked_pantry_refs(
    conn: &Connection,
    household: &HouseholdId,
) -> Result<Vec<IngredientRef>, StorageError> {
    // Collected as strings first: the id constructors do not return `rusqlite::Error`, so they
    // cannot be applied inside the `query_map` closure.
    let ids = |sql: &str| -> Result<Vec<String>, StorageError> {
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt
            .query_map(params![household.as_str()], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    };
    let catalog = ids("SELECT ingredient_id FROM pantry_item
          WHERE household_id = ?1 AND ingredient_id IS NOT NULL
          ORDER BY ingredient_id")?;
    let custom = ids("SELECT custom_ingredient_id FROM pantry_item
          WHERE household_id = ?1 AND custom_ingredient_id IS NOT NULL
          ORDER BY custom_ingredient_id")?;
    let mut marked = Vec::with_capacity(catalog.len() + custom.len());
    for id in catalog {
        marked.push(IngredientRef::Catalog(IngredientId::new(&id)?));
    }
    for id in custom {
        marked.push(IngredientRef::Custom(CustomIngredientId::new(&id)?));
    }
    Ok(marked)
}

/// Marks or unmarks one identity for one household in one IMMEDIATE transaction, and returns
/// the entry as stored — the same return-what-was-written contract [`save_restrictions`] and
/// [`save_planning_cycle`] keep. IMMEDIATE because it reads (`require_household`, the identity
/// check) before it writes. Idempotent in both directions: marking twice leaves one row and
/// unmarking twice leaves none, so a repeated tap is never an error.
///
/// Storing a mark is never an assertion about quantity, and removing one restores *no record*
/// rather than recording absence (invariant 6, PRD §10).
pub fn set_pantry_mark(
    conn: &mut Connection,
    household: &HouseholdId,
    ingredient: &IngredientRef,
    marked: bool,
) -> Result<PantryEntry, StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, household)?;
    let (entry, _) = set_pantry_mark_in(&tx, household, ingredient, marked)?;
    tx.commit()?;
    Ok(entry)
}

/// Marks or unmarks every identity in `ingredients` in one IMMEDIATE transaction and returns
/// only the refs whose row actually changed — so a caller undoing a bulk mark sends exactly
/// that set back and never clears a mark that predated it (MVP-016 purchased→pantry). Any
/// failure (an absent or foreign identity) rolls the whole batch back.
pub fn set_pantry_marks(
    conn: &mut Connection,
    household: &HouseholdId,
    ingredients: &[IngredientRef],
    marked: bool,
) -> Result<Vec<IngredientRef>, StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, household)?;
    let mut changed = Vec::new();
    for ingredient in ingredients {
        let (_, did_change) = set_pantry_mark_in(&tx, household, ingredient, marked)?;
        if did_change {
            changed.push(ingredient.clone());
        }
    }
    tx.commit()?;
    Ok(changed)
}

/// The body `set_pantry_mark` and `set_pantry_marks` share; the caller owns the transaction
/// and has already checked the household. The `bool` is whether a row was inserted or
/// deleted — `false` when the mark was already in the requested state.
fn set_pantry_mark_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
    ingredient: &IngredientRef,
    marked: bool,
) -> Result<(PantryEntry, bool), StorageError> {
    check_ingredient_ref(tx, household, ingredient)?;
    let (catalog_id, custom_id) = match ingredient {
        IngredientRef::Catalog(id) => (Some(id.as_str()), None),
        IngredientRef::Custom(id) => (None, Some(id.as_str())),
    };
    let changed = if marked {
        // Untargeted `DO NOTHING`: a targeted `ON CONFLICT(household_id, ingredient_id)` will
        // not prepare against a partial index unless the index's WHERE clause is repeated.
        // Untargeted still propagates the CHECK violation, which is the wanted behaviour.
        tx.execute(
            "INSERT INTO pantry_item (household_id, ingredient_id, custom_ingredient_id)
             VALUES (?1, ?2, ?3) ON CONFLICT DO NOTHING",
            params![household.as_str(), catalog_id, custom_id],
        )?
    } else {
        tx.execute(
            "DELETE FROM pantry_item
             WHERE household_id = ?1 AND ingredient_id IS ?2 AND custom_ingredient_id IS ?3",
            params![household.as_str(), catalog_id, custom_id],
        )?
    };
    let (name, aliases) = match ingredient {
        IngredientRef::Catalog(id) => (
            tx.query_row(
                "SELECT canonical_name FROM ingredient WHERE id = ?1",
                params![id.as_str()],
                |r| r.get::<_, String>(0),
            )?,
            load_aliases(tx, id)?,
        ),
        IngredientRef::Custom(id) => (
            tx.query_row(
                "SELECT name FROM custom_ingredient WHERE id = ?1",
                params![id.as_str()],
                |r| r.get::<_, String>(0),
            )?,
            Vec::new(),
        ),
    };
    Ok((
        PantryEntry {
            ingredient: ingredient.clone(),
            name,
            aliases,
            marked,
        },
        changed == 1,
    ))
}

/// `(id, title)` of a recipe plus its line names, for listing without loading whole recipes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeSummary {
    pub id: RecipeId,
    pub title: String,
    /// Line names in position order, for restriction assessment without loading whole recipes.
    pub line_names: Vec<String>,
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

/// Every referenced ingredient must exist, and every custom one must belong to `household`.
/// Shared by `check_line_refs` and `set_pantry_mark`, so a foreign or absent identity is
/// rejected identically whichever way it arrives.
fn check_ingredient_ref(
    conn: &Connection,
    household: &HouseholdId,
    ingredient: &IngredientRef,
) -> Result<(), StorageError> {
    match ingredient {
        IngredientRef::Catalog(id) => {
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM ingredient WHERE id = ?1)",
                params![id.as_str()],
                |r| r.get(0),
            )?;
            if !exists {
                return Err(StorageError::NoSuchIngredient(id.as_str().to_owned()));
            }
        }
        IngredientRef::Custom(id) => {
            let owner: Option<String> = conn
                .query_row(
                    "SELECT household_id FROM custom_ingredient WHERE id = ?1",
                    params![id.as_str()],
                    |r| r.get(0),
                )
                .optional()?;
            match owner {
                None => return Err(StorageError::NoSuchIngredient(id.as_str().to_owned())),
                Some(actual) if actual != household.as_str() => {
                    return Err(StorageError::CustomIngredientHouseholdMismatch {
                        ingredient: id.as_str().to_owned(),
                        expected: household.as_str().to_owned(),
                        actual,
                    });
                }
                Some(_) => {}
            }
        }
    }
    Ok(())
}

/// Every referenced ingredient must exist, and every custom one must belong to the recipe's
/// household. All probes run before any write, so a rejected save leaves no partial row.
fn check_line_refs(conn: &Connection, recipe: &Recipe) -> Result<(), StorageError> {
    for line in recipe.lines() {
        if let Some(r) = line.ingredient() {
            check_ingredient_ref(conn, recipe.household_id(), r)?;
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
    write_recipe(&tx, recipe)?;
    tx.commit()?;
    Ok(())
}

/// Ownership probe + the recipe/provenance/line writes. Callers own `require_household` and
/// `check_line_refs`, so a multi-recipe caller runs the household check once, not per recipe;
/// the ownership probe stays here because it is per-recipe.
///
/// The four rights columns move as **one group**, never per-column: when the row already
/// exists and the incoming provenance carries no rights, all four are left untouched — that is
/// the MVP-008 edit path, which cannot supply them (`RecipeDto`'s rights scalars are
/// output-only). When the incoming provenance does carry rights, all four are overwritten from
/// it, so an incoming absent attribution clears the stored one rather than silently retaining
/// a credit the caller did not give. `starter_slug` follows the same rule independently.
fn write_recipe(tx: &Transaction<'_>, recipe: &Recipe) -> Result<(), StorageError> {
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
        "INSERT INTO recipe (id, household_id, title, servings, prep_minutes, instructions)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
             title = excluded.title,
             servings = excluded.servings,
             prep_minutes = excluded.prep_minutes,
             instructions = excluded.instructions",
        params![
            id,
            recipe.household_id().as_str(),
            recipe.title(),
            recipe.servings(),
            recipe.prep_minutes(),
            recipe.instructions(),
        ],
    )?;
    let p = recipe.provenance();
    let rights = p.rights();
    let has_rights = rights.is_some();
    tx.execute(
        "INSERT INTO recipe_provenance
             (recipe_id, kind, source_url, source_name, source_author,
              rights_basis, attribution, modifications, verified_on, starter_slug)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(recipe_id) DO UPDATE SET
             kind = excluded.kind,
             source_url = excluded.source_url,
             source_name = excluded.source_name,
             source_author = excluded.source_author,
             rights_basis = CASE WHEN ?11 THEN excluded.rights_basis ELSE rights_basis END,
             attribution = CASE WHEN ?11 THEN excluded.attribution ELSE attribution END,
             modifications = CASE WHEN ?11 THEN excluded.modifications ELSE modifications END,
             verified_on = CASE WHEN ?11 THEN excluded.verified_on ELSE verified_on END,
             starter_slug = COALESCE(excluded.starter_slug, starter_slug)",
        params![
            id,
            p.kind().as_str(),
            p.source_url(),
            p.source_name(),
            p.source_author(),
            rights.map(|r| r.basis().as_str()),
            rights.and_then(RecipeRights::attribution),
            rights.and_then(RecipeRights::modifications),
            rights.map(|r| format_civil_date(r.verified_on())),
            p.starter_slug(),
            has_rights,
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
    Ok(())
}

/// What one `install_starter_content` call actually wrote. `installed: 0` with
/// `catalog_installed > 0` is the empty-by-design case, not a swallowed failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StarterInstallReport {
    /// Recipes written this call.
    pub installed: usize,
    /// Slugs this household already holds, archived ones included.
    pub skipped: usize,
    /// Catalog rows written this call.
    pub catalog_installed: usize,
}

/// Seeds the global catalog and any starter recipe whose slug this household does not already
/// hold, in one transaction. Idempotent and re-runnable: a slug already present — including on
/// an archived recipe — is skipped, so archiving a starter recipe is never undone.
///
/// **Install-once by design.** A revised recipe does not overwrite an installed row, and once
/// every catalog id and slug is present the call short-circuits before opening a transaction,
/// so a catalog correction carrying no new id never lands either. Correcting installed content
/// is a later card's work; see the MVP-011 handoff row.
///
/// Idempotency is code-enforced, not schema-enforced: `recipe_provenance` has no
/// `household_id`, so a partial unique index on `(household_id, starter_slug)` is not
/// expressible without touching `recipe`. The single process-wide connection mutex in the
/// bridge crate makes a concurrent second install unreachable, and the slug check shares one
/// transaction with the writes.
pub fn install_starter_content(
    conn: &mut Connection,
    household: &HouseholdId,
    catalog: &[Ingredient],
    recipes: &[Recipe],
) -> Result<StarterInstallReport, StorageError> {
    // First, and outside the short-circuit: with an empty shipped set every launch after the
    // first takes the short-circuit path, so a household check living in the write branch
    // would never run in production.
    require_household(conn, household)?;
    let held_ids: HashSet<String> = conn
        .prepare("SELECT id FROM ingredient")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<_, _>>()?;
    let held_slugs: HashSet<String> = conn
        .prepare(
            "SELECT p.starter_slug FROM recipe_provenance p
             JOIN recipe r ON r.id = p.recipe_id
             WHERE r.household_id = ?1 AND p.starter_slug IS NOT NULL",
        )?
        .query_map(params![household.as_str()], |r| r.get::<_, String>(0))?
        .collect::<Result<_, _>>()?;
    let mut pending = Vec::new();
    for recipe in recipes {
        let slug = recipe
            .provenance()
            .starter_slug()
            .ok_or_else(|| StorageError::MissingStarterSlug(recipe.title().to_owned()))?;
        if !held_slugs.contains(slug) {
            pending.push(recipe);
        }
    }
    let missing_catalog = catalog.iter().any(|i| !held_ids.contains(i.id().as_str()));
    if pending.is_empty() && !missing_catalog {
        return Ok(StarterInstallReport {
            installed: 0,
            skipped: recipes.len(),
            catalog_installed: 0,
        });
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    // The whole catalog, not only the missing ids: `upsert_ingredient_rows` has replace-whole
    // semantics, so upserting only new ids would strand a corrected canonical name, store
    // category or alias set on a device that reaches this branch. It only reaches it when a new
    // id or a new slug is pending — see the install-once note on this fn: a correction carrying
    // neither never gets here at all.
    for ingredient in catalog {
        upsert_ingredient_rows(&tx, ingredient)?;
    }
    for recipe in &pending {
        check_line_refs(&tx, recipe)?;
        write_recipe(&tx, recipe)?;
    }
    tx.commit()?;
    Ok(StarterInstallReport {
        installed: pending.len(),
        skipped: recipes.len() - pending.len(),
        catalog_installed: catalog.len(),
    })
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

/// The four stored rights columns, before the read rule below decides what they mean.
struct RightsRow {
    basis: Option<String>,
    attribution: Option<String>,
    modifications: Option<String>,
    verified_on: Option<String>,
}

/// Rights are present **iff** `rights_basis` and `verified_on` are both non-NULL. Every other
/// shape is `CorruptRights`, never coercion: half a rights record silently read as "no rights"
/// would drop an attribution the row still carries, which is exactly the invariant-12 leak the
/// rights columns exist to prevent. `starter_slug` is independent of this rule.
fn rights_from_row(recipe: &str, row: RightsRow) -> Result<Option<RecipeRights>, StorageError> {
    let corrupt = |detail: &str| StorageError::CorruptRights {
        recipe: recipe.to_owned(),
        detail: detail.to_owned(),
    };
    let (basis, verified_on) = match (row.basis, row.verified_on) {
        (Some(basis), Some(verified_on)) => (basis, verified_on),
        (Some(_), None) => return Err(corrupt("rights_basis is set but verified_on is NULL")),
        (None, Some(_)) => return Err(corrupt("verified_on is set but rights_basis is NULL")),
        (None, None) => {
            return match (row.attribution, row.modifications) {
                (None, None) => Ok(None),
                _ => Err(corrupt(
                    "attribution or modifications without a rights basis",
                )),
            };
        }
    };
    let basis = RightsBasis::parse(&basis)
        .map_err(|e| corrupt(&format!("unreadable rights_basis: {e}")))?;
    // The column CHECK only proves the GLOB shape, so `2026-13-45` reaches here.
    RecipeRights::new(basis, row.attribution, row.modifications, &verified_on)
        .map(Some)
        .map_err(|e| corrupt(&format!("unreadable rights record: {e}")))
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
            "SELECT title, servings, prep_minutes, instructions, archived_at FROM recipe
             WHERE id = ?1 AND household_id = ?2",
            params![id.as_str(), household.as_str()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<u32>>(1)?,
                    r.get::<_, Option<u32>>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .optional()?;
    let Some((title, servings, prep_minutes, instructions, archived_at)) = row else {
        return Ok(None);
    };
    let archived_at = archived_at.as_deref().map(parse_civil_date).transpose()?;
    let provenance = conn
        .query_row(
            "SELECT kind, source_url, source_name, source_author,
                    rights_basis, attribution, modifications, verified_on, starter_slug
             FROM recipe_provenance WHERE recipe_id = ?1",
            params![id.as_str()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    RightsRow {
                        basis: r.get(4)?,
                        attribution: r.get(5)?,
                        modifications: r.get(6)?,
                        verified_on: r.get(7)?,
                    },
                    r.get::<_, Option<String>>(8)?,
                ))
            },
        )
        .optional()?;
    let Some((kind, source_url, source_name, source_author, rights_row, starter_slug)) = provenance
    else {
        return Err(StorageError::CorruptProvenance(id.as_str().to_owned()));
    };
    let provenance = RecipeProvenance::with_rights(
        ProvenanceKind::parse(&kind)?,
        source_url,
        source_name,
        source_author,
        rights_from_row(id.as_str(), rights_row)?,
        starter_slug,
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
            prep_minutes,
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
    let mut summaries = stmt
        .query_map(params![household.as_str()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .map(|row| {
            let (id, title) = row?;
            Ok(RecipeSummary {
                id: RecipeId::new(id)?,
                title,
                line_names: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    // One query for every listed recipe's line names, not one per recipe: MVP-009 assesses
    // every summary on each listing.
    let mut lines = conn.prepare(
        "SELECT l.recipe_id, l.name FROM recipe_ingredient_line l
         JOIN recipe r ON r.id = l.recipe_id
         WHERE r.household_id = ?1 AND (r.archived_at IS NULL) = ?2
         ORDER BY l.recipe_id, l.position",
    )?;
    let mut by_recipe: HashMap<String, Vec<String>> = HashMap::new();
    for row in lines.query_map(
        params![household.as_str(), listing == RecipeListing::Active],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )? {
        let (recipe_id, name) = row?;
        by_recipe.entry(recipe_id).or_default().push(name);
    }
    for summary in &mut summaries {
        if let Some(names) = by_recipe.remove(summary.id.as_str()) {
            summary.line_names = names;
        }
    }
    Ok(summaries)
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

/// Every recipe component must name a recipe in `meal.household_id()` — archived ones
/// included, since an occurrence keeps its reference when its recipe is archived (MVP-008
/// decision). All probes run before any write, as `check_line_refs` does.
fn check_component_recipes(conn: &Connection, meal: &PlannedMeal) -> Result<(), StorageError> {
    for component in meal.components() {
        let MealComponent::Recipe { recipe_id, .. } = component else {
            continue;
        };
        let owner: Option<String> = conn
            .query_row(
                "SELECT household_id FROM recipe WHERE id = ?1",
                params![recipe_id.as_str()],
                |r| r.get(0),
            )
            .optional()?;
        match owner {
            None => {
                return Err(StorageError::NoSuchRecipe {
                    recipe: recipe_id.as_str().to_owned(),
                    household: meal.household_id().as_str().to_owned(),
                })
            }
            Some(actual) if actual != meal.household_id().as_str() => {
                return Err(StorageError::ComponentRecipeMismatch {
                    recipe: recipe_id.as_str().to_owned(),
                    expected: meal.household_id().as_str().to_owned(),
                    actual,
                });
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// Every freeform component must carry a note `MealComponent::freeform` would accept. That
/// constructor is the only place the rule lives, but an enum variant's fields are as public as
/// the enum, so `MealComponent::Freeform { note }` builds one around it — and the read path
/// routes through `freeform()`, so such a row commits and is then unreadable, taking the whole
/// date range with it. The rule is re-run through the constructor rather than restated, since
/// the v7 CHECK behind it can only trim ASCII whitespace and `str::trim` strips more.
/// Runs before any write, as `check_component_recipes` does.
fn check_freeform_notes(meal: &PlannedMeal) -> Result<(), StorageError> {
    for component in meal.components() {
        if let MealComponent::Freeform { note } = component {
            MealComponent::freeform(note.as_str())?;
        }
    }
    Ok(())
}

/// Inserts or wholly replaces the occurrence and its component set in one IMMEDIATE
/// transaction. Checked before any write, in order: the household exists; it has a planning
/// cycle (`NoPlanningCycle` — a household that has never read its cycle cannot plan yet);
/// the slot is enabled (`SlotNotEnabled`, **write-time only** — disabling a slot later never
/// hides or deletes what was planned in it); every recipe component belongs to this household;
/// every freeform component carries a note its own constructor would accept
/// (`FreeformNeedsANote`, which a caller building the variant directly can otherwise bypass);
/// an existing id owned elsewhere is `NoSuchPlannedMeal`, never hijacked; an existing locked row
/// refuses `Automation` outright (`LockedPlannedMeal`, invariant 18); and a `(household, date,
/// slot)` already held by a *different* id is `OccupiedSlot`, typed before the UNIQUE fires.
///
/// `locked` is **never written here**, as `archived_at` is not by `save_recipe`: a fresh row
/// takes the column default and an update leaves the column alone, so the value's own
/// `locked()` is not what ends up stored — callers that need it re-read.
pub fn save_planned_meal(
    conn: &mut Connection,
    meal: &PlannedMeal,
    source: WriteSource,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    save_planned_meal_in(&tx, meal, source)?;
    tx.commit()?;
    Ok(())
}

/// The body `save_planned_meal` and `apply_plan_and_record` share; the caller owns the
/// transaction, as `set_pantry_mark_in` and `insert_rows` are shaped. Public for the same
/// reason `load_planning_snapshot_in` is: the decision recorder composes it under one
/// IMMEDIATE transaction with the ledger append.
pub fn save_planned_meal_in(
    tx: &Transaction<'_>,
    meal: &PlannedMeal,
    source: WriteSource,
) -> Result<(), StorageError> {
    let household = meal.household_id();
    require_household(tx, household)?;
    let cycle = load_planning_cycle(tx, household)?
        .ok_or_else(|| StorageError::NoPlanningCycle(household.as_str().to_owned()))?;
    if !cycle.scope().contains(meal.slot()) {
        return Err(StorageError::SlotNotEnabled {
            household: household.as_str().to_owned(),
            slot: meal.slot().as_str().to_owned(),
        });
    }
    check_component_recipes(tx, meal)?;
    check_freeform_notes(meal)?;
    let id = meal.id().as_str();
    let existing: Option<(String, bool)> = tx
        .query_row(
            "SELECT household_id, locked FROM planned_meal WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((owner, locked)) = existing {
        if owner != household.as_str() {
            return Err(StorageError::NoSuchPlannedMeal {
                meal: id.to_owned(),
                household: household.as_str().to_owned(),
            });
        }
        if locked && source == WriteSource::Automation {
            return Err(StorageError::LockedPlannedMeal(id.to_owned()));
        }
    }
    let date = format_civil_date(meal.date());
    let holder: Option<String> = tx
        .query_row(
            "SELECT id FROM planned_meal WHERE household_id = ?1 AND date = ?2 AND slot = ?3",
            params![household.as_str(), date, meal.slot().as_str()],
            |r| r.get(0),
        )
        .optional()?;
    if holder.is_some_and(|h| h != id) {
        return Err(StorageError::OccupiedSlot {
            household: household.as_str().to_owned(),
            date,
            slot: meal.slot().as_str().to_owned(),
        });
    }
    tx.execute(
        "INSERT INTO planned_meal (id, household_id, date, slot) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET date = excluded.date, slot = excluded.slot",
        params![id, household.as_str(), date, meal.slot().as_str()],
    )?;
    tx.execute(
        "DELETE FROM meal_component WHERE planned_meal_id = ?1",
        params![id],
    )?;
    for (position, component) in meal.components().iter().enumerate() {
        let (recipe_id, note, scale) = match component {
            MealComponent::Recipe { recipe_id, scale } => (Some(recipe_id.as_str()), None, *scale),
            MealComponent::Leftovers { note }
            | MealComponent::DiningOut { note }
            | MealComponent::FrozenQuick { note }
            | MealComponent::Open { note } => (None, note.as_deref(), None),
            MealComponent::Freeform { note } => (None, Some(note.as_str()), None),
        };
        tx.execute(
            "INSERT INTO meal_component
             (planned_meal_id, position, kind, recipe_id, note, scale_numer, scale_denom)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                position as u32,
                component.kind_str(),
                recipe_id,
                note,
                scale.map(Rational::numer),
                scale.map(Rational::denom),
            ],
        )?;
    }
    Ok(())
}

/// Sets or clears the lock on exactly this household's occurrence. `Automation` is refused
/// outright, whichever way the flag is going: a lock is the user's Tier-0 word (invariant 18),
/// and letting automation clear one would reopen every write the lock exists to refuse.
/// Idempotent for the user; absent or foreign is `NoSuchPlannedMeal`.
pub fn set_planned_meal_lock(
    conn: &mut Connection,
    household: &HouseholdId,
    id: &PlannedMealId,
    locked: bool,
    source: WriteSource,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    set_planned_meal_lock_in(&tx, household, id, locked, source)?;
    tx.commit()?;
    Ok(())
}

/// The body `set_planned_meal_lock` shares with the decision recorder; the caller owns the
/// transaction. The `Automation` refusal travels with the body: a lock is the user's Tier-0
/// word wherever the write comes from.
pub fn set_planned_meal_lock_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
    id: &PlannedMealId,
    locked: bool,
    source: WriteSource,
) -> Result<(), StorageError> {
    if source == WriteSource::Automation {
        return Err(StorageError::LockedPlannedMeal(id.as_str().to_owned()));
    }
    let changed = tx.execute(
        "UPDATE planned_meal SET locked = ?1 WHERE id = ?2 AND household_id = ?3",
        params![locked, id.as_str(), household.as_str()],
    )?;
    if changed == 0 {
        return Err(StorageError::NoSuchPlannedMeal {
            meal: id.as_str().to_owned(),
            household: household.as_str().to_owned(),
        });
    }
    Ok(())
}

/// Removes the occurrence and, by cascade, its components. A hard delete is right here: an
/// occurrence is current plan state, not a planner record (invariant 20 guards `PlannerRun`,
/// MVP-023's table). `Automation` on a locked row is `LockedPlannedMeal`; absent or foreign is
/// `NoSuchPlannedMeal`.
pub fn delete_planned_meal(
    conn: &mut Connection,
    household: &HouseholdId,
    id: &PlannedMealId,
    source: WriteSource,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    delete_planned_meal_in(&tx, household, id, source)?;
    tx.commit()?;
    Ok(())
}

/// The body `delete_planned_meal` and `apply_plan_and_record` share; the caller owns the
/// transaction. Public for the decision recorder, like `save_planned_meal_in`.
pub fn delete_planned_meal_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
    id: &PlannedMealId,
    source: WriteSource,
) -> Result<(), StorageError> {
    let locked: Option<bool> = tx
        .query_row(
            "SELECT locked FROM planned_meal WHERE id = ?1 AND household_id = ?2",
            params![id.as_str(), household.as_str()],
            |r| r.get(0),
        )
        .optional()?;
    match locked {
        None => {
            return Err(StorageError::NoSuchPlannedMeal {
                meal: id.as_str().to_owned(),
                household: household.as_str().to_owned(),
            })
        }
        Some(true) if source == WriteSource::Automation => {
            return Err(StorageError::LockedPlannedMeal(id.as_str().to_owned()));
        }
        Some(_) => {}
    }
    tx.execute(
        "DELETE FROM planned_meal WHERE id = ?1",
        params![id.as_str()],
    )?;
    Ok(())
}

/// One stored component, before it goes back through the domain constructors.
struct ComponentRow {
    position: usize,
    kind: String,
    recipe_id: Option<String>,
    note: Option<String>,
    scale_numer: Option<u32>,
    scale_denom: Option<u32>,
}

const COMPONENT_COLUMNS: &str =
    "c.planned_meal_id, c.position, c.kind, c.recipe_id, c.note, c.scale_numer, c.scale_denom";

fn component_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<(String, ComponentRow)> {
    Ok((
        r.get(0)?,
        ComponentRow {
            position: r.get::<_, u32>(1)? as usize,
            kind: r.get(2)?,
            recipe_id: r.get(3)?,
            note: r.get(4)?,
            scale_numer: r.get(5)?,
            scale_denom: r.get(6)?,
        },
    ))
}

/// A row whose columns do not describe its `kind` — including a half-present scale — is
/// `CorruptComponent` with its coordinates, never coerced (the rule `restriction_from_row`
/// states). An unknown `kind` token or a blank freeform note is the domain's own error.
fn component_from_row(meal: &str, row: ComponentRow) -> Result<MealComponent, StorageError> {
    let corrupt = || StorageError::CorruptComponent {
        meal: meal.to_owned(),
        position: row.position,
        kind: row.kind.clone(),
        recipe_id: row.recipe_id.clone(),
        scale_numer: row.scale_numer,
        scale_denom: row.scale_denom,
    };
    let scale = match (row.scale_numer, row.scale_denom) {
        (Some(n), Some(d)) => Some(MealComponent::parse_scale(n, d)?),
        (None, None) => None,
        _ => return Err(corrupt()),
    };
    let recipe_id = row.recipe_id.clone().map(RecipeId::new).transpose()?;
    MealComponent::parse_row(&row.kind, recipe_id, row.note.clone(), scale)?.ok_or_else(corrupt)
}

fn meal_from_rows(
    household: &HouseholdId,
    id: String,
    date: String,
    slot: String,
    locked: bool,
    rows: Vec<ComponentRow>,
) -> Result<PlannedMeal, StorageError> {
    let components = rows
        .into_iter()
        .map(|row| component_from_row(&id, row))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PlannedMeal::new(
        PlannedMealId::new(id)?,
        household.clone(),
        parse_civil_date(&date)?,
        MealSlot::parse(&slot)?,
        components,
        locked,
    )?)
}

/// The occurrence `id` **in `household`**, or `None` — another household's is `None`, never
/// the row. Every stored value goes back through the domain constructors.
pub fn load_planned_meal(
    conn: &Connection,
    household: &HouseholdId,
    id: &PlannedMealId,
) -> Result<Option<PlannedMeal>, StorageError> {
    let row = conn
        .query_row(
            "SELECT date, slot, locked FROM planned_meal WHERE id = ?1 AND household_id = ?2",
            params![id.as_str(), household.as_str()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, bool>(2)?,
                ))
            },
        )
        .optional()?;
    let Some((date, slot, locked)) = row else {
        return Ok(None);
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT {COMPONENT_COLUMNS} FROM meal_component c
         WHERE c.planned_meal_id = ?1 ORDER BY c.position"
    ))?;
    let rows = stmt
        .query_map(params![id.as_str()], |r| Ok(component_row(r)?.1))?
        .collect::<Result<Vec<_>, _>>()?;
    meal_from_rows(household, id.as_str().to_owned(), date, slot, locked, rows).map(Some)
}

/// Every occurrence in `household` dated `from..=to`, ordered by date then `MealSlot`
/// canonical order — sorted in Rust after parse, since the slot vocabulary lives here, not in
/// SQL. Components for the whole listing come from one query, as `list_recipes` line names do.
pub fn list_planned_meals(
    conn: &Connection,
    household: &HouseholdId,
    from: CivilDate,
    to: CivilDate,
) -> Result<Vec<PlannedMeal>, StorageError> {
    let (from, to) = (format_civil_date(from), format_civil_date(to));
    let mut components = conn.prepare(&format!(
        "SELECT {COMPONENT_COLUMNS} FROM meal_component c
         JOIN planned_meal p ON p.id = c.planned_meal_id
         WHERE p.household_id = ?1 AND p.date BETWEEN ?2 AND ?3
         ORDER BY c.planned_meal_id, c.position"
    ))?;
    let mut by_meal: HashMap<String, Vec<ComponentRow>> = HashMap::new();
    for row in components.query_map(params![household.as_str(), from, to], component_row)? {
        let (meal_id, row) = row?;
        by_meal.entry(meal_id).or_default().push(row);
    }
    let mut stmt = conn.prepare(
        "SELECT id, date, slot, locked FROM planned_meal
         WHERE household_id = ?1 AND date BETWEEN ?2 AND ?3 ORDER BY date, id",
    )?;
    let mut meals = stmt
        .query_map(params![household.as_str(), from, to], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, bool>(3)?,
            ))
        })?
        .map(|row| {
            let (id, date, slot, locked) = row?;
            let rows = by_meal.remove(&id).unwrap_or_default();
            meal_from_rows(household, id, date, slot, locked, rows)
        })
        .collect::<Result<Vec<_>, StorageError>>()?;
    meals.sort_by_key(|m| (m.date(), m.slot()));
    Ok(meals)
}

/// The derivation's input snapshot for `household` over `from..=to`, read in one transaction
/// so meals, recipes, identities and pantry marks come from one consistent state. Archived
/// recipes are included: an occurrence may still name one (MVP-012). Recipes are loaded once
/// per distinct id, bounded by the distinct recipes in range.
pub fn load_shopping_input(
    conn: &mut Connection,
    household: &HouseholdId,
    from: CivilDate,
    to: CivilDate,
) -> Result<ShoppingInput, StorageError> {
    let tx = conn.transaction()?;
    require_household(&tx, household)?;
    let meals = list_planned_meals(&tx, household, from, to)?;
    let mut recipe_ids: Vec<&RecipeId> = meals
        .iter()
        .flat_map(|m| m.components().iter())
        .filter_map(|c| match c {
            MealComponent::Recipe { recipe_id, .. } => Some(recipe_id),
            _ => None,
        })
        .collect();
    recipe_ids.sort_by_key(|id| id.as_str());
    recipe_ids.dedup();
    let mut recipes = Vec::with_capacity(recipe_ids.len());
    for id in recipe_ids {
        if let Some(record) = load_recipe(&tx, household, id)? {
            recipes.push(record.recipe);
        }
    }
    let customs: HashMap<String, CustomIngredient> = list_custom_ingredients(&tx, household)?
        .into_iter()
        .map(|c| (c.id().as_str().to_owned(), c))
        .collect();
    let mut identities: Vec<(IngredientRef, IdentityInfo)> = Vec::new();
    // Keyed by `(kind, id)` strings: `IngredientRef` carries no `Hash`, and a bare id would
    // let a custom ingredient shadow a catalog one sharing its id string.
    let mut seen: HashSet<(bool, String)> = HashSet::new();
    for line in recipes.iter().flat_map(|r| r.lines().iter()) {
        let Some(r) = line.ingredient() else {
            continue;
        };
        let seen_key = match r {
            IngredientRef::Catalog(id) => (true, id.as_str().to_owned()),
            IngredientRef::Custom(id) => (false, id.as_str().to_owned()),
        };
        if !seen.insert(seen_key) {
            continue;
        }
        let info = match r {
            IngredientRef::Catalog(id) => load_ingredient(&tx, id)?.map(|i| IdentityInfo {
                name: i.canonical_name().to_owned(),
                store_category: i.store_category().map(str::to_owned),
            }),
            IngredientRef::Custom(id) => customs.get(id.as_str()).map(|c| IdentityInfo {
                name: c.name().to_owned(),
                store_category: c.store_category().map(str::to_owned),
            }),
        };
        if let Some(info) = info {
            identities.push((r.clone(), info));
        }
    }
    let pantry_marked = list_marked_pantry_refs(&tx, household)?;
    tx.commit()?;
    Ok(ShoppingInput {
        from,
        to,
        meals,
        recipes,
        identities,
        pantry_marked,
    })
}

/// `load_shopping_input` then `food_domain::derive_shopping_list`: the one read the bridge
/// exposes. Never stores anything.
pub fn load_shopping_list(
    conn: &mut Connection,
    household: &HouseholdId,
    from: CivilDate,
    to: CivilDate,
) -> Result<ShoppingList, StorageError> {
    let input = load_shopping_input(conn, household, from, to)?;
    Ok(derive_shopping_list(&input))
}

/// One derived line's user state for one cycle window (MVP-016). `checked_against` is the
/// `quantity_token` the check was made against, `Some` iff `checked`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingLineState {
    pub key: String,
    pub checked: bool,
    pub checked_against: Option<String>,
    pub hidden: bool,
    pub restored: bool,
}

impl ShoppingLineState {
    fn is_blank(&self) -> bool {
        !self.checked && !self.hidden && !self.restored
    }
}

/// Every stored line state for `household` over exactly `from..=to`, in key order. A window
/// that differs in either bound reads none — the rows are window-keyed by design.
pub fn list_shopping_line_states(
    conn: &Connection,
    household: &HouseholdId,
    from: CivilDate,
    to: CivilDate,
) -> Result<Vec<ShoppingLineState>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT line_key, checked, checked_against, hidden, restored FROM shopping_line_state
         WHERE household_id = ?1 AND from_date = ?2 AND to_date = ?3 ORDER BY line_key",
    )?;
    let rows = stmt
        .query_map(
            params![
                household.as_str(),
                format_civil_date(from),
                format_civil_date(to)
            ],
            |r| {
                Ok(ShoppingLineState {
                    key: r.get(0)?,
                    checked: r.get(1)?,
                    checked_against: r.get(2)?,
                    hidden: r.get(3)?,
                    restored: r.get(4)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Upserts one line's state and returns what was stored. A state with every flag off is a
/// row that says nothing, so it is deleted instead of written and returned as-is;
/// both halves of the `checked` ⇔ `checked_against` pair are enforced here, so it can never
/// drift: `checked_against` is forced to `None` when `checked` is off, and a check with no
/// token is `Shopping(CheckWithoutQuantity)` rather than the table CHECK's untyped failure.
/// A blank key is `Shopping(BlankKey)`. Both rejections land before any write.
pub fn set_shopping_line_state(
    conn: &mut Connection,
    household: &HouseholdId,
    from: CivilDate,
    to: CivilDate,
    state: &ShoppingLineState,
) -> Result<ShoppingLineState, StorageError> {
    if state.key.trim().is_empty() {
        return Err(StorageError::Shopping(ShoppingError::BlankKey));
    }
    if state.checked && state.checked_against.is_none() {
        return Err(StorageError::Shopping(ShoppingError::CheckWithoutQuantity));
    }
    let stored = ShoppingLineState {
        key: state.key.clone(),
        checked: state.checked,
        checked_against: if state.checked {
            state.checked_against.clone()
        } else {
            None
        },
        hidden: state.hidden,
        restored: state.restored,
    };
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, household)?;
    let (from, to) = (format_civil_date(from), format_civil_date(to));
    if stored.is_blank() {
        tx.execute(
            "DELETE FROM shopping_line_state
             WHERE household_id = ?1 AND from_date = ?2 AND to_date = ?3 AND line_key = ?4",
            params![household.as_str(), from, to, stored.key],
        )?;
    } else {
        tx.execute(
            "INSERT INTO shopping_line_state
             (household_id, from_date, to_date, line_key, checked, checked_against, hidden,
              restored)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(household_id, from_date, to_date, line_key) DO UPDATE SET
                 checked = excluded.checked,
                 checked_against = excluded.checked_against,
                 hidden = excluded.hidden,
                 restored = excluded.restored",
            params![
                household.as_str(),
                from,
                to,
                stored.key,
                stored.checked,
                stored.checked_against,
                stored.hidden,
                stored.restored,
            ],
        )?;
    }
    tx.commit()?;
    Ok(stored)
}

/// A household-owned item the user typed rather than a recipe derived (MVP-016). Fully
/// editable, since nothing derives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShoppingManualItem {
    pub id: ShoppingManualItemId,
    pub household_id: HouseholdId,
    pub name: String,
    pub note: Option<String>,
    pub checked: bool,
}

/// Exactly this household's manual items, case-insensitively by name then id — the pantry's
/// collation, for the pantry's reason.
pub fn list_shopping_manual_items(
    conn: &Connection,
    household: &HouseholdId,
) -> Result<Vec<ShoppingManualItem>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, note, checked FROM shopping_manual_item
         WHERE household_id = ?1 ORDER BY name COLLATE NOCASE, id",
    )?;
    let rows = stmt
        .query_map(params![household.as_str()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, bool>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|(id, name, note, checked)| {
            Ok(ShoppingManualItem {
                id: ShoppingManualItemId::new(id)?,
                household_id: household.clone(),
                name,
                note,
                checked,
            })
        })
        .collect()
}

/// Inserts or wholly replaces the item and returns what was stored: the name trimmed (blank
/// is `Shopping(BlankName)` before any write) and a blank note mapped to `None`. An existing
/// id owned by another household is `NoSuchShoppingItem`, never hijacked.
pub fn save_shopping_manual_item(
    conn: &mut Connection,
    item: &ShoppingManualItem,
) -> Result<ShoppingManualItem, StorageError> {
    let name = item.name.trim();
    if name.is_empty() {
        return Err(StorageError::Shopping(ShoppingError::BlankName));
    }
    let stored = ShoppingManualItem {
        id: item.id.clone(),
        household_id: item.household_id.clone(),
        name: name.to_owned(),
        note: item
            .note
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(str::to_owned),
        checked: item.checked,
    };
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, &stored.household_id)?;
    let owner: Option<String> = tx
        .query_row(
            "SELECT household_id FROM shopping_manual_item WHERE id = ?1",
            params![stored.id.as_str()],
            |r| r.get(0),
        )
        .optional()?;
    if owner.is_some_and(|o| o != stored.household_id.as_str()) {
        return Err(StorageError::NoSuchShoppingItem(
            stored.id.as_str().to_owned(),
        ));
    }
    tx.execute(
        "INSERT INTO shopping_manual_item (id, household_id, name, note, checked)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name, note = excluded.note, checked = excluded.checked",
        params![
            stored.id.as_str(),
            stored.household_id.as_str(),
            stored.name,
            stored.note,
            stored.checked,
        ],
    )?;
    tx.commit()?;
    Ok(stored)
}

/// Removes exactly this household's item; absent or foreign is `NoSuchShoppingItem`.
pub fn delete_shopping_manual_item(
    conn: &mut Connection,
    household: &HouseholdId,
    id: &ShoppingManualItemId,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let changed = tx.execute(
        "DELETE FROM shopping_manual_item WHERE id = ?1 AND household_id = ?2",
        params![id.as_str(), household.as_str()],
    )?;
    if changed == 0 {
        return Err(StorageError::NoSuchShoppingItem(id.as_str().to_owned()));
    }
    tx.commit()?;
    Ok(())
}

/// "Start over" for one window, in one transaction: every line state of that window goes,
/// and every *checked* manual item goes. Unchecked manual items carry — they were not
/// bought, and the reset is about this trip, not the household's list of wants.
pub fn reset_shopping_list(
    conn: &mut Connection,
    household: &HouseholdId,
    from: CivilDate,
    to: CivilDate,
) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, household)?;
    tx.execute(
        "DELETE FROM shopping_line_state
         WHERE household_id = ?1 AND from_date = ?2 AND to_date = ?3",
        params![
            household.as_str(),
            format_civil_date(from),
            format_civil_date(to)
        ],
    )?;
    tx.execute(
        "DELETE FROM shopping_manual_item WHERE household_id = ?1 AND checked = 1",
        params![household.as_str()],
    )?;
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
    fn an_existing_v1_database_migrates_to_v10_without_losing_data() {
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
        assert_eq!(schema_version(&conn).unwrap(), 10);
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
    fn empty_db_migrates_to_v10() {
        let conn = open(":memory:").unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 10);
    }

    // --- Step 4: schema v3 -----------------------------------------------------------------

    /// Test-level stand-in for the on-device v2→v3 migration, as the v1 test is for v1→v2.
    #[test]
    fn an_existing_v2_database_migrates_to_v10_without_losing_data() {
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
        assert_eq!(schema_version(&conn).unwrap(), 10);
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
    fn an_existing_v3_database_migrates_to_v10_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 3).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 3);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            insert_pre_v6_recipe(&raw, "h", "r");
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 10);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "household_member"), 1);
        assert_eq!(count(&conn, "recipe"), 1);
        assert_eq!(count(&conn, "household_restriction"), 0);
        assert_eq!(count(&conn, "member_food_preference"), 0);
    }

    // --- schema v5 -------------------------------------------------------------------------

    /// Test-level stand-in for the on-device v4→latest migration, in the pattern of the v3→v4
    /// test: a recipe saved at v4 must survive the `ALTER TABLE`s that add `archived_at` (v5)
    /// and `prep_minutes`/the rights columns (v6).
    #[test]
    fn an_existing_v4_database_migrates_to_v10_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 4).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 4);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            insert_pre_v6_recipe(&raw, "h", "r");
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 10);
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
            insert_pre_v6_recipe(&raw, "h", "r");
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

    fn text_line(name: &str) -> IngredientLine {
        line(name, name, Quantity::Unknown, Unit::None)
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

    /// Writes a minimal recipe with the columns a **pre-v6** `recipe` table has. `save_recipe`
    /// is a latest-schema writer — it writes `prep_minutes` — so a migration test that stands
    /// a database up at v3/v4 cannot use it to place the row it is about to migrate.
    fn insert_pre_v6_recipe(conn: &Connection, household: &str, id: &str) {
        conn.execute(
            "INSERT INTO recipe (id, household_id, title, servings, instructions)
             VALUES (?1, ?2, ?3, 4, 'Cook it.')",
            params![id, household, format!("Recipe {id}")],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO recipe_provenance (recipe_id, kind) VALUES (?1, 'authored')",
            params![id],
        )
        .unwrap();
    }

    fn recipe(household: &str, id: &str, lines: Vec<IngredientLine>) -> Recipe {
        Recipe::new(
            RecipeId::new(id).unwrap(),
            HouseholdId::new(household).unwrap(),
            format!("Recipe {id}"),
            Some(4),
            None,
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
                    line_names: vec![],
                },
                RecipeSummary {
                    id: RecipeId::new("r3").unwrap(),
                    title: "Apple pie".to_owned(),
                    line_names: vec![],
                },
                RecipeSummary {
                    id: RecipeId::new("r1").unwrap(),
                    title: "Zucchini bake".to_owned(),
                    line_names: vec![],
                },
            ]
        );
    }

    // --- MVP-009: line names on summaries -------------------------------------------------

    fn line_names(conn: &Connection, household: &str, listing: RecipeListing) -> Vec<Vec<String>> {
        list_recipes(conn, &HouseholdId::new(household).unwrap(), listing)
            .unwrap()
            .into_iter()
            .map(|s| s.line_names)
            .collect()
    }

    #[test]
    fn list_recipes_carries_line_names_in_position_order() {
        let mut conn = open(":memory:").unwrap();
        seed_for_recipes(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", three_lines())).unwrap();
        assert_eq!(
            line_names(&conn, "h", RecipeListing::Active),
            vec![vec!["flour", "nana's mix", "something"]]
        );
    }

    /// Two households and an archived recipe: names never cross a household or the archive
    /// marker, and a listing's names line up with its own summaries.
    #[test]
    fn line_names_are_household_and_listing_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed_for_recipes(&mut conn, "h1");
        seed(&mut conn, "h2");
        save_recipe(&mut conn, &recipe("h1", "a", vec![text_line("butter")])).unwrap();
        save_recipe(&mut conn, &recipe("h1", "b", vec![text_line("peanuts")])).unwrap();
        save_recipe(&mut conn, &recipe("h2", "c", vec![text_line("shrimp")])).unwrap();
        archive_recipe(
            &mut conn,
            &HouseholdId::new("h1").unwrap(),
            &RecipeId::new("b").unwrap(),
            parse_civil_date("2026-08-29").unwrap(),
        )
        .unwrap();
        assert_eq!(
            line_names(&conn, "h1", RecipeListing::Active),
            vec![vec!["butter"]]
        );
        assert_eq!(
            line_names(&conn, "h1", RecipeListing::Archived),
            vec![vec!["peanuts"]]
        );
        assert_eq!(
            line_names(&conn, "h2", RecipeListing::Active),
            vec![vec!["shrimp"]]
        );
        assert!(line_names(&conn, "h2", RecipeListing::Archived).is_empty());
    }

    #[test]
    fn a_stub_recipe_has_no_line_names() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        assert_eq!(
            line_names(&conn, "h", RecipeListing::Active),
            vec![Vec::<String>::new()]
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
        assert_eq!(schema_version(&conn).unwrap(), 10);
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

    /// The reviewed marker, enabled, for `id`.
    fn reviewed_marker(id: &str) -> Policy {
        Policy::new(
            PolicyId::new(format!("{id}:food.restrictions_reviewed")).unwrap(),
            hid(id),
            "food",
            planner::FoodPolicies::RESTRICTIONS_REVIEWED,
            std::collections::BTreeMap::new(),
            true,
            EvidenceSource::ExplicitUser,
        )
        .unwrap()
    }

    fn marker_is_live(conn: &Connection, id: &str) -> bool {
        planner::FoodPolicies::from_policies(&list_policies(conn, &hid(id), "food").unwrap())
            .restrictions_reviewed
    }

    /// A review answers "is this set complete?" about the set that was reviewed, so changing
    /// the set retires the answer in the same transaction.
    #[test]
    fn changing_the_restriction_set_retires_the_reviewed_marker() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_policy(&mut conn, &reviewed_marker("h")).unwrap();
        assert!(marker_is_live(&conn, "h"));
        save_restrictions(&mut conn, &hid("h"), &mixed_set()).unwrap();
        assert!(!marker_is_live(&conn, "h"));
    }

    /// The latch the marker used to be: reviewed while empty, a restriction added, then
    /// removed again. The set is empty exactly as it was, but nobody has confirmed *this*
    /// emptiness, so `RESTRICTIONS_NOT_CONFIGURED` must be free to fire again.
    #[test]
    fn an_emptied_restriction_set_does_not_read_as_reviewed() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_restrictions(&mut conn, &hid("h"), &HouseholdRestrictions::default()).unwrap();
        save_policy(&mut conn, &reviewed_marker("h")).unwrap();
        let peanuts = HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Peanuts)]);
        save_restrictions(&mut conn, &hid("h"), &peanuts).unwrap();
        save_restrictions(&mut conn, &hid("h"), &HouseholdRestrictions::default()).unwrap();
        assert!(load_restrictions(&conn, &hid("h"))
            .unwrap()
            .restrictions()
            .is_empty());
        assert!(!marker_is_live(&conn, "h"));
    }

    /// Expected-to-pass: re-saving the same set is not a change, so the marker survives —
    /// otherwise opening the Restrictions screen and pressing Save would re-ask a question
    /// the household just answered. And the clear is household-scoped.
    #[test]
    fn an_unchanged_save_keeps_the_marker_and_never_crosses_households() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        save_restrictions(&mut conn, &hid("h1"), &mixed_set()).unwrap();
        save_policy(&mut conn, &reviewed_marker("h1")).unwrap();
        save_policy(&mut conn, &reviewed_marker("h2")).unwrap();
        save_restrictions(&mut conn, &hid("h1"), &mixed_set()).unwrap();
        assert!(
            marker_is_live(&conn, "h1"),
            "an identical set is not a change"
        );
        save_restrictions(&mut conn, &hid("h1"), &HouseholdRestrictions::default()).unwrap();
        assert!(!marker_is_live(&conn, "h1"));
        assert!(marker_is_live(&conn, "h2"), "h2 said nothing");
    }

    /// A row a newer build wrote reads back as `CorruptRestriction` (`rust/src/api/recipe.rs`
    /// seeds exactly this shape), and replacing the whole set is the only operation that can
    /// clear it. The marker-retire guard must therefore not read the stored set through
    /// `restriction_from_row`, or the repair path closes on the state it exists to repair.
    #[test]
    fn a_restriction_row_this_build_cannot_read_can_still_be_replaced() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_restriction(&conn, "h", "sulphites", None);
        assert!(
            load_restrictions(&conn, &hid("h")).is_err(),
            "the starting state is the unreadable one"
        );
        save_restrictions(&mut conn, &hid("h"), &mixed_set()).unwrap();
        assert_eq!(load_restrictions(&conn, &hid("h")).unwrap(), mixed_set());
    }

    /// An unreadable row is not the set being written, so it counts as a change and the
    /// reviewed marker retires — the safe direction: re-asking whether the set is complete,
    /// never suppressing the question for a state nobody confirmed.
    #[test]
    fn replacing_an_unreadable_row_retires_the_reviewed_marker() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        raw_restriction(&conn, "h", "sulphites", None);
        save_policy(&mut conn, &reviewed_marker("h")).unwrap();
        assert!(marker_is_live(&conn, "h"));
        save_restrictions(&mut conn, &hid("h"), &HouseholdRestrictions::default()).unwrap();
        assert!(load_restrictions(&conn, &hid("h"))
            .unwrap()
            .restrictions()
            .is_empty());
        assert!(!marker_is_live(&conn, "h"));
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

    // --- MVP-011 steps 4-6: schema v6, rights persistence, starter install ---------------

    fn load_record_err(conn: &Connection, household: &str, id: &str) -> StorageError {
        load_recipe(
            conn,
            &HouseholdId::new(household).unwrap(),
            &RecipeId::new(id).unwrap(),
        )
        .unwrap_err()
    }

    fn rights(basis: RightsBasis, attribution: Option<&str>) -> RecipeRights {
        RecipeRights::new(
            basis,
            attribution.map(str::to_owned),
            Some("halved the salt".to_owned()),
            "2026-08-29",
        )
        .unwrap()
    }

    fn starter_provenance(slug: &str, rights: Option<RecipeRights>) -> RecipeProvenance {
        RecipeProvenance::with_rights(
            ProvenanceKind::Starter,
            None,
            None,
            Some("Kimatta".to_owned()),
            rights,
            Some(slug.to_owned()),
        )
        .unwrap()
    }

    fn starter_recipe(household: &str, id: &str, slug: &str, catalog_id: &str) -> Recipe {
        let line = IngredientLine::new(
            "1 tbsp olive oil",
            "olive oil",
            Some(IngredientRef::Catalog(
                IngredientId::new(catalog_id).unwrap(),
            )),
            Quantity::Exact(Rational::new(1, 1).unwrap()),
            Unit::Known(UnitKind::Tablespoon),
            None,
            false,
        )
        .unwrap();
        Recipe::new(
            RecipeId::new(id).unwrap(),
            HouseholdId::new(household).unwrap(),
            format!("Starter {slug}"),
            Some(2),
            Some(15),
            "Cook.",
            vec![line],
            starter_provenance(slug, Some(rights(RightsBasis::Original, Some("Kimatta")))),
        )
        .unwrap()
    }

    fn set_provenance_column(conn: &Connection, recipe: &str, column: &str, value: Option<&str>) {
        conn.execute(
            &format!("UPDATE recipe_provenance SET {column} = ?1 WHERE recipe_id = ?2"),
            params![value, recipe],
        )
        .unwrap();
    }

    #[test]
    fn a_migrated_recipe_has_no_prep_estimate_and_no_rights() {
        // No backfill: a recipe written before v6 has genuinely never had a prep estimate,
        // and must not acquire a fabricated rights row.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 5).unwrap();
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            insert_pre_v6_recipe(&raw, "h", "r");
        }
        let conn = open(&path).unwrap();
        let loaded = load(&conn, "h", "r").unwrap();
        assert_eq!(loaded.prep_minutes(), None);
        assert_eq!(loaded.provenance().rights(), None);
        assert_eq!(loaded.provenance().starter_slug(), None);
    }

    #[test]
    fn an_existing_v5_database_migrates_to_v10_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 5).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 5);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            insert_pre_v6_recipe(&raw, "h", "r");
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 10);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "recipe"), 1);
        assert_eq!(load(&conn, "h", "r").unwrap().title(), "Recipe r");
    }

    // --- MVP-012 step 2: schema v7, planned meals ---------------------------------------

    const RAW_PLANNED_MEAL: &str = "INSERT INTO planned_meal (id, household_id, date, slot)
        VALUES (?1, ?2, ?3, ?4)";
    const RAW_COMPONENT: &str = "INSERT INTO meal_component
        (planned_meal_id, position, kind, recipe_id, note, scale_numer, scale_denom)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

    /// A household, a recipe and one occurrence row, inserted raw so the component-level
    /// constraints below are the only thing under test.
    fn raw_occurrence(conn: &mut Connection) {
        seed(conn, "h");
        save_recipe(conn, &recipe("h", "r", vec![])).unwrap();
        conn.execute(RAW_PLANNED_MEAL, params!["pm", "h", "2026-08-29", "dinner"])
            .unwrap();
    }

    /// Test-level stand-in for the on-device v6→v7 migration, in the pattern of its
    /// predecessors: a recipe saved at v6 must survive the two new tables, which touch nothing
    /// that exists.
    #[test]
    fn an_existing_v6_database_migrates_to_v10_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 6).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 6);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            save_recipe(&mut raw, &recipe("h", "r", vec![])).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 10);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "recipe"), 1);
        assert_eq!(load(&conn, "h", "r").unwrap().title(), "Recipe r");
        assert_eq!(count(&conn, "planned_meal"), 0);
        assert_eq!(count(&conn, "meal_component"), 0);
    }

    /// Absent-at-6 then present-at-7, for the reason `the_line_ingredient_foreign_keys_are_indexed`
    /// gives: the claim is that migration 7 ships the indexes, not that the latest schema has
    /// them.
    #[test]
    fn the_component_recipe_foreign_key_is_indexed() {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_version(&mut conn, 6).unwrap();
        assert!(index_names(&conn, "meal_component").is_empty());
        assert!(index_names(&conn, "planned_meal").is_empty());
        MIGRATIONS.to_version(&mut conn, 7).unwrap();
        assert!(index_names(&conn, "meal_component").contains(&"meal_component_recipe".to_owned()));
        assert!(
            index_names(&conn, "planned_meal").contains(&"planned_meal_household_date".to_owned())
        );
    }

    fn index_names(conn: &Connection, table: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND tbl_name = ?1")
            .unwrap();
        let names = stmt
            .query_map(params![table], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        names
    }

    #[test]
    fn a_second_occurrence_in_the_same_slot_is_refused_by_the_unique_constraint() {
        let mut conn = open(":memory:").unwrap();
        raw_occurrence(&mut conn);
        let err = conn
            .execute(
                RAW_PLANNED_MEAL,
                params!["pm2", "h", "2026-08-29", "dinner"],
            )
            .unwrap_err();
        assert_constraint_violation(err);
        // The same date in another slot, and the same slot on another date, are both fine.
        conn.execute(RAW_PLANNED_MEAL, params!["pm3", "h", "2026-08-29", "lunch"])
            .unwrap();
        conn.execute(
            RAW_PLANNED_MEAL,
            params!["pm4", "h", "2026-08-30", "dinner"],
        )
        .unwrap();
        assert_eq!(count(&conn, "planned_meal"), 3);
    }

    #[test]
    fn a_recipe_component_without_a_recipe_id_is_refused_by_the_check() {
        let mut conn = open(":memory:").unwrap();
        raw_occurrence(&mut conn);
        let none = Option::<String>::None;
        let err = conn
            .execute(
                RAW_COMPONENT,
                params![
                    "pm",
                    0,
                    "recipe",
                    none,
                    none,
                    Option::<u32>::None,
                    Option::<u32>::None
                ],
            )
            .unwrap_err();
        assert_constraint_violation(err);
        // The mirror: a non-recipe row carrying a recipe reference.
        let err = conn
            .execute(
                RAW_COMPONENT,
                params![
                    "pm",
                    0,
                    "leftovers",
                    "r",
                    none,
                    Option::<u32>::None,
                    Option::<u32>::None
                ],
            )
            .unwrap_err();
        assert_constraint_violation(err);
        assert_eq!(count(&conn, "meal_component"), 0);
    }

    #[test]
    fn a_half_present_scale_is_refused_by_the_check() {
        let mut conn = open(":memory:").unwrap();
        raw_occurrence(&mut conn);
        let none = Option::<String>::None;
        for (numer, denom) in [
            (Some(1), None),
            (None, Some(2)),
            (Some(0), Some(1)),
            (Some(1), Some(0)),
        ] {
            let err = conn
                .execute(
                    RAW_COMPONENT,
                    params!["pm", 0, "recipe", "r", none, numer, denom],
                )
                .unwrap_err();
            assert_constraint_violation(err);
        }
        conn.execute(RAW_COMPONENT, params!["pm", 0, "recipe", "r", none, 3, 2])
            .unwrap();
        assert_eq!(count(&conn, "meal_component"), 1);
    }

    #[test]
    fn a_scaled_non_recipe_component_is_refused_by_the_check() {
        let mut conn = open(":memory:").unwrap();
        raw_occurrence(&mut conn);
        let none = Option::<String>::None;
        let err = conn
            .execute(
                RAW_COMPONENT,
                params!["pm", 0, "leftovers", none, "chili", 1, 2],
            )
            .unwrap_err();
        assert_constraint_violation(err);
        // And the other direction: a recipe row carrying a note.
        let err = conn
            .execute(
                RAW_COMPONENT,
                params![
                    "pm",
                    0,
                    "recipe",
                    "r",
                    "x",
                    Option::<u32>::None,
                    Option::<u32>::None
                ],
            )
            .unwrap_err();
        assert_constraint_violation(err);
        assert_eq!(count(&conn, "meal_component"), 0);
    }

    /// The one CHECK that constrains a note's *value* rather than the kind/column pairing,
    /// because `freeform` is the one kind whose note is not optional. Without it a blank note
    /// is writable and then unreadable — `parse_row` routes freeform through `freeform()`,
    /// which refuses it, and one such row fails a whole date range. The set is ASCII
    /// whitespace, wider than SQLite's space-only `trim` default and still narrower than the
    /// `str::trim` the reader applies, which is why `check_freeform_notes` guards in Rust too.
    #[test]
    fn a_blank_freeform_note_is_refused_by_the_check() {
        let mut conn = open(":memory:").unwrap();
        raw_occurrence(&mut conn);
        let none = Option::<String>::None;
        let no_scale = (Option::<u32>::None, Option::<u32>::None);
        for blank in ["", " ", "\t", "\n", "\r", "\u{b}", "\u{c}", " \t\r\n "] {
            let err = conn
                .execute(
                    RAW_COMPONENT,
                    params!["pm", 0, "freeform", none, blank, no_scale.0, no_scale.1],
                )
                .unwrap_err();
            assert_constraint_violation(err);
        }
        // A NULL note under the same kind is refused too: `parse_row` reports that shape as
        // corrupt rather than coercing it.
        let err = conn
            .execute(
                RAW_COMPONENT,
                params!["pm", 0, "freeform", none, none, no_scale.0, no_scale.1],
            )
            .unwrap_err();
        assert_constraint_violation(err);
        // The note is checked, never trimmed: surrounding whitespace is stored verbatim.
        conn.execute(
            RAW_COMPONENT,
            params!["pm", 0, "freeform", none, " pizza ", no_scale.0, no_scale.1],
        )
        .unwrap();
        let stored: String = conn
            .query_row("SELECT note FROM meal_component", [], |r| r.get(0))
            .unwrap();
        assert_eq!(stored, " pizza ");
        // Only `freeform` is constrained this way; the optional-note kinds are untouched.
        conn.execute(
            RAW_COMPONENT,
            params!["pm", 1, "leftovers", none, "", no_scale.0, no_scale.1],
        )
        .unwrap();
        assert_eq!(count(&conn, "meal_component"), 2);
    }

    // --- MVP-012 steps 3–4: planned meal storage, lock contract, scoping -----------------

    fn pmid(raw: &str) -> PlannedMealId {
        PlannedMealId::new(raw).unwrap()
    }

    fn rid(raw: &str) -> RecipeId {
        RecipeId::new(raw).unwrap()
    }

    fn scaled(recipe: &str, numer: u32, denom: u32) -> MealComponent {
        MealComponent::recipe(rid(recipe), Some(rat(numer, denom)))
    }

    fn leftovers(note: &str) -> MealComponent {
        MealComponent::Leftovers {
            note: Some(note.to_owned()),
        }
    }

    fn occurrence(
        household: &str,
        id: &str,
        date: &str,
        slot: MealSlot,
        components: Vec<MealComponent>,
    ) -> PlannedMeal {
        PlannedMeal::new(
            pmid(id),
            hid(household),
            parse_civil_date(date).unwrap(),
            slot,
            components,
            false,
        )
        .unwrap()
    }

    /// A household with a lunch+dinner cycle and one recipe `r-<household>`, so a meal test
    /// never trips the cycle or recipe checks unless that is what it is testing.
    fn seed_for_meals(conn: &mut Connection, household: &str) {
        seed(conn, household);
        save_planning_cycle(
            conn,
            &cycle(
                household,
                "2026-08-29",
                7,
                &[MealSlot::Lunch, MealSlot::Dinner],
            ),
        )
        .unwrap();
        save_recipe(conn, &recipe(household, &format!("r-{household}"), vec![])).unwrap();
    }

    /// A typical dinner for `h`: a scaled recipe, an as-written recipe and leftovers.
    fn dinner(id: &str, date: &str) -> PlannedMeal {
        occurrence(
            "h",
            id,
            date,
            MealSlot::Dinner,
            vec![
                scaled("r-h", 3, 2),
                MealComponent::recipe(rid("r-h"), None),
                leftovers("chili"),
            ],
        )
    }

    fn load_meal(conn: &Connection, household: &str, id: &str) -> Option<PlannedMeal> {
        load_planned_meal(conn, &hid(household), &pmid(id)).unwrap()
    }

    fn user_save(conn: &mut Connection, meal: &PlannedMeal) -> Result<(), StorageError> {
        save_planned_meal(conn, meal, WriteSource::User)
    }

    fn lock(
        conn: &mut Connection,
        household: &str,
        id: &str,
        locked: bool,
        source: WriteSource,
    ) -> Result<(), StorageError> {
        set_planned_meal_lock(conn, &hid(household), &pmid(id), locked, source)
    }

    fn locked_raw(conn: &Connection, id: &str) -> bool {
        conn.query_row(
            "SELECT locked FROM planned_meal WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn listed(conn: &Connection, household: &str, from: &str, to: &str) -> Vec<(String, MealSlot)> {
        list_planned_meals(
            conn,
            &hid(household),
            parse_civil_date(from).unwrap(),
            parse_civil_date(to).unwrap(),
        )
        .unwrap()
        .iter()
        .map(|m| (format_civil_date(m.date()), m.slot()))
        .collect()
    }

    /// AC-1.
    #[test]
    fn a_multi_component_occurrence_round_trips_by_date_and_slot() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let meal = dinner("pm", "2026-08-30");
        user_save(&mut conn, &meal).unwrap();
        assert_eq!(load_meal(&conn, "h", "pm"), Some(meal));
        assert_eq!(count(&conn, "meal_component"), 3);
    }

    /// AC-1: every non-recipe kind is a kind token, never an absent recipe (stop condition).
    #[test]
    fn every_non_recipe_kind_round_trips() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let mixed = occurrence(
            "h",
            "pm",
            "2026-08-29",
            MealSlot::Dinner,
            vec![
                leftovers(" chili "),
                MealComponent::DiningOut { note: None },
                MealComponent::FrozenQuick {
                    note: Some("pierogi".to_owned()),
                },
                MealComponent::freeform(" pizza night ").unwrap(),
            ],
        );
        let open = occurrence(
            "h",
            "pm-open",
            "2026-08-29",
            MealSlot::Lunch,
            vec![MealComponent::Open {
                note: Some("away".to_owned()),
            }],
        );
        user_save(&mut conn, &mixed).unwrap();
        user_save(&mut conn, &open).unwrap();
        assert_eq!(load_meal(&conn, "h", "pm"), Some(mixed));
        assert_eq!(load_meal(&conn, "h", "pm-open"), Some(open));
        let null_recipes: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM meal_component WHERE recipe_id IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(null_recipes, 5);
        let kinds: Vec<String> = conn
            .prepare("SELECT DISTINCT kind FROM meal_component ORDER BY kind")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            kinds,
            [
                "dining_out",
                "freeform",
                "frozen_quick",
                "leftovers",
                "open"
            ]
        );
    }

    /// AC-2: a reorder survives a re-save and each scale follows its recipe.
    #[test]
    fn save_replaces_the_whole_component_set_and_keeps_order() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let mut meal = dinner("pm", "2026-08-30");
        user_save(&mut conn, &meal).unwrap();
        meal.move_component(2, 0).unwrap();
        meal.set_scale(2, Some(rat(1, 4))).unwrap();
        user_save(&mut conn, &meal).unwrap();
        let loaded = load_meal(&conn, "h", "pm").unwrap();
        assert_eq!(
            loaded.components(),
            &[leftovers("chili"), scaled("r-h", 3, 2), scaled("r-h", 1, 4),]
        );
        assert_eq!(count(&conn, "meal_component"), 3);
    }

    #[test]
    fn scales_are_stored_in_lowest_terms() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let meal = occurrence(
            "h",
            "pm",
            "2026-08-29",
            MealSlot::Dinner,
            vec![scaled("r-h", 2, 4)],
        );
        user_save(&mut conn, &meal).unwrap();
        let stored: (u32, u32) = conn
            .query_row(
                "SELECT scale_numer, scale_denom FROM meal_component WHERE planned_meal_id = 'pm'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(stored, (1, 2));
    }

    #[test]
    fn list_returns_an_inclusive_date_range_in_date_then_slot_order() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        // Inserted out of order so the read cannot be passing by rowid.
        for (id, date, slot) in [
            ("a", "2026-08-30", MealSlot::Dinner),
            ("b", "2026-08-30", MealSlot::Lunch),
            ("c", "2026-08-29", MealSlot::Dinner),
            ("d", "2026-08-31", MealSlot::Lunch),
            ("e", "2026-09-01", MealSlot::Dinner),
            ("f", "2026-08-28", MealSlot::Dinner),
        ] {
            user_save(
                &mut conn,
                &occurrence("h", id, date, slot, vec![leftovers(id)]),
            )
            .unwrap();
        }
        assert_eq!(
            listed(&conn, "h", "2026-08-29", "2026-08-31"),
            [
                ("2026-08-29".to_owned(), MealSlot::Dinner),
                ("2026-08-30".to_owned(), MealSlot::Lunch),
                ("2026-08-30".to_owned(), MealSlot::Dinner),
                ("2026-08-31".to_owned(), MealSlot::Lunch),
            ]
        );
        // Components ride along from the one listing query.
        let all = list_planned_meals(
            &conn,
            &hid("h"),
            parse_civil_date("2026-08-29").unwrap(),
            parse_civil_date("2026-08-31").unwrap(),
        )
        .unwrap();
        assert_eq!(all[1].components(), &[leftovers("b")]);
    }

    #[test]
    fn list_of_an_empty_range_is_empty() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        assert!(listed(&conn, "h", "2026-09-01", "2026-09-07").is_empty());
        // An inverted range is simply empty, not an error.
        assert!(listed(&conn, "h", "2026-08-31", "2026-08-29").is_empty());
    }

    /// AC-4.
    #[test]
    fn a_component_naming_another_households_recipe_is_rejected_before_any_write() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        seed_for_meals(&mut conn, "h2");
        let meal = occurrence(
            "h",
            "pm",
            "2026-08-29",
            MealSlot::Dinner,
            vec![leftovers("x"), MealComponent::recipe(rid("r-h2"), None)],
        );
        let err = user_save(&mut conn, &meal).unwrap_err();
        assert!(
            matches!(&err, StorageError::ComponentRecipeMismatch { recipe, expected, actual }
                if recipe == "r-h2" && expected == "h" && actual == "h2"),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "planned_meal"), 0);
        assert_eq!(count(&conn, "meal_component"), 0);
        let ghost = occurrence(
            "h",
            "pm",
            "2026-08-29",
            MealSlot::Dinner,
            vec![MealComponent::recipe(rid("ghost"), None)],
        );
        assert!(matches!(
            user_save(&mut conn, &ghost).unwrap_err(),
            StorageError::NoSuchRecipe { .. }
        ));
        assert_eq!(count(&conn, "planned_meal"), 0);
    }

    /// Discharges the half of MVP-008's archive policy that had only a proxy until now.
    #[test]
    fn an_archived_recipe_is_still_a_valid_component() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let meal = dinner("pm", "2026-08-30");
        user_save(&mut conn, &meal).unwrap();
        archive(&mut conn, "h", "r-h", "2026-08-30").unwrap();
        assert_eq!(load_meal(&conn, "h", "pm"), Some(meal.clone()));
        // And a new occurrence may still name it.
        user_save(&mut conn, &dinner("pm2", "2026-08-31")).unwrap();
        assert_eq!(count(&conn, "planned_meal"), 2);
    }

    #[test]
    fn a_slot_outside_the_enabled_scope_is_rejected_on_write() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let meal = occurrence(
            "h",
            "pm",
            "2026-08-29",
            MealSlot::Breakfast,
            vec![leftovers("x")],
        );
        let err = user_save(&mut conn, &meal).unwrap_err();
        assert!(
            matches!(&err, StorageError::SlotNotEnabled { household, slot }
                if household == "h" && slot == "breakfast"),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "planned_meal"), 0);
    }

    /// The write-time-only half of the slot rule.
    #[test]
    fn disabling_a_slot_later_keeps_existing_occurrences_readable() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let lunch = occurrence(
            "h",
            "pm",
            "2026-08-29",
            MealSlot::Lunch,
            vec![leftovers("x")],
        );
        user_save(&mut conn, &lunch).unwrap();
        save_planning_cycle(&mut conn, &cycle("h", "2026-08-29", 7, &[MealSlot::Dinner])).unwrap();
        assert_eq!(load_meal(&conn, "h", "pm"), Some(lunch.clone()));
        assert_eq!(
            listed(&conn, "h", "2026-08-29", "2026-08-29"),
            [("2026-08-29".to_owned(), MealSlot::Lunch)]
        );
        // A re-save into the now-disabled slot is refused; the stored row is untouched.
        assert!(matches!(
            user_save(&mut conn, &lunch).unwrap_err(),
            StorageError::SlotNotEnabled { .. }
        ));
        assert_eq!(load_meal(&conn, "h", "pm"), Some(lunch));
    }

    #[test]
    fn a_household_with_no_cycle_cannot_plan_yet() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let meal = occurrence(
            "h",
            "pm",
            "2026-08-29",
            MealSlot::Dinner,
            vec![leftovers("x")],
        );
        let err = user_save(&mut conn, &meal).unwrap_err();
        assert!(
            matches!(&err, StorageError::NoPlanningCycle(h) if h == "h"),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "planned_meal"), 0);
    }

    #[test]
    fn saving_a_new_id_into_an_occupied_slot_is_occupied_slot() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        let err = user_save(&mut conn, &dinner("pm2", "2026-08-30")).unwrap_err();
        assert!(
            matches!(&err, StorageError::OccupiedSlot { household, date, slot }
                if household == "h" && date == "2026-08-30" && slot == "dinner"),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "planned_meal"), 1);
        // Re-saving the holder itself into its own slot is an update, not a conflict.
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        assert_eq!(count(&conn, "planned_meal"), 1);
    }

    /// Reaches a shape the DDL CHECKs forbid, so the reader's own refusal is what is proved.
    #[test]
    fn a_corrupt_component_row_is_reported_with_its_coordinates() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        conn.pragma_update(None, "ignore_check_constraints", "ON")
            .unwrap();
        conn.execute(
            "UPDATE meal_component SET scale_denom = NULL WHERE planned_meal_id = 'pm' AND position = 0",
            [],
        )
        .unwrap();
        let err = load_planned_meal(&conn, &hid("h"), &pmid("pm")).unwrap_err();
        assert!(
            matches!(&err, StorageError::CorruptComponent { meal, position: 0, kind, recipe_id, scale_numer: Some(3), scale_denom: None }
                if meal == "pm" && kind == "recipe" && recipe_id.as_deref() == Some("r-h")),
            "got {err:?}"
        );
        conn.execute(
            "UPDATE meal_component SET scale_numer = NULL, note = 'x'
             WHERE planned_meal_id = 'pm' AND position = 0",
            [],
        )
        .unwrap();
        let err = load_planned_meal(&conn, &hid("h"), &pmid("pm")).unwrap_err();
        assert!(
            matches!(&err, StorageError::CorruptComponent { position: 0, kind, .. } if kind == "recipe"),
            "got {err:?}"
        );
        // The listing path refuses the same row rather than dropping the meal.
        assert!(list_planned_meals(
            &conn,
            &hid("h"),
            parse_civil_date("2026-08-30").unwrap(),
            parse_civil_date("2026-08-30").unwrap(),
        )
        .is_err());
    }

    /// `MealComponent`'s variants have public fields — Rust has no per-variant privacy — so a
    /// caller can hand storage a `Freeform` that `freeform()` would have refused. `Automation`
    /// is the caller this guards: MVP-023 builds domain values itself rather than through the
    /// bridge, whose `component_to_domain` already routes through `parse_row`.
    #[test]
    fn a_blank_freeform_note_is_refused_before_anything_is_written() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let meal = occurrence(
            "h",
            "pm",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::Freeform {
                note: String::new(),
            }],
        );
        let err = save_planned_meal(&mut conn, &meal, WriteSource::Automation).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::PlannedMeal(PlannedMealError::FreeformNeedsANote)
            ),
            "got {err:?}"
        );
        assert_eq!(count(&conn, "planned_meal"), 0);
        assert_eq!(count(&conn, "meal_component"), 0);
    }

    /// The guard is the domain constructor rather than a re-spelling of its rule, because
    /// Rust's `str::trim` strips more than SQLite's: none of these notes is blank to a schema
    /// `CHECK (trim(note) <> '')`, which removes spaces only, and every one of them is blank
    /// to `freeform()`. Writing the rule twice would have drifted here.
    #[test]
    fn freeform_notes_blank_only_to_rust_are_refused_too() {
        for blank in ["\t", "\n", "\u{a0}", " \t\r\n "] {
            let mut conn = open(":memory:").unwrap();
            seed_for_meals(&mut conn, "h");
            let meal = occurrence(
                "h",
                "pm",
                "2026-08-30",
                MealSlot::Dinner,
                vec![MealComponent::Freeform {
                    note: blank.to_owned(),
                }],
            );
            let err = user_save(&mut conn, &meal).unwrap_err();
            assert!(
                matches!(
                    &err,
                    StorageError::PlannedMeal(PlannedMealError::FreeformNeedsANote)
                ),
                "{blank:?}: got {err:?}"
            );
            assert_eq!(count(&conn, "meal_component"), 0);
        }
    }

    #[test]
    fn an_unknown_kind_row_is_rejected_on_read() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        conn.execute(
            "UPDATE meal_component SET kind = 'takeout' WHERE planned_meal_id = 'pm' AND position = 2",
            [],
        )
        .unwrap();
        let err = load_planned_meal(&conn, &hid("h"), &pmid("pm")).unwrap_err();
        assert!(
            matches!(&err, StorageError::PlannedMeal(PlannedMealError::UnknownComponentKind(k))
                if k == "takeout"),
            "got {err:?}"
        );
    }

    #[test]
    fn occurrence_for_an_absent_household_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        let meal = occurrence(
            "ghost",
            "pm",
            "2026-08-29",
            MealSlot::Dinner,
            vec![leftovers("x")],
        );
        assert!(matches!(
            user_save(&mut conn, &meal).unwrap_err(),
            StorageError::NoSuchHousehold(_)
        ));
        assert!(matches!(
            lock(&mut conn, "ghost", "pm", true, WriteSource::User).unwrap_err(),
            StorageError::NoSuchPlannedMeal { .. }
        ));
        assert_eq!(count(&conn, "planned_meal"), 0);
    }

    /// AC-4: h2 can neither see nor touch h1's occurrence, and h1's value is equal after every
    /// attempt.
    #[test]
    fn planned_meal_reads_and_writes_are_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        seed_for_meals(&mut conn, "h2");
        let meal = dinner("pm", "2026-08-30");
        user_save(&mut conn, &meal).unwrap();
        assert_eq!(load_meal(&conn, "h2", "pm"), None);
        assert!(listed(&conn, "h2", "2026-08-01", "2026-12-31").is_empty());
        let hijack = occurrence(
            "h2",
            "pm",
            "2026-08-30",
            MealSlot::Dinner,
            vec![leftovers("theirs")],
        );
        assert!(matches!(
            user_save(&mut conn, &hijack).unwrap_err(),
            StorageError::NoSuchPlannedMeal { .. }
        ));
        assert_eq!(load_meal(&conn, "h", "pm"), Some(meal.clone()));
        assert!(matches!(
            lock(&mut conn, "h2", "pm", true, WriteSource::User).unwrap_err(),
            StorageError::NoSuchPlannedMeal { .. }
        ));
        assert!(!locked_raw(&conn, "pm"));
        assert!(matches!(
            delete_planned_meal(&mut conn, &hid("h2"), &pmid("pm"), WriteSource::User).unwrap_err(),
            StorageError::NoSuchPlannedMeal { .. }
        ));
        assert_eq!(load_meal(&conn, "h", "pm"), Some(meal));
        assert_eq!(count(&conn, "planned_meal"), 1);
    }

    /// AC-3.
    #[test]
    fn a_user_write_to_a_locked_occurrence_succeeds() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        lock(&mut conn, "h", "pm", true, WriteSource::User).unwrap();
        let edited = occurrence(
            "h",
            "pm",
            "2026-08-30",
            MealSlot::Dinner,
            vec![leftovers("changed")],
        );
        user_save(&mut conn, &edited).unwrap();
        let loaded = load_meal(&conn, "h", "pm").unwrap();
        assert_eq!(loaded.components(), &[leftovers("changed")]);
        assert!(loaded.locked());
    }

    /// AC-3, adversarial: neither a component swap nor a delete gets through, and nothing moves.
    #[test]
    fn an_automation_write_to_a_locked_occurrence_is_refused_and_writes_nothing() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let meal = dinner("pm", "2026-08-30");
        user_save(&mut conn, &meal).unwrap();
        lock(&mut conn, "h", "pm", true, WriteSource::User).unwrap();
        let swapped = occurrence(
            "h",
            "pm",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::DiningOut { note: None }],
        );
        let err = save_planned_meal(&mut conn, &swapped, WriteSource::Automation).unwrap_err();
        assert!(
            matches!(&err, StorageError::LockedPlannedMeal(id) if id == "pm"),
            "got {err:?}"
        );
        let err = delete_planned_meal(&mut conn, &hid("h"), &pmid("pm"), WriteSource::Automation)
            .unwrap_err();
        assert!(matches!(err, StorageError::LockedPlannedMeal(_)));
        let loaded = load_meal(&conn, "h", "pm").unwrap();
        assert_eq!(loaded.components(), meal.components());
        assert!(loaded.locked());
        assert_eq!(count(&conn, "meal_component"), 3);
    }

    #[test]
    fn an_automation_write_to_an_unlocked_occurrence_succeeds() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        let swapped = occurrence(
            "h",
            "pm",
            "2026-08-30",
            MealSlot::Dinner,
            vec![MealComponent::DiningOut { note: None }],
        );
        save_planned_meal(&mut conn, &swapped, WriteSource::Automation).unwrap();
        assert_eq!(load_meal(&conn, "h", "pm"), Some(swapped));
        delete_planned_meal(&mut conn, &hid("h"), &pmid("pm"), WriteSource::Automation).unwrap();
        assert_eq!(count(&conn, "planned_meal"), 0);
    }

    #[test]
    fn unlock_then_automation_write_succeeds() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        lock(&mut conn, "h", "pm", true, WriteSource::User).unwrap();
        lock(&mut conn, "h", "pm", false, WriteSource::User).unwrap();
        let swapped = occurrence(
            "h",
            "pm",
            "2026-08-30",
            MealSlot::Dinner,
            vec![leftovers("auto")],
        );
        save_planned_meal(&mut conn, &swapped, WriteSource::Automation).unwrap();
        assert_eq!(load_meal(&conn, "h", "pm"), Some(swapped));
    }

    #[test]
    fn automation_may_create_an_occurrence_in_a_free_slot() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let meal = dinner("pm", "2026-08-30");
        save_planned_meal(&mut conn, &meal, WriteSource::Automation).unwrap();
        assert_eq!(load_meal(&conn, "h", "pm"), Some(meal));
        assert!(!locked_raw(&conn, "pm"));
    }

    #[test]
    fn lock_is_idempotent_and_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        lock(&mut conn, "h", "pm", true, WriteSource::User).unwrap();
        lock(&mut conn, "h", "pm", true, WriteSource::User).unwrap();
        assert!(locked_raw(&conn, "pm"));
        lock(&mut conn, "h", "pm", false, WriteSource::User).unwrap();
        lock(&mut conn, "h", "pm", false, WriteSource::User).unwrap();
        assert!(!locked_raw(&conn, "pm"));
        assert!(matches!(
            lock(&mut conn, "h", "absent", true, WriteSource::User).unwrap_err(),
            StorageError::NoSuchPlannedMeal { meal, household } if meal == "absent" && household == "h"
        ));
    }

    /// Closes the unlock-then-write bypass: automation may neither set nor clear a lock.
    #[test]
    fn automation_cannot_change_lock_state() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        assert!(matches!(
            lock(&mut conn, "h", "pm", true, WriteSource::Automation).unwrap_err(),
            StorageError::LockedPlannedMeal(_)
        ));
        assert!(!locked_raw(&conn, "pm"));
        lock(&mut conn, "h", "pm", true, WriteSource::User).unwrap();
        assert!(matches!(
            lock(&mut conn, "h", "pm", false, WriteSource::Automation).unwrap_err(),
            StorageError::LockedPlannedMeal(_)
        ));
        assert!(locked_raw(&conn, "pm"));
    }

    /// The re-read value, not the DTO echo, is what is asserted: the save carries `locked =
    /// false` in the value and the row stays locked.
    #[test]
    fn save_never_changes_lock_state() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        lock(&mut conn, "h", "pm", true, WriteSource::User).unwrap();
        let mut unlocked_value = dinner("pm", "2026-08-30");
        unlocked_value.set_locked(false);
        user_save(&mut conn, &unlocked_value).unwrap();
        assert!(locked_raw(&conn, "pm"));
        assert!(load_meal(&conn, "h", "pm").unwrap().locked());
    }

    #[test]
    fn a_new_occurrence_is_never_locked_on_insert() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        let mut meal = dinner("pm", "2026-08-30");
        meal.set_locked(true);
        user_save(&mut conn, &meal).unwrap();
        assert!(!locked_raw(&conn, "pm"));
        assert!(!load_meal(&conn, "h", "pm").unwrap().locked());
    }

    #[test]
    fn delete_removes_the_occurrence_and_its_components() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        user_save(&mut conn, &dinner("keep", "2026-08-31")).unwrap();
        delete_planned_meal(&mut conn, &hid("h"), &pmid("pm"), WriteSource::User).unwrap();
        assert_eq!(load_meal(&conn, "h", "pm"), None);
        assert_eq!(count(&conn, "planned_meal"), 1);
        assert_eq!(count(&conn, "meal_component"), 3);
        // The recipe the components named is untouched.
        assert_eq!(count(&conn, "recipe"), 1);
        assert!(matches!(
            delete_planned_meal(&mut conn, &hid("h"), &pmid("pm"), WriteSource::User).unwrap_err(),
            StorageError::NoSuchPlannedMeal { .. }
        ));
    }

    #[test]
    fn deleting_a_household_cascades_to_its_planned_meals() {
        let mut conn = open(":memory:").unwrap();
        seed_for_meals(&mut conn, "h");
        seed_for_meals(&mut conn, "h2");
        user_save(&mut conn, &dinner("pm", "2026-08-30")).unwrap();
        user_save(
            &mut conn,
            &occurrence(
                "h2",
                "pm2",
                "2026-08-30",
                MealSlot::Dinner,
                vec![leftovers("x")],
            ),
        )
        .unwrap();
        conn.execute("DELETE FROM household WHERE id = 'h'", [])
            .unwrap();
        assert_eq!(count(&conn, "planned_meal"), 1);
        assert_eq!(count(&conn, "meal_component"), 1);
        assert_eq!(
            load_meal(&conn, "h2", "pm2").unwrap().components(),
            &[leftovers("x")]
        );
    }

    #[test]
    fn a_zero_prep_minutes_row_is_refused_by_the_check() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        let err = conn
            .execute("UPDATE recipe SET prep_minutes = 0 WHERE id = 'r'", [])
            .unwrap_err();
        assert!(err.to_string().contains("CHECK constraint failed"), "{err}");
    }

    #[test]
    fn a_non_civil_verified_on_row_is_refused_by_the_check() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        let err = conn
            .execute(
                "UPDATE recipe_provenance SET verified_on = '29-08-2026' WHERE recipe_id = 'r'",
                [],
            )
            .unwrap_err();
        assert!(err.to_string().contains("CHECK constraint failed"), "{err}");
    }

    #[test]
    fn rights_and_slug_round_trip() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i", "olive oil", &[])).unwrap();
        let r = starter_recipe("h", "r", "a-slug", "i");
        save_recipe(&mut conn, &r).unwrap();
        assert_eq!(load(&conn, "h", "r"), Some(r));
    }

    #[test]
    fn provenance_without_rights_round_trips_as_none() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let r = recipe("h", "r", vec![]);
        save_recipe(&mut conn, &r).unwrap();
        let loaded = load(&conn, "h", "r").unwrap();
        assert_eq!(loaded.provenance().rights(), None);
        assert_eq!(loaded.provenance().starter_slug(), None);
    }

    #[test]
    fn prep_minutes_round_trips_and_absence_stays_absent() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i", "olive oil", &[])).unwrap();
        save_recipe(&mut conn, &starter_recipe("h", "r", "a-slug", "i")).unwrap();
        assert_eq!(load(&conn, "h", "r").unwrap().prep_minutes(), Some(15));
        save_recipe(&mut conn, &recipe("h", "r2", vec![])).unwrap();
        assert_eq!(load(&conn, "h", "r2").unwrap().prep_minutes(), None);
    }

    #[test]
    fn editing_a_starter_recipe_keeps_its_rights_and_slug() {
        // Risk 1: the MVP-008 edit path loads a `RecipeDto` whose rights scalars are
        // output-only, so a re-save carries no rights at all. It must not blank the columns.
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i", "olive oil", &[])).unwrap();
        save_recipe(&mut conn, &starter_recipe("h", "r", "a-slug", "i")).unwrap();
        let edited = Recipe::new(
            RecipeId::new("r").unwrap(),
            HouseholdId::new("h").unwrap(),
            "Renamed by the user",
            Some(2),
            Some(20),
            "Cook.",
            vec![],
            RecipeProvenance::new(ProvenanceKind::Starter, None, None, None).unwrap(),
        )
        .unwrap();
        save_recipe(&mut conn, &edited).unwrap();
        let loaded = load(&conn, "h", "r").unwrap();
        assert_eq!(loaded.title(), "Renamed by the user");
        assert_eq!(loaded.prep_minutes(), Some(20));
        assert_eq!(loaded.provenance().starter_slug(), Some("a-slug"));
        let kept = loaded.provenance().rights().unwrap();
        assert_eq!(kept.basis(), RightsBasis::Original);
        assert_eq!(kept.attribution(), Some("Kimatta"));
        assert_eq!(kept.modifications(), Some("halved the salt"));
    }

    #[test]
    fn an_edit_that_supplies_rights_replaces_them() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i", "olive oil", &[])).unwrap();
        save_recipe(&mut conn, &starter_recipe("h", "r", "a-slug", "i")).unwrap();
        let mut replaced = starter_recipe("h", "r", "a-slug", "i");
        replaced = Recipe::new(
            RecipeId::new("r").unwrap(),
            HouseholdId::new("h").unwrap(),
            replaced.title().to_owned(),
            replaced.servings(),
            replaced.prep_minutes(),
            replaced.instructions().to_owned(),
            replaced.lines().to_vec(),
            starter_provenance("a-slug", Some(rights(RightsBasis::Cc0, Some("Someone")))),
        )
        .unwrap();
        save_recipe(&mut conn, &replaced).unwrap();
        let kept = load(&conn, "h", "r").unwrap();
        let kept = kept.provenance().rights().unwrap();
        assert_eq!(kept.basis(), RightsBasis::Cc0);
        assert_eq!(kept.attribution(), Some("Someone"));
    }

    #[test]
    fn an_edit_supplying_rights_with_no_attribution_clears_the_stored_one() {
        // The four rights columns move as one group: an incoming record that names no
        // attribution says there is none, and must not silently retain the old credit.
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i", "olive oil", &[])).unwrap();
        save_recipe(&mut conn, &starter_recipe("h", "r", "a-slug", "i")).unwrap();
        let base = starter_recipe("h", "r", "a-slug", "i");
        let cleared = Recipe::new(
            RecipeId::new("r").unwrap(),
            HouseholdId::new("h").unwrap(),
            base.title().to_owned(),
            base.servings(),
            base.prep_minutes(),
            base.instructions().to_owned(),
            base.lines().to_vec(),
            starter_provenance("a-slug", Some(rights(RightsBasis::Original, None))),
        )
        .unwrap();
        save_recipe(&mut conn, &cleared).unwrap();
        let loaded = load(&conn, "h", "r").unwrap();
        assert_eq!(loaded.provenance().rights().unwrap().attribution(), None);
    }

    #[test]
    fn a_corrupt_rights_basis_row_is_reported_not_coerced() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i", "olive oil", &[])).unwrap();
        save_recipe(&mut conn, &starter_recipe("h", "r", "a-slug", "i")).unwrap();
        set_provenance_column(&conn, "r", "rights_basis", Some("cc_by"));
        let err = load_record_err(&conn, "h", "r");
        assert!(
            matches!(&err, StorageError::CorruptRights { recipe, detail }
                if recipe == "r" && detail.contains("unreadable rights_basis")),
            "{err:?}"
        );
    }

    #[test]
    fn a_partial_rights_row_is_reported_not_coerced() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i", "olive oil", &[])).unwrap();
        save_recipe(&mut conn, &starter_recipe("h", "r", "a-slug", "i")).unwrap();
        set_provenance_column(&conn, "r", "verified_on", None);
        let err = load_record_err(&conn, "h", "r");
        assert!(
            matches!(&err, StorageError::CorruptRights { detail, .. }
                if detail.contains("rights_basis is set but verified_on is NULL")),
            "{err:?}"
        );
        // And the mirror.
        set_provenance_column(&conn, "r", "verified_on", Some("2026-08-29"));
        set_provenance_column(&conn, "r", "rights_basis", None);
        let err = load_record_err(&conn, "h", "r");
        assert!(
            matches!(&err, StorageError::CorruptRights { detail, .. }
                if detail.contains("verified_on is set but rights_basis is NULL")),
            "{err:?}"
        );
    }

    #[test]
    fn an_attribution_without_a_basis_is_reported_not_dropped() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_recipe(&mut conn, &recipe("h", "r", vec![])).unwrap();
        set_provenance_column(&conn, "r", "attribution", Some("Someone"));
        let err = load_record_err(&conn, "h", "r");
        assert!(
            matches!(&err, StorageError::CorruptRights { detail, .. }
                if detail.contains("attribution or modifications without a rights basis")),
            "{err:?}"
        );
    }

    #[test]
    fn a_glob_passing_non_date_verified_on_is_reported() {
        // `2026-13-45` satisfies the column GLOB and still is not a civil date.
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i", "olive oil", &[])).unwrap();
        save_recipe(&mut conn, &starter_recipe("h", "r", "a-slug", "i")).unwrap();
        set_provenance_column(&conn, "r", "verified_on", Some("2026-13-45"));
        let err = load_record_err(&conn, "h", "r");
        assert!(
            matches!(&err, StorageError::CorruptRights { detail, .. }
                if detail.contains("unreadable rights record")),
            "{err:?}"
        );
    }

    // Expected-to-pass pins on the `write_recipe` extraction: the guards `save_recipe` kept
    // for itself are not covered by the line-set pin, so they are asserted directly.
    #[test]
    fn save_into_an_absent_household_is_still_no_such_household() {
        let mut conn = open(":memory:").unwrap();
        let err = save_recipe(&mut conn, &recipe("nope", "r", vec![])).unwrap_err();
        assert!(
            matches!(err, StorageError::NoSuchHousehold(ref h) if h == "nope"),
            "{err:?}"
        );
    }

    #[test]
    fn saving_over_another_households_recipe_id_is_still_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        save_recipe(&mut conn, &recipe("h1", "r", vec![])).unwrap();
        let err = save_recipe(&mut conn, &recipe("h2", "r", vec![])).unwrap_err();
        assert!(
            matches!(&err, StorageError::NoSuchRecipe { recipe, household }
                if recipe == "r" && household == "h2"),
            "{err:?}"
        );
        assert_eq!(
            load(&conn, "h1", "r").unwrap().household_id().as_str(),
            "h1"
        );
    }

    // --- install_starter_content --------------------------------------------------------

    fn catalog() -> Vec<Ingredient> {
        vec![
            ingredient("i-oil", "olive oil", &["extra virgin olive oil"]),
            ingredient("i-salt", "salt", &[]),
        ]
    }

    fn starter_set(household: &str) -> Vec<Recipe> {
        vec![
            starter_recipe(household, "sr-1", "slug-one", "i-oil"),
            starter_recipe(household, "sr-2", "slug-two", "i-salt"),
        ]
    }

    #[test]
    fn installing_into_an_empty_household_installs_every_recipe() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let report =
            install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        assert_eq!(
            report,
            StarterInstallReport {
                installed: 2,
                skipped: 0,
                catalog_installed: 2
            }
        );
        assert_eq!(ids(&conn, "h", RecipeListing::Active), vec!["sr-1", "sr-2"]);
        assert_eq!(
            load(&conn, "h", "sr-1")
                .unwrap()
                .provenance()
                .starter_slug(),
            Some("slug-one")
        );
    }

    #[test]
    fn installing_twice_installs_nothing_the_second_time() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        let report =
            install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        assert_eq!(
            report,
            StarterInstallReport {
                installed: 0,
                skipped: 2,
                catalog_installed: 0
            }
        );
        assert_eq!(count(&conn, "recipe"), 2);
        assert_eq!(count(&conn, "ingredient"), 2);
    }

    #[test]
    fn a_second_install_writes_no_rows() {
        // Risk 10. `total_changes()` can prove "no rows were written"; it cannot prove "no
        // transaction was opened", and this test claims only what it can prove.
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        let before = conn.total_changes();
        install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        assert_eq!(conn.total_changes(), before);
    }

    #[test]
    fn an_archived_starter_recipe_is_not_reinstalled() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        archive_recipe(
            &mut conn,
            &hid("h"),
            &RecipeId::new("sr-1").unwrap(),
            parse_civil_date("2026-08-29").unwrap(),
        )
        .unwrap();
        let report =
            install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        assert_eq!(report.installed, 0);
        assert_eq!(report.skipped, 2);
        assert_eq!(ids(&conn, "h", RecipeListing::Active), vec!["sr-2"]);
        assert_eq!(ids(&conn, "h", RecipeListing::Archived), vec!["sr-1"]);
    }

    #[test]
    fn an_edited_installed_starter_recipe_is_not_overwritten() {
        // Install-once (Risk 9): a revised entry does not reach a device that holds the slug.
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        let mut edited = starter_recipe("h", "sr-1", "slug-one", "i-oil");
        edited = Recipe::new(
            RecipeId::new("sr-1").unwrap(),
            hid("h"),
            "The user's own title",
            edited.servings(),
            edited.prep_minutes(),
            edited.instructions().to_owned(),
            edited.lines().to_vec(),
            edited.provenance().clone(),
        )
        .unwrap();
        save_recipe(&mut conn, &edited).unwrap();
        install_starter_content(&mut conn, &hid("h"), &catalog(), &starter_set("h")).unwrap();
        assert_eq!(
            load(&conn, "h", "sr-1").unwrap().title(),
            "The user's own title"
        );
    }

    #[test]
    fn install_is_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        install_starter_content(&mut conn, &hid("h1"), &catalog(), &starter_set("h1")).unwrap();
        let mut second = starter_set("h2");
        second = second
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                Recipe::new(
                    RecipeId::new(format!("h2-{i}")).unwrap(),
                    hid("h2"),
                    r.title().to_owned(),
                    r.servings(),
                    r.prep_minutes(),
                    r.instructions().to_owned(),
                    r.lines().to_vec(),
                    r.provenance().clone(),
                )
                .unwrap()
            })
            .collect();
        let report = install_starter_content(&mut conn, &hid("h2"), &catalog(), &second).unwrap();
        assert_eq!(report.installed, 2);
        assert_eq!(
            ids(&conn, "h1", RecipeListing::Active),
            vec!["sr-1", "sr-2"]
        );
        assert_eq!(
            ids(&conn, "h2", RecipeListing::Active),
            vec!["h2-0", "h2-1"]
        );
    }

    #[test]
    fn installing_an_empty_recipe_set_still_seeds_the_catalog_and_reports_it() {
        // Risk 4: the zero-cook-review case must read as empty by design, not as nothing
        // happening at all.
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let report = install_starter_content(&mut conn, &hid("h"), &catalog(), &[]).unwrap();
        assert_eq!(report.installed, 0);
        assert_eq!(report.skipped, 0);
        assert_eq!(report.catalog_installed, 2);
        assert_eq!(count(&conn, "ingredient"), 2);
    }

    #[test]
    fn a_changed_alias_set_is_replaced_whole_when_any_id_is_new() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        install_starter_content(&mut conn, &hid("h"), &catalog()[..1], &[]).unwrap();
        let revised = vec![
            ingredient("i-oil", "olive oil", &["evoo"]),
            ingredient("i-salt", "salt", &[]),
        ];
        install_starter_content(&mut conn, &hid("h"), &revised, &[]).unwrap();
        assert_eq!(
            load_ingredient(&conn, &IngredientId::new("i-oil").unwrap())
                .unwrap()
                .unwrap()
                .aliases(),
            ["evoo"]
        );
    }

    #[test]
    fn a_starter_recipe_without_a_slug_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = install_starter_content(
            &mut conn,
            &hid("h"),
            &catalog(),
            &[recipe("h", "r", vec![])],
        )
        .unwrap_err();
        assert!(
            matches!(&err, StorageError::MissingStarterSlug(t) if t == "Recipe r"),
            "{err:?}"
        );
        assert_eq!(count(&conn, "recipe"), 0);
        assert_eq!(count(&conn, "ingredient"), 0);
    }

    #[test]
    fn a_recipe_line_naming_an_unseeded_catalog_id_rolls_back_the_whole_install() {
        // The catalog upserts happen earlier in the same transaction, so the guarantee is
        // rollback, not write-avoidance: the catalog must be absent afterwards too.
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let bad = vec![starter_recipe("h", "sr-1", "slug-one", "i-missing")];
        let err = install_starter_content(&mut conn, &hid("h"), &catalog(), &bad).unwrap_err();
        assert!(
            matches!(&err, StorageError::NoSuchIngredient(i) if i == "i-missing"),
            "{err:?}"
        );
        assert_eq!(count(&conn, "recipe"), 0);
        assert_eq!(count(&conn, "ingredient"), 0);
    }

    #[test]
    fn install_for_an_absent_household_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        let err = install_starter_content(&mut conn, &hid("nope"), &catalog(), &[]).unwrap_err();
        assert!(
            matches!(&err, StorageError::NoSuchHousehold(h) if h == "nope"),
            "{err:?}"
        );
    }

    #[test]
    fn install_for_an_absent_household_is_rejected_after_a_successful_install() {
        // Adversarial: run it against a database where the catalog is already fully seeded,
        // so the call takes the short-circuit path. The naive ordering — household check
        // inside the write branch — passes the test above vacuously and fails this one.
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        install_starter_content(&mut conn, &hid("h"), &catalog(), &[]).unwrap();
        let err = install_starter_content(&mut conn, &hid("nope"), &catalog(), &[]).unwrap_err();
        assert!(
            matches!(&err, StorageError::NoSuchHousehold(h) if h == "nope"),
            "{err:?}"
        );
    }

    // --- MVP-014 step 2: schema v8, the optional binary pantry ---------------------------

    const RAW_PANTRY: &str = "INSERT INTO pantry_item
        (household_id, ingredient_id, custom_ingredient_id) VALUES (?1, ?2, ?3)";

    /// A household, one catalog ingredient and one custom ingredient of that household, so
    /// the pantry constraints below are the only thing a raw insert can trip.
    fn seed_for_pantry(conn: &mut Connection, household: &str) {
        seed(conn, household);
        upsert_ingredient(conn, &ingredient("flour", "flour", &["plain flour"])).unwrap();
        upsert_custom_ingredient(conn, &custom(&format!("c-{household}"), household, "mix"))
            .unwrap();
    }

    /// Test-level stand-in for the on-device v7→v8 migration, in the pattern of its
    /// predecessors: everything saved at v7 must survive a migration that only adds a table.
    #[test]
    fn an_existing_v7_database_migrates_to_v10_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 7).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 7);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed(&mut raw, "h");
            save_recipe(&mut raw, &recipe("h", "r", vec![])).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 10);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "recipe"), 1);
        assert_eq!(load(&conn, "h", "r").unwrap().title(), "Recipe r");
        assert_eq!(count(&conn, "pantry_item"), 0);
    }

    /// Adversarial: the exactly-one-of CHECK is what keeps a pantry row a single identity.
    /// It fires before FK enforcement, so a row naming two ids that do not exist still
    /// reports the CHECK.
    #[test]
    fn a_pantry_row_naming_both_or_neither_ingredient_kinds_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        let both = conn
            .execute(RAW_PANTRY, params!["h", "flour", "c-h"])
            .unwrap_err();
        assert!(
            both.to_string().contains("CHECK constraint failed"),
            "{both}"
        );
        let neither = conn
            .execute(
                RAW_PANTRY,
                params!["h", Option::<String>::None, Option::<String>::None],
            )
            .unwrap_err();
        assert!(
            neither.to_string().contains("CHECK constraint failed"),
            "{neither}"
        );
        assert_eq!(count(&conn, "pantry_item"), 0);
    }

    /// Adversarial: the two partial unique indexes are the table's real key, since either
    /// column is NULL half the time and so no PRIMARY KEY can carry it. A silently
    /// ineffective index would let the duplicates through; scoping the key to the household
    /// wrongly would refuse the second household.
    #[test]
    fn the_pantry_identity_indexes_reject_a_duplicate_row() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        seed_for_pantry(&mut conn, "h2");
        conn.execute(RAW_PANTRY, params!["h", "flour", Option::<String>::None])
            .unwrap();
        let dup = conn
            .execute(RAW_PANTRY, params!["h", "flour", Option::<String>::None])
            .unwrap_err();
        assert!(
            dup.to_string().contains("UNIQUE constraint failed"),
            "{dup}"
        );

        conn.execute(RAW_PANTRY, params!["h", Option::<String>::None, "c-h"])
            .unwrap();
        let dup_custom = conn
            .execute(RAW_PANTRY, params!["h", Option::<String>::None, "c-h"])
            .unwrap_err();
        assert!(
            dup_custom.to_string().contains("UNIQUE constraint failed"),
            "{dup_custom}"
        );

        conn.execute(RAW_PANTRY, params!["h2", "flour", Option::<String>::None])
            .unwrap();
        assert_eq!(count(&conn, "pantry_item"), 3);
    }

    /// Absent-at-7 then present-at-8, for the reason
    /// `the_line_ingredient_foreign_keys_are_indexed` gives: the claim is that migration 8
    /// ships the indexes, not that the latest schema has them. The household-leading unique
    /// indexes cannot serve an FK-parent-delete lookup, so each FK is indexed on its own.
    #[test]
    fn the_pantry_foreign_keys_are_indexed() {
        let mut conn = Connection::open_in_memory().unwrap();
        MIGRATIONS.to_version(&mut conn, 7).unwrap();
        assert!(index_names(&conn, "pantry_item").is_empty());
        MIGRATIONS.to_version(&mut conn, 8).unwrap();
        let names = index_names(&conn, "pantry_item");
        for expected in [
            "pantry_item_catalog",
            "pantry_item_custom",
            "pantry_item_ingredient",
            "pantry_item_custom_ingredient",
        ] {
            assert!(names.contains(&expected.to_owned()), "{expected} missing");
        }
    }

    // --- MVP-014 step 4: the pantry read projection and the mark command -----------------

    fn catalog_ref(id: &str) -> IngredientRef {
        IngredientRef::Catalog(IngredientId::new(id).unwrap())
    }

    fn custom_ref(id: &str) -> IngredientRef {
        IngredientRef::Custom(CustomIngredientId::new(id).unwrap())
    }

    fn pantry(conn: &Connection, household: &str) -> Vec<PantryEntry> {
        list_pantry_entries(conn, &hid(household)).unwrap()
    }

    fn marked_names(entries: &[PantryEntry]) -> Vec<&str> {
        entries
            .iter()
            .filter(|e| e.marked)
            .map(|e| e.name.as_str())
            .collect()
    }

    #[test]
    fn marking_an_identity_persists_and_unmarking_removes_it() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        let stored = set_pantry_mark(&mut conn, &hid("h"), &catalog_ref("flour"), true).unwrap();
        assert!(stored.marked);
        assert_eq!(stored.name, "flour");
        assert_eq!(count(&conn, "pantry_item"), 1);
        assert_eq!(marked_names(&pantry(&conn, "h")), vec!["flour"]);

        let custom_stored =
            set_pantry_mark(&mut conn, &hid("h"), &custom_ref("c-h"), true).unwrap();
        assert!(custom_stored.marked);
        assert_eq!(custom_stored.name, "mix");
        assert_eq!(count(&conn, "pantry_item"), 2);

        let cleared = set_pantry_mark(&mut conn, &hid("h"), &catalog_ref("flour"), false).unwrap();
        assert!(!cleared.marked);
        assert_eq!(cleared.name, "flour");
        assert_eq!(marked_names(&pantry(&conn, "h")), vec!["mix"]);

        set_pantry_mark(&mut conn, &hid("h"), &custom_ref("c-h"), false).unwrap();
        assert_eq!(count(&conn, "pantry_item"), 0);
    }

    /// Edge: the UI toggles one row at a time and may repeat a tap, so both directions must
    /// be idempotent rather than raising on the second call.
    #[test]
    fn marking_and_unmarking_are_both_idempotent() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        for _ in 0..2 {
            set_pantry_mark(&mut conn, &hid("h"), &catalog_ref("flour"), true).unwrap();
        }
        assert_eq!(count(&conn, "pantry_item"), 1);
        for _ in 0..2 {
            set_pantry_mark(&mut conn, &hid("h"), &catalog_ref("flour"), false).unwrap();
        }
        assert_eq!(count(&conn, "pantry_item"), 0);
    }

    /// AC-2: nothing marked is a listing of unmarked identities, never an empty screen and
    /// never a claim the household is out of anything.
    #[test]
    fn an_empty_pantry_lists_every_identity_as_unmarked() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        let entries = pantry(&conn, "h");
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| !e.marked));
    }

    #[test]
    fn list_pantry_entries_covers_the_catalog_and_only_this_households_custom_ingredients() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        seed_for_pantry(&mut conn, "h2");
        let entries = pantry(&conn, "h");
        let refs: Vec<&IngredientRef> = entries.iter().map(|e| &e.ingredient).collect();
        assert_eq!(refs, vec![&catalog_ref("flour"), &custom_ref("c-h")]);
    }

    /// R1: `ORDER BY` on a TEXT column is BINARY-collated, so without `COLLATE NOCASE` every
    /// capitalised custom name sorts ahead of the whole lowercase catalog.
    #[test]
    fn pantry_entries_are_ordered_case_insensitively() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(&mut conn, &ingredient("i-black", "black pepper", &[])).unwrap();
        upsert_custom_ingredient(&mut conn, &custom("c-bacon", "h", "Bacon")).unwrap();
        let entries = pantry(&conn, "h");
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["Bacon", "black pepper"]);
    }

    #[test]
    fn catalog_entries_carry_their_aliases_and_custom_entries_carry_none() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        upsert_ingredient(
            &mut conn,
            &ingredient("chickpeas", "chickpeas", &["garbanzo beans", "ceci"]),
        )
        .unwrap();
        upsert_ingredient(&mut conn, &ingredient("shrimp", "shrimp", &["prawns"])).unwrap();
        upsert_custom_ingredient(&mut conn, &custom("c-h", "h", "nana's mix")).unwrap();
        let entries = pantry(&conn, "h");
        let by_name = |n: &str| {
            entries
                .iter()
                .find(|e| e.name == n)
                .unwrap_or_else(|| panic!("{n} missing"))
        };
        assert_eq!(by_name("chickpeas").aliases, vec!["ceci", "garbanzo beans"]);
        assert_eq!(by_name("shrimp").aliases, vec!["prawns"]);
        assert!(by_name("nana's mix").aliases.is_empty());
    }

    /// The narrow read the shopping snapshot uses must agree with the browse read on which
    /// identities are marked, and must be scoped just as tightly.
    #[test]
    fn list_marked_pantry_refs_returns_only_this_households_marked_identities() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        seed_for_pantry(&mut conn, "h2");
        upsert_ingredient(&mut conn, &ingredient("sugar", "sugar", &[])).unwrap();
        for r in [catalog_ref("flour"), custom_ref("c-h")] {
            set_pantry_mark(&mut conn, &hid("h"), &r, true).unwrap();
        }
        // h2 marks a catalog identity h did not, and `sugar` stays unmarked everywhere.
        set_pantry_mark(&mut conn, &hid("h2"), &catalog_ref("sugar"), true).unwrap();

        assert_eq!(
            list_marked_pantry_refs(&conn, &hid("h")).unwrap(),
            vec![catalog_ref("flour"), custom_ref("c-h")]
        );
        assert_eq!(
            list_marked_pantry_refs(&conn, &hid("h2")).unwrap(),
            vec![catalog_ref("sugar")]
        );
        // The browse read is the authority it must reproduce.
        assert_eq!(marked_names(&pantry(&conn, "h")), vec!["flour", "mix"]);
    }

    /// The contract [`list_pantry_entries`] states and [`load_shopping_input`] relied on
    /// through it: no records reads as no records, and an absent household is not an error.
    #[test]
    fn list_marked_pantry_refs_of_an_empty_or_absent_pantry_is_empty() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        assert!(list_marked_pantry_refs(&conn, &hid("h"))
            .unwrap()
            .is_empty());
        assert!(list_marked_pantry_refs(&conn, &hid("nobody"))
            .unwrap()
            .is_empty());
    }

    /// Adversarial (AC-3): a mark is one household's record and no other household's read or
    /// write may see or clear it.
    #[test]
    fn pantry_reads_and_writes_are_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h1");
        seed_for_pantry(&mut conn, "h2");
        set_pantry_mark(&mut conn, &hid("h1"), &catalog_ref("flour"), true).unwrap();
        assert_eq!(marked_names(&pantry(&conn, "h1")), vec!["flour"]);
        assert!(marked_names(&pantry(&conn, "h2")).is_empty());

        set_pantry_mark(&mut conn, &hid("h2"), &catalog_ref("flour"), false).unwrap();
        assert_eq!(
            marked_names(&pantry(&conn, "h1")),
            vec!["flour"],
            "h2's unmark must not clear h1's row"
        );
        assert_eq!(count(&conn, "pantry_item"), 1);
    }

    /// Adversarial: the identity check is the same one `check_line_refs` applies, so a
    /// foreign custom ingredient is refused by name of the requesting household.
    #[test]
    fn a_custom_ingredient_of_another_household_cannot_be_marked() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h1");
        seed_for_pantry(&mut conn, "h2");
        let err = set_pantry_mark(&mut conn, &hid("h1"), &custom_ref("c-h2"), true).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::CustomIngredientHouseholdMismatch { ingredient, expected, actual }
                    if ingredient == "c-h2" && expected == "h1" && actual == "h2"
            ),
            "{err:?}"
        );
        assert_eq!(count(&conn, "pantry_item"), 0);
    }

    #[test]
    fn marking_an_absent_ingredient_is_rejected_and_writes_nothing() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        let err = set_pantry_mark(&mut conn, &hid("h"), &catalog_ref("ghost"), true).unwrap_err();
        assert!(
            matches!(&err, StorageError::NoSuchIngredient(i) if i == "ghost"),
            "{err:?}"
        );
        assert_eq!(count(&conn, "pantry_item"), 0);
    }

    /// The read deliberately does *not* `require_household`: a consumer listing before
    /// bootstrap gets the catalog with nothing marked, not `NoSuchHousehold`. The write
    /// rejects the same household (below); the asymmetry is the documented contract, and
    /// adding `require_household` here for symmetry would break it silently.
    #[test]
    fn an_absent_household_reads_as_the_catalog_with_nothing_marked() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        // Marked for `h`, so this proves the ghost read does not leak another household's
        // marks rather than only that an empty table reads unmarked.
        set_pantry_mark(&mut conn, &hid("h"), &catalog_ref("flour"), true).unwrap();
        let entries = list_pantry_entries(&conn, &hid("ghost")).unwrap();
        assert_eq!(
            entries.iter().map(|e| &e.ingredient).collect::<Vec<_>>(),
            vec![&catalog_ref("flour")],
            "the catalog, and no other household's customs"
        );
        assert!(entries.iter().all(|e| !e.marked));
    }

    #[test]
    fn marking_for_an_absent_household_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        let err =
            set_pantry_mark(&mut conn, &hid("ghost"), &catalog_ref("flour"), true).unwrap_err();
        assert!(
            matches!(&err, StorageError::NoSuchHousehold(h) if h == "ghost"),
            "{err:?}"
        );
        assert_eq!(count(&conn, "pantry_item"), 0);
    }

    /// AC-1: the mark is durable, not process state.
    #[test]
    fn pantry_state_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut conn = open(&path).unwrap();
            seed_for_pantry(&mut conn, "h");
            set_pantry_mark(&mut conn, &hid("h"), &catalog_ref("flour"), true).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(marked_names(&pantry(&conn, "h")), vec!["flour"]);
    }

    /// Expected to pass by analogy with
    /// `deleting_a_household_cascades_recipes_lines_provenance_and_custom_ingredients`, and
    /// written anyway because `pantry_item` is the first table that is simultaneously a
    /// cascading child of `household` and a `NO ACTION` child of `custom_ingredient`, itself
    /// a cascading child of `household`: one delete exercises both edges.
    #[test]
    fn deleting_a_household_cascades_to_its_pantry_items() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        seed_for_pantry(&mut conn, "h2");
        conn.execute(RAW_PANTRY, params!["h", "flour", Option::<String>::None])
            .unwrap();
        conn.execute(RAW_PANTRY, params!["h", Option::<String>::None, "c-h"])
            .unwrap();
        conn.execute(RAW_PANTRY, params!["h2", "flour", Option::<String>::None])
            .unwrap();
        conn.execute("DELETE FROM household WHERE id = 'h'", [])
            .unwrap();
        assert_eq!(count(&conn, "pantry_item"), 1);
        let survivor: String = conn
            .query_row("SELECT household_id FROM pantry_item", [], |r| r.get(0))
            .unwrap();
        assert_eq!(survivor, "h2");
    }

    // --- MVP-016 step 3: schema v9, shopping line states and manual items ----------------

    const RAW_LINE_STATE: &str = "INSERT INTO shopping_line_state
        (household_id, from_date, to_date, line_key, checked, checked_against, hidden, restored)
        VALUES (?1, '2026-08-29', '2026-09-04', ?2, ?3, ?4, ?5, ?6)";

    /// Test-level stand-in for the on-device v8→v9 migration, in the pattern of its
    /// predecessors: everything saved at v8 must survive a migration that only adds tables.
    #[test]
    fn an_existing_v8_database_migrates_to_v10_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 8).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 8);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed_for_pantry(&mut raw, "h");
            set_pantry_mark(&mut raw, &hid("h"), &catalog_ref("flour"), true).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 10);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "pantry_item"), 1);
        assert_eq!(count(&conn, "shopping_line_state"), 0);
        assert_eq!(count(&conn, "shopping_manual_item"), 0);
    }

    /// Test-level stand-in for the on-device v9→v10 migration (MVP-023): everything saved at
    /// v9 — a shopping overlay row included — survives a migration that only adds tables.
    #[test]
    fn an_existing_v9_database_migrates_to_v10_without_losing_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut raw = Connection::open(&path).unwrap();
            MIGRATIONS.to_version(&mut raw, 9).unwrap();
            assert_eq!(schema_version(&raw).unwrap(), 9);
            raw.pragma_update(None, "foreign_keys", "ON").unwrap();
            seed_for_pantry(&mut raw, "h");
            set_pantry_mark(&mut raw, &hid("h"), &catalog_ref("flour"), true).unwrap();
            raw.execute(
                RAW_LINE_STATE,
                params!["h", "m:x", 1, Some("exact:1/1|none"), 0, 0],
            )
            .unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 10);
        assert_eq!(count(&conn, "household"), 1);
        assert_eq!(count(&conn, "pantry_item"), 1);
        assert_eq!(count(&conn, "shopping_line_state"), 1);
        assert_eq!(count(&conn, "policy"), 0);
        assert_eq!(count(&conn, "controller_ledger"), 0);
    }

    /// Adversarial: an all-false row is a row that says nothing, and storage deletes rather
    /// than writes one; the CHECK is what stops a raw write from leaving one behind.
    #[test]
    fn a_shopping_line_state_row_with_no_flag_set_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = conn
            .execute(
                RAW_LINE_STATE,
                params!["h", "m:x", 0, Option::<String>::None, 0, 0],
            )
            .unwrap_err();
        assert!(err.to_string().contains("CHECK constraint failed"), "{err}");
        let blank = conn
            .execute(
                RAW_LINE_STATE,
                params!["h", "", 0, Option::<String>::None, 1, 0],
            )
            .unwrap_err();
        assert!(
            blank.to_string().contains("CHECK constraint failed"),
            "{blank}"
        );
        assert_eq!(count(&conn, "shopping_line_state"), 0);
    }

    /// Adversarial: a check without the amount it was made against cannot be compared later,
    /// and an amount without a check is stale data; both shapes are refused.
    #[test]
    fn a_checked_line_state_without_its_quantity_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = conn
            .execute(
                RAW_LINE_STATE,
                params!["h", "m:x", 1, Option::<String>::None, 0, 0],
            )
            .unwrap_err();
        assert!(err.to_string().contains("CHECK constraint failed"), "{err}");
        let stale = conn
            .execute(
                RAW_LINE_STATE,
                params!["h", "m:x", 0, Some("exact:1/1|none"), 1, 0],
            )
            .unwrap_err();
        assert!(
            stale.to_string().contains("CHECK constraint failed"),
            "{stale}"
        );
        conn.execute(
            RAW_LINE_STATE,
            params!["h", "m:x", 1, Some("exact:1/1|none"), 0, 0],
        )
        .unwrap();
        assert_eq!(count(&conn, "shopping_line_state"), 1);
    }

    #[test]
    fn a_blank_manual_item_name_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        for name in ["", " \t\n"] {
            let err = conn
                .execute(
                    "INSERT INTO shopping_manual_item (id, household_id, name, note, checked)
                     VALUES ('mi', 'h', ?1, NULL, 0)",
                    params![name],
                )
                .unwrap_err();
            assert!(err.to_string().contains("CHECK constraint failed"), "{err}");
        }
        assert_eq!(count(&conn, "shopping_manual_item"), 0);
    }

    #[test]
    fn deleting_a_household_cascades_to_its_shopping_rows() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        seed(&mut conn, "h2");
        for h in ["h", "h2"] {
            conn.execute(
                RAW_LINE_STATE,
                params![h, "m:x", 1, Some("exact:1/1|none"), 0, 0],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO shopping_manual_item (id, household_id, name, note, checked)
                 VALUES (?1, ?2, 'batteries', NULL, 0)",
                params![format!("mi-{h}"), h],
            )
            .unwrap();
        }
        conn.execute("DELETE FROM household WHERE id = 'h'", [])
            .unwrap();
        assert_eq!(count(&conn, "shopping_line_state"), 1);
        assert_eq!(count(&conn, "shopping_manual_item"), 1);
        let survivor: String = conn
            .query_row("SELECT household_id FROM shopping_manual_item", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(survivor, "h2");
    }

    // --- MVP-016 steps 4–6: line states, manual items, reset, bulk pantry marks ----------

    fn d(raw: &str) -> CivilDate {
        parse_civil_date(raw).unwrap()
    }

    fn checked(key: &str, against: &str) -> ShoppingLineState {
        ShoppingLineState {
            key: key.to_owned(),
            checked: true,
            checked_against: Some(against.to_owned()),
            hidden: false,
            restored: false,
        }
    }

    fn flags(key: &str, hidden: bool, restored: bool) -> ShoppingLineState {
        ShoppingLineState {
            key: key.to_owned(),
            checked: false,
            checked_against: None,
            hidden,
            restored,
        }
    }

    fn states(conn: &Connection, household: &str, from: &str, to: &str) -> Vec<ShoppingLineState> {
        list_shopping_line_states(conn, &hid(household), d(from), d(to)).unwrap()
    }

    fn set_state(
        conn: &mut Connection,
        household: &str,
        from: &str,
        to: &str,
        state: &ShoppingLineState,
    ) -> ShoppingLineState {
        set_shopping_line_state(conn, &hid(household), d(from), d(to), state).unwrap()
    }

    fn item(id: &str, household: &str, name: &str, checked: bool) -> ShoppingManualItem {
        ShoppingManualItem {
            id: ShoppingManualItemId::new(id).unwrap(),
            household_id: hid(household),
            name: name.to_owned(),
            note: None,
            checked,
        }
    }

    fn item_names(conn: &Connection, household: &str) -> Vec<String> {
        list_shopping_manual_items(conn, &hid(household))
            .unwrap()
            .into_iter()
            .map(|i| i.name)
            .collect()
    }

    #[test]
    fn a_line_state_persists_and_reads_back() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let stored = set_state(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            &checked("m:catalog:flour:volume_us:req:known", "exact:2/1|known:cup"),
        );
        assert!(stored.checked);
        assert_eq!(
            stored.checked_against.as_deref(),
            Some("exact:2/1|known:cup")
        );
        let hidden = set_state(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            &flags("s:pm-1:0:0", true, false),
        );
        assert!(hidden.hidden);
        let read = states(&conn, "h", "2026-08-29", "2026-09-04");
        assert_eq!(read, vec![stored.clone(), hidden]);
        // A re-save replaces the row rather than adding one, and unchecking drops the token
        // even when the caller left it in place.
        let mut unchecked = stored.clone();
        unchecked.checked = false;
        unchecked.restored = true;
        let restored = set_state(&mut conn, "h", "2026-08-29", "2026-09-04", &unchecked);
        assert_eq!(restored.checked_against, None);
        assert!(restored.restored);
        assert_eq!(count(&conn, "shopping_line_state"), 2);
    }

    #[test]
    fn clearing_every_flag_removes_the_row() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        set_state(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            &checked("m:x", "unknown|none"),
        );
        assert_eq!(count(&conn, "shopping_line_state"), 1);
        let cleared = set_state(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            &flags("m:x", false, false),
        );
        assert_eq!(cleared, flags("m:x", false, false));
        assert_eq!(count(&conn, "shopping_line_state"), 0);
        // Clearing an absent row is a no-op, never an error.
        set_state(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            &flags("m:x", false, false),
        );
        let err = set_shopping_line_state(
            &mut conn,
            &hid("h"),
            d("2026-08-29"),
            d("2026-09-04"),
            &checked("  ", "unknown|none"),
        )
        .unwrap_err();
        assert!(
            matches!(err, StorageError::Shopping(ShoppingError::BlankKey)),
            "{err:?}"
        );
    }

    /// The other half of the `checked` ⇔ `checked_against` pair: the function rejects the
    /// shape itself rather than letting it reach the table CHECK as an untyped `Sqlite`.
    #[test]
    fn a_check_without_its_quantity_is_a_typed_error_not_a_check_violation() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let err = set_shopping_line_state(
            &mut conn,
            &hid("h"),
            d("2026-08-29"),
            d("2026-09-04"),
            &ShoppingLineState {
                key: "m:x".to_owned(),
                checked: true,
                checked_against: None,
                hidden: false,
                restored: false,
            },
        )
        .unwrap_err();
        assert!(
            matches!(
                err,
                StorageError::Shopping(ShoppingError::CheckWithoutQuantity)
            ),
            "{err:?}"
        );
        assert_eq!(count(&conn, "shopping_line_state"), 0);
    }

    /// Adversarial: two households and two windows, each with a state under the same key.
    #[test]
    fn line_state_is_scoped_to_household_and_window() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        set_state(
            &mut conn,
            "h1",
            "2026-08-29",
            "2026-09-04",
            &checked("m:x", "unknown|none"),
        );
        set_state(
            &mut conn,
            "h2",
            "2026-08-29",
            "2026-09-04",
            &flags("m:x", true, false),
        );
        set_state(
            &mut conn,
            "h1",
            "2026-09-05",
            "2026-09-11",
            &flags("m:x", false, true),
        );
        assert_eq!(
            states(&conn, "h1", "2026-08-29", "2026-09-04"),
            vec![checked("m:x", "unknown|none")]
        );
        assert_eq!(
            states(&conn, "h2", "2026-08-29", "2026-09-04"),
            vec![flags("m:x", true, false)]
        );
        assert_eq!(
            states(&conn, "h1", "2026-09-05", "2026-09-11"),
            vec![flags("m:x", false, true)]
        );
        // Clearing h2's row leaves h1's two untouched.
        set_state(
            &mut conn,
            "h2",
            "2026-08-29",
            "2026-09-04",
            &flags("m:x", false, false),
        );
        assert_eq!(count(&conn, "shopping_line_state"), 2);
    }

    /// Decision 3 pin: states are keyed by the exact window, so a window that moved by a
    /// cycle-settings edit reads none — and the rows are still there, not deleted.
    #[test]
    fn a_changed_window_reads_no_states() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        set_state(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            &checked("m:x", "unknown|none"),
        );
        assert!(states(&conn, "h", "2026-08-30", "2026-09-05").is_empty());
        assert!(states(&conn, "h", "2026-08-29", "2026-09-05").is_empty());
        assert_eq!(count(&conn, "shopping_line_state"), 1);
    }

    #[test]
    fn line_state_for_an_absent_household_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        let err = set_shopping_line_state(
            &mut conn,
            &hid("ghost"),
            d("2026-08-29"),
            d("2026-09-04"),
            &checked("m:x", "unknown|none"),
        )
        .unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)), "{err:?}");
        assert_eq!(count(&conn, "shopping_line_state"), 0);
    }

    /// AC-2: a check survives a restart.
    #[test]
    fn line_state_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut conn = open(&path).unwrap();
            seed(&mut conn, "h");
            set_state(
                &mut conn,
                "h",
                "2026-08-29",
                "2026-09-04",
                &checked("m:x", "unknown|none"),
            );
            save_shopping_manual_item(&mut conn, &item("mi-1", "h", "batteries", false)).unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(
            states(&conn, "h", "2026-08-29", "2026-09-04"),
            vec![checked("m:x", "unknown|none")]
        );
        assert_eq!(item_names(&conn, "h"), vec!["batteries"]);
    }

    #[test]
    fn a_manual_item_saves_and_lists() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let mut raw = item("mi-1", "h", "  batteries ", false);
        raw.note = Some("  AA  ".to_owned());
        let stored = save_shopping_manual_item(&mut conn, &raw).unwrap();
        assert_eq!(stored.name, "batteries");
        assert_eq!(stored.note.as_deref(), Some("AA"));
        assert_eq!(
            list_shopping_manual_items(&conn, &hid("h")).unwrap(),
            vec![stored]
        );
        // A blank note is `None`, not an empty string.
        let mut blank_note = item("mi-2", "h", "foil", true);
        blank_note.note = Some("  ".to_owned());
        assert_eq!(
            save_shopping_manual_item(&mut conn, &blank_note)
                .unwrap()
                .note,
            None
        );
    }

    #[test]
    fn saving_an_existing_id_replaces_it() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        save_shopping_manual_item(&mut conn, &item("mi-1", "h", "batteries", false)).unwrap();
        let replaced =
            save_shopping_manual_item(&mut conn, &item("mi-1", "h", "candles", true)).unwrap();
        assert!(replaced.checked);
        assert_eq!(item_names(&conn, "h"), vec!["candles"]);
        assert_eq!(count(&conn, "shopping_manual_item"), 1);
        delete_shopping_manual_item(&mut conn, &hid("h"), &replaced.id).unwrap();
        assert_eq!(count(&conn, "shopping_manual_item"), 0);
        let err = delete_shopping_manual_item(&mut conn, &hid("h"), &replaced.id).unwrap_err();
        assert!(
            matches!(err, StorageError::NoSuchShoppingItem(_)),
            "{err:?}"
        );
    }

    /// Adversarial: h2 can neither overwrite nor delete h1's item by naming its id.
    #[test]
    fn a_manual_item_of_another_household_cannot_be_replaced_or_deleted() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        let original = item("mi-1", "h1", "batteries", false);
        save_shopping_manual_item(&mut conn, &original).unwrap();
        let err =
            save_shopping_manual_item(&mut conn, &item("mi-1", "h2", "hijack", true)).unwrap_err();
        assert!(
            matches!(err, StorageError::NoSuchShoppingItem(_)),
            "{err:?}"
        );
        let err = delete_shopping_manual_item(
            &mut conn,
            &hid("h2"),
            &ShoppingManualItemId::new("mi-1").unwrap(),
        )
        .unwrap_err();
        assert!(
            matches!(err, StorageError::NoSuchShoppingItem(_)),
            "{err:?}"
        );
        assert_eq!(
            list_shopping_manual_items(&conn, &hid("h1")).unwrap(),
            vec![original]
        );
        assert!(list_shopping_manual_items(&conn, &hid("h2"))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn manual_items_are_ordered_case_insensitively() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        for (id, name) in [("a", "candles"), ("b", "Batteries"), ("c", "apples")] {
            save_shopping_manual_item(&mut conn, &item(id, "h", name, false)).unwrap();
        }
        assert_eq!(
            item_names(&conn, "h"),
            vec!["apples", "Batteries", "candles"]
        );
    }

    #[test]
    fn reset_clears_this_windows_states_and_only_checked_items() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        seed(&mut conn, "h2");
        set_state(
            &mut conn,
            "h",
            "2026-08-29",
            "2026-09-04",
            &checked("m:x", "unknown|none"),
        );
        set_state(
            &mut conn,
            "h",
            "2026-09-05",
            "2026-09-11",
            &flags("m:x", true, false),
        );
        set_state(
            &mut conn,
            "h2",
            "2026-08-29",
            "2026-09-04",
            &flags("m:x", true, false),
        );
        save_shopping_manual_item(&mut conn, &item("mi-1", "h", "bought", true)).unwrap();
        save_shopping_manual_item(&mut conn, &item("mi-2", "h", "still wanted", false)).unwrap();
        save_shopping_manual_item(&mut conn, &item("mi-3", "h2", "theirs", true)).unwrap();
        reset_shopping_list(&mut conn, &hid("h"), d("2026-08-29"), d("2026-09-04")).unwrap();
        assert!(states(&conn, "h", "2026-08-29", "2026-09-04").is_empty());
        assert_eq!(states(&conn, "h", "2026-09-05", "2026-09-11").len(), 1);
        assert_eq!(states(&conn, "h2", "2026-08-29", "2026-09-04").len(), 1);
        assert_eq!(item_names(&conn, "h"), vec!["still wanted"]);
        assert_eq!(item_names(&conn, "h2"), vec!["theirs"]);
        let err = reset_shopping_list(&mut conn, &hid("ghost"), d("2026-08-29"), d("2026-09-04"))
            .unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)), "{err:?}");
    }

    #[test]
    fn a_blank_manual_name_is_rejected_before_any_write() {
        let mut conn = open(":memory:").unwrap();
        for name in ["", "  \t"] {
            // The ghost household would be `NoSuchHousehold` if the name check came second.
            let err = save_shopping_manual_item(&mut conn, &item("mi-1", "ghost", name, false))
                .unwrap_err();
            assert!(
                matches!(err, StorageError::Shopping(ShoppingError::BlankName)),
                "{err:?}"
            );
        }
        assert_eq!(count(&conn, "shopping_manual_item"), 0);
    }

    /// One foreign custom ref in the batch fails the whole batch: the earlier catalog mark
    /// is rolled back too.
    #[test]
    fn bulk_marks_land_in_one_transaction_or_not_at_all() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        seed_for_pantry(&mut conn, "h2");
        let err = set_pantry_marks(
            &mut conn,
            &hid("h"),
            &[catalog_ref("flour"), custom_ref("c-h2")],
            true,
        )
        .unwrap_err();
        assert!(
            matches!(err, StorageError::CustomIngredientHouseholdMismatch { .. }),
            "{err:?}"
        );
        assert_eq!(count(&conn, "pantry_item"), 0);
        let err =
            set_pantry_marks(&mut conn, &hid("ghost"), &[catalog_ref("flour")], true).unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)), "{err:?}");
    }

    /// R2-4 pin: a pre-existing mark is not in the returned set, so an undo that sends the
    /// returned set back leaves it in place.
    #[test]
    fn bulk_marking_returns_only_the_refs_it_changed() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        set_pantry_mark(&mut conn, &hid("h"), &catalog_ref("flour"), true).unwrap();
        let changed = set_pantry_marks(
            &mut conn,
            &hid("h"),
            &[catalog_ref("flour"), custom_ref("c-h")],
            true,
        )
        .unwrap();
        assert_eq!(changed, vec![custom_ref("c-h")]);
        assert_eq!(count(&conn, "pantry_item"), 2);
        let undone = set_pantry_marks(&mut conn, &hid("h"), &changed, false).unwrap();
        assert_eq!(undone, vec![custom_ref("c-h")]);
        assert_eq!(marked_names(&pantry(&conn, "h")), vec!["flour"]);
    }

    #[test]
    fn bulk_marking_is_idempotent_and_reversible() {
        let mut conn = open(":memory:").unwrap();
        seed_for_pantry(&mut conn, "h");
        let refs = [catalog_ref("flour"), custom_ref("c-h")];
        assert_eq!(
            set_pantry_marks(&mut conn, &hid("h"), &refs, true).unwrap(),
            refs.to_vec()
        );
        assert!(set_pantry_marks(&mut conn, &hid("h"), &refs, true)
            .unwrap()
            .is_empty());
        assert_eq!(count(&conn, "pantry_item"), 2);
        assert_eq!(
            set_pantry_marks(&mut conn, &hid("h"), &refs, false).unwrap(),
            refs.to_vec()
        );
        assert!(set_pantry_marks(&mut conn, &hid("h"), &refs, false)
            .unwrap()
            .is_empty());
        assert_eq!(count(&conn, "pantry_item"), 0);
        assert!(set_pantry_marks(&mut conn, &hid("h"), &[], true)
            .unwrap()
            .is_empty());
    }

    // --- MVP-015 step 4: the shopping snapshot -------------------------------------------

    fn shopping_lines(list: &ShoppingList) -> Vec<&ShoppingLine> {
        list.groups.iter().flat_map(|g| g.lines.iter()).collect()
    }

    fn snapshot(conn: &mut Connection, household: &str, from: &str, to: &str) -> ShoppingInput {
        load_shopping_input(
            conn,
            &hid(household),
            parse_civil_date(from).unwrap(),
            parse_civil_date(to).unwrap(),
        )
        .unwrap()
    }

    /// `h` with the lunch+dinner cycle, catalog `i-oil` (aisle) and `i-salt` (aisle), custom
    /// `c-h` (no category), recipe `shop-h` naming all three plus an unresolved line.
    fn seed_for_shopping(conn: &mut Connection, household: &str) {
        seed_for_meals(conn, household);
        for i in catalog() {
            upsert_ingredient(conn, &i).unwrap();
        }
        upsert_custom_ingredient(conn, &custom(&format!("c-{household}"), household, "mix"))
            .unwrap();
        let lines = vec![
            full_line(
                "1 cup oil",
                "oil",
                Some(IngredientRef::Catalog(IngredientId::new("i-oil").unwrap())),
                Quantity::Exact(rat(1, 1)),
                Unit::Known(UnitKind::Cup),
                None,
                false,
            ),
            full_line(
                "1 tsp salt",
                "salt",
                Some(IngredientRef::Catalog(IngredientId::new("i-salt").unwrap())),
                Quantity::Exact(rat(1, 1)),
                Unit::Known(UnitKind::Teaspoon),
                None,
                false,
            ),
            full_line(
                "2 handfuls mix",
                "mix",
                Some(IngredientRef::Custom(
                    CustomIngredientId::new(format!("c-{household}")).unwrap(),
                )),
                Quantity::Exact(rat(2, 1)),
                Unit::Other("handful".to_owned()),
                None,
                false,
            ),
            text_line("a splash of something"),
        ];
        save_recipe(
            conn,
            &recipe(household, &format!("shop-{household}"), lines),
        )
        .unwrap();
    }

    fn shop_dinner(household: &str, id: &str, date: &str, numer: u32, denom: u32) -> PlannedMeal {
        occurrence(
            household,
            id,
            date,
            MealSlot::Dinner,
            vec![
                scaled(&format!("shop-{household}"), numer, denom),
                leftovers("chili"),
            ],
        )
    }

    #[test]
    fn an_inclusive_date_range_selects_exactly_the_planned_occurrences() {
        let mut conn = open(":memory:").unwrap();
        seed_for_shopping(&mut conn, "h");
        for (id, date) in [
            ("a", "2026-08-28"),
            ("b", "2026-08-29"),
            ("c", "2026-08-31"),
            ("d", "2026-09-01"),
        ] {
            user_save(&mut conn, &shop_dinner("h", id, date, 1, 1)).unwrap();
        }
        let input = snapshot(&mut conn, "h", "2026-08-29", "2026-08-31");
        let ids: Vec<&str> = input.meals.iter().map(|m| m.id().as_str()).collect();
        assert_eq!(ids, ["b", "c"]);
        assert_eq!(
            input.recipes.len(),
            1,
            "one recipe loaded once, not per meal"
        );
        assert_eq!(input.recipes[0].id().as_str(), "shop-h");
        let list = load_shopping_list(
            &mut conn,
            &hid("h"),
            parse_civil_date("2026-08-29").unwrap(),
            parse_civil_date("2026-08-31").unwrap(),
        )
        .unwrap();
        assert_eq!(list.contribution_count, 8);
        assert_eq!(list.non_recipe_components, 2);
        let oil = shopping_lines(&list)
            .into_iter()
            .find(|l| l.name == "olive oil")
            .unwrap();
        assert_eq!(oil.quantity, Quantity::Exact(rat(2, 1)));
        // Inverted range: empty, by `list_planned_meals` precedent — not an error.
        let inverted = snapshot(&mut conn, "h", "2026-08-31", "2026-08-29");
        assert!(inverted.meals.is_empty());
        assert!(inverted.recipes.is_empty());
    }

    #[test]
    fn store_categories_come_from_the_catalog_and_custom_rows() {
        let mut conn = open(":memory:").unwrap();
        seed_for_shopping(&mut conn, "h");
        user_save(&mut conn, &shop_dinner("h", "pm", "2026-08-29", 1, 1)).unwrap();
        let input = snapshot(&mut conn, "h", "2026-08-29", "2026-08-29");
        let mut identities: Vec<(String, Option<String>)> = input
            .identities
            .iter()
            .map(|(_, i)| (i.name.clone(), i.store_category.clone()))
            .collect();
        identities.sort();
        assert_eq!(
            identities,
            vec![
                ("mix".to_owned(), None),
                ("olive oil".to_owned(), Some("aisle".to_owned())),
                ("salt".to_owned(), Some("aisle".to_owned())),
            ]
        );
        let list = derive_shopping_list(&input);
        let categories: Vec<Option<&str>> =
            list.groups.iter().map(|g| g.category.as_deref()).collect();
        assert_eq!(categories, vec![Some("aisle"), None]);
    }

    #[test]
    fn a_marked_pantry_identity_arrives_in_the_snapshot() {
        let mut conn = open(":memory:").unwrap();
        seed_for_shopping(&mut conn, "h");
        user_save(&mut conn, &shop_dinner("h", "pm", "2026-08-29", 1, 1)).unwrap();
        let salt = IngredientRef::Catalog(IngredientId::new("i-salt").unwrap());
        set_pantry_mark(&mut conn, &hid("h"), &salt, true).unwrap();
        let input = snapshot(&mut conn, "h", "2026-08-29", "2026-08-29");
        assert_eq!(input.pantry_marked, vec![salt]);
        let list = derive_shopping_list(&input);
        let lines = shopping_lines(&list);
        assert_eq!(lines.len(), 4);
        let salt_line = lines.iter().find(|l| l.name == "salt").unwrap();
        assert_eq!(salt_line.status, LineStatus::OmittedPantryMarked);
        assert!(lines
            .iter()
            .filter(|l| l.name != "salt")
            .all(|l| l.status == LineStatus::Needed));
    }

    #[test]
    fn shopping_snapshot_reads_an_archived_recipes_lines() {
        let mut conn = open(":memory:").unwrap();
        seed_for_shopping(&mut conn, "h");
        user_save(&mut conn, &shop_dinner("h", "pm", "2026-08-29", 1, 1)).unwrap();
        archive(&mut conn, "h", "shop-h", "2026-08-29").unwrap();
        let input = snapshot(&mut conn, "h", "2026-08-29", "2026-08-29");
        assert_eq!(input.recipes.len(), 1);
        assert_eq!(input.recipes[0].lines().len(), 4);
        assert_eq!(derive_shopping_list(&input).contribution_count, 4);
    }

    #[test]
    fn shopping_snapshot_for_an_absent_household_is_rejected() {
        let mut conn = open(":memory:").unwrap();
        let err = load_shopping_input(
            &mut conn,
            &hid("ghost"),
            parse_civil_date("2026-08-29").unwrap(),
            parse_civil_date("2026-08-29").unwrap(),
        )
        .unwrap_err();
        assert!(
            matches!(err, StorageError::NoSuchHousehold(ref h) if h == "ghost"),
            "{err:?}"
        );
    }

    /// Adversarial: h2's meals, pantry marks and custom ingredient never reach h1's snapshot.
    #[test]
    fn shopping_snapshot_is_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed_for_shopping(&mut conn, "h");
        seed_for_shopping(&mut conn, "h2");
        user_save(&mut conn, &shop_dinner("h", "pm-h", "2026-08-29", 1, 1)).unwrap();
        user_save(&mut conn, &shop_dinner("h2", "pm-h2", "2026-08-29", 3, 1)).unwrap();
        let oil = IngredientRef::Catalog(IngredientId::new("i-oil").unwrap());
        set_pantry_mark(&mut conn, &hid("h2"), &oil, true).unwrap();
        let input = snapshot(&mut conn, "h", "2026-08-29", "2026-08-29");
        assert_eq!(input.meals.len(), 1);
        assert_eq!(input.meals[0].id().as_str(), "pm-h");
        assert_eq!(input.recipes.len(), 1);
        assert_eq!(input.recipes[0].household_id().as_str(), "h");
        assert!(input.pantry_marked.is_empty());
        assert!(input
            .identities
            .iter()
            .all(|(r, _)| r != &IngredientRef::Custom(CustomIngredientId::new("c-h2").unwrap())));
        let list = derive_shopping_list(&input);
        assert_eq!(list.contribution_count, 4);
        let oil_line = shopping_lines(&list)
            .into_iter()
            .find(|l| l.name == "olive oil")
            .unwrap();
        assert_eq!(oil_line.status, LineStatus::Needed);
        assert_eq!(oil_line.quantity, Quantity::Exact(rat(1, 1)));
    }
}
