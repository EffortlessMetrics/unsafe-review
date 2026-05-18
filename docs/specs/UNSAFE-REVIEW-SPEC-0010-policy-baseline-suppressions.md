# UNSAFE-REVIEW-SPEC-0010: Policy, baseline, suppressions

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a deterministic policy layer before it can support no-new-debt
CI, accepted baseline debt, or local suppressions. The implementation must make every
policy decision explainable and must avoid accidental blocking while the analyzer is
still calibrating.

## Behavior

Policy is advisory by default. A repository may opt into stricter modes in this order:

1. `advisory`: emit cards, policy decisions, and exit successfully unless internal
   execution fails.
2. `no_new_debt`: fail only when a card is not matched by the accepted baseline or a
   valid suppression.
3. `calibrated_blocking`: fail for configured severities, hazard classes, or missing
   obligations after support-tier promotion.

Baselines and suppressions use exact counted identity. The matcher must be stable
across output formats and must explain whether a card is new, baseline-known,
suppressed, expired, stale, or policy-blocking.

## Policy file roles

- `policy/unsafe-review.toml`: global mode, output settings, receipt paths, staleness
  windows, and enabled classifiers.
- `policy/unsafe-review-baseline.toml`: accepted existing cards with card identity,
  reason, owner, accepted date, and optional expiry.
- `policy/unsafe-review-suppressions.toml`: narrow suppressions for false positives or
  intentionally reviewed risk with reason, owner, and expiry.
- Companion policy ledgers may constrain generated files, non-Rust files, executable
  files, workflows, processes, networking, clippy exceptions, and panic allowances.

## Card identity matching

A baseline or suppression entry may match only when all required identity fields match:

- stable card id or legacy id alias
- repository-relative path
- operation kind
- hazard class
- line anchor or snippet hash according to policy
- occurrence count when multiple equivalent seams appear in one file

If a file edit moves a card without changing the seam, the matcher may classify the
entry as relocated only when the snippet hash and operation identity still match. If
matching is ambiguous, the card is new.

## Decision model

Each card must receive one policy decision:

- `new`: not matched by baseline or suppression
- `baseline_known`: matched by a live baseline entry
- `suppressed`: matched by a live suppression entry
- `expired`: matched only by expired entries
- `stale`: matched by entries that no longer correspond to emitted cards
- `blocking`: fails the selected policy mode

JSON output must expose the decision, matched entry id, reason, owner, expiry, and any
ambiguity. Human and Markdown output must summarize counts by decision.

## Non-goals

- no silent broad suppressions
- no wildcard suppression that hides all unsafe code in a repository
- no default blocking in the absence of explicit policy mode
- no claim that baseline-known debt is safe
- no duplicate truth outside this spec and linked policy files

## Required evidence

- fixture-backed baseline match, suppression match, expired entry, stale entry, and
  no-new-debt failure cases
- JSON golden tests for policy decisions and matched entry metadata
- human output smoke coverage for policy count summaries
- policy documentation for every supported policy key
- tests proving advisory mode does not fail on review cards

## Acceptance examples

- In advisory mode, a new raw-pointer card is reported as `new` and the process exits
  successfully.
- In no-new-debt mode, the same card exits with policy failure unless it exactly
  matches a live baseline or suppression entry.
- A suppression with an expired date does not hide the card; the card reports the
  expired entry and explains that it is policy-blocking in strict modes.
- A baseline entry for a card that no longer exists is reported as stale in repo or CI
  summaries so users can clean the policy ledger.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Policy matching can move from planned to experimental only after fixtures cover every
decision state and no-new-debt mode is exercised by repository dogfood CI.
