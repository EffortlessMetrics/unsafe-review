#![forbid(unsafe_code)]
mod command;
mod execute;
mod parse;

pub fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let command = parse::parse(args.into_iter().collect())?;
    execute::execute(command)
}
