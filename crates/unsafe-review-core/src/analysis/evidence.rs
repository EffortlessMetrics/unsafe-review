use crate::analysis::scanner::ScannedSite;
use crate::domain::{
    ContractEvidence, DischargeEvidence, EvidenceState, ObligationEvidence, ReachEvidence,
    RelatedTest, SafetyObligation,
};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn contract_evidence(site: &ScannedSite) -> ContractEvidence {
    let context = site.context_before.join("\n");
    if context.contains("# Safety") {
        return ContractEvidence::present("Nearby `# Safety` documentation was detected");
    }
    if context.contains("SAFETY:") || site.site.snippet.contains("SAFETY:") {
        return ContractEvidence::present("Nearby `SAFETY:` comment was detected");
    }
    ContractEvidence::missing()
}

pub(crate) fn obligation_evidence(
    site: &ScannedSite,
    obligations: &[SafetyObligation],
    contract: &ContractEvidence,
    reach: &ReachEvidence,
) -> Vec<ObligationEvidence> {
    let text = code_context(site);
    let lower = text.to_ascii_lowercase();
    obligations
        .iter()
        .map(|obligation| ObligationEvidence {
            obligation: obligation.clone(),
            contract: contract_state(contract),
            discharge: discharge_state_for(&obligation.key, &lower),
            reach: reach_state(reach),
            witness: EvidenceState::missing("No imported witness receipt was found"),
        })
        .collect()
}

pub(crate) fn summarize_discharge(evidence: &[ObligationEvidence]) -> DischargeEvidence {
    if evidence.is_empty() {
        return DischargeEvidence::missing();
    }
    if evidence
        .iter()
        .all(|obligation| obligation.discharge.present)
    {
        return DischargeEvidence::present(
            "All inferred safety obligations have visible local guard evidence",
        );
    }
    if evidence
        .iter()
        .any(|obligation| obligation.discharge.present)
    {
        return DischargeEvidence::missing_with(
            "Some inferred safety obligations are missing local guard evidence",
        );
    }
    DischargeEvidence::missing()
}

fn code_context(site: &ScannedSite) -> String {
    site.context_before
        .iter()
        .chain(std::iter::once(&site.site.snippet))
        .chain(site.context_after.iter())
        .map(|line| {
            line.split_once("//")
                .map_or(line.as_str(), |(code, _comment)| code)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn contract_state(contract: &ContractEvidence) -> EvidenceState {
    if contract.present {
        EvidenceState::present(&contract.summary)
    } else {
        EvidenceState::missing(&contract.summary)
    }
}

fn reach_state(reach: &ReachEvidence) -> EvidenceState {
    if reach.state == "unreached" || reach.state == "unknown" {
        EvidenceState::missing(&reach.summary)
    } else {
        EvidenceState::present(&reach.summary)
    }
}

fn discharge_state_for(key: &str, lower: &str) -> EvidenceState {
    match key {
        "alignment" => {
            if has_alignment_guard(lower) {
                EvidenceState::present("Alignment guard code was detected")
            } else {
                EvidenceState::missing("No alignment guard code was detected")
            }
        }
        "bounds" | "valid-range" => {
            if has_length_or_bounds_guard(lower) {
                EvidenceState::present("Length or bounds guard code was detected")
            } else {
                EvidenceState::missing("No length or bounds guard code was detected")
            }
        }
        "capacity" => {
            if lower.contains("capacity") || lower.contains("cap()") {
                EvidenceState::present("Capacity guard code was detected")
            } else {
                EvidenceState::missing("No capacity guard code was detected")
            }
        }
        "non-null" | "pointer-live" => {
            if lower.contains("is_null") || lower.contains("non_null") || lower.contains("nonnull")
            {
                EvidenceState::present("Nullability guard code was detected")
            } else {
                EvidenceState::missing("No nullability guard code was detected")
            }
        }
        _ => EvidenceState::missing("No obligation-specific guard code was detected"),
    }
}

fn has_length_or_bounds_guard(lower: &str) -> bool {
    lower.contains("len") && (lower.contains(">=") || lower.contains('<'))
}

fn has_alignment_guard(lower: &str) -> bool {
    lower.contains("is_aligned")
        || lower.contains("align_offset")
        || lower.contains("align_of")
        || lower.contains("addr() %")
        || lower.contains("as usize %")
}

pub(crate) fn reach_evidence(
    root: &Path,
    owner: Option<&String>,
) -> (ReachEvidence, Vec<RelatedTest>) {
    let Some(owner) = owner else {
        return (
            ReachEvidence {
                state: "unknown".to_string(),
                summary: "No owner function could be inferred".to_string(),
            },
            Vec::new(),
        );
    };
    let mut tests = Vec::new();
    let test_files = collect_test_files(root).unwrap_or_default();
    for rel in test_files {
        let abs = root.join(&rel);
        let Ok(text) = fs::read_to_string(&abs) else {
            continue;
        };
        if !text.contains(owner) {
            continue;
        }
        let mut last_test: Option<(String, usize)> = None;
        for (idx, line) in text.lines().enumerate() {
            if line.contains("#[test]") {
                last_test = Some(("test".to_string(), idx + 1));
            }
            if let Some(name) = parse_test_name(line) {
                last_test = Some((name, idx + 1));
            }
            if line.contains(owner) {
                let (name, line_no) = last_test
                    .clone()
                    .unwrap_or_else(|| (format!("mentions {owner}"), idx + 1));
                tests.push(RelatedTest {
                    name,
                    file: rel.to_string_lossy().replace('\\', "/"),
                    line: line_no,
                });
                break;
            }
        }
    }
    if tests.is_empty() {
        (
            ReachEvidence {
                state: "unreached".to_string(),
                summary: format!("No static test mention of owner `{owner}` was found"),
            },
            tests,
        )
    } else {
        (
            ReachEvidence {
                state: "owner_reached".to_string(),
                summary: format!(
                    "{} related test file(s) mention owner `{owner}`",
                    tests.len()
                ),
            },
            tests,
        )
    }
}

fn parse_test_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !(trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ")) {
        return None;
    }
    let pos = trimmed.find("fn ")?;
    let rest = &trimmed[pos + 3..];
    let mut name = String::new();
    for ch in rest.chars() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            name.push(ch);
        } else {
            break;
        }
    }
    (!name.is_empty()).then_some(name)
}

fn collect_test_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    visit(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

fn visit(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|err| format!("read {} failed: {err}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read_dir entry failed: {err}"))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if matches!(
                name.as_str(),
                ".git" | "target" | ".unsafe-review" | "node_modules"
            ) {
                continue;
            }
            visit(root, &path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            let rel_text = rel.to_string_lossy();
            if rel_text.contains("tests")
                || rel_text.contains("test")
                || fs::read_to_string(&path).is_ok_and(|text| text.contains("#[test]"))
            {
                out.push(rel);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        OperationFamily, SourceLocation, UnsafeOperation, UnsafeSite, UnsafeSiteKind,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn site_with_context(before: Vec<&str>, snippet: &str, after: Vec<&str>) -> ScannedSite {
        ScannedSite {
            site: UnsafeSite {
                location: SourceLocation::new(PathBuf::from("src/lib.rs"), 10, 5),
                kind: UnsafeSiteKind::Operation,
                owner: Some("read_word".to_string()),
                visibility: "private".to_string(),
                public_api_surface: false,
                changed: true,
                snippet: snippet.to_string(),
            },
            operation: UnsafeOperation {
                family: OperationFamily::RawPointerRead,
                expression: snippet.to_string(),
            },
            context_before: before.into_iter().map(str::to_string).collect(),
            context_after: after.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn contract_evidence_accepts_safety_docs_and_safety_comments() {
        let doc_site = site_with_context(
            vec!["/// # Safety", "/// caller must pass a valid pointer"],
            "unsafe { ptr.read() }",
            Vec::new(),
        );
        let comment_site = site_with_context(
            Vec::new(),
            "unsafe { ptr.read() } // SAFETY: checked above",
            Vec::new(),
        );
        let missing_site = site_with_context(Vec::new(), "unsafe { ptr.read() }", Vec::new());

        assert!(contract_evidence(&doc_site).present);
        assert!(contract_evidence(&comment_site).present);
        assert!(!contract_evidence(&missing_site).present);
    }

    #[test]
    fn obligation_evidence_ignores_comment_only_guards() {
        let site = site_with_context(
            vec!["// len >= 4", "// align_of::<u32>() proves alignment"],
            "unsafe { ptr.read() }",
            Vec::new(),
        );
        let contract = ContractEvidence::present("contract present");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "test mentions owner".to_string(),
        };
        let evidence = obligation_evidence(
            &site,
            &[
                SafetyObligation::new("bounds", "bounds checked"),
                SafetyObligation::new("alignment", "alignment checked"),
            ],
            &contract,
            &reach,
        );

        assert!(evidence.iter().all(|item| item.contract.present));
        assert!(evidence.iter().all(|item| item.reach.present));
        assert!(evidence.iter().all(|item| !item.discharge.present));
        assert!(!summarize_discharge(&evidence).present);
    }

    #[test]
    fn obligation_evidence_detects_real_guards_and_partial_discharge() {
        let site = site_with_context(
            vec!["if len < 4 { return None; }"],
            "unsafe { ptr.read() }",
            vec!["let _ = core::mem::align_of::<u32>();"],
        );
        let contract = ContractEvidence::present("contract present");
        let reach = ReachEvidence {
            state: "unreached".to_string(),
            summary: "no tests".to_string(),
        };
        let evidence = obligation_evidence(
            &site,
            &[
                SafetyObligation::new("bounds", "bounds checked"),
                SafetyObligation::new("alignment", "alignment checked"),
                SafetyObligation::new("initialized", "initialized"),
            ],
            &contract,
            &reach,
        );

        let present = evidence
            .iter()
            .filter(|item| item.discharge.present)
            .count();
        assert_eq!(present, 2);
        assert!(evidence.iter().all(|item| !item.reach.present));
        assert_eq!(
            summarize_discharge(&evidence).summary,
            "Some inferred safety obligations are missing local guard evidence"
        );
    }

    #[test]
    fn reach_evidence_reports_named_test_mentions() -> Result<(), String> {
        let root = unique_temp_dir("reach_evidence")?;
        let tests_dir = root.join("tests");
        fs::create_dir_all(&tests_dir).map_err(|err| err.to_string())?;
        fs::write(
            tests_dir.join("reach.rs"),
            "#[test]\nfn reaches_read_word() { read_word(); }\n",
        )
        .map_err(|err| err.to_string())?;

        let owner = "read_word".to_string();
        let (reach, tests) = reach_evidence(&root, Some(&owner));
        fs::remove_dir_all(&root).map_err(|err| err.to_string())?;

        assert_eq!(reach.state, "owner_reached");
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "reaches_read_word");
        assert_eq!(tests[0].file, "tests/reach.rs");
        Ok(())
    }

    fn unique_temp_dir(name: &str) -> Result<PathBuf, String> {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| err.to_string())?
            .as_nanos();
        path.push(format!("unsafe-review-{name}-{nanos}"));
        fs::create_dir_all(&path).map_err(|err| err.to_string())?;
        Ok(path)
    }
}
