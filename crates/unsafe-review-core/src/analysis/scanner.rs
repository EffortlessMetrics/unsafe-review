mod detection;
mod text;

use self::detection::{detect_site, detect_syntax_sites, operation_block_start_lines};
use self::text::{
    context_slice, find_owner, first_non_ws_column, is_public_api_surface, is_public_surface,
    parse_fn_name,
};
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
        if trimmed.starts_with("//") {
            continue;
        }
        let Some((kind, family)) = detect_site(trimmed) else {
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
        let visibility = if is_public_surface(trimmed) {
            "public"
        } else {
            "private"
        }
        .to_string();
        let public_api_surface = is_public_api_surface(&kind, trimmed);
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

fn site_key(
    line: usize,
    kind: &UnsafeSiteKind,
    family: &OperationFamily,
) -> (usize, String, String) {
    (line, kind.as_str().to_string(), family.as_str().to_string())
}
