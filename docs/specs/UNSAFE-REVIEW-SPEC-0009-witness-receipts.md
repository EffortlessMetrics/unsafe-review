# UNSAFE-REVIEW-SPEC-0009: Witness receipts

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for witness receipts.

## Behavior

Receipts record configured, ran, test-targeted, or site-reached witness strength
and limitations. The current implementation imports JSON receipts from:

```text
.unsafe-review/receipts/*.json
```

Each receipt is matched by exact counted `card_id`. A matching receipt marks the
card's top-level witness evidence present and marks obligation-level witness
evidence present. Receipt import does not discharge contracts, guards, or reach
evidence.

The receipt shape is represented in the core SDK as the serde-backed
`WitnessReceipt` DTO. Importers and future native adapters must use that same
shape instead of inventing parallel receipt schemas.

The CLI may render a receipt template from explicit user-provided metadata. That
template output is only a JSON authoring aid; it must not run witness commands or
claim that a witness succeeded.

The CLI may import a receipt from saved Miri output. The adapter must read an
existing log file, reject empty output, reject failure-looking output, require
`test result: ok`, and emit a normal `WitnessReceipt` with `tool = "miri"` and
`strength = "ran"`. It must not execute Miri, infer site reach, or create a
card.

The CLI may import a receipt from saved `cargo-careful` output. The adapter must
read an existing log file, reject empty output, reject failure-looking output,
require `test result: ok`, and emit a normal `WitnessReceipt` with
`tool = "cargo-careful"` and `strength = "ran"`. It must not execute
`cargo-careful`, infer site reach, or create a card.

The CLI may import a receipt from saved sanitizer output. The adapter must read
an existing log file, require an explicit sanitizer `tool` of `asan`, `msan`,
`tsan`, or `lsan`, reject empty output, reject failure-looking output, require
`test result: ok`, and emit a normal `WitnessReceipt` with that sanitizer tool
and `strength = "ran"`. It must not execute a sanitizer, infer site reach, or
create a card.

The CLI may also validate receipt files without running analysis. Validation must
use the same importer checks as normal card analysis so users do not get a
separate receipt truth.

Receipt JSON fields:

```json
{
  "schema_version": "0.1",
  "card_id": "UR-...-c1",
  "tool": "miri",
  "strength": "ran",
  "author": "core/fixtures",
  "recorded_at": "2026-05-18T00:00:00Z",
  "expires_at": "2026-08-18",
  "summary": "focused witness passed",
  "command": "cargo +nightly miri test read_header",
  "limitations": ["fixture only"]
}
```

`strength` must be one of:

- `configured`
- `ran`
- `test_targeted`
- `site_reached`

`tool` must be one of the supported witness lanes:

- `miri`
- `cargo-careful`
- `asan`
- `msan`
- `tsan`
- `lsan`
- `loom`
- `shuttle`
- `kani`
- `crux`
- `human-deep-review`
- `unsupported`

`author` must be non-empty. `recorded_at` must be a UTC timestamp in
`YYYY-MM-DDTHH:MM:SSZ` form. `expires_at` must be a `YYYY-MM-DD` date on or
after the `recorded_at` date.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no witness execution by `unsafe-review`
- no receipt match without exact card identity
- no claim that a receipt proves arbitrary callers or the whole repository safe

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- analyzer tests for exact receipt import
- receipt parser tests for strength and identity validation
- policy documentation when behavior is configurable

## Acceptance examples

- A matching receipt removes the `witness` missing-evidence item.
- A matching receipt marks obligation-level witness evidence present.
- A receipt with unknown tool is rejected.
- A receipt with unknown strength is rejected.
- A receipt with uncounted card identity is rejected.
- A receipt missing author, timestamp, or expiry metadata is rejected.
- A receipt whose expiry predates its recorded date is rejected.
- If receipt scope is limited, the receipt summary keeps that limitation visible.
- The core `WitnessReceipt` DTO round-trips through serde JSON and validates the
  same required fields as the importer.
- The CLI receipt-template command writes a valid receipt JSON object but does
  not execute the recorded command.
- The CLI Miri saved-output adapter writes a receipt from a success-looking Miri
  log and rejects failure-looking output.
- The CLI `cargo-careful` saved-output adapter writes a receipt from a
  success-looking `cargo-careful` log and rejects failure-looking output.
- The CLI sanitizer saved-output adapter writes a receipt from a success-looking
  sanitizer log, rejects unsupported sanitizer tools, and rejects
  failure-looking output.
- The CLI receipt-validate command counts importable receipts and rejects the
  same invalid receipt files as normal analysis.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
cargo test -p unsafe-review-core receipt
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
