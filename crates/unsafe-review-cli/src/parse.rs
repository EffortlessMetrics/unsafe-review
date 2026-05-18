use crate::command::{CheckOptions, Command, Format};
use std::path::PathBuf;

pub(crate) fn parse(args: Vec<String>) -> Result<Command, String> {
    let mut rest = args.into_iter().skip(1).collect::<Vec<_>>();
    if rest.is_empty() {
        return Ok(Command::Help);
    }
    let command = rest.remove(0);
    if rest
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return Ok(Command::Help);
    }
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
                options.diff = Some(PathBuf::from(value(&args, idx, "--diff")?));
            }
            arg if arg.starts_with("--diff=") => {
                options.diff = Some(PathBuf::from(inline_value(arg, "--diff")?));
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
    if options.base.is_some() && options.diff.is_some() {
        return Err("--base and --diff are mutually exclusive".to_string());
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
            value => assign_positional(&mut id, value, "card id")?,
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
                parse_context_format(value(&args, idx, "--format")?)?;
            }
            arg if arg.starts_with("--format=") => {
                parse_context_format(inline_value(arg, "--format")?)?;
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown context argument `{value}`"));
            }
            value => assign_positional(&mut id, value, "card id")?,
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

fn parse_context_format(raw: &str) -> Result<(), String> {
    match parse_format(raw)? {
        Format::Json => Ok(()),
        Format::Human | Format::Markdown => {
            Err("context packets are only available as json".to_string())
        }
    }
}

fn parse_max_cards(raw: &str) -> Result<usize, String> {
    raw.parse::<usize>()
        .map_err(|err| format!("invalid --max-cards `{raw}`: {err}"))
}

fn assign_positional(slot: &mut Option<String>, value: &str, name: &str) -> Result<(), String> {
    if slot.replace(value.to_string()).is_some() {
        return Err(format!("multiple {name} values provided"));
    }
    Ok(())
}

fn inline_value<'a>(arg: &'a str, flag: &str) -> Result<&'a str, String> {
    let value = arg
        .strip_prefix(flag)
        .and_then(|rest| rest.strip_prefix('='))
        .ok_or_else(|| format!("missing value for {flag}"))?;
    if value.is_empty() {
        return Err(format!("missing value for {flag}"));
    }
    Ok(value)
}

fn value<'a>(args: &'a [String], idx: usize, flag: &str) -> Result<&'a str, String> {
    let Some(value) = args.get(idx).map(|value| value.as_str()) else {
        return Err(format!("missing value for {flag}"));
    };
    if value.starts_with('-') {
        return Err(format!("missing value for {flag}"));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Command, String> {
        parse(args.iter().map(|arg| (*arg).to_string()).collect())
    }

    #[test]
    fn parses_equals_style_check_flags() -> Result<(), String> {
        let command = parse_args(&[
            "unsafe-review",
            "check",
            "--root=fixtures/raw_pointer_deref",
            "--diff=change.diff",
            "--format=json",
            "--out=cards.json",
            "--max-cards=7",
        ])?;

        let Command::Check(options) = command else {
            return Err("expected check command".to_string());
        };
        assert_eq!(options.root, PathBuf::from("fixtures/raw_pointer_deref"));
        assert_eq!(options.diff, Some(PathBuf::from("change.diff")));
        assert_eq!(options.format, Format::Json);
        assert_eq!(options.out, Some(PathBuf::from("cards.json")));
        assert_eq!(options.max_cards, Some(7));
        Ok(())
    }

    #[test]
    fn rejects_ambiguous_diff_sources() -> Result<(), String> {
        let result = parse_args(&[
            "unsafe-review",
            "check",
            "--base",
            "origin/main",
            "--diff",
            "change.diff",
        ]);
        let Err(err) = result else {
            return Err("base and diff should be mutually exclusive".to_string());
        };

        assert!(err.contains("mutually exclusive"));
        Ok(())
    }

    #[test]
    fn accepts_readme_context_json_flag() -> Result<(), String> {
        let command = parse_args(&[
            "unsafe-review",
            "context",
            "UR-src-lib-rs-1-raw-pointer-read",
            "--json",
        ])?;

        assert_eq!(
            command,
            Command::Context {
                root: PathBuf::from("."),
                id: "UR-src-lib-rs-1-raw-pointer-read".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn rejects_duplicate_card_ids() -> Result<(), String> {
        let result = parse_args(&["unsafe-review", "explain", "UR-one", "UR-two"]);
        let Err(err) = result else {
            return Err("duplicate card ids should fail".to_string());
        };

        assert!(err.contains("multiple card id"));
        Ok(())
    }

    #[test]
    fn treats_subcommand_help_as_help() -> Result<(), String> {
        let command = parse_args(&["unsafe-review", "check", "--help"])?;

        assert_eq!(command, Command::Help);
        Ok(())
    }
}
