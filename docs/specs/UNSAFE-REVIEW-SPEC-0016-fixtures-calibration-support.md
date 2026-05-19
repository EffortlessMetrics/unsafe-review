# UNSAFE-REVIEW-SPEC-0016: Fixtures, calibration, support tiers

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for fixtures, calibration, support tiers.

## Behavior

Every promoted claim must map to fixture/golden/receipt proof and a support-tier entry.

The calibration manifest lives at:

```text
fixtures/calibration.toml
```

Each case must name:

- fixture directory
- calibration kind: `positive`, `negative`, or `false_positive_control`
- human-readable claim
- related support-tier surface
- expected card count
- expected class for non-empty card cases
- optional expected operation family
- optional expected hazard

`cargo xtask check-calibration` validates the manifest against committed fixture
goldens. It is a proof-index check, not a claim that the fixture corpus is
complete or representative of real-world unsafe Rust.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- machine-checked calibration manifest for the core fixture claims
- policy documentation when behavior is configurable

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- Calibration manifest entries fail if expected card counts, classes, operation
  families, or hazards drift away from committed fixture goldens.

## CI proof

```bash
cargo xtask check-pr
cargo xtask check-calibration
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
