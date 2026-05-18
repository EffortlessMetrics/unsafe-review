use crate::domain::{HazardKind, OperationFamily, SafetyObligation};

pub(crate) fn hazards_for(family: &OperationFamily) -> Vec<HazardKind> {
    match family {
        OperationFamily::RawPointerDeref
        | OperationFamily::RawPointerRead
        | OperationFamily::RawPointerWrite => vec![
            HazardKind::PointerValidity,
            HazardKind::Alignment,
            HazardKind::InitializedMemory,
            HazardKind::SameAllocation,
        ],
        OperationFamily::RawPointerReadUnaligned => vec![
            HazardKind::PointerValidity,
            HazardKind::InitializedMemory,
            HazardKind::SameAllocation,
        ],
        OperationFamily::PointerArithmetic => vec![
            HazardKind::SameAllocation,
            HazardKind::Bounds,
            HazardKind::AliasingOrProvenance,
        ],
        OperationFamily::CopyNonOverlapping => vec![
            HazardKind::PointerValidity,
            HazardKind::Bounds,
            HazardKind::AliasingOrProvenance,
        ],
        OperationFamily::SliceFromRawParts => vec![
            HazardKind::PointerValidity,
            HazardKind::Alignment,
            HazardKind::InitializedMemory,
            HazardKind::Bounds,
            HazardKind::SameAllocation,
        ],
        OperationFamily::VecFromRawParts => vec![
            HazardKind::PointerValidity,
            HazardKind::Alignment,
            HazardKind::InitializedMemory,
            HazardKind::Bounds,
            HazardKind::DropOrDeallocation,
            HazardKind::LeakOrOwnershipTransfer,
        ],
        OperationFamily::StrFromUtf8Unchecked => vec![HazardKind::InvalidValue],
        OperationFamily::MaybeUninitAssumeInit => {
            vec![HazardKind::InitializedMemory, HazardKind::InvalidValue]
        }
        OperationFamily::VecSetLen => vec![HazardKind::InitializedMemory, HazardKind::Bounds],
        OperationFamily::Transmute | OperationFamily::Zeroed => vec![
            HazardKind::InvalidValue,
            HazardKind::LayoutOrRepr,
            HazardKind::AliasingOrProvenance,
        ],
        OperationFamily::DropInPlace => vec![
            HazardKind::PointerValidity,
            HazardKind::InitializedMemory,
            HazardKind::DropOrDeallocation,
        ],
        OperationFamily::AtomicPointerState => {
            vec![HazardKind::AtomicOrdering, HazardKind::DropOrDeallocation]
        }
        OperationFamily::UnwrapUnchecked | OperationFamily::UnreachableUnchecked => {
            vec![HazardKind::InvalidValue]
        }
        OperationFamily::UnsafeFnCall => vec![HazardKind::Unknown],
        OperationFamily::BoxFromRaw => vec![
            HazardKind::PointerValidity,
            HazardKind::DropOrDeallocation,
            HazardKind::LeakOrOwnershipTransfer,
        ],
        OperationFamily::NonNullUnchecked => vec![HazardKind::PointerValidity],
        OperationFamily::PinUnchecked => vec![HazardKind::PinInvariant],
        OperationFamily::GetUnchecked => vec![HazardKind::Bounds],
        OperationFamily::UnsafeImplSendSync => {
            vec![HazardKind::SendSyncInvariant, HazardKind::AtomicOrdering]
        }
        OperationFamily::Ffi => vec![HazardKind::FfiAbi, HazardKind::FfiOwnership],
        OperationFamily::StaticMut => vec![HazardKind::StaticMutGlobalState],
        OperationFamily::InlineAsm => vec![HazardKind::InlineAsm, HazardKind::TargetFeature],
        OperationFamily::TargetFeature => vec![HazardKind::TargetFeature],
        OperationFamily::Unknown => vec![HazardKind::Unknown],
    }
}

pub(crate) fn obligations_for(family: &OperationFamily) -> Vec<SafetyObligation> {
    match family {
        OperationFamily::RawPointerDeref
        | OperationFamily::RawPointerRead
        | OperationFamily::RawPointerWrite => vec![
            SafetyObligation::new(
                "pointer-live",
                "pointer is live and dereferenceable for the accessed type",
            ),
            SafetyObligation::new("bounds", "buffer has enough bytes for the accessed type"),
            SafetyObligation::new("alignment", "pointer is aligned for the accessed type"),
            SafetyObligation::new("initialized", "memory is initialized for the accessed type"),
            SafetyObligation::new("allocation", "access remains inside one live allocation"),
        ],
        OperationFamily::RawPointerReadUnaligned => vec![
            SafetyObligation::new(
                "pointer-live",
                "pointer is live and dereferenceable for the accessed type",
            ),
            SafetyObligation::new("bounds", "buffer has enough bytes for the accessed type"),
            SafetyObligation::new("initialized", "memory is initialized for the accessed type"),
            SafetyObligation::new("allocation", "access remains inside one live allocation"),
        ],
        OperationFamily::SliceFromRawParts => vec![
            SafetyObligation::new("pointer-live", "pointer is valid for `len` elements"),
            SafetyObligation::new("alignment", "pointer is aligned for the element type"),
            SafetyObligation::new("initialized", "memory range is initialized"),
            SafetyObligation::new("allocation", "range fits in one allocation"),
        ],
        OperationFamily::VecFromRawParts => vec![
            SafetyObligation::new(
                "pointer-live",
                "pointer was allocated by a compatible allocator for `capacity` elements",
            ),
            SafetyObligation::new("alignment", "pointer is aligned for the element type"),
            SafetyObligation::new("initialized", "first `len` elements are initialized"),
            SafetyObligation::new("capacity", "`len` is at most `capacity`"),
            SafetyObligation::new(
                "ownership",
                "the constructed Vec receives unique ownership and will not double-free",
            ),
        ],
        OperationFamily::MaybeUninitAssumeInit => vec![SafetyObligation::new(
            "initialized",
            "all fields/elements are initialized and valid before `assume_init`",
        )],
        OperationFamily::VecSetLen => vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ],
        OperationFamily::Transmute => vec![
            SafetyObligation::new("layout", "source and destination layouts are compatible"),
            SafetyObligation::new(
                "valid-value",
                "destination value satisfies Rust validity rules",
            ),
        ],
        OperationFamily::Zeroed => vec![SafetyObligation::new(
            "valid-zero",
            "all-zero bit pattern is a valid value for the target type",
        )],
        OperationFamily::DropInPlace => vec![
            SafetyObligation::new("pointer-live", "pointer is valid for dropping one value"),
            SafetyObligation::new("initialized", "pointed-to value is initialized"),
            SafetyObligation::new(
                "ownership",
                "value will not be dropped again or observed after drop",
            ),
        ],
        OperationFamily::AtomicPointerState => vec![
            SafetyObligation::new(
                "state-transition",
                "atomic pointer state transition preserves ownership invariants",
            ),
            SafetyObligation::new(
                "ordering",
                "atomic ordering is strong enough for readers and drop paths",
            ),
        ],
        OperationFamily::UnwrapUnchecked => vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )],
        OperationFamily::UnreachableUnchecked => vec![SafetyObligation::new(
            "unreachable",
            "control flow cannot reach this path before `unreachable_unchecked`",
        )],
        OperationFamily::UnsafeFnCall => vec![SafetyObligation::new(
            "callee-contract",
            "callee safety preconditions are satisfied",
        )],
        OperationFamily::CopyNonOverlapping => vec![
            SafetyObligation::new("non-overlap", "source and destination do not overlap"),
            SafetyObligation::new("valid-range", "both ranges are valid for count elements"),
        ],
        OperationFamily::UnsafeImplSendSync => vec![SafetyObligation::new(
            "thread-safety",
            "internal mutation and aliasing invariants uphold Send/Sync contract",
        )],
        OperationFamily::Ffi => vec![
            SafetyObligation::new(
                "abi",
                "foreign declaration matches ABI and layout on both sides",
            ),
            SafetyObligation::new(
                "ownership",
                "ownership, lifetime, and nullability contract is explicit",
            ),
        ],
        OperationFamily::PinUnchecked => vec![SafetyObligation::new(
            "pin",
            "value will not move and projections preserve pinning invariants",
        )],
        OperationFamily::GetUnchecked => vec![SafetyObligation::new(
            "bounds",
            "index is in bounds for the collection",
        )],
        OperationFamily::BoxFromRaw => vec![SafetyObligation::new(
            "ownership",
            "raw pointer was produced by compatible allocator and is uniquely owned",
        )],
        OperationFamily::PointerArithmetic => vec![SafetyObligation::new(
            "bounds",
            "pointer arithmetic stays in-bounds or one-past inside the same allocation",
        )],
        OperationFamily::NonNullUnchecked => vec![SafetyObligation::new(
            "non-null",
            "pointer is non-null before constructing NonNull",
        )],
        OperationFamily::StaticMut => vec![SafetyObligation::new(
            "global-state",
            "all access is synchronized and does not violate aliasing rules",
        )],
        OperationFamily::InlineAsm => vec![SafetyObligation::new(
            "asm",
            "inline assembly obeys register, memory, and target invariants",
        )],
        OperationFamily::TargetFeature => vec![SafetyObligation::new(
            "target-feature",
            "callers only execute this path on supported hardware",
        )],
        OperationFamily::StrFromUtf8Unchecked => {
            vec![SafetyObligation::new("utf8", "bytes are valid UTF-8")]
        }
        OperationFamily::Unknown => vec![SafetyObligation::new(
            "unknown",
            "unsafe contract could not be inferred from this syntax shape",
        )],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obligation_keys(family: &OperationFamily) -> Vec<String> {
        obligations_for(family)
            .into_iter()
            .map(|obligation| obligation.key)
            .collect()
    }

    #[test]
    fn raw_pointer_read_keeps_distinct_hazards_and_obligation_keys() {
        assert_eq!(
            hazards_for(&OperationFamily::RawPointerRead),
            vec![
                HazardKind::PointerValidity,
                HazardKind::Alignment,
                HazardKind::InitializedMemory,
                HazardKind::SameAllocation,
            ]
        );
        assert_eq!(
            obligation_keys(&OperationFamily::RawPointerRead),
            vec![
                "pointer-live".to_string(),
                "bounds".to_string(),
                "alignment".to_string(),
                "initialized".to_string(),
                "allocation".to_string(),
            ]
        );
    }

    #[test]
    fn unaligned_reads_do_not_require_alignment_obligation() {
        assert_eq!(
            hazards_for(&OperationFamily::RawPointerReadUnaligned),
            vec![
                HazardKind::PointerValidity,
                HazardKind::InitializedMemory,
                HazardKind::SameAllocation,
            ]
        );
        assert_eq!(
            obligation_keys(&OperationFamily::RawPointerReadUnaligned),
            vec![
                "pointer-live".to_string(),
                "bounds".to_string(),
                "initialized".to_string(),
                "allocation".to_string(),
            ]
        );
    }

    #[test]
    fn specialized_operations_map_to_targeted_obligations() {
        assert_eq!(
            hazards_for(&OperationFamily::VecSetLen),
            vec![HazardKind::InitializedMemory, HazardKind::Bounds]
        );
        assert_eq!(
            obligation_keys(&OperationFamily::VecSetLen),
            vec!["capacity".to_string(), "initialized".to_string()]
        );
        assert_eq!(
            hazards_for(&OperationFamily::UnsafeImplSendSync),
            vec![HazardKind::SendSyncInvariant, HazardKind::AtomicOrdering]
        );
        assert_eq!(
            obligation_keys(&OperationFamily::UnsafeImplSendSync),
            vec!["thread-safety".to_string()]
        );
        assert_eq!(
            hazards_for(&OperationFamily::DropInPlace),
            vec![
                HazardKind::PointerValidity,
                HazardKind::InitializedMemory,
                HazardKind::DropOrDeallocation,
            ]
        );
        assert_eq!(
            obligation_keys(&OperationFamily::DropInPlace),
            vec![
                "pointer-live".to_string(),
                "initialized".to_string(),
                "ownership".to_string(),
            ]
        );
        assert_eq!(
            hazards_for(&OperationFamily::UnsafeFnCall),
            vec![HazardKind::Unknown]
        );
        assert_eq!(
            obligation_keys(&OperationFamily::UnsafeFnCall),
            vec!["callee-contract".to_string()]
        );
        assert_eq!(
            hazards_for(&OperationFamily::AtomicPointerState),
            vec![HazardKind::AtomicOrdering, HazardKind::DropOrDeallocation]
        );
        assert_eq!(
            obligation_keys(&OperationFamily::AtomicPointerState),
            vec!["state-transition".to_string(), "ordering".to_string()]
        );
        assert_eq!(
            hazards_for(&OperationFamily::UnreachableUnchecked),
            vec![HazardKind::InvalidValue]
        );
        assert_eq!(
            obligation_keys(&OperationFamily::UnreachableUnchecked),
            vec!["unreachable".to_string()]
        );
    }
}
