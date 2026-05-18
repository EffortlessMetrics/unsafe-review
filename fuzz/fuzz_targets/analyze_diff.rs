#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;
use unsafe_review_core::{
    AnalysisMode, AnalyzeInput, DiffSource, PolicyMode, Scope, analyze, collect_context,
    explain_card, render_human, render_json, render_markdown,
};

const MAX_INPUT_BYTES: usize = 16 * 1024;
const MAX_SYNTHETIC_LINES: usize = 256;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data.get(..data.len().min(MAX_INPUT_BYTES)).unwrap_or(data))
    else {
        return;
    };

    let diff = diff_input(input);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/raw_pointer_alignment");
    let analysis = AnalyzeInput {
        root,
        scope: Scope::Diff,
        diff: DiffSource::Text(diff),
        mode: AnalysisMode::Draft,
        policy: PolicyMode::Advisory,
        include_unchanged_tests: true,
        max_cards: Some(16),
    };

    let Ok(output) = analyze(analysis) else {
        return;
    };

    let _json = render_json(&output);
    let _human = render_human(&output);
    let _markdown = render_markdown(&output);
    for card in &output.cards {
        let _detail = explain_card(&output, &card.id);
        let _packet = collect_context(&output, &card.id);
    }
});

fn diff_input(input: &str) -> String {
    if input.contains("+++ b/") && input.contains("@@") {
        input.to_string()
    } else {
        synthetic_diff(input)
    }
}

fn synthetic_diff(payload: &str) -> String {
    let mut diff = String::from(
        "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1,0 +1,0 @@\n",
    );
    for line in payload.lines().take(MAX_SYNTHETIC_LINES) {
        diff.push('+');
        diff.push_str(line);
        diff.push('\n');
    }
    diff
}
