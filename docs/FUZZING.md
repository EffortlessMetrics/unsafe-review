# Fuzzing

`unsafe-review` fuzzes the source analyzer as a black-box API boundary. The
current fuzz target builds a temporary crate from arbitrary UTF-8 Rust source,
runs a repo-scope analysis, and exercises the JSON, human, Markdown, detail, and
agent-packet renderers for any cards that are produced.

## Prerequisites

Install `cargo-fuzz` when you want coverage-guided fuzzing:

```bash
cargo install cargo-fuzz
```

## Smoke check

The PR check compiles the fuzz harness without starting a long-running fuzzing
session:

```bash
cargo xtask check-fuzz
```

You can also run the underlying command directly:

```bash
cargo check --manifest-path fuzz/Cargo.toml --bins
```

## Coverage-guided run

Run the analyzer target with the bundled seed corpus:

```bash
cargo fuzz run analyze_source -- -max_total_time=60
```

Useful longer local sessions include:

```bash
cargo fuzz run analyze_source -- -jobs=$(nproc) -workers=$(nproc)
cargo fuzz cmin analyze_source
```

Crashing inputs are written under `fuzz/artifacts/` and should be minimized
before promoting a regression seed into `fuzz/corpus/analyze_source/`.
