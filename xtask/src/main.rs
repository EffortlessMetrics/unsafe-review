#![forbid(unsafe_code)]
use std::process::Command;

fn main() {
    if let Err(err) = run(std::env::args().collect()) {
        eprintln!("xtask: {err}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.get(1).map(|arg| arg.as_str()) {
        None | Some("help") | Some("--help") => {
            println!("xtask commands: check-pr, check-docs, check-policy");
            Ok(())
        }
        Some("check-pr") => {
            check_docs()?;
            check_policy()?;
            println!("check-pr: ok");
            Ok(())
        }
        Some("check-docs") => check_docs(),
        Some("check-policy") => check_policy(),
        Some(other) => Err(format!("unknown xtask command `{other}`")),
    }
}

fn check_docs() -> Result<(), String> {
    for path in [
        "README.md",
        "docs/MISSION.md",
        "docs/ROADMAP.md",
        "docs/specs/README.md",
        "docs/status/SUPPORT_TIERS.md",
    ] {
        if !std::path::Path::new(path).exists() {
            return Err(format!("required doc missing: {path}"));
        }
    }
    println!("check-docs: ok");
    Ok(())
}

fn check_policy() -> Result<(), String> {
    for path in [
        "policy/unsafe-review.toml",
        "policy/non-rust-allowlist.toml",
        "policy/clippy-lints.toml",
    ] {
        if !std::path::Path::new(path).exists() {
            return Err(format!("required policy missing: {path}"));
        }
    }
    println!("check-policy: ok");
    Ok(())
}

#[allow(
    dead_code,
    reason = "Reserved for future xtask wrappers that need shell command execution."
)]
fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|err| format!("failed to run {program}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} failed with status {status}"))
    }
}
