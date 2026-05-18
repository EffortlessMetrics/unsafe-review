# PR and CI model

Default PR runs cheap static review. The GitHub Actions workflow also cancels superseded runs for the same ref, uses a locked dependency graph, caches Cargo downloads, and can be started manually with `workflow_dispatch` for ad hoc verification.

```text
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo run --locked -p xtask -- check-pr
unsafe-review check --base origin/main --format json
```

Witness tools are routed, not run everywhere. Miri, sanitizers, Loom, and Kani
belong in targeted PR, nightly, or release lanes unless repo policy says
otherwise.
