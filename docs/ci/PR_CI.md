# PR and CI model

Default PR runs cheap static review:

```text
cargo fmt --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo run --locked -p xtask -- check-pr
```

Witness tools are routed, not run everywhere. Miri, sanitizers, Loom, and Kani
belong in targeted PR, nightly, or release lanes unless repo policy says
otherwise.
