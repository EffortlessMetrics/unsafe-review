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
| `BurntSushi/memchr` | `db1a77d4b556a1321e136ca0514e43e74ea5fcc3` | completed with `--max-cards 50` after capped-scan hardening |

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

A follow-up dogfood pass on `memchr` fixed the timeout and another false
positive:

- capped repo scans now stop once `--max-cards` cards are emitted
- Rust file discovery prioritizes Cargo-like source roots before miscellaneous
  `.rs` data files
- `#[cfg(target_feature = "...")]` predicates are not classified as target
  feature obligations
- real `#[target_feature(enable = "...")]` attributes remain classified as
  target-feature review surfaces

Additional regression proof was added through:

- workspace discovery ordering tests
- capped repo scan tests
- scanner unit tests for `cfg(target_feature)` versus `target_feature`
  attributes
- `fixtures/cfg_target_feature_not_operation`
- `fixtures/calibration.toml` false-positive-control coverage

A real PR-diff dogfood pass on `BurntSushi/memchr#215` exposed and fixed an
evidence-quality gap:

- operation cards inside an unsafe function now include the enclosing owner
  doc block in their evidence context
- an operation inside a documented `unsafe fn` can inherit the owner
  `# Safety` contract instead of reporting a separate `contract_missing` gap

Regression proof was added with a repo-mode analyzer test for pointer
arithmetic inside a documented unsafe function.

A real PR-diff dogfood pass on `servo/rust-smallvec#407` exposed and fixed an
owner-inference gap:

- owner inference now ignores comment lines while scanning backward
- comment prose such as `` `Drop` impl would drop `` is no longer parsed as an
  `impl would` owner
- witness commands now use the real owner when one is available

Regression proof was added with a scanner unit test for comment text during
owner inference.

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

### `memchr`

Before capped-scan hardening:

```text
result: timed out before JSON output
largest early discovered file: benchmarks/haystacks/code/rust-library.rs
```

After capped-scan hardening, before the `cfg(target_feature)` fix:

```text
elapsed_seconds: 7.5
cards: 50
miri_unsupported: 0
contract_missing: 23
guard_missing: 27
target_feature operation cards: 16
```

The first cards included `#[cfg(target_feature = "neon")]` and
`#[cfg(not(target_feature = "neon"))]` false positives.

After the `cfg(target_feature)` fix:

```text
elapsed_seconds: 4.59
cards: 50
miri_unsupported: 0
contract_missing: 20
guard_missing: 30
target_feature operation cards: 10
```

Remaining target-feature cards in the top-50 capped output correspond to real
`#[target_feature(enable = "neon")]` attributes.

### `memchr#215`

PR: `https://github.com/BurntSushi/memchr/pull/215`

The PR fixes minor UB flagged by Miri by adding a targeted `#[cfg(miri)]` test,
strengthening `# Safety` docs, and changing the guard around pointer arithmetic
inside `find_in_chunk`.

Before owner-contract inheritance:

```text
elapsed_seconds: 0.81
changed_rust_files: 2
cards: 2
contract_missing: 1
guard_missing: 1
guarded_unwitnessed: 0
```

The pointer arithmetic card for `let cur = cur.add(offset);` was reported as
`contract_missing` even though the enclosing unsafe function's `# Safety` docs
described the `cur.add(offset) < end` obligation.

After owner-contract inheritance:

```text
elapsed_seconds: 2.25
changed_rust_files: 2
cards: 2
contract_missing: 0
guard_missing: 1
guarded_unwitnessed: 1
```

The pointer arithmetic card now recognizes the enclosing `# Safety` contract and
local guard evidence, then routes the remaining gap to Miri/cargo-careful as a
witness need.

### `rust-smallvec#407`

PR: `https://github.com/servo/rust-smallvec/pull/407`

The PR fixes a use-after-free in `DrainFilter::keep_rest` for zero-capacity
`SmallVec`s by changing the ZST guard and adding a Miri-targeted regression
test.

Before comment-aware owner inference:

```text
elapsed_seconds: 33.48
changed_rust_files: 2
cards: 1
contract_missing: 1
owner: would
verify: cargo +nightly miri test would
```

The owner was inferred from comment prose:

```text
// Normally `Drop` impl would drop [tail] ...
```

After comment-aware owner inference:

```text
elapsed_seconds: 34.8
changed_rust_files: 2
cards: 1
contract_missing: 1
owner: keep_rest
verify: cargo +nightly miri test keep_rest
```

The card class did not change. The improvement is that the card now points the
reviewer and witness route at the real owner instead of prose.

### `rust-smallvec#64`

PR: `https://github.com/servo/rust-smallvec/pull/64`

The PR simplifies `SmallVec::pop` by replacing `ptr::replace(...,
mem::uninitialized())` with a raw `ptr::read(end_ptr)` and then shrinking the
length with `set_len(last_index)`.

Before last-index shrink evidence:

```text
changed_rust_files: 1
cards: 3
contract_missing: 3
operation families: pointer_arithmetic, raw_pointer_read, vec_set_len
set_len discharge: missing local guard evidence
```

After last-index shrink evidence:

```text
changed_rust_files: 1
cards: 3
contract_missing: 3
operation families: pointer_arithmetic, raw_pointer_read, vec_set_len
set_len discharge: all inferred obligations have visible local guard evidence
```

The card class did not change because the changed unsafe block still lacks local
`SAFETY:` contract text. The useful improvement is narrower: the `Vec::set_len`
card now recognizes the local `self.len == 0` guard plus
`last_index = self.len - 1` as shrink evidence, so the remaining prompt is
contract/witness review rather than a missing initialized-range guard.

### `rust-smallvec#254`

PR: `https://github.com/servo/rust-smallvec/pull/254`

The PR fixes a potential buffer overflow in `insert_many` by restructuring the
unsafe insertion path and adding a regression test.

Dogfood output:

```text
elapsed_seconds: 17.6
changed_rust_files: 2
cards: 9
contract_missing: 9
guard_missing: 0
operation families: pointer_arithmetic, vec_set_len, raw_pointer_write
owner: insert_many
```

This run did not receive a scanner patch. It is useful because the cards point
at a dense changed unsafe block where pointer arithmetic, `ptr::copy`,
`ptr::write`, and `set_len` operations moved without local `SAFETY:` contract
text. That is a legitimate PR-review prompt: the tool is not claiming the fix
is wrong, only that the changed unsafe seam needs explicit contract evidence and
the usual witness route.

### `arrayvec#308`

PR: `https://github.com/bluss/arrayvec/pull/308`

The PR fixes a double-free for ZSTs with `Drop` during `.extend()` by changing
the write path and adding a safe-API regression test.

Dogfood output:

```text
elapsed_seconds: 9.2
changed_rust_files: 2
cards: 1
contract_missing: 1
guard_missing: 0
operation_family: raw_pointer_write
owner: extend_from_iter
verify: cargo +nightly miri test extend_from_iter
```

This run did not require a scanner fix. It is useful because the card points to
the changed raw pointer write inside `extend_from_iter`, recognizes visible
guard evidence, and routes the remaining missing contract/witness work to the
owner-specific Miri/cargo-careful path.

### `arrayvec#138`

PR: `https://github.com/bluss/arrayvec/pull/138`

The PR changes `ArrayString::try_push` and `encode_utf8` to write UTF-8 bytes
through raw pointers into possibly uninitialized storage.

Before attributed unsafe-fn dedupe:

```text
changed_rust_files: 2
cards: 9
contract_missing: 9
operation families: pointer_arithmetic, vec_set_len, raw_pointer_write, unknown
duplicate: attributed unsafe fn `write` emitted at both attribute and signature lines
```

After attributed unsafe-fn dedupe:

```text
changed_rust_files: 2
cards: 8
contract_missing: 8
operation families: pointer_arithmetic, vec_set_len, raw_pointer_write, unknown
```

This run exposed and fixed a scanner duplicate: syntax-backed attributed
`unsafe fn` declarations should not be emitted again by fallback line scanning
on the signature line. The remaining cards are legitimate advisory prompts for
missing contract evidence around the raw-pointer UTF-8 write path, including the
public unsafe `encode_utf8` API.

### `arrayvec#288`

PR: `https://github.com/bluss/arrayvec/pull/288`

The PR reduces `unsafe` usage in `ArrayString` and adds safety comments around
remaining `set_len` uses.

Dogfood output:

```text
elapsed_seconds: 5.34
changed_rust_files: 1
cards: 9
contract_missing: 0
guard_missing: 9
operation families: vec_set_len, unknown
```

This run did not receive a scanner patch. It captured a real limitation in the
current guard model: `Vec::set_len` evidence was sparse for initialization
patterns such as `MaybeUninit::new` loops, const-generic `CAP` capacity facts,
and shrink operations like `truncate`, `clear`, and `pop` where the initialized
range obligation is not the right shape. A later fixture-backed follow-up now
recognizes visible `MaybeUninit::new` initialization loops and const `CAP`
capacity evidence for `Vec::set_len`; shrink and broader initialization
patterns remain limited. The cards are still useful as advisory review prompts,
but this sample should not be used as support-tier promotion evidence.

Follow-up rerun after the fixture-backed `Vec::set_len` initialization evidence
improvement:

```text
elapsed_seconds: 9.3
changed_rust_files: 1
cards: 9
contract_missing: 0
guard_missing: 7
guarded_unwitnessed: 2
operation families: vec_set_len, unknown
```

The two improved cards are still advisory only:

```text
from_byte_string  line 140  vec_set_len  guarded_unwitnessed
try_push_str      line 316  vec_set_len  guarded_unwitnessed
```

They moved out of `guard_missing` because the local context contains visible
initialization evidence and capacity evidence. The remaining seven cards still
need better modeling or reviewer inspection, especially shrink-style `set_len`
uses and the direct unsafe `set_len` API card.

Follow-up rerun after adding fixture-backed `set_len(0)` clear evidence:

```text
elapsed_seconds: 4.6
changed_rust_files: 1
cards: 9
contract_missing: 0
guard_missing: 6
guarded_unwitnessed: 3
operation families: vec_set_len, unknown
```

The additional improved card is still advisory only:

```text
clear  line 436  vec_set_len  guarded_unwitnessed  self.set_len(0);
```

It moved out of `guard_missing` because setting length to zero cannot exceed
capacity and introduces no initialized extended range. The remaining six cards
still need better modeling or reviewer inspection, especially non-zero
shrink-style `set_len` uses, the `encode_utf8` write pattern, and the direct
unsafe `set_len` API card.

Follow-up rerun after adding fixture-backed non-zero shrink evidence:

```text
elapsed_seconds: 6.0
changed_rust_files: 1
cards: 9
contract_missing: 0
guard_missing: 3
guarded_unwitnessed: 5
unsafe_unreached: 1
operation families: vec_set_len, unknown
```

The additional improved cards are still advisory only:

```text
pop       line 351  vec_set_len  guarded_unwitnessed
truncate  line 387  vec_set_len  guarded_unwitnessed
remove    line 426  vec_set_len  unsafe_unreached
```

They moved out of `guard_missing` because local code shows `new_len` is no
greater than the current initialized length, so these calls do not introduce an
initialized extended range and cannot exceed the current length. The `remove`
card remains actionable as `unsafe_unreached` because the static reach search
did not find a related test mention. The remaining three `guard_missing` cards
are the `encode_utf8` unsafe block, the matching `set_len(len + n)` write
pattern, and the direct public unsafe `set_len` API card.

## Proof

Targeted local validation:

```bash
rtk cargo fmt --check
rtk cargo test -p unsafe-review-core scanner --locked
rtk cargo test -p unsafe-review-core workspace --locked
rtk cargo test -p unsafe-review-core capped_repo_scan --locked
rtk cargo test -p unsafe-review-core owner_safety --locked
rtk cargo test -p unsafe-review-core fixture_card_goldens_match_rendered_json --locked
rtk cargo run --locked -p xtask -- check-calibration
```

Dogfood commands:

```bash
rtk cargo run --locked -p unsafe-review -- repo --root target/dogfood-work/smallvec --format json --max-cards 50 --out target/dogfood-work/smallvec.unsafe-review.after.json
rtk cargo run --locked -p unsafe-review -- repo --root target/dogfood-work/arrayvec --format json --max-cards 50 --out target/dogfood-work/arrayvec.unsafe-review.after.json
rtk cargo run --locked -p unsafe-review -- repo --root target/dogfood-work/memchr --format json --max-cards 50 --out target/dogfood-work/memchr.unsafe-review.after-cap-targetfeature.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/memchr --diff target/dogfood-work/memchr-pr215.raw.diff --format json --max-cards 20 --out target/dogfood-work/memchr-pr215.owner-contract.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/smallvec --diff target/dogfood-work/smallvec-pr407.raw.diff --format json --max-cards 20 --out target/dogfood-work/smallvec-pr407.owner-fix.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/smallvec --diff target/dogfood-work/smallvec-pr64.raw.diff --format json --max-cards 20 --out target/dogfood-work/smallvec-pr64.after-last-index-shrink.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr308.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr308.unsafe-review.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr138.raw.diff --format json --max-cards 30 --out target/dogfood-work/arrayvec-pr138.after-attributed-dedupe.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.unsafe-review.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-setlen-init.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-setlen-zero.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-setlen-shrink.json
```

The dogfood reruns used a temporary `CARGO_TARGET_DIR` to avoid a Windows file
lock on the default debug binary.

## Current support posture

Real-crate dogfood is experimental.

The repo may claim:

- the first real-crate dogfood slice was run on `rust-smallvec` and `arrayvec`
- a capped `memchr` dogfood snapshot now completes
- real PR-diff dogfood runs on `memchr#215`, `rust-smallvec#407`,
  `rust-smallvec#64`, `rust-smallvec#254`, `arrayvec#308`, `arrayvec#138`,
  and `arrayvec#288` produce card output
- dogfood found and fixed import/declaration and `cfg(target_feature)`
  false positives
- capped repo scans stop after the requested card cap
- operation cards can inherit enclosing unsafe function `# Safety` docs
- owner inference ignores comments while scanning backward
- one fixture-backed `Vec::set_len` initialization-evidence improvement changed
  two `arrayvec#288` cards from `guard_missing` to `guarded_unwitnessed`
- one fixture-backed `set_len(0)` clear-evidence improvement changed another
  `arrayvec#288` card from `guard_missing` to `guarded_unwitnessed`
- one fixture-backed non-zero shrink-evidence improvement changed two more
  `arrayvec#288` cards from `guard_missing` to `guarded_unwitnessed` and one
  card from `guard_missing` to `unsafe_unreached`
- one fixture-backed last-index shrink improvement changed the `rust-smallvec#64`
  `set_len(last_index)` card from missing local guard evidence to fully
  discharged guard evidence while preserving the contract/witness prompt
- attributed unsafe function declarations are deduped between syntax-backed
  extraction and fallback line scanning
- false-positive regression coverage exists in fixtures and calibration
- dogfood output remains advisory static review evidence

The repo must not claim:

- calibrated false-positive or false-negative rates
- usable-alpha support-tier promotion
- full-repository coverage from top-50 capped snapshots
- uncapped repo-scan performance
- general PR-diff usefulness from four PRs
- memory-safety proof
- UB-free status
- witness execution
- blocking policy readiness

## Known limits

- Only three real crates completed in this slice.
- The successful dogfood snapshots were capped at 50 cards.
- Only seven real PR diffs were measured.
- `memchr` completion depends on capped-scan behavior; uncapped performance is
  still unmeasured.
- No human audit was performed for every emitted card.
- `arrayvec#288` shows that `Vec::set_len` guard evidence still needs better
  modeling; visible `MaybeUninit::new` initialization loops and const `CAP`
  capacity evidence now have fixture and dogfood-rerun coverage, and
  non-zero shrink and `set_len(0)` clear evidence have fixture and dogfood-rerun
  coverage, and last-index shrink evidence has fixture and `rust-smallvec#64`
  coverage, while other `set_len` and broader initialization patterns remain
  limited.
- `arrayvec#138` shows the raw-pointer UTF-8 write path still needs better
  contract evidence, especially around unsafe helper APIs and tests with unsafe
  blocks.
- These runs do not prove absence of missed unsafe seams.

## Next useful work

Continue dogfood before promotion:

- run uncapped or sampled repo inventories on additional unsafe-heavy crates
- measure card usefulness on real PR diffs, not only whole-repo snapshots
- record repeated false-positive categories as fixture regressions
- continue improving `Vec::set_len` obligation/evidence modeling before
  support-tier promotion
- keep support tiers experimental until dogfood evidence justifies promotion

Defer:

- default blocking policy
- calibrated badge claims
- release-grade adoption claims
- automatic witness execution
