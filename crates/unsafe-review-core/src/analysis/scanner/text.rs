use crate::domain::UnsafeSiteKind;

pub(super) fn first_non_ws_column(line: &str) -> usize {
    line.chars()
        .position(|ch| !ch.is_whitespace())
        .map_or(1, |pos| pos + 1)
}

pub(super) fn context_slice(lines: &[&str], start: usize, end: usize) -> Vec<String> {
    lines[start..end]
        .iter()
        .map(|line| line.trim().to_string())
        .collect()
}

pub(super) fn compact_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn normalize_call_spacing(text: &str) -> String {
    text.replace(" (", "(")
}

pub(super) fn is_public_surface(snippet: &str) -> bool {
    let compact = compact_whitespace(snippet);
    compact.starts_with("pub ") || compact.contains(" pub ")
}

pub(super) fn is_public_api_surface(kind: &UnsafeSiteKind, snippet: &str) -> bool {
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

pub(super) fn find_owner(lines: &[&str], idx: usize) -> Option<String> {
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

pub(super) fn parse_fn_name(line: &str) -> Option<String> {
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
