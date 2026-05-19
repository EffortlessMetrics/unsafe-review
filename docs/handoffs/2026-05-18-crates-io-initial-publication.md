# Crates.io initial publication receipt

Date: 2026-05-18

Scope: record the manual `0.1.0` crates.io publication for the initial
experimental `unsafe-review` publish surface.

## Published crates

Published in dependency order:

| Crate | Version | Registry URL | docs.rs |
|---|---:|---|---|
| `unsafe-review-core` | `0.1.0` | <https://crates.io/crates/unsafe-review-core/0.1.0> | <https://docs.rs/unsafe-review-core/0.1.0/unsafe_review_core/> |
| `unsafe-review-cli` | `0.1.0` | <https://crates.io/crates/unsafe-review-cli/0.1.0> | <https://docs.rs/unsafe-review-cli/0.1.0/unsafe_review_cli/> |
| `unsafe-review` | `0.1.0` | <https://crates.io/crates/unsafe-review/0.1.0> | <https://docs.rs/unsafe-review/0.1.0/unsafe_review/> |

The published git tag is:

```text
v0.1.0 -> 6df02339e46139102872f48e52e71fd3d2a0faf1
```

## Pre-publish verification

Run from `origin/main` commit `6df02339e46139102872f48e52e71fd3d2a0faf1`:

```bash
rtk cargo fmt --check
rtk cargo check --workspace --all-targets --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo run --locked -p xtask -- check-pr
rtk cargo run --locked -p xtask -- check-calibration
rtk cargo run --locked -p xtask -- check-dogfood
rtk cargo package -p unsafe-review-core --list
rtk cargo package -p unsafe-review-cli --list
rtk cargo package -p unsafe-review --list
rtk cargo publish -p unsafe-review-core --dry-run
```

Observed result:

- formatting, workspace check, clippy, workspace tests, `check-pr`,
  `check-calibration`, and `check-dogfood` passed
- package lists completed for all three publishable crates
- `unsafe-review-core` packaged, verified, and reached the dry-run upload
  boundary successfully
- crate-name searches initially returned no exact published names

## Publish commands

The first publish was manual and topological:

```bash
rtk cargo publish -p unsafe-review-core
rtk cargo search unsafe-review-core --limit 10
rtk cargo publish -p unsafe-review-cli --dry-run
rtk cargo publish -p unsafe-review-cli
rtk cargo search unsafe-review-cli --limit 10
rtk cargo publish -p unsafe-review --dry-run
rtk cargo publish -p unsafe-review
rtk cargo search unsafe-review --limit 10
```

Observed result:

- `unsafe-review-core 0.1.0` uploaded and Cargo waited for registry
  availability
- `unsafe-review-cli 0.1.0` dry-run downloaded published
  `unsafe-review-core 0.1.0`, then the crate uploaded and Cargo waited for
  registry availability
- `unsafe-review 0.1.0` dry-run downloaded published `unsafe-review-cli 0.1.0`,
  then the facade crate uploaded and Cargo waited for registry availability
- `cargo search unsafe-review --limit 10` returned all three published crates

## Post-publish smoke

The published facade was installed from crates.io:

```bash
rtk cargo install unsafe-review --locked --force
```

Observed result:

```text
installed `unsafe-review.exe` v0.1.0 under the Cargo binary directory
```

Product smoke commands:

```bash
rtk unsafe-review --help
rtk unsafe-review check --root fixtures/raw_pointer_alignment --diff fixtures/raw_pointer_alignment/change.diff --format json
rtk unsafe-review check --root fixtures/raw_pointer_alignment --diff fixtures/raw_pointer_alignment/change.diff --format pr-summary --out target/unsafe-review/pr-summary.md
rtk unsafe-review repo --root fixtures/raw_pointer_alignment --format markdown
```

Observed result:

- `unsafe-review --help` printed the expected command surface and trust boundary
- JSON fixture smoke emitted one `guard_missing` raw pointer alignment card
- PR summary smoke wrote `target/unsafe-review/pr-summary.md`
- repo Markdown smoke emitted static repo posture with one open guard gap

docs.rs pages were checked with `curl -I` and returned HTTP `200` for all three
crate documentation URLs listed above.

## Trust boundary

`0.1.0` is an experimental static unsafe-review evidence tool.

It is not:

```text
memory-safety proof
UB-free claim
Miri-clean claim
site-execution proof
target-feature availability proof
default policy gate
automatic PR comment publisher
automatic unsafe-code repair tool
```

Witness receipts remain scoped imported evidence. They do not prove repository
safety, and they do not discharge missing contracts, guards, reach evidence, or
policy readiness unless the matching receipt explicitly supports that lane and
strength.

## Known limits

- Support tiers remain experimental/advisory unless the detailed support ledger
  says otherwise.
- The schema is `0.1` but not yet a long-term compatibility promise.
- Real-crate dogfood is useful but not calibrated precision/recall.
- No witness tools are executed by default.
- No default no-new-debt or blocking CI policy is enabled.
- Live LSP/editor integration remains planned; current LSP output is saved JSON.
- Agent packets are copy-only and do not execute repairs.

## Next lane

After this publication receipt merges, the next work should stabilize the
external first-use path and then continue the dogfood-calibrated evidence loop:

```text
install and first-use docs
doctor output polish
docs.rs/package sanity follow-up if needed
dogfood outcome index
receipt audit hardening
repo inventory JSON schema
advisory no-new-debt report
```

Do not start default blocking policy, automatic comments, witness execution, or
automatic unsafe rewrites from this publication alone.
