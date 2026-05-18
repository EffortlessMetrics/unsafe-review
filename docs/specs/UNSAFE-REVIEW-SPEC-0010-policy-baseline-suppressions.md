# UNSAFE-REVIEW-SPEC-0010: Policy, baseline, suppressions

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for policy, baseline, suppressions.

## Behavior

Use exact counted identity for baselines and suppressions. Default advisory,
then no-new-debt, then calibrated blocking.

The current implementation validates ledger shape and applies advisory
classification for exact card identity matches. Baseline and suppression
entries, when present, must include:

- `card_id`: exact counted `UR-*` review-card identity ending in `-cN`
- `owner`
- `reason`
- `evidence`
- `review_after` for baseline entries
- `review_after` or `expires` for suppression entries

Date fields use `YYYY-MM-DD`. A ledger with `status = "empty"` must not contain
entries.

When a generated card ID exactly matches a baseline entry, the card class is
`baseline_known`, priority is lowered, and the card is excluded from actionable
gap counts. When it exactly matches a suppression entry, the card class is
`suppressed`, priority is lowered, and the card is excluded from actionable gap
counts.

No-new-debt and blocking modes remain later policy work. Matching is exact and
counted; broad path, owner, or operation-family suppression is not supported.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no broad baseline or suppression patterns
- no no-new-debt or blocking behavior from ledger validation alone

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- xtask policy-ledger schema tests
- analyzer tests for exact baseline and suppression matches
- policy documentation when behavior is configurable

## Acceptance examples

- Empty baseline and suppression ledgers pass `check-policy`.
- Non-empty baseline entries require exact counted identity, owner, reason,
  evidence, and `review_after`.
- Non-empty suppression entries require exact counted identity, owner, reason,
  evidence, and either `review_after` or `expires`.
- Uncounted card identities are rejected.
- Exact baseline matches classify cards as `baseline_known`.
- Exact suppression matches classify cards as `suppressed`.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
cargo test -p xtask ledger
cargo test -p unsafe-review-core baseline_policy
cargo test -p unsafe-review-core suppression_policy
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
