# Dogfood outcome index

Date: 2026-05-18
Status: experimental selected-corpus evidence
Source manifest: [`corpus.toml`](corpus.toml)
Machine-readable index: [`index.json`](index.json)

This index is a front panel for the real-crate dogfood corpus. It summarizes
which crates and PR diffs have been used to exercise `unsafe-review`, where the
saved-output artifacts are expected to live, and which outcome movement is
currently recorded.

## Trust Boundary

Dogfood corpus records are static unsafe contract review evidence. They are not
a proof of memory safety, not UB-free status, not a Miri result unless an exact
witness receipt is attached, and not calibrated precision or recall.

Generated scan outputs are intentionally recorded as `local_untracked` artifacts
under `target/dogfood-work/`. Re-run the command in `corpus.toml` when a fresh
local artifact is needed.

## Corpus Summary

| Measure | Count |
|---|---:|
| Repositories | 7 |
| Total targets | 25 |
| Capped repo snapshots | 7 |
| PR diff targets | 18 |
| Checked-in scan outputs | 0 |

## Repository Coverage

| Repository | Snapshot targets | PR diff targets | Primary exercise |
|---|---:|---:|---|
| `servo/rust-smallvec` | 1 | 4 | Raw pointer reads/writes, `Vec::set_len`, pointer arithmetic, unsafe impls, owner inference |
| `bluss/arrayvec` | 1 | 5 | `MaybeUninit`, `Vec::set_len`, raw pointer reads/writes, UTF-8, drop/deallocation cards |
| `BurntSushi/memchr` | 1 | 1 | SIMD target-feature contracts, pointer arithmetic, unchecked constructors |
| `rust-lang/hashbrown` | 1 | 7 | Large-file syntax scanning, `MaybeUninit`, pointer arithmetic, unchecked/infallible operations, dedupe |
| `tokio-rs/bytes` | 1 | 1 | `Vec::from_raw_parts`, slice construction, ownership-transfer review cards |
| `crossbeam-rs/crossbeam` | 1 | 0 | Unsafe Send/Sync, atomics, raw pointer, and ownership-transfer cards |
| `tokio-rs/mio` | 1 | 0 | Unsafe function call contracts, `Vec::set_len`, zeroed values, pointer operations, and unsafe Send/Sync route cards |

## Recorded Outcome Movement

| Target | Before | After | New | Resolved | Improved | Regressed | Unchanged | Notes |
|---|---|---|---:|---:|---:|---:|---:|---|
| `memchr-capped` target-feature contract evidence | `target/dogfood-work/memchr.unsafe-review.after-slice-end-pointer-evidence.json` | `target/dogfood-work/memchr.unsafe-review.after-target-feature-contract-evidence.json` | 0 | 0 | 10 | 0 | 40 | Documented `#[target_feature]` declarations moved from `guard_missing` to `guarded_unwitnessed`; no target-feature availability, site execution, or soundness claim. |

## Target Groups

### Capped Repo Snapshots

- `smallvec-capped`
- `arrayvec-capped`
- `memchr-capped`
- `hashbrown-capped`
- `bytes-capped`
- `crossbeam-capped`
- `mio-capped`

### PR Diffs

- `memchr-pr215`
- `smallvec-pr407`
- `smallvec-pr277`
- `smallvec-pr64`
- `smallvec-pr254`
- `arrayvec-pr308`
- `arrayvec-pr138`
- `arrayvec-pr187`
- `arrayvec-pr174`
- `arrayvec-pr288`
- `hashbrown-pr469`
- `hashbrown-pr501`
- `hashbrown-pr556`
- `hashbrown-pr657`
- `hashbrown-pr667`
- `hashbrown-pr692`
- `hashbrown-pr693`
- `bytes-pr826`

## Local Workflow

Validate the dogfood manifest:

```bash
rtk cargo run --locked -p xtask -- check-dogfood
```

Run one target by copying its command from [`corpus.toml`](corpus.toml). Compare
saved snapshots when an analyzer change is meant to improve card quality:

```bash
rtk cargo run --locked -p unsafe-review -- outcome \
  --before target/dogfood-work/before.json \
  --after target/dogfood-work/after.json \
  --format markdown \
  --out target/dogfood-work/outcome.md
```

Update this index only when the corpus manifest or recorded outcome evidence
changes. Do not use it to claim calibrated precision, safety, or policy
readiness.
