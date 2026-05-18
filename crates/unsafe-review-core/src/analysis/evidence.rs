use crate::analysis::scanner::ScannedSite;
use crate::domain::{
    ContractEvidence, DischargeEvidence, EvidenceState, ObligationEvidence, OperationFamily,
    ReachEvidence, RelatedTest, SafetyObligation, UnsafeSiteKind,
};
use std::fs;
use std::path::{Path, PathBuf};

const PUBLIC_UNSAFE_API_CONTRACT_DISCHARGE: &str = "Public unsafe API declaration is a caller-contract site; local guard evidence is not expected at the declaration";
const DOCUMENTED_PRIVATE_UNSAFE_CONTRACT_DISCHARGE: &str = "Documented private unsafe declaration is a caller-contract site; local guard evidence is not expected at the declaration";

pub(crate) fn contract_evidence(site: &ScannedSite) -> ContractEvidence {
    let context = site.context_before.join("\n");
    if let Some(summary) = safety_doc_summary(&context) {
        return ContractEvidence::present(summary);
    }
    if site.site.public_api_surface {
        return ContractEvidence::missing_with(
            "Public unsafe API is missing nearby `# Safety` documentation",
        );
    }
    if context.contains("SAFETY:") || site.site.snippet.contains("SAFETY:") {
        return ContractEvidence::present("Nearby `SAFETY:` comment was detected");
    }
    ContractEvidence::missing()
}

fn safety_doc_summary(context: &str) -> Option<&'static str> {
    for line in context.lines() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("///")
            || trimmed.starts_with("//!")
            || trimmed.starts_with("#[doc"))
        {
            continue;
        }
        if trimmed.contains("# Safety") {
            return Some("Nearby `# Safety` documentation was detected");
        }
        if trimmed.contains("Safety:") {
            return Some("Nearby `Safety:` documentation was detected");
        }
    }
    None
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
            discharge: discharge_state_for(site, &obligation.key, &lower, contract),
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
        if evidence.iter().all(|obligation| {
            obligation.discharge.summary == PUBLIC_UNSAFE_API_CONTRACT_DISCHARGE
                || obligation.discharge.summary == DOCUMENTED_PRIVATE_UNSAFE_CONTRACT_DISCHARGE
        }) {
            return DischargeEvidence::present(&evidence[0].discharge.summary);
        }
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

fn discharge_state_for(
    site: &ScannedSite,
    key: &str,
    lower: &str,
    contract: &ContractEvidence,
) -> EvidenceState {
    let family = &site.operation.family;
    if is_public_unsafe_contract_obligation(site, key) {
        return EvidenceState::present(PUBLIC_UNSAFE_API_CONTRACT_DISCHARGE);
    }
    if is_documented_private_unsafe_contract_obligation(site, key, contract) {
        return EvidenceState::present(DOCUMENTED_PRIVATE_UNSAFE_CONTRACT_DISCHARGE);
    }
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
            if has_capacity_guard(family, lower) {
                EvidenceState::present("Capacity guard code was detected")
            } else {
                EvidenceState::missing("No capacity guard code was detected")
            }
        }
        "initialized" => {
            if family == &OperationFamily::VecSetLen && has_set_len_initialization_evidence(lower) {
                EvidenceState::present("Initialization evidence was detected")
            } else if family == &OperationFamily::SliceFromRawParts
                && has_maybeuninit_slice_context(lower)
            {
                EvidenceState::present("MaybeUninit slice element evidence was detected")
            } else if family == &OperationFamily::VecSetLen {
                EvidenceState::missing("No initialization evidence was detected")
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
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
        "callee-contract" => {
            if family == &OperationFamily::UnsafeFnCall
                && has_encode_utf8_remaining_capacity_evidence(lower)
            {
                EvidenceState::present("Unsafe call argument guard code was detected")
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
            }
        }
        _ => EvidenceState::missing("No obligation-specific guard code was detected"),
    }
}

fn is_public_unsafe_contract_obligation(site: &ScannedSite, key: &str) -> bool {
    key == "unknown"
        && site.site.public_api_surface
        && site.operation.family == OperationFamily::Unknown
        && matches!(
            site.site.kind,
            UnsafeSiteKind::UnsafeFn | UnsafeSiteKind::UnsafeTrait
        )
}

fn is_documented_private_unsafe_contract_obligation(
    site: &ScannedSite,
    key: &str,
    contract: &ContractEvidence,
) -> bool {
    key == "unknown"
        && !site.site.public_api_surface
        && contract.present
        && contract.summary.contains("documentation")
        && site.operation.family == OperationFamily::Unknown
        && matches!(
            site.site.kind,
            UnsafeSiteKind::UnsafeFn | UnsafeSiteKind::UnsafeTrait
        )
}

fn has_length_or_bounds_guard(lower: &str) -> bool {
    let has_comparison = lower.contains(">=") || lower.contains('<');
    (has_comparison && (lower.contains("len") || lower.contains("num_ctrl_bytes")))
        || has_len_capacity_equality_guard(lower)
}

fn has_len_capacity_equality_guard(lower: &str) -> bool {
    let compact = compact_code(lower);
    let has_equality = compact.contains("==")
        || compact.contains("assert_eq!(")
        || compact.contains("debug_assert_eq!(");
    has_equality
        && compact.contains("len")
        && (compact.contains("capacity") || contains_word(&compact, "cap"))
}

fn has_capacity_guard(family: &OperationFamily, lower: &str) -> bool {
    lower.contains("capacity")
        || lower.contains("cap()")
        || (family == &OperationFamily::VecSetLen && contains_word(lower, "cap"))
        || (family == &OperationFamily::VecSetLen && has_set_len_shrink_evidence(lower))
}

fn has_set_len_initialization_evidence(lower: &str) -> bool {
    has_set_len_shrink_evidence(lower)
        || has_set_len_call_result_initialization_evidence(lower)
        || lower.contains("maybeuninit::new")
        || lower.contains(".write(")
        || lower.contains("ptr::write")
        || lower.contains("copy_nonoverlapping")
        || lower.contains("copy_to_nonoverlapping")
}

fn has_set_len_call_result_initialization_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    compact.contains("encode_utf8(")
        && (compact.contains(".set_len(len+n)") || compact.contains(".set_len(old_len+n)"))
}

fn has_encode_utf8_remaining_capacity_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    compact.contains("encode_utf8(c,ptr,remaining_cap)")
        && compact.contains("remaining_cap=self.capacity()-len")
        && compact.contains("ptr")
}

fn has_maybeuninit_slice_context(lower: &str) -> bool {
    let compact = compact_code(lower);
    compact.contains("from_raw_parts_mut(") && compact.contains("maybeuninit")
}

fn has_set_len_shrink_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    if compact.contains(".set_len(0)") {
        return true;
    }
    if compact.contains(".set_len(last_index)")
        && (compact.contains("last_index=self.len-1")
            || compact.contains("last_index=self.len()-1")
            || (compact.contains("last_index=")
                && (compact.contains(".len-1") || compact.contains(".len()-1"))))
        && (compact.contains("self.len==0")
            || compact.contains("self.len()==0")
            || compact.contains(".len==0")
            || compact.contains(".len()==0")
            || compact.contains("self.len>0")
            || compact.contains("self.len()>0")
            || compact.contains("!self.is_empty()"))
    {
        return true;
    }
    if compact.contains(".set_len(start)")
        && (compact.contains("start<=len")
            || (compact.contains("start<=end") && compact.contains("end<=len")))
        && (compact.contains("len=self.len()")
            || (compact.contains("letlen=") && compact.contains(".len()")))
    {
        return true;
    }
    if !compact.contains(".set_len(new_len)") {
        return false;
    }
    ((compact.contains("new_len<=") || compact.contains("new_len<")) && compact.contains(".len()"))
        || (compact.contains("new_len=") && compact.contains(".len()-"))
        || (compact.contains("len=self.len()") && compact.contains("new_len=len-"))
}

fn compact_code(lower: &str) -> String {
    lower
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect()
}

fn has_alignment_guard(lower: &str) -> bool {
    lower.contains("is_aligned")
        || lower.contains("align_offset")
        || lower.contains("align_of")
        || lower.contains("addr() %")
        || lower.contains("as usize %")
}

fn contains_word(text: &str, word: &str) -> bool {
    text.split(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
        .any(|token| token == word)
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

    fn site_with_context(
        context_before: Vec<&str>,
        snippet: &str,
        context_after: Vec<&str>,
    ) -> ScannedSite {
        site_with_family(
            OperationFamily::RawPointerRead,
            context_before,
            snippet,
            context_after,
        )
    }

    fn site_with_family(
        family: OperationFamily,
        context_before: Vec<&str>,
        snippet: &str,
        context_after: Vec<&str>,
    ) -> ScannedSite {
        ScannedSite {
            site: UnsafeSite {
                location: SourceLocation::new(PathBuf::from("src/lib.rs"), 1, 1),
                kind: UnsafeSiteKind::Operation,
                owner: Some("read_one".to_string()),
                visibility: "private".to_string(),
                public_api_surface: false,
                changed: true,
                snippet: snippet.to_string(),
            },
            operation: UnsafeOperation {
                family,
                expression: snippet.to_string(),
            },
            context_before: context_before.into_iter().map(str::to_string).collect(),
            context_after: context_after.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn contract_evidence_accepts_safety_docs_and_safety_comments() {
        let doc_site = site_with_context(
            vec!["/// # Safety", "/// pointer must be valid"],
            "ptr.read()",
            vec![],
        );
        let safety_colon_doc_site = site_with_context(
            vec!["/// Safety: pointer must be valid"],
            "ptr.read()",
            vec![],
        );
        let comment_site = site_with_context(
            vec!["// SAFETY: caller checked pointer"],
            "ptr.read()",
            vec![],
        );
        let missing_site = site_with_context(vec!["// ordinary comment"], "ptr.read()", vec![]);

        assert!(contract_evidence(&doc_site).present);
        assert!(contract_evidence(&safety_colon_doc_site).present);
        assert!(contract_evidence(&comment_site).present);
        assert!(!contract_evidence(&missing_site).present);
    }

    #[test]
    fn obligation_evidence_ignores_guards_that_only_appear_in_comments() {
        let obligations = vec![SafetyObligation::new(
            "alignment",
            "pointer is aligned for the accessed type",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let misleading_comment = site_with_context(
            vec!["// SAFETY: checked elsewhere"],
            "ptr.read() // align_of::<u32>() proves this",
            vec![],
        );
        let local_guard = site_with_context(
            vec!["if (ptr as usize) % std::mem::align_of::<u32>() != 0 { return None; }"],
            "ptr.read()",
            vec![],
        );

        assert!(
            !obligation_evidence(&misleading_comment, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&local_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
    }

    #[test]
    fn set_len_initialization_evidence_is_operation_specific() {
        let obligations = vec![SafetyObligation::new(
            "initialized",
            "elements in the extended range are initialized",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let context = vec!["*dst = MaybeUninit::new(*src);"];
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            context.clone(),
            "out.set_len(CAP);",
            vec![],
        );
        let raw_read = site_with_family(
            OperationFamily::RawPointerRead,
            context,
            "ptr.read()",
            vec![],
        );

        let set_len_evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);
        let raw_read_evidence = obligation_evidence(&raw_read, &obligations, &contract, &reach);

        assert!(set_len_evidence[0].discharge.present);
        assert_eq!(
            raw_read_evidence[0].discharge.summary,
            "No obligation-specific guard code was detected"
        );
    }

    #[test]
    fn set_len_capacity_evidence_accepts_const_cap_token() {
        let obligations = vec![SafetyObligation::new(
            "capacity",
            "new length is at most capacity",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec!["pub struct Buffer<const CAP: usize> {"],
            "out.set_len(CAP);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
    }

    #[test]
    fn set_len_shrink_discharges_capacity_and_initialized_obligations() {
        let obligations = vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec!["if new_len <= values.len() {"],
            "values.set_len(new_len);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence.iter().all(|item| item.discharge.present));
    }

    #[test]
    fn set_len_zero_discharges_capacity_and_initialized_obligations() {
        let obligations = vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec![],
            "values.set_len(0);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence.iter().all(|item| item.discharge.present));
    }

    #[test]
    fn set_len_call_result_discharges_initialized_obligation() {
        let obligations = vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec![
                "let remaining_cap = self.capacity() - len;",
                "let n = encode_utf8(c, ptr, remaining_cap)?;",
            ],
            "self.set_len(len + n);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence.iter().all(|item| item.discharge.present));
    }

    #[test]
    fn set_len_last_index_shrink_discharges_capacity_and_initialized_obligations() {
        let obligations = vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec![
                "if self.len == 0 {",
                "    return None;",
                "}",
                "let last_index = self.len - 1;",
            ],
            "self.set_len(last_index);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence.iter().all(|item| item.discharge.present));
    }

    #[test]
    fn set_len_last_index_shrink_accepts_len_method_receiver() {
        let obligations = vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec![
                "if values.len() == 0 {",
                "    return None;",
                "}",
                "let last_index = values.len() - 1;",
            ],
            "values.set_len(last_index);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence.iter().all(|item| item.discharge.present));
    }

    #[test]
    fn set_len_last_index_shrink_requires_non_empty_guard() {
        let obligations = vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec!["let last_index = values.len() - 1;"],
            "values.set_len(last_index);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence.iter().all(|item| !item.discharge.present));
    }

    #[test]
    fn set_len_start_bound_shrink_discharges_capacity_and_initialized_obligations() {
        let obligations = vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec![
                "let len = values.len();",
                "assert!(start <= end);",
                "assert!(end <= len);",
            ],
            "values.set_len(start);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence.iter().all(|item| item.discharge.present));
    }

    #[test]
    fn set_len_start_bound_shrink_requires_upper_bound() {
        let obligations = vec![
            SafetyObligation::new("capacity", "new length is at most capacity"),
            SafetyObligation::new(
                "initialized",
                "elements in the extended range are initialized",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let set_len = site_with_family(
            OperationFamily::VecSetLen,
            vec!["let len = values.len();", "assert!(start <= end);"],
            "values.set_len(start);",
            vec![],
        );

        let evidence = obligation_evidence(&set_len, &obligations, &contract, &reach);

        assert!(evidence.iter().all(|item| !item.discharge.present));
    }

    #[test]
    fn len_capacity_equality_discharges_bounds_obligation() {
        let obligations = vec![SafetyObligation::new(
            "bounds",
            "buffer has enough bytes for the accessed type",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let raw_read = site_with_family(
            OperationFamily::RawPointerRead,
            vec!["debug_assert_eq!(self.len(), self.capacity());"],
            "ptr::read(self.as_ptr() as *const [T; CAP])",
            vec![],
        );

        let evidence = obligation_evidence(&raw_read, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
    }

    #[test]
    fn encode_utf8_remaining_capacity_discharges_unsafe_call_obligation() {
        let obligations = vec![SafetyObligation::new(
            "callee-contract",
            "callee safety preconditions are satisfied",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let unsafe_call = site_with_family(
            OperationFamily::UnsafeFnCall,
            vec![
                "let ptr = self.xs[len..].as_mut_ptr() as *mut u8;",
                "let remaining_cap = self.capacity() - len;",
                "// SAFETY: `ptr` points to `remaining_cap` bytes.",
            ],
            "match unsafe { encode_utf8(c, ptr, remaining_cap) } {",
            vec![],
        );

        let evidence = obligation_evidence(&unsafe_call, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
    }

    #[test]
    fn maybeuninit_slice_discharges_initialized_obligation_only() {
        let obligations = vec![
            SafetyObligation::new("initialized", "memory range is initialized"),
            SafetyObligation::new("alignment", "pointer is aligned for the element type"),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let slice = site_with_family(
            OperationFamily::SliceFromRawParts,
            vec!["fn ctrl_slice(&mut self) -> &mut [core::mem::MaybeUninit<Tag>] {"],
            "unsafe { core::slice::from_raw_parts_mut(self.ctrl.as_ptr().cast(), self.len) }",
            vec![],
        );

        let evidence = obligation_evidence(&slice, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
        assert!(!evidence[1].discharge.present);
    }

    #[test]
    fn summarize_discharge_distinguishes_none_some_and_all_present() {
        let missing = EvidenceState::missing("missing");
        let present = EvidenceState::present("present");
        let base = |key: &str, discharge: EvidenceState| ObligationEvidence {
            obligation: SafetyObligation::new(key, "obligation"),
            contract: EvidenceState::present("contract"),
            discharge,
            reach: EvidenceState::present("reach"),
            witness: EvidenceState::missing("witness"),
        };

        assert!(!summarize_discharge(&[]).present);
        assert!(!summarize_discharge(&[base("bounds", missing.clone())]).present);
        assert!(summarize_discharge(&[base("bounds", present.clone())]).present);
        let partial = summarize_discharge(&[base("bounds", present), base("alignment", missing)]);
        assert!(!partial.present);
        assert!(partial.summary.contains("Some inferred"));
    }

    #[test]
    fn reach_evidence_finds_unit_and_integration_tests_by_owner_name() -> Result<(), String> {
        let root = unique_temp_dir()?;
        fs::create_dir_all(root.join("src")).map_err(|err| err.to_string())?;
        fs::create_dir_all(root.join("tests")).map_err(|err| err.to_string())?;
        fs::write(
            root.join("src/reach_test.rs"),
            r#"
#[cfg(test)]
mod tests {
    #[test]
    fn reaches_read_one_in_unit_test() {
        read_one();
    }
}
"#,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            root.join("tests/reach.rs"),
            r#"
#[test]
fn reaches_read_one_in_integration_test() {
    unsafe_review_fixture::read_one();
}
"#,
        )
        .map_err(|err| err.to_string())?;
        let owner = "read_one".to_string();

        let (reach, related_tests) = reach_evidence(&root, Some(&owner));

        fs::remove_dir_all(&root).map_err(|err| err.to_string())?;
        assert_eq!(reach.state, "owner_reached");
        assert_eq!(related_tests.len(), 2);
        assert!(
            related_tests
                .iter()
                .any(|test| test.name == "reaches_read_one_in_unit_test")
        );
        assert!(
            related_tests
                .iter()
                .any(|test| test.file == "tests/reach.rs"
                    && test.name == "reaches_read_one_in_integration_test")
        );
        Ok(())
    }

    #[test]
    fn reach_evidence_reports_unknown_when_owner_is_missing() {
        let (reach, related_tests) = reach_evidence(Path::new("."), None);

        assert_eq!(reach.state, "unknown");
        assert!(related_tests.is_empty());
    }

    fn unique_temp_dir() -> Result<PathBuf, String> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| err.to_string())?
            .as_nanos();
        let root = std::env::temp_dir().join(format!("unsafe-review-evidence-test-{nanos}"));
        fs::create_dir_all(&root).map_err(|err| err.to_string())?;
        Ok(root)
    }
}
