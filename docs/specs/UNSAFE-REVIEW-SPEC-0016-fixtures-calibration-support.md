# UNSAFE-REVIEW-SPEC-0016: Fixtures, calibration, support tiers

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for fixtures, calibration, support tiers.

The repository has early fixtures and expected card output. It still needs a
calibration harness that turns those fixtures into promotion gates and keeps the
support-tier document aligned with implemented behavior.

## Implementation status

Partially implemented. Fixture directories and expected JSON files exist. A full
golden-test harness, false-positive ledger, and tier-promotion workflow are still
planned.

## Behavior

Every promoted claim must map to fixture/golden/receipt proof and a support-tier entry.

### Fixture layout

Each fixture must include:

- `Cargo.toml` and source files required to compile or parse the example;
- `change.diff` for diff-scope analysis;
- `expected.cards.json` for canonical card output;
- optional `expected.summary.md` for renderer snapshots;
- optional `receipts/` directory for witness receipt import examples;
- optional `README.md` explaining the unsafe pattern and expected limitation.

Fixtures must be small, deterministic, and free of network requirements.

### Calibration classes

Fixtures must be classified as one of:

- `positive`: a card should be emitted;
- `negative`: no card should be emitted;
- `limitation`: a card should name a static-analysis limit;
- `regression`: a previously observed false positive or false negative;
- `receipt`: receipt import or matching behavior;
- `policy`: baseline, suppression, exit-code, or drift behavior.

### Support-tier promotion

A capability may move from `planned` to `scaffold` only when a compile or smoke
proof exists. It may move from `scaffold` to `experimental` only when at least one
positive and one negative fixture cover it. It may move from `experimental` to
`usable alpha` only when:

- fixture coverage includes common happy paths and known limits;
- JSON and human/Markdown projections agree;
- policy behavior is documented when relevant;
- dogfood output has been reviewed;
- known false positives and false negatives are recorded.

### False-positive and false-negative tracking

Calibration must maintain a ledger of analyzer misses and noisy cards. Each entry
must include a fixture or reproduction path, affected capability, expected
behavior, actual behavior, severity, and disposition.

### Drift handling

Golden updates must be deliberate. The update workflow must show semantic diffs
of cards, not only byte-level JSON changes. Card id churn is a regression unless
caused by an intentional schema or identity change.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no benchmark claims without a benchmark spec
- no promotion based only on manual inspection

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- golden-test harness for every fixture's `expected.cards.json`
- support-tier consistency check that fails when a promoted claim lacks proof
- false-positive/false-negative ledger tests or validation

## Acceptance examples

- Adding a new hazard detector requires at least one positive fixture and one
  negative fixture before the support tier can be promoted.
- A changed unsafe seam produces one review card with stable identity in its
  golden output.
- A safe-code fixture remains card-free.
- A golden update that changes card ids is surfaced as identity drift.
- If evidence is not knowable statically, the expected fixture output names the
  limitation instead of overclaiming.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
