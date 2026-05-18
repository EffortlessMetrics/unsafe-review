use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn check_json_matches_fixture_golden() -> Result<(), String> {
    let fixture = fixture_root("raw_pointer_alignment");
    let output = unsafe_review_command()
        .arg("check")
        .arg("--root")
        .arg(&fixture)
        .arg("--diff")
        .arg(fixture.join("change.diff"))
        .arg("--format")
        .arg("json")
        .output()
        .map_err(|err| format!("failed to run unsafe-review check: {err}"))?;
    let stdout = success_stdout(output)?;
    let actual = parse_json(&stdout)?;
    let expected_cards = fixture_expected_cards("raw_pointer_alignment")?;

    assert_eq!(actual["schema_version"], "0.1");
    assert_eq!(actual["tool"], "unsafe-review");
    assert_eq!(actual["scope"], "diff");
    assert_eq!(actual["summary"]["cards"], 1);
    assert_eq!(actual["cards"], expected_cards);
    Ok(())
}

#[test]
fn check_out_writes_report_without_printing_payload() -> Result<(), String> {
    let fixture = fixture_root("raw_pointer_alignment");
    let temp = temp_dir("check-out")?;
    let report = temp.join("nested").join("report.json");
    let output = unsafe_review_command()
        .arg("check")
        .arg("--root")
        .arg(&fixture)
        .arg("--diff")
        .arg(fixture.join("change.diff"))
        .arg("--json")
        .arg("--out")
        .arg(&report)
        .output()
        .map_err(|err| format!("failed to run unsafe-review check --out: {err}"))?;
    let stdout = success_stdout(output)?;
    if !stdout.trim().is_empty() {
        return Err(format!(
            "expected empty stdout when --out is set, got {stdout:?}"
        ));
    }

    let actual = parse_json_file(&report)?;
    assert_eq!(
        actual["cards"],
        fixture_expected_cards("raw_pointer_alignment")?
    );
    remove_dir_all(&temp)?;
    Ok(())
}

#[test]
fn explain_and_context_can_read_a_reported_card_from_repo_scan() -> Result<(), String> {
    let fixture = fixture_root("raw_pointer_alignment");
    let card_id = "UR-src-lib-rs-8-read-header-raw_pointer_read";

    let explain_output = unsafe_review_command()
        .arg("explain")
        .arg("--root")
        .arg(&fixture)
        .arg(card_id)
        .output()
        .map_err(|err| format!("failed to run unsafe-review explain: {err}"))?;
    let explanation = success_stdout(explain_output)?;
    assert_contains(&explanation, card_id)?;
    assert_contains(&explanation, "raw_pointer_read")?;

    let context_output = unsafe_review_command()
        .arg("context")
        .arg("--root")
        .arg(&fixture)
        .arg(card_id)
        .output()
        .map_err(|err| format!("failed to run unsafe-review context: {err}"))?;
    let context = parse_json(&success_stdout(context_output)?)?;
    assert_eq!(context["card_id"], card_id);
    assert_eq!(context["context"]["file"], "src/lib.rs");
    Ok(())
}

#[test]
fn badges_writes_both_badge_contracts() -> Result<(), String> {
    let fixture = fixture_root("raw_pointer_alignment");
    let temp = temp_dir("badges")?;
    let output = unsafe_review_command()
        .arg("badges")
        .arg("--root")
        .arg(&fixture)
        .arg("--out")
        .arg(&temp)
        .output()
        .map_err(|err| format!("failed to run unsafe-review badges: {err}"))?;
    let stdout = success_stdout(output)?;
    assert_contains(&stdout, "wrote badges")?;

    let main = parse_json_file(&temp.join("unsafe-review.json"))?;
    let plus = parse_json_file(&temp.join("unsafe-review-plus.json"))?;
    assert_eq!(main["schemaVersion"], 1);
    assert_eq!(main["label"], "unsafe-review");
    assert_eq!(main["message"], "1 open gaps");
    assert_eq!(plus["schemaVersion"], 1);
    assert_eq!(plus["label"], "unsafe-review+");
    assert_eq!(plus["message"], "0 contract / 1 guard / 0 witness");
    remove_dir_all(&temp)?;
    Ok(())
}

fn unsafe_review_command() -> ProcessCommand {
    let mut command = ProcessCommand::new(env!("CARGO_BIN_EXE_unsafe-review"));
    command.current_dir(repo_root());
    command
}

fn fixture_expected_cards(name: &str) -> Result<Value, String> {
    parse_json_file(&fixture_root(name).join("expected.cards.json"))
}

fn fixture_root(name: &str) -> PathBuf {
    repo_root().join("fixtures").join(name)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn parse_json_file(path: &PathBuf) -> Result<Value, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    parse_json(&text)
}

fn parse_json(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|err| format!("JSON parse failed: {err}\n{text}"))
}

fn success_stdout(output: Output) -> Result<String, String> {
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout was not valid UTF-8: {err}"))?;
    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr was not valid UTF-8: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "command exited with {}\nstdout:\n{}\nstderr:\n{}",
            output.status, stdout, stderr
        ));
    }
    Ok(stdout)
}

fn assert_contains(haystack: &str, needle: &str) -> Result<(), String> {
    if !haystack.contains(needle) {
        return Err(format!(
            "expected output to contain {needle:?}, got:\n{haystack}"
        ));
    }
    Ok(())
}

fn temp_dir(name: &str) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system time before UNIX_EPOCH: {err}"))?
        .as_nanos();
    let path = repo_root()
        .join("target")
        .join("e2e-tmp")
        .join(format!("{name}-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&path).map_err(|err| format!("create {} failed: {err}", path.display()))?;
    Ok(path)
}

fn remove_dir_all(path: &PathBuf) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|err| format!("remove {} failed: {err}", path.display()))?;
    }
    Ok(())
}
