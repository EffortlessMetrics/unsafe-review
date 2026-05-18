# UNSAFE-REVIEW-SPEC-0010: Policy, baseline, suppressions

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for policy, baseline, suppressions.

## Behavior

Use exact counted identity for baselines and suppressions. Default advisory, then no-new-debt, then calibrated blocking.

Policy evaluation classifies each review card as new, baseline-known,
suppressed, or policy-blocked without changing the underlying card content.
Baseline and suppression files are ledgers over stable card identity, not fuzzy
ignore lists.

## Policy modes

- `advisory`: always exits successfully unless the command itself fails; cards are
  reported with policy classification.
- `no-new-debt`: exits non-zero when an unsuppressed card is not present in the
  accepted baseline.
- `calibrated-blocking`: exits non-zero only for supported hazard/obligation
  combinations whose fixtures and support-tier entries permit blocking.

The default mode is `advisory`. Any stronger mode must be selected by policy file
or CLI flag and must be visible in JSON and human output.

## Baseline rules

- A baseline entry matches by stable card ID plus operation kind, hazard kind,
  obligation set, and repository-relative path.
- Line numbers may be displayed as context, but they are not sufficient for a
  match.
- A baseline entry can include expiry, owner, reason, and last-reviewed metadata.
- Baseline entries that no longer match current cards are reported as stale.
- A card that changes hazard, obligation, or operation identity becomes new debt
  even if it appears near the same source lines.

## Suppression rules

- A suppression must name a stable card ID or a narrowly scoped policy selector.
- Every suppression requires a reason, owner, creation date, and expiry or review
  cadence.
- Suppressions are rendered on the card as policy metadata; they do not remove the
  card from JSON artifacts.
- Broad path-only suppressions are invalid unless an allowlist policy explicitly
  enables them for generated code.
- Expired suppressions are ignored for exit-code decisions and reported as policy
  findings.

## Exit-code requirements

- Analyzer/runtime errors use a distinct exit-code path from policy failures.
- `advisory` mode returns success when cards are found.
- `no-new-debt` returns failure only for unsuppressed cards absent from baseline.
- `calibrated-blocking` returns failure only for cards eligible under support-tier
  rules.
- JSON output includes enough policy state for CI to explain the exit code without
  scraping human text.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no regex-only blanket ignores for unsafe operations
- no deletion of cards from durable artifacts due to policy state

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- policy fixtures covering new debt, baseline-known debt, active suppression,
  expired suppression, and stale baseline entries

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- In advisory mode, an unsuppressed new card is reported but does not fail the
  process.
- In no-new-debt mode, the same card fails unless a matching baseline or active
  suppression exists.
- Moving code without changing card identity keeps the baseline match; changing
  hazard or obligation identity does not.

## Implementation backlog

1. Parse `policy/unsafe-review.toml` and referenced baseline/suppression ledgers.
2. Add policy classification fields to review-card output DTOs.
3. Implement stale baseline and expired suppression reporting.
4. Wire policy mode to process exit codes in the CLI.
5. Add policy fixture tests and golden output.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
