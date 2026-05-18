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

    fn scanned_site(before: &[&str], snippet: &str, after: &[&str]) -> ScannedSite {
        ScannedSite {
            site: UnsafeSite {
                location: SourceLocation::new(PathBuf::from("src/lib.rs"), 1, 1),
                kind: UnsafeSiteKind::Operation,
                owner: Some("checked_read".to_string()),
                visibility: "private".to_string(),
                public_api_surface: false,
                changed: true,
                snippet: snippet.to_string(),
            },
            operation: UnsafeOperation {
                family: OperationFamily::RawPointerRead,
                expression: snippet.to_string(),
            },
            context_before: before.iter().map(|line| (*line).to_string()).collect(),
            context_after: after.iter().map(|line| (*line).to_string()).collect(),
        }
    }

    fn obligation(key: &str) -> SafetyObligation {
        SafetyObligation::new(key, "test obligation")
    }

    #[test]
    fn guard_detection_ignores_comment_only_evidence() {
        let site = scanned_site(
            &["// len >= needed", "// ptr.is_null() was checked elsewhere"],
            "unsafe { ptr.read() }",
            &["// align_of::<u32>()"],
        );
        let reach = ReachEvidence {
            state: "static-mention".to_string(),
            summary: "related test mentions owner".to_string(),
        };
        let evidence = obligation_evidence(
            &site,
            &[
                obligation("bounds"),
                obligation("non-null"),
                obligation("alignment"),
            ],
            &ContractEvidence::present("documented"),
            &reach,
        );

        assert!(evidence.iter().all(|item| !item.discharge.present));
        assert!(evidence.iter().all(|item| item.contract.present));
        assert!(evidence.iter().all(|item| item.reach.present));
    }

    #[test]
    fn guard_detection_requires_obligation_specific_code_patterns() -> Result<(), String> {
        let site = scanned_site(
            &[
                "let enough = src.len() >= needed;",
                "let aligned = ptr.addr() % align == 0;",
            ],
            "unsafe { ptr.read() }",
            &["if ptr.is_null() { return None; }"],
        );
        let reach = ReachEvidence {
            state: "static-mention".to_string(),
            summary: "related test mentions owner".to_string(),
        };
        let evidence = obligation_evidence(
            &site,
            &[
                obligation("bounds"),
                obligation("alignment"),
                obligation("non-null"),
            ],
            &ContractEvidence::present("documented"),
            &reach,
        );

        for key in ["bounds", "alignment", "non-null"] {
            let Some(item) = evidence.iter().find(|item| item.obligation.key == key) else {
                return Err(format!("missing evidence for obligation {key}"));
            };
            assert!(item.discharge.present, "{key} should have guard evidence");
        }
        assert!(summarize_discharge(&evidence).present);
        Ok(())
    }

    #[test]
    fn summarize_discharge_distinguishes_partial_from_empty() {
        let missing = EvidenceState::missing("missing");
        let present = EvidenceState::present("present");
        let base = ObligationEvidence {
            obligation: obligation("bounds"),
            contract: present.clone(),
            discharge: missing.clone(),
            reach: present.clone(),
            witness: missing.clone(),
        };
        assert!(!summarize_discharge(&[]).present);
        assert_eq!(
            summarize_discharge(std::slice::from_ref(&base)),
            DischargeEvidence::missing()
        );

        let mut partial = base.clone();
        partial.discharge = present;
        let summary = summarize_discharge(&[base, partial]);
        assert!(!summary.present);
        assert!(summary.summary.contains("Some inferred"));
    }

    #[test]
    fn reach_evidence_uses_named_test_before_owner_mention() -> Result<(), String> {
        let root = unique_temp_dir("unsafe-review-reach")?;
        let tests_dir = root.join("tests");
        fs::create_dir_all(&tests_dir)
            .map_err(|err| format!("create {} failed: {err}", tests_dir.display()))?;
        fs::write(
            tests_dir.join("checked_read.rs"),
            "#[test]\nfn reaches_checked_read() {\n    checked_read();\n}\n",
        )
        .map_err(|err| format!("write test failed: {err}"))?;

        let owner = "checked_read".to_string();
        let (reach, related) = reach_evidence(&root, Some(&owner));
        fs::remove_dir_all(&root)
            .map_err(|err| format!("cleanup {} failed: {err}", root.display()))?;

        assert_eq!(reach.state, "owner_reached");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].name, "reaches_checked_read");
        assert_eq!(related[0].line, 2);
        assert_eq!(related[0].file, "tests/checked_read.rs");
        Ok(())
    }

    #[test]
    fn reach_evidence_reports_unknown_without_owner() {
        let (reach, related) = reach_evidence(Path::new("."), None);

        assert_eq!(reach.state, "unknown");
        assert!(related.is_empty());
    }

    fn unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock went backwards: {err}"))?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path)
            .map_err(|err| format!("create {} failed: {err}", path.display()))?;
        Ok(path)
    }
}
