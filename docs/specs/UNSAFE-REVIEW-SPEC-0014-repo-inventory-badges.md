# UNSAFE-REVIEW-SPEC-0014: Repo inventory and badges

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

Repository inventory and badges help teams track unsafe-review debt over time, but
badges can easily be misread as safety claims. The repo and badge commands still need
precise output contracts, counts, and wording rules.

## Behavior

Repo mode scans the configured repository scope and summarizes unsafe-review evidence
without requiring a diff. Badge mode projects selected inventory counts into small
machine-readable and image-friendly badge descriptors. Badges must never claim that a
repository is UB-free, memory-safe, or fully verified.

## Inventory data contract

Repo inventory JSON must include:

- repository root and analysis timestamp when available
- analyzer version and support-tier summary
- policy mode and policy file paths used
- total cards and cards by hazard class, operation kind, severity, and policy decision
- counts of missing contract, discharge, reach, and witness evidence
- baseline-known, suppressed, expired, stale, and new card counts
- receipt counts by witness kind and strength
- static limitations encountered during scan
- top files or packages by open review gaps

Inventory must be reproducible for the same inputs except for explicit timestamps or
version metadata.

## Badge contract

Badge output may include JSON descriptors, Markdown snippets, or shield-compatible
files. Supported badge labels are:

- `unsafe-review gaps`: count of new or blocking cards
- `unsafe-review baseline`: count of baseline-known cards
- `unsafe-review suppressed`: count of live suppressions
- `unsafe-review receipts`: count of attached receipts by strongest state
- `unsafe-review tier`: lowest support tier among enabled promoted claims

Badge messages must use review-evidence language. Forbidden badge messages include
`safe`, `UB-free`, `verified`, and equivalent wording.

## Counting rules

- Count cards, not unsafe lines, unless the output label explicitly says otherwise.
- Count suppressed and baseline-known cards separately from new cards.
- Expired suppressions count as open gaps in strict policy modes.
- Stale baseline or suppression entries count as policy-ledger maintenance, not open
  unsafe gaps.
- Static limitations are counted separately and must not be converted into passing
  status.

## Non-goals

- no memory-safety score
- no badge that implies absence of undefined behavior
- no default repository-wide blocking
- no historical trend storage in v1
- no duplicate truth outside review cards, policy decisions, and receipts

## Required evidence

- repo inventory fixture with new, baseline-known, suppressed, expired, and stale
  entries
- badge output tests for all supported labels
- test rejecting forbidden badge wording
- JSON golden test for inventory count stability
- documentation explaining badge trust boundaries

## Acceptance examples

- A repository with two new cards, one baseline-known card, and one live suppression
  reports each count in separate inventory fields.
- A badge for open gaps says `unsafe-review gaps: 2`, not `unsafe: safe`.
- A stale baseline entry appears in inventory maintenance counts but does not reduce
  the open gap count.
- Static limitations are visible in inventory even when no review cards are emitted.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Repo inventory and badges can move from scaffold to experimental only after count
fixtures, forbidden-wording tests, and badge descriptor tests exist.
