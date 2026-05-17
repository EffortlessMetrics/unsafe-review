#![forbid(unsafe_code)]

fn main() {
    if let Err(err) = unsafe_review_cli::run(std::env::args()) {
        eprintln!("unsafe-review: {err}");
        std::process::exit(2);
    }
}
