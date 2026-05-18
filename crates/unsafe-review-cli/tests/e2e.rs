use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const RAW_POINTER_CARD_ID: &str = "UR-src-lib-rs-8-read-header-raw_pointer_read";

#[test]
fn check_json_reports_raw_pointer_fixture_card() -> Result<(), Box<dyn Error>> {
    let fixture = fixture_root("raw_pointer_alignment");
    let diff = fixture.join("change.diff");

    let output = unsafe_review_command()
        .arg("check")
        .arg("--root")
        .arg(&fixture)
        .arg("--diff")
        .arg(&diff)
        .arg("--format")
        .arg("json")
        .output()?;

    assert_success(&output);
    let stdout = stdout(&output)?;
    let json: Value = serde_json::from_str(&stdout)?;

    assert_eq!(json["tool"], "unsafe-review");
    assert_eq!(json["scope"], "diff");
    assert_eq!(json["summary"]["cards"], 1);
    assert_eq!(json["summary"]["guard_missing"], 1);
    assert_eq!(json["cards"][0]["id"], RAW_POINTER_CARD_ID);
    assert_eq!(json["cards"][0]["class"], "guard_missing");
    assert_eq!(json["cards"][0]["site"]["file"], "src/lib.rs");
    assert_eq!(json["cards"][0]["operation_family"], "raw_pointer_read");
    assert!(
        json["cards"][0]["hazards"]
            .as_array()
            .is_some_and(|hazards| hazards.iter().any(|hazard| hazard == "alignment"))
    );

    Ok(())
}

#[test]
fn cargo_subcommand_style_check_writes_markdown_out_file() -> Result<(), Box<dyn Error>> {
    let fixture = fixture_root("raw_pointer_alignment");
    let diff = fixture.join("change.diff");
    let temp = temp_dir("markdown-out")?;
    let report = temp.join("nested").join("unsafe-review.md");

    let output = unsafe_review_command()
        .arg("unsafe-review")
        .arg("check")
        .arg("--root")
        .arg(&fixture)
        .arg("--diff")
        .arg(&diff)
        .arg("--markdown")
        .arg("--out")
        .arg(&report)
        .output()?;

    assert_success(&output);
    assert_eq!(stdout(&output)?, "");

    let report_text = fs::read_to_string(&report)?;
    assert!(report_text.contains("# unsafe-review"));
    assert!(report_text.contains(RAW_POINTER_CARD_ID));
    assert!(report_text.contains("`guard_missing`"));

    fs::remove_dir_all(temp)?;
    Ok(())
}

#[test]
fn repo_badges_explain_and_context_cover_artifact_commands() -> Result<(), Box<dyn Error>> {
    let fixture = fixture_root("raw_pointer_alignment");
    let temp = temp_dir("artifacts")?;
    let badges = temp.join("badges");

    let badges_output = unsafe_review_command()
        .arg("badges")
        .arg("--root")
        .arg(&fixture)
        .arg("--out")
        .arg(&badges)
        .output()?;
    assert_success(&badges_output);
    assert!(stdout(&badges_output)?.contains("wrote badges"));

    let main_badge: Value =
        serde_json::from_str(&fs::read_to_string(badges.join("unsafe-review.json"))?)?;
    let plus_badge: Value =
        serde_json::from_str(&fs::read_to_string(badges.join("unsafe-review-plus.json"))?)?;
    assert_eq!(main_badge["schemaVersion"], 1);
    assert_eq!(main_badge["message"], "1 open gaps");
    assert_eq!(plus_badge["message"], "0 contract / 1 guard / 0 witness");

    let explain_output = unsafe_review_command()
        .arg("explain")
        .arg("--root")
        .arg(&fixture)
        .arg(RAW_POINTER_CARD_ID)
        .output()?;
    assert_success(&explain_output);
    let explain_text = stdout(&explain_output)?;
    assert!(explain_text.contains("# unsafe-review card"));
    assert!(explain_text.contains("**Class:** `guard_missing`"));
    assert!(explain_text.contains("## Required safety conditions"));

    let context_output = unsafe_review_command()
        .arg("context")
        .arg("--root")
        .arg(&fixture)
        .arg(RAW_POINTER_CARD_ID)
        .output()?;
    assert_success(&context_output);
    let context_json: Value = serde_json::from_str(&stdout(&context_output)?)?;
    assert_eq!(context_json["card_id"], RAW_POINTER_CARD_ID);
    assert_eq!(
        context_json["context"]["operation"],
        "unsafe { ptr.cast::<Header>().read() }"
    );
    assert!(context_json["do_not_do"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item == "do not add a broad suppression")
    }));

    fs::remove_dir_all(temp)?;
    Ok(())
}

#[test]
fn safe_fixture_emits_no_cards_in_human_output() -> Result<(), Box<dyn Error>> {
    let fixture = fixture_root("safe_code_no_cards");

    let output = unsafe_review_command()
        .arg("repo")
        .arg("--root")
        .arg(&fixture)
        .output()?;

    assert_success(&output);
    let stdout = stdout(&output)?;
    assert!(stdout.contains("cards: 0, open gaps: 0"));
    assert!(stdout.contains("No unsafe-review cards found."));

    Ok(())
}

fn unsafe_review_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-unsafe-review"))
}

fn fixture_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join(name)
}

fn temp_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let unique = format!(
        "unsafe-review-e2e-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    );
    let path = std::env::temp_dir().join(unique);
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stdout(output: &Output) -> Result<String, Box<dyn Error>> {
    Ok(String::from_utf8(output.stdout.clone())?)
}
