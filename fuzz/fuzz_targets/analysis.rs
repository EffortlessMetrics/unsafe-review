#![no_main]

use libfuzzer_sys::fuzz_target;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use unsafe_review_core::{
    AnalysisMode, AnalyzeInput, DiffSource, PolicyMode, Scope, analyze, render_human, render_json,
    render_markdown,
};

const MAX_INPUT_BYTES: usize = 48 * 1024;
static WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES || data.is_empty() {
        return;
    }

    let input = AnalysisFuzzInput::from_bytes(data);
    let Some(workspace) = FuzzWorkspace::new("analysis") else {
        return;
    };
    if !write_fuzz_workspace(workspace.path(), &input.source) {
        return;
    }

    let diff = if input.use_repo_scan {
        DiffSource::NoneRepoScan
    } else {
        DiffSource::Text(input.diff.into_owned())
    };
    let scope = if input.use_repo_scan {
        Scope::Repo
    } else {
        Scope::Diff
    };
    let mode = if input.use_repo_scan {
        AnalysisMode::Repo
    } else {
        AnalysisMode::Draft
    };

    if let Ok(output) = analyze(AnalyzeInput {
        root: workspace.path().to_path_buf(),
        scope,
        diff,
        mode,
        policy: PolicyMode::Advisory,
        include_unchanged_tests: true,
        max_cards: input.max_cards,
    }) {
        let json = render_json(&output);
        if serde_json::from_str::<serde_json::Value>(&json).is_err() {
            panic!("render_json produced invalid JSON");
        }
        if input.render_human_output {
            let _ = render_human(&output);
        }
        if input.render_markdown_output {
            let _ = render_markdown(&output);
        }
    }
});

struct AnalysisFuzzInput<'a> {
    source: Cow<'a, str>,
    diff: Cow<'a, str>,
    use_repo_scan: bool,
    render_human_output: bool,
    render_markdown_output: bool,
    max_cards: Option<usize>,
}

impl<'a> AnalysisFuzzInput<'a> {
    fn from_bytes(data: &'a [u8]) -> Self {
        let flags = data[0];
        let payload = &data[1..];
        let split = payload
            .iter()
            .position(|byte| *byte == 0xff)
            .unwrap_or(payload.len());
        let (source, diff_with_separator) = payload.split_at(split);
        let diff = diff_with_separator.get(1..).unwrap_or_default();
        let max_cards = if flags & 0b0000_1000 == 0 {
            None
        } else {
            Some(usize::from((flags >> 4).max(1)))
        };
        Self {
            source: String::from_utf8_lossy(source),
            diff: String::from_utf8_lossy(diff),
            use_repo_scan: flags & 0b0000_0001 != 0,
            render_human_output: flags & 0b0000_0010 != 0,
            render_markdown_output: flags & 0b0000_0100 != 0,
            max_cards,
        }
    }
}

fn write_fuzz_workspace(root: &Path, source: &str) -> bool {
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).is_ok() && fs::write(src_dir.join("lib.rs"), source).is_ok()
}

struct FuzzWorkspace {
    path: PathBuf,
}

impl FuzzWorkspace {
    fn new(name: &str) -> Option<Self> {
        let id = WORKSPACE_ID.fetch_add(1, Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "unsafe-review-fuzz-{name}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).ok()?;
        Some(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for FuzzWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
