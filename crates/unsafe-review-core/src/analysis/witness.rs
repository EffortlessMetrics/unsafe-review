use crate::domain::{HazardKind, WitnessKind, WitnessRoute};

pub(crate) fn routes_for(hazards: &[HazardKind], owner: Option<&String>) -> Vec<WitnessRoute> {
    if hazards.iter().any(|h| {
        matches!(
            h,
            HazardKind::SendSyncInvariant
                | HazardKind::AtomicOrdering
                | HazardKind::StaticMutGlobalState
        )
    }) {
        return vec![WitnessRoute {
            kind: WitnessKind::Loom,
            reason: "Concurrency or shared-mutable-state hazard; interleaving model is a better first witness than Miri alone".to_string(),
            command: owner.map(|name| format!("cargo test {name}_loom -- --nocapture")),
            required: false,
        }];
    }
    if hazards
        .iter()
        .any(|h| matches!(h, HazardKind::FfiAbi | HazardKind::FfiOwnership))
    {
        return vec![WitnessRoute {
            kind: WitnessKind::AddressSanitizer,
            reason: "FFI boundary detected; sanitizer/cargo-careful is usually more suitable than Miri for foreign calls".to_string(),
            command: owner.map(|name| format!("RUSTFLAGS='-Z sanitizer=address' cargo +nightly test {name}")),
            required: false,
        }];
    }
    if hazards.iter().any(|h| {
        matches!(
            h,
            HazardKind::InvalidValue
                | HazardKind::InitializedMemory
                | HazardKind::Alignment
                | HazardKind::PointerValidity
                | HazardKind::AliasingOrProvenance
        )
    }) {
        return vec![WitnessRoute {
            kind: WitnessKind::Miri,
            reason: "Pure-Rust UB-adjacent hazard; Miri is the strongest concrete-execution witness when the path is supported".to_string(),
            command: owner.map(|name| format!("cargo +nightly miri test {name}")),
            required: false,
        }, WitnessRoute {
            kind: WitnessKind::CargoCareful,
            reason: "cargo-careful is a cheaper compatibility-oriented runtime check for several unsafe preconditions".to_string(),
            command: owner.map(|name| format!("cargo +nightly careful test {name}")),
            required: false,
        }];
    }
    vec![WitnessRoute {
        kind: WitnessKind::HumanDeepReview,
        reason: "The analyzer could not infer a precise witness route; use manual unsafe contract review".to_string(),
        command: None,
        required: false,
    }]
}
