#![forbid(unsafe_code)]

fn main() {
    let mut args = std::env::args().collect::<Vec<_>>();
    if args.get(1).is_some_and(|arg| arg == "unsafe-review") {
        let _removed = args.remove(1);
    }
    if let Err(err) = unsafe_review_cli::run(args) {
        eprintln!("cargo-unsafe-review: {err}");
        std::process::exit(2);
    }
}
