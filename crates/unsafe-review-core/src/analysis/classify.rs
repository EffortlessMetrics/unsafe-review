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
            summary: "owner reached".to_string(),
        }
    }

    fn unreached() -> ReachEvidence {
        ReachEvidence {
            state: "unreached".to_string(),
            summary: "owner unreached".to_string(),
        }
    }

    #[test]
    fn ffi_hazards_are_classified_as_miri_unsupported_before_evidence_gaps() {
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
    fn concurrency_hazards_require_loom_before_contract_checks() {
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
    fn missing_contract_takes_precedence_over_missing_guards() {
        let (class, priority, confidence) = classify(
            &[HazardKind::Alignment],
            &ContractEvidence::missing(),
            &DischargeEvidence::missing(),
            &unreached(),
        );

        assert_eq!(class, ReviewClass::ContractMissing);
        assert_eq!(priority, Priority::High);
        assert_eq!(confidence, Confidence::High);
    }

    #[test]
    fn present_contract_without_discharge_is_guard_missing() {
        let (class, priority, confidence) = classify(
            &[HazardKind::Alignment],
            &present_contract(),
            &DischargeEvidence::missing(),
            &reached(),
        );

        assert_eq!(class, ReviewClass::GuardMissing);
        assert_eq!(priority, Priority::High);
        assert_eq!(confidence, Confidence::Medium);
    }

    #[test]
    fn guarded_but_unreached_sites_are_called_out() {
        let (class, priority, confidence) = classify(
            &[HazardKind::Alignment],
            &present_contract(),
            &present_discharge(),
            &unreached(),
        );

        assert_eq!(class, ReviewClass::UnsafeUnreached);
        assert_eq!(priority, Priority::Medium);
        assert_eq!(confidence, Confidence::Medium);
    }

    #[test]
    fn guarded_reached_sites_remain_unwitnessed_until_receipts_exist() {
        let (class, priority, confidence) = classify(
            &[HazardKind::Alignment],
            &present_contract(),
            &present_discharge(),
            &reached(),
        );

        assert_eq!(class, ReviewClass::GuardedUnwitnessed);
        assert_eq!(priority, Priority::Medium);
        assert_eq!(confidence, Confidence::Medium);
    }
}
