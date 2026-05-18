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
    for (idx, raw) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        let code_trimmed = mask_non_code_segments(raw).trim().to_string();
        if code_trimmed.is_empty() {
            continue;
        }
        let Some((kind, family)) = detect_site(&code_trimmed) else {
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
        let visibility = if code_trimmed.starts_with("pub ") || code_trimmed.contains(" pub ") {
            "public"
        } else {
            "private"
        }
        .to_string();
        let public_api_surface = code_trimmed.contains("pub unsafe fn")
            || code_trimmed.contains("pub unsafe trait")
            || code_trimmed.contains("pub unsafe impl");
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

fn mask_non_code_segments(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.char_indices().peekable();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while let Some((idx, ch)) = chars.next() {
        if in_string {
            out.push(' ');
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if in_char {
            out.push(' ');
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                in_char = false;
            }
            continue;
        }

        if ch == '/' && matches!(chars.peek(), Some((_next_idx, '/'))) {
            out.extend(std::iter::repeat_n(' ', line[idx..].chars().count()));
            break;
        }

        if let ('r', Some(hashes)) = (ch, raw_string_hashes(line, idx)) {
            out.push(' ');
            for _ in 0..hashes {
                let _ = chars.next();
                out.push(' ');
            }
            let _ = chars.next();
            out.push(' ');

            let terminator = format!("\"{}", "#".repeat(hashes));
            while let Some((raw_idx, raw_ch)) = chars.next() {
                out.push(' ');
                if line[raw_idx..].starts_with(&terminator) {
                    for _ in 0..hashes {
                        let _ = chars.next();
                        out.push(' ');
                    }
                    break;
                }
                if raw_ch == '\n' {
                    break;
                }
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(' ');
            continue;
        }

        if ch == '\'' && looks_like_char_literal(line, idx) {
            in_char = true;
            out.push(' ');
            continue;
        }

        out.push(ch);
    }

    out
}

fn raw_string_hashes(line: &str, idx: usize) -> Option<usize> {
    let rest = line.get(idx..)?;
    let mut chars = rest.chars();
    if chars.next()? != 'r' {
        return None;
    }
    let mut hashes = 0usize;
    for ch in chars {
        match ch {
            '#' => hashes += 1,
            '"' => return Some(hashes),
            _ => return None,
        }
    }
    None
}

fn looks_like_char_literal(line: &str, idx: usize) -> bool {
    let Some(rest) = line.get(idx + '\''.len_utf8()..) else {
        return false;
    };
    let mut escaped = false;
    for (offset, ch) in rest.char_indices() {
        if offset > 8 {
            return false;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '\'' {
            return true;
        }
        if ch.is_whitespace() {
            return false;
        }
    }
    false
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
    let declaration = declaration_prefix(&compact);
    match fact.kind.as_str() {
        "FN" if declaration.contains("unsafe fn") => {
            Some((UnsafeSiteKind::UnsafeFn, OperationFamily::Unknown))
        }
        "TRAIT" if declaration.contains("unsafe trait") => {
            Some((UnsafeSiteKind::UnsafeTrait, OperationFamily::Unknown))
        }
        "IMPL" if declaration.contains("unsafe impl") && declaration.contains(" Send") => Some((
            UnsafeSiteKind::UnsafeImplSend,
            OperationFamily::UnsafeImplSendSync,
        )),
        "IMPL" if declaration.contains("unsafe impl") && declaration.contains(" Sync") => Some((
            UnsafeSiteKind::UnsafeImplSync,
            OperationFamily::UnsafeImplSendSync,
        )),
        "IMPL" if declaration.contains("unsafe impl") => {
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

fn declaration_prefix(compact: &str) -> &str {
    compact
        .split_once('{')
        .map_or(compact, |(declaration, _body)| declaration.trim())
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
    use std::{env, fs, time::SystemTime};

    #[test]
    fn masks_unsafe_keywords_inside_non_code_segments() {
        let masked = mask_non_code_segments(
            "let text = \"unsafe { ptr::read(x) }\"; // unsafe fn fake() {}",
        );

        assert!(!masked.contains("unsafe"));
        assert!(!masked.contains("ptr::read"));
        assert!(masked.contains("let text ="));
    }

    #[test]
    fn masks_raw_strings_before_line_fallback_detection() {
        let masked = mask_non_code_segments("let text = r#\"unsafe { core::ptr::read(ptr) }\"#;");

        assert_eq!(detect_site(masked.trim()), None);
    }

    #[test]
    fn scan_file_ignores_unsafe_text_in_literals_and_comments() -> Result<(), String> {
        let root = temp_fixture_dir()?;
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).map_err(|err| err.to_string())?;
        fs::write(
            src_dir.join("lib.rs"),
            r##"
pub fn describe() -> &'static str {
    "unsafe { core::ptr::read(ptr) }"
}

pub fn raw_describe() -> &'static str {
    r#"pub unsafe fn fake() {}"# // unsafe impl Send for Fake {}
}
"##,
        )
        .map_err(|err| err.to_string())?;

        let sites = scan_file(&root, &PathBuf::from("src/lib.rs"), None, true)?;

        assert!(
            sites.is_empty(),
            "unexpected false-positive sites: {sites:#?}"
        );
        fs::remove_dir_all(root).map_err(|err| err.to_string())?;
        Ok(())
    }

    fn temp_fixture_dir() -> Result<PathBuf, String> {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|err| err.to_string())?
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "unsafe-review-scanner-test-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).map_err(|err| err.to_string())?;
        Ok(path)
    }
}
