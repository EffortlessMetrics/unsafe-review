use crate::analysis::scanner::ScannedSite;
use crate::domain::{
    ContractEvidence, DischargeEvidence, EvidenceState, ObligationEvidence, OperationFamily,
    ReachEvidence, RelatedTest, SafetyObligation, UnsafeSiteKind,
};
use std::fs;
use std::path::{Path, PathBuf};

const PUBLIC_UNSAFE_API_CONTRACT_DISCHARGE: &str = "Public unsafe API declaration is a caller-contract site; local guard evidence is not expected at the declaration";
const DOCUMENTED_PRIVATE_UNSAFE_CONTRACT_DISCHARGE: &str = "Documented private unsafe declaration is a caller-contract site; local guard evidence is not expected at the declaration";
const TARGET_FEATURE_CONTRACT_DISCHARGE: &str = "Documented target-feature declaration is a caller-contract site; local guard evidence is not expected at the attribute";

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
    if let Some(summary) = safety_comment_summary(&context, &site.site.snippet) {
        return ContractEvidence::present(summary);
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

fn safety_comment_summary(context: &str, snippet: &str) -> Option<&'static str> {
    for line in context.lines().chain(snippet.lines()) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            continue;
        }
        if !(trimmed.starts_with("//")
            || trimmed.contains("// SAFETY:")
            || trimmed.contains("// Safety:"))
        {
            continue;
        }
        if trimmed.contains("SAFETY:") {
            return Some("Nearby `SAFETY:` comment was detected");
        }
        if trimmed.contains("Safety:") {
            return Some("Nearby `Safety:` comment was detected");
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
            if family == &OperationFamily::RawPointerWrite && has_u8_write_bytes_context(lower) {
                EvidenceState::present("u8 raw write alignment evidence was detected")
            } else if has_alignment_guard(site, lower) {
                EvidenceState::present("Alignment guard code was detected")
            } else {
                EvidenceState::missing("No alignment guard code was detected")
            }
        }
        "bounds" | "valid-range" => {
            if has_bounds_guard(site, lower)
                || (family == &OperationFamily::PointerArithmetic
                    && has_slice_end_pointer_arithmetic_evidence(lower))
            {
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
            } else if family == &OperationFamily::RawPointerWrite
                && has_maybeuninit_raw_write_context(lower)
            {
                EvidenceState::present("MaybeUninit raw write target evidence was detected")
            } else if family == &OperationFamily::RawPointerWrite
                && has_u8_write_bytes_context(lower)
            {
                EvidenceState::present("u8 write_bytes target evidence was detected")
            } else if family == &OperationFamily::VecSetLen {
                EvidenceState::missing("No initialization evidence was detected")
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
            }
        }
        "non-null" | "pointer-live" => {
            if has_nullability_guard(site, lower) {
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
            } else if family == &OperationFamily::UnsafeFnCall
                && has_unchecked_constructor_availability_evidence(
                    &site.operation.expression,
                    lower,
                )
            {
                EvidenceState::present("Unchecked constructor availability guard code was detected")
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
            }
        }
        "valid-value" => {
            if family == &OperationFamily::UnwrapUnchecked
                && has_unwrap_unchecked_infallible_result_evidence(lower)
            {
                EvidenceState::present(
                    "Infallible Result state evidence was detected before unwrap_unchecked",
                )
            } else if family == &OperationFamily::UnwrapUnchecked
                && has_unwrap_unchecked_receiver_state_evidence(lower)
            {
                EvidenceState::present(
                    "Same-receiver Option/Result state evidence was detected before unwrap_unchecked",
                )
            } else if family == &OperationFamily::Transmute
                && has_transmute_u8_bool_valid_value_evidence(lower)
            {
                EvidenceState::present("Transmute u8-to-bool valid-value evidence was detected")
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
            }
        }
        "layout" => {
            if family == &OperationFamily::Transmute && has_transmute_layout_size_evidence(lower) {
                EvidenceState::present("Transmute layout size evidence was detected")
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
            }
        }
        "unreachable" => {
            if family == &OperationFamily::UnreachableUnchecked
                && has_unreachable_unchecked_infallible_path_evidence(lower)
            {
                EvidenceState::present(
                    "Infallible error-path evidence was detected before unreachable_unchecked",
                )
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
            }
        }
        "target-feature" => {
            if family == &OperationFamily::TargetFeature && contract.present {
                EvidenceState::present(TARGET_FEATURE_CONTRACT_DISCHARGE)
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
            }
        }
        "utf8" => {
            if family == &OperationFamily::StrFromUtf8Unchecked
                && has_from_utf8_unchecked_validation_evidence(lower)
            {
                EvidenceState::present(
                    "Same-buffer UTF-8 validation evidence was detected before from_utf8_unchecked",
                )
            } else {
                EvidenceState::missing("No obligation-specific guard code was detected")
            }
        }
        "valid-zero" => {
            if family == &OperationFamily::Zeroed && has_zeroed_known_valid_zero_type(lower) {
                EvidenceState::present(
                    "Known valid-zero target type evidence was detected before zeroed",
                )
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

fn has_bounds_guard(site: &ScannedSite, lower: &str) -> bool {
    if site.operation.family == OperationFamily::GetUnchecked
        && let Some((receiver, index)) =
            get_unchecked_receiver_and_index(&site.operation.expression)
    {
        return has_get_unchecked_bounds_guard(lower, &receiver, &index);
    }
    has_length_or_bounds_guard(lower)
}

fn get_unchecked_receiver_and_index(expression: &str) -> Option<(String, String)> {
    let compact = compact_code(&expression.to_ascii_lowercase());
    for marker in [".get_unchecked_mut(", ".get_unchecked("] {
        let Some(receiver) = receiver_before_marker(&compact, marker) else {
            continue;
        };
        let call_pos = compact.find(marker)? + marker.len();
        let argument_text = &compact[call_pos..];
        let argument_end = matching_call_argument_end(argument_text)?;
        let index = &argument_text[..argument_end];
        if !receiver.is_empty() && !index.is_empty() {
            return Some((receiver.to_string(), index.to_string()));
        }
    }
    None
}

fn has_get_unchecked_bounds_guard(lower: &str, receiver: &str, index: &str) -> bool {
    let compact = compact_code(lower);
    let receiver = compact_code(&receiver.to_ascii_lowercase());
    let index = compact_code(&index.to_ascii_lowercase());
    if receiver.is_empty() || index.is_empty() {
        return false;
    }
    let len = format!("{receiver}.len()");
    has_get_unchecked_bounds_predicate(&compact, &format!("{index}<{len}"))
        || has_get_unchecked_bounds_predicate(&compact, &format!("{len}>{index}"))
        || has_get_unchecked_bounds_early_return(&compact, &format!("{index}>={len}"))
        || has_get_unchecked_bounds_early_return(&compact, &format!("{len}<={index}"))
}

fn has_get_unchecked_bounds_predicate(compact: &str, predicate: &str) -> bool {
    contains_receiver_fragment(compact, predicate)
        || compact.contains(&format!("if{predicate}"))
        || compact.contains(&format!("assert!({predicate}"))
        || compact.contains(&format!("debug_assert!({predicate}"))
}

fn has_get_unchecked_bounds_early_return(compact: &str, predicate: &str) -> bool {
    let guard = format!("if{predicate}{{");
    let Some((_prefix, after_guard)) = compact.split_once(&guard) else {
        return false;
    };
    after_guard
        .split_once('}')
        .map_or(after_guard, |(guard_body, _after)| guard_body)
        .contains("return")
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

fn has_slice_end_pointer_arithmetic_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    for line in lower.lines() {
        let line = compact_code(line);
        let Some(after_let) = line.strip_prefix("let") else {
            continue;
        };
        let Some((binding, expr)) = after_let.split_once('=') else {
            continue;
        };
        let Some(slice_expr) = expr.strip_suffix(".as_ptr();") else {
            continue;
        };
        if !binding.is_empty()
            && !slice_expr.is_empty()
            && compact.contains(&format!("{binding}.add({slice_expr}.len())"))
        {
            return true;
        }
    }
    false
}

fn has_capacity_guard(family: &OperationFamily, lower: &str) -> bool {
    if family == &OperationFamily::VecSetLen {
        return has_set_len_capacity_evidence(lower);
    }
    lower.contains("capacity") || lower.contains("cap()")
}

fn has_set_len_capacity_evidence(lower: &str) -> bool {
    has_set_len_shrink_evidence(lower)
        || has_set_len_call_result_initialization_evidence(lower)
        || has_set_len_const_cap_evidence(lower)
        || has_capacity_bound_guard(lower)
}

fn has_capacity_bound_guard(lower: &str) -> bool {
    let compact = compact_code(lower);
    let mentions_capacity = compact.contains("capacity()")
        || compact.contains(".cap()")
        || contains_word(lower, "cap")
        || contains_word(lower, "capacity");
    let has_guard_context = compact.contains("assert!(")
        || compact.contains("debug_assert!(")
        || compact.contains("if");
    let has_comparison = compact.contains("<=")
        || compact.contains(">=")
        || compact.contains('<')
        || compact.contains('>');
    mentions_capacity && has_guard_context && has_comparison
}

fn has_set_len_const_cap_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    compact.contains(".set_len(cap)")
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

fn has_unchecked_constructor_availability_evidence(expression: &str, lower: &str) -> bool {
    let compact_expression = compact_code(&expression.to_ascii_lowercase());
    let Some(receiver) = unchecked_constructor_receiver(&compact_expression) else {
        return false;
    };
    let compact = compact_code(lower);
    let before_call = compact
        .find(&compact_expression)
        .map_or(compact.as_str(), |call_pos| &compact[..call_pos]);
    before_call.contains(&format!("{receiver}::is_available()"))
}

fn unchecked_constructor_receiver(compact_expression: &str) -> Option<&str> {
    let call_pos = compact_expression.find("::new_unchecked")?;
    let before_call = &compact_expression[..call_pos];
    let receiver_start = before_call
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!is_receiver_path_char(ch)).then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    let receiver = &before_call[receiver_start..];
    (!receiver.is_empty()).then_some(receiver)
}

fn has_unwrap_unchecked_infallible_result_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    let Some((before_call, receiver)) = unwrap_unchecked_receiver_context(&compact) else {
        return false;
    };
    has_infallible_assignment_to_receiver(before_call, receiver)
}

fn has_infallible_assignment_to_receiver(before_call: &str, receiver: &str) -> bool {
    let let_assignment = format!("let{receiver}=");
    let assignment = format!("{receiver}=");
    before_call.split(';').any(|statement| {
        statement.contains("fallibility::infallible")
            && (contains_receiver_fragment(statement, &let_assignment)
                || contains_receiver_fragment(statement, &assignment))
    })
}

fn has_unwrap_unchecked_receiver_state_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    let Some((before_call, receiver)) = unwrap_unchecked_receiver_context(&compact) else {
        return false;
    };

    before_call.contains(&format!("{receiver}.is_some()"))
        || before_call.contains(&format!("{receiver}.is_ok()"))
        || has_receiver_early_return_guard(before_call, receiver, "is_none")
        || has_receiver_early_return_guard(before_call, receiver, "is_err")
}

fn unwrap_unchecked_receiver_context(compact: &str) -> Option<(&str, &str)> {
    let call_pos = compact.find(".unwrap_unchecked(")?;
    let before_call = &compact[..call_pos];
    let receiver_start = before_call
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!is_receiver_path_char(ch)).then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    let receiver = &before_call[receiver_start..];
    (!receiver.is_empty()).then_some((before_call, receiver))
}

fn has_receiver_early_return_guard(before_call: &str, receiver: &str, predicate: &str) -> bool {
    let guard = format!("if{receiver}.{predicate}(){{");
    let Some((_prefix, after_guard)) = before_call.split_once(&guard) else {
        return false;
    };
    after_guard
        .split_once('}')
        .map_or(after_guard, |(guard_body, _after)| guard_body)
        .contains("return")
}

fn has_unreachable_unchecked_infallible_path_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    compact.contains("fallibility::infallible") && compact.contains("unreachable_unchecked(")
}

fn has_from_utf8_unchecked_validation_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    let Some((before_call, argument)) = from_utf8_unchecked_argument_context(&compact) else {
        return false;
    };
    let validation = format!("from_utf8({argument})");

    before_call.contains(&format!("{validation}.is_ok()"))
        || has_validation_early_return_guard(before_call, &validation, "is_err")
}

fn from_utf8_unchecked_argument_context(compact: &str) -> Option<(&str, &str)> {
    let marker = "from_utf8_unchecked(";
    let call_pos = compact.find(marker)?;
    let before_call = &compact[..call_pos];
    let after_marker = &compact[call_pos + marker.len()..];
    let argument_end = matching_call_argument_end(after_marker)?;
    let argument = &after_marker[..argument_end];
    (!argument.is_empty()).then_some((before_call, argument))
}

fn matching_call_argument_end(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in text.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' if depth == 0 => return Some(idx),
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn has_validation_early_return_guard(before_call: &str, validation: &str, predicate: &str) -> bool {
    let guard = format!("{validation}.{predicate}(){{");
    let Some((_prefix, after_guard)) = before_call.split_once(&guard) else {
        return false;
    };
    after_guard
        .split_once('}')
        .map_or(after_guard, |(guard_body, _after)| guard_body)
        .contains("return")
}

fn has_zeroed_known_valid_zero_type(lower: &str) -> bool {
    let compact = compact_code(lower);
    let Some(target_type) = zeroed_target_type(&compact) else {
        return false;
    };
    matches!(
        target_type,
        "()" | "bool"
            | "char"
            | "f32"
            | "f64"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
    )
}

fn has_transmute_layout_size_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    let Some((before_call, source_type, destination_type, _argument)) =
        transmute_call_context(&compact)
    else {
        return false;
    };
    let normalized = normalize_size_of_paths(before_call);
    has_size_of_equality(&normalized, source_type, destination_type)
}

fn has_transmute_u8_bool_valid_value_evidence(lower: &str) -> bool {
    let compact = compact_code(lower);
    let Some((before_call, source_type, destination_type, argument)) =
        transmute_call_context(&compact)
    else {
        return false;
    };
    let Some(argument) = source_value_identifier(argument) else {
        return false;
    };
    if source_type != "u8" || destination_type != "bool" {
        return false;
    }
    has_u8_bool_value_guard(before_call, argument)
}

fn transmute_call_context(compact: &str) -> Option<(&str, &str, &str, &str)> {
    for marker in ["transmute::<", "transmute_copy::<"] {
        let Some(marker_start) = compact.find(marker) else {
            continue;
        };
        let before_call = &compact[..marker_start];
        let start = marker_start + marker.len();
        let after_marker = &compact[start..];
        let end = matching_generic_argument_end(after_marker)?;
        let arguments = &after_marker[..end];
        let after_arguments = after_marker.get(end + 1..)?;
        let after_open = after_arguments.strip_prefix('(')?;
        let argument_end = matching_call_argument_end(after_open)?;
        let argument = &after_open[..argument_end];
        if let Some((source_type, destination_type)) = split_top_level_pair(arguments) {
            return Some((before_call, source_type, destination_type, argument));
        }
    }
    None
}

fn split_top_level_pair(text: &str) -> Option<(&str, &str)> {
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    for (idx, ch) in text.char_indices() {
        match ch {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 => {
                let left = &text[..idx];
                let right = &text[idx + 1..];
                return (!left.is_empty() && !right.is_empty()).then_some((left, right));
            }
            _ => {}
        }
    }
    None
}

fn normalize_size_of_paths(compact: &str) -> String {
    compact
        .replace("core::mem::size_of", "size_of")
        .replace("std::mem::size_of", "size_of")
        .replace("mem::size_of", "size_of")
}

fn has_size_of_equality(compact: &str, left_type: &str, right_type: &str) -> bool {
    let left = format!("size_of::<{left_type}>()");
    let right = format!("size_of::<{right_type}>()");
    compact.contains(&format!("{left}=={right}"))
        || compact.contains(&format!("{right}=={left}"))
        || has_size_assert_eq(compact, &left, &right)
        || has_size_assert_eq(compact, &right, &left)
}

fn has_size_assert_eq(compact: &str, left: &str, right: &str) -> bool {
    compact.contains(&format!("assert_eq!({left},{right}"))
        || compact.contains(&format!("debug_assert_eq!({left},{right}"))
}

fn has_u8_bool_value_guard(before_call: &str, argument: &str) -> bool {
    before_call.contains(&format!("{argument}<=1"))
        || before_call.contains(&format!("1>={argument}"))
        || before_call.contains(&format!("{argument}<2"))
        || before_call.contains(&format!("2>{argument}"))
        || before_call.contains(&format!("matches!({argument},0|1)"))
        || before_call.contains(&format!("matches!({argument},1|0)"))
        || before_call.contains(&format!("{argument}==0||{argument}==1"))
        || before_call.contains(&format!("{argument}==1||{argument}==0"))
        || has_u8_bool_invalid_early_return_guard(before_call, argument)
}

fn has_u8_bool_invalid_early_return_guard(before_call: &str, argument: &str) -> bool {
    has_invalid_byte_returning_branch(before_call, &format!("{argument}>1"))
        || has_invalid_byte_returning_branch(before_call, &format!("1<{argument}"))
        || has_invalid_byte_returning_branch(before_call, &format!("{argument}>=2"))
        || has_invalid_byte_returning_branch(before_call, &format!("2<={argument}"))
}

fn has_invalid_byte_returning_branch(before_call: &str, predicate: &str) -> bool {
    let guard = format!("if{predicate}{{");
    let Some((_prefix, after_guard)) = before_call.split_once(&guard) else {
        return false;
    };
    after_guard
        .split_once('}')
        .map_or(after_guard, |(guard_body, _after)| guard_body)
        .contains("return")
}

fn is_simple_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn source_value_identifier(argument: &str) -> Option<&str> {
    if is_simple_identifier(argument) {
        return Some(argument);
    }
    let referenced = argument.strip_prefix('&')?;
    is_simple_identifier(referenced).then_some(referenced)
}

fn zeroed_target_type(compact: &str) -> Option<&str> {
    let marker = "zeroed::<";
    let start = compact.find(marker)? + marker.len();
    let after_marker = &compact[start..];
    let end = matching_generic_argument_end(after_marker)?;
    let target_type = &after_marker[..end];
    (!target_type.is_empty()).then_some(target_type)
}

fn matching_generic_argument_end(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (idx, ch) in text.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' if depth == 0 => return Some(idx),
            '>' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn has_maybeuninit_slice_context(lower: &str) -> bool {
    let compact = compact_code(lower);
    compact.contains("from_raw_parts_mut(") && compact.contains("maybeuninit")
}

fn has_maybeuninit_raw_write_context(lower: &str) -> bool {
    let compact = compact_code(lower);
    (compact.contains("write_bytes(") || compact.contains("ptr::write("))
        && compact.contains("maybeuninit")
}

fn has_u8_write_bytes_context(lower: &str) -> bool {
    let compact = compact_code(lower);
    compact.contains("write_bytes(") && compact.contains(":*mutu8")
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

fn has_alignment_guard(site: &ScannedSite, lower: &str) -> bool {
    let compact = compact_code(lower);
    if let Some(receiver) = raw_pointer_alignment_receiver(&site.operation.expression) {
        return has_same_receiver_alignment_guard(&compact, &receiver);
    }
    lower.contains("is_aligned")
        || lower.contains("align_offset")
        || lower.contains("addr() %")
        || lower.contains("as usize %")
        || compact.contains("addr()%")
        || compact.contains("asusize)%")
        || compact.contains("asusize%")
}

fn has_same_receiver_alignment_guard(compact: &str, receiver: &str) -> bool {
    let receiver = compact_code(&receiver.to_ascii_lowercase());
    contains_receiver_fragment(compact, &format!("{receiver}.addr()%"))
        || contains_receiver_fragment(compact, &format!("{receiver}asusize)%"))
        || contains_receiver_fragment(compact, &format!("{receiver}asusize%"))
        || contains_receiver_fragment(compact, &format!("({receiver}asusize)%"))
        || contains_receiver_fragment(compact, &format!("({receiver}asusize%"))
        || same_receiver_method_call(compact, &receiver, "is_aligned")
        || same_receiver_method_call(compact, &receiver, "align_offset")
}

fn same_receiver_method_call(compact: &str, receiver: &str, method: &str) -> bool {
    let direct = format!("{receiver}.{method}");
    if contains_receiver_fragment(compact, &direct) {
        return true;
    }
    let cast_prefix = format!("{receiver}.cast");
    let mut cursor = compact;
    let mut offset = 0usize;
    while let Some(pos) = cursor.find(&cast_prefix) {
        let start = offset + pos;
        let before = compact[..start].chars().next_back();
        let starts_on_boundary = before.is_none_or(|ch| !is_receiver_path_char(ch));
        let after_receiver = &compact[start + receiver.len()..];
        let end = after_receiver
            .find([';', '{', '}'])
            .unwrap_or(after_receiver.len());
        if starts_on_boundary && after_receiver[..end].contains(&format!(".{method}")) {
            return true;
        }
        let next = pos + cast_prefix.len();
        offset += next;
        cursor = &cursor[next..];
    }
    false
}

fn contains_receiver_fragment(compact: &str, fragment: &str) -> bool {
    let mut cursor = compact;
    let mut offset = 0usize;
    while let Some(pos) = cursor.find(fragment) {
        let start = offset + pos;
        let before = compact[..start].chars().next_back();
        if before.is_none_or(|ch| !is_receiver_path_char(ch)) {
            return true;
        }
        let next = pos + fragment.len();
        offset += next;
        cursor = &cursor[next..];
    }
    false
}

fn raw_pointer_alignment_receiver(expression: &str) -> Option<String> {
    let compact = compact_code(&expression.to_ascii_lowercase());
    if let Some(receiver) = receiver_before_marker(&compact, ".cast::<") {
        return Some(receiver.to_string());
    }
    if let Some(receiver) = receiver_before_marker(&compact, ".read(") {
        return Some(receiver.to_string());
    }
    if let Some(receiver) = receiver_before_marker(&compact, ".read_volatile(") {
        return Some(receiver.to_string());
    }
    if let Some(receiver) = receiver_before_marker(&compact, ".write(") {
        return Some(receiver.to_string());
    }
    if let Some(receiver) = receiver_before_marker(&compact, ".write_volatile(") {
        return Some(receiver.to_string());
    }
    None
}

fn receiver_before_marker<'a>(compact: &'a str, marker: &str) -> Option<&'a str> {
    let pos = compact.find(marker)?;
    let before_marker = &compact[..pos];
    let receiver_start = before_marker
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!is_receiver_path_char(ch)).then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    let receiver = &before_marker[receiver_start..];
    (!receiver.is_empty()).then_some(receiver)
}

fn has_nullability_guard(site: &ScannedSite, lower: &str) -> bool {
    let compact = compact_code(lower);
    if let Some(arg) = nonnull_new_unchecked_argument(&site.operation.expression) {
        let arg = compact_code(&arg.to_ascii_lowercase());
        return compact.contains(&format!("nonnull::new({arg})"))
            || has_null_early_return_guard(&compact, &arg);
    }
    lower.contains("is_null") || compact.contains("nonnull::new(")
}

fn has_null_early_return_guard(compact: &str, arg: &str) -> bool {
    let guard = format!("if{arg}.is_null(){{");
    let Some((_prefix, after_guard)) = compact.split_once(&guard) else {
        return false;
    };
    after_guard
        .split_once('}')
        .map_or(after_guard, |(guard_body, _after)| guard_body)
        .contains("return")
}

fn nonnull_new_unchecked_argument(expression: &str) -> Option<String> {
    let compact = compact_code(&expression.to_ascii_lowercase());
    let marker = "nonnull::new_unchecked(";
    let start = compact.find(marker)? + marker.len();
    let rest = &compact[start..];
    let mut depth = 0usize;
    let mut end = rest.len();
    for (idx, ch) in rest.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => {
                end = idx;
                break;
            }
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    let arg = rest[..end].trim();
    (!arg.is_empty()).then(|| arg.to_string())
}

fn contains_word(text: &str, word: &str) -> bool {
    text.split(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
        .any(|token| token == word)
}

fn is_receiver_path_char(ch: char) -> bool {
    ch == '_' || ch == ':' || ch == '.' || ch.is_ascii_alphanumeric()
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
        let safety_colon_comment_site = site_with_context(
            vec!["// Safety: caller checked pointer"],
            "ptr.read()",
            vec![],
        );
        let missing_site = site_with_context(vec!["// ordinary comment"], "ptr.read()", vec![]);

        assert!(contract_evidence(&doc_site).present);
        assert!(contract_evidence(&safety_colon_doc_site).present);
        assert!(contract_evidence(&comment_site).present);
        assert!(contract_evidence(&safety_colon_comment_site).present);
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
    fn bare_align_of_does_not_discharge_alignment() {
        let obligations = vec![SafetyObligation::new(
            "alignment",
            "pointer is aligned for the accessed type",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let align_of_only = site_with_context(
            vec!["let _required = core::mem::align_of::<Header>();"],
            "ptr.read()",
            vec![],
        );
        let modulo_guard = site_with_context(
            vec!["if (ptr as usize) % core::mem::align_of::<Header>() != 0 { return None; }"],
            "ptr.read()",
            vec![],
        );

        assert!(
            !obligation_evidence(&align_of_only, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&modulo_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
    }

    #[test]
    fn alignment_guard_must_match_raw_pointer_receiver_when_known() {
        let obligations = vec![SafetyObligation::new(
            "alignment",
            "pointer is aligned for the accessed type",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let other_pointer_guard = site_with_context(
            vec!["if (other_ptr as usize) % core::mem::align_of::<Header>() != 0 { return None; }"],
            "ptr.cast::<Header>().read()",
            vec![],
        );
        let matching_modulo_guard = site_with_context(
            vec!["if (ptr as usize) % core::mem::align_of::<Header>() != 0 { return None; }"],
            "ptr.cast::<Header>().read()",
            vec![],
        );
        let matching_method_guard = site_with_context(
            vec!["if !ptr.cast::<Header>().is_aligned() { return None; }"],
            "ptr.cast::<Header>().read()",
            vec![],
        );

        assert!(
            !obligation_evidence(&other_pointer_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&matching_modulo_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&matching_method_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
    }

    #[test]
    fn nonnull_constructor_name_does_not_discharge_nullability() {
        let obligations = vec![SafetyObligation::new(
            "non-null",
            "pointer is non-null before constructing NonNull",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let constructor_only = site_with_family(
            OperationFamily::NonNullUnchecked,
            vec![],
            "NonNull::new_unchecked(ptr)",
            vec![],
        );
        let explicit_guard = site_with_family(
            OperationFamily::NonNullUnchecked,
            vec!["if ptr.is_null() { return None; }"],
            "NonNull::new_unchecked(ptr)",
            vec![],
        );

        assert!(
            !obligation_evidence(&constructor_only, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&explicit_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
    }

    #[test]
    fn nonnull_guard_must_match_new_unchecked_argument() {
        let obligations = vec![SafetyObligation::new(
            "non-null",
            "pointer is non-null before constructing NonNull",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let matching_guard = site_with_family(
            OperationFamily::NonNullUnchecked,
            vec!["NonNull::new(ptr)?;"],
            "NonNull::new_unchecked(ptr)",
            vec![],
        );
        let other_guard = site_with_family(
            OperationFamily::NonNullUnchecked,
            vec!["NonNull::new(other)?;"],
            "NonNull::new_unchecked(ptr)",
            vec![],
        );
        let method_guard = site_with_family(
            OperationFamily::NonNullUnchecked,
            vec!["if bucket.as_ptr().is_null() { return None; }"],
            "NonNull::new_unchecked(bucket.as_ptr())",
            vec![],
        );

        assert!(
            obligation_evidence(&matching_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            !obligation_evidence(&other_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&method_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
    }

    #[test]
    fn nonnull_is_null_guard_must_exit_before_unchecked_constructor() {
        let obligations = vec![SafetyObligation::new(
            "non-null",
            "pointer is non-null before constructing NonNull",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let non_returning_guard = site_with_family(
            OperationFamily::NonNullUnchecked,
            vec!["if ptr.is_null() { log_null(); }"],
            "NonNull::new_unchecked(ptr)",
            vec![],
        );
        let returning_guard = site_with_family(
            OperationFamily::NonNullUnchecked,
            vec!["if ptr.is_null() { return None; }"],
            "NonNull::new_unchecked(ptr)",
            vec![],
        );

        assert!(
            !obligation_evidence(&non_returning_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&returning_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
    }

    #[test]
    fn slice_end_pointer_arithmetic_discharges_bounds() {
        let obligations = vec![SafetyObligation::new(
            "bounds",
            "pointer arithmetic stays in-bounds or one-past inside the same allocation",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let slice_end = site_with_family(
            OperationFamily::PointerArithmetic,
            vec!["let start = haystack.as_ptr();"],
            "let end = start.add(haystack.len());",
            vec![],
        );
        let mismatched_len = site_with_family(
            OperationFamily::PointerArithmetic,
            vec!["let start = needle.as_ptr();"],
            "let end = start.add(haystack.len());",
            vec![],
        );

        assert!(
            obligation_evidence(&slice_end, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            !obligation_evidence(&mismatched_len, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
    }

    #[test]
    fn documented_target_feature_declaration_does_not_require_local_guard() {
        let obligations = vec![SafetyObligation::new(
            "target-feature",
            "callers only execute this path on supported hardware",
        )];
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let site = site_with_family(
            OperationFamily::TargetFeature,
            vec!["/// # Safety"],
            "#[target_feature(enable = \"sse2\")]",
            vec!["pub unsafe fn find_raw() {}"],
        );

        let with_contract = obligation_evidence(
            &site,
            &obligations,
            &ContractEvidence::present("contract"),
            &reach,
        );
        let without_contract =
            obligation_evidence(&site, &obligations, &ContractEvidence::missing(), &reach);

        assert!(with_contract[0].discharge.present);
        assert!(!without_contract[0].discharge.present);
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
    fn set_len_capacity_mention_without_bound_is_not_guard_evidence() {
        let obligations = vec![SafetyObligation::new(
            "capacity",
            "new length is at most capacity",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let observation_only = site_with_family(
            OperationFamily::VecSetLen,
            vec!["let capacity = values.capacity();", "record(capacity);"],
            "values.set_len(new_len);",
            vec![],
        );
        let bounded = site_with_family(
            OperationFamily::VecSetLen,
            vec!["assert!(new_len <= values.capacity());"],
            "values.set_len(new_len);",
            vec![],
        );

        assert!(
            !obligation_evidence(&observation_only, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&bounded, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
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
    fn unchecked_constructor_availability_guard_discharges_callee_contract() {
        let obligations = vec![SafetyObligation::new(
            "callee-contract",
            "callee safety preconditions are satisfied",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let guarded = site_with_family(
            OperationFamily::UnsafeFnCall,
            vec!["if One::is_available() {"],
            "unsafe { Some(One::new_unchecked(needle)) }",
            vec!["}"],
        );
        let unguarded = site_with_family(
            OperationFamily::UnsafeFnCall,
            vec![],
            "unsafe { Some(One::new_unchecked(needle)) }",
            vec![],
        );

        let guarded_evidence = obligation_evidence(&guarded, &obligations, &contract, &reach);
        let unguarded_evidence = obligation_evidence(&unguarded, &obligations, &contract, &reach);

        assert!(guarded_evidence[0].discharge.present);
        assert!(!unguarded_evidence[0].discharge.present);
    }

    #[test]
    fn unchecked_constructor_availability_guard_requires_same_receiver_and_precedes_call() {
        let obligations = vec![SafetyObligation::new(
            "callee-contract",
            "callee safety preconditions are satisfied",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let other_receiver = site_with_family(
            OperationFamily::UnsafeFnCall,
            vec!["if Two::is_available() {"],
            "unsafe { Some(One::new_unchecked(needle)) }",
            vec!["}"],
        );
        let post_call = site_with_family(
            OperationFamily::UnsafeFnCall,
            vec![],
            "unsafe { Some(One::new_unchecked(needle)) }",
            vec!["if One::is_available() { record_available(); }"],
        );
        let generic_call = site_with_family(
            OperationFamily::UnsafeFnCall,
            vec!["if One::is_available() {"],
            "unsafe { Some(One::new_unchecked::<Needle>(needle)) }",
            vec!["}"],
        );

        let other_receiver_evidence =
            obligation_evidence(&other_receiver, &obligations, &contract, &reach);
        let post_call_evidence = obligation_evidence(&post_call, &obligations, &contract, &reach);
        let generic_call_evidence =
            obligation_evidence(&generic_call, &obligations, &contract, &reach);

        assert!(!other_receiver_evidence[0].discharge.present);
        assert!(!post_call_evidence[0].discharge.present);
        assert!(generic_call_evidence[0].discharge.present);
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
    fn get_unchecked_bounds_guard_must_match_receiver_when_known() {
        let obligations = vec![SafetyObligation::new(
            "bounds",
            "index is in bounds for the collection",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let other_receiver_guard = site_with_family(
            OperationFamily::GetUnchecked,
            vec!["if index < other_values.len() {"],
            "unsafe { values.get_unchecked_mut(index) }",
            vec!["}"],
        );
        let matching_guard = site_with_family(
            OperationFamily::GetUnchecked,
            vec!["if index < values.len() {"],
            "unsafe { values.get_unchecked_mut(index) }",
            vec!["}"],
        );
        let matching_return_guard = site_with_family(
            OperationFamily::GetUnchecked,
            vec!["if index >= values.len() { return None; }"],
            "unsafe { values.get_unchecked_mut(index) }",
            vec![],
        );

        assert!(
            !obligation_evidence(&other_receiver_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&matching_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
        assert!(
            obligation_evidence(&matching_return_guard, &obligations, &contract, &reach)[0]
                .discharge
                .present
        );
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
    fn maybeuninit_raw_write_discharges_initialized_obligation_only() {
        let obligations = vec![
            SafetyObligation::new("initialized", "memory is initialized for the accessed type"),
            SafetyObligation::new("alignment", "pointer is aligned for the accessed type"),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let raw_write = site_with_family(
            OperationFamily::RawPointerWrite,
            vec!["impl TagSliceExt for [core::mem::MaybeUninit<Tag>] {"],
            "unsafe { self.as_mut_ptr().write_bytes(tag.0, self.len()) }",
            vec![],
        );

        let evidence = obligation_evidence(&raw_write, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
        assert!(!evidence[1].discharge.present);
    }

    #[test]
    fn u8_write_bytes_discharges_alignment_and_initialized_obligations_only() {
        let obligations = vec![
            SafetyObligation::new("initialized", "memory is initialized for the accessed type"),
            SafetyObligation::new("alignment", "pointer is aligned for the accessed type"),
            SafetyObligation::new("pointer-live", "pointer is live"),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let raw_write = site_with_family(
            OperationFamily::RawPointerWrite,
            vec!["pub fn fill_bytes(ptr: *mut u8, len: usize, byte: u8) {"],
            "unsafe { ptr.write_bytes(byte, len) }",
            vec![],
        );

        let evidence = obligation_evidence(&raw_write, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
        assert!(evidence[1].discharge.present);
        assert!(!evidence[2].discharge.present);
    }

    #[test]
    fn unwrap_unchecked_infallible_result_discharges_valid_value_obligation() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let unwrap = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec![
                "let result = reserve_rehash(additional, Fallibility::Infallible);",
                "// SAFETY: infallible mode converts allocation errors before this point.",
            ],
            "unsafe { result.unwrap_unchecked() }",
            vec![],
        );

        let evidence = obligation_evidence(&unwrap, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
    }

    #[test]
    fn unwrap_unchecked_infallible_result_evidence_requires_result_receiver() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let option_unwrap = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec!["let result = reserve_rehash(additional, Fallibility::Infallible);"],
            "unsafe { option.unwrap_unchecked() }",
            vec![],
        );

        let evidence = obligation_evidence(&option_unwrap, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn unwrap_unchecked_infallible_result_must_match_receiver_assignment() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let other_result = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec![
                "let other_result = reserve_rehash(additional, Fallibility::Infallible);",
                "let result = reserve_rehash(additional, Fallibility::Fallible);",
            ],
            "unsafe { result.unwrap_unchecked() }",
            vec![],
        );

        let evidence = obligation_evidence(&other_result, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn unwrap_unchecked_same_receiver_state_discharges_valid_value_obligation() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let option = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec!["if option.is_some() {"],
            "unsafe { option.unwrap_unchecked() }",
            vec!["}"],
        );
        let result = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec!["if result.is_ok() {"],
            "unsafe { result.unwrap_unchecked() }",
            vec!["}"],
        );

        let option_evidence = obligation_evidence(&option, &obligations, &contract, &reach);
        let result_evidence = obligation_evidence(&result, &obligations, &contract, &reach);

        assert!(option_evidence[0].discharge.present);
        assert!(result_evidence[0].discharge.present);
    }

    #[test]
    fn unwrap_unchecked_state_evidence_requires_same_receiver() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let unchecked = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec!["if other.is_some() {"],
            "unsafe { option.unwrap_unchecked() }",
            vec!["}"],
        );

        let evidence = obligation_evidence(&unchecked, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn unwrap_unchecked_state_evidence_must_precede_call() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let unchecked = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec![],
            "unsafe { option.unwrap_unchecked() }",
            vec!["if option.is_some() {}"],
        );

        let evidence = obligation_evidence(&unchecked, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn unwrap_unchecked_early_return_state_guard_discharges_valid_value_obligation() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let option = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec!["if option.is_none() {", "    return 0;", "}"],
            "unsafe { option.unwrap_unchecked() }",
            vec![],
        );
        let result = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec!["if result.is_err() {", "    return 0;", "}"],
            "unsafe { result.unwrap_unchecked() }",
            vec![],
        );

        let option_evidence = obligation_evidence(&option, &obligations, &contract, &reach);
        let result_evidence = obligation_evidence(&result, &obligations, &contract, &reach);

        assert!(option_evidence[0].discharge.present);
        assert!(result_evidence[0].discharge.present);
    }

    #[test]
    fn unwrap_unchecked_early_return_guard_requires_returning_branch() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "value is known to be `Some` or `Ok` before `unwrap_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let unchecked = site_with_family(
            OperationFamily::UnwrapUnchecked,
            vec!["if option.is_none() {", "    observe_none();", "}"],
            "unsafe { option.unwrap_unchecked() }",
            vec![],
        );

        let evidence = obligation_evidence(&unchecked, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn from_utf8_validation_discharges_utf8_obligation() {
        let obligations = vec![SafetyObligation::new("utf8", "bytes are valid UTF-8")];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let checked = site_with_family(
            OperationFamily::StrFromUtf8Unchecked,
            vec!["if core::str::from_utf8(bytes).is_ok() {"],
            "unsafe { core::str::from_utf8_unchecked(bytes) }",
            vec!["}"],
        );
        let early_return = site_with_family(
            OperationFamily::StrFromUtf8Unchecked,
            vec![
                "if core::str::from_utf8(bytes).is_err() {",
                "    return \"\";",
                "}",
            ],
            "unsafe { core::str::from_utf8_unchecked(bytes) }",
            vec![],
        );

        let checked_evidence = obligation_evidence(&checked, &obligations, &contract, &reach);
        let return_evidence = obligation_evidence(&early_return, &obligations, &contract, &reach);

        assert!(checked_evidence[0].discharge.present);
        assert!(return_evidence[0].discharge.present);
    }

    #[test]
    fn from_utf8_validation_requires_same_buffer_and_returning_branch() {
        let obligations = vec![SafetyObligation::new("utf8", "bytes are valid UTF-8")];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let wrong_buffer = site_with_family(
            OperationFamily::StrFromUtf8Unchecked,
            vec!["if core::str::from_utf8(other).is_ok() {"],
            "unsafe { core::str::from_utf8_unchecked(bytes) }",
            vec!["}"],
        );
        let non_returning_branch = site_with_family(
            OperationFamily::StrFromUtf8Unchecked,
            vec![
                "if core::str::from_utf8(bytes).is_err() {",
                "    log_invalid();",
                "}",
            ],
            "unsafe { core::str::from_utf8_unchecked(bytes) }",
            vec![],
        );

        let wrong_buffer_evidence =
            obligation_evidence(&wrong_buffer, &obligations, &contract, &reach);
        let non_returning_evidence =
            obligation_evidence(&non_returning_branch, &obligations, &contract, &reach);

        assert!(!wrong_buffer_evidence[0].discharge.present);
        assert!(!non_returning_evidence[0].discharge.present);
    }

    #[test]
    fn zeroed_known_valid_zero_types_discharge_valid_zero_obligation() {
        let obligations = vec![SafetyObligation::new(
            "valid-zero",
            "all-zero bit pattern is a valid value for the target type",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };

        for target_type in [
            "()", "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "u8",
            "u16", "u32", "u64", "u128", "usize",
        ] {
            let expression = format!("unsafe {{ core::mem::zeroed::<{target_type}>() }}");
            let zeroed = site_with_family(
                OperationFamily::Zeroed,
                vec!["pub fn zero_value() {"],
                &expression,
                vec![],
            );

            let evidence = obligation_evidence(&zeroed, &obligations, &contract, &reach);

            assert!(
                evidence[0].discharge.present,
                "{target_type} should be recognized as a known valid-zero target"
            );
        }
    }

    #[test]
    fn zeroed_nonnull_target_keeps_valid_zero_obligation_missing() {
        let obligations = vec![SafetyObligation::new(
            "valid-zero",
            "all-zero bit pattern is a valid value for the target type",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let zeroed = site_with_family(
            OperationFamily::Zeroed,
            vec!["pub fn invalid_zeroed_nonnull() -> core::ptr::NonNull<u8> {"],
            "unsafe { core::mem::zeroed::<core::ptr::NonNull<u8>>() }",
            vec![],
        );

        let evidence = obligation_evidence(&zeroed, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn transmute_size_equality_discharges_layout_obligation_only() {
        let obligations = vec![
            SafetyObligation::new("layout", "source and destination layouts are compatible"),
            SafetyObligation::new(
                "valid-value",
                "destination value satisfies Rust validity rules",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec!["assert_eq!(core::mem::size_of::<u8>(), core::mem::size_of::<bool>());"],
            "unsafe { core::mem::transmute::<u8, bool>(value) }",
            vec![],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
        assert!(!evidence[1].discharge.present);
    }

    #[test]
    fn transmute_copy_size_equality_discharges_layout_obligation_only() {
        let obligations = vec![
            SafetyObligation::new("layout", "source and destination layouts are compatible"),
            SafetyObligation::new(
                "valid-value",
                "destination value satisfies Rust validity rules",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec!["debug_assert_eq!(core::mem::size_of::<u8>(), core::mem::size_of::<bool>());"],
            "unsafe { core::mem::transmute_copy::<u8, bool>(&value) }",
            vec![],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
        assert!(!evidence[1].discharge.present);
    }

    #[test]
    fn transmute_size_equality_requires_matching_type_pair() {
        let obligations = vec![SafetyObligation::new(
            "layout",
            "source and destination layouts are compatible",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec!["assert_eq!(core::mem::size_of::<u8>(), core::mem::size_of::<u16>());"],
            "unsafe { core::mem::transmute::<u8, bool>(value) }",
            vec![],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn transmute_size_equality_must_precede_call() {
        let obligations = vec![SafetyObligation::new(
            "layout",
            "source and destination layouts are compatible",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec![],
            "unsafe { core::mem::transmute::<u8, bool>(value) }",
            vec!["assert_eq!(core::mem::size_of::<u8>(), core::mem::size_of::<bool>());"],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn transmute_u8_bool_guard_discharges_valid_value_obligation_only() {
        let obligations = vec![
            SafetyObligation::new("layout", "source and destination layouts are compatible"),
            SafetyObligation::new(
                "valid-value",
                "destination value satisfies Rust validity rules",
            ),
        ];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec!["assert!(value <= 1);"],
            "unsafe { core::mem::transmute::<u8, bool>(value) }",
            vec![],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
        assert!(evidence[1].discharge.present);
    }

    #[test]
    fn transmute_copy_u8_bool_guard_discharges_valid_value_obligation() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "destination value satisfies Rust validity rules",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec!["assert!(value <= 1);"],
            "unsafe { core::mem::transmute_copy::<u8, bool>(&value) }",
            vec![],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
    }

    #[test]
    fn transmute_u8_bool_guard_requires_same_argument_and_preceding_guard() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "destination value satisfies Rust validity rules",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let other_arg = site_with_family(
            OperationFamily::Transmute,
            vec!["assert!(other <= 1);"],
            "unsafe { core::mem::transmute::<u8, bool>(value) }",
            vec![],
        );
        let post_call_guard = site_with_family(
            OperationFamily::Transmute,
            vec![],
            "unsafe { core::mem::transmute::<u8, bool>(value) }",
            vec!["assert!(value <= 1);"],
        );
        let unsupported_pair = site_with_family(
            OperationFamily::Transmute,
            vec!["assert!(value <= 1);"],
            "unsafe { core::mem::transmute::<u8, char>(value) }",
            vec![],
        );

        let other_arg_evidence = obligation_evidence(&other_arg, &obligations, &contract, &reach);
        let post_call_evidence =
            obligation_evidence(&post_call_guard, &obligations, &contract, &reach);
        let unsupported_pair_evidence =
            obligation_evidence(&unsupported_pair, &obligations, &contract, &reach);

        assert!(!other_arg_evidence[0].discharge.present);
        assert!(!post_call_evidence[0].discharge.present);
        assert!(!unsupported_pair_evidence[0].discharge.present);
    }

    #[test]
    fn transmute_u8_bool_early_return_guard_discharges_valid_value_obligation() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "destination value satisfies Rust validity rules",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec!["if value > 1 {", "    return false;", "}"],
            "unsafe { core::mem::transmute::<u8, bool>(value) }",
            vec![],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
    }

    #[test]
    fn transmute_copy_u8_bool_early_return_guard_discharges_valid_value_obligation() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "destination value satisfies Rust validity rules",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec!["if value > 1 {", "    return false;", "}"],
            "unsafe { core::mem::transmute_copy::<u8, bool>(&value) }",
            vec![],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
    }

    #[test]
    fn transmute_u8_bool_early_return_guard_requires_returning_branch() {
        let obligations = vec![SafetyObligation::new(
            "valid-value",
            "destination value satisfies Rust validity rules",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let transmute = site_with_family(
            OperationFamily::Transmute,
            vec!["if value > 1 {", "    log_invalid();", "}"],
            "unsafe { core::mem::transmute::<u8, bool>(value) }",
            vec![],
        );

        let evidence = obligation_evidence(&transmute, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
    }

    #[test]
    fn unreachable_unchecked_infallible_path_discharges_unreachable_obligation() {
        let obligations = vec![SafetyObligation::new(
            "unreachable",
            "control flow cannot reach this path before `unreachable_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let unreachable = site_with_family(
            OperationFamily::UnreachableUnchecked,
            vec![
                "match fallible_with_capacity(Fallibility::Infallible) {",
                "    Ok(value) => value,",
                "    // SAFETY: infallible mode handles allocation errors before this point.",
            ],
            "Err(_) => unsafe { hint::unreachable_unchecked() },",
            vec![],
        );

        let evidence = obligation_evidence(&unreachable, &obligations, &contract, &reach);

        assert!(evidence[0].discharge.present);
    }

    #[test]
    fn unreachable_unchecked_evidence_requires_infallible_context() {
        let obligations = vec![SafetyObligation::new(
            "unreachable",
            "control flow cannot reach this path before `unreachable_unchecked`",
        )];
        let contract = ContractEvidence::present("contract");
        let reach = ReachEvidence {
            state: "owner_reached".to_string(),
            summary: "reached".to_string(),
        };
        let unreachable = site_with_family(
            OperationFamily::UnreachableUnchecked,
            vec!["match fallible_with_capacity(Fallibility::Fallible) {"],
            "Err(_) => unsafe { hint::unreachable_unchecked() },",
            vec![],
        );

        let evidence = obligation_evidence(&unreachable, &obligations, &contract, &reach);

        assert!(!evidence[0].discharge.present);
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
