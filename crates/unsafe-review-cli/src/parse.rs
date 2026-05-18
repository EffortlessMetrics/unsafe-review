use crate::command::{CheckOptions, Command, DiffInput, Format};
use std::path::PathBuf;

pub(crate) fn parse(args: Vec<String>) -> Result<Command, String> {
    let mut rest = args.into_iter().skip(1).collect::<Vec<_>>();
    if rest.is_empty() {
        return Ok(Command::Help);
    }
    let command = rest.remove(0);
    match command.as_str() {
        "--help" | "-h" | "help" => Ok(Command::Help),
        "--version" | "-V" => Ok(Command::Version),
        "doctor" => parse_doctor(rest),
        "check" => parse_check(rest).map(Command::Check),
        "repo" => parse_check(rest).map(Command::Repo),
        "pilot" => parse_check(rest).map(|mut options| {
            options.max_cards = Some(options.max_cards.unwrap_or(5));
            Command::Pilot(options)
        }),
        "badges" => parse_badges(rest),
        "explain" => parse_explain(rest),
        "context" => parse_context(rest),
        other => Err(format!(
            "unknown command `{other}`. Run `unsafe-review --help`."
        )),
    }
}

fn parse_doctor(args: Vec<String>) -> Result<Command, String> {
    let mut root = PathBuf::from(".");
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                idx += 1;
                root = PathBuf::from(value(&args, idx, "--root")?);
            }
            arg if arg.starts_with("--root=") => {
                root = PathBuf::from(inline_value(arg, "--root")?);
            }
            other => return Err(format!("unknown doctor argument `{other}`")),
        }
        idx += 1;
    }
    Ok(Command::Doctor { root })
}

fn parse_check(args: Vec<String>) -> Result<CheckOptions, String> {
    let mut options = CheckOptions::default();
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                idx += 1;
                options.root = PathBuf::from(value(&args, idx, "--root")?);
            }
            arg if arg.starts_with("--root=") => {
                options.root = PathBuf::from(inline_value(arg, "--root")?);
            }
            "--base" => {
                idx += 1;
                options.base = Some(value(&args, idx, "--base")?.to_string());
            }
            arg if arg.starts_with("--base=") => {
                options.base = Some(inline_value(arg, "--base")?.to_string());
            }
            "--diff" => {
                idx += 1;
                options.diff = Some(parse_diff_input(value(&args, idx, "--diff")?));
            }
            arg if arg.starts_with("--diff=") => {
                options.diff = Some(parse_diff_input(inline_value(arg, "--diff")?));
            }
            "--format" => {
                idx += 1;
                options.format = parse_format(value(&args, idx, "--format")?)?;
            }
            arg if arg.starts_with("--format=") => {
                options.format = parse_format(inline_value(arg, "--format")?)?;
            }
            "--json" => options.format = Format::Json,
            "--markdown" => options.format = Format::Markdown,
            "--out" => {
                idx += 1;
                options.out = Some(PathBuf::from(value(&args, idx, "--out")?));
            }
            arg if arg.starts_with("--out=") => {
                options.out = Some(PathBuf::from(inline_value(arg, "--out")?));
            }
            "--max-cards" => {
                idx += 1;
                options.max_cards = Some(parse_max_cards(value(&args, idx, "--max-cards")?)?);
            }
            arg if arg.starts_with("--max-cards=") => {
                options.max_cards = Some(parse_max_cards(inline_value(arg, "--max-cards")?)?);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }
    validate_check_options(&options)?;
    Ok(options)
}

fn parse_badges(args: Vec<String>) -> Result<Command, String> {
    let mut root = PathBuf::from(".");
    let mut out = PathBuf::from("badges");
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                idx += 1;
                root = PathBuf::from(value(&args, idx, "--root")?);
            }
            arg if arg.starts_with("--root=") => {
                root = PathBuf::from(inline_value(arg, "--root")?);
            }
            "--out" => {
                idx += 1;
                out = PathBuf::from(value(&args, idx, "--out")?);
            }
            arg if arg.starts_with("--out=") => {
                out = PathBuf::from(inline_value(arg, "--out")?);
            }
            other => return Err(format!("unknown badges argument `{other}`")),
        }
        idx += 1;
    }
    Ok(Command::Badges { root, out })
}

fn parse_explain(args: Vec<String>) -> Result<Command, String> {
    let mut root = PathBuf::from(".");
    let mut format = Format::Markdown;
    let mut id: Option<String> = None;
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                idx += 1;
                root = PathBuf::from(value(&args, idx, "--root")?);
            }
            arg if arg.starts_with("--root=") => {
                root = PathBuf::from(inline_value(arg, "--root")?);
            }
            "--format" => {
                idx += 1;
                format = parse_format(value(&args, idx, "--format")?)?;
            }
            arg if arg.starts_with("--format=") => {
                format = parse_format(inline_value(arg, "--format")?)?;
            }
            "--json" => format = Format::Json,
            "--markdown" => format = Format::Markdown,
            value if value.starts_with('-') => {
                return Err(format!("unknown explain argument `{value}`"));
            }
            value => set_card_id(&mut id, value)?,
        }
        idx += 1;
    }
    Ok(Command::Explain {
        root,
        id: id.ok_or_else(|| "missing card id".to_string())?,
        format,
    })
}

fn parse_context(args: Vec<String>) -> Result<Command, String> {
    let mut root = PathBuf::from(".");
    let mut id: Option<String> = None;
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                idx += 1;
                root = PathBuf::from(value(&args, idx, "--root")?);
            }
            arg if arg.starts_with("--root=") => {
                root = PathBuf::from(inline_value(arg, "--root")?);
            }
            "--json" => {}
            "--format" => {
                idx += 1;
                let raw = value(&args, idx, "--format")?;
                if parse_format(raw)? != Format::Json {
                    return Err("context only supports json output".to_string());
                }
            }
            arg if arg.starts_with("--format=") => {
                if parse_format(inline_value(arg, "--format")?)? != Format::Json {
                    return Err("context only supports json output".to_string());
                }
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown context argument `{value}`"));
            }
            value => set_card_id(&mut id, value)?,
        }
        idx += 1;
    }
    Ok(Command::Context {
        root,
        id: id.ok_or_else(|| "missing card id".to_string())?,
    })
}

fn parse_diff_input(raw: &str) -> DiffInput {
    if raw == "-" {
        DiffInput::Stdin
    } else {
        DiffInput::File(PathBuf::from(raw))
    }
}

fn validate_check_options(options: &CheckOptions) -> Result<(), String> {
    if options.base.is_some() && options.diff.is_some() {
        return Err("choose only one of --base or --diff".to_string());
    }
    Ok(())
}

fn set_card_id(id: &mut Option<String>, value: &str) -> Result<(), String> {
    if id.replace(value.to_string()).is_some() {
        return Err("expected exactly one card id".to_string());
    }
    Ok(())
}

fn parse_max_cards(raw: &str) -> Result<usize, String> {
    raw.parse::<usize>()
        .map_err(|err| format!("invalid --max-cards `{raw}`: {err}"))
}

fn parse_format(raw: &str) -> Result<Format, String> {
    match raw {
        "human" => Ok(Format::Human),
        "json" | "repo-json" => Ok(Format::Json),
        "markdown" | "md" => Ok(Format::Markdown),
        "pr-summary" | "github-summary" | "github-markdown" => Ok(Format::PrSummary),
        "sarif" => Ok(Format::Sarif),
        "comment-plan" | "comments" => Ok(Format::CommentPlan),
        "lsp" | "lsp-json" | "editor-json" => Ok(Format::Lsp),
        other => Err(format!("unknown format `{other}`")),
    }
}

fn value<'a>(args: &'a [String], idx: usize, flag: &str) -> Result<&'a str, String> {
    let Some(value) = args.get(idx).map(|value| value.as_str()) else {
        return Err(format!("missing value for {flag}"));
    };
    if value != "-" && value.starts_with('-') {
        return Err(format!("missing value for {flag}"));
    }
    Ok(value)
}

fn inline_value<'a>(arg: &'a str, flag: &str) -> Result<&'a str, String> {
    let Some(value) = arg
        .strip_prefix(flag)
        .and_then(|rest| rest.strip_prefix('='))
    else {
        return Err(format!("missing value for {flag}"));
    };
    if value.is_empty() {
        return Err(format!("missing value for {flag}"));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pr_summary_format_for_check() -> Result<(), String> {
        let command = parse(args(["unsafe-review", "check", "--format", "pr-summary"]))?;
        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };
        assert_eq!(options.format, Format::PrSummary);
        Ok(())
    }

    #[test]
    fn parses_github_summary_alias_for_check() -> Result<(), String> {
        let command = parse(args([
            "unsafe-review",
            "check",
            "--format",
            "github-summary",
        ]))?;
        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };
        assert_eq!(options.format, Format::PrSummary);
        Ok(())
    }

    #[test]
    fn parses_sarif_format_for_check() -> Result<(), String> {
        let command = parse(args(["unsafe-review", "check", "--format", "sarif"]))?;
        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };
        assert_eq!(options.format, Format::Sarif);
        Ok(())
    }

    #[test]
    fn parses_comment_plan_format_for_check() -> Result<(), String> {
        let command = parse(args(["unsafe-review", "check", "--format", "comment-plan"]))?;
        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };
        assert_eq!(options.format, Format::CommentPlan);
        Ok(())
    }

    #[test]
    fn parses_lsp_format_for_check() -> Result<(), String> {
        let command = parse(args(["unsafe-review", "check", "--format", "lsp"]))?;
        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };
        assert_eq!(options.format, Format::Lsp);
        Ok(())
    }

    #[test]
    fn parses_equals_style_artifact_flags_for_check() -> Result<(), String> {
        let command = parse(args([
            "unsafe-review",
            "check",
            "--root=fixtures/raw_pointer_deref",
            "--diff=-",
            "--format=sarif",
            "--out=target/unsafe-review/cards.sarif",
            "--max-cards=7",
        ]))?;
        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };

        assert_eq!(options.root, PathBuf::from("fixtures/raw_pointer_deref"));
        assert_eq!(options.diff, Some(DiffInput::Stdin));
        assert_eq!(options.format, Format::Sarif);
        assert_eq!(
            options.out,
            Some(PathBuf::from("target/unsafe-review/cards.sarif"))
        );
        assert_eq!(options.max_cards, Some(7));
        Ok(())
    }

    #[test]
    fn parses_stdin_diff_for_check() -> Result<(), String> {
        let command = parse(args(["unsafe-review", "check", "--diff", "-", "--json"]))?;
        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };

        assert_eq!(options.diff, Some(DiffInput::Stdin));
        assert_eq!(options.format, Format::Json);
        Ok(())
    }

    #[test]
    fn rejects_conflicting_diff_sources_for_check() {
        let command = parse(args([
            "unsafe-review",
            "check",
            "--base",
            "origin/main",
            "--diff",
            "change.diff",
        ]));

        assert_eq!(
            command,
            Err("choose only one of --base or --diff".to_string())
        );
    }

    #[test]
    fn rejects_missing_values_when_next_argument_is_a_flag() {
        let diff = parse(args(["unsafe-review", "check", "--diff", "--json"]));
        let format = parse(args(["unsafe-review", "check", "--format", "--out"]));

        assert_eq!(diff, Err("missing value for --diff".to_string()));
        assert_eq!(format, Err("missing value for --format".to_string()));
    }

    #[test]
    fn parses_context_json_alias() -> Result<(), String> {
        let command = parse(args(["unsafe-review", "context", "--json", "UR-card"]))?;

        assert_eq!(
            command,
            Command::Context {
                root: PathBuf::from("."),
                id: "UR-card".to_string()
            }
        );
        Ok(())
    }

    #[test]
    fn parses_equals_style_explain_and_context_flags() -> Result<(), String> {
        let explain = parse(args([
            "unsafe-review",
            "explain",
            "--root=fixtures/raw_pointer_deref",
            "--format=json",
            "UR-card",
        ]))?;
        assert_eq!(
            explain,
            Command::Explain {
                root: PathBuf::from("fixtures/raw_pointer_deref"),
                id: "UR-card".to_string(),
                format: Format::Json,
            }
        );

        let context = parse(args([
            "unsafe-review",
            "context",
            "--root=fixtures/raw_pointer_deref",
            "--format=json",
            "UR-card",
        ]))?;
        assert_eq!(
            context,
            Command::Context {
                root: PathBuf::from("fixtures/raw_pointer_deref"),
                id: "UR-card".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn parses_equals_style_doctor_and_badges_flags() -> Result<(), String> {
        let doctor = parse(args(["unsafe-review", "doctor", "--root=fixtures"]))?;
        assert_eq!(
            doctor,
            Command::Doctor {
                root: PathBuf::from("fixtures"),
            }
        );

        let badges = parse(args([
            "unsafe-review",
            "badges",
            "--root=fixtures",
            "--out=target/badges",
        ]))?;
        assert_eq!(
            badges,
            Command::Badges {
                root: PathBuf::from("fixtures"),
                out: PathBuf::from("target/badges"),
            }
        );
        Ok(())
    }

    #[test]
    fn rejects_non_json_context_format() {
        let command = parse(args([
            "unsafe-review",
            "context",
            "--format",
            "markdown",
            "UR-card",
        ]));

        assert_eq!(
            command,
            Err("context only supports json output".to_string())
        );
    }

    #[test]
    fn rejects_duplicate_card_ids() {
        let explain = parse(args(["unsafe-review", "explain", "UR-one", "UR-two"]));
        let context = parse(args(["unsafe-review", "context", "UR-one", "UR-two"]));

        assert_eq!(explain, Err("expected exactly one card id".to_string()));
        assert_eq!(context, Err("expected exactly one card id".to_string()));
    }

    fn args<const N: usize>(values: [&str; N]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }
}
