use crate::command::{CheckOptions, Command, Format};
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use unsafe_review_core::{
    AnalysisMode, AnalyzeInput, CardId, DiffSource, PolicyMode, Scope, analyze, collect_context,
    explain_card, render_human, render_json, render_markdown, render_pr_summary,
};

pub(crate) fn execute(command: Command) -> Result<(), String> {
    match command {
        Command::Help => {
            print_help();
            Ok(())
        }
        Command::Version => {
            println!("unsafe-review {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Doctor { root } => doctor(&root),
        Command::Check(options) => run_check(options, Scope::Diff, AnalysisMode::Draft),
        Command::Repo(options) => run_check(options, Scope::Repo, AnalysisMode::Repo),
        Command::Pilot(options) => run_check(options, Scope::Diff, AnalysisMode::Draft),
        Command::Badges { root, out } => badges(&root, &out),
        Command::Explain { root, id, format } => explain(&root, &id, format),
        Command::Context { root, id } => context(&root, &id),
    }
}

fn run_check(options: CheckOptions, scope: Scope, mode: AnalysisMode) -> Result<(), String> {
    let diff = diff_source(&options)?;
    let output = analyze(AnalyzeInput {
        root: options.root,
        scope,
        diff,
        mode,
        policy: PolicyMode::Advisory,
        include_unchanged_tests: true,
        max_cards: options.max_cards,
    })?;
    let rendered = render_with_format(&output, &options.format);
    if let Some(path) = options.out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
        }
        fs::write(&path, rendered)
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    } else {
        println!("{rendered}");
    }
    Ok(())
}

fn diff_source(options: &CheckOptions) -> Result<DiffSource, String> {
    if let Some(path) = &options.diff {
        return Ok(DiffSource::File(path.clone()));
    }
    if let Some(base) = &options.base {
        let output = ProcessCommand::new("git")
            .arg("diff")
            .arg(format!("{base}...HEAD"))
            .current_dir(&options.root)
            .output()
            .map_err(|err| format!("failed to run git diff: {err}"))?;
        if !output.status.success() {
            return Err(format!(
                "git diff failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        return Ok(DiffSource::Text(
            String::from_utf8_lossy(&output.stdout).into_owned(),
        ));
    }
    Ok(DiffSource::NoneRepoScan)
}

fn render_with_format(output: &unsafe_review_core::AnalyzeOutput, format: &Format) -> String {
    match format {
        Format::Human => render_human(output),
        Format::Json => render_json(output),
        Format::Markdown => render_markdown(output),
        Format::PrSummary => render_pr_summary(output),
    }
}

fn doctor(root: &Path) -> Result<(), String> {
    if !root.is_dir() {
        return Err(format!("root {} is not a directory", root.display()));
    }
    println!("unsafe-review doctor");
    println!("root: {}", root.display());
    println!("git: {}", tool_available("git"));
    println!("miri command available: {}", tool_available("cargo"));
    println!("policy: advisory by default");
    Ok(())
}

fn tool_available(name: &str) -> &'static str {
    if ProcessCommand::new(name).arg("--version").output().is_ok() {
        "yes"
    } else {
        "no"
    }
}

fn badges(root: &Path, out: &Path) -> Result<(), String> {
    fs::create_dir_all(out).map_err(|err| format!("create {} failed: {err}", out.display()))?;
    let output = analyze(AnalyzeInput {
        root: root.to_path_buf(),
        scope: Scope::Repo,
        diff: DiffSource::NoneRepoScan,
        mode: AnalysisMode::Repo,
        policy: PolicyMode::Advisory,
        include_unchanged_tests: true,
        max_cards: None,
    })?;
    let color = if output.summary.open_actionable_gaps == 0 {
        "green"
    } else if output.summary.open_actionable_gaps < 10 {
        "yellow"
    } else {
        "orange"
    };
    let main = format!(
        "{{\n  \"schemaVersion\": 1,\n  \"label\": \"unsafe-review\",\n  \"message\": \"{} open gaps\",\n  \"color\": \"{}\"\n}}\n",
        output.summary.open_actionable_gaps, color
    );
    let plus = format!(
        "{{\n  \"schemaVersion\": 1,\n  \"label\": \"unsafe-review+\",\n  \"message\": \"{} contract / {} guard / {} witness\",\n  \"color\": \"{}\"\n}}\n",
        output.summary.contract_missing,
        output.summary.guard_missing,
        output.summary.guarded_unwitnessed,
        color
    );
    fs::write(out.join("unsafe-review.json"), main)
        .map_err(|err| format!("write badge failed: {err}"))?;
    fs::write(out.join("unsafe-review-plus.json"), plus)
        .map_err(|err| format!("write badge failed: {err}"))?;
    println!("wrote badges to {}", out.display());
    Ok(())
}

fn explain(root: &Path, id: &str, format: Format) -> Result<(), String> {
    let output = analyze(AnalyzeInput {
        root: root.to_path_buf(),
        scope: Scope::Repo,
        diff: DiffSource::NoneRepoScan,
        mode: AnalysisMode::Repo,
        policy: PolicyMode::Advisory,
        include_unchanged_tests: true,
        max_cards: None,
    })?;
    let id = CardId(id.to_string());
    let Some(detail) = explain_card(&output, &id) else {
        return Err(format!("card `{id}` not found"));
    };
    match format {
        Format::Json => {
            let Some(packet) = collect_context(&output, &id) else {
                return Err(format!("card `{id}` not found"));
            };
            println!("{packet}");
        }
        _ => println!("{detail}"),
    }
    Ok(())
}

fn context(root: &Path, id: &str) -> Result<(), String> {
    let output = analyze(AnalyzeInput {
        root: root.to_path_buf(),
        scope: Scope::Repo,
        diff: DiffSource::NoneRepoScan,
        mode: AnalysisMode::Repo,
        policy: PolicyMode::Advisory,
        include_unchanged_tests: true,
        max_cards: None,
    })?;
    let id = CardId(id.to_string());
    let Some(packet) = collect_context(&output, &id) else {
        return Err(format!("card `{id}` not found"));
    };
    println!("{packet}");
    Ok(())
}

fn print_help() {
    println!("unsafe-review: cheap unsafe contract review for Rust");
    println!();
    println!("Commands:");
    println!(
        "  check   [--root .] [--base origin/main | --diff file] [--format human|json|markdown|pr-summary]"
    );
    println!("  repo    [--root .] [--format json]");
    println!("  pilot   [--root .] [--base origin/main] [--max-cards 5]");
    println!("  badges  [--root .] [--out badges]");
    println!("  explain [--root .] <card-id>");
    println!("  context [--root .] <card-id>");
    println!("  doctor  [--root .]");
    println!();
    println!("Trust boundary: static review evidence, not soundness proof.");
}
