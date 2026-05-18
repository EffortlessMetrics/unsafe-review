# Fuzzing

`unsafe-review` has a `cargo-fuzz` harness for the core analysis pipeline. The
harness builds an ephemeral Rust workspace from fuzz input, runs both diff and
repo-scan analysis paths, renders JSON for every successful analysis result, and
optionally exercises human and Markdown rendering.

## Prerequisites

```bash
cargo install cargo-fuzz
```

`cargo-fuzz` drives LLVM libFuzzer and usually runs with a nightly toolchain. The
regular repository checks stay stable-only; fuzzing is an opt-in robustness pass.

## Run the analysis harness

```bash
cargo fuzz run analysis
```

Useful bounded runs while developing:

```bash
cargo fuzz run analysis -- -max_total_time=60
cargo fuzz run analysis -- -runs=1000
```

## Corpus and artifacts

Seed inputs live under `fuzz/corpus/analysis/`. Crashes and minimized reproducers
are written under `fuzz/artifacts/` by `cargo-fuzz`; those generated artifacts
should not be committed unless they are promoted into a deterministic regression
test or a small seed corpus entry.
