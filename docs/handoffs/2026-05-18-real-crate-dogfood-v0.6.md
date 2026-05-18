# Real-crate dogfood v0.6 receipt

Date: 2026-05-18
Status: first real-crate dogfood slice landed
Owner: core/scanner/status

## What landed

This slice starts real-crate dogfood measurement and records a scanner
false-positive hardening fix found by that dogfood.

Dogfood repositories:

| Repo | Commit | Result |
|---|---|---|
| `servo/rust-smallvec` | `bc8a854926a8d940164f6c4ad4fc6efe51962e93` | completed with `--max-cards 50` |
| `bluss/arrayvec` | `1bc606d8c83a34b8fae9dd117bfeab10f90d2ca7` | completed with `--max-cards 50` |
| `BurntSushi/memchr` | `db1a77d4b556a1321e136ca0514e43e74ea5fcc3` | timed out before JSON output |

The first two completed runs exposed two noisy false positives:

- `extern crate ...;` was classified as an FFI boundary.
- `use core::ptr::copy_nonoverlapping;` was classified as an unsafe operation.

The scanner now distinguishes import/declaration text from executable unsafe
operation sites:

- `extern crate` is not an FFI boundary.
- `use` / `pub use` items are not operation calls.
- operation families detected by text fallback now require call-like syntax.
- `unsafe extern "C" { ... }` remains classified as an FFI boundary.

Regression proof was added through:

- scanner unit tests for `extern crate` and import-only unsafe operation paths
- `fixtures/imports_not_unsafe_operations`
- `fixtures/calibration.toml` false-positive-control coverage

## Dogfood observations

The before/after numbers below are top-50 capped repo inventory snapshots, not
full-repository rates.

### `rust-smallvec`

Before the fix:

```text
cards: 50
miri_unsupported: 2
contract_missing: 26
guard_missing: 18
requires_loom: 4
ffi operation cards: 2
```

The first cards included `extern crate test;`, `extern crate std;`, and
`use core::ptr::copy_nonoverlapping;` false positives.

After the fix:

```text
cards: 50
miri_unsupported: 0
contract_missing: 22
guard_missing: 22
guarded_unwitnessed: 2
requires_loom: 4
ffi operation cards: 0
```

The first cards are real unsafe review surfaces such as unsafe blocks, unsafe
functions, pointer arithmetic, raw pointer reads/writes, `Vec::set_len`, and
unsafe impl Send/Sync cards.

### `arrayvec`

Before the fix:

```text
cards: 50
miri_unsupported: 2
contract_missing: 46
guard_missing: 2
ffi operation cards: 2
```

The first cards included `extern crate arrayvec;` false positives from bench
files.

After the fix:

```text
cards: 50
miri_unsupported: 0
contract_missing: 48
guard_missing: 2
ffi operation cards: 0
```

The first cards are real unsafe review surfaces such as
`MaybeUninit::assume_init`, `Vec::set_len`, pointer arithmetic, raw pointer
operations, `str::from_utf8_unchecked`, and `zeroed`.

## Proof

Targeted local validation:

```bash
rtk cargo fmt --check
rtk cargo test -p unsafe-review-core scanner --locked
rtk cargo test -p unsafe-review-core fixture_card_goldens_match_rendered_json --locked
rtk cargo run --locked -p xtask -- check-calibration
```

Dogfood commands:

```bash
rtk cargo run --locked -p unsafe-review -- repo --root target/dogfood-work/smallvec --format json --max-cards 50 --out target/dogfood-work/smallvec.unsafe-review.after.json
rtk cargo run --locked -p unsafe-review -- repo --root target/dogfood-work/arrayvec --format json --max-cards 50 --out target/dogfood-work/arrayvec.unsafe-review.after.json
```

The dogfood reruns used a temporary `CARGO_TARGET_DIR` to avoid a Windows file
lock on the default debug binary.

## Current support posture

Real-crate dogfood is experimental.

The repo may claim:

- the first real-crate dogfood slice was run on `rust-smallvec` and `arrayvec`
- dogfood found and fixed import/declaration false positives
- false-positive regression coverage exists in fixtures and calibration
- dogfood output remains advisory static review evidence

The repo must not claim:

- calibrated false-positive or false-negative rates
- usable-alpha support-tier promotion
- full-repository coverage from top-50 capped snapshots
- memory-safety proof
- UB-free status
- witness execution
- blocking policy readiness

## Known limits

- Only two real crates completed in this slice.
- The successful dogfood snapshots were capped at 50 cards.
- `memchr` timed out before producing JSON output.
- No human audit was performed for every emitted card.
- These runs do not prove absence of missed unsafe seams.

## Next useful work

Continue dogfood before promotion:

- run uncapped or sampled repo inventories on additional unsafe-heavy crates
- measure card usefulness on real PR diffs, not only whole-repo snapshots
- record repeated false-positive categories as fixture regressions
- keep support tiers experimental until dogfood evidence justifies promotion

Defer:

- default blocking policy
- calibrated badge claims
- release-grade adoption claims
- automatic witness execution
