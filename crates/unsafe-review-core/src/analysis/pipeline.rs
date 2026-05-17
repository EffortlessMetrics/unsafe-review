use super::{classify, evidence, obligations, scanner, witness};
use crate::api::{AnalysisMode, AnalyzeInput, AnalyzeOutput, DiffSource, Scope, Summary};
use crate::domain::{CardId, MissingEvidence, NextAction, ReviewCard};
use crate::input::{diff, workspace};
use crate::util::slug;
use std::fs;

pub(crate) fn analyze(input: AnalyzeInput) -> Result<AnalyzeOutput, String> {
    let repo_mode = matches!(input.scope, Scope::Repo) || matches!(input.mode, AnalysisMode::Repo);
    let diff_index = load_diff_index(&input.diff)?;
    let all_rust_files = workspace::discover_rust_files(&input.root)?;
    let candidate_files = if repo_mode || diff_index.is_empty() {
        all_rust_files.clone()
    } else {
        all_rust_files
            .iter()
            .filter(|path| diff_index.contains_file(path))
            .cloned()
            .collect::<Vec<_>>()
    };

    let mut cards = Vec::new();
    for rel in &candidate_files {
        let scanned = scanner::scan_file(&input.root, rel, Some(&diff_index), repo_mode)?;
        for scanned_site in scanned {
            let hazards = obligations::hazards_for(&scanned_site.operation.family);
            let obligations = obligations::obligations_for(&scanned_site.operation.family);
            let contract = evidence::contract_evidence(&scanned_site);
            let discharge = evidence::discharge_evidence(&scanned_site);
            let (reach, related_tests) =
                evidence::reach_evidence(&input.root, scanned_site.site.owner.as_ref());
            let routes = witness::routes_for(&hazards, scanned_site.site.owner.as_ref());
            let (class, priority, confidence) =
                classify::classify(&hazards, &contract, &discharge, &reach);
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

            let verify_commands = routes
                .iter()
                .filter_map(|route| route.command.clone())
                .collect::<Vec<_>>();
            let next_action = NextAction {
                summary: next_action_summary(&class, scanned_site.operation.family.as_str()),
                verify_commands,
            };
            let id = card_id(&scanned_site);
            cards.push(ReviewCard {
                id,
                class,
                priority,
                confidence,
                site: scanned_site.site,
                operation: scanned_site.operation,
                hazards,
                obligations,
                contract,
                discharge,
                reach,
                witness: crate::domain::WitnessEvidence::missing(),
                missing,
                routes,
                next_action,
                related_tests,
            });
        }
    }
    cards.sort_by(|left, right| {
        left.site
            .location
            .file
            .cmp(&right.site.location.file)
            .then(left.site.location.line.cmp(&right.site.location.line))
    });
    if let Some(max_cards) = input.max_cards {
        cards.truncate(max_cards);
    }
    let summary = summarize(all_rust_files.len(), candidate_files.len(), &cards);
    Ok(AnalyzeOutput {
        schema_version: "0.1".to_string(),
        tool: "unsafe-review".to_string(),
        root: input.root,
        scope: input.scope,
        mode: input.mode,
        policy: input.policy,
        summary,
        cards,
    })
}

fn load_diff_index(source: &DiffSource) -> Result<diff::DiffIndex, String> {
    match source {
        DiffSource::NoneRepoScan => Ok(diff::DiffIndex::default()),
        DiffSource::Text(text) => Ok(diff::parse_unified_diff(text)),
        DiffSource::File(path) => {
            let text = fs::read_to_string(path)
                .map_err(|err| format!("read diff {} failed: {err}", path.display()))?;
            Ok(diff::parse_unified_diff(&text))
        }
    }
}

fn summarize(rust_files: usize, changed_rust_files: usize, cards: &[ReviewCard]) -> Summary {
    let mut summary = Summary {
        rust_files,
        changed_rust_files,
        unsafe_sites: cards.len(),
        cards: cards.len(),
        ..Summary::default()
    };
    for card in cards {
        if card.class.is_actionable() {
            summary.open_actionable_gaps += 1;
        }
        match &card.class {
            crate::domain::ReviewClass::ContractMissing => summary.contract_missing += 1,
            crate::domain::ReviewClass::GuardMissing => summary.guard_missing += 1,
            crate::domain::ReviewClass::GuardedUnwitnessed => summary.guarded_unwitnessed += 1,
            crate::domain::ReviewClass::UnsafeUnreached => summary.unsafe_unreached += 1,
            crate::domain::ReviewClass::RequiresLoom => summary.requires_loom += 1,
            crate::domain::ReviewClass::MiriUnsupported => summary.miri_unsupported += 1,
            crate::domain::ReviewClass::StaticUnknown => summary.static_unknown += 1,
            _ => {}
        }
    }
    summary
}

fn next_action_summary(class: &crate::domain::ReviewClass, operation: &str) -> String {
    match class {
        crate::domain::ReviewClass::ContractMissing => "Add a precise `# Safety` section or `SAFETY:` comment that names the required conditions.".to_string(),
        crate::domain::ReviewClass::GuardMissing => format!("Add or expose the local guard that discharges the `{operation}` safety obligation."),
        crate::domain::ReviewClass::RequiresLoom => "Add or update a Loom/Shuttle model for the changed concurrency invariant.".to_string(),
        crate::domain::ReviewClass::MiriUnsupported => "Use sanitizer/cargo-careful or an explicit FFI boundary contract; Miri may not exercise this seam.".to_string(),
        crate::domain::ReviewClass::UnsafeUnreached => "Add or identify a focused test path that reaches the safe wrapper around this unsafe seam.".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{AnalysisMode, DiffSource, PolicyMode};
    use crate::domain::{HazardKind, ReviewClass};
    use std::path::PathBuf;

    #[test]
    fn raw_pointer_alignment_fixture_emits_guard_missing_card() -> Result<(), String> {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/raw_pointer_alignment");
        let output = analyze(AnalyzeInput {
            root: root.clone(),
            scope: Scope::Diff,
            diff: DiffSource::File(root.join("change.diff")),
            mode: AnalysisMode::Draft,
            policy: PolicyMode::Advisory,
            include_unchanged_tests: true,
            max_cards: None,
        })?;

        assert_eq!(output.summary.cards, 1);
        assert_eq!(output.summary.guard_missing, 1);
        assert_eq!(output.summary.unsafe_unreached, 0);

        let card = &output.cards[0];
        assert_eq!(card.class, ReviewClass::GuardMissing);
        assert!(card.hazards.contains(&HazardKind::Alignment));
        assert!(card.contract.present);
        assert!(!card.discharge.present);
        assert_eq!(card.reach.state, "owner_reached");
        assert!(card.missing.iter().any(|missing| missing.kind == "guard"));
        Ok(())
    }
}
