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

    fn site_with_context(before: &[&str], snippet: &str, after: &[&str]) -> ScannedSite {
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

    #[test]
    fn contract_evidence_accepts_safety_docs_and_inline_comments() {
        let documented = site_with_context(
            &["/// # Safety", "/// caller validates ptr"],
            "unsafe { *ptr }",
            &[],
        );
        let commented = site_with_context(&[], "unsafe { *ptr } // SAFETY: ptr checked above", &[]);
        let missing = site_with_context(&["// ordinary comment"], "unsafe { *ptr }", &[]);

        assert!(contract_evidence(&documented).present);
        assert!(contract_evidence(&commented).present);
        assert!(!contract_evidence(&missing).present);
    }

    #[test]
    fn guard_detection_ignores_comment_only_alignment_claims() {
        let site = site_with_context(
            &["// ptr.is_aligned() would be nice", "if len >= 1 {"],
            "core::ptr::read(ptr)",
            &["}"],
        );
        let obligations = vec![
            SafetyObligation::new("alignment", "pointer is aligned"),
            SafetyObligation::new("bounds", "pointer has enough bytes"),
        ];
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "test reaches owner".to_string(),
        };
        let evidence = obligation_evidence(
            &site,
            &obligations,
            &ContractEvidence::present("contract"),
            &reach,
        );

        assert_eq!(evidence.len(), 2);
        assert!(!evidence[0].discharge.present);
        assert!(evidence[1].discharge.present);
    }

    #[test]
    fn summarize_discharge_requires_every_obligation_to_be_present() {
        let obligation = SafetyObligation::new("bounds", "bounds checked");
        let present = ObligationEvidence {
            obligation: obligation.clone(),
            contract: EvidenceState::present("contract"),
            discharge: EvidenceState::present("guard"),
            reach: EvidenceState::present("reach"),
            witness: EvidenceState::missing("witness"),
        };
        let missing = ObligationEvidence {
            obligation,
            contract: EvidenceState::present("contract"),
            discharge: EvidenceState::missing("guard"),
            reach: EvidenceState::present("reach"),
            witness: EvidenceState::missing("witness"),
        };

        assert!(!summarize_discharge(&[]).present);
        assert!(summarize_discharge(std::slice::from_ref(&present)).present);
        assert!(!summarize_discharge(&[present, missing]).present);
    }

    #[test]
    fn discharge_detection_covers_guard_variants() {
        for guard in [
            "ptr.is_aligned()",
            "ptr.align_offset(align) == 0",
            "core::mem::align_of::<u32>()",
            "ptr.addr() % 4 == 0",
            "ptr as usize % 4 == 0",
        ] {
            assert!(discharge_state_for("alignment", guard).present);
        }

        assert!(discharge_state_for("bounds", "if index < slice.len() {}").present);
        assert!(discharge_state_for("valid-range", "if slice.len() >= needed {}").present);
        assert!(!discharge_state_for("bounds", "let len = slice.len();").present);
        assert!(!discharge_state_for("bounds", "if index < limit {}").present);

        assert!(discharge_state_for("capacity", "if vec.capacity() >= needed {}").present);
        assert!(discharge_state_for("capacity", "if vec.cap() >= needed {}").present);
        assert!(discharge_state_for("non-null", "if !ptr.is_null() {}").present);
        assert!(discharge_state_for("pointer-live", "let ptr = non_null::new(ptr);").present);
        assert!(discharge_state_for("pointer-live", "let ptr: nonnull<u8> = ptr;").present);
        assert!(!discharge_state_for("unknown", "if ptr.is_null() {}").present);
    }

    #[test]
    fn reach_evidence_reports_related_tests_with_precise_lines() -> Result<(), String> {
        let root = TempRoot::create()?;
        fs::create_dir_all(root.path.join("tests"))
            .map_err(|err| format!("create tests dir failed: {err}"))?;
        fs::write(
            root.path.join("tests/reach.rs"),
            "#[test]\nfn integration_reaches_owner() {\n    checked_read();\n}\n",
        )
        .map_err(|err| format!("write reach.rs failed: {err}"))?;
        fs::write(
            root.path.join("tests/mention.rs"),
            "// helper\n// mention only\nchecked_read();\n",
        )
        .map_err(|err| format!("write mention.rs failed: {err}"))?;
        fs::write(
            root.path.join("tests/attribute_only.rs"),
            "#[test]\nchecked_read();\n",
        )
        .map_err(|err| format!("write attribute_only.rs failed: {err}"))?;

        let owner = "checked_read".to_string();
        let (reach, tests) = reach_evidence(&root.path, Some(&owner));

        assert_eq!(reach.state, "owner_reached");
        assert_eq!(tests.len(), 3);
        assert!(tests.iter().any(|test| {
            test.file == "tests/reach.rs"
                && test.name == "integration_reaches_owner"
                && test.line == 2
        }));
        assert!(tests.iter().any(|test| {
            test.file == "tests/mention.rs"
                && test.name == "mentions checked_read"
                && test.line == 3
        }));
        assert!(tests.iter().any(|test| {
            test.file == "tests/attribute_only.rs" && test.name == "test" && test.line == 1
        }));
        Ok(())
    }

    #[test]
    fn collect_test_files_includes_inline_unit_tests_outside_test_paths() -> Result<(), String> {
        let root = TempRoot::create()?;
        fs::create_dir_all(root.path.join("src"))
            .map_err(|err| format!("create src dir failed: {err}"))?;
        fs::write(
            root.path.join("src/lib.rs"),
            "pub fn helper() {}\n#[test]\nfn helper_is_reachable() {}\n",
        )
        .map_err(|err| format!("write lib.rs failed: {err}"))?;
        fs::create_dir_all(root.path.join("testdata"))
            .map_err(|err| format!("create testdata dir failed: {err}"))?;
        fs::write(
            root.path.join("testdata/helper.rs"),
            "pub fn fixture_helper() {}\n",
        )
        .map_err(|err| format!("write helper.rs failed: {err}"))?;

        let files = collect_test_files(&root.path)?;
        assert_eq!(
            files,
            vec![
                PathBuf::from("src/lib.rs"),
                PathBuf::from("testdata/helper.rs")
            ]
        );
        Ok(())
    }

    #[test]
    fn reach_evidence_handles_unknown_and_unmentioned_owners() -> Result<(), String> {
        let root = TempRoot::create()?;
        fs::create_dir_all(root.path.join("tests"))
            .map_err(|err| format!("create tests dir failed: {err}"))?;
        fs::write(
            root.path.join("tests/other.rs"),
            "#[test]\nfn unrelated() {\n    assert!(true);\n}\n",
        )
        .map_err(|err| format!("write other.rs failed: {err}"))?;

        let (unknown, unknown_tests) = reach_evidence(&root.path, None);
        assert_eq!(unknown.state, "unknown");
        assert!(unknown_tests.is_empty());

        let owner = "checked_read".to_string();
        let (unreached, unreached_tests) = reach_evidence(&root.path, Some(&owner));
        assert_eq!(unreached.state, "unreached");
        assert!(unreached_tests.is_empty());
        Ok(())
    }

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn create() -> Result<Self, String> {
            let mut path = std::env::temp_dir();
            path.push(format!(
                "unsafe-review-evidence-test-{}-{}",
                std::process::id(),
                unique_suffix()
            ));
            fs::create_dir_all(&path)
                .map_err(|err| format!("create temp root {} failed: {err}", path.display()))?;
            Ok(Self { path })
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn unique_suffix() -> u128 {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos(),
            Err(_err) => 0,
        }
    }

    #[test]
    fn parse_test_name_accepts_public_and_private_test_fns_only() {
        assert_eq!(
            parse_test_name("fn reaches_owner() {"),
            Some("reaches_owner".to_string())
        );
        assert_eq!(
            parse_test_name("pub fn reaches_owner() {"),
            Some("reaches_owner".to_string())
        );
        assert_eq!(parse_test_name("async fn reaches_owner() {"), None);
    }
}
