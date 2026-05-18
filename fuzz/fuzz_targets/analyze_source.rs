#![no_main]

use libfuzzer_sys::fuzz_target;
use std::fs;
use unsafe_review_core::{
    AnalysisMode, AnalyzeInput, DiffSource, PolicyMode, Scope, analyze, collect_context,
    explain_card, render_human, render_json, render_markdown,
};

const MAX_SOURCE_BYTES: usize = 16 * 1024;

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_SOURCE_BYTES {
        return;
    }
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if source.contains('\0') {
        return;
    }
    exercise_analyzer(source);
});

fn exercise_analyzer(source: &str) {
    let Ok(root) = tempfile::tempdir() else {
        return;
    };
    let src_dir = root.path().join("src");
    if fs::create_dir(&src_dir).is_err() {
        return;
    }
    if fs::write(root.path().join("Cargo.toml"), fixture_manifest()).is_err() {
        return;
    }
    if fs::write(src_dir.join("lib.rs"), source).is_err() {
        return;
    }

    let input = AnalyzeInput {
        root: root.path().to_path_buf(),
        scope: Scope::Repo,
        diff: DiffSource::NoneRepoScan,
        mode: AnalysisMode::Repo,
        policy: PolicyMode::Advisory,
        include_unchanged_tests: true,
        max_cards: Some(64),
    };

    let Ok(output) = analyze(input) else {
        return;
    };

    let rendered_json = render_json(&output);
    let _parsed_json: Result<serde_json::Value, _> = serde_json::from_str(&rendered_json);
    let _human = render_human(&output);
    let _markdown = render_markdown(&output);

    for card in output.cards.iter().take(8) {
        let _detail = explain_card(&output, &card.id);
        let _packet = collect_context(&output, &card.id);
    }
}

fn fixture_manifest() -> &'static str {
    r#"[package]
name = "unsafe-review-fuzz-fixture"
version = "0.0.0"
edition = "2024"
publish = false
"#
}
