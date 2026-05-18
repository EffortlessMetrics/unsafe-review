# UNSAFE-REVIEW-SPEC-0011: PR and CI output

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for pr and ci output.

## Behavior

PR output must be sparse: summary first, at most a few high-confidence inline comments, durable JSON/Markdown/SARIF artifacts.

All CI and PR surfaces are projections of ReviewCard plus policy state. They must
not recalculate hazards, obligations, or witness strength independently.

## Artifacts

A CI run writes these artifacts when requested:

- `unsafe-review.cards.json`: canonical cards and policy classifications.
- `unsafe-review.summary.md`: reviewer-facing summary suitable for a GitHub step
  summary or PR body.
- `unsafe-review.sarif.json`: SARIF projection for code-scanning systems.
- `unsafe-review.witness-plan.json`: commands and missing evidence grouped by
  card/obligation for follow-up witness work.

Artifact paths are configurable but must be repository-relative by default.
Existing artifacts are overwritten atomically for the current run.

## Summary format

The Markdown summary includes:

1. run mode, policy mode, and exit-code explanation;
2. counts by policy classification and severity;
3. top changed unsafe seams sorted by review urgency;
4. missing evidence grouped by obligation;
5. links or paths to JSON, SARIF, and witness-plan artifacts;
6. trust-boundary language that says the tool reports review gaps, not proof.

When no cards are found, the summary says no changed unsafe seams were reported by
this static scan and still names the scan limitations.

## Inline comment policy

- Inline comments are opt-in.
- Comments are capped per run and deduplicated by card ID.
- Only changed lines or narrow diff ranges are eligible.
- A comment may include at most one primary next action plus a link/path to the
  full card artifact.
- Static limitations, baseline-known cards, and suppressed cards should generally
  stay in the summary rather than inline comments.

## SARIF requirements

- SARIF rules are derived from hazard and obligation taxonomy IDs.
- SARIF result fingerprints use stable card identity.
- SARIF locations point to the unsafe seam when known and include related
  locations for contract, discharge, test reach, or witness evidence.
- SARIF messages preserve the trust boundary and do not claim vulnerability or UB
  unless the card explicitly represents observed failing evidence.
- Policy state is carried in properties so code-scanning consumers can distinguish
  new debt from baseline-known or suppressed findings.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no chatty bot behavior that comments on every card by default
- no CI-specific alternate schema for cards

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- golden fixtures for Markdown summary, SARIF, and witness-plan output

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- A PR with five cards emits one summary and at most the configured inline comment
  cap.
- A baseline-known card appears in JSON and SARIF with baseline policy state, but
  it does not produce a new-debt failure.
- A SARIF consumer can correlate the same card across runs through fingerprints.

## Implementation backlog

1. Add Markdown summary renderer for CI/PR context.
2. Add SARIF renderer with taxonomy-derived rule IDs.
3. Add witness-plan artifact renderer.
4. Add CLI flags/config for artifact paths and inline comment caps.
5. Add golden tests for summary, SARIF, and witness-plan artifacts.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
