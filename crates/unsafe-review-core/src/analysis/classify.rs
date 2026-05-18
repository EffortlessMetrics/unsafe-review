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
        ContractEvidence::present("documented")
    }

    fn present_discharge() -> DischargeEvidence {
        DischargeEvidence::present("guarded")
    }

    fn reached() -> ReachEvidence {
        ReachEvidence {
            state: "static-mention".to_string(),
            summary: "related test mentions owner".to_string(),
        }
    }

    #[test]
    fn ffi_hazards_are_reported_before_evidence_gaps() {
        let (class, priority, confidence) = classify(
            &[HazardKind::FfiAbi],
            &ContractEvidence::missing(),
            &DischargeEvidence::missing(),
            &ReachEvidence {
                state: "unreached".to_string(),
                summary: "no test".to_string(),
            },
        );

        assert_eq!(class, ReviewClass::MiriUnsupported);
        assert_eq!(priority, Priority::Medium);
        assert_eq!(confidence, Confidence::Medium);
    }

    #[test]
    fn loom_hazards_are_reported_before_contract_gaps() {
        let (class, priority, confidence) = classify(
            &[HazardKind::SendSyncInvariant],
            &ContractEvidence::missing(),
            &DischargeEvidence::missing(),
            &reached(),
        );

        assert_eq!(class, ReviewClass::RequiresLoom);
        assert_eq!(priority, Priority::High);
        assert_eq!(confidence, Confidence::Medium);
    }

    #[test]
    fn ordinary_hazards_follow_contract_guard_reach_order() {
        let hazard = [HazardKind::PointerValidity];

        let (missing_contract, _, missing_contract_confidence) = classify(
            &hazard,
            &ContractEvidence::missing(),
            &present_discharge(),
            &reached(),
        );
        assert_eq!(missing_contract, ReviewClass::ContractMissing);
        assert_eq!(missing_contract_confidence, Confidence::High);

        let (missing_guard, missing_guard_priority, _) = classify(
            &hazard,
            &present_contract(),
            &DischargeEvidence::missing(),
            &reached(),
        );
        assert_eq!(missing_guard, ReviewClass::GuardMissing);
        assert_eq!(missing_guard_priority, Priority::High);

        let (unreached, unreached_priority, _) = classify(
            &hazard,
            &present_contract(),
            &present_discharge(),
            &ReachEvidence {
                state: "unreached".to_string(),
                summary: "no test".to_string(),
            },
        );
        assert_eq!(unreached, ReviewClass::UnsafeUnreached);
        assert_eq!(unreached_priority, Priority::Medium);

        let (guarded, guarded_priority, guarded_confidence) = classify(
            &hazard,
            &present_contract(),
            &present_discharge(),
            &reached(),
        );
        assert_eq!(guarded, ReviewClass::GuardedUnwitnessed);
        assert_eq!(guarded_priority, Priority::Medium);
        assert_eq!(guarded_confidence, Confidence::Medium);
    }
}
