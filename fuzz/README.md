# Fuzzing

This directory contains `cargo-fuzz` targets for unsafe-review. The fuzz package
is intentionally excluded from the main workspace so normal `cargo check
--workspace` and release builds do not pull in libFuzzer dependencies.

## Targets

- `analyze_diff`: fuzzes unified diff handling through the public analysis API,
  then renders every supported output format and per-card detail/agent packets.
  Arbitrary non-diff input is wrapped in a synthetic diff for
  `fixtures/raw_pointer_alignment/src/lib.rs`, keeping the target focused on
  diff-coordinate parsing and card rendering instead of falling back to a full
  repository scan.

## Running locally

Install the runner once:

```sh
cargo install cargo-fuzz
```

Run a short smoke test:

```sh
cargo +nightly fuzz run analyze_diff -- -runs=256
```

Run a longer session and keep any new minimized regressions under
`fuzz/artifacts/analyze_diff/` for triage:

```sh
cargo +nightly fuzz run analyze_diff
```
