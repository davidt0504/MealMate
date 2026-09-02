//! Kernel policy and ledger persistence, the planner's snapshot loader and the one applier
//! (MVP-023). The planner itself never sees a connection: `load_planning_snapshot` reads
//! everything in one transaction, `food_domain::planner::cover_cycle` is pure, and
//! `apply_plan_and_record` writes the result and its ledger row in one transaction.

use std::collections::BTreeMap;

use food_domain::planner::{FoodPolicies, PlanningSnapshot, ProposedMeal, RecipeCandidateInfo};
use household_core::{
    EvidenceSource, HouseholdId, LedgerEntry, LedgerEntryId, MemberId, OutcomeStatus, Policy,
    PolicyId, ReasonCode,
};
use jiff_span::days;
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};

use crate::{
    delete_planned_meal_in, list_marked_pantry_refs, list_planned_meals, load_household_by_id,
    load_member_preferences, load_planning_cycle, load_restrictions, require_household,
    save_planned_meal_in, shipped_starter_content, CivilDate, PlannedMeal, PlannedMealId,
    StorageError, WriteSource,
};

/// Calendar arithmetic through `food_domain`'s re-exported date type only; this module adds
/// no `jiff` dependency of its own.
mod jiff_span {
    use crate::CivilDate;

    /// `date ± n` days, or `None` past the calendar. The one caller drops the history window
    /// whole when either bound is `None` — it does not shorten it.
    pub fn days(date: CivilDate, n: i64) -> Option<CivilDate> {
        let mut out = date;
        let step = if n < 0 { -1 } else { 1 };
        for _ in 0..n.abs() {
            out = if step < 0 {
                out.yesterday().ok()?
            } else {
                out.tomorrow().ok()?
            };
        }
        Some(out)
    }
}

// --- policies ---------------------------------------------------------------------------------------

fn encode_parameters(parameters: &BTreeMap<String, String>) -> String {
    parameters
        .iter()
        .map(|(k, v)| format!("{k}\t{v}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The inverse of `encode_parameters`. A stored line with no tab could not have been written
/// by the encoder, so it is reported rather than dropped — the same treatment `list_policies`
/// gives a corrupt `source` and `list_ledger_entries` gives a corrupt status token. An empty
/// blob is an empty map, and `"k\t"` is `("k", "")`; both are well-formed.
fn decode_parameters(id: &str, raw: &str) -> Result<BTreeMap<String, String>, StorageError> {
    raw.lines()
        .map(|line| {
            line.split_once('\t')
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .ok_or_else(|| StorageError::CorruptRow {
                    table: "policy",
                    id: id.to_owned(),
                    column: "parameters",
                    value: line.to_owned(),
                })
        })
        .collect()
}

/// Inserts or wholly replaces one policy in one IMMEDIATE transaction; an id owned by another
/// household is `NoSuchHousehold` for the caller's household rather than hijacked.
pub fn save_policy(conn: &mut Connection, policy: &Policy) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, &policy.household_id)?;
    let owner: Option<String> = tx
        .query_row(
            "SELECT household_id FROM policy WHERE id = ?1",
            params![policy.id.as_str()],
            |r| r.get(0),
        )
        .optional()?;
    if owner.is_some_and(|o| o != policy.household_id.as_str()) {
        return Err(StorageError::NoSuchHousehold(
            policy.household_id.as_str().to_owned(),
        ));
    }
    tx.execute(
        "INSERT INTO policy (id, household_id, domain, policy_type, parameters, enabled, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
             domain = excluded.domain, policy_type = excluded.policy_type,
             parameters = excluded.parameters, enabled = excluded.enabled,
             source = excluded.source",
        params![
            policy.id.as_str(),
            policy.household_id.as_str(),
            policy.domain,
            policy.policy_type,
            encode_parameters(&policy.parameters),
            policy.enabled,
            policy.source.as_str(),
        ],
    )?;
    tx.commit()?;
    Ok(())
}

/// This household's policies in `domain`, by `policy_type` then id. Every row goes back
/// through `Policy::new`, so a stored row that could not be a policy is reported.
pub fn list_policies(
    conn: &Connection,
    household: &HouseholdId,
    domain: &str,
) -> Result<Vec<Policy>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, policy_type, parameters, enabled, source FROM policy
         WHERE household_id = ?1 AND domain = ?2 ORDER BY policy_type, id",
    )?;
    let rows = stmt
        .query_map(params![household.as_str(), domain], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, bool>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(|(id, policy_type, parameters, enabled, source)| {
            let source =
                EvidenceSource::parse(&source).ok_or_else(|| StorageError::CorruptRow {
                    table: "policy",
                    id: id.clone(),
                    column: "source",
                    value: source.clone(),
                })?;
            // Bound before `Policy::new` so the parameter decode can still borrow `id`.
            let policy_id = PolicyId::new(id.clone())?;
            Ok(Policy::new(
                policy_id,
                household.clone(),
                domain,
                policy_type,
                decode_parameters(&id, &parameters)?,
                enabled,
                source,
            )?)
        })
        .collect()
}

// --- ledger -----------------------------------------------------------------------------------------

/// Appends one row at `seq = max + 1` inside the caller's transaction. Nothing here can
/// update or delete, and no INSERT can land on an existing `id` or `seq`: the v10 triggers
/// refuse all three at the engine, so `INSERT OR REPLACE` cannot quietly rewrite or evict a
/// row either.
pub fn append_ledger_entry_in(
    tx: &Transaction<'_>,
    entry: &LedgerEntry,
) -> Result<(), StorageError> {
    require_household(tx, &entry.household_id)?;
    let seq: i64 = tx.query_row(
        "SELECT COALESCE(MAX(seq), 0) + 1 FROM controller_ledger",
        [],
        |r| r.get(0),
    )?;
    tx.execute(
        "INSERT INTO controller_ledger
         (id, seq, household_id, controller_id, algorithm_version, snapshot_hash,
          reason_codes, selected_action, prior_status, resulting_status, payload)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            entry.id.as_str(),
            seq,
            entry.household_id.as_str(),
            entry.controller_id,
            entry.algorithm_version,
            entry.snapshot_hash,
            entry
                .reason_codes
                .iter()
                .map(ReasonCode::as_str)
                .collect::<Vec<_>>()
                .join(" "),
            entry.selected_action,
            entry.prior_status.as_str(),
            entry.resulting_status.as_str(),
            entry.payload,
        ],
    )?;
    Ok(())
}

pub fn append_ledger_entry(conn: &mut Connection, entry: &LedgerEntry) -> Result<(), StorageError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    append_ledger_entry_in(&tx, entry)?;
    tx.commit()?;
    Ok(())
}

/// This household's ledger rows in `seq` order, with the sequence number beside each.
pub fn list_ledger_entries(
    conn: &Connection,
    household: &HouseholdId,
) -> Result<Vec<(i64, LedgerEntry)>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, seq, controller_id, algorithm_version, snapshot_hash, reason_codes,
                selected_action, prior_status, resulting_status, payload
         FROM controller_ledger WHERE household_id = ?1 ORDER BY seq",
    )?;
    let rows = stmt
        .query_map(params![household.as_str()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, u32>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, String>(9)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    rows.into_iter()
        .map(
            |(
                id,
                seq,
                controller_id,
                algorithm_version,
                snapshot_hash,
                codes,
                action,
                prior,
                resulting,
                payload,
            )| {
                let status = |column: &'static str, value: &str| {
                    OutcomeStatus::parse(value).ok_or_else(|| StorageError::CorruptRow {
                        table: "controller_ledger",
                        id: id.clone(),
                        column,
                        value: value.to_owned(),
                    })
                };
                let prior_status = status("prior_status", &prior)?;
                let resulting_status = status("resulting_status", &resulting)?;
                let reason_codes = codes
                    .split_whitespace()
                    .map(ReasonCode::new)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((
                    seq,
                    LedgerEntry {
                        id: LedgerEntryId::new(id.clone())?,
                        household_id: household.clone(),
                        controller_id,
                        algorithm_version,
                        snapshot_hash,
                        reason_codes,
                        selected_action: action,
                        prior_status,
                        resulting_status,
                        payload,
                    },
                ))
            },
        )
        .collect()
}

// --- apply ------------------------------------------------------------------------------------------

/// Writes the proposed plan and its ledger row in **one** IMMEDIATE transaction. Per slot:
/// a slot whose stored components already equal the proposal is skipped (§9.9 — an
/// unchanged slot keeps its id); otherwise the existing unlocked occurrence is deleted and
/// the proposal saved as `Automation` under `ids[i]`. A locked occurrence that differs is
/// `LockedPlannedMeal`, and the transaction — meals and ledger — rolls back whole. Returns
/// the number of slots changed. `ids` must carry one fresh id per proposed slot; a short
/// slice is `IdCountMismatch` before anything is read or written.
///
/// The snapshot the plan was built from is re-derived inside this transaction and its hash
/// compared with `entry.snapshot_hash`: planning is a separate, earlier transaction, so a
/// concurrent edit between the two would otherwise be overwritten as `Automation` with no
/// error. A mismatch is `StalePlan` and rolls the whole transaction back. That re-derivation
/// also makes two failures reachable here that apply never had before — `NoPlanningCycle`, if
/// the cycle was deleted between plan and apply, and a `PlanningError` from
/// `window_containing` if `offset_cycles` no longer lands on the calendar.
///
/// Slot *resolution* (a lock, or an explicit `Open`) is not re-checked here: `tier0::filter`
/// keeps only the `ExistingPlan` candidate for a resolved slot, so the proposal equals the
/// stored components and the unchanged-slot skip below fires. The *lock* half is additionally
/// enforced at the engine, through `delete_planned_meal_in` → `LockedPlannedMeal`.
pub fn apply_plan_and_record(
    conn: &mut Connection,
    proposed: &[ProposedMeal],
    ids: &[PlannedMealId],
    entry: &LedgerEntry,
    today: CivilDate,
    offset_cycles: i32,
) -> Result<usize, StorageError> {
    let household = &entry.household_id;
    // Cheapest check first: an arity error needs no transaction and no snapshot read.
    if ids.len() < proposed.len() {
        return Err(StorageError::IdCountMismatch {
            expected: proposed.len(),
            got: ids.len(),
        });
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    require_household(&tx, household)?;
    let found = load_planning_snapshot_in(&tx, household, today, offset_cycles)?.snapshot_hash();
    if found != entry.snapshot_hash {
        return Err(StorageError::StalePlan {
            expected: entry.snapshot_hash.clone(),
            found,
        });
    }
    let mut changed = 0;
    for (index, meal) in proposed.iter().enumerate() {
        let existing = tx
            .query_row(
                "SELECT id FROM planned_meal WHERE household_id = ?1 AND date = ?2 AND slot = ?3",
                params![
                    household.as_str(),
                    crate::format_civil_date(meal.date),
                    meal.slot.as_str()
                ],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .map(PlannedMealId::new)
            .transpose()?;
        if let Some(id) = &existing {
            let stored = crate::load_planned_meal(&tx, household, id)?;
            if stored
                .as_ref()
                .is_some_and(|s| s.components() == meal.components.as_slice())
            {
                continue;
            }
            delete_planned_meal_in(&tx, household, id, WriteSource::Automation)?;
        }
        let Some(id) = ids.get(index) else {
            return Err(StorageError::IdCountMismatch {
                expected: proposed.len(),
                got: ids.len(),
            });
        };
        let occurrence = PlannedMeal::new(
            id.clone(),
            household.clone(),
            meal.date,
            meal.slot,
            meal.components.clone(),
            false,
        )?;
        save_planned_meal_in(&tx, &occurrence, WriteSource::Automation)?;
        changed += 1;
    }
    append_ledger_entry_in(&tx, entry)?;
    tx.commit()?;
    Ok(changed)
}

// --- snapshot ---------------------------------------------------------------------------------------

/// The seven planner-visible fields of every active recipe, in **two** queries rather than the
/// one-per-recipe `load_recipe` fan-out the loader used to do — that materialised the whole
/// record (instructions, provenance, every parsed quantity and unit) to keep seven fields, so a
/// 200-recipe household issued 201 queries per `cover_cycle`.
///
/// The projection reproduces `load_recipe`'s three load-bearing behaviours exactly: `Active` is
/// `archived_at IS NULL`, line order is `ORDER BY position` (it feeds `line_names`, the veto
/// matcher's input and the snapshot hash), and a line's ref is catalog or custom by which of the
/// two nullable columns is set — the table's own `CHECK` at `lib.rs:264` makes that *at most* one,
/// so a line with neither column set is legal and is the `(None, None)` arm below. A
/// recipe with no `recipe_provenance` row stays `CorruptProvenance`, as `load_recipe` reports it,
/// rather than being silently dropped by an inner join.
fn list_recipe_candidate_info(
    tx: &Transaction<'_>,
    household: &HouseholdId,
) -> Result<Vec<RecipeCandidateInfo>, StorageError> {
    let mut by_recipe: BTreeMap<String, (Vec<String>, Vec<crate::IngredientRef>)> = BTreeMap::new();
    let mut stmt = tx.prepare(
        "SELECT l.recipe_id, l.name, l.ingredient_id, l.custom_ingredient_id
         FROM recipe_ingredient_line l
         JOIN recipe r ON r.id = l.recipe_id
         WHERE r.household_id = ?1 AND r.archived_at IS NULL
         ORDER BY l.recipe_id, l.position",
    )?;
    let lines = stmt
        .query_map(params![household.as_str()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (recipe_id, name, catalog, custom) in lines {
        let entry = by_recipe.entry(recipe_id).or_default();
        entry.0.push(name);
        match (catalog, custom) {
            (Some(id), _) => entry
                .1
                .push(crate::IngredientRef::Catalog(crate::IngredientId::new(
                    &id,
                )?)),
            (None, Some(id)) => entry.1.push(crate::IngredientRef::Custom(
                crate::CustomIngredientId::new(&id)?,
            )),
            (None, None) => {}
        }
    }
    let mut stmt = tx.prepare(
        "SELECT r.id, r.title, r.servings, r.prep_minutes, p.recipe_id, p.starter_slug
         FROM recipe r
         LEFT JOIN recipe_provenance p ON p.recipe_id = r.id
         WHERE r.household_id = ?1 AND r.archived_at IS NULL",
    )?;
    let rows = stmt
        .query_map(params![household.as_str()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<u32>>(2)?,
                r.get::<_, Option<u32>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (id, title, servings, prep_minutes, provenance_id, starter_slug) in rows {
        if provenance_id.is_none() {
            return Err(StorageError::CorruptProvenance(id));
        }
        let (line_names, ingredient_refs) = by_recipe.remove(&id).unwrap_or_default();
        out.push(RecipeCandidateInfo {
            id: crate::RecipeId::new(&id)?,
            title,
            servings,
            prep_minutes,
            line_names,
            ingredient_refs,
            starter_slug,
        });
    }
    Ok(out)
}

/// Everything the planner reads, from one read transaction, every collection in the
/// canonical order `PlanningSnapshot` documents. The window is the one `offset_cycles` away
/// from the one containing `today`; a household with no cycle yet is `NoPlanningCycle`.
pub fn load_planning_snapshot(
    conn: &mut Connection,
    household: &HouseholdId,
    today: CivilDate,
    offset_cycles: i32,
) -> Result<PlanningSnapshot, StorageError> {
    let tx = conn.transaction()?;
    let snapshot = load_planning_snapshot_in(&tx, household, today, offset_cycles)?;
    tx.commit()?;
    Ok(snapshot)
}

/// The same read inside the caller's transaction, so an applier can re-derive the snapshot it
/// planned from without releasing its write lock. Extracted for the same reason
/// `append_ledger_entry_in` and `save_planned_meal_in` are.
pub fn load_planning_snapshot_in(
    tx: &Transaction<'_>,
    household: &HouseholdId,
    today: CivilDate,
    offset_cycles: i32,
) -> Result<PlanningSnapshot, StorageError> {
    require_household(tx, household)?;
    let cycle = load_planning_cycle(tx, household)?
        .ok_or_else(|| StorageError::NoPlanningCycle(household.as_str().to_owned()))?;
    let window = cycle.window_containing(today, offset_cycles)?;
    let dates = window.dates();
    let (anchor, last) = (dates[0], *dates.last().expect("non-empty"));
    let record = load_household_by_id(tx, household)?
        .ok_or_else(|| StorageError::NoSuchHousehold(household.as_str().to_owned()))?;
    let mut members: Vec<MemberId> = record.members.iter().map(|m| m.id.clone()).collect();
    members.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    let mut preferences = Vec::with_capacity(members.len());
    for member in &members {
        preferences.push((
            member.clone(),
            load_member_preferences(tx, household, member)?,
        ));
    }
    let restrictions = load_restrictions(tx, household)?;
    let policies = FoodPolicies::from_policies(&list_policies(tx, household, "food")?);
    let mut recipes = list_recipe_candidate_info(tx, household)?;
    recipes.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    // Held slugs include archived recipes, as `install_starter_content` counts them: an
    // archived starter was a household decision, not an invitation to offer it again.
    let held: Vec<String> = tx
        .prepare(
            "SELECT p.starter_slug FROM recipe_provenance p
             JOIN recipe r ON r.id = p.recipe_id
             WHERE r.household_id = ?1 AND p.starter_slug IS NOT NULL",
        )?
        .query_map(params![household.as_str()], |r| r.get::<_, String>(0))?
        .collect::<Result<_, _>>()?;
    let mut starter: Vec<_> = shipped_starter_content()
        .map_err(|e| StorageError::CorruptProvenance(e.to_string()))?
        .recipes
        .into_iter()
        .filter(|s| !held.contains(&s.slug))
        .collect();
    starter.sort_by(|a, b| a.slug.cmp(&b.slug));
    let existing = list_planned_meals(tx, household, anchor, last)?;
    let history = match (days(anchor, -14), days(anchor, -1)) {
        (Some(from), Some(to)) => list_planned_meals(tx, household, from, to)?,
        _ => Vec::new(),
    };
    let pantry_marked = list_marked_pantry_refs(tx, household)?;
    Ok(PlanningSnapshot {
        household_id: household.clone(),
        anchor,
        length_days: window.length_days(),
        scope: window.scope().clone(),
        today,
        members,
        restrictions,
        preferences,
        policies,
        recipes,
        starter,
        existing,
        history,
        pantry_marked,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use household_core::{Household, HouseholdMember, OutcomeStatus};

    use super::*;
    use crate::{
        insert_household, open, parse_civil_date, save_planned_meal, save_planning_cycle,
        save_recipe, save_restrictions, set_planned_meal_lock, HouseholdRestrictions,
        MealComponent, MealScope, MealSlot, PlanningCycle, ProvenanceKind, Recipe, RecipeId,
        RecipeProvenance, Restriction, RestrictionKind,
    };

    fn hid(id: &str) -> HouseholdId {
        HouseholdId::new(id).unwrap()
    }

    fn seed(conn: &mut Connection, id: &str) {
        let h = Household {
            id: hid(id),
            name: None,
        };
        let m = HouseholdMember {
            id: MemberId::new(format!("m-{id}")).unwrap(),
            household_id: h.id.clone(),
            display_name: "Me".to_owned(),
        };
        insert_household(conn, &h, std::slice::from_ref(&m)).unwrap();
        let cycle = PlanningCycle::new(
            hid(id),
            parse_civil_date("2026-08-29").unwrap(),
            3,
            MealScope::dinner_only(),
        )
        .unwrap();
        save_planning_cycle(conn, &cycle).unwrap();
        save_restrictions(
            conn,
            &hid(id),
            &HouseholdRestrictions::new([Restriction::Known(RestrictionKind::Peanuts)]),
        )
        .unwrap();
    }

    fn recipe(conn: &mut Connection, household: &str, id: &str, title: &str) {
        let r = Recipe::new(
            RecipeId::new(id).unwrap(),
            hid(household),
            title,
            Some(4),
            Some(20),
            "",
            vec![],
            RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
        )
        .unwrap();
        save_recipe(conn, &r).unwrap();
    }

    fn rc(id: &str) -> MealComponent {
        MealComponent::recipe(RecipeId::new(id).unwrap(), None)
    }

    fn meal(id: &str, household: &str, date: &str, components: Vec<MealComponent>) -> PlannedMeal {
        PlannedMeal::new(
            PlannedMealId::new(id).unwrap(),
            hid(household),
            parse_civil_date(date).unwrap(),
            MealSlot::Dinner,
            components,
            false,
        )
        .unwrap()
    }

    fn policy(
        id: &str,
        household: &str,
        ty: &str,
        params: &[(&str, &str)],
        enabled: bool,
    ) -> Policy {
        Policy::new(
            PolicyId::new(id).unwrap(),
            hid(household),
            "food",
            ty,
            params
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect::<BTreeMap<_, _>>(),
            enabled,
            EvidenceSource::ExplicitUser,
        )
        .unwrap()
    }

    /// `apply_plan_and_record` re-derives the snapshot inside its transaction and refuses a
    /// plan built from a different one, so an apply test must carry the live hash.
    fn entry_now(
        conn: &mut Connection,
        id: &str,
        household: &str,
        action: &str,
        payload: &str,
    ) -> LedgerEntry {
        let hash = load_planning_snapshot(conn, &hid(household), today(), 0)
            .unwrap()
            .snapshot_hash();
        LedgerEntry {
            snapshot_hash: hash,
            ..entry(id, household, action, payload)
        }
    }

    fn entry(id: &str, household: &str, action: &str, payload: &str) -> LedgerEntry {
        LedgerEntry {
            id: LedgerEntryId::new(id).unwrap(),
            household_id: hid(household),
            controller_id: "food".to_owned(),
            algorithm_version: 1,
            snapshot_hash: "0123456789abcdef".to_owned(),
            reason_codes: vec![
                ReasonCode::new("SLOT_COVERED").unwrap(),
                ReasonCode::new("RESTRICTION_CONFLICT:rules_v1").unwrap(),
            ],
            selected_action: action.to_owned(),
            prior_status: OutcomeStatus::Unresolved,
            resulting_status: OutcomeStatus::TentativelyCovered,
            payload: payload.to_owned(),
        }
    }

    fn today() -> CivilDate {
        parse_civil_date("2026-08-29").unwrap()
    }

    // --- Step 8: policies and the ledger --------------------------------------------------

    #[test]
    fn a_policy_round_trips_with_parameters() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        let p = policy(
            "p1",
            "h",
            FoodPolicies::SLOT_WINDOW,
            &[("slot", "dinner"), ("minutes", "45"), ("note", "a=b c")],
            true,
        );
        save_policy(&mut conn, &p).unwrap();
        assert_eq!(
            list_policies(&conn, &hid("h"), "food").unwrap(),
            vec![p.clone()]
        );
        // Replace-whole: parameters and enabled flag are overwritten, not merged.
        let mut q = policy(
            "p1",
            "h",
            FoodPolicies::SLOT_WINDOW,
            &[("slot", "lunch")],
            false,
        );
        q.source = EvidenceSource::Derived;
        save_policy(&mut conn, &q).unwrap();
        assert_eq!(list_policies(&conn, &hid("h"), "food").unwrap(), vec![q]);
        let empty = policy("p2", "h", FoodPolicies::DINING_OUT, &[], true);
        save_policy(&mut conn, &empty).unwrap();
        let listed = list_policies(&conn, &hid("h"), "food").unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(
            listed[0].policy_type,
            FoodPolicies::DINING_OUT,
            "by type then id"
        );
        assert!(listed[0].parameters.is_empty());
    }

    #[test]
    fn policies_are_household_and_domain_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        save_policy(
            &mut conn,
            &policy("a", "h1", FoodPolicies::DINING_OUT, &[], true),
        )
        .unwrap();
        save_policy(
            &mut conn,
            &policy("b", "h2", FoodPolicies::DINING_OUT, &[], true),
        )
        .unwrap();
        let mut other = policy("c", "h1", "window", &[], true);
        other.domain = "schedule".to_owned();
        save_policy(&mut conn, &other).unwrap();
        assert_eq!(list_policies(&conn, &hid("h1"), "food").unwrap().len(), 1);
        assert_eq!(list_policies(&conn, &hid("h2"), "food").unwrap().len(), 1);
        assert_eq!(
            list_policies(&conn, &hid("h1"), "schedule").unwrap().len(),
            1
        );
        // h2 cannot hijack h1's id, and a ghost household is refused.
        let err = save_policy(
            &mut conn,
            &policy("a", "h2", FoodPolicies::DINING_OUT, &[], false),
        )
        .unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)), "{err:?}");
        assert!(list_policies(&conn, &hid("h1"), "food").unwrap()[0].enabled);
        let err = save_policy(
            &mut conn,
            &policy("z", "ghost", FoodPolicies::DINING_OUT, &[], true),
        )
        .unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)), "{err:?}");
        // Deleting a household cascades its policies.
        conn.execute("DELETE FROM household WHERE id = 'h2'", [])
            .unwrap();
        assert!(list_policies(&conn, &hid("h2"), "food").unwrap().is_empty());
    }

    #[test]
    fn ledger_rows_append_in_order() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        seed(&mut conn, "h2");
        append_ledger_entry(&mut conn, &entry("l1", "h", "propose", "p1")).unwrap();
        append_ledger_entry(&mut conn, &entry("l2", "h2", "propose", "p2")).unwrap();
        append_ledger_entry(&mut conn, &entry("l3", "h", "apply", "p3")).unwrap();
        let rows = list_ledger_entries(&conn, &hid("h")).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, 1);
        assert_eq!(rows[0].1, entry("l1", "h", "propose", "p1"));
        assert_eq!(rows[1].0, 3, "seq is global, household order is preserved");
        assert_eq!(rows[1].1.selected_action, "apply");
        assert_eq!(rows[1].1.reason_codes.len(), 2);
        assert_eq!(
            rows[1].1.reason_codes[1].as_str(),
            "RESTRICTION_CONFLICT:rules_v1"
        );
        let err = append_ledger_entry(&mut conn, &entry("l1", "h", "propose", "dup")).unwrap_err();
        assert!(matches!(err, StorageError::Sqlite(_)), "{err:?}");
        let err = append_ledger_entry(&mut conn, &entry("l9", "ghost", "propose", "")).unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)), "{err:?}");
    }

    #[test]
    fn ledger_update_is_refused() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        append_ledger_entry(&mut conn, &entry("l1", "h", "propose", "p1")).unwrap();
        for sql in [
            "UPDATE controller_ledger SET payload = 'x' WHERE id = 'l1'",
            "UPDATE controller_ledger SET resulting_status = 'covered'",
            "UPDATE controller_ledger SET seq = 99",
        ] {
            let err = conn.execute(sql, []).unwrap_err();
            assert!(
                err.to_string().contains("controller_ledger is append-only"),
                "{sql}: {err}"
            );
        }
        assert_eq!(
            list_ledger_entries(&conn, &hid("h")).unwrap()[0].1.payload,
            "p1"
        );
    }

    #[test]
    fn ledger_delete_is_refused() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        append_ledger_entry(&mut conn, &entry("l1", "h", "propose", "p1")).unwrap();
        for sql in [
            "DELETE FROM controller_ledger WHERE id = 'l1'",
            "DELETE FROM controller_ledger",
        ] {
            let err = conn.execute(sql, []).unwrap_err();
            assert!(
                err.to_string().contains("controller_ledger is append-only"),
                "{sql}: {err}"
            );
        }
        assert_eq!(list_ledger_entries(&conn, &hid("h")).unwrap().len(), 1);
    }

    #[test]
    fn deleting_a_household_with_ledger_rows_is_refused() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        append_ledger_entry(&mut conn, &entry("l1", "h", "propose", "p1")).unwrap();
        let err = conn
            .execute("DELETE FROM household WHERE id = 'h'", [])
            .unwrap_err();
        assert!(
            err.to_string().contains("FOREIGN KEY constraint failed"),
            "{err}"
        );
        assert_eq!(list_ledger_entries(&conn, &hid("h")).unwrap().len(), 1);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM household", [], |r| r.get::<_, u32>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn ledger_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kimatta.db");
        {
            let mut conn = open(&path).unwrap();
            seed(&mut conn, "h");
            append_ledger_entry(&mut conn, &entry("l1", "h", "propose", "p1")).unwrap();
        }
        let conn = open(&path).unwrap();
        let rows = list_ledger_entries(&conn, &hid("h")).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, entry("l1", "h", "propose", "p1"));
    }

    #[test]
    fn a_corrupt_status_token_is_reported_not_coerced() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        conn.execute(
            "INSERT INTO controller_ledger
             (id, seq, household_id, controller_id, algorithm_version, snapshot_hash,
              reason_codes, selected_action, prior_status, resulting_status, payload)
             VALUES ('x', 1, 'h', 'food', 1, 'h', '', 'propose', 'fine', 'covered', '')",
            [],
        )
        .unwrap();
        let err = list_ledger_entries(&conn, &hid("h")).unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::CorruptRow {
                    table: "controller_ledger",
                    column: "prior_status",
                    ..
                }
            ),
            "{err:?}"
        );
    }

    // --- Step 9: the snapshot loader and the applier ----------------------------------------

    #[test]
    fn planning_snapshot_is_household_scoped() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h1");
        seed(&mut conn, "h2");
        recipe(&mut conn, "h1", "r1", "Rice");
        recipe(&mut conn, "h2", "r2", "Soup");
        save_policy(
            &mut conn,
            &policy("p", "h2", FoodPolicies::DINING_OUT, &[], true),
        )
        .unwrap();
        save_planned_meal(
            &mut conn,
            &meal("m2", "h2", "2026-08-29", vec![rc("r2")]),
            WriteSource::User,
        )
        .unwrap();
        let s = load_planning_snapshot(&mut conn, &hid("h1"), today(), 0).unwrap();
        assert_eq!(s.household_id, hid("h1"));
        assert_eq!(s.recipes.len(), 1);
        assert_eq!(s.recipes[0].id.as_str(), "r1");
        assert!(!s.policies.dining_out_enabled);
        assert!(s.existing.is_empty());
        assert_eq!(s.members, vec![MemberId::new("m-h1").unwrap()]);
        assert_eq!(s.restrictions.restrictions().len(), 1);
        assert_eq!(s.length_days, 3);
        assert_eq!(s.anchor, today());
        // Exactly the shipped (cook-reviewed) set, which is empty until MVP-011 AC-3 lands.
        assert_eq!(
            s.starter.len(),
            shipped_starter_content().unwrap().recipes.len()
        );
        let err = load_planning_snapshot(&mut conn, &hid("ghost"), today(), 0).unwrap_err();
        assert!(matches!(err, StorageError::NoSuchHousehold(_)), "{err:?}");
    }

    #[test]
    fn snapshot_reads_history_window_before_the_anchor() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        recipe(&mut conn, "h", "r1", "Rice");
        for (id, date) in [
            ("old", "2026-08-14"),
            ("in-window", "2026-08-15"),
            ("yesterday", "2026-08-28"),
            ("anchor", "2026-08-29"),
        ] {
            save_planned_meal(
                &mut conn,
                &meal(id, "h", date, vec![rc("r1")]),
                WriteSource::User,
            )
            .unwrap();
        }
        let s = load_planning_snapshot(&mut conn, &hid("h"), today(), 0).unwrap();
        let history: Vec<&str> = s.history.iter().map(|m| m.id().as_str()).collect();
        assert_eq!(history, vec!["in-window", "yesterday"]);
        assert_eq!(s.existing.len(), 1);
        assert_eq!(s.existing[0].id().as_str(), "anchor");
        // The next window: existing empty there, history now holds the anchor-week meals.
        let next = load_planning_snapshot(&mut conn, &hid("h"), today(), 1).unwrap();
        assert_eq!(next.anchor, parse_civil_date("2026-09-01").unwrap());
        assert!(next.existing.is_empty());
        assert!(next.history.iter().any(|m| m.id().as_str() == "anchor"));
    }

    #[test]
    fn snapshot_orders_every_collection_canonically() {
        let build = |ids: &[&str], members: &[&str]| -> String {
            let mut conn = open(":memory:").unwrap();
            let h = Household {
                id: hid("h"),
                name: None,
            };
            let ms: Vec<HouseholdMember> = members
                .iter()
                .map(|m| HouseholdMember {
                    id: MemberId::new(*m).unwrap(),
                    household_id: hid("h"),
                    display_name: (*m).to_owned(),
                })
                .collect();
            insert_household(&mut conn, &h, &ms).unwrap();
            save_planning_cycle(
                &mut conn,
                &PlanningCycle::new(hid("h"), today(), 3, MealScope::dinner_only()).unwrap(),
            )
            .unwrap();
            for id in ids {
                recipe(&mut conn, "h", id, &format!("T{id}"));
                save_policy(
                    &mut conn,
                    &policy(id, "h", FoodPolicies::HARD_VETO, &[("subject", id)], true),
                )
                .unwrap();
            }
            load_planning_snapshot(&mut conn, &hid("h"), today(), 0)
                .unwrap()
                .snapshot_hash()
        };
        assert_eq!(
            build(&["b", "a", "c"], &["m2", "m1"]),
            build(&["c", "b", "a"], &["m1", "m2"])
        );
        assert_ne!(build(&["a"], &["m1"]), build(&["b"], &["m1"]));
    }

    #[test]
    fn snapshot_without_a_cycle_is_no_planning_cycle() {
        let mut conn = open(":memory:").unwrap();
        let h = Household {
            id: hid("h"),
            name: None,
        };
        insert_household(&mut conn, &h, &[]).unwrap();
        let err = load_planning_snapshot(&mut conn, &hid("h"), today(), 0).unwrap_err();
        assert!(matches!(err, StorageError::NoPlanningCycle(_)), "{err:?}");
    }

    fn proposed(date: &str, components: Vec<MealComponent>) -> ProposedMeal {
        ProposedMeal {
            date: parse_civil_date(date).unwrap(),
            slot: MealSlot::Dinner,
            components,
        }
    }

    fn ids(n: usize) -> Vec<PlannedMealId> {
        (0..n)
            .map(|i| PlannedMealId::new(format!("new-{i}")).unwrap())
            .collect()
    }

    #[test]
    fn apply_replaces_only_changed_unlocked_slots_and_records_in_one_transaction() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        recipe(&mut conn, "h", "r1", "Rice");
        recipe(&mut conn, "h", "r2", "Soup");
        save_planned_meal(
            &mut conn,
            &meal("keep", "h", "2026-08-29", vec![rc("r1")]),
            WriteSource::User,
        )
        .unwrap();
        save_planned_meal(
            &mut conn,
            &meal("swap", "h", "2026-08-30", vec![rc("r1")]),
            WriteSource::User,
        )
        .unwrap();
        let plan = vec![
            proposed("2026-08-29", vec![rc("r1")]),
            proposed("2026-08-30", vec![rc("r2")]),
            proposed(
                "2026-08-31",
                vec![MealComponent::FrozenQuick { note: None }],
            ),
        ];
        let changed = {
            let e = entry_now(&mut conn, "l1", "h", "apply", "p");
            apply_plan_and_record(&mut conn, &plan, &ids(3), &e, today(), 0)
        }
        .unwrap();
        assert_eq!(changed, 2);
        let meals = list_planned_meals(
            &conn,
            &hid("h"),
            today(),
            parse_civil_date("2026-08-31").unwrap(),
        )
        .unwrap();
        let ids_after: Vec<&str> = meals.iter().map(|m| m.id().as_str()).collect();
        assert_eq!(ids_after, vec!["keep", "new-1", "new-2"]);
        assert_eq!(meals[1].components(), &[rc("r2")]);
        assert_eq!(list_ledger_entries(&conn, &hid("h")).unwrap().len(), 1);
    }

    #[test]
    fn unchanged_slots_keep_their_ids() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        recipe(&mut conn, "h", "r1", "Rice");
        save_planned_meal(
            &mut conn,
            &meal("keep", "h", "2026-08-29", vec![rc("r1")]),
            WriteSource::User,
        )
        .unwrap();
        let plan = vec![proposed("2026-08-29", vec![rc("r1")])];
        let changed = {
            let e = entry_now(&mut conn, "l1", "h", "apply", "p");
            apply_plan_and_record(&mut conn, &plan, &ids(1), &e, today(), 0)
        }
        .unwrap();
        assert_eq!(changed, 0);
        let meals = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        assert_eq!(meals[0].id().as_str(), "keep");
        assert_eq!(
            list_ledger_entries(&conn, &hid("h")).unwrap().len(),
            1,
            "still recorded"
        );
    }

    /// The projected recipe load must agree with `load_recipe` field for field: `line_names`
    /// order and `ingredient_refs` feed the veto matcher and the snapshot hash, so a
    /// mis-ordered or mis-discriminated join would change ledger identity silently. Archived
    /// recipes stay out, and a ref-less line contributes a name but no ref.
    #[test]
    fn snapshot_recipes_match_the_full_loader() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        recipe(&mut conn, "h", "r-b", "Beans");
        recipe(&mut conn, "h", "r-a", "Apples");
        // A recipe whose lines mix a catalog ref, a custom ref and a ref-less line, in an
        // order that is deliberately not alphabetical: `line_names` order feeds the veto
        // matcher and the snapshot hash, so a join that lost `ORDER BY position` would show
        // up here and nowhere else.
        conn.execute(
            "INSERT INTO ingredient (id, canonical_name) VALUES ('i-flour', 'flour')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO custom_ingredient (id, household_id, name) VALUES ('c-mix', 'h', 'mix')",
            [],
        )
        .unwrap();
        let line = |name: &str, ingredient: Option<crate::IngredientRef>| {
            crate::IngredientLine::new(
                format!("1 cup {name}"),
                name,
                ingredient,
                crate::Quantity::Unknown,
                crate::Unit::None,
                None,
                false,
            )
            .unwrap()
        };
        save_recipe(
            &mut conn,
            &Recipe::new(
                RecipeId::new("r-mixed").unwrap(),
                hid("h"),
                "Mixed",
                Some(4),
                Some(20),
                "",
                vec![
                    line("zucchini", None),
                    line(
                        "flour",
                        Some(crate::IngredientRef::Catalog(
                            crate::IngredientId::new("i-flour").unwrap(),
                        )),
                    ),
                    line(
                        "mix",
                        Some(crate::IngredientRef::Custom(
                            crate::CustomIngredientId::new("c-mix").unwrap(),
                        )),
                    ),
                ],
                RecipeProvenance::new(ProvenanceKind::Authored, None, None, None).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let s = load_planning_snapshot(&mut conn, &hid("h"), today(), 0).unwrap();
        let mixed = s
            .recipes
            .iter()
            .find(|r| r.id.as_str() == "r-mixed")
            .expect("the mixed recipe is a candidate");
        assert_eq!(mixed.line_names, vec!["zucchini", "flour", "mix"]);
        assert_eq!(
            mixed.ingredient_refs,
            vec![
                crate::IngredientRef::Catalog(crate::IngredientId::new("i-flour").unwrap()),
                crate::IngredientRef::Custom(crate::CustomIngredientId::new("c-mix").unwrap()),
            ],
            "the ref-less line contributes a name but no ref, and catalog precedes custom \
             because that is their line order"
        );
        let hash_before = s.snapshot_hash();
        let mut expected: Vec<RecipeCandidateInfo> = Vec::new();
        for summary in crate::list_recipes(&conn, &hid("h"), crate::RecipeListing::Active).unwrap()
        {
            let r = crate::load_recipe(&conn, &hid("h"), &summary.id)
                .unwrap()
                .unwrap()
                .recipe;
            expected.push(RecipeCandidateInfo {
                id: r.id().clone(),
                title: r.title().to_owned(),
                servings: r.servings(),
                prep_minutes: r.prep_minutes(),
                line_names: r.lines().iter().map(|l| l.name().to_owned()).collect(),
                ingredient_refs: r
                    .lines()
                    .iter()
                    .filter_map(crate::IngredientLine::ingredient)
                    .cloned()
                    .collect(),
                starter_slug: r.provenance().starter_slug().map(str::to_owned),
            });
        }
        expected.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        assert_eq!(s.recipes, expected);
        assert!(!s.recipes.is_empty());
        // An archived recipe is not a candidate, and dropping it changes the hash.
        crate::archive_recipe(
            &mut conn,
            &hid("h"),
            &RecipeId::new("r-b").unwrap(),
            today(),
        )
        .unwrap();
        let after = load_planning_snapshot(&mut conn, &hid("h"), today(), 0).unwrap();
        assert!(after.recipes.iter().all(|r| r.id.as_str() != "r-b"));
        assert_ne!(after.snapshot_hash(), hash_before);
    }

    /// A short `ids` slice is a caller bug, not a vanished row, and it is caught before the
    /// transaction opens — so nothing is read and nothing is written.
    #[test]
    fn apply_with_fewer_ids_than_slots_is_an_id_count_mismatch() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        recipe(&mut conn, "h", "r1", "Rice");
        let plan = vec![
            proposed("2026-08-29", vec![rc("r1")]),
            proposed("2026-08-30", vec![rc("r1")]),
        ];
        let e = entry_now(&mut conn, "l1", "h", "apply", "p");
        let err = apply_plan_and_record(&mut conn, &plan, &ids(1), &e, today(), 0).unwrap_err();
        assert!(
            matches!(
                err,
                StorageError::IdCountMismatch {
                    expected: 2,
                    got: 1
                }
            ),
            "{err:?}"
        );
        assert!(list_planned_meals(&conn, &hid("h"), today(), today())
            .unwrap()
            .is_empty());
        assert!(list_ledger_entries(&conn, &hid("h")).unwrap().is_empty());
    }

    /// Planning and applying are separate transactions, so a user edit can land between them.
    /// The applier re-derives the snapshot inside its own transaction and refuses rather than
    /// overwriting the edit as `Automation`.
    #[test]
    fn an_edit_between_planning_and_apply_is_refused_not_overwritten() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        recipe(&mut conn, "h", "r1", "Rice");
        recipe(&mut conn, "h", "r2", "Soup");
        // The plan is built from the state as it stands now.
        let e = entry_now(&mut conn, "l1", "h", "apply", "p");
        let plan = vec![proposed("2026-08-29", vec![rc("r1")])];
        // Now the user saves their own dinner into that slot.
        save_planned_meal(
            &mut conn,
            &meal("mine", "h", "2026-08-29", vec![rc("r2")]),
            WriteSource::User,
        )
        .unwrap();
        let err = apply_plan_and_record(&mut conn, &plan, &ids(1), &e, today(), 0).unwrap_err();
        let StorageError::StalePlan { expected, found } = &err else {
            panic!("{err:?}");
        };
        assert_ne!(expected, found);
        assert_eq!(expected, &e.snapshot_hash);
        // The edit survives, still the user's, and the refused run recorded nothing.
        let m = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].id().as_str(), "mine");
        assert_eq!(m[0].components(), &[rc("r2")]);
        assert!(list_ledger_entries(&conn, &hid("h")).unwrap().is_empty());
    }

    /// `INSERT OR REPLACE` skips the BEFORE DELETE trigger unless `recursive_triggers` is on,
    /// so the append-only guarantee needs its own BEFORE INSERT trigger. Two halves: a PK
    /// collision would rewrite the row, and a `seq` collision would evict an unrelated one.
    #[test]
    fn ledger_insert_or_replace_is_refused() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        append_ledger_entry(&mut conn, &entry("l1", "h", "propose", "p1")).unwrap();
        append_ledger_entry(&mut conn, &entry("l2", "h", "propose", "p2")).unwrap();
        for sql in [
            // Same id: a rewrite of row l1.
            "INSERT OR REPLACE INTO controller_ledger
             (id, seq, household_id, controller_id, algorithm_version, snapshot_hash,
              reason_codes, selected_action, prior_status, resulting_status, payload)
             VALUES ('l1', 9, 'h', 'food', 1, 'h', '', 'propose', 'unresolved', 'covered', 'x')",
            // Fresh id, colliding seq: a silent eviction of row l1.
            "INSERT OR REPLACE INTO controller_ledger
             (id, seq, household_id, controller_id, algorithm_version, snapshot_hash,
              reason_codes, selected_action, prior_status, resulting_status, payload)
             VALUES ('l9', 1, 'h', 'food', 1, 'h', '', 'propose', 'unresolved', 'covered', 'x')",
        ] {
            let err = conn.execute(sql, []).unwrap_err();
            assert!(
                err.to_string().contains("controller_ledger is append-only"),
                "{sql}: {err}"
            );
        }
        let rows = list_ledger_entries(&conn, &hid("h")).unwrap();
        assert_eq!(rows.len(), 2, "no row rewritten and none evicted");
        assert_eq!(rows[0].1.payload, "p1");
        assert_eq!(rows[1].1.payload, "p2");
    }

    /// A corrupt `policy` row names its own table, not the ledger's; and a `parameters` blob
    /// the encoder could not have written is reported rather than silently decoded away.
    #[test]
    fn a_corrupt_policy_row_names_the_policy_table() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        conn.execute(
            "INSERT INTO policy (id, household_id, domain, policy_type, parameters, enabled, source)
             VALUES ('p1', 'h', 'food', 'food.hard_veto', 'subject\tpeanut', 1, 'bogus')",
            [],
        )
        .unwrap();
        let err = list_policies(&conn, &hid("h"), "food").unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::CorruptRow {
                    table: "policy",
                    column: "source",
                    ..
                }
            ),
            "{err:?}"
        );
        assert!(err.to_string().contains("policy row p1"), "{err}");
        conn.execute(
            "UPDATE policy SET source = 'explicit_user', parameters = 'subjectpeanut'
             WHERE id = 'p1'",
            [],
        )
        .unwrap();
        let err = list_policies(&conn, &hid("h"), "food").unwrap_err();
        assert!(
            matches!(
                &err,
                StorageError::CorruptRow {
                    table: "policy",
                    column: "parameters",
                    ..
                }
            ),
            "{err:?}"
        );
        // Expected-to-pass halves: both of these are well-formed and must stay lossless.
        for (blob, expected) in [("", vec![]), ("k\t", vec![("k", "")])] {
            conn.execute(
                "UPDATE policy SET parameters = ?1 WHERE id = 'p1'",
                params![blob],
            )
            .unwrap();
            let p = list_policies(&conn, &hid("h"), "food").unwrap();
            let got: Vec<(&str, &str)> = p[0]
                .parameters
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            assert_eq!(got, expected, "{blob:?}");
        }
    }

    #[test]
    fn apply_never_touches_a_locked_slot() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        recipe(&mut conn, "h", "r1", "Rice");
        recipe(&mut conn, "h", "r2", "Soup");
        save_planned_meal(
            &mut conn,
            &meal("lock", "h", "2026-08-29", vec![rc("r1")]),
            WriteSource::User,
        )
        .unwrap();
        set_planned_meal_lock(
            &mut conn,
            &hid("h"),
            &PlannedMealId::new("lock").unwrap(),
            true,
            WriteSource::User,
        )
        .unwrap();
        // Same components as the lock: skipped, never rewritten.
        let same = vec![proposed("2026-08-29", vec![rc("r1")])];
        assert_eq!(
            {
                let e = entry_now(&mut conn, "l1", "h", "apply", "p");
                apply_plan_and_record(&mut conn, &same, &ids(1), &e, today(), 0)
            }
            .unwrap(),
            0
        );
        let m = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        assert_eq!(m[0].id().as_str(), "lock");
        assert!(m[0].locked());
        // Different components: refused as `LockedPlannedMeal`.
        let differ = vec![proposed("2026-08-29", vec![rc("r2")])];
        let err = {
            let e = entry_now(&mut conn, "l2", "h", "apply", "p");
            apply_plan_and_record(&mut conn, &differ, &ids(1), &e, today(), 0)
        }
        .unwrap_err();
        assert!(matches!(err, StorageError::LockedPlannedMeal(_)), "{err:?}");
        let m = list_planned_meals(&conn, &hid("h"), today(), today()).unwrap();
        assert_eq!(m[0].components(), &[rc("r1")]);
    }

    #[test]
    fn apply_rolls_back_meals_and_ledger_on_a_locked_conflict() {
        let mut conn = open(":memory:").unwrap();
        seed(&mut conn, "h");
        recipe(&mut conn, "h", "r1", "Rice");
        recipe(&mut conn, "h", "r2", "Soup");
        save_planned_meal(
            &mut conn,
            &meal("lock", "h", "2026-08-30", vec![rc("r1")]),
            WriteSource::User,
        )
        .unwrap();
        set_planned_meal_lock(
            &mut conn,
            &hid("h"),
            &PlannedMealId::new("lock").unwrap(),
            true,
            WriteSource::User,
        )
        .unwrap();
        // Slot 1 would be written first; slot 2 hits the lock; nothing must remain.
        let plan = vec![
            proposed("2026-08-29", vec![rc("r2")]),
            proposed("2026-08-30", vec![rc("r2")]),
        ];
        let err = {
            let e = entry_now(&mut conn, "l1", "h", "apply", "p");
            apply_plan_and_record(&mut conn, &plan, &ids(2), &e, today(), 0)
        }
        .unwrap_err();
        assert!(matches!(err, StorageError::LockedPlannedMeal(_)), "{err:?}");
        let meals = list_planned_meals(
            &conn,
            &hid("h"),
            today(),
            parse_civil_date("2026-08-31").unwrap(),
        )
        .unwrap();
        assert_eq!(
            meals.len(),
            1,
            "the first slot's write was rolled back: {meals:?}"
        );
        assert_eq!(meals[0].id().as_str(), "lock");
        assert!(
            list_ledger_entries(&conn, &hid("h")).unwrap().is_empty(),
            "no ledger row either"
        );
        // Too few ids is also a whole rollback.
        let plan = vec![
            proposed("2026-08-29", vec![rc("r2")]),
            proposed("2026-08-31", vec![rc("r1")]),
        ];
        assert!({
            let e = entry_now(&mut conn, "l2", "h", "apply", "p");
            apply_plan_and_record(&mut conn, &plan, &ids(1), &e, today(), 0)
        }
        .is_err());
        assert_eq!(
            list_planned_meals(
                &conn,
                &hid("h"),
                today(),
                parse_civil_date("2026-08-31").unwrap()
            )
            .unwrap()
            .len(),
            1
        );
        assert!(list_ledger_entries(&conn, &hid("h")).unwrap().is_empty());
    }
}
