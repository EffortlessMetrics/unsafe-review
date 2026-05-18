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

## Mutation testing

Mutation testing is intentionally outside the default PR gate because it is much
slower than the compile, lint, and unit-test lane. Developers can run the same
workspace-oriented defaults locally with:

```text
cargo install cargo-mutants
cargo mutants
```

The checked-in `.cargo/mutants.toml` file keeps the default run focused on
library and CLI logic while excluding entry-point shims with no product logic.
