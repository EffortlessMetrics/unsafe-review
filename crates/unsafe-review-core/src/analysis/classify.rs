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
        ContractEvidence {
            present,
            summary: "contract".to_string(),
        }
    }

    fn discharge(present: bool) -> DischargeEvidence {
        DischargeEvidence {
            present,
            summary: "discharge".to_string(),
        }
    }

    fn reach(state: &str) -> ReachEvidence {
        ReachEvidence {
            state: state.to_string(),
            summary: "reach".to_string(),
        }
    }

    #[test]
    fn ffi_hazards_are_routed_before_generic_contract_gaps() {
        let (class, priority, confidence) = classify(
            &[HazardKind::FfiAbi],
            &contract(false),
            &discharge(false),
            &reach("unreached"),
        );

        assert_eq!(class, ReviewClass::MiriUnsupported);
        assert_eq!(priority, Priority::Medium);
        assert_eq!(confidence, Confidence::Medium);
    }

    #[test]
    fn concurrency_hazards_require_loom_before_generic_guard_gaps() {
        let (class, priority, confidence) = classify(
            &[HazardKind::SendSyncInvariant],
            &contract(true),
            &discharge(false),
            &reach("owner_reached"),
        );

        assert_eq!(class, ReviewClass::RequiresLoom);
        assert_eq!(priority, Priority::High);
        assert_eq!(confidence, Confidence::Medium);
    }

    #[test]
    fn concurrency_hazards_are_routed_before_contract_gaps() {
        for hazard in [
            HazardKind::SendSyncInvariant,
            HazardKind::AtomicOrdering,
            HazardKind::StaticMutGlobalState,
        ] {
            let (class, priority, confidence) = classify(
                &[hazard],
                &contract(false),
                &discharge(false),
                &reach("unreached"),
            );

            assert_eq!(class, ReviewClass::RequiresLoom);
            assert_eq!(priority, Priority::High);
            assert_eq!(confidence, Confidence::Medium);
        }
    }

    #[test]
    fn ordinary_unsafe_sites_progress_through_evidence_states() {
        let hazards = [HazardKind::PointerValidity];

        assert_eq!(
            classify(
                &hazards,
                &contract(false),
                &discharge(false),
                &reach("unreached")
            ),
            (
                ReviewClass::ContractMissing,
                Priority::High,
                Confidence::High
            )
        );
        assert_eq!(
            classify(
                &hazards,
                &contract(true),
                &discharge(false),
                &reach("owner_reached")
            ),
            (
                ReviewClass::GuardMissing,
                Priority::High,
                Confidence::Medium
            )
        );
        assert_eq!(
            classify(
                &hazards,
                &contract(true),
                &discharge(true),
                &reach("unreached")
            ),
            (
                ReviewClass::UnsafeUnreached,
                Priority::Medium,
                Confidence::Medium
            )
        );
        assert_eq!(
            classify(
                &hazards,
                &contract(true),
                &discharge(true),
                &reach("owner_reached")
            ),
            (
                ReviewClass::GuardedUnwitnessed,
                Priority::Medium,
                Confidence::Medium
            )
        );
    }
}
