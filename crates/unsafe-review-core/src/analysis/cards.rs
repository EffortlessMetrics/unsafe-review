use super::{classify, evidence, obligations, scanner, witness};
use crate::domain::{CardId, MissingEvidence, NextAction, ReviewCard, ReviewClass};
use crate::util::slug;
use std::path::Path;

pub(crate) fn build(root: &Path, scanned_site: scanner::ScannedSite) -> ReviewCard {
    let hazards = obligations::hazards_for(&scanned_site.operation.family);
    let obligations = obligations::obligations_for(&scanned_site.operation.family);
    let contract = evidence::contract_evidence(&scanned_site);
    let (reach, related_tests) = evidence::reach_evidence(root, scanned_site.site.owner.as_ref());
    let obligation_evidence =
        evidence::obligation_evidence(&scanned_site, &obligations, &contract, &reach);
    let discharge = evidence::summarize_discharge(&obligation_evidence);
    let routes = witness::routes_for(&hazards, scanned_site.site.owner.as_ref());
    let (class, priority, confidence) = classify::classify(&hazards, &contract, &discharge, &reach);
    let missing = missing_evidence(&contract, &discharge, &reach);
    let next_action = next_action(&class, scanned_site.operation.family.as_str(), &routes);
    let id = card_id(&scanned_site);

    ReviewCard {
        id,
        class,
        priority,
        confidence,
        site: scanned_site.site,
        operation: scanned_site.operation,
        hazards,
        obligations,
        obligation_evidence,
        contract,
        discharge,
        reach,
        witness: crate::domain::WitnessEvidence::missing(),
        missing,
        routes,
        next_action,
        related_tests,
    }
}

fn missing_evidence(
    contract: &crate::domain::ContractEvidence,
    discharge: &crate::domain::DischargeEvidence,
    reach: &crate::domain::ReachEvidence,
) -> Vec<MissingEvidence> {
    let mut missing = Vec::new();
    if !contract.present {
        missing.push(MissingEvidence::new(
            "contract",
            "Missing `# Safety` documentation or `SAFETY:` comment",
        ));
    }
    if !discharge.present {
        missing.push(MissingEvidence::new(
            "guard",
            "Missing visible local guard for inferred safety obligations",
        ));
    }
    if reach.state == "unreached" {
        missing.push(MissingEvidence::new(
            "reach",
            "No related test path was found by static search",
        ));
    }
    missing.push(MissingEvidence::new(
        "witness",
        "No witness receipt imported for this card",
    ));
    missing
}

fn next_action(
    class: &ReviewClass,
    operation: &str,
    routes: &[crate::domain::WitnessRoute],
) -> NextAction {
    let verify_commands = routes
        .iter()
        .filter_map(|route| route.command.clone())
        .collect::<Vec<_>>();
    NextAction {
        summary: next_action_summary(class, operation),
        verify_commands,
    }
}

fn next_action_summary(class: &ReviewClass, operation: &str) -> String {
    match class {
        ReviewClass::ContractMissing => "Add a precise `# Safety` section or `SAFETY:` comment that names the required conditions.".to_string(),
        ReviewClass::GuardMissing => format!("Add or expose the local guard that discharges the `{operation}` safety obligation."),
        ReviewClass::RequiresLoom => "Add or update a Loom/Shuttle model for the changed concurrency invariant.".to_string(),
        ReviewClass::MiriUnsupported => "Use sanitizer/cargo-careful or an explicit FFI boundary contract; Miri may not exercise this seam.".to_string(),
        ReviewClass::UnsafeUnreached => "Add or identify a focused test path that reaches the safe wrapper around this unsafe seam.".to_string(),
        _ => "Attach a focused witness receipt or mark the static limitation explicitly.".to_string(),
    }
}

fn card_id(scanned: &scanner::ScannedSite) -> CardId {
    let file = scanned
        .site
        .location
        .file
        .to_string_lossy()
        .replace(['/', '\\'], "_");
    let owner = scanned
        .site
        .owner
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    CardId(format!(
        "UR-{}-{}-{}-{}",
        slug(&file),
        scanned.site.location.line,
        slug(&owner),
        scanned.operation.family.as_str()
    ))
}
