use super::{classify, evidence, obligations, receipts, scanner, witness};
use crate::api::{AnalysisMode, AnalyzeInput, AnalyzeOutput, DiffSource, Scope, Summary};
use crate::domain::{CardId, MissingEvidence, NextAction, Priority, ReviewCard, ReviewClass};
use crate::input::{diff, workspace};
use crate::policy::PolicyState;
use crate::util::{slug, stable_hash_hex};
use std::collections::BTreeMap;
use std::fs;

pub(crate) fn analyze(input: AnalyzeInput) -> Result<AnalyzeOutput, String> {
    let repo_mode = matches!(input.scope, Scope::Repo) || matches!(input.mode, AnalysisMode::Repo);
    let diff_index = load_diff_index(&input.diff)?;
    let all_rust_files = workspace::discover_rust_files(&input.root)?;
    let package = package_name(&input.root);
    let policy_state = PolicyState::load(&input.root)?;
    let receipt_index = receipts::ReceiptIndex::load(&input.root)?;
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
    let mut identity_counts = BTreeMap::new();
    for rel in &candidate_files {
        let scanned = scanner::scan_file(&input.root, rel, Some(&diff_index), repo_mode)?;
        for scanned_site in scanned {
            let hazards = obligations::hazards_for(&scanned_site.operation.family);
            let obligations = obligations::obligations_for(&scanned_site.operation.family);
            let contract = evidence::contract_evidence(&scanned_site);
            let (reach, related_tests) =
                evidence::reach_evidence(&input.root, scanned_site.site.owner.as_ref());
            let mut obligation_evidence =
                evidence::obligation_evidence(&scanned_site, &obligations, &contract, &reach);
            let discharge = evidence::summarize_discharge(&obligation_evidence);
            let routes = witness::routes_for(&hazards, scanned_site.site.owner.as_ref());
            let (mut class, mut priority, confidence) =
                classify::classify(&hazards, &contract, &discharge, &reach);
            let mut missing = Vec::new();
            if !contract.present {
                let contract_missing_message = if scanned_site.site.public_api_surface {
                    "Missing public `# Safety` documentation for unsafe API"
                } else {
                    "Missing `# Safety` documentation or `SAFETY:` comment"
                };
                missing.push(MissingEvidence::new("contract", contract_missing_message));
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
            let verify_commands = routes
                .iter()
                .filter_map(|route| route.command.clone())
                .collect::<Vec<_>>();
            let id = card_id(&package, &scanned_site, &hazards, &mut identity_counts);
            let witness_evidence = receipt_index
                .evidence_for(&id)
                .unwrap_or_else(crate::domain::WitnessEvidence::missing);
            if witness_evidence.present {
                for evidence in &mut obligation_evidence {
                    evidence.witness =
                        crate::domain::EvidenceState::present(&witness_evidence.summary);
                }
                if class == ReviewClass::GuardedUnwitnessed {
                    class = ReviewClass::GuardedAndWitnessed;
                    priority = Priority::Low;
                }
            }
            if policy_state.is_suppressed(&id) {
                class = ReviewClass::Suppressed;
                priority = Priority::Low;
            } else if policy_state.is_baseline_known(&id) {
                class = ReviewClass::BaselineKnown;
                priority = Priority::Low;
            }
            let next_action = NextAction {
                summary: next_action_summary(&class, scanned_site.operation.family.as_str()),
                verify_commands,
            };
            if !witness_evidence.present {
                missing.push(MissingEvidence::new(
                    "witness",
                    "No witness receipt imported for this card",
                ));
            }
            cards.push(ReviewCard {
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
                witness: witness_evidence,
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

fn package_name(root: &std::path::Path) -> String {
    let Ok(text) = fs::read_to_string(root.join("Cargo.toml")) else {
        return root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("workspace")
            .to_string();
    };
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package || !trimmed.starts_with("name") {
            continue;
        }
        let Some((_key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let name = value.trim().trim_matches('"');
        if !name.is_empty() {
            return name.to_string();
        }
    }
    root.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string()
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
        crate::domain::ReviewClass::BaselineKnown => "Known baseline card; keep the ledger owner and review date current.".to_string(),
        crate::domain::ReviewClass::Suppressed => "Suppressed card; keep the owner, reason, evidence, and review or expiry date current.".to_string(),
        _ => "Attach a focused witness receipt or mark the static limitation explicitly.".to_string(),
    }
}

fn card_id(
    package: &str,
    scanned: &scanner::ScannedSite,
    hazards: &[crate::domain::HazardKind],
    identity_counts: &mut BTreeMap<String, usize>,
) -> CardId {
    let base = card_identity_base(package, scanned, hazards);
    let next = identity_counts.entry(base.clone()).or_insert(0);
    *next += 1;
    CardId(format!("{base}-c{}", *next))
}

fn card_identity_base(
    package: &str,
    scanned: &scanner::ScannedSite,
    hazards: &[crate::domain::HazardKind],
) -> String {
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
    let normalized = normalize_snippet(&scanned.operation.expression);
    let snippet_hash = stable_hash_hex(&normalized);
    let hazard = hazards.first().map_or("unknown", |hazard| hazard.as_str());
    format!(
        "UR-{}-{}-{}-{}-{}-{}-{}-{}",
        slug(package),
        slug(&file),
        slug(&owner),
        scanned.site.kind.as_str(),
        scanned.operation.family.as_str(),
        slug(&operation_path(scanned)),
        &snippet_hash[..12],
        hazard
    )
}

fn normalize_snippet(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn operation_path(scanned: &scanner::ScannedSite) -> String {
    if scanned.operation.family == crate::domain::OperationFamily::RawPointerDeref {
        return "deref".to_string();
    }
    if scanned.operation.family == crate::domain::OperationFamily::Unknown {
        return scanned
            .site
            .owner
            .clone()
            .unwrap_or_else(|| scanned.site.kind.as_str().to_string());
    }
    let normalized = normalize_snippet(&scanned.operation.expression);
    let target = normalized
        .split('(')
        .next()
        .unwrap_or(normalized.as_str())
        .trim();
    if let Some((_prefix, method)) = target.rsplit_once('.') {
        return method.trim_matches(':').to_string();
    }
    if let Some((_prefix, function)) = target.rsplit_once("::") {
        return function.trim_matches(':').to_string();
    }
    scanned.operation.family.as_str().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{AnalysisMode, DiffSource, PolicyMode};
    use crate::domain::{HazardKind, OperationFamily, ReviewCard, ReviewClass, UnsafeSiteKind};
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn raw_pointer_v1_operation_cards_are_concrete() -> Result<(), String> {
        let cases = [
            (
                "raw_pointer_alignment",
                OperationFamily::RawPointerRead,
                true,
            ),
            ("raw_pointer_deref", OperationFamily::RawPointerDeref, true),
            (
                "raw_pointer_read_unaligned",
                OperationFamily::RawPointerReadUnaligned,
                false,
            ),
            (
                "raw_pointer_write_assignment",
                OperationFamily::RawPointerWrite,
                true,
            ),
            (
                "split_raw_pointer_read_call",
                OperationFamily::RawPointerRead,
                true,
            ),
        ];

        for (fixture, expected_family, expects_alignment) in cases {
            let output = fixture_output(fixture)?;
            let card = single_card(fixture, &output)?;

            assert_eq!(
                card.operation.family, expected_family,
                "{fixture} should emit the concrete operation family"
            );
            assert_eq!(card.site.kind, UnsafeSiteKind::Operation);
            assert_eq!(card.class, ReviewClass::GuardMissing);
            assert_eq!(
                card.hazards.contains(&HazardKind::Alignment),
                expects_alignment,
                "{fixture} alignment hazard expectation drifted"
            );
            assert!(card.contract.present);
            assert_eq!(card.reach.state, "owner_reached");
            assert!(card.missing.iter().any(|missing| missing.kind == "guard"));
            assert!(
                card.next_action
                    .verify_commands
                    .iter()
                    .any(|command| command.contains("miri test")),
                "{fixture} should recommend a concrete Miri witness"
            );
            assert_no_unknown_wrapper_card(fixture, &output);
        }
        Ok(())
    }

    #[test]
    fn unaligned_raw_pointer_read_does_not_require_alignment_guard() -> Result<(), String> {
        let output = fixture_output("raw_pointer_read_unaligned")?;
        let card = single_card("raw_pointer_read_unaligned", &output)?;

        assert_eq!(
            card.operation.family,
            OperationFamily::RawPointerReadUnaligned
        );
        assert!(!card.hazards.contains(&HazardKind::Alignment));
        assert!(
            card.obligations
                .iter()
                .all(|obligation| obligation.key != "alignment")
        );
        assert!(card.contract.present);
        assert!(
            obligation_discharge_present(card, "bounds"),
            "length evidence should still satisfy the bounds obligation"
        );
        assert_eq!(card.class, ReviewClass::GuardMissing);
        Ok(())
    }

    #[test]
    fn raw_pointer_v1_evidence_stays_obligation_specific() -> Result<(), String> {
        for fixture in ["raw_pointer_alignment", "comment_alignment_not_guard"] {
            let output = fixture_output(fixture)?;
            let card = single_card(fixture, &output)?;

            assert!(
                obligation_discharge_present(card, "bounds"),
                "{fixture} should keep length or bounds evidence"
            );
            assert!(
                !obligation_discharge_present(card, "alignment"),
                "{fixture} should not let comments or length checks discharge alignment"
            );
            assert!(!card.discharge.present);
        }
        Ok(())
    }

    #[test]
    fn raw_pointer_v1_negative_cases_stay_pinned() -> Result<(), String> {
        let safe_reference = fixture_output("safe_reference_deref_no_cards")?;
        assert_eq!(safe_reference.summary.cards, 0);
        assert!(safe_reference.cards.is_empty());

        let split_unknown = fixture_output("split_unsafe_block")?;
        let card = single_card("split_unsafe_block", &split_unknown)?;
        assert_eq!(card.site.kind, UnsafeSiteKind::UnsafeBlock);
        assert_eq!(card.operation.family, OperationFamily::Unknown);
        assert_eq!(card.class, ReviewClass::ContractMissing);
        Ok(())
    }

    #[test]
    fn public_unsafe_api_contract_evidence_requires_safety_docs() -> Result<(), String> {
        for fixture in [
            "public_unsafe_fn_missing_safety",
            "public_unsafe_trait_missing_safety",
            "public_unsafe_fn_safety_comment_not_docs",
        ] {
            let output = fixture_output(fixture)?;
            let card = single_card(fixture, &output)?;

            assert_eq!(card.class, ReviewClass::ContractMissing);
            assert!(card.site.public_api_surface);
            assert!(!card.contract.present);
            assert!(
                card.contract.summary.contains("# Safety"),
                "{fixture} should ask for public safety docs"
            );
            assert!(
                card.missing.iter().any(|missing| missing.kind == "contract"
                    && missing.message.contains("public `# Safety`")),
                "{fixture} should not accept local SAFETY prose as public API docs"
            );
            assert!(
                card.site.owner.is_some(),
                "{fixture} should preserve the public API owner in the card"
            );
        }
        Ok(())
    }

    #[test]
    fn private_unsafe_helper_can_use_local_safety_comment() -> Result<(), String> {
        let output = fixture_output("private_unsafe_helper_safety_comment")?;
        let card = single_card("private_unsafe_helper_safety_comment", &output)?;

        assert_eq!(card.class, ReviewClass::GuardMissing);
        assert!(!card.site.public_api_surface);
        assert!(card.contract.present);
        assert_eq!(card.reach.state, "owner_reached");
        assert!(
            !card
                .missing
                .iter()
                .any(|missing| missing.kind == "contract")
        );
        Ok(())
    }

    #[test]
    fn card_identity_is_stable_across_line_drift() -> Result<(), String> {
        let original = fixture_output("raw_pointer_alignment")?;
        let drifted = fixture_output("raw_pointer_alignment_line_drift")?;
        let original_card = single_card("raw_pointer_alignment", &original)?;
        let drifted_card = single_card("raw_pointer_alignment_line_drift", &drifted)?;

        assert_ne!(
            original_card.site.location.line, drifted_card.site.location.line,
            "fixture should prove line drift occurred"
        );
        assert_eq!(
            original_card.id, drifted_card.id,
            "card identity should not include the line number"
        );
        Ok(())
    }

    #[test]
    fn card_identity_counts_duplicate_sites() -> Result<(), String> {
        let output = fixture_output("duplicate_raw_pointer_reads")?;
        if output.cards.len() != 2 {
            return Err(format!(
                "duplicate_raw_pointer_reads should emit two cards, got {}",
                output.cards.len()
            ));
        }
        let first = &output.cards[0];
        let second = &output.cards[1];

        assert_eq!(first.operation.family, OperationFamily::RawPointerRead);
        assert_eq!(second.operation.family, OperationFamily::RawPointerRead);
        assert_ne!(first.id, second.id);
        assert!(first.id.0.ends_with("-c1"));
        assert!(second.id.0.ends_with("-c2"));
        assert_eq!(
            identity_without_count(&first.id),
            identity_without_count(&second.id)
        );
        Ok(())
    }

    #[test]
    fn baseline_policy_marks_exact_card_identity_as_known() -> Result<(), String> {
        let root = copy_fixture_to_temp("raw_pointer_alignment", "unsafe-review-baseline-match")?;
        let card_id = single_card("raw_pointer_alignment", &fixture_output_at(&root)?)?
            .id
            .0
            .clone();
        write_policy_ledger(
            &root,
            "unsafe-review-baseline.toml",
            &card_id,
            "review_after",
        )?;

        let output = fixture_output_at(&root)?;
        let card = single_card("raw_pointer_alignment baseline", &output)?;

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp fixture failed: {err}"))?;
        assert_eq!(card.class, ReviewClass::BaselineKnown);
        assert_eq!(card.priority, Priority::Low);
        assert_eq!(output.summary.open_actionable_gaps, 0);
        assert!(card.next_action.summary.contains("Known baseline"));
        Ok(())
    }

    #[test]
    fn suppression_policy_marks_exact_card_identity_as_suppressed() -> Result<(), String> {
        let root =
            copy_fixture_to_temp("raw_pointer_alignment", "unsafe-review-suppression-match")?;
        let card_id = single_card("raw_pointer_alignment", &fixture_output_at(&root)?)?
            .id
            .0
            .clone();
        write_policy_ledger(
            &root,
            "unsafe-review-suppressions.toml",
            &card_id,
            "expires",
        )?;

        let output = fixture_output_at(&root)?;
        let card = single_card("raw_pointer_alignment suppression", &output)?;

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp fixture failed: {err}"))?;
        assert_eq!(card.class, ReviewClass::Suppressed);
        assert_eq!(card.priority, Priority::Low);
        assert_eq!(output.summary.open_actionable_gaps, 0);
        assert!(card.next_action.summary.contains("Suppressed"));
        Ok(())
    }

    #[test]
    fn imported_receipt_marks_witness_evidence_present_by_exact_card_identity() -> Result<(), String>
    {
        let root = copy_fixture_to_temp("raw_pointer_alignment", "unsafe-review-receipt-match")?;
        let card_id = single_card("raw_pointer_alignment", &fixture_output_at(&root)?)?
            .id
            .0
            .clone();
        write_receipt(&root, &card_id)?;

        let output = fixture_output_at(&root)?;
        let card = single_card("raw_pointer_alignment receipt", &output)?;

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp fixture failed: {err}"))?;
        assert!(card.witness.present);
        assert!(card.witness.summary.contains("miri"));
        assert!(card.witness.summary.contains("ran"));
        assert!(!card.missing.iter().any(|missing| missing.kind == "witness"));
        assert!(
            card.obligation_evidence
                .iter()
                .all(|evidence| evidence.witness.present)
        );
        Ok(())
    }

    fn fixture_output(name: &str) -> Result<AnalyzeOutput, String> {
        let root = fixture_root(name);
        fixture_output_at(&root)
    }

    fn fixture_output_at(root: &Path) -> Result<AnalyzeOutput, String> {
        analyze(AnalyzeInput {
            root: root.to_path_buf(),
            scope: Scope::Diff,
            diff: DiffSource::File(root.join("change.diff")),
            mode: AnalysisMode::Draft,
            policy: PolicyMode::Advisory,
            include_unchanged_tests: true,
            max_cards: None,
        })
    }

    fn fixture_root(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(name)
    }

    fn single_card<'a>(fixture: &str, output: &'a AnalyzeOutput) -> Result<&'a ReviewCard, String> {
        if output.cards.len() != 1 {
            return Err(format!(
                "{fixture} should emit exactly one card, got {}",
                output.cards.len()
            ));
        }
        Ok(&output.cards[0])
    }

    fn obligation_discharge_present(card: &ReviewCard, key: &str) -> bool {
        card.obligation_evidence
            .iter()
            .find(|evidence| evidence.obligation.key == key)
            .is_some_and(|evidence| evidence.discharge.present)
    }

    fn assert_no_unknown_wrapper_card(fixture: &str, output: &AnalyzeOutput) {
        assert!(
            output.cards.iter().all(|card| {
                !(card.site.kind == UnsafeSiteKind::UnsafeBlock
                    && card.operation.family == OperationFamily::Unknown)
            }),
            "{fixture} should suppress duplicate unknown unsafe-block wrapper cards"
        );
    }

    fn identity_without_count(id: &CardId) -> &str {
        id.0.rsplit_once("-c")
            .map_or(id.0.as_str(), |(base, _count)| base)
    }

    fn copy_fixture_to_temp(name: &str, prefix: &str) -> Result<PathBuf, String> {
        let source = fixture_root(name);
        let target = unique_temp_dir(prefix)?;
        copy_dir_all(&source, &target)?;
        Ok(target)
    }

    fn copy_dir_all(source: &Path, target: &Path) -> Result<(), String> {
        fs::create_dir_all(target)
            .map_err(|err| format!("create {} failed: {err}", target.display()))?;
        let entries = fs::read_dir(source)
            .map_err(|err| format!("read {} failed: {err}", source.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("read_dir entry failed: {err}"))?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());
            if source_path.is_dir() {
                copy_dir_all(&source_path, &target_path)?;
            } else {
                fs::copy(&source_path, &target_path).map_err(|err| {
                    format!(
                        "copy {} to {} failed: {err}",
                        source_path.display(),
                        target_path.display()
                    )
                })?;
            }
        }
        Ok(())
    }

    fn write_policy_ledger(
        root: &Path,
        file_name: &str,
        card_id: &str,
        date_key: &str,
    ) -> Result<(), String> {
        let policy_dir = root.join("policy");
        fs::create_dir_all(&policy_dir)
            .map_err(|err| format!("create {} failed: {err}", policy_dir.display()))?;
        fs::write(
            policy_dir.join(file_name),
            format!(
                r#"schema_version = "0.1"
status = "active"

[[entries]]
card_id = "{card_id}"
owner = "core/policy"
reason = "fixture policy match"
evidence = "test fixture"
{date_key} = "2026-08-01"
"#
            ),
        )
        .map_err(|err| format!("write policy ledger failed: {err}"))
    }

    fn write_receipt(root: &Path, card_id: &str) -> Result<(), String> {
        let receipt_dir = root.join(".unsafe-review").join("receipts");
        fs::create_dir_all(&receipt_dir)
            .map_err(|err| format!("create {} failed: {err}", receipt_dir.display()))?;
        fs::write(
            receipt_dir.join("miri.json"),
            format!(
                r#"{{
  "schema_version": "0.1",
  "card_id": "{card_id}",
  "tool": "miri",
  "strength": "ran",
  "summary": "focused fixture witness passed",
  "command": "cargo +nightly miri test read_header",
  "limitations": ["fixture only"]
}}"#
            ),
        )
        .map_err(|err| format!("write receipt failed: {err}"))
    }

    fn unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("system clock before UNIX_EPOCH: {err}"))?
            .as_nanos();
        Ok(std::env::temp_dir().join(format!("{prefix}-{nanos}")))
    }
}
