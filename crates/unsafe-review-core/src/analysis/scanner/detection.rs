use super::super::syntax::{ParsedSource, SyntaxNodeFact};
use super::text::{compact_whitespace, normalize_call_spacing};
use crate::domain::{OperationFamily, UnsafeSiteKind};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(super) struct DetectedSyntaxSite {
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) kind: UnsafeSiteKind,
    pub(super) family: OperationFamily,
    pub(super) card_snippet: String,
    pub(super) source_snippet: String,
}

pub(super) fn detect_site(line: &str) -> Option<(UnsafeSiteKind, OperationFamily)> {
    if line.contains("unsafe impl") && line.contains("Send") {
        return Some((
            UnsafeSiteKind::UnsafeImplSend,
            OperationFamily::UnsafeImplSendSync,
        ));
    }
    if line.contains("unsafe impl") && line.contains("Sync") {
        return Some((
            UnsafeSiteKind::UnsafeImplSync,
            OperationFamily::UnsafeImplSendSync,
        ));
    }
    if line.contains("unsafe fn") {
        return Some((UnsafeSiteKind::UnsafeFn, OperationFamily::Unknown));
    }
    if line.contains("unsafe trait") {
        return Some((UnsafeSiteKind::UnsafeTrait, OperationFamily::Unknown));
    }
    if line.contains("unsafe impl") {
        return Some((UnsafeSiteKind::UnsafeImpl, OperationFamily::Unknown));
    }
    if line.contains("extern \"") || line.starts_with("extern ") || line.contains("unsafe extern") {
        return Some((UnsafeSiteKind::ExternBlock, OperationFamily::Ffi));
    }
    if line.contains("static mut") {
        return Some((UnsafeSiteKind::StaticMut, OperationFamily::StaticMut));
    }
    if line.contains("copy_nonoverlapping") {
        return Some((
            UnsafeSiteKind::Operation,
            OperationFamily::CopyNonOverlapping,
        ));
    }
    if line.contains("from_raw_parts") {
        return Some((
            UnsafeSiteKind::Operation,
            OperationFamily::SliceFromRawParts,
        ));
    }
    if line.contains("from_utf8_unchecked") {
        return Some((
            UnsafeSiteKind::Operation,
            OperationFamily::StrFromUtf8Unchecked,
        ));
    }
    if line.contains("assume_init") {
        return Some((
            UnsafeSiteKind::Operation,
            OperationFamily::MaybeUninitAssumeInit,
        ));
    }
    if line.contains("set_len") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::VecSetLen));
    }
    if line.contains("transmute") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::Transmute));
    }
    if line.contains("zeroed") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::Zeroed));
    }
    if line.contains("Box::from_raw") || line.contains("from_raw(") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::BoxFromRaw));
    }
    if line.contains("Pin::new_unchecked") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::PinUnchecked));
    }
    if line.contains("get_unchecked") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::GetUnchecked));
    }
    if line.contains("new_unchecked") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::NonNullUnchecked));
    }
    if line.contains(".read()") || line.contains("ptr::read") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::RawPointerRead));
    }
    if is_raw_pointer_write(line) {
        return Some((UnsafeSiteKind::Operation, OperationFamily::RawPointerWrite));
    }
    if line.contains(".add(") || line.contains(".offset(") {
        return Some((
            UnsafeSiteKind::Operation,
            OperationFamily::PointerArithmetic,
        ));
    }
    if line.contains("asm!") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::InlineAsm));
    }
    if line.contains("target_feature") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::TargetFeature));
    }
    if line.contains("unsafe {") || line == "unsafe" {
        return Some((UnsafeSiteKind::UnsafeBlock, OperationFamily::Unknown));
    }
    None
}

pub(super) fn detect_syntax_sites(parsed: &ParsedSource) -> Vec<DetectedSyntaxSite> {
    let mut sites = Vec::new();
    let unsafe_block_ranges = unsafe_block_ranges(parsed);
    let operation_block_ranges = operation_block_ranges(parsed, &unsafe_block_ranges);
    for fact in &parsed.nodes {
        let Some((kind, family)) =
            detect_syntax_site(fact, &unsafe_block_ranges, &operation_block_ranges)
        else {
            continue;
        };
        let card_snippet = card_snippet_for(fact, &kind);
        sites.push(DetectedSyntaxSite {
            line: fact.line,
            column: fact.column,
            kind,
            family,
            card_snippet,
            source_snippet: fact.snippet.clone(),
        });
    }
    sites.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.column.cmp(&right.column))
    });
    sites
}

pub(super) fn operation_block_start_lines(parsed: &ParsedSource) -> BTreeSet<usize> {
    let unsafe_block_ranges = unsafe_block_ranges(parsed);
    let operation_block_ranges = operation_block_ranges(parsed, &unsafe_block_ranges);
    parsed
        .nodes
        .iter()
        .filter(|fact| {
            fact.kind == "BLOCK_EXPR" && operation_block_ranges.contains(&(fact.start, fact.end))
        })
        .map(|fact| fact.line)
        .collect()
}

fn detect_syntax_site(
    fact: &SyntaxNodeFact,
    unsafe_block_ranges: &[(usize, usize)],
    operation_block_ranges: &BTreeSet<(usize, usize)>,
) -> Option<(UnsafeSiteKind, OperationFamily)> {
    let compact = compact_whitespace(&fact.snippet);
    if compact.starts_with("//") {
        return None;
    }
    match fact.kind.as_str() {
        "FN" if compact.contains("unsafe fn") => {
            Some((UnsafeSiteKind::UnsafeFn, OperationFamily::Unknown))
        }
        "TRAIT" if compact.contains("unsafe trait") => {
            Some((UnsafeSiteKind::UnsafeTrait, OperationFamily::Unknown))
        }
        "IMPL" if compact.contains("unsafe impl") && compact.contains(" Send") => Some((
            UnsafeSiteKind::UnsafeImplSend,
            OperationFamily::UnsafeImplSendSync,
        )),
        "IMPL" if compact.contains("unsafe impl") && compact.contains(" Sync") => Some((
            UnsafeSiteKind::UnsafeImplSync,
            OperationFamily::UnsafeImplSendSync,
        )),
        "IMPL" if compact.contains("unsafe impl") => {
            Some((UnsafeSiteKind::UnsafeImpl, OperationFamily::Unknown))
        }
        "EXTERN_BLOCK" if compact.contains("extern") => {
            Some((UnsafeSiteKind::ExternBlock, OperationFamily::Ffi))
        }
        "STATIC" if compact.contains("static mut") => {
            Some((UnsafeSiteKind::StaticMut, OperationFamily::StaticMut))
        }
        "BLOCK_EXPR"
            if is_unknown_unsafe_block(&compact)
                && !operation_block_ranges.contains(&(fact.start, fact.end)) =>
        {
            Some((UnsafeSiteKind::UnsafeBlock, OperationFamily::Unknown))
        }
        "PREFIX_EXPR"
            if is_raw_pointer_deref(&compact) && is_inside_range(fact, unsafe_block_ranges) =>
        {
            Some((UnsafeSiteKind::Operation, OperationFamily::RawPointerDeref))
        }
        "CALL_EXPR" | "METHOD_CALL_EXPR" | "MACRO_EXPR" => {
            detect_site(&normalize_call_spacing(&compact))
        }
        _ => None,
    }
}

fn card_snippet_for(fact: &SyntaxNodeFact, kind: &UnsafeSiteKind) -> String {
    let compact = compact_whitespace(&fact.snippet);
    match kind {
        UnsafeSiteKind::UnsafeBlock => "unsafe {".to_string(),
        UnsafeSiteKind::UnsafeFn
        | UnsafeSiteKind::UnsafeTrait
        | UnsafeSiteKind::UnsafeImpl
        | UnsafeSiteKind::UnsafeImplSend
        | UnsafeSiteKind::UnsafeImplSync
        | UnsafeSiteKind::ExternBlock => compact
            .split_once('{')
            .map_or(compact.clone(), |(head, _tail)| {
                format!("{} {{", head.trim())
            }),
        UnsafeSiteKind::Operation => normalize_call_spacing(&compact),
        _ => compact,
    }
}

fn is_unknown_unsafe_block(compact: &str) -> bool {
    compact.starts_with("unsafe {")
        && !matches!(
            detect_site(compact),
            Some((UnsafeSiteKind::Operation, _family))
        )
}

fn is_raw_pointer_deref(compact: &str) -> bool {
    compact.starts_with('*') && !compact.starts_with("**")
}

fn is_raw_pointer_write(line: &str) -> bool {
    line.contains("ptr::write")
        || line.contains("ptr.write(")
        || line.contains(".as_mut_ptr().write(")
        || line.contains(".cast_mut().write(")
        || (line.contains(".cast::<") && line.contains(".write("))
}

fn unsafe_block_ranges(parsed: &ParsedSource) -> Vec<(usize, usize)> {
    parsed
        .nodes
        .iter()
        .filter(|fact| {
            fact.kind == "BLOCK_EXPR" && compact_whitespace(&fact.snippet).starts_with("unsafe {")
        })
        .map(|fact| (fact.start, fact.end))
        .collect()
}

fn operation_block_ranges(
    parsed: &ParsedSource,
    unsafe_block_ranges: &[(usize, usize)],
) -> BTreeSet<(usize, usize)> {
    parsed
        .nodes
        .iter()
        .filter(|fact| syntax_operation_in_unsafe_block(fact, unsafe_block_ranges))
        .filter_map(|fact| containing_range(fact, unsafe_block_ranges))
        .collect()
}

fn syntax_operation_in_unsafe_block(
    fact: &SyntaxNodeFact,
    unsafe_block_ranges: &[(usize, usize)],
) -> bool {
    if !is_inside_range(fact, unsafe_block_ranges) {
        return false;
    }
    let compact = compact_whitespace(&fact.snippet);
    match fact.kind.as_str() {
        "PREFIX_EXPR" => is_raw_pointer_deref(&compact),
        "CALL_EXPR" | "METHOD_CALL_EXPR" | "MACRO_EXPR" => matches!(
            detect_site(&normalize_call_spacing(&compact)),
            Some((UnsafeSiteKind::Operation, _family))
        ),
        _ => false,
    }
}

fn containing_range(fact: &SyntaxNodeFact, ranges: &[(usize, usize)]) -> Option<(usize, usize)> {
    ranges
        .iter()
        .copied()
        .find(|(start, end)| fact.start >= *start && fact.end <= *end)
}

fn is_inside_range(fact: &SyntaxNodeFact, ranges: &[(usize, usize)]) -> bool {
    ranges
        .iter()
        .any(|(start, end)| fact.start >= *start && fact.end <= *end)
}
