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
            "--fail-on-gaps" => options.fail_on_gaps = true,
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
    validate_check_options(&options)?;
    Ok(options)
}

fn validate_check_options(options: &CheckOptions) -> Result<(), String> {
    if options.base.is_some() && options.diff.is_some() {
        return Err("--base and --diff cannot be used together".to_string());
    }
    Ok(())
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
            value => set_id(&mut id, value, "explain")?,
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
            "--json" => {}
            "--format" => {
                idx += 1;
                let format = parse_format(value(&args, idx, "--format")?)?;
                if format != Format::Json {
                    return Err("context only supports json format".to_string());
                }
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown context argument `{value}`"));
            }
            value => set_id(&mut id, value, "context")?,
        }
        idx += 1;
    }
    Ok(Command::Context {
        root,
        id: id.ok_or_else(|| "missing card id".to_string())?,
    })
}

fn set_id(id: &mut Option<String>, value: &str, command: &str) -> Result<(), String> {
    if id.replace(value.to_string()).is_some() {
        return Err(format!("multiple card ids supplied to {command}"));
    }
    Ok(())
}

fn parse_format(raw: &str) -> Result<Format, String> {
    match raw {
        "human" => Ok(Format::Human),
        "json" | "repo-json" => Ok(Format::Json),
        "markdown" | "md" => Ok(Format::Markdown),
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

    fn parse_cli(args: &[&str]) -> Result<Command, String> {
        parse(args.iter().map(|arg| (*arg).to_string()).collect())
    }

    #[test]
    fn check_accepts_fail_on_gaps() -> Result<(), String> {
        let command = parse_cli(&["unsafe-review", "check", "--fail-on-gaps"])?;
        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };
        assert!(options.fail_on_gaps);
        Ok(())
    }

    #[test]
    fn check_rejects_base_and_diff_together() -> Result<(), String> {
        let result = parse_cli(&[
            "unsafe-review",
            "check",
            "--base",
            "origin/main",
            "--diff",
            "change.diff",
        ]);
        let Err(err) = result else {
            return Err("conflicting diff sources should fail".to_string());
        };
        assert!(err.contains("--base and --diff cannot be used together"));
        Ok(())
    }

    #[test]
    fn context_accepts_json_aliases() -> Result<(), String> {
        let command = parse_cli(&[
            "unsafe-review",
            "context",
            "UR-src-lib-rs-1-example",
            "--json",
        ])?;
        assert_eq!(
            command,
            Command::Context {
                root: PathBuf::from("."),
                id: "UR-src-lib-rs-1-example".to_string(),
            }
        );

        let command = parse_cli(&[
            "unsafe-review",
            "context",
            "--format",
            "json",
            "UR-src-lib-rs-1-example",
        ])?;
        assert_eq!(
            command,
            Command::Context {
                root: PathBuf::from("."),
                id: "UR-src-lib-rs-1-example".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn context_rejects_non_json_format() -> Result<(), String> {
        let result = parse_cli(&[
            "unsafe-review",
            "context",
            "--format",
            "markdown",
            "UR-src-lib-rs-1-example",
        ]);
        let Err(err) = result else {
            return Err("context markdown should fail".to_string());
        };
        assert!(err.contains("context only supports json format"));
        Ok(())
    }

    #[test]
    fn explain_rejects_multiple_card_ids() -> Result<(), String> {
        let result = parse_cli(&["unsafe-review", "explain", "first", "second"]);
        let Err(err) = result else {
            return Err("multiple ids should fail".to_string());
        };
        assert!(err.contains("multiple card ids supplied to explain"));
        Ok(())
    }
}
