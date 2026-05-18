# Witness receipt import v0.5 receipt

Date: 2026-05-18
Status: exact-card witness receipt import and fixture proof landed
Owner: CLI/core/policy

## What landed

The first witness receipt slice imports user-provided JSON receipts from:

```text
.unsafe-review/receipts/*.json
```

Merged PRs:

- `#138 receipts: import exact card witness receipts`
- `#140 receipts: validate witness receipt metadata`
- `#141 test(fixtures): add receipted review card golden`
- `#146 test(cli): cover receipted fixture output`

The receipt importer:

- parses JSON receipt files from the workspace root
- requires exact counted `ReviewCard` identity in `card_id`
- accepts explicit receipt strengths: `configured`, `ran`, `test_targeted`, and
  `site_reached`
- rejects unknown receipt strengths
- rejects uncounted card identities
- requires non-empty `author`
- requires `recorded_at` in `YYYY-MM-DDTHH:MM:SSZ` UTC timestamp form
- requires `expires_at` in `YYYY-MM-DD` date form
- rejects receipts whose expiry predates their recorded date
- marks top-level witness evidence present for exact matches
- marks obligation-level witness evidence present for exact matches
- removes the `witness` missing-evidence item for exact matches
- has a committed `raw_pointer_alignment_receipted` fixture/golden proving rendered
  card output
- has CLI e2e coverage proving `check --format json` imports the receipt,
  removes missing witness evidence, and keeps the guard gap visible

Receipt import does not create analyzer truth. It attaches external witness
evidence to an existing `ReviewCard`.

## Proof

The merged PR passed the hosted Rust workspace, advisory workflow, CodeRabbit,
and GitGuardian checks before merge.

Targeted local validation added during this slice included:

```bash
rtk cargo test -p unsafe-review-core receipt --locked
rtk cargo test -p unsafe-review-core imported_receipt --locked
rtk cargo test -p unsafe-review-core fixture_card_goldens_match_rendered_json --locked
rtk cargo test -p unsafe-review --test e2e check_json_imports_witness_receipts_without_hiding_guard_gaps --locked
rtk cargo run --locked -p xtask -- check-fixtures
```

The recurring workspace gate also passed:

```bash
rtk cargo fmt --check
rtk cargo check --workspace --all-targets --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo run --locked -p xtask -- check-pr
rtk git diff --check
```

## Current support posture

Witness receipt import is experimental.

The repo may claim:

- receipts are imported from `.unsafe-review/receipts/*.json`
- receipts match exact counted `ReviewCard` identities only
- matching receipts mark witness evidence present in card JSON
- matching receipts remove missing witness evidence
- receipt strength remains explicit in imported evidence summaries
- receipt author, recorded timestamp, expiry, command, and limitations remain
  visible in imported evidence summaries
- the `raw_pointer_alignment_receipted` golden proves a receipt does not hide
  the still-missing alignment guard
- CLI JSON output preserves the same behavior end to end

The repo must not claim:

- memory-safety proof
- UB-free status
- that `unsafe-review` ran Miri, cargo-careful, sanitizers, Loom, Shuttle, Kani,
  or Crux
- site execution without a `site_reached` receipt
- repository-wide witness coverage from a focused receipt
- default blocking or branch-protection readiness

## Known limits

- Receipt matching is exact `card_id` only.
- Receipt import does not validate that the recorded command actually ran.
- Receipt import does not parse native Miri, sanitizer, Loom, Kani, or Crux
  output.
- Receipt import does not discharge contract, guard, or reach evidence.
- Duplicate receipts for the same card are rejected instead of merged.
- Receipt import validates metadata shape, but it does not verify author identity
  or clock freshness against the current date.

## Next useful work

Prefer dogfood and native adapter proof before adding automation:

- import receipts for real unsafe-review dogfood PRs and inspect card wording
- add native Miri or cargo-careful receipt adapters only after the JSON shape
  holds up
- keep witness execution separate from receipt import

Defer:

- automatic witness execution
- witness-backed blocking policy
- broad or fuzzy receipt matching
- native tool-output parsing without fixture proof
- repository safety badges based on receipts
