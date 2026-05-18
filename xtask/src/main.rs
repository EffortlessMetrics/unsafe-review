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
            println!("xtask commands: check-pr, check-docs, check-policy, check-support-tiers");
            Ok(())
        }
        Some("check-pr") => {
            check_docs()?;
            check_policy()?;
            check_support_tiers()?;
            check_tracked_generated_artifacts()?;
            println!("check-pr: ok");
            Ok(())
        }
        Some("check-docs") => check_docs(),
        Some("check-policy") => check_policy(),
        Some("check-support-tiers") => check_support_tiers(),
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
    check_workflow_policy()?;
    parse_toml_file(Path::new(".unsafe-review/goals/active.toml"))?;
    println!("check-policy: ok");
    Ok(())
}

fn check_workflow_policy() -> Result<(), String> {
    let policy_path = "policy/workflow-allowlist.toml";
    let workflow_path = ".github/workflows/ci.yml";
    let policy = parse_toml_file(Path::new(policy_path))?;
    let workflow = read_to_string(Path::new(workflow_path))?;

    let allowed_actions = require_toml_string_array(&policy, "allowed_actions", policy_path)?;
    for action in workflow_actions(&workflow) {
        if !allowed_actions.iter().any(|allowed| allowed == action) {
            return Err(format!(
                "{workflow_path} uses an action not listed in {policy_path}: `{action}`"
            ));
        }
    }
    for action in &allowed_actions {
        if !workflow.contains(&format!("uses: {action}")) {
            return Err(format!(
                "{workflow_path} is missing allowed action from {policy_path}: `{action}`"
            ));
        }
    }

    for command in require_toml_string_array(&policy, "required_commands", policy_path)? {
        if !workflow.contains(&format!("run: {command}")) {
            return Err(format!(
                "{workflow_path} is missing required command from {policy_path}: `{command}`"
            ));
        }
    }

    if !workflow.contains("timeout-minutes:") {
        return Err(format!("{workflow_path} is missing a job timeout"));
    }
    if !workflow.contains("cancel-in-progress: true") {
        return Err(format!(
            "{workflow_path} is missing concurrency cancellation"
        ));
    }

    Ok(())
}

fn workflow_actions(workflow: &str) -> Vec<&str> {
    workflow
        .lines()
        .filter_map(|line| line.trim().split_once("uses: ").map(|(_, action)| action))
        .collect()
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

fn require_toml_string(value: &toml::Value, key: &str, path: &str) -> Result<(), String> {
    match value.get(key).and_then(toml::Value::as_str) {
        Some(_) => Ok(()),
        None => Err(format!("{path} is missing string key `{key}`")),
    }
}

fn require_toml_string_array(
    value: &toml::Value,
    key: &str,
    path: &str,
) -> Result<Vec<String>, String> {
    let Some(items) = value.get(key).and_then(toml::Value::as_array) else {
        return Err(format!("{path} is missing array key `{key}`"));
    };
    if items.is_empty() {
        return Err(format!("{path} array key `{key}` must not be empty"));
    }
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{path} array key `{key}` contains a non-string value"))
        })
        .collect()
}

fn require_file(path: &str) -> Result<(), String> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err(format!("required file missing: {path}"))
    }
}

fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))
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
    fn workflow_action_parser_reads_uses_lines() {
        assert_eq!(
            workflow_actions("steps:\n  - uses: actions/checkout@v6\n"),
            vec!["actions/checkout@v6"]
        );
    }

    #[test]
    fn workflow_policy_lists_required_ci_contract() -> Result<(), String> {
        let policy_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| "xtask manifest path has no parent".to_string())?
            .join("policy/workflow-allowlist.toml");
        let policy = parse_toml_file(&policy_path)?;
        let actions = require_toml_string_array(
            &policy,
            "allowed_actions",
            "policy/workflow-allowlist.toml",
        )?;
        let commands = require_toml_string_array(
            &policy,
            "required_commands",
            "policy/workflow-allowlist.toml",
        )?;

        assert!(
            actions
                .iter()
                .any(|action| action == "Swatinem/rust-cache@v2")
        );
        assert!(commands.iter().all(|command| command.starts_with("cargo ")));
        Ok(())
    }

    #[test]
    fn generated_artifact_detector_is_narrow() {
        assert!(is_forbidden_generated_path("target/debug/tool.exe"));
        assert!(is_forbidden_generated_path("reports/cards.sarif"));
        assert!(!is_forbidden_generated_path("Cargo.lock"));
        assert!(!is_forbidden_generated_path("docs/status/SUPPORT_TIERS.md"));
    }
}
