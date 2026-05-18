# UNSAFE-REVIEW-SPEC-0009: Witness receipts

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

Witness tools such as Miri, `cargo-careful`, sanitizers, Loom, Shuttle, Kani, and
Crux can raise confidence in a review card, but only if `unsafe-review` records
exactly what was configured, what ran, what code or tests it reached, and what it
cannot prove. The receipt importer still needs a product contract before it is
implemented so receipts never become an unqualified memory-safety claim.

## Behavior

A receipt is an immutable, machine-readable record that may be attached to one or
more review cards. Receipts report witness strength as one of the following ordered
states:

1. `configured`: the witness is configured in repository policy or workflow files.
2. `ran`: a run completed and the command, status, and artifact identity are known.
3. `test_targeted`: the run targeted a test, binary, example, or property that is
   related to the card's changed seam.
4. `site_reached`: the run provides evidence that execution reached the reviewed
   unsafe seam or wrapper.

Receipt import must preserve the distinction between tool success and safety proof.
A passing receipt may discharge a card's witness obligation only at the matching
strength level and only for the operation classes the witness is documented to cover.
A failing receipt must be surfaced as review evidence, not hidden by baseline or
suppression logic.

## Receipt data contract

Every imported receipt must include:

- `receipt_id`: stable identifier derived from witness kind, command, artifact hash,
  and repository-relative target.
- `witness_kind`: one of `miri`, `cargo_careful`, `asan`, `tsan`, `msan`, `loom`,
  `shuttle`, `kani`, `crux`, or `custom`.
- `strength`: one of the ordered states above.
- `command`: the exact command or workflow step when known.
- `status`: `passed`, `failed`, `timed_out`, `skipped`, or `unknown`.
- `artifact`: optional repository-relative path or external artifact descriptor.
- `target`: optional test, binary, example, package, or property name.
- `covered_cards`: zero or more review-card ids matched by policy, target, or reach
  evidence.
- `limitations`: human-readable limitations such as unsupported target, no reach
  proof, nightly-only witness, or incomplete sanitizer coverage.
- `observed_at`: timestamp if supplied by the artifact, otherwise omitted rather
  than synthesized.

Importers may add witness-specific fields, but review-card classification must only
consume the normalized fields above.

## Matching rules

- Match by exact review-card id when a receipt names cards explicitly.
- Otherwise match by repository-relative path and target metadata.
- `site_reached` requires either direct instrumentation output, a supported coverage
  receipt, or a policy-approved reach marker. A passing test name alone is not site
  reach evidence.
- A receipt with unknown status or unknown target may be attached as context but must
  not discharge an obligation.
- Stale receipts older than the configured policy window remain visible but count as
  `configured` at most.

## CLI and output requirements

- `unsafe-review check` must be able to load receipts from configured paths.
- JSON output must include receipt summaries on each affected obligation.
- Human and Markdown output must show the strongest receipt state and limitations.
- Receipt import errors must be diagnostics unless policy mode explicitly escalates
  them to failure.

## Non-goals

- no claim that a passing witness proves absence of undefined behavior
- no requirement to parse every native artifact format in v1
- no automatic test generation or witness execution
- no receipt mutation after import
- no duplicate truth outside this spec and linked policy files

## Required evidence

- fixture receipts for at least one passing and one failing Miri run
- fixture receipts for one sanitizer route that is configured but not site-reached
- JSON golden tests for each receipt strength level
- human output smoke coverage for attached and unattached receipts
- policy documentation for receipt paths, staleness windows, and custom witnesses

## Acceptance examples

- A changed raw-pointer card with a matching Miri receipt at `site_reached` shows the
  Miri obligation as present, includes the receipt id, and still lists any unrelated
  missing obligations.
- A sanitizer workflow configured for the package but without reach evidence is shown
  as `configured`, not `site_reached`.
- A failing Kani proof attached to a card is visible in JSON and Markdown output and
  cannot be suppressed unless the exact card id is suppressed by policy.
- A stale receipt remains listed as context and explains that it did not discharge the
  current witness obligation.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Receipt import can move from planned to experimental only after normalized DTOs,
fixture receipts, JSON golden tests, and at least one dogfood receipt from this
repository exist.
