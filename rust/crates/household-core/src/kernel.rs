//! Kernel primitives beyond identity (PRD v3 §7.2–§7.8), landed at their first call site
//! (principles §25): the food planner in `food-domain` and the ledger in `kimatta-storage`.
//! Nothing here names a food type. Dates are ISO text because the kernel has no date type
//! and must not grow one for a horizon it only routes.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::{HouseholdId, LedgerEntryId, PolicyId};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum KernelError {
    #[error("{field} must not be empty or whitespace")]
    Empty { field: &'static str },
    #[error("{0:?} is not a reason code: non-empty, no whitespace")]
    InvalidReasonCode(String),
    #[error("policy parameter {key:?}={value:?} contains a tab or newline")]
    InvalidPolicyParameter { key: String, value: String },
}

/// Where a fact came from (§7.2). Coarse on purpose: numerical probabilities wait for
/// calibration data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvidenceSource {
    ExplicitUser,
    VerifiedIntegration,
    Derived,
    InferredBehavior,
    ImportedUntrusted,
}

impl EvidenceSource {
    pub const ALL: [Self; 5] = [
        Self::ExplicitUser,
        Self::VerifiedIntegration,
        Self::Derived,
        Self::InferredBehavior,
        Self::ImportedUntrusted,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitUser => "explicit_user",
            Self::VerifiedIntegration => "verified_integration",
            Self::Derived => "derived",
            Self::InferredBehavior => "inferred_behavior",
            Self::ImportedUntrusted => "imported_untrusted",
        }
    }

    /// Exact match, never trimmed — the `id_newtype!` rule.
    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_str() == raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Confidence {
    Explicit,
    High,
    Low,
    Unknown,
}

impl Confidence {
    pub const ALL: [Self; 4] = [Self::Explicit, Self::High, Self::Low, Self::Unknown];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::High => "high",
            Self::Low => "low",
            Self::Unknown => "unknown",
        }
    }
}

/// Qualitative band (§11): benefit and effort are hand-built thresholds, never fake VOI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Band {
    Low,
    Medium,
    High,
}

impl Band {
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Urgency {
    Low,
    High,
}

impl Urgency {
    pub const ALL: [Self; 2] = [Self::Low, Self::High];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reversibility {
    Reversible,
    Irreversible,
}

impl Reversibility {
    pub const ALL: [Self; 2] = [Self::Reversible, Self::Irreversible];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reversible => "reversible",
            Self::Irreversible => "irreversible",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequiredAuthority {
    None,
    HouseholdMember,
}

impl RequiredAuthority {
    pub const ALL: [Self; 2] = [Self::None, Self::HouseholdMember];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::HouseholdMember => "household_member",
        }
    }
}

/// A controller's verdict on its horizon (§2.2 at cycle level). Per-slot states are the
/// controller's own vocabulary; the kernel only carries the whole-horizon answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutcomeStatus {
    Unresolved,
    TentativelyCovered,
    Covered,
    NeedsAttention,
}

impl OutcomeStatus {
    pub const ALL: [Self; 4] = [
        Self::Unresolved,
        Self::TentativelyCovered,
        Self::Covered,
        Self::NeedsAttention,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unresolved => "unresolved",
            Self::TentativelyCovered => "tentatively_covered",
            Self::Covered => "covered",
            Self::NeedsAttention => "needs_attention",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_str() == raw)
    }
}

/// A debugging/explanation token (§9.5). No whitespace, so a ledger can store a list of them
/// space-joined and split it back without an escape scheme.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReasonCode(String);

impl ReasonCode {
    pub fn new(raw: impl Into<String>) -> Result<Self, KernelError> {
        let raw = raw.into();
        if raw.is_empty() || raw.chars().any(char::is_whitespace) {
            return Err(KernelError::InvalidReasonCode(raw));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Inclusive ISO civil-date bounds as text: the kernel routes a horizon, it never does
/// calendar arithmetic on one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Horizon {
    pub from: String,
    pub to: String,
}

/// The policy envelope (§7.3). The kernel stores and routes; the food controller interprets
/// `food.*` types. Parameters are a flat string map — no universal policy DSL in v3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub id: PolicyId,
    pub household_id: HouseholdId,
    pub domain: String,
    pub policy_type: String,
    pub parameters: BTreeMap<String, String>,
    pub enabled: bool,
    pub source: EvidenceSource,
}

impl Policy {
    /// Keys and values may not contain a tab, a line feed or a carriage return: storage encodes
    /// the map as `key\tvalue` lines, and a value that could forge a line boundary would be read
    /// back as a different map. A carriage return forges one too — the decoder splits with
    /// `str::lines()`, which strips a trailing `\r` from every `\n`-terminated line, so a value
    /// ending in `\r` would silently lose it. Blank keys are refused for the same reason a blank
    /// id is.
    pub fn new(
        id: PolicyId,
        household_id: HouseholdId,
        domain: impl Into<String>,
        policy_type: impl Into<String>,
        parameters: BTreeMap<String, String>,
        enabled: bool,
        source: EvidenceSource,
    ) -> Result<Self, KernelError> {
        let domain = non_blank(domain.into(), "domain")?;
        let policy_type = non_blank(policy_type.into(), "policy_type")?;
        for (key, value) in &parameters {
            let forged = |s: &str| s.contains('\t') || s.contains('\n') || s.contains('\r');
            if key.trim().is_empty() || forged(key) || forged(value) {
                return Err(KernelError::InvalidPolicyParameter {
                    key: key.clone(),
                    value: value.clone(),
                });
            }
        }
        Ok(Self {
            id,
            household_id,
            domain,
            policy_type,
            parameters,
            enabled,
            source,
        })
    }
}

fn non_blank(value: String, field: &'static str) -> Result<String, KernelError> {
    if value.trim().is_empty() {
        Err(KernelError::Empty { field })
    } else {
        Ok(value)
    }
}

/// §7.4: whether the desired state is covered on the horizon, and why not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeAssessment {
    pub controller_id: String,
    pub horizon: Horizon,
    pub status: OutcomeStatus,
    pub unresolved_issues: Vec<String>,
    pub assumptions: Vec<ReasonCode>,
    pub reason_codes: Vec<ReasonCode>,
}

/// §7.5: a proposal is not authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionProposal {
    pub id: String,
    pub controller_id: String,
    pub action_type: String,
    pub expected_benefit_band: Band,
    pub confidence: Confidence,
    pub reversibility: Reversibility,
    pub required_authority: RequiredAuthority,
    pub deadline: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

/// §7.6: produced, never acted on. The controller does not decide to notify (principles §9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionRequest {
    pub id: String,
    pub controller_id: String,
    pub urgency: Urgency,
    pub deadline: Option<String>,
    pub decision_benefit_band: Band,
    pub estimated_effort_band: Band,
    pub options: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

/// §7.7: one appended planner decision. Storage refuses updates and deletes, so a later run
/// can only add a row (invariant 20). A human correction is its own appended row naming the
/// prior entry in `payload`, not a column here — no writer exists before `MVP-024`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub id: LedgerEntryId,
    pub household_id: HouseholdId,
    pub controller_id: String,
    pub algorithm_version: u32,
    pub snapshot_hash: String,
    pub reason_codes: Vec<ReasonCode>,
    pub selected_action: String,
    pub prior_status: OutcomeStatus,
    pub resulting_status: OutcomeStatus,
    pub payload: String,
}

/// §7.8, kept tiny. `Context` is the controller's own snapshot type, so the kernel needs no
/// generic `ControlContext`; food-specific commands stay outside the trait.
pub trait HouseholdController {
    type Context;
    type Error;

    fn assess(&self, ctx: &Self::Context) -> Result<OutcomeAssessment, Self::Error>;
    fn propose(&self, ctx: &Self::Context) -> Result<Vec<ActionProposal>, Self::Error>;
}
