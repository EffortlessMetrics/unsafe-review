use crate::domain::{OperationFamily, SourceLocation, UnsafeOperation, UnsafeSite, UnsafeSiteKind};
use crate::input::diff::DiffIndex;
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
    let mut out = Vec::new();
    for (idx, raw) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        let Some((kind, family)) = detect_site(trimmed) else {
            continue;
        };
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
