use serde_json::Value;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn check_json_stdout_can_feed_context_command() -> Result<(), Box<dyn Error>> {
    let fixture = fixture_root("raw_pointer_alignment");
    let output = run_success([
        OsStr::new("check"),
        OsStr::new("--root"),
        fixture.as_os_str(),
        OsStr::new("--diff"),
        fixture.join("change.diff").as_os_str(),
        OsStr::new("--format"),
        OsStr::new("json"),
    ])?;
    let stdout = stdout_text(&output)?;
    let value = parse_json(&stdout)?;

    assert_eq!(value["schema_version"], "0.1");
    assert_eq!(value["scope"], "diff");
    assert_eq!(value["summary"]["cards"], 1);
    assert_eq!(value["cards"][0]["class"], "guard_missing");
    assert_eq!(value["cards"][0]["site"]["file"], "src/lib.rs");
    assert_eq!(value["cards"][0]["operation_family"], "raw_pointer_read");

    let card_id = json_str(&value["cards"][0]["id"], "cards[0].id")?;
    let context = run_success([
        OsStr::new("context"),
        OsStr::new("--root"),
        fixture.as_os_str(),
        OsStr::new(card_id),
    ])?;
    let packet = parse_json(&stdout_text(&context)?)?;

    assert_eq!(packet["schema_version"], "0.1");
    assert_eq!(packet["tool"], "unsafe-review");
    assert_eq!(packet["card_id"], card_id);
    assert_eq!(packet["context"]["file"], "src/lib.rs");
    assert!(packet["required_safety_conditions"].is_array());
    assert!(packet["stop_conditions"].is_array());
    Ok(())
}

#[test]
fn cargo_alias_markdown_out_and_badges_write_expected_files() -> Result<(), Box<dyn Error>> {
    let fixture = fixture_root("raw_pointer_alignment");
    let temp = TempDir::new("unsafe-review-cli-e2e")?;
    let report = temp.path().join("nested").join("report.md");
    let output = run_success([
        OsStr::new("unsafe-review"),
        OsStr::new("check"),
        OsStr::new("--root"),
        fixture.as_os_str(),
        OsStr::new("--diff"),
        fixture.join("change.diff").as_os_str(),
        OsStr::new("--format"),
        OsStr::new("markdown"),
        OsStr::new("--out"),
        report.as_os_str(),
        OsStr::new("--max-cards"),
        OsStr::new("1"),
    ])?;

    assert_eq!(stdout_text(&output)?.trim(), "");
    let markdown = fs::read_to_string(&report)?;
    assert!(markdown.starts_with("# unsafe-review"));
    assert!(markdown.contains("guard_missing"));
    assert!(markdown.contains("raw_pointer_read"));
    assert!(markdown.contains("## Trust boundary"));

    let badge_dir = temp.path().join("badges");
    let badge_output = run_success([
        OsStr::new("badges"),
        OsStr::new("--root"),
        fixture.as_os_str(),
        OsStr::new("--out"),
        badge_dir.as_os_str(),
    ])?;
    assert!(stdout_text(&badge_output)?.contains("wrote badges to"));

    let main_badge = parse_json(&fs::read_to_string(badge_dir.join("unsafe-review.json"))?)?;
    let plus_badge = parse_json(&fs::read_to_string(
        badge_dir.join("unsafe-review-plus.json"),
    )?)?;
    assert_eq!(main_badge["schemaVersion"], 1);
    assert_eq!(main_badge["label"], "unsafe-review");
    assert_eq!(main_badge["message"], "1 open gaps");
    assert_eq!(plus_badge["label"], "unsafe-review+");
    assert_eq!(plus_badge["message"], "0 contract / 1 guard / 0 witness");
    Ok(())
}

fn run_success<I, S>(args: I) -> Result<Output, Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(bin_path()).args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "command failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output)
}

fn stdout_text(output: &Output) -> Result<String, Box<dyn Error>> {
    String::from_utf8(output.stdout.clone()).map_err(Into::into)
}

fn parse_json(text: &str) -> Result<Value, Box<dyn Error>> {
    serde_json::from_str(text).map_err(Into::into)
}

fn json_str<'a>(value: &'a Value, path: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .as_str()
        .ok_or_else(|| format!("expected {path} to be a string").into())
}

fn bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-unsafe-review"))
}

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Result<Self, Box<dyn Error>> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.path);
    }
}
