# PR and CI model

Default PR runs cheap static review on the pinned Rust toolchain:

```text
cargo fmt --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo xtask check-pr
```

The workflow keeps permissions read-only, cancels superseded runs for the same
branch or PR, restores Cargo build caches on every run, and only saves cache
entries from trusted pushes to `main` or `master`. Dependabot also tracks both
GitHub Actions and Cargo dependency updates so CI maintenance stays visible.

Witness tools are routed, not run everywhere. Miri, sanitizers, Loom, and Kani
belong in targeted PR, nightly, or release lanes unless repo policy says
otherwise.
