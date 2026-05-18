# PR and CI model

Default PR runs use one Rust workflow with independent, cache-backed jobs for the
checks below. Each job installs the pinned workspace Rust toolchain, runs with a
locked dependency graph where Cargo supports it, and cancels stale runs when a
new commit is pushed to the same ref.

```text
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p xtask -- check-pr
```

Witness tools are routed, not run everywhere. Miri, sanitizers, Loom, and Kani
belong in targeted PR, nightly, or release lanes unless repo policy says
otherwise.
