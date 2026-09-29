//! Meal exclusions (OPT-007 §8): "Never suggest this meal" as a confirmed, reversible policy on
//! a dish's identity, beside the legacy phrase rules it does not reinterpret. A policy change
//! commits at once and independently of any draft's Undo; it never removes a saved meal, alters
//! a restriction, erases a commitment or regenerates anything. The draft the household acted
//! from keeps its choices and is re-assessed against the new rule; every other draft meets the
//! change as a needs-review on its next command, because the rule is part of its review identity.

use std::collections::BTreeMap;

use food_domain::planner::snapshot::{is_dish_identity, text_digest};
use food_domain::planner::FoodPolicies;
use household_core::{
    EvidenceSource, HouseholdId, LedgerEntry, LedgerEntryId, OutcomeStatus, Policy, PolicyId,
};
use kimatta_storage::rusqlite::{Transaction, TransactionBehavior};
use kimatta_storage::{
    append_ledger_entry_in, list_policies, load_draft_in, load_planning_snapshot_in, load_recipe,
    save_policy_in, shipped_starter_content, Connection, RecipeId, StorageError,
};

use crate::planning_drafts::{
    acting_draft_fresh, refresh_after_policy, DraftContext, DraftEnvelope, DraftView,
};
use crate::ApplicationError;

pub const ACTION_POLICY_ADD: &str = "policy_add";
pub const ACTION_POLICY_REMOVE: &str = "policy_remove";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExclusionKind {
    /// Identity rule: exactly this dish, whatever it is called or contains.
    Dish {
        identity: String,
        /// The title when the rule was made; the dish may since be archived or renamed.
        title: Option<String>,
        /// Still in the household's recipes (or the shipped starter list).
        available: bool,
    },
    /// Legacy phrase rule: whole-word match against every title *and* ingredient line.
    Phrase { subject: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExclusionView {
    pub policy_id: String,
    pub kind: ExclusionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExclusionOutcome {
    pub exclusions: Vec<ExclusionView>,
    /// The acting draft, re-assessed with its choices kept; `None` when no draft acted or it
    /// had moved on (its next command will ask for review).
    pub draft: Option<DraftView>,
}

fn dish_available(
    conn: &Connection,
    household: &HouseholdId,
    identity: &str,
) -> Result<bool, ApplicationError> {
    if let Some(id) = identity.strip_prefix("recipe:") {
        let record = load_recipe(conn, household, &RecipeId::new(id)?)?;
        return Ok(record.is_some_and(|r| r.archived_at.is_none()));
    }
    let slug = identity.strip_prefix("starter:").unwrap_or_default();
    let shipped =
        shipped_starter_content().map_err(|e| StorageError::CorruptProvenance(e.to_string()))?;
    Ok(shipped.recipes.iter().any(|s| s.slug == slug))
}

fn dish_title(
    conn: &Connection,
    household: &HouseholdId,
    identity: &str,
) -> Result<Option<String>, ApplicationError> {
    if let Some(id) = identity.strip_prefix("recipe:") {
        return Ok(
            load_recipe(conn, household, &RecipeId::new(id)?)?.map(|r| r.recipe.title().to_owned())
        );
    }
    let slug = identity.strip_prefix("starter:").unwrap_or_default();
    let shipped =
        shipped_starter_content().map_err(|e| StorageError::CorruptProvenance(e.to_string()))?;
    Ok(shipped
        .recipes
        .iter()
        .find(|s| s.slug == slug)
        .map(|s| s.title.clone()))
}

/// The identity Tier 0 compares: an installed starter is its `starter:<slug>`, however the card
/// named it (`food_domain::planner::draft::recipe_identity` applies the same rule).
fn canonical_dish(
    conn: &Connection,
    household: &HouseholdId,
    dish: &str,
) -> Result<String, ApplicationError> {
    if let Some(id) = dish.strip_prefix("recipe:") {
        if let Some(record) = load_recipe(conn, household, &RecipeId::new(id)?)? {
            if let Some(slug) = record.recipe.provenance().starter_slug() {
                return Ok(format!("starter:{slug}"));
            }
        }
    }
    Ok(dish.to_owned())
}

/// Every enabled exclusion, identity rules first, each group in a stable order.
pub fn list_exclusions(
    conn: &Connection,
    household: &HouseholdId,
) -> Result<Vec<ExclusionView>, ApplicationError> {
    let mut dishes = Vec::new();
    let mut phrases = Vec::new();
    for p in list_policies(conn, household, "food")? {
        if !p.enabled {
            continue;
        }
        match p.policy_type.as_str() {
            FoodPolicies::RECIPE_VETO => {
                let Some(identity) = p.parameters.get("dish").filter(|d| is_dish_identity(d))
                else {
                    continue;
                };
                dishes.push(ExclusionView {
                    policy_id: p.id.as_str().to_owned(),
                    kind: ExclusionKind::Dish {
                        identity: identity.clone(),
                        title: p.parameters.get("title").cloned(),
                        available: dish_available(conn, household, identity)?,
                    },
                });
            }
            FoodPolicies::HARD_VETO => {
                if let Some(subject) = p.parameters.get("subject") {
                    phrases.push(ExclusionView {
                        policy_id: p.id.as_str().to_owned(),
                        kind: ExclusionKind::Phrase {
                            subject: subject.clone(),
                        },
                    });
                }
            }
            _ => {}
        }
    }
    let key = |v: &ExclusionView| match &v.kind {
        ExclusionKind::Dish {
            title, identity, ..
        } => (
            title.clone().unwrap_or_default().to_lowercase(),
            identity.clone(),
        ),
        ExclusionKind::Phrase { subject } => (subject.to_lowercase(), v.policy_id.clone()),
    };
    dishes.sort_by_key(key);
    phrases.sort_by_key(key);
    dishes.extend(phrases);
    Ok(dishes)
}

fn begin(conn: &mut Connection) -> Result<Transaction<'_>, ApplicationError> {
    Ok(conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StorageError::from)?)
}

/// The ledger row for a policy change: the window's status on both sides when the household
/// has a cycle, since the rule can change what the saved plan's checks say.
fn record_policy(
    tx: &Transaction<'_>,
    ctx: &DraftContext,
    action: &str,
    payload: String,
    before: Option<OutcomeStatus>,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<(), ApplicationError> {
    let snapshot =
        match load_planning_snapshot_in(tx, &ctx.household_id, ctx.today, ctx.offset_cycles) {
            Ok(s) => Some(s),
            Err(StorageError::NoPlanningCycle(_)) => None,
            Err(e) => return Err(e.into()),
        };
    let after = snapshot.as_ref().map(food_domain::planner::assess_slots);
    let policies_text = list_policies(tx, &ctx.household_id, "food")?
        .iter()
        .map(|p| {
            format!(
                "{}:{}:{:?}:{}",
                p.id.as_str(),
                p.policy_type,
                p.parameters,
                p.enabled
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    append_ledger_entry_in(
        tx,
        &LedgerEntry {
            id: LedgerEntryId::new(mint_id())?,
            household_id: ctx.household_id.clone(),
            controller_id: food_domain::planner::CONTROLLER_ID.to_owned(),
            algorithm_version: food_domain::planner::PLANNER_ALGORITHM_VERSION,
            snapshot_hash: snapshot
                .as_ref()
                .map(|s| s.snapshot_hash())
                .unwrap_or_else(|| text_digest(&policies_text)),
            reason_codes: after
                .as_ref()
                .map(|(_, a)| a.reason_codes.clone())
                .unwrap_or_default(),
            selected_action: action.to_owned(),
            prior_status: before.unwrap_or(OutcomeStatus::Unresolved),
            resulting_status: after
                .map(|(_, a)| a.status)
                .unwrap_or(OutcomeStatus::Unresolved),
            payload,
        },
    )?;
    Ok(())
}

fn status_before(
    tx: &Transaction<'_>,
    ctx: &DraftContext,
) -> Result<Option<OutcomeStatus>, ApplicationError> {
    match load_planning_snapshot_in(tx, &ctx.household_id, ctx.today, ctx.offset_cycles) {
        Ok(s) => Ok(Some(food_domain::planner::assess_slots(&s).1.status)),
        Err(StorageError::NoPlanningCycle(_)) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Checked before the rule is written: see [`acting_draft_fresh`]. With draft edits switched off
/// (§12) the rule is still written but never carries the draft along, which would be an edit.
fn fresh_acting<'e>(
    tx: &Transaction<'_>,
    ctx: &DraftContext,
    acting: Option<&'e DraftEnvelope>,
) -> Result<Option<&'e DraftEnvelope>, ApplicationError> {
    match acting {
        Some(env) if env.context.household_id != ctx.household_id => {
            Err(ApplicationError::DraftNotFound(env.draft_id.clone()))
        }
        Some(env) if ctx.edits_enabled && acting_draft_fresh(tx, env)? => Ok(Some(env)),
        _ => Ok(None),
    }
}

fn finish(
    tx: Transaction<'_>,
    ctx: &DraftContext,
    fresh: Option<&DraftEnvelope>,
) -> Result<ExclusionOutcome, ApplicationError> {
    let draft = match fresh {
        Some(env) => {
            let row = load_draft_in(&tx, &ctx.household_id, &env.draft_id)?
                .ok_or_else(|| ApplicationError::DraftNotFound(env.draft_id.clone()))?;
            Some(refresh_after_policy(&tx, env, row)?)
        }
        None => None,
    };
    let exclusions = list_exclusions(&tx, &ctx.household_id)?;
    tx.commit().map_err(StorageError::from)?;
    Ok(ExclusionOutcome { exclusions, draft })
}

/// "Never suggest this meal" for each dish identity, after the household confirmed the list.
/// A dish must be one of the household's current recipes or a shipped starter; a fallback card
/// has no dish and so no identity to name. An identical enabled rule makes this a no-op.
pub fn add_dish_exclusions(
    conn: &mut Connection,
    ctx: &DraftContext,
    dishes: &[String],
    acting: Option<&DraftEnvelope>,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<ExclusionOutcome, ApplicationError> {
    let tx = begin(conn)?;
    let before = status_before(&tx, ctx)?;
    let fresh = fresh_acting(&tx, ctx, acting)?;
    let existing: Vec<String> = list_policies(&tx, &ctx.household_id, "food")?
        .into_iter()
        .filter(|p| p.enabled && p.policy_type == FoodPolicies::RECIPE_VETO)
        .filter_map(|p| p.parameters.get("dish").cloned())
        .collect();
    let mut added = Vec::new();
    for asked in dishes {
        if !is_dish_identity(asked) || !dish_available(&tx, &ctx.household_id, asked)? {
            return Err(ApplicationError::InvalidChoice(asked.clone()));
        }
        let dish = &canonical_dish(&tx, &ctx.household_id, asked)?;
        if existing.contains(dish) || added.contains(dish) {
            continue;
        }
        let mut parameters = BTreeMap::from([("dish".to_owned(), dish.clone())]);
        if let Some(title) = dish_title(&tx, &ctx.household_id, asked)? {
            parameters.insert("title".to_owned(), title);
        }
        let policy = Policy::new(
            PolicyId::new(mint_id())?,
            ctx.household_id.clone(),
            "food",
            FoodPolicies::RECIPE_VETO,
            parameters,
            true,
            EvidenceSource::ExplicitUser,
        )
        .map_err(StorageError::from)?;
        save_policy_in(&tx, &policy)?;
        added.push(dish.clone());
    }
    if !added.is_empty() {
        record_policy(
            &tx,
            ctx,
            ACTION_POLICY_ADD,
            format!(
                "policy={}\ndishes={}\n",
                FoodPolicies::RECIPE_VETO,
                added.join(" ")
            ),
            before,
            mint_id,
        )?;
    }
    finish(tx, ctx, fresh)
}

/// Disables one exclusion — identity or legacy phrase — keeping its row as history. Removing a
/// rule makes the dish eligible again; it regenerates nothing.
pub fn remove_exclusion(
    conn: &mut Connection,
    ctx: &DraftContext,
    policy_id: &str,
    acting: Option<&DraftEnvelope>,
    mint_id: &mut dyn FnMut() -> String,
) -> Result<ExclusionOutcome, ApplicationError> {
    let tx = begin(conn)?;
    let before = status_before(&tx, ctx)?;
    let fresh = fresh_acting(&tx, ctx, acting)?;
    let policy = list_policies(&tx, &ctx.household_id, "food")?
        .into_iter()
        .find(|p| {
            p.id.as_str() == policy_id
                && p.enabled
                && matches!(
                    p.policy_type.as_str(),
                    FoodPolicies::RECIPE_VETO | FoodPolicies::HARD_VETO
                )
        })
        .ok_or_else(|| ApplicationError::ExclusionNotFound(policy_id.to_owned()))?;
    let disabled = Policy::new(
        policy.id.clone(),
        policy.household_id.clone(),
        policy.domain.clone(),
        policy.policy_type.clone(),
        policy.parameters.clone(),
        false,
        EvidenceSource::ExplicitUser,
    )
    .map_err(StorageError::from)?;
    save_policy_in(&tx, &disabled)?;
    record_policy(
        &tx,
        ctx,
        ACTION_POLICY_REMOVE,
        format!(
            "policy={}\nid={}\nparameters={:?}\n",
            policy.policy_type,
            policy.id.as_str(),
            policy.parameters
        ),
        before,
        mint_id,
    )?;
    finish(tx, ctx, fresh)
}
