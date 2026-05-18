# Fixture calibration v0.6 receipt

Date: 2026-05-18
Status: fixture calibration proof index landed
Owner: core/fixtures/status

## What landed

The first calibration slice adds a machine-checkable proof index for the core
fixture claims:

```text
fixtures/calibration.toml
cargo xtask check-calibration
```

Merged PR:

- `#172 calibration: add fixture proof manifest`

The manifest records positive, negative, and false-positive-control cases for
the core review-card milestone:

- raw pointer alignment
- public unsafe function missing `# Safety`
- `MaybeUninit::assume_init`
- `Vec::set_len`
- `transmute` invalid-value review
- unsafe impl Send/Sync routing
- FFI sanitizer route
- safe code emits no card
- safe reference dereference emits no card
- comment/TODO alignment text does not count as guard evidence

`cargo xtask check-calibration` validates each manifest case against committed
fixture goldens:

- fixture exists
- claim is present
- case kind is known
- expected card count matches `expected.cards.json`
- expected card class is present for non-empty cases
- expected operation family is present when specified
- expected hazard is present when specified
- required core fixtures all have manifest cases
- positive, negative, and false-positive-control kinds are all represented

The check is now part of `cargo xtask check-pr`.

## Proof

The merged PR passed hosted checks before merge.

Targeted local validation:

```bash
rtk cargo test -p xtask calibration --locked
rtk cargo run --locked -p xtask -- check-calibration
rtk cargo run --locked -p xtask -- check-pr
```

The recurring workspace gate also passed:

```bash
rtk cargo fmt --check
rtk cargo check --workspace --all-targets --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk git diff --check
```

## Current support posture

Fixture calibration is experimental.

The repo may claim:

- the core fixture claims are indexed in `fixtures/calibration.toml`
- `cargo xtask check-calibration` validates manifest claims against committed
  fixture goldens
- `cargo xtask check-pr` includes the calibration manifest check
- the manifest covers positive, negative, and false-positive-control fixture
  classes

The repo must not claim:

- real-world calibration
- false-positive or false-negative rates
- usable-alpha support-tier promotion
- memory-safety proof
- UB-free status
- witness execution
- blocking policy readiness

## Known limits

- The manifest indexes curated fixtures only.
- It does not measure real unsafe-heavy crates.
- It does not prove absence of missed unsafe seams.
- It does not replace dogfood receipts or real PR outcome review.
- It does not justify default no-new-debt or blocking policy.

## Next useful work

Prefer dogfood and measurement before promotion:

- run `unsafe-review` on selected real unsafe-heavy crates
- collect false-positive and false-negative notes against real PRs
- use outcome comparison on saved repo snapshots from real dogfood runs
- keep support tiers experimental until dogfood evidence justifies promotion

Defer:

- blocking policy defaults
- release-grade calibration claims
- broad dashboard surfaces
- automatic witness execution
