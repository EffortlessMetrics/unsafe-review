use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn check_json_matches_all_fixture_goldens() -> Result<(), String> {
    for fixture in fixture_names()? {
        let output = run_unsafe_review([
            "check".to_string(),
            "--root".to_string(),
            fixture_root(&fixture)?.display().to_string(),
            "--diff".to_string(),
            fixture_root(&fixture)?
                .join("change.diff")
                .display()
                .to_string(),
            "--json".to_string(),
        ])?;
        assert_success(&output, &fixture)?;

        let actual = parse_stdout_json(&output, &fixture)?;
        let expected_cards = read_expected_cards(&fixture)?;
        assert_eq!(json_field(&actual, "cards")?, &expected_cards, "{fixture}");
        assert_eq!(
            json_field(json_field(&actual, "summary")?, "cards")?,
            &Value::from(expected_cards.as_array().map_or(0, Vec::len)),
            "{fixture} summary.cards should match golden card count",
        );
        assert_eq!(
            json_field(&actual, "scope")?,
            &Value::from("diff"),
            "{fixture}"
        );
        assert_eq!(
            json_field(&actual, "mode")?,
            &Value::from("draft"),
            "{fixture}"
        );
    }
    Ok(())
}

#[test]
fn cargo_subcommand_alias_writes_markdown_to_requested_output() -> Result<(), String> {
    let fixture = "raw_pointer_alignment";
    let out_dir = unique_temp_dir("unsafe-review-e2e")?;
    let out_file = out_dir.join("nested").join("review.md");
    let output = run_unsafe_review([
        "unsafe-review".to_string(),
        "check".to_string(),
        "--root".to_string(),
        fixture_root(fixture)?.display().to_string(),
        "--diff".to_string(),
        fixture_root(fixture)?
            .join("change.diff")
            .display()
            .to_string(),
        "--markdown".to_string(),
        "--out".to_string(),
        out_file.display().to_string(),
    ])?;
    assert_success(&output, "markdown --out")?;
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");

    let markdown = fs::read_to_string(&out_file)
        .map_err(|err| format!("read {} failed: {err}", out_file.display()))?;
    assert!(
        markdown.contains("UR-src-lib-rs-8-read-header-raw_pointer_read"),
        "markdown output should include the fixture card id",
    );
    assert!(
        markdown.contains("cargo +nightly miri test read_header"),
        "markdown output should include witness commands",
    );

    fs::remove_dir_all(&out_dir)
        .map_err(|err| format!("cleanup {} failed: {err}", out_dir.display()))?;
    Ok(())
}

#[test]
fn check_reports_missing_diff_file_as_cli_failure() -> Result<(), String> {
    let fixture = "safe_code_no_cards";
    let missing_diff = fixture_root(fixture)?.join("missing.diff");
    let output = run_unsafe_review([
        "check".to_string(),
        "--root".to_string(),
        fixture_root(fixture)?.display().to_string(),
        "--diff".to_string(),
        missing_diff.display().to_string(),
        "--json".to_string(),
    ])?;

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cargo-unsafe-review: read diff"),
        "stderr should identify the unreadable diff: {stderr}",
    );
    Ok(())
}

fn fixture_names() -> Result<Vec<String>, String> {
    let fixtures_dir = repo_root()?.join("fixtures");
    let mut fixtures = Vec::new();
    for entry in fs::read_dir(&fixtures_dir)
        .map_err(|err| format!("read {} failed: {err}", fixtures_dir.display()))?
    {
        let entry = entry.map_err(|err| format!("read fixture dir entry failed: {err}"))?;
        let path = entry.path();
        if !path.join("expected.cards.json").is_file() {
            continue;
        }
        let name = entry.file_name().into_string().map_err(|name| {
            format!("non-UTF-8 fixture path {:?} under {}", name, path.display())
        })?;
        fixtures.push(name);
    }
    fixtures.sort();
    if fixtures.is_empty() {
        return Err("no e2e fixtures with expected.cards.json were found".to_string());
    }
    Ok(fixtures)
}

fn run_unsafe_review(args: impl IntoIterator<Item = String>) -> Result<Output, String> {
    Command::new(binary_path()?)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run cargo-unsafe-review: {err}"))
}

fn binary_path() -> Result<PathBuf, String> {
    std::env::var_os("CARGO_BIN_EXE_cargo-unsafe-review")
        .map(PathBuf::from)
        .ok_or_else(|| "CARGO_BIN_EXE_cargo-unsafe-review was not set by cargo".to_string())
}

fn fixture_root(name: &str) -> Result<PathBuf, String> {
    Ok(repo_root()?.join("fixtures").join(name))
}

fn repo_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|err| format!("canonicalize repo root failed: {err}"))
}

fn read_expected_cards(fixture: &str) -> Result<Value, String> {
    let path = fixture_root(fixture)?.join("expected.cards.json");
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse {} failed: {err}", path.display()))
}

fn parse_stdout_json(output: &Output, label: &str) -> Result<Value, String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|err| format!("parse stdout for {label} failed: {err}\n{stdout}"))
}

fn json_field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value
        .get(key)
        .ok_or_else(|| format!("missing json field `{key}` in {value}"))
}

fn assert_success(output: &Output, label: &str) -> Result<(), String> {
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{label} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

fn unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        monotonic_nanos()?
    ));
    fs::create_dir_all(&path).map_err(|err| format!("create {} failed: {err}", path.display()))?;
    Ok(path)
}

fn monotonic_nanos() -> Result<u128, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|err| format!("system clock is before unix epoch: {err}"))
}
