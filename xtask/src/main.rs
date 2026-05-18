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

const FIXTURE_REQUIRED_FILES: &[&str] = &[
    "Cargo.toml",
    "change.diff",
    "expected.cards.json",
    "src/lib.rs",
];

struct XtaskCommand {
    name: &'static str,
    summary: &'static str,
}

const COMMANDS: &[XtaskCommand] = &[
    XtaskCommand {
        name: "check-pr",
        summary: "run every repository invariant enforced by PR CI",
    },
    XtaskCommand {
        name: "check-docs",
        summary: "validate required docs, doc indexes, and portable paths",
    },
    XtaskCommand {
        name: "check-policy",
        summary: "validate checked-in policy TOML files",
    },
    XtaskCommand {
        name: "check-support-tiers",
        summary: "validate docs/status/SUPPORT_TIERS.md tier names",
    },
    XtaskCommand {
        name: "check-fixtures",
        summary: "validate fixture layout and golden review-card JSON",
    },
];

fn main() {
    if let Err(err) = run(std::env::args().collect()) {
        eprintln!("xtask: {err}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.get(1).map(String::as_str) else {
        print_help();
        return Ok(());
    };

    if matches!(command, "help" | "--help" | "-h") {
        print_help();
        return Ok(());
    }

    if args.len() > 2 {
        return Err(format!(
            "command `{command}` does not accept extra arguments: {}",
            args[2..].join(" ")
        ));
    }

    match command {
        "check-pr" => {
            check_docs()?;
            check_policy()?;
            check_support_tiers()?;
            check_fixtures()?;
            check_tracked_generated_artifacts()?;
            println!("check-pr: ok");
            Ok(())
        }
        "check-docs" => check_docs(),
        "check-policy" => check_policy(),
        "check-support-tiers" => check_support_tiers(),
        "check-fixtures" => check_fixtures(),
        other => Err(format!(
            "unknown xtask command `{other}`\nrun `cargo xtask --help` for available commands"
        )),
    }
}

fn print_help() {
    println!("Repository automation commands:\n");
    println!("Usage: cargo xtask <command>\n");
    println!("Commands:");
    for command in COMMANDS {
        println!("  {:<20} {}", command.name, command.summary);
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
    let root = Path::new("fixtures");
    let entries =
        fs::read_dir(root).map_err(|err| format!("read {} failed: {err}", root.display()))?;
    let mut fixture_count = 0usize;

    for entry in entries {
        let entry = entry.map_err(|err| format!("read_dir entry failed: {err}"))?;
        let fixture = entry.path();
        if !fixture.is_dir() {
            continue;
        }
        fixture_count += 1;
        check_fixture(&fixture)?;
    }

    if fixture_count == 0 {
        return Err(format!("{} has no fixture directories", root.display()));
    }

    println!("check-fixtures: ok");
    Ok(())
}

fn check_fixture(fixture: &Path) -> Result<(), String> {
    let fixture_name = fixture
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("non-UTF-8 fixture path {}", fixture.display()))?;

    for required in FIXTURE_REQUIRED_FILES {
        let path = fixture.join(required);
        if !path.is_file() {
            return Err(format!(
                "fixture `{fixture_name}` is missing required file {}",
                path.display()
            ));
        }
    }

    let manifest = parse_toml_file(&fixture.join("Cargo.toml"))?;
    require_toml_string(
        &manifest,
        "package.name",
        &fixture.join("Cargo.toml").display().to_string(),
    )?;

    let expected_cards = fixture.join("expected.cards.json");
    let text = read_to_string(&expected_cards)?;
    let value = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|err| format!("{} is not valid JSON: {err}", expected_cards.display()))?;
    if !value.is_array() {
        return Err(format!(
            "{} must contain a JSON array of review cards",
            expected_cards.display()
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

fn require_toml_string(value: &toml::Value, key: &str, path: &str) -> Result<(), String> {
    let mut current = value;
    for part in key.split('.') {
        let Some(next) = current.get(part) else {
            return Err(format!("{path} is missing string key `{key}`"));
        };
        current = next;
    }

    if current.as_str().is_some() {
        Ok(())
    } else {
        Err(format!("{path} is missing string key `{key}`"))
    }
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
    fn generated_artifact_detector_is_narrow() {
        assert!(is_forbidden_generated_path("target/debug/tool.exe"));
        assert!(is_forbidden_generated_path("reports/cards.sarif"));
        assert!(!is_forbidden_generated_path("Cargo.lock"));
        assert!(!is_forbidden_generated_path("docs/status/SUPPORT_TIERS.md"));
    }

    #[test]
    fn require_toml_string_reads_dotted_paths() {
        let value = toml::Value::Table(toml::Table::from_iter([(
            "package".to_string(),
            toml::Value::Table(toml::Table::from_iter([(
                "name".to_string(),
                "fixture".into(),
            )])),
        )]));

        assert_eq!(
            require_toml_string(&value, "package.name", "Cargo.toml"),
            Ok(())
        );
        assert_eq!(
            require_toml_string(&value, "package.version", "Cargo.toml"),
            Err("Cargo.toml is missing string key `package.version`".to_string())
        );
    }
}
