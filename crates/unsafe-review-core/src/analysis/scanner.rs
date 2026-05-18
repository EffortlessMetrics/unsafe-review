use super::syntax::{ParsedSource, SyntaxNodeFact};
use crate::domain::{OperationFamily, SourceLocation, UnsafeOperation, UnsafeSite, UnsafeSiteKind};
use crate::input::diff::DiffIndex;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct ScannedSite {
    pub(crate) site: UnsafeSite,
    pub(crate) operation: UnsafeOperation,
    pub(crate) context_before: Vec<String>,
    pub(crate) context_after: Vec<String>,
}

pub(crate) fn scan_file(
    root: &Path,
    rel: &PathBuf,
    diff: Option<&DiffIndex>,
    repo_mode: bool,
) -> Result<Vec<ScannedSite>, String> {
    let abs = root.join(rel);
    let text =
        fs::read_to_string(&abs).map_err(|err| format!("read {} failed: {err}", abs.display()))?;
    let lines: Vec<&str> = text.lines().collect();
    let parsed = super::syntax::parse_source(text.as_str());
    let syntax_sites = if parsed.parse_errors.is_empty() {
        detect_syntax_sites(&parsed)
    } else {
        Vec::new()
    };
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for (idx, raw) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        let Some((kind, family)) = detect_site(trimmed) else {
            continue;
        };
        seen.insert(site_key(line_no, &kind, &family));
        let changed = diff.is_none_or(|d| repo_mode || d.contains_near(rel, line_no));
        if !changed && !repo_mode {
            continue;
        }
        let owner = find_owner(&lines, idx);
        let visibility = if trimmed.starts_with("pub ") || trimmed.contains(" pub ") {
            "public"
        } else {
            "private"
        }
        .to_string();
        let public_api_surface = trimmed.contains("pub unsafe fn")
            || trimmed.contains("pub unsafe trait")
            || trimmed.contains("pub unsafe impl");
        let context_before = context_slice(&lines, idx.saturating_sub(8), idx);
        let context_after = context_slice(&lines, idx + 1, (idx + 8).min(lines.len()));
        out.push(ScannedSite {
            site: UnsafeSite {
                location: SourceLocation::new(rel.clone(), line_no, first_non_ws_column(raw)),
                kind,
                owner,
                visibility,
                public_api_surface,
                changed,
                snippet: trimmed.to_string(),
            },
            operation: UnsafeOperation {
                family,
                expression: trimmed.to_string(),
            },
            context_before,
            context_after,
        });
    }

    for detected in syntax_sites {
        if !seen.insert(site_key(detected.line, &detected.kind, &detected.family)) {
            continue;
        }
        let changed = diff.is_none_or(|d| repo_mode || d.contains_near(rel, detected.line));
        if !changed && !repo_mode {
            continue;
        }
        let idx = detected.line.saturating_sub(1);
        let owner = parse_fn_name(&detected.source_snippet).or_else(|| find_owner(&lines, idx));
        let visibility = if is_public_surface(&detected.source_snippet) {
            "public"
        } else {
            "private"
        }
        .to_string();
        let public_api_surface = is_public_api_surface(&detected.kind, &detected.source_snippet);
        let context_before = context_slice(&lines, idx.saturating_sub(8), idx.min(lines.len()));
        let context_after = context_slice(
            &lines,
            (idx + 1).min(lines.len()),
            (idx + 8).min(lines.len()),
        );
        out.push(ScannedSite {
            site: UnsafeSite {
                location: SourceLocation::new(rel.clone(), detected.line, detected.column),
                kind: detected.kind,
                owner,
                visibility,
                public_api_surface,
                changed,
                snippet: detected.card_snippet.clone(),
            },
            operation: UnsafeOperation {
                family: detected.family,
                expression: detected.card_snippet,
            },
            context_before,
            context_after,
        });
    }
    out.sort_by(|left, right| {
        left.site
            .location
            .line
            .cmp(&right.site.location.line)
            .then(left.site.location.column.cmp(&right.site.location.column))
    });
    Ok(out)
}

fn first_non_ws_column(line: &str) -> usize {
    line.chars()
        .position(|ch| !ch.is_whitespace())
        .map_or(1, |pos| pos + 1)
}

fn context_slice(lines: &[&str], start: usize, end: usize) -> Vec<String> {
    lines[start..end]
        .iter()
        .map(|line| line.trim().to_string())
        .collect()
}

fn detect_site(line: &str) -> Option<(UnsafeSiteKind, OperationFamily)> {
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
    if line.contains("new_unchecked") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::NonNullUnchecked));
    }
    if line.contains("Pin::new_unchecked") || line.contains("get_unchecked_mut") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::PinUnchecked));
    }
    if line.contains("get_unchecked") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::GetUnchecked));
    }
    if line.contains(".read()") || line.contains("ptr::read") {
        return Some((UnsafeSiteKind::Operation, OperationFamily::RawPointerRead));
    }
    if line.contains(".write(") || line.contains("ptr::write") {
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

#[derive(Clone, Debug)]
struct DetectedSyntaxSite {
    line: usize,
    column: usize,
    kind: UnsafeSiteKind,
    family: OperationFamily,
    card_snippet: String,
    source_snippet: String,
}

fn detect_syntax_sites(parsed: &ParsedSource) -> Vec<DetectedSyntaxSite> {
    let mut sites = Vec::new();
    for fact in &parsed.nodes {
        let Some((kind, family)) = detect_syntax_site(fact) else {
            continue;
        };
        let _span_len = fact.end.saturating_sub(fact.start);
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

fn detect_syntax_site(fact: &SyntaxNodeFact) -> Option<(UnsafeSiteKind, OperationFamily)> {
    let compact = compact_whitespace(&fact.snippet);
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
        "BLOCK_EXPR" if is_unknown_unsafe_block(&compact) => {
            Some((UnsafeSiteKind::UnsafeBlock, OperationFamily::Unknown))
        }
        "CALL_EXPR" | "METHOD_CALL_EXPR" | "MACRO_EXPR" => detect_site(&compact),
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

fn compact_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn site_key(
    line: usize,
    kind: &UnsafeSiteKind,
    family: &OperationFamily,
) -> (usize, String, String) {
    (line, kind.as_str().to_string(), family.as_str().to_string())
}

fn is_public_surface(snippet: &str) -> bool {
    let compact = compact_whitespace(snippet);
    compact.starts_with("pub ") || compact.contains(" pub ")
}

fn is_public_api_surface(kind: &UnsafeSiteKind, snippet: &str) -> bool {
    if !matches!(
        kind,
        UnsafeSiteKind::UnsafeFn
            | UnsafeSiteKind::UnsafeTrait
            | UnsafeSiteKind::UnsafeImpl
            | UnsafeSiteKind::UnsafeImplSend
            | UnsafeSiteKind::UnsafeImplSync
    ) {
        return false;
    }
    is_public_surface(snippet)
}

fn find_owner(lines: &[&str], idx: usize) -> Option<String> {
    for raw in lines[..=idx].iter().rev().take(80) {
        let line = raw.trim();
        if let Some(name) = parse_fn_name(line) {
            return Some(name);
        }
        if line.starts_with("impl ") || line.starts_with("pub impl ") {
            return Some("impl".to_string());
        }
    }
    None
}

fn parse_fn_name(line: &str) -> Option<String> {
    let marker = "fn ";
    let pos = line.find(marker)?;
    let rest = &line[pos + marker.len()..];
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
