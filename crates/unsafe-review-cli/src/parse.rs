use crate::command::{CheckOptions, Command, Format};
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
            "--base" => {
                idx += 1;
                options.base = Some(value(&args, idx, "--base")?.to_string());
            }
            "--diff" => {
                idx += 1;
                options.diff = Some(PathBuf::from(value(&args, idx, "--diff")?));
            }
            "--format" => {
                idx += 1;
                options.format = parse_format(value(&args, idx, "--format")?)?;
            }
            "--json" => options.format = Format::Json,
            "--markdown" => options.format = Format::Markdown,
            "--out" => {
                idx += 1;
                options.out = Some(PathBuf::from(value(&args, idx, "--out")?));
            }
            "--max-cards" => {
                idx += 1;
                let raw = value(&args, idx, "--max-cards")?;
                options.max_cards = Some(
                    raw.parse::<usize>()
                        .map_err(|err| format!("invalid --max-cards `{raw}`: {err}"))?,
                );
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }
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
            "--out" => {
                idx += 1;
                out = PathBuf::from(value(&args, idx, "--out")?);
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
            "--format" => {
                idx += 1;
                format = parse_format(value(&args, idx, "--format")?)?;
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown explain argument `{value}`"));
            }
            value => id = Some(value.to_string()),
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
            value if value.starts_with('-') => {
                return Err(format!("unknown context argument `{value}`"));
            }
            value => id = Some(value.to_string()),
        }
        idx += 1;
    }
    Ok(Command::Context {
        root,
        id: id.ok_or_else(|| "missing card id".to_string())?,
    })
}

fn parse_format(raw: &str) -> Result<Format, String> {
    match raw {
        "human" => Ok(Format::Human),
        "json" | "repo-json" => Ok(Format::Json),
        "markdown" | "md" => Ok(Format::Markdown),
        "pr-summary" | "github-summary" | "github-markdown" => Ok(Format::PrSummary),
        other => Err(format!("unknown format `{other}`")),
    }
}

fn value<'a>(args: &'a [String], idx: usize, flag: &str) -> Result<&'a str, String> {
    args.get(idx)
        .map(|value| value.as_str())
        .ok_or_else(|| format!("missing value for {flag}"))
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

    fn args<const N: usize>(values: [&str; N]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }
}
