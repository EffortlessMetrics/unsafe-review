# PR and CI model

Default PR runs cheap static review:

```text
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
unsafe-review check --base origin/main --format json
```

Witness tools are routed, not run everywhere. Miri, sanitizers, Loom, and Kani
belong in targeted PR, nightly, or release lanes unless repo policy says
otherwise.

## Mutation testing lane

Mutation testing is intentionally not part of default PR CI. Run the non-blocking scheduled or
manual GitHub Actions mutation lane, or run locally:

```text
cargo mutants --workspace
```

The mutation lane uses `.cargo/mutants.toml` to focus on product Rust crates and
avoid mutating documentation, policy, fixture, and build-artifact paths.
