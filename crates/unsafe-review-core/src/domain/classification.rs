#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewClass {
    GuardedAndWitnessed,
    GuardedUnwitnessed,
    ContractMissing,
    GuardMissing,
    ReachableUnwitnessed,
    UnsafeUnreached,
    WitnessMismatch,
    RequiresLoom,
    RequiresSanitizer,
    RequiresKaniOrCrux,
    MiriUnsupported,
    StaticUnknown,
    BaselineKnown,
    Suppressed,
}

impl ReviewClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GuardedAndWitnessed => "guarded_and_witnessed",
            Self::GuardedUnwitnessed => "guarded_unwitnessed",
            Self::ContractMissing => "contract_missing",
            Self::GuardMissing => "guard_missing",
            Self::ReachableUnwitnessed => "reachable_unwitnessed",
            Self::UnsafeUnreached => "unsafe_unreached",
            Self::WitnessMismatch => "witness_mismatch",
            Self::RequiresLoom => "requires_loom",
            Self::RequiresSanitizer => "requires_sanitizer",
            Self::RequiresKaniOrCrux => "requires_kani_or_crux",
            Self::MiriUnsupported => "miri_unsupported",
            Self::StaticUnknown => "static_unknown",
            Self::BaselineKnown => "baseline_known",
            Self::Suppressed => "suppressed",
        }
    }

    pub fn is_actionable(&self) -> bool {
        matches!(
            self,
            Self::GuardedUnwitnessed
                | Self::ContractMissing
                | Self::GuardMissing
                | Self::ReachableUnwitnessed
                | Self::UnsafeUnreached
                | Self::RequiresLoom
                | Self::RequiresSanitizer
                | Self::RequiresKaniOrCrux
                | Self::MiriUnsupported
                | Self::StaticUnknown
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
    Unknown,
}

impl Confidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Unknown => "unknown",
        }
    }
}
