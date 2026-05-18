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
- A receipt with unknown strength is rejected.
- A receipt with uncounted card identity is rejected.
- A receipt missing author, timestamp, or expiry metadata is rejected.
- A receipt whose expiry predates its recorded date is rejected.
- If receipt scope is limited, the receipt summary keeps that limitation visible.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
cargo test -p unsafe-review-core receipt
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
