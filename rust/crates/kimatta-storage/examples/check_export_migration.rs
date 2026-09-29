//! Proves a real database export migrates forward without losing data.
//!
//! Usage: `cargo run -p kimatta-storage --example check_export_migration -- <export.db>`
//!
//! An example rather than a test because its only meaningful inputs are real household exports,
//! which are personal data and never committed; a test would have to skip when they are absent,
//! and a skipping test on the one input that matters is a green that proves nothing. `cargo test`
//! still compiles examples, so this cannot rot.
//!
//! The input file is only ever read: it is copied once, and the second copy is taken from the
//! first, so nothing here opens the user's file read-write.
//!
//! ponytail: row survival only — no column-value diffing. Migration 13 rewrites
//! `recipe_ingredient_line.ingredient_id` without changing any count, and that effect is covered
//! on-device by `tools/ac1_migration_check.sh`. Widen this only when a migration starts rewriting
//! content columns whose correctness nothing else pins.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use kimatta_storage::{open, schema_version, validate_export};
use rusqlite::Connection;

/// Migrations 11 and 12 remove occurrences of a withdrawn recipe only from this date onward;
/// history is preserved. Duplicated here because the cut-off is the one thing a caller cannot
/// read back out of the migrated database.
const QUARANTINE_CUTOFF: &str = "2026-09-09";

/// A foreign-key violation as `PRAGMA foreign_key_check` reports it. `rowid` is `None` for a
/// `WITHOUT ROWID` table.
type Violation = (String, Option<i64>, String, i64);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input: PathBuf = std::env::args()
        .nth(1)
        .ok_or("usage: check_export_migration <export.db>")?
        .into();

    // The production gate, read-only: SQLite header, `household` table, integrity, and a refusal
    // of anything newer than this build.
    let from = validate_export(&input)?;
    let latest = schema_version(&open(":memory:")?)?;
    if from == latest {
        return Err(format!(
            "export is already schema {latest}; this check would prove nothing about migration"
        )
        .into());
    }

    let staging = std::env::temp_dir().join(format!("kimatta-export-check-{}", std::process::id()));
    std::fs::create_dir_all(&staging)?;
    let migrated = staging.join("migrated.db");
    let baseline = staging.join("baseline.db");
    std::fs::copy(&input, &migrated)?;
    std::fs::copy(&migrated, &baseline)?;

    let (report, failures) = inspect(&migrated, &baseline, latest)?;
    print!("{report}");

    if failures.is_empty() {
        std::fs::remove_dir_all(&staging)?;
        println!(
            "OK: {} migrates {from} -> {latest} with no unexplained row loss",
            input.display()
        );
        return Ok(());
    }
    for failure in &failures {
        println!("FAIL: {failure}");
    }
    println!(
        "\nstaging kept for diagnosis: {}\n  delete it when you are done — it holds a real \
         household export",
        staging.display()
    );
    Err(format!("{} check(s) failed", failures.len()).into())
}

/// Migrates the staged copy and compares it against the untouched baseline. Returns the human
/// report and every failure found — all of them, not just the first, so one run gives the whole
/// picture.
fn inspect(
    migrated: &Path,
    baseline: &Path,
    latest: u32,
) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
    let conn = open(migrated)?;
    let reached = schema_version(&conn)?;
    let mut failures = Vec::new();
    let mut report = String::new();

    if reached != latest {
        failures.push(format!(
            "migrated copy is schema {reached}, expected {latest}"
        ));
    }
    let verdict: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if verdict != "ok" {
        failures.push(format!("migrated copy fails integrity_check: {verdict}"));
    }

    conn.execute("ATTACH DATABASE ?1 AS src", [baseline.to_string_lossy()])?;

    let before = tables(&conn, "src")?;
    let after = tables(&conn, "main")?;
    if !before.contains("household") || count(&conn, "src", "household")? == 0 {
        // Without this a silently empty baseline would make every 0 -> 0 comparison pass.
        failures.push("baseline holds no household row; the copy is not a real export".to_owned());
    }

    let explained_meals = explained_deletions(&conn, &before, &after)?;

    writeln!(report, "table                          before   after")?;
    for table in &before {
        if !after.contains(table) {
            failures.push(format!("migration dropped table {table}"));
            continue;
        }
        let b = count(&conn, "src", table)?;
        let a = count(&conn, "main", table)?;
        let mut note = String::new();
        if a > b {
            // Growth is never a failure: a migration inserting rows is ordinary, and no stranding
            // defect produces it. Only shrinkage needs explaining.
            note = "added by migration".to_owned();
        } else if a < b {
            match table.as_str() {
                "planned_meal" => match unexplained_meals(&conn, &explained_meals)? {
                    empty if empty.is_empty() => {
                        note = format!("{} removed, all explained by quarantine", b - a);
                    }
                    unexplained => {
                        for (id, date) in &unexplained {
                            failures.push(format!(
                                "planned_meal {id} ({date}) disappeared and is not explained by \
                                 the rights quarantine"
                            ));
                        }
                    }
                },
                "meal_component" => {
                    let (swept, pre_existing, unexplained) =
                        component_losses(&conn, &explained_meals)?;
                    note = format!("{swept} swept with their planned meal");
                    if pre_existing > 0 {
                        let _ = write!(note, ", {pre_existing} pre-existing orphans swept");
                    }
                    for row in &unexplained {
                        failures.push(format!(
                            "meal_component for planned_meal {row} disappeared and belongs to no \
                             removed meal"
                        ));
                    }
                }
                _ => failures.push(format!("{table} lost {} row(s)", b - a)),
            }
        }
        writeln!(report, "{table:<30} {b:>6}  {a:>6}  {note}")?;
    }
    for table in &after {
        if !before.contains(table) {
            writeln!(report, "{table:<30}      -       -  added by migration")?;
        }
    }

    // One rule, not a count comparison: counts cannot tell one violation fixed from another
    // introduced. A violation the baseline already had is reported, never blamed on the migration.
    let was = violations(&conn, "src")?;
    let now = violations(&conn, "main")?;
    for v in now.difference(&was) {
        failures.push(format!(
            "migration introduced a foreign-key violation: table {}, parent {}, fkid {}",
            v.0, v.2, v.3
        ));
    }
    let persisted = now.intersection(&was).count();
    writeln!(
        report,
        "\nforeign_key_check: {} introduced, {persisted} pre-existing and not introduced here",
        now.difference(&was).count()
    )?;

    Ok((report, failures))
}

/// Every real table in one attached database. Indexes and triggers also live in `sqlite_master`,
/// and `SELECT count(*)` against one errors "no such table", so the `type` filter is load-bearing
/// rather than tidiness.
fn tables(conn: &Connection, schema: &str) -> rusqlite::Result<BTreeSet<String>> {
    let sql = format!(
        "SELECT name FROM {schema}.sqlite_master
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
    );
    conn.prepare(&sql)?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect()
}

fn count(conn: &Connection, schema: &str, table: &str) -> Result<i64, Box<dyn std::error::Error>> {
    if table.contains('"') {
        // A dev tool pointed at your own database, not a parser for hostile input.
        return Err(format!("refusing to quote table name {table:?}").into());
    }
    Ok(conn.query_row(
        &format!("SELECT count(*) FROM {schema}.\"{table}\""),
        [],
        |r| r.get(0),
    )?)
}

/// Planned meals the baseline holds and the migrated copy does not, which migrations 11/12 are
/// entitled to have removed: dated from the cut-off onward, and carrying a component whose recipe
/// is now quarantined.
///
/// The components are read from the **baseline**: migration 15 sweeps them out of the migrated
/// copy, so looking there would leave every deletion unexplained.
fn explained_deletions(
    conn: &Connection,
    before: &BTreeSet<String>,
    after: &BTreeSet<String>,
) -> rusqlite::Result<BTreeSet<String>> {
    if !before.contains("planned_meal") || !after.contains("recipe_quarantine") {
        return Ok(BTreeSet::new());
    }
    let sql = "SELECT DISTINCT p.id FROM src.planned_meal p
               JOIN src.meal_component c ON c.planned_meal_id = p.id
               JOIN main.recipe_quarantine q ON q.recipe_id = c.recipe_id
               WHERE p.date >= ?1
                 AND p.id NOT IN (SELECT id FROM main.planned_meal)";
    conn.prepare(sql)?
        .query_map([QUARANTINE_CUTOFF], |r| r.get::<_, String>(0))?
        .collect()
}

/// Planned meals that vanished with no quarantine explanation — the failure case.
fn unexplained_meals(
    conn: &Connection,
    explained: &BTreeSet<String>,
) -> rusqlite::Result<Vec<(String, String)>> {
    let sql = "SELECT p.id, p.date FROM src.planned_meal p
               WHERE p.id NOT IN (SELECT id FROM main.planned_meal) ORDER BY p.date";
    let rows: Vec<(String, String)> = conn
        .prepare(sql)?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows
        .into_iter()
        .filter(|(id, _)| !explained.contains(id))
        .collect())
}

/// Components the migrated copy lost, split three ways: swept along with a planned meal whose
/// removal is explained, swept as an orphan the baseline already carried, and anything else.
fn component_losses(
    conn: &Connection,
    explained: &BTreeSet<String>,
) -> rusqlite::Result<(usize, usize, Vec<String>)> {
    let sql = "SELECT c.planned_meal_id,
                      c.planned_meal_id NOT IN (SELECT id FROM src.planned_meal)
               FROM src.meal_component c
               WHERE NOT EXISTS (
                   SELECT 1 FROM main.meal_component m
                   WHERE m.planned_meal_id = c.planned_meal_id AND m.position = c.position
               )";
    let rows: Vec<(String, bool)> = conn
        .prepare(sql)?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;

    let mut swept = 0;
    let mut pre_existing = 0;
    let mut unexplained = Vec::new();
    for (parent, was_orphan) in rows {
        if was_orphan {
            pre_existing += 1;
        } else if explained.contains(&parent) {
            swept += 1;
        } else {
            unexplained.push(parent);
        }
    }
    Ok((swept, pre_existing, unexplained))
}

fn violations(conn: &Connection, schema: &str) -> rusqlite::Result<BTreeSet<Violation>> {
    conn.prepare(&format!("PRAGMA {schema}.foreign_key_check"))?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect()
}
