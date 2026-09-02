//! Household identity kernel primitives (PRD v3 §7.1). Knows nothing about food.
#![forbid(unsafe_code)]

pub mod kernel;

pub use kernel::*;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdError {
    #[error("id must not be empty or whitespace")]
    Empty,
}

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// Accepts the id verbatim; rejects empty/whitespace, never trims.
            pub fn new(raw: impl Into<String>) -> Result<Self, IdError> {
                let raw = raw.into();
                if raw.trim().is_empty() {
                    Err(IdError::Empty)
                } else {
                    Ok(Self(raw))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

id_newtype!(HouseholdId);
id_newtype!(MemberId);
// Declared here rather than in `kernel.rs`: `macro_rules!` is textually scoped, and exporting
// the macro for two ids is more coupling than two extra lines beside the ids it already mints.
id_newtype!(PolicyId);
id_newtype!(LedgerEntryId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Household {
    pub id: HouseholdId,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HouseholdMember {
    pub id: MemberId,
    pub household_id: HouseholdId,
    pub display_name: String,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn rejects_empty_and_whitespace() {
        assert_eq!(HouseholdId::new("").unwrap_err(), IdError::Empty);
        assert_eq!(HouseholdId::new(" \t\n").unwrap_err(), IdError::Empty);
        assert_eq!(MemberId::new("").unwrap_err(), IdError::Empty);
    }

    #[test]
    fn round_trips_without_trimming() {
        assert_eq!(HouseholdId::new("h-1").unwrap().as_str(), "h-1");
        assert_eq!(HouseholdId::new(" h-1 ").unwrap().as_str(), " h-1 ");
        assert_eq!(MemberId::new(" m-1 ").unwrap().as_str(), " m-1 ");
    }

    // --- MVP-023 step 2: kernel primitives ------------------------------------------------

    #[test]
    fn kernel_ids_reject_empty_and_never_trim() {
        assert_eq!(PolicyId::new("").unwrap_err(), IdError::Empty);
        assert_eq!(LedgerEntryId::new(" \t").unwrap_err(), IdError::Empty);
        assert_eq!(PolicyId::new(" p-1 ").unwrap().as_str(), " p-1 ");
        assert_eq!(LedgerEntryId::new(" l-1 ").unwrap().as_str(), " l-1 ");
    }

    #[test]
    fn reason_code_rejects_blank_and_whitespace() {
        for raw in ["", "   ", "\t", "HAS SPACE", "TAB\tIN", "NL\nIN", " LEAD"] {
            assert_eq!(
                ReasonCode::new(raw).unwrap_err(),
                KernelError::InvalidReasonCode(raw.to_owned()),
                "{raw:?} must not be a reason code"
            );
        }
        assert_eq!(
            ReasonCode::new("REPEATED_SACRIFICE:m-1").unwrap().as_str(),
            "REPEATED_SACRIFICE:m-1"
        );
    }

    fn policy(parameters: BTreeMap<String, String>) -> Result<Policy, KernelError> {
        Policy::new(
            PolicyId::new("p").unwrap(),
            HouseholdId::new("h").unwrap(),
            "food",
            "food.dining_out",
            parameters,
            true,
            EvidenceSource::ExplicitUser,
        )
    }

    #[test]
    fn policy_rejects_tab_line_feed_or_carriage_return_in_parameters() {
        for (key, value) in [
            ("a\tb", "v"),
            ("a\nb", "v"),
            ("k", "v\tw"),
            ("k", "v\nw"),
            ("k\n", ""),
            // A trailing `\r` forges a line boundary just as `\n` does: the decoder splits
            // with `str::lines()`, which strips it from every `\n`-terminated line.
            ("k", "v\r"),
            ("k", "v\rw"),
            ("a\rb", "v"),
        ] {
            let parameters = BTreeMap::from([(key.to_owned(), value.to_owned())]);
            assert_eq!(
                policy(parameters).unwrap_err(),
                KernelError::InvalidPolicyParameter {
                    key: key.to_owned(),
                    value: value.to_owned(),
                },
                "{key:?}={value:?} must be refused"
            );
        }
        let ok = policy(BTreeMap::from([(
            "subject".to_owned(),
            "olives".to_owned(),
        )]))
        .unwrap();
        assert_eq!(
            ok.parameters.get("subject").map(String::as_str),
            Some("olives")
        );
        assert!(policy(BTreeMap::new()).is_ok(), "no parameters is legal");
        // The refusal set is exactly what `str::lines()` can act on. Vertical tab and form
        // feed round-trip losslessly, so widening past `{\t, \n, \r}` would reject values the
        // encoding handles fine.
        for value in ["v\u{0b}w", "v\u{0c}w", "v w"] {
            assert!(
                policy(BTreeMap::from([("k".to_owned(), value.to_owned())])).is_ok(),
                "{value:?} must still be accepted"
            );
        }
    }

    #[test]
    fn policy_rejects_blank_domain_and_type() {
        for (domain, policy_type) in [("", "food.x"), ("food", " "), (" ", "")] {
            let err = Policy::new(
                PolicyId::new("p").unwrap(),
                HouseholdId::new("h").unwrap(),
                domain,
                policy_type,
                BTreeMap::new(),
                true,
                EvidenceSource::Derived,
            )
            .unwrap_err();
            assert!(
                matches!(err, KernelError::Empty { .. }),
                "{domain:?}/{policy_type:?} gave {err:?}"
            );
        }
    }

    /// Compile-level documentation of the contract: a controller is `assess` and `propose`
    /// over a caller-chosen context, and nothing else. The context here is a unit type, so
    /// the kernel demonstrably needs no food type to host one.
    #[test]
    fn a_controller_impl_compiles_against_the_trait() {
        struct Unit;
        impl HouseholdController for Unit {
            type Context = u32;
            type Error = KernelError;

            fn assess(&self, ctx: &u32) -> Result<OutcomeAssessment, KernelError> {
                Ok(OutcomeAssessment {
                    controller_id: "unit".to_owned(),
                    horizon: Horizon {
                        from: "2026-08-29".to_owned(),
                        to: "2026-09-04".to_owned(),
                    },
                    status: if *ctx > 0 {
                        OutcomeStatus::Covered
                    } else {
                        OutcomeStatus::Unresolved
                    },
                    unresolved_issues: vec![],
                    assumptions: vec![],
                    reason_codes: vec![ReasonCode::new("UNIT")?],
                })
            }

            fn propose(&self, _ctx: &u32) -> Result<Vec<ActionProposal>, KernelError> {
                Ok(vec![ActionProposal {
                    id: "unit:apply".to_owned(),
                    controller_id: "unit".to_owned(),
                    action_type: "apply".to_owned(),
                    expected_benefit_band: Band::High,
                    confidence: Confidence::High,
                    reversibility: Reversibility::Reversible,
                    required_authority: RequiredAuthority::HouseholdMember,
                    deadline: None,
                    reason_codes: vec![],
                }])
            }
        }
        let c = Unit;
        assert_eq!(c.assess(&1).unwrap().status, OutcomeStatus::Covered);
        assert_eq!(c.assess(&0).unwrap().status, OutcomeStatus::Unresolved);
        assert_eq!(c.propose(&1).unwrap()[0].action_type, "apply");
    }

    /// Hard-coded token tables: an added variant must break this, not be mirrored by it.
    #[test]
    fn every_kernel_enum_has_a_stable_token() {
        assert_eq!(
            EvidenceSource::ALL.map(EvidenceSource::as_str),
            [
                "explicit_user",
                "verified_integration",
                "derived",
                "inferred_behavior",
                "imported_untrusted",
            ]
        );
        for s in EvidenceSource::ALL {
            assert_eq!(EvidenceSource::parse(s.as_str()), Some(s));
        }
        assert_eq!(EvidenceSource::parse("Derived"), None);
        assert_eq!(
            Confidence::ALL.map(Confidence::as_str),
            ["explicit", "high", "low", "unknown"]
        );
        assert_eq!(Band::ALL.map(Band::as_str), ["low", "medium", "high"]);
        assert_eq!(Urgency::ALL.map(Urgency::as_str), ["low", "high"]);
        assert_eq!(
            Reversibility::ALL.map(Reversibility::as_str),
            ["reversible", "irreversible"]
        );
        assert_eq!(
            RequiredAuthority::ALL.map(RequiredAuthority::as_str),
            ["none", "household_member"]
        );
        assert_eq!(
            OutcomeStatus::ALL.map(OutcomeStatus::as_str),
            [
                "unresolved",
                "tentatively_covered",
                "covered",
                "needs_attention",
            ]
        );
        for s in OutcomeStatus::ALL {
            assert_eq!(OutcomeStatus::parse(s.as_str()), Some(s));
        }
        assert_eq!(OutcomeStatus::parse(""), None);
    }
}
