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
        ContractEvidence::present("documented safety preconditions")
    }

    fn present_discharge() -> DischargeEvidence {
        DischargeEvidence::present("local guard checks the preconditions")
    }

    fn reached() -> ReachEvidence {
        ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "owner is exercised by a related test".to_string(),
        }
    }

    fn unreached() -> ReachEvidence {
        ReachEvidence {
            state: "unreached".to_string(),
            summary: "no related test path was found".to_string(),
        }
    }

    #[test]
    fn ffi_hazards_are_miri_unsupported_before_contract_gaps() {
        let (class, priority, confidence) = classify(
            &[HazardKind::FfiAbi],
            &ContractEvidence::missing(),
            &DischargeEvidence::missing(),
            &unreached(),
        );

        assert_eq!(class, ReviewClass::MiriUnsupported);
        assert_eq!(priority, Priority::Medium);
        assert_eq!(confidence, Confidence::Medium);
    }

    #[test]
    fn concurrency_hazards_require_loom_before_contract_gaps() {
        let (class, priority, confidence) = classify(
            &[HazardKind::SendSyncInvariant],
            &ContractEvidence::missing(),
            &DischargeEvidence::missing(),
            &unreached(),
        );

        assert_eq!(class, ReviewClass::RequiresLoom);
        assert_eq!(priority, Priority::High);
        assert_eq!(confidence, Confidence::Medium);
    }

    #[test]
    fn pure_rust_hazards_progress_through_evidence_gaps() {
        let hazards = [HazardKind::PointerValidity];

        let missing_contract = classify(
            &hazards,
            &ContractEvidence::missing(),
            &present_discharge(),
            &reached(),
        );
        assert_eq!(
            missing_contract,
            (
                ReviewClass::ContractMissing,
                Priority::High,
                Confidence::High
            )
        );

        let missing_guard = classify(
            &hazards,
            &present_contract(),
            &DischargeEvidence::missing(),
            &reached(),
        );
        assert_eq!(
            missing_guard,
            (
                ReviewClass::GuardMissing,
                Priority::High,
                Confidence::Medium
            )
        );

        let missing_reach = classify(
            &hazards,
            &present_contract(),
            &present_discharge(),
            &unreached(),
        );
        assert_eq!(
            missing_reach,
            (
                ReviewClass::UnsafeUnreached,
                Priority::Medium,
                Confidence::Medium
            )
        );

        let unwitnessed = classify(
            &hazards,
            &present_contract(),
            &present_discharge(),
            &reached(),
        );
        assert_eq!(
            unwitnessed,
            (
                ReviewClass::GuardedUnwitnessed,
                Priority::Medium,
                Confidence::Medium
            )
        );
    }
}
