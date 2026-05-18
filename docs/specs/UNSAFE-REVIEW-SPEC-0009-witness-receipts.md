# UNSAFE-REVIEW-SPEC-0009: Witness receipts

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for witness receipts.
Witness routing can recommend Miri, cargo-careful, sanitizer, Loom, Kani, or Crux work, but reviewers
also need a safe way to import what was actually configured, run, reached, and observed.

## Behavior

Receipts record configured, ran, test-targeted, or site-reached witness strength and limitations.

A receipt is evidence about a witness attempt. It may strengthen a card's evidence state, but it does
not prove soundness. Receipt import must preserve limitations and must not collapse different witness
strengths into a boolean pass/fail.

## Receipt strength

Receipt strength is ordered but not interchangeable:

1. `configured`: the repository or CI has a witness job configured.
2. `ran`: the witness command completed for the relevant revision.
3. `test_targeted`: the witness covered a test target that is related to the unsafe site.
4. `site_reached`: the witness observed execution of the specific unsafe site or seam.
5. `failed_relevant`: the witness failed in a way plausibly related to the card.
6. `inconclusive`: the witness ran but could not establish relevance.

A stronger receipt may satisfy obligations that require weaker witness evidence only when the witness
kind is appropriate for the hazard. For example, Miri reachability may support pointer validity review,
while Loom is relevant to concurrency obligations.

## Receipt schema

Imported receipts must include:

- receipt id and schema version;
- source tool (`miri`, `cargo-careful`, `asan`, `tsan`, `ubsan`, `loom`, `kani`, `crux`, or `manual`);
- repository revision, command, environment summary, and timestamp;
- status and strength;
- target package, target kind, and test name when known;
- matched card ids or site selectors;
- limitations and assumptions;
- raw artifact paths or digests when available.

Manual receipts are allowed only when they include owner, reason, and verification notes.

## Matching rules

Receipt matching must be explicit and auditable:

1. Prefer exact card id matches.
2. Fall back to site selectors only when path, range, and operation kind all match.
3. Do not apply a receipt across revisions unless the card identity is stable and the receipt declares
   the revision relationship.
4. Preserve unmatched receipts in import summaries for diagnosis.
5. Report stale, malformed, or unsupported receipts as limitations rather than evidence.

## Implementation still required

- Define serde-backed receipt DTOs and validation errors.
- Add receipt import CLI plumbing and policy/config paths.
- Implement receipt-to-card matching and unmatched receipt reporting.
- Update obligation evidence to reflect imported receipt strength.
- Add fixture receipts for Miri, cargo-careful, sanitizers, Loom, Kani, Crux, manual receipts,
  stale receipts, and malformed receipts.
- Render receipt evidence and limitations in human, JSON, Markdown, SARIF, LSP, and agent packets.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no parsing of arbitrary full tool logs as the only source of truth
- no claim that a passing witness proves absence of undefined behavior

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- receipt schema round-trip tests
- stale and malformed receipt tests

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- A Miri receipt that exactly matches a card id appears on that card as witness evidence.
- A receipt for a different revision is reported as stale unless explicitly declared compatible.
- A malformed receipt does not hide missing evidence and is visible in the import summary.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
