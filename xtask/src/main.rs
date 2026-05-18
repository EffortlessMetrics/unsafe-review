#![forbid(unsafe_code)]
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REQUIRED_DOCS: &[&str] = &[
    "README.md",
    "docs/MISSION.md",
    "docs/ROADMAP.md",
    "docs/specs/README.md",
    "docs/status/SUPPORT_TIERS.md",
];

const POLICY_FILES: &[&str] = &[
    "policy/unsafe-review.toml",
    "policy/unsafe-review-baseline.toml",
    "policy/unsafe-review-suppressions.toml",
    "policy/clippy-lints.toml",
    "policy/clippy-exceptions.toml",
    "policy/no-panic-allowlist.toml",
    "policy/non-rust-allowlist.toml",
    "policy/generated-allowlist.toml",
    "policy/executable-allowlist.toml",
    "policy/workflow-allowlist.toml",
    "policy/process-allowlist.toml",
    "policy/network-allowlist.toml",
];

const FIXTURE_REQUIRED_FILES: &[&str] = &["Cargo.toml", "change.diff", "src/lib.rs"];

const FIXTURE_EXPECTED_CARDS_EXCEPTIONS: &[&str] = &[
    "duplicate_raw_pointer_reads",
    "raw_pointer_alignment_line_drift",
];

const FIXTURE_PACKAGE_PREFIX_EXCEPTIONS: &[(&str, &str)] =
    &[("raw_pointer_alignment_line_drift", "raw-pointer-alignment")];

const KNOWN_SUPPORT_TIERS: &[&str] = &["scaffold", "experimental", "planned", "deferred"];

fn main() {
    if let Err(err) = run(std::env::args().collect()) {
        eprintln!("xtask: {err}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.get(1).map(|arg| arg.as_str()) {
        None | Some("help") | Some("--help") => {
            println!(
                "xtask commands: check-pr, check-docs, check-policy, check-support-tiers, check-fixtures"
            );
            Ok(())
        }
        Some("check-pr") => {
            check_docs()?;
            check_policy()?;
            check_support_tiers()?;
            check_fixtures()?;
            check_tracked_generated_artifacts()?;
            println!("check-pr: ok");
            Ok(())
        }
        Some("check-docs") => check_docs(),
        Some("check-policy") => check_policy(),
        Some("check-support-tiers") => check_support_tiers(),
        Some("check-fixtures") => check_fixtures(),
        Some(other) => Err(format!("unknown xtask command `{other}`")),
    }
}

fn check_docs() -> Result<(), String> {
    for path in REQUIRED_DOCS {
        require_file(path)?;
    }
    check_index(
        Path::new("docs/specs"),
        Path::new("docs/specs/README.md"),
        "UNSAFE-REVIEW-SPEC-",
    )?;
    check_index(
        Path::new("docs/adr"),
        Path::new("docs/adr/README.md"),
        "UNSAFE-REVIEW-ADR-",
    )?;
    check_index(
        Path::new("docs/proposals"),
        Path::new("docs/proposals/README.md"),
        "UNSAFE-REVIEW-PROP-",
    )?;
    check_no_windows_paths(&[
        Path::new("README.md"),
        Path::new("MANIFEST.md"),
        Path::new("docs"),
        Path::new("plans"),
        Path::new(".unsafe-review"),
        Path::new("policy"),
    ])?;
    println!("check-docs: ok");
    Ok(())
}

fn check_policy() -> Result<(), String> {
    for path in POLICY_FILES {
        let value = parse_toml_file(Path::new(path))?;
        require_toml_string(&value, "schema_version", path)?;
    }
    parse_toml_file(Path::new(".unsafe-review/goals/active.toml"))?;
    println!("check-policy: ok");
    Ok(())
}

fn check_support_tiers() -> Result<(), String> {
    let path = "docs/status/SUPPORT_TIERS.md";
    let text = read_to_string(Path::new(path))?;
    let mut rows = 0usize;
    for (line_no, line) in text.lines().enumerate() {
        let Some(tier) = support_tier_from_row(line) else {
            continue;
        };
        rows += 1;
        if !KNOWN_SUPPORT_TIERS.contains(&tier) {
            return Err(format!(
                "{path}:{} uses unknown support tier `{tier}`",
                line_no + 1
            ));
        }
    }
    if rows == 0 {
        return Err(format!("{path} has no support-tier rows"));
    }
    println!("check-support-tiers: ok");
    Ok(())
}

fn check_fixtures() -> Result<(), String> {
    let dirs = fixture_dirs(Path::new("fixtures"))?;
    if dirs.is_empty() {
        return Err("fixtures directory has no fixture cases".to_string());
    }
    for dir in &dirs {
        check_fixture(dir)?;
    }
    println!("check-fixtures: ok ({} fixtures)", dirs.len());
    Ok(())
}

fn check_fixture(dir: &Path) -> Result<(), String> {
    let name = fixture_dir_name(dir)?;
    if !is_snake_case_name(name) {
        return Err(format!(
            "{} must use a lowercase snake_case fixture name",
            dir.display()
        ));
    }

    for relative in FIXTURE_REQUIRED_FILES {
        require_fixture_file(dir, relative)?;
    }

    let cargo = parse_toml_file(&dir.join("Cargo.toml"))?;
    let package_name = cargo
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("{}/Cargo.toml is missing package.name", dir.display()))?;
    let expected_prefix = fixture_package_prefix(name);
    if !package_name.starts_with(&expected_prefix) {
        return Err(format!(
            "{}/Cargo.toml package.name `{package_name}` does not start with `{expected_prefix}`",
            dir.display()
        ));
    }

    let expected_cards = dir.join("expected.cards.json");
    if expected_cards.is_file() {
        let expected_cards = parse_json_file(&expected_cards)?;
        if !expected_cards.is_array() {
            return Err(format!(
                "{}/expected.cards.json must contain a JSON array of cards",
                dir.display()
            ));
        }
    } else if !FIXTURE_EXPECTED_CARDS_EXCEPTIONS.contains(&name) {
        return Err(format!(
            "fixture {} is missing expected.cards.json",
            dir.display()
        ));
    }

    let diff = read_to_string(&dir.join("change.diff"))?;
    if !looks_like_git_diff(&diff) {
        return Err(format!(
            "{}/change.diff does not look like a unified git diff",
            dir.display()
        ));
    }

    Ok(())
}

fn check_index(dir: &Path, readme: &Path, prefix: &str) -> Result<(), String> {
    let index = read_to_string(readme)?;
    let mut ids = BTreeSet::new();
    for path in markdown_files(dir)? {
        if path == readme {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return Err(format!("non-UTF-8 file name in {}", dir.display()));
        };
        if !name.starts_with(prefix) {
            return Err(format!(
                "{} does not use expected `{prefix}` prefix",
                path.display()
            ));
        }
        let id = name.trim_end_matches(".md");
        if !ids.insert(id.to_string()) {
            return Err(format!("duplicate source-of-truth id `{id}`"));
        }
        if !index.contains(name) {
            return Err(format!(
                "{} is missing from {}",
                path.display(),
                readme.display()
            ));
        }
    }
    Ok(())
}

fn check_no_windows_paths(paths: &[&Path]) -> Result<(), String> {
    for path in paths {
        visit_text(path, &mut |file| {
            let text = read_to_string(file)?;
            for (line_no, line) in text.lines().enumerate() {
                if has_windows_path(line) {
                    return Err(format!(
                        "{}:{} contains a Windows-style path",
                        file.display(),
                        line_no + 1
                    ));
                }
            }
            Ok(())
        })?;
    }
    Ok(())
}

fn check_tracked_generated_artifacts() -> Result<(), String> {
    let output = Command::new("git")
        .args(["ls-files"])
        .output()
        .map_err(|err| format!("failed to run git ls-files: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    for path in String::from_utf8_lossy(&output.stdout).lines() {
        if is_forbidden_generated_path(path) {
            return Err(format!("generated artifact is tracked: {path}"));
        }
    }
    Ok(())
}

fn parse_toml_file(path: &Path) -> Result<toml::Value, String> {
    let text = read_to_string(path)?;
    text.parse::<toml::Value>()
        .map_err(|err| format!("{} is not valid TOML: {err}", path.display()))
}

fn parse_json_file(path: &Path) -> Result<serde_json::Value, String> {
    let text = read_to_string(path)?;
    serde_json::from_str(&text)
        .map_err(|err| format!("{} is not valid JSON: {err}", path.display()))
}

fn require_toml_string(value: &toml::Value, key: &str, path: &str) -> Result<(), String> {
    match value.get(key).and_then(toml::Value::as_str) {
        Some(_) => Ok(()),
        None => Err(format!("{path} is missing string key `{key}`")),
    }
}

fn require_file(path: &str) -> Result<(), String> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err(format!("required file missing: {path}"))
    }
}

fn require_fixture_file(dir: &Path, relative: &str) -> Result<(), String> {
    let path = dir.join(relative);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("fixture {} is missing {relative}", dir.display()))
    }
}

fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))
}

fn fixture_dirs(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|err| format!("read {} failed: {err}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read_dir entry failed: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    visit_text(dir, &mut |path| {
        if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path.to_path_buf());
        }
        Ok(())
    })?;
    files.sort();
    Ok(files)
}

fn visit_text(
    dir_or_file: &Path,
    f: &mut impl FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    if dir_or_file.is_file() {
        if is_text_file(dir_or_file) {
            f(dir_or_file)?;
        }
        return Ok(());
    }
    let entries = fs::read_dir(dir_or_file)
        .map_err(|err| format!("read {} failed: {err}", dir_or_file.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read_dir entry failed: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            visit_text(&path, f)?;
        } else if is_text_file(&path) {
            f(&path)?;
        }
    }
    Ok(())
}

fn is_text_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| matches!(ext.to_str(), Some("md" | "toml" | "yml" | "yaml" | "txt")))
}

fn support_tier_from_row(line: &str) -> Option<&str> {
    if !line.starts_with('|') || line.contains("---") || line.contains("Capability") {
        return None;
    }
    let columns = line
        .split('|')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .collect::<Vec<_>>();
    columns.get(1).copied()
}

fn fixture_dir_name(path: &Path) -> Result<&str, String> {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| format!("{} has a non-UTF-8 fixture directory name", path.display()))
}

fn fixture_package_prefix(name: &str) -> String {
    FIXTURE_PACKAGE_PREFIX_EXCEPTIONS
        .iter()
        .find_map(|(fixture, package_prefix)| (*fixture == name).then_some(*package_prefix))
        .unwrap_or(name)
        .replace('_', "-")
}

fn is_snake_case_name(name: &str) -> bool {
    let mut previous_underscore = false;
    let mut has_segment_char = false;
    for ch in name.chars() {
        match ch {
            'a'..='z' | '0'..='9' => {
                previous_underscore = false;
                has_segment_char = true;
            }
            '_' if has_segment_char && !previous_underscore => {
                previous_underscore = true;
            }
            _ => return false,
        }
    }
    has_segment_char && !previous_underscore
}

fn looks_like_git_diff(text: &str) -> bool {
    text.contains("diff --git") && text.contains("--- a/") && text.contains("+++ b/")
}

fn has_windows_path(line: &str) -> bool {
    line.contains(":\\") || line.contains("\\\\")
}

fn is_forbidden_generated_path(path: &str) -> bool {
    path.starts_with("target/")
        || path.starts_with("badges/")
        || path.ends_with(".sarif")
        || path.ends_with(".profraw")
        || path.ends_with(".profdata")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_tier_parser_reads_tier_column() {
        assert_eq!(
            support_tier_from_row("| Review cards | scaffold | CLI | proof | limit |"),
            Some("scaffold")
        );
        assert_eq!(support_tier_from_row("|---|---|"), None);
    }

    #[test]
    fn fixture_names_must_be_snake_case() {
        assert!(is_snake_case_name("raw_pointer_deref"));
        assert!(is_snake_case_name("ffi_sanitizer_route"));
        assert!(!is_snake_case_name("RawPointerDeref"));
        assert!(!is_snake_case_name("raw-pointer-deref"));
        assert!(!is_snake_case_name("raw_pointer_deref_"));
        assert!(!is_snake_case_name("_raw_pointer_deref"));
    }

    #[test]
    fn fixture_dir_name_reads_last_path_component() -> Result<(), String> {
        assert_eq!(
            fixture_dir_name(Path::new("fixtures/raw_pointer_deref"))?,
            "raw_pointer_deref"
        );
        Ok(())
    }

    #[test]
    fn expected_card_exceptions_are_exact_fixture_names() {
        assert!(FIXTURE_EXPECTED_CARDS_EXCEPTIONS.contains(&"duplicate_raw_pointer_reads"));
        assert!(FIXTURE_EXPECTED_CARDS_EXCEPTIONS.contains(&"raw_pointer_alignment_line_drift"));
        assert!(!FIXTURE_EXPECTED_CARDS_EXCEPTIONS.contains(&"raw_pointer_alignment"));
    }

    #[test]
    fn fixture_package_prefix_can_preserve_identity_fixture_package() {
        assert_eq!(
            fixture_package_prefix("raw_pointer_alignment_line_drift"),
            "raw-pointer-alignment"
        );
        assert_eq!(
            fixture_package_prefix("duplicate_raw_pointer_reads"),
            "duplicate-raw-pointer-reads"
        );
    }

    #[test]
    fn git_diff_shape_requires_file_headers() {
        assert!(looks_like_git_diff(
            "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n"
        ));
        assert!(!looks_like_git_diff(
            "diff --git a/src/lib.rs b/src/lib.rs\n"
        ));
    }

    #[test]
    fn generated_artifact_detector_is_narrow() {
        assert!(is_forbidden_generated_path("target/debug/tool.exe"));
        assert!(is_forbidden_generated_path("reports/cards.sarif"));
        assert!(!is_forbidden_generated_path("Cargo.lock"));
        assert!(!is_forbidden_generated_path("docs/status/SUPPORT_TIERS.md"));
    }
}
