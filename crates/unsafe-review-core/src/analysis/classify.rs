use crate::domain::{
    Confidence, ContractEvidence, DischargeEvidence, HazardKind, Priority, ReachEvidence,
    ReviewClass,
};

pub(crate) fn classify(
    hazards: &[HazardKind],
    contract: &ContractEvidence,
    discharge: &DischargeEvidence,
    reach: &ReachEvidence,
) -> (ReviewClass, Priority, Confidence) {
    if hazards
        .iter()
        .any(|h| matches!(h, HazardKind::FfiAbi | HazardKind::FfiOwnership))
    {
        return (
            ReviewClass::MiriUnsupported,
            Priority::Medium,
            Confidence::Medium,
        );
    }
    if hazards.iter().any(|h| {
        matches!(
            h,
            HazardKind::SendSyncInvariant
                | HazardKind::AtomicOrdering
                | HazardKind::StaticMutGlobalState
        )
    }) {
        return (
            ReviewClass::RequiresLoom,
            Priority::High,
            Confidence::Medium,
        );
    }
    if !contract.present {
        return (
            ReviewClass::ContractMissing,
            Priority::High,
            Confidence::High,
        );
    }
    if !discharge.present {
        return (
            ReviewClass::GuardMissing,
            Priority::High,
            Confidence::Medium,
        );
    }
    if reach.state == "unreached" {
        return (
            ReviewClass::UnsafeUnreached,
            Priority::Medium,
            Confidence::Medium,
        );
    }
    (
        ReviewClass::GuardedUnwitnessed,
        Priority::Medium,
        Confidence::Medium,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract(present: bool) -> ContractEvidence {
        if present {
            ContractEvidence::present("contract evidence")
        } else {
            ContractEvidence::missing()
        }
    }

    fn discharge(present: bool) -> DischargeEvidence {
        if present {
            DischargeEvidence::present("guard evidence")
        } else {
            DischargeEvidence::missing()
        }
    }

    fn reach(state: &str) -> ReachEvidence {
        ReachEvidence {
            state: state.to_string(),
            summary: "reach evidence".to_string(),
        }
    }

    #[test]
    fn ffi_hazards_are_routed_before_contract_gaps() {
        assert_eq!(
            classify(
                &[HazardKind::FfiAbi],
                &contract(false),
                &discharge(false),
                &reach("unreached"),
            ),
            (
                ReviewClass::MiriUnsupported,
                Priority::Medium,
                Confidence::Medium,
            )
        );
    }

    #[test]
    fn concurrency_hazards_require_loom_before_contract_gaps() {
        for hazard in [
            HazardKind::SendSyncInvariant,
            HazardKind::AtomicOrdering,
            HazardKind::StaticMutGlobalState,
        ] {
            assert_eq!(
                classify(
                    &[hazard],
                    &contract(false),
                    &discharge(false),
                    &reach("unreached"),
                ),
                (
                    ReviewClass::RequiresLoom,
                    Priority::High,
                    Confidence::Medium
                )
            );
        }
    }

    #[test]
    fn missing_contract_is_high_confidence_contract_gap() {
        assert_eq!(
            classify(
                &[HazardKind::PointerValidity],
                &contract(false),
                &discharge(true),
                &reach("owner_reached"),
            ),
            (
                ReviewClass::ContractMissing,
                Priority::High,
                Confidence::High
            )
        );
    }

    #[test]
    fn missing_guard_is_high_priority_guard_gap() {
        assert_eq!(
            classify(
                &[HazardKind::Bounds],
                &contract(true),
                &discharge(false),
                &reach("owner_reached"),
            ),
            (
                ReviewClass::GuardMissing,
                Priority::High,
                Confidence::Medium
            )
        );
    }

    #[test]
    fn reached_and_guarded_sites_remain_unwitnessed() {
        assert_eq!(
            classify(
                &[HazardKind::Alignment],
                &contract(true),
                &discharge(true),
                &reach("owner_reached"),
            ),
            (
                ReviewClass::GuardedUnwitnessed,
                Priority::Medium,
                Confidence::Medium,
            )
        );
    }

    #[test]
    fn unreached_sites_are_reported_after_contract_and_guard_evidence() {
        assert_eq!(
            classify(
                &[HazardKind::Alignment],
                &contract(true),
                &discharge(true),
                &reach("unreached"),
            ),
            (
                ReviewClass::UnsafeUnreached,
                Priority::Medium,
                Confidence::Medium
            )
        );
    }
}
