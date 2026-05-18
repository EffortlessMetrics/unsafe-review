# UNSAFE-REVIEW-SPEC-0011: PR and CI output

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for PR and CI output.
The CLI already emits human, JSON, and Markdown output; CI integrations still need durable artifacts,
compact summaries, stable failure rules, and a reviewer-friendly projection.

## Behavior

PR output must be sparse: summary first, at most a few high-confidence inline comments, durable JSON/Markdown/SARIF artifacts.

CI output is a projection of review cards and policy decisions. It must never invent independent
findings and must link each annotation back to a review-card id. The default posture remains advisory
unless policy explicitly opts into a failing mode.

## Artifact set

A complete CI run should be able to produce these files under a configured artifact directory:

- `unsafe-review.cards.json`: canonical machine-readable analysis output.
- `unsafe-review.summary.md`: GitHub step summary / PR body compatible Markdown.
- `unsafe-review.sarif.json`: SARIF 2.1.0 results for code scanning systems.
- `unsafe-review.policy.json`: policy decision details when policy is enabled.
- `unsafe-review.witness-plan.json`: witness routes and commands for follow-up jobs.

The artifact directory must be created if missing. Artifact writes must be atomic enough that partial
files are not mistaken for successful output after a failed run.

## GitHub summary projection

The summary renderer must include:

1. run posture: advisory, no-new-debt, or blocking;
2. total cards and open actionable gaps;
3. counts by class, priority, policy state, and witness status;
4. top cards sorted by priority, then confidence, then path and line;
5. suggested next commands for the top unresolved card set;
6. trust boundary text: static review evidence, not a soundness proof.

The summary must remain useful when no cards are emitted and when policy parsing fails.

## Inline comment policy

Inline comments are optional and must be conservative:

- comment only on changed lines in the PR diff;
- cap comments per run, with a default no larger than five;
- prefer high-confidence actionable cards over informational cards;
- do not comment on cards hidden by valid suppressions;
- include the card id and link or path to the full artifact;
- deduplicate comments across reruns when the hosting platform supports it.

## SARIF mapping

SARIF output must map each review card as follows:

- `ruleId`: hazard or obligation family plus review class;
- `level`: derived from priority and policy state;
- `message`: short next-action summary;
- `locations`: primary unsafe site path, range, and snippet where available;
- `partialFingerprints`: stable card id and site identity;
- `properties`: hazards, obligations, missing evidence, witness routes, policy state,
  and trust-boundary metadata.

## Implementation still required

- Add SARIF rendering with golden tests.
- Add GitHub summary rendering that is distinct from generic Markdown when needed.
- Add CLI flags or subcommands for artifact directory, summary file, SARIF file, and witness-plan file.
- Wire policy decisions into process exit codes and CI text.
- Implement optional inline-comment payload generation without requiring network access.
- Add artifact write failure tests and no-card success tests.
- Document a minimal GitHub Actions workflow using generated artifacts.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no direct network calls to GitHub from the core analyzer
- no noisy bot behavior by default

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- SARIF schema validation or parser smoke coverage
- GitHub summary golden coverage

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- A CI run in advisory mode writes JSON, summary Markdown, and SARIF artifacts and exits zero.
- A CI run in no-new-debt mode exits non-zero when an unsuppressed new actionable card exists.
- The SARIF result contains the same card id as canonical JSON output.
- Inline comment payload generation emits no more than the configured cap and only for changed lines.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
