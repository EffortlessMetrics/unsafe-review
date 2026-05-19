# Crates.io initial publication readiness

Date: 2026-05-18

Scope: prepare the `unsafe-review` workspace for the first crates.io
publication without changing analyzer behavior or promoting support-tier claims.

## Publish surface

The initial public surface is:

```text
unsafe-review-core  # SDK / ReviewCard engine
unsafe-review-cli   # CLI adapter and cargo subcommand binary
unsafe-review       # product facade / cargo install handle
```

`xtask` is internal automation and is not part of the publish surface.

## Readiness changes

This readiness pass:

```text
replaces placeholder MIT and Apache-2.0 license files with full license text
adds crate-local README files for unsafe-review-core and unsafe-review-cli
adds docs.rs documentation metadata for all three published crates
keeps path + version dependency pairs for workspace-local product crates
records the manual publish order and proof commands
```

It does not change ReviewCard generation, evidence classification, witness
receipts, policy behavior, PR artifacts, LSP projection, or agent packets.

## Publish order

Publish in dependency order:

```bash
cargo publish -p unsafe-review-core
cargo publish -p unsafe-review-cli
cargo publish -p unsafe-review
```

Do not publish the facade first. The facade depends on both product crates, and
the CLI crate depends on the core crate.

## Required pre-publish proof

Run from a clean `main` checkout after this readiness PR merges:

```bash
cargo fmt --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run --locked -p xtask -- check-pr
cargo run --locked -p xtask -- check-calibration
cargo package -p unsafe-review-core --list
cargo package -p unsafe-review-cli --list
cargo package -p unsafe-review --list
cargo publish -p unsafe-review-core --dry-run
git diff --check
```

For the first publication, downstream dry-runs depend on upstream crates already
existing in the crates.io index. Run them as staged publish gates:

```bash
cargo publish -p unsafe-review-core --dry-run
cargo publish -p unsafe-review-core

cargo publish -p unsafe-review-cli --dry-run
cargo publish -p unsafe-review-cli

cargo publish -p unsafe-review --dry-run
cargo publish -p unsafe-review
```

If a downstream dry-run is attempted before its local product dependencies have
been published, Cargo is expected to report that the upstream package is not yet
found in the crates.io index.

Confirm crate-name availability immediately before publishing:

```bash
cargo search unsafe-review-core --limit 10
cargo search unsafe-review-cli --limit 10
cargo search unsafe-review --limit 10
```

The authoritative registry check remains the publish response.

## Trust boundary

The first publication is an experimental static unsafe-review evidence tool. It
must not be described as calibrated blocking policy, a UB prover, a Miri
replacement, a Miri-clean result, or a repository safety claim.

Witness receipt import remains scoped evidence. It does not discharge missing
contracts, guards, reach evidence, or policy readiness by itself.

## Post-publish receipt

After successful publication, open a separate documentation PR recording:

```text
published crate versions
crate URLs
publish commands and order
install smoke command
fixture smoke command
known limits
next lane
```

Tag `v0.1.0` only after all three crate publishes and the install smoke pass.

## Current pre-publish proof refresh

After the dogfood-calibrated evidence lane closeout, the following
non-publishing checks were refreshed from current `origin/main`:

```bash
rtk cargo package -p unsafe-review-core --list
rtk cargo package -p unsafe-review-cli --list
rtk cargo package -p unsafe-review --list
rtk cargo search unsafe-review-core --limit 10
rtk cargo search unsafe-review-cli --limit 10
rtk cargo search unsafe-review --limit 10
rtk cargo publish -p unsafe-review-core --dry-run
rtk cargo publish -p unsafe-review-cli --dry-run
rtk cargo publish -p unsafe-review --dry-run
```

Observed result:

- all three `cargo package --list` commands completed
- all three `cargo search` commands returned no matching rows for the exact
  crate names
- `unsafe-review-core` packaged, verified, and reached the dry-run upload
  boundary successfully
- `unsafe-review-cli` dry-run stopped because `unsafe-review-core` is not yet in
  the crates.io index
- `unsafe-review` dry-run stopped because `unsafe-review-cli` is not yet in the
  crates.io index

Those downstream dry-run failures are expected before the dependency crates are
actually published. They confirm the documented publish order:

```text
unsafe-review-core -> unsafe-review-cli -> unsafe-review
```

No crates were published, no tag was created, and no release receipt was
recorded by this proof refresh.
