# UNSAFE-REVIEW-SPEC-0011: PR and CI output

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

PR and CI integrations must make unsafe-review evidence easy to act on without
spamming reviewers or creating a second source of truth. SARIF, GitHub summaries,
inline comments, and durable artifacts still need precise implementation rules.

## Behavior

PR output is sparse and artifact-first:

1. Emit canonical review-card JSON for machines.
2. Emit Markdown summary for humans.
3. Emit SARIF for code-scanning consumers.
4. Optionally emit a small number of high-confidence inline comments.

All PR and CI projections must derive from the same review cards and policy decisions.
They must preserve the trust boundary: `unsafe-review` reports review evidence, not a
memory-safety proof and not a UB-free badge.

## Required artifacts

A CI run should be able to produce these repository-relative artifacts:

- `unsafe-review.cards.json`: canonical card list and policy decisions.
- `unsafe-review.summary.md`: reviewer-facing summary suitable for a GitHub job
  summary or PR comment.
- `unsafe-review.sarif`: SARIF projection for changed-file diagnostics.
- `unsafe-review.witness-plan.json`: optional list of witness commands or receipts
  needed for missing evidence.

Generated artifacts must not be committed by default.

## SARIF projection

- One SARIF result corresponds to one review card.
- SARIF `ruleId` must be the operation or obligation family, not a volatile card id.
- SARIF location must use repository-relative paths and best-known line or byte span.
- SARIF message must include missing evidence and next action.
- SARIF properties must include card id, hazard class, obligation ids, policy decision,
  support tier, and witness routes.
- SARIF must not mark a card as an error unless policy mode decided it is blocking.

## GitHub summary projection

The summary must include:

- counts by policy decision and severity
- top changed unsafe seams grouped by file
- missing obligation summary
- witness routes and receipt status
- explicit trust-boundary wording
- links or artifact names for full JSON, SARIF, and witness plan

## Inline comment policy

Inline comments are optional and off by default until calibrated. When enabled they
must be limited by policy and should prefer:

- changed lines only
- high-confidence card locations
- new or blocking cards
- one comment per card at most
- a run-level cap to avoid review spam

Ambiguous locations, baseline-known cards, and static limitations should stay in the
summary instead of inline comments.

## Non-goals

- no direct dependency on a single CI provider in core analysis
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside review cards and policy decisions
- no automatic source edits from PR comments
- no generated artifact commits by default

## Required evidence

- SARIF schema smoke test over a fixture card
- Markdown summary golden test with new, baseline-known, and suppressed counts
- inline-comment selection tests for caps and changed-line filtering
- CI proof that generated SARIF artifacts are not tracked
- documentation for recommended GitHub Actions usage

## Acceptance examples

- A PR with three new cards emits three JSON cards, three SARIF results, one Markdown
  summary, and zero inline comments when inline comments are disabled.
- In no-new-debt mode, a new blocking card is represented as a SARIF error while a
  baseline-known card remains a warning or note.
- A card outside changed lines appears in the summary and artifacts but is not selected
  for inline comment.
- The GitHub summary includes the trust-boundary wording and links to the durable
  artifacts.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

PR and CI output can move from planned to experimental only after SARIF, summary, and
inline-comment selection fixtures are golden-tested and used in a dogfood workflow.
