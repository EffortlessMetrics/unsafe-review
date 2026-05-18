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
    let syntax_operation_lines = syntax_sites
        .iter()
        .filter(|site| site.kind == UnsafeSiteKind::Operation)
        .map(|site| site.line)
        .collect::<BTreeSet<_>>();
    let syntax_operation_block_lines = operation_block_start_lines(&parsed);
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut line_comment_state = LineCommentState::default();
    for (idx, raw) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        let detection_line = line_for_text_detection(raw, &mut line_comment_state);
        let detection_trimmed = detection_line.trim();
        if detection_trimmed.is_empty() {
            continue;
        }
        let Some((kind, family)) = detect_site(detection_trimmed) else {
            continue;
        };
        if kind == UnsafeSiteKind::UnsafeBlock
            && family == OperationFamily::Unknown
            && (syntax_operation_lines.contains(&line_no)
                || syntax_operation_block_lines.contains(&line_no))
        {
            continue;
        }
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
        let owner = parse_fn_name(&detected.source_snippet)
            .or_else(|| parse_trait_name(&detected.source_snippet))
            .or_else(|| parse_impl_owner(&detected.source_snippet))
            .or_else(|| find_owner(&lines, idx));
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

#[derive(Default)]
struct LineCommentState {
    block_depth: usize,
}

fn line_for_text_detection(line: &str, state: &mut LineCommentState) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    let mut string_hashes = 0usize;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if state.block_depth > 0 {
            if ch == '/' && chars.peek() == Some(&'*') {
                state.block_depth += 1;
                let _ = chars.next();
            } else if ch == '*' && chars.peek() == Some(&'/') {
                state.block_depth = state.block_depth.saturating_sub(1);
                let _ = chars.next();
            }
            continue;
        }

        if in_string {
            if string_hashes == 0 {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                    out.push('"');
                }
                continue;
            }

            if ch == '"' && raw_string_hashes_at_end(&mut chars, string_hashes) {
                in_string = false;
                out.push('"');
            }
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            break;
        }
        if ch == '/' && chars.peek() == Some(&'*') {
            state.block_depth += 1;
            let _ = chars.next();
            continue;
        }
        if ch == 'r'
            && let Some(hashes) = raw_string_hashes_at_start(&mut chars)
        {
            for _ in 0..hashes {
                let _ = chars.next();
            }
            let _ = chars.next();
            in_string = true;
            string_hashes = hashes;
            out.push('"');
            continue;
        }
        if ch == '"' {
            in_string = true;
            string_hashes = 0;
            escaped = false;
            out.push('"');
            continue;
        }
        out.push(ch);
    }

    out
}

fn raw_string_hashes_at_start<I>(chars: &mut std::iter::Peekable<I>) -> Option<usize>
where
    I: Iterator<Item = char> + Clone,
{
    let mut clone = chars.clone();
    let mut hashes = 0usize;
    while clone.peek() == Some(&'#') {
        hashes += 1;
        let _ = clone.next();
    }
    (clone.peek() == Some(&'"')).then_some(hashes)
}

fn raw_string_hashes_at_end<I>(chars: &mut std::iter::Peekable<I>, hashes: usize) -> bool
where
    I: Iterator<Item = char> + Clone,
{
    let mut clone = chars.clone();
    for _ in 0..hashes {
        if clone.next() != Some('#') {
            return false;
        }
    }
    for _ in 0..hashes {
        let _ = chars.next();
    }
    true
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
    let unsafe_block_ranges = unsafe_block_ranges(parsed);
    let operation_block_ranges = operation_block_ranges(parsed, &unsafe_block_ranges);
    for fact in &parsed.nodes {
        let Some((kind, family)) =
            detect_syntax_site(fact, &unsafe_block_ranges, &operation_block_ranges)
        else {
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

fn operation_block_start_lines(parsed: &ParsedSource) -> BTreeSet<usize> {
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
        "CALL_EXPR" | "METHOD_CALL_EXPR" | "MACRO_EXPR" => {
            matches!(
                detect_site(&normalize_call_spacing(&compact)),
                Some((UnsafeSiteKind::Operation, _family))
            )
        }
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

fn compact_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_call_spacing(text: &str) -> String {
    text.replace(" (", "(")
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
        if let Some(name) = parse_impl_owner(line) {
            return Some(name);
        }
        if let Some(name) = parse_trait_name(line) {
            return Some(name);
        }
        if let Some(name) = parse_fn_name(line) {
            return Some(name);
        }
        if line.starts_with("impl ") || line.starts_with("pub impl ") {
            return Some("impl".to_string());
        }
    }
    None
}

fn parse_impl_owner(line: &str) -> Option<String> {
    if !line.contains("impl ") {
        return None;
    }
    let owner_start = line
        .find(" for ")
        .map(|pos| pos + " for ".len())
        .or_else(|| line.find("impl ").map(|pos| pos + "impl ".len()))?;
    parse_ident(&line[owner_start..])
}

fn parse_fn_name(line: &str) -> Option<String> {
    let marker = "fn ";
    let pos = line.find(marker)?;
    let rest = &line[pos + marker.len()..];
    parse_ident(rest)
}

fn parse_trait_name(line: &str) -> Option<String> {
    let marker = "trait ";
    let pos = line.find(marker)?;
    let rest = &line[pos + marker.len()..];
    parse_ident(rest)
}

fn parse_ident(rest: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn text_detection_ignores_comment_only_unsafe_tokens() {
        let mut state = LineCommentState::default();

        let line = line_for_text_detection("// unsafe { ptr::read(ptr) }", &mut state);

        assert!(line.trim().is_empty());
        assert_eq!(detect_site(line.trim()), None);
    }

    #[test]
    fn text_detection_ignores_block_comment_unsafe_tokens() {
        let mut state = LineCommentState::default();

        let first = line_for_text_detection("/* unsafe {", &mut state);
        let second = line_for_text_detection("ptr::read(ptr); */ pub fn safe() {}", &mut state);

        assert!(first.trim().is_empty());
        assert_eq!(second.trim(), "pub fn safe() {}");
        assert_eq!(detect_site(second.trim()), None);
    }

    #[test]
    fn text_detection_ignores_string_literal_unsafe_tokens_but_preserves_extern_abi() {
        let mut state = LineCommentState::default();

        let string_line = line_for_text_detection(
            "let text = \"unsafe { core::ptr::read(ptr) }\";",
            &mut state,
        );
        let extern_line = line_for_text_detection("unsafe extern \"C\" {", &mut state);

        assert_eq!(detect_site(string_line.trim()), None);
        assert_eq!(
            detect_site(extern_line.trim()),
            Some((UnsafeSiteKind::ExternBlock, OperationFamily::Ffi))
        );
    }

    #[test]
    fn scan_file_does_not_emit_cards_for_unsafe_words_in_comments_or_strings() -> Result<(), String>
    {
        let root = unique_temp_dir()?;
        fs::create_dir_all(root.join("src"))
            .map_err(|err| format!("create temp src failed: {err}"))?;
        fs::write(
            root.join("src/lib.rs"),
            "pub fn safe_text() -> &'static str {\n    /* unsafe { core::ptr::read(ptr) } */\n    \"unsafe { core::ptr::read(ptr) }\"\n}\n",
        )
        .map_err(|err| format!("write temp source failed: {err}"))?;

        let sites = scan_file(&root, &PathBuf::from("src/lib.rs"), None, true)?;

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp dir failed: {err}"))?;
        assert!(sites.is_empty(), "unexpected sites: {sites:#?}");
        Ok(())
    }

    fn unique_temp_dir() -> Result<PathBuf, String> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("system clock before UNIX_EPOCH: {err}"))?
            .as_nanos();
        Ok(std::env::temp_dir().join(format!("unsafe-review-scanner-test-{nanos}")))
    }
}
