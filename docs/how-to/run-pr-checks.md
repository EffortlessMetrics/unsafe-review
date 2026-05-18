# How to run PR checks

Use this guide before opening or updating a pull request.

## Run the standard maintainer gate

```bash
cargo xtask check-pr
```

`check-pr` verifies required documentation files, proposal/spec/ADR indexes,
policy TOML files, support-tier names, and forbidden tracked generated artifacts.

## Run Rust checks when code changed

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For documentation-only changes, `cargo xtask check-pr` is the required repository
consistency check. Run the Rust checks too when the change touches Rust code,
Cargo metadata, fixtures that affect analyzer behavior, or policy consumed by
code.

## Interpret failures

- Missing index entry: add the new proposal, spec, or ADR file name to the
  matching `README.md`.
- Unknown support tier: use one of the tier names accepted by `xtask` and defined
  in [`../status/SUPPORT_TIERS.md`](../status/SUPPORT_TIERS.md).
- Tracked generated artifact: remove generated outputs such as badge directories,
  SARIF files, or profiling data from Git.
