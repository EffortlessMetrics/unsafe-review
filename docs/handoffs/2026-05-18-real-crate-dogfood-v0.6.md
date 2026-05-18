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
| `rust-lang/hashbrown` | `7b3bba6eb4b2f03636155c918552b5f30c1a05b3` | PR-diff dogfood completed for `hashbrown#692` |

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

A real PR-diff dogfood pass on `rust-lang/hashbrown#692` exposed and fixed a
core operation classification gap:

- `slice::from_raw_parts_mut(...)` now uses the existing
  `slice_from_raw_parts` operation family instead of generic `unsafe_fn_call`
- raw pointer `write_bytes` calls now use the existing `raw_pointer_write`
  operation family instead of generic `unsafe_fn_call`
- `index < self.num_ctrl_bytes()` debug assertions now count as visible bounds
  guard evidence for pointer arithmetic cards
- private unsafe function declarations with explicit `# Safety` docs are treated
  as caller-contract sites rather than local guard sites
- `&'static mut ...` lifetime/type text is not classified as a `static mut`
  item

Regression proof was added with a fixture golden for
`slice_from_raw_parts_mut`, a fixture golden for `raw_pointer_write_bytes`, and
fixture coverage for `pointer_arithmetic_num_ctrl_bytes_guard`, plus scanner
tests for `&'static mut` versus real `static mut` items. A
`documented_private_unsafe_fn` fixture pins the private `# Safety` declaration
case without changing the older `SAFETY:`-comment helper behavior.

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

### `rust-smallvec#277`

PR: `https://github.com/servo/rust-smallvec/pull/277`

The PR fixes Miri `-Ztag-raw-pointers` issues by changing raw-pointer aliasing
patterns in `Drain` and `insert_many`.

Before start-bound shrink evidence:

```text
changed_rust_files: 1
cards: 9
contract_missing: 9
operation families: pointer_arithmetic, vec_set_len, slice_from_raw_parts, nonnull_unchecked
drain set_len discharge: missing local guard evidence
```

After start-bound shrink evidence:

```text
changed_rust_files: 1
cards: 9
contract_missing: 9
operation families: pointer_arithmetic, vec_set_len, slice_from_raw_parts, nonnull_unchecked
drain set_len discharge: all inferred obligations have visible local guard evidence
```

The card class did not change because the changed unsafe block still lacks local
`SAFETY:` contract text. The useful improvement is narrower: the `Vec::set_len`
card for `self.set_len(start)` now recognizes the local `start <= end <= len`
guard chain as shrink evidence, so the remaining prompt is contract/witness
review instead of missing initialized-range guard evidence.

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

### `arrayvec#187`

PR: `https://github.com/bluss/arrayvec/pull/187`

The PR adds `ArrayVec::take` and a public unsafe
`into_inner_unchecked` helper that reads the initialized backing array with
`ptr::read`.

Dogfood output:

```text
changed_rust_files: 2
cards: 3
contract_missing: 3
operation families: unknown, raw_pointer_read
owners: into_inner, into_inner_unchecked
```

This run did not require a scanner patch. It is useful because it shows a
legitimate contract-quality prompt: the new public unsafe API uses `Safety:`
prose rather than a `# Safety` section, so the public unsafe function and the
raw `ptr::read` remain `contract_missing` prompts. The safe `into_inner` wrapper
has a related test mention, while `into_inner_unchecked` itself is still
statically unreached by name. The output remains advisory and does not claim the
change is wrong.

### `arrayvec#174`

PR: `https://github.com/bluss/arrayvec/pull/174`

The PR rewrites `ArrayVec::retain` to mirror `Vec::retain`, with temporary
length clearing, pointer moves, drop-on-panic cleanup, inline unsafe references,
and `copy_nonoverlapping` backshifts.

Before inline unsafe-operation dedupe:

```text
changed_rust_files: 1
cards: 11
contract_missing: 11
operation families: vec_set_len, pointer_arithmetic, unknown, raw_pointer_deref, copy_nonoverlapping
duplicate: inline `unsafe { &mut *cur }` emitted both unknown unsafe-block and raw_pointer_deref cards
```

After inline unsafe-operation dedupe:

```text
changed_rust_files: 1
cards: 10
contract_missing: 10
operation families: vec_set_len, pointer_arithmetic, unknown, raw_pointer_deref, copy_nonoverlapping
```

After drop/deallocation operation modeling:

```text
changed_rust_files: 1
cards: 10
contract_missing: 10
operation families: vec_set_len, pointer_arithmetic, drop_in_place, raw_pointer_deref, copy_nonoverlapping
unknown cards: 0
```

This run first exposed and fixed a scanner duplicate: an inline unsafe block
that contains a concrete raw pointer dereference should not also emit a generic
unknown unsafe-block wrapper card on the same line. A follow-up fixture-backed
operation slice then modeled `ptr::drop_in_place` as `drop_in_place`, replacing
the remaining `unknown` card with drop/deallocation hazards while preserving the
contract and witness prompts.

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

Follow-up rerun after treating documented public unsafe API declarations as
contract-only sites where local guard evidence is not expected:

```text
changed_rust_files: 1
cards: 9
contract_missing: 0
guard_missing: 2
guarded_unwitnessed: 5
unsafe_unreached: 2
operation families: vec_set_len, unknown
```

The improved card is still advisory only:

```text
set_len  line 452  unsafe_fn  unsafe_unreached
```

It moved out of `guard_missing` because the public unsafe API now has a
recognized `# Safety` contract, and the declaration itself should not require a
local guard. The card remains actionable as `unsafe_unreached` because static
reach did not find a related test mention, and no witness receipt is attached.
The remaining two `guard_missing` cards are the `encode_utf8` unsafe block and
the matching `set_len(len + n)` write pattern.

Follow-up rerun after adding fixture-backed call-result initialization evidence
for `set_len(len + n)`:

```text
changed_rust_files: 1
cards: 9
contract_missing: 0
guard_missing: 1
guarded_unwitnessed: 6
unsafe_unreached: 2
operation families: vec_set_len, unknown
```

The additional improved card is still advisory only:

```text
try_push  line 249  vec_set_len  guarded_unwitnessed  self.set_len(len + n);
```

It moved out of `guard_missing` because the local code records the number of
bytes returned from `encode_utf8` and extends the length by that value. The
remaining `guard_missing` card is the `encode_utf8` unsafe-call wrapper, which
still needs a future unsafe-call operation family or more specific contract
modeling.

Follow-up rerun after adding fixture-backed unsafe-call wrapper detection:

```text
changed_rust_files: 1
cards: 9
contract_missing: 0
guard_missing: 1
guarded_unwitnessed: 6
unsafe_unreached: 2
operation families: vec_set_len, unsafe_fn_call, unknown
```

The remaining `guard_missing` card is still advisory only:

```text
try_push  line 244  unsafe_fn_call  guard_missing
```

It moved from `unknown` to `unsafe_fn_call`, with the callee identity captured as
`encode_utf8`. The card still asks for discharge/witness evidence because the
tool does not infer the callee's full safety contract from the function name or
nearby prose.

### `hashbrown#692`

PR: `https://github.com/rust-lang/hashbrown/pull/692`

The PR fixes potential UB in `RawTableInner::fallible_with_capacity` and touches
raw table control-byte allocation paths.

Initial dogfood output:

```text
changed_rust_files: 2
cards: 4
contract_missing: 0
guard_missing: 4
operation families: unsafe_fn_call, unknown, pointer_arithmetic
unsafe_fn_call cards: 2
```

This run exposed a classification gap: the changed
`slice::from_raw_parts_mut(...)` call was labeled as a generic
`unsafe_fn_call`, which hid the more specific slice range obligations already
modeled for `slice::from_raw_parts`. Building a fixture for the case also
exposed a false-positive boundary: `&'static mut [u8]` type/lifetime text must
not be treated as a `static mut` item.

Follow-up rerun after adding fixture-backed mutable slice detection and the
static-lifetime false-positive guard:

```text
changed_rust_files: 2
cards: 4
contract_missing: 0
guard_missing: 4
operation families: unsafe_fn_call, unknown, pointer_arithmetic, slice_from_raw_parts
unsafe_fn_call cards: 1
slice_from_raw_parts cards: 1
```

The improved card is still advisory only:

```text
ctrl_slice  line 2648  slice_from_raw_parts  guard_missing
```

It now carries the pointer validity, alignment, initialized-memory, bounds, and
same-allocation obligations for the mutable slice construction. No witness was
executed, and the remaining `unknown` card on the documented unsafe helper
declaration remains a separate modeling limit.

Follow-up rerun after adding fixture-backed `write_bytes` detection:

```text
changed_rust_files: 2
cards: 4
contract_missing: 0
guard_missing: 4
operation families: raw_pointer_write, unknown, pointer_arithmetic, slice_from_raw_parts
unsafe_fn_call cards: 0
raw_pointer_write cards: 1
```

The additional improved card is still advisory only:

```text
fill_tag  line 80  raw_pointer_write  guard_missing
```

It moved from generic `unsafe_fn_call` to `raw_pointer_write`, so the card now
uses the pointer validity, alignment, initialized-memory, and allocation
obligation vocabulary already used by raw pointer writes. No witness was
executed.

Follow-up rerun after adding fixture-backed `num_ctrl_bytes` bounds evidence for
pointer arithmetic:

```text
changed_rust_files: 2
cards: 4
contract_missing: 0
guard_missing: 3
guarded_unwitnessed: 1
operation families: raw_pointer_write, unknown, pointer_arithmetic, slice_from_raw_parts
```

The improved card is still advisory only:

```text
ctrl  line 2642  pointer_arithmetic  guarded_unwitnessed
```

It moved from `guard_missing` to `guarded_unwitnessed` because the local context
contains `debug_assert!(index < self.num_ctrl_bytes())`. No witness was
executed, and other pointer-arithmetic bound naming patterns remain uncalibrated.

Follow-up rerun after treating documented private unsafe declarations as
caller-contract sites:

```text
changed_rust_files: 2
cards: 4
contract_missing: 0
guard_missing: 2
guarded_unwitnessed: 2
operation families: raw_pointer_write, unknown, pointer_arithmetic, slice_from_raw_parts
```

The improved declaration card is still advisory only:

```text
ctrl  line 2639  unsafe_fn/unknown  guarded_unwitnessed
```

It moved from `guard_missing` to `guarded_unwitnessed` because the private
unsafe declaration has explicit `# Safety` documentation and a related static
test mention. This does not infer the safety contract of unsafe call sites and
does not execute a witness.

## Proof

Targeted local validation:

```bash
rtk cargo fmt --check
rtk cargo test -p unsafe-review-core scanner --locked
rtk cargo test -p unsafe-review-core workspace --locked
rtk cargo test -p unsafe-review-core capped_repo_scan --locked
rtk cargo test -p unsafe-review-core owner_safety --locked
rtk cargo test -p unsafe-review-core slice_from_raw_parts_mut_uses_slice_operation_family --locked
rtk cargo test -p unsafe-review-core scan_file_does_not_classify_static_lifetime_mut_reference_as_static_mut --locked
rtk cargo test -p unsafe-review-core scan_file_classifies_static_mut_items --locked
rtk cargo test -p unsafe-review-core text_detection_classifies_raw_pointer_write_bytes_as_write --locked
rtk cargo test -p unsafe-review-core raw_pointer_v1_operation_cards_are_concrete --locked
rtk cargo test -p unsafe-review-core pointer_arithmetic_num_ctrl_bytes_guard_is_discharged --locked
rtk cargo test -p unsafe-review-core documented_private_unsafe_fn_does_not_require_local_guard --locked
rtk cargo test -p unsafe-review-core private_unsafe_helper_can_use_local_safety_comment --locked
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
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/smallvec --diff target/dogfood-work/smallvec-pr277.raw.diff --format json --max-cards 30 --out target/dogfood-work/smallvec-pr277.after-start-bound-shrink.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/smallvec --diff target/dogfood-work/smallvec-pr64.raw.diff --format json --max-cards 20 --out target/dogfood-work/smallvec-pr64.after-last-index-shrink.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr308.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr308.unsafe-review.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr138.raw.diff --format json --max-cards 30 --out target/dogfood-work/arrayvec-pr138.after-attributed-dedupe.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr187.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr187.unsafe-review.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr174.raw.diff --format json --max-cards 30 --out target/dogfood-work/arrayvec-pr174.after-inline-dedupe.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr174.raw.diff --format json --max-cards 30 --out target/dogfood-work/arrayvec-pr174.after-drop-in-place.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.unsafe-review.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-setlen-init.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-setlen-zero.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-setlen-shrink.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-public-unsafe-contract.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-call-result-init.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/arrayvec --diff target/dogfood-work/arrayvec-pr288.raw.diff --format json --max-cards 20 --out target/dogfood-work/arrayvec-pr288.after-unsafe-fn-call.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/hashbrown --diff target/dogfood-work/hashbrown-pr692.raw.diff --format json --max-cards 30 --out target/dogfood-work/hashbrown-pr692.after-slice-mut.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/hashbrown --diff target/dogfood-work/hashbrown-pr692.raw.diff --format json --max-cards 30 --out target/dogfood-work/hashbrown-pr692.after-write-bytes.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/hashbrown --diff target/dogfood-work/hashbrown-pr692.raw.diff --format json --max-cards 30 --out target/dogfood-work/hashbrown-pr692.after-num-ctrl-guard.json
rtk cargo run --locked -p unsafe-review -- check --root target/dogfood-work/hashbrown --diff target/dogfood-work/hashbrown-pr692.raw.diff --format json --max-cards 30 --out target/dogfood-work/hashbrown-pr692.after-private-contract.json
```

The dogfood reruns used a temporary `CARGO_TARGET_DIR` to avoid a Windows file
lock on the default debug binary.

## Current support posture

Real-crate dogfood is experimental.

The repo may claim:

- the first real-crate dogfood slice includes capped repo snapshots on
  `rust-smallvec`, `arrayvec`, and `memchr`, plus PR-diff dogfood on
  `hashbrown`
- a capped `memchr` dogfood snapshot now completes
- real PR-diff dogfood runs on `memchr#215`, `rust-smallvec#407`,
  `rust-smallvec#277`, `rust-smallvec#64`, `rust-smallvec#254`,
  `arrayvec#308`, `arrayvec#138`, `arrayvec#187`, `arrayvec#174`, and
  `arrayvec#288`, and `hashbrown#692` produce card output
- dogfood found and fixed import/declaration and `cfg(target_feature)`
  false positives
- `&'static mut` type/lifetime text is not classified as a `static mut` item
- capped repo scans stop after the requested card cap
- operation cards can inherit enclosing unsafe function `# Safety` docs
- owner inference ignores comments while scanning backward
- owner inference prefers real function declarations over `impl Trait`
  parameter text
- inline unsafe blocks with concrete same-line raw pointer operations are
  deduped instead of emitting generic unknown wrapper cards
- `ptr::drop_in_place` is modeled as a fixture-backed drop/deallocation
  operation family in `arrayvec#174`
- one fixture-backed `Vec::set_len` initialization-evidence improvement changed
  two `arrayvec#288` cards from `guard_missing` to `guarded_unwitnessed`
- one fixture-backed `set_len(0)` clear-evidence improvement changed another
  `arrayvec#288` card from `guard_missing` to `guarded_unwitnessed`
- one fixture-backed non-zero shrink-evidence improvement changed two more
  `arrayvec#288` cards from `guard_missing` to `guarded_unwitnessed` and one
  card from `guard_missing` to `unsafe_unreached`
- one fixture-backed start-bound shrink improvement changed the
  `rust-smallvec#277` `self.set_len(start)` card from missing local guard
  evidence to fully discharged guard evidence while preserving the
  contract/witness prompt
- one fixture-backed last-index shrink improvement changed the `rust-smallvec#64`
  `set_len(last_index)` card from missing local guard evidence to fully
  discharged guard evidence while preserving the contract/witness prompt
- one fixture-backed public unsafe API evidence improvement changed the
  `arrayvec#288` documented `set_len` declaration from a missing local guard
  prompt to an `unsafe_unreached` contract/reach/witness prompt
- one fixture-backed call-result initialization improvement changed the
  `arrayvec#288` `set_len(len + n)` card from `guard_missing` to
  `guarded_unwitnessed`
- one fixture-backed unsafe-call wrapper improvement changed the `arrayvec#288`
  `encode_utf8` unsafe block from `unknown` to `unsafe_fn_call` while preserving
  the missing-discharge prompt
- one fixture-backed mutable slice improvement changed the `hashbrown#692`
  `slice::from_raw_parts_mut` card from generic `unsafe_fn_call` to
  `slice_from_raw_parts`
- one fixture-backed raw pointer write improvement changed the `hashbrown#692`
  `write_bytes` card from generic `unsafe_fn_call` to `raw_pointer_write`
- one fixture-backed pointer arithmetic improvement changed the `hashbrown#692`
  `ctrl` pointer arithmetic card from `guard_missing` to `guarded_unwitnessed`
  when `index < self.num_ctrl_bytes()` is visible
- one fixture-backed contract improvement changed the `hashbrown#692` private
  documented `unsafe fn ctrl` declaration from `guard_missing` to
  `guarded_unwitnessed`
- attributed unsafe function declarations are deduped between syntax-backed
  extraction and fallback line scanning
- false-positive regression coverage exists in fixtures and calibration
- dogfood output remains advisory static review evidence

The repo must not claim:

- calibrated false-positive or false-negative rates
- usable-alpha support-tier promotion
- full-repository coverage from top-50 capped snapshots
- uncapped repo-scan performance
- general PR-diff usefulness from eleven PRs
- memory-safety proof
- UB-free status
- witness execution
- blocking policy readiness

## Known limits

- Only three real crates completed capped repo snapshots in this slice; four
  crates have at least one snapshot or PR-diff receipt.
- The successful dogfood snapshots were capped at 50 cards.
- Only eleven real PR diffs were measured.
- `memchr` completion depends on capped-scan behavior; uncapped performance is
  still unmeasured.
- No human audit was performed for every emitted card.
- `arrayvec#288` shows that `Vec::set_len` guard evidence still needs better
  modeling; visible `MaybeUninit::new` initialization loops and const `CAP`
  capacity evidence now have fixture and dogfood-rerun coverage, and
  non-zero shrink and `set_len(0)` clear evidence have fixture and dogfood-rerun
  coverage, start-bound shrink evidence has fixture and `rust-smallvec#277`
  coverage, and last-index shrink evidence has fixture and `rust-smallvec#64`
  coverage, while other `set_len` and broader initialization patterns remain
  limited.
- `arrayvec#138` shows the raw-pointer UTF-8 write path still needs better
  contract evidence, especially around unsafe helper APIs and tests with unsafe
  blocks.
- `arrayvec#187` shows public unsafe helper APIs with `Safety:` prose remain
  contract prompts until they expose a recognized `# Safety` section or nearby
  `SAFETY:` evidence.
- Public unsafe API declarations with recognized `# Safety` docs no longer ask
  for local declaration guards, but static reach remains a heuristic name search.
- `arrayvec#288` now labels the `encode_utf8` wrapper as `unsafe_fn_call`, but
  callee-specific safety contract inference remains future work.
- `arrayvec#174` now has a fixture-backed `ptr::drop_in_place` card, but broader
  drop/deallocation modeling beyond that operation remains narrow.
- `hashbrown#692` now has a fixture-backed `slice::from_raw_parts_mut` card, but
  broader slice range proof remains source-level and advisory.
- `hashbrown#692` now has a fixture-backed `write_bytes` card, but broader
  byte-pattern validity and destination-type modeling remains source-level and
  advisory.
- `hashbrown#692` now recognizes `num_ctrl_bytes` bounds evidence for pointer
  arithmetic, but broader pointer-arithmetic guard naming remains uncalibrated.
- `hashbrown#692` now treats private unsafe declarations with explicit
  `# Safety` docs as caller-contract sites, but unsafe-call-specific callee
  contract inference remains future work.
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
