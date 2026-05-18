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
- `#148 output: add witness plan format`
- `#150 receipts: validate witness tool lanes`
- `#153 receipts: expose sdk receipt dto`
- `#155 receipts: add receipt template command`
- `#157 receipts: add receipt validate command`
- `#160 receipts: add miri saved-output adapter`
- `#162 receipts: add cargo-careful saved-output adapter`
- `#164 receipts: add sanitizer saved-output adapter`

The receipt importer:

- parses JSON receipt files from the workspace root
- requires exact counted `ReviewCard` identity in `card_id`
- accepts explicit receipt strengths: `configured`, `ran`, `test_targeted`, and
  `site_reached`
- rejects unknown witness tool lanes
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
- adds `--format witness-plan` as a card-sourced route artifact that lists
  recommended witnesses, commands, imported receipt evidence, missing evidence,
  and the trust boundary
- exposes the receipt JSON shape as the serde-backed
  `unsafe_review_core::WitnessReceipt` DTO so SDK consumers and future native
  adapters share the same schema as the importer
- adds `unsafe-review receipt template` as a validated JSON authoring aid for
  explicit witness receipts
- adds `unsafe-review receipt validate` as a receipt-only validation command
  that reuses the importer checks without running analysis or witnesses
- adds `unsafe-review receipt import-miri` as a saved-output adapter that reads
  an existing Miri log, requires `test result: ok`, rejects failure-looking
  output, and writes a normal `tool = "miri"`, `strength = "ran"` receipt
- records saved-output adapter limitations directly in the generated receipt:
  `unsafe-review` did not run Miri, and `ran` strength does not claim site reach
- adds `unsafe-review receipt import-careful` as a saved-output adapter that
  reads an existing `cargo-careful` log, requires `test result: ok`, rejects
  failure-looking output, and writes a normal `tool = "cargo-careful"`,
  `strength = "ran"` receipt
- records matching `cargo-careful` limitations directly in the generated
  receipt: `unsafe-review` did not run `cargo-careful`, and `ran` strength does
  not claim site reach
- adds `unsafe-review receipt import-sanitizer` as a saved-output adapter that
  reads an existing sanitizer log, requires an explicit `asan`, `msan`, `tsan`,
  or `lsan` tool, requires `test result: ok`, rejects failure-looking output,
  and writes a normal sanitizer `strength = "ran"` receipt
- records matching sanitizer limitations directly in the generated receipt:
  `unsafe-review` did not run a sanitizer, and `ran` strength does not claim site
  reach

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
rtk cargo test -p unsafe-review-core witness_plan --locked
rtk cargo test -p unsafe-review-cli witness_plan --locked
rtk cargo test -p unsafe-review --test e2e check_artifact_formats_context_and_explain_work_end_to_end --locked
rtk cargo run --locked -p xtask -- check-fixtures
```

The SDK receipt DTO follow-up also passed:

```bash
rtk cargo test -p unsafe-review-core receipt --locked
rtk cargo test -p unsafe-review-core imported_receipt --locked
rtk cargo test -p unsafe-review --test e2e check_json_imports_witness_receipts_without_hiding_guard_gaps --locked
```

The receipt-template follow-up also passed:

```bash
rtk cargo test -p unsafe-review-cli receipt_template --locked
rtk cargo test -p unsafe-review --test e2e receipt_template --locked
```

The receipt-validate follow-up also passed:

```bash
rtk cargo test -p unsafe-review-cli receipt_validate --locked
rtk cargo test -p unsafe-review --test e2e receipt_validate --locked
```

The saved-output Miri adapter follow-up also passed:

```bash
rtk cargo test -p unsafe-review-core miri_receipt --locked
rtk cargo test -p unsafe-review-cli receipt_import_miri --locked
rtk cargo test -p unsafe-review --test e2e receipt_import_miri --locked
```

The saved-output `cargo-careful` adapter follow-up also passed:

```bash
rtk cargo test -p unsafe-review-core careful_receipt --locked
rtk cargo test -p unsafe-review-cli receipt_import_careful --locked
rtk cargo test -p unsafe-review --test e2e receipt_import_careful --locked
```

The saved-output sanitizer adapter follow-up also passed:

```bash
rtk cargo test -p unsafe-review-core sanitizer_receipt --locked
rtk cargo test -p unsafe-review-cli receipt_import_sanitizer --locked
rtk cargo test -p unsafe-review --test e2e receipt_import_sanitizer --locked
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
- receipt `tool` values must match known witness lanes
- the receipt JSON shape is backed by `unsafe_review_core::WitnessReceipt`
- `unsafe-review receipt template` can render a validated receipt JSON object
  from explicit user metadata
- `unsafe-review receipt validate` can count importable receipt files through
  the same validation path used by card analysis
- `unsafe-review receipt import-miri` can convert a saved Miri success log into
  a normal exact-card receipt with `tool = "miri"` and `strength = "ran"`
- the saved-output Miri adapter rejects empty, failure-looking, and
  non-success-looking logs
- generated Miri receipts keep visible limitations that `unsafe-review` did not
  run Miri and does not claim site reach
- `unsafe-review receipt import-careful` can convert a saved `cargo-careful`
  success log into a normal exact-card receipt with `tool = "cargo-careful"` and
  `strength = "ran"`
- the saved-output `cargo-careful` adapter rejects empty, failure-looking, and
  non-success-looking logs
- generated `cargo-careful` receipts keep visible limitations that
  `unsafe-review` did not run `cargo-careful` and does not claim site reach
- `unsafe-review receipt import-sanitizer` can convert a saved sanitizer success
  log into a normal exact-card receipt with explicit `asan`, `msan`, `tsan`, or
  `lsan` tool and `strength = "ran"`
- the saved-output sanitizer adapter rejects unsupported sanitizer tools, empty
  logs, failure-looking logs, and non-success-looking logs
- generated sanitizer receipts keep visible limitations that `unsafe-review`
  did not run a sanitizer and does not claim site reach
- the `raw_pointer_alignment_receipted` golden proves a receipt does not hide
  the still-missing alignment guard
- CLI JSON output preserves the same behavior end to end
- `--format witness-plan` emits a route plan from existing cards without
  executing witness tools

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
- The Miri adapter reads saved success logs only; it does not execute Miri or
  parse native UB diagnostics into cards.
- The `cargo-careful` adapter reads saved success logs only; it does not execute
  `cargo-careful` or parse diagnostics into cards.
- The sanitizer adapter reads saved success logs only; it does not execute
  sanitizers or parse diagnostics into cards.
- Receipt import does not parse Loom, Kani, or Crux output.
- Receipt import does not discharge contract, guard, or reach evidence.
- Duplicate receipts for the same card are rejected instead of merged.
- Receipt import validates metadata shape, but it does not verify author identity
  or clock freshness against the current date.
- Receipt template output is an authoring aid only; it does not run or validate
  the recorded witness command.
- Receipt validation does not analyze cards, run witnesses, or prove witness
  success.

## Next useful work

Prefer dogfood and native adapter proof before adding automation:

- import receipts for real unsafe-review dogfood PRs and inspect card wording
- dogfood the saved-output Miri, `cargo-careful`, and sanitizer adapters before
  adding more native receipt parsers
- keep witness execution separate from receipt import

Defer:

- automatic witness execution
- witness-backed blocking policy
- broad or fuzzy receipt matching
- native tool-output parsing without fixture proof
- repository safety badges based on receipts
