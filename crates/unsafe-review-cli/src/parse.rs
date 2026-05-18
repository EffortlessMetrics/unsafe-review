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
    let mut saw_base = false;
    let mut saw_diff = false;
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                idx += 1;
                options.root = PathBuf::from(value(&args, idx, "--root")?);
            }
            "--base" => {
                if saw_diff {
                    return Err("--base and --diff are mutually exclusive".to_string());
                }
                saw_base = true;
                idx += 1;
                options.base = Some(value(&args, idx, "--base")?.to_string());
            }
            "--diff" => {
                if saw_base {
                    return Err("--base and --diff are mutually exclusive".to_string());
                }
                saw_diff = true;
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
            "--json" => format = Format::Json,
            "--markdown" => format = Format::Markdown,
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
                    return Err("context only supports JSON output".to_string());
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

fn parse_format(raw: &str) -> Result<Format, String> {
    match raw {
        "human" => Ok(Format::Human),
        "json" | "repo-json" => Ok(Format::Json),
        "markdown" | "md" => Ok(Format::Markdown),
        other => Err(format!("unknown format `{other}`")),
    }
}

fn set_id(id: &mut Option<String>, value: &str, command: &str) -> Result<(), String> {
    if id.replace(value.to_string()).is_some() {
        return Err(format!("multiple {command} card ids supplied"));
    }
    Ok(())
}

fn value<'a>(args: &'a [String], idx: usize, flag: &str) -> Result<&'a str, String> {
    args.get(idx)
        .map(|value| value.as_str())
        .ok_or_else(|| format!("missing value for {flag}"))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::command::{Command, Format};
    use std::path::PathBuf;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn context_accepts_json_alias_after_card_id() -> Result<(), String> {
        let command = parse(args(&["unsafe-review", "context", "UR-card", "--json"]))?;
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
    fn context_rejects_non_json_format() {
        let err = parse(args(&[
            "unsafe-review",
            "context",
            "UR-card",
            "--format",
            "markdown",
        ]));
        assert_eq!(err, Err("context only supports JSON output".to_string()));
    }

    #[test]
    fn check_rejects_base_and_diff_together() {
        let err = parse(args(&[
            "unsafe-review",
            "check",
            "--base",
            "origin/main",
            "--diff",
            "change.diff",
        ]));
        assert_eq!(
            err,
            Err("--base and --diff are mutually exclusive".to_string())
        );
    }

    #[test]
    fn explain_rejects_multiple_card_ids() {
        let err = parse(args(&["unsafe-review", "explain", "UR-one", "UR-two"]));
        assert_eq!(err, Err("multiple explain card ids supplied".to_string()));
    }

    #[test]
    fn explain_accepts_json_alias() -> Result<(), String> {
        let command = parse(args(&["unsafe-review", "explain", "UR-card", "--json"]))?;
        assert_eq!(
            command,
            Command::Explain {
                root: PathBuf::from("."),
                id: "UR-card".to_string(),
                format: Format::Json
            }
        );
        Ok(())
    }
}
