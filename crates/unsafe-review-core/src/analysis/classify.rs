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

    fn reached() -> ReachEvidence {
        ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "owner test found".to_string(),
        }
    }

    fn unreached() -> ReachEvidence {
        ReachEvidence {
            state: "unreached".to_string(),
            summary: "no test found".to_string(),
        }
    }

    #[test]
    fn prioritizes_specialized_hazards_before_static_evidence_gaps() {
        let (ffi_class, ffi_priority, ffi_confidence) = classify(
            &[HazardKind::FfiAbi],
            &ContractEvidence::missing(),
            &DischargeEvidence::missing(),
            &reached(),
        );
        assert_eq!(ffi_class, ReviewClass::MiriUnsupported);
        assert_eq!(ffi_priority, Priority::Medium);
        assert_eq!(ffi_confidence, Confidence::Medium);

        let (loom_class, loom_priority, loom_confidence) = classify(
            &[HazardKind::SendSyncInvariant],
            &ContractEvidence::missing(),
            &DischargeEvidence::missing(),
            &reached(),
        );
        assert_eq!(loom_class, ReviewClass::RequiresLoom);
        assert_eq!(loom_priority, Priority::High);
        assert_eq!(loom_confidence, Confidence::Medium);
    }

    #[test]
    fn classifies_contract_guard_and_reach_gaps_in_order() {
        let hazards = [HazardKind::PointerValidity];

        let (missing_contract, missing_contract_priority, missing_contract_confidence) = classify(
            &hazards,
            &ContractEvidence::missing(),
            &DischargeEvidence::missing(),
            &unreached(),
        );
        assert_eq!(missing_contract, ReviewClass::ContractMissing);
        assert_eq!(missing_contract_priority, Priority::High);
        assert_eq!(missing_contract_confidence, Confidence::High);

        let (missing_guard, missing_guard_priority, missing_guard_confidence) = classify(
            &hazards,
            &ContractEvidence::present("# Safety documents pointer validity"),
            &DischargeEvidence::missing(),
            &unreached(),
        );
        assert_eq!(missing_guard, ReviewClass::GuardMissing);
        assert_eq!(missing_guard_priority, Priority::High);
        assert_eq!(missing_guard_confidence, Confidence::Medium);

        let (unreached_class, unreached_priority, unreached_confidence) = classify(
            &hazards,
            &ContractEvidence::present("# Safety documents pointer validity"),
            &DischargeEvidence::present("guard checks bounds"),
            &unreached(),
        );
        assert_eq!(unreached_class, ReviewClass::UnsafeUnreached);
        assert_eq!(unreached_priority, Priority::Medium);
        assert_eq!(unreached_confidence, Confidence::Medium);
    }

    #[test]
    fn classifies_complete_static_evidence_as_guarded_unwitnessed() {
        let (class, priority, confidence) = classify(
            &[HazardKind::Bounds],
            &ContractEvidence::present("# Safety documents bounds"),
            &DischargeEvidence::present("guard checks len"),
            &reached(),
        );

        assert_eq!(class, ReviewClass::GuardedUnwitnessed);
        assert_eq!(priority, Priority::Medium);
        assert_eq!(confidence, Confidence::Medium);
    }
}
