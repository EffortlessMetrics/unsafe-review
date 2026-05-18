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

    fn present_contract() -> ContractEvidence {
        ContractEvidence::present("contract present")
    }

    fn present_discharge() -> DischargeEvidence {
        DischargeEvidence::present("guards present")
    }

    fn reached() -> ReachEvidence {
        ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "test mentions owner".to_string(),
        }
    }

    #[test]
    fn prioritizes_special_routing_classes_before_general_gaps() {
        let missing_contract = ContractEvidence::missing();
        let missing_discharge = DischargeEvidence::missing();
        let unreached = ReachEvidence {
            state: "unreached".to_string(),
            summary: "no tests".to_string(),
        };

        assert_eq!(
            classify(
                &[HazardKind::FfiAbi],
                &missing_contract,
                &missing_discharge,
                &unreached
            ),
            (
                ReviewClass::MiriUnsupported,
                Priority::Medium,
                Confidence::Medium
            )
        );
        assert_eq!(
            classify(
                &[HazardKind::SendSyncInvariant],
                &missing_contract,
                &missing_discharge,
                &unreached
            ),
            (
                ReviewClass::RequiresLoom,
                Priority::High,
                Confidence::Medium
            )
        );
    }

    #[test]
    fn classifies_contract_guard_reach_and_witness_gaps_in_order() {
        let missing_contract = ContractEvidence::missing();
        let missing_discharge = DischargeEvidence::missing();
        let unreached = ReachEvidence {
            state: "unreached".to_string(),
            summary: "no tests".to_string(),
        };

        assert_eq!(
            classify(
                &[HazardKind::PointerValidity],
                &missing_contract,
                &present_discharge(),
                &reached()
            ),
            (
                ReviewClass::ContractMissing,
                Priority::High,
                Confidence::High
            )
        );
        assert_eq!(
            classify(
                &[HazardKind::PointerValidity],
                &present_contract(),
                &missing_discharge,
                &reached()
            ),
            (
                ReviewClass::GuardMissing,
                Priority::High,
                Confidence::Medium
            )
        );
        assert_eq!(
            classify(
                &[HazardKind::PointerValidity],
                &present_contract(),
                &present_discharge(),
                &unreached
            ),
            (
                ReviewClass::UnsafeUnreached,
                Priority::Medium,
                Confidence::Medium
            )
        );
        assert_eq!(
            classify(
                &[HazardKind::PointerValidity],
                &present_contract(),
                &present_discharge(),
                &reached()
            ),
            (
                ReviewClass::GuardedUnwitnessed,
                Priority::Medium,
                Confidence::Medium
            )
        );
    }
}
