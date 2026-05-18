#![forbid(unsafe_code)]
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "check-pr",
        description: "run every repository policy check used by PR CI",
    },
    CommandSpec {
        name: "check-docs",
        description: "verify required docs, indexes, and path spelling",
    },
    CommandSpec {
        name: "check-policy",
        description: "validate policy TOML files and required schema metadata",
    },
    CommandSpec {
        name: "check-support-tiers",
        description: "validate support-tier status table values",
    },
    CommandSpec {
        name: "check-fixtures",
        description: "validate fixture directories and expected card JSON files",
    },
];

struct CommandSpec {
    name: &'static str,
    description: &'static str,
}

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
    let root = workspace_root()?;
    std::env::set_current_dir(&root)
        .map_err(|err| format!("failed to enter workspace root {}: {err}", root.display()))?;

    let command = args.get(1).map_or("help", String::as_str);
    match command {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "check-pr" => {
            require_no_extra_args(&args, command)?;
            check_docs()?;
            check_policy()?;
            check_support_tiers()?;
            check_fixtures()?;
            check_tracked_generated_artifacts()?;
            println!("check-pr: ok");
            Ok(())
        }
        "check-docs" => {
            require_no_extra_args(&args, command)?;
            check_docs()
        }
        "check-policy" => {
            require_no_extra_args(&args, command)?;
            check_policy()
        }
        "check-support-tiers" => {
            require_no_extra_args(&args, command)?;
            check_support_tiers()
        }
        "check-fixtures" => {
            require_no_extra_args(&args, command)?;
            check_fixtures()
        }
        other => Err(format!(
            "unknown xtask command `{other}`\n\nRun `cargo xtask help` for available commands."
        )),
    }
}

fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "failed to resolve workspace root from xtask manifest path".to_string())
}

fn print_help() {
    println!("Repository automation commands:\n");
    for command in COMMANDS {
        println!("  {:<20} {}", command.name, command.description);
    }
    println!("\nRun with `cargo xtask <command>` from any directory in the workspace.");
}

fn require_no_extra_args(args: &[String], command: &str) -> Result<(), String> {
    if args.len() <= 2 {
        return Ok(());
    }
    Err(format!(
        "`{command}` does not accept arguments: {}",
        args[2..].join(" ")
    ))
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

fn check_fixtures() -> Result<(), String> {
    let root = Path::new("fixtures");
    if !root.is_dir() {
        return Err("fixtures directory is missing".to_string());
    }

    let mut fixture_count = 0usize;
    let entries = fs::read_dir(root).map_err(|err| format!("read fixtures failed: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read fixtures entry failed: {err}"))?;
        let path = entry.path();
        if !path.is_dir() {
            return Err(format!(
                "fixtures may only contain fixture directories: {}",
                path.display()
            ));
        }
        fixture_count += 1;
        check_fixture_dir(&path)?;
    }

    if fixture_count == 0 {
        return Err("fixtures directory has no fixture cases".to_string());
    }

    println!("check-fixtures: ok");
    Ok(())
}

fn check_fixture_dir(dir: &Path) -> Result<(), String> {
    for required in [
        "Cargo.toml",
        "change.diff",
        "expected.cards.json",
        "src/lib.rs",
    ] {
        require_file_path(&dir.join(required))?;
    }
    parse_json_file(&dir.join("expected.cards.json"))?;
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

fn parse_json_file(path: &Path) -> Result<(), String> {
    let text = read_to_string(path)?;
    let value = text
        .parse::<serde_json::Value>()
        .map_err(|err| format!("{} is not valid JSON: {err}", path.display()))?;
    if matches!(value, serde_json::Value::Array(_)) {
        Ok(())
    } else {
        Err(format!("{} must contain a JSON array", path.display()))
    }
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

fn require_file(path: &str) -> Result<(), String> {
    require_file_path(Path::new(path))
}

fn require_file_path(path: &Path) -> Result<(), String> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("required file missing: {}", path.display()))
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
    fn extra_arg_validator_rejects_trailing_values() -> Result<(), String> {
        let args = vec![
            "xtask".to_string(),
            "check-docs".to_string(),
            "unexpected".to_string(),
        ];
        let Err(err) = require_no_extra_args(&args, "check-docs") else {
            return Err("extra argument should be rejected".to_string());
        };
        assert!(err.contains("unexpected"));
        require_no_extra_args(&args[..2], "check-docs")?;
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
