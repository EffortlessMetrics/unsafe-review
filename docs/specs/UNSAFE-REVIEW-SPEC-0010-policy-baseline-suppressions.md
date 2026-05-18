# UNSAFE-REVIEW-SPEC-0010: Policy, baseline, suppressions

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for policy, baseline, suppressions.

The current analyzer can emit review cards in advisory mode, but it still needs the
policy engine that decides whether those cards are new debt, already accepted
risk, intentionally suppressed noise, or a release-blocking finding.

## Implementation status

Planned. The public API already names `advisory`, `no-new-debt`, and `blocking`
policy modes; this spec defines the behavior that must exist before those modes
are more than labels.

## Behavior

Use exact counted identity for baselines and suppressions. Default advisory, then no-new-debt, then calibrated blocking.

### Policy modes

- `advisory`: never fails the command because of review cards. Output must still
  mark each card as open, baseline-matched, suppression-matched, or policy-ignored.
- `no-new-debt`: fails only when a card is not covered by the active baseline or
  an active suppression and the card's severity is at or above the configured
  threshold.
- `blocking`: fails on every non-suppressed card at or above the configured
  threshold, including baseline-matched cards, unless the baseline entry is
  explicitly marked `accepted_for_blocking`.

### Baseline file

The baseline file records cards that already existed before enforcement began.
Each entry must include:

- stable card id;
- operation/hazard pair;
- normalized relative path;
- line-span fingerprint or enclosing-item fingerprint;
- first-seen timestamp;
- last-seen timestamp;
- optional expiry timestamp;
- optional owner;
- optional rationale.

Baseline matching must be deterministic and explainable. A card is
baseline-matched only when its stable id matches and at least one location
fingerprint still matches. If the id matches but the fingerprint does not, the
card is `baseline_drifted` and must be treated as new debt in `no-new-debt` and
`blocking` modes.

### Suppression file

Suppressions are narrower than baselines and must be intentionally scoped. Each
suppression entry must include:

- stable card id or explicit match expression;
- normalized relative path or path glob;
- reason;
- owner;
- expiry timestamp;
- allowed hazard ids;
- allowed operation ids;
- maximum severity covered by the suppression.

A suppression without owner, reason, or expiry is invalid. Expired suppressions
must be reported and must not suppress cards. A suppression that matches no card
must be reported as stale.

### Exit status

- `0`: command completed and policy passed.
- `1`: command completed and policy failed.
- `2`: invalid user input, invalid policy file, unreadable baseline, or malformed
  suppression.
- `3`: internal analyzer error.

Output formats must include enough policy metadata for CI and editor consumers to
reconstruct the decision without re-running policy.

### Update workflow

The CLI must eventually support a deliberate baseline refresh command. Refreshing
a baseline must not silently copy suppressions, and it must preserve first-seen
metadata for unchanged entries.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no automatic expiry extension
- no repository-wide waiver that hides all unsafe findings

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- baseline match, drift, expiry, and stale-entry golden tests
- suppression owner/reason/expiry validation tests
- exit-code tests for `advisory`, `no-new-debt`, and `blocking`

## Acceptance examples

- A card matching the active baseline passes `no-new-debt` and is rendered as
  baseline-matched.
- A card with the same id but a changed fingerprint is rendered as
  `baseline_drifted` and fails `no-new-debt`.
- An expired suppression is reported, does not hide the card, and causes policy
  failure if the unsuppressed card exceeds the active threshold.
- A stale suppression is reported as cleanup debt without causing a policy
  failure in advisory mode.
- Invalid policy files fail before analysis results are trusted.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
