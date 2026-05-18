# Mutation testing

Mutation testing is the slow lane for checking whether tests fail when product
logic is changed in small, mechanical ways. The repository uses `cargo-mutants`
for this lane.

## Local command

```bash
cargo install cargo-mutants
cargo mutants --workspace
```

The default configuration lives in `.cargo/mutants.toml`. It limits the mutation
scope to product crates and skips documentation, policy ledgers, fixtures, and
build artifacts so local runs stay focused on code whose behavior is exercised by
unit and golden fixture tests.

## CI model

Pull request CI stays cheap and deterministic. Mutation testing runs as a
non-blocking scheduled workflow and can also be started manually from GitHub
Actions. If a mutation run finds survivors, add focused
tests near the affected logic before changing the mutation configuration.
