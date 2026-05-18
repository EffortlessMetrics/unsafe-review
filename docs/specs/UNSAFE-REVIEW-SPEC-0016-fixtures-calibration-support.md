# UNSAFE-REVIEW-SPEC-0016: Fixtures, calibration, support tiers

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` can only make useful claims if each claim is backed by fixtures,
golden output, and support-tier documentation. The calibration system still needs an
implementation contract for promoting analyzer behavior from scaffold to experimental
and eventually usable alpha.

## Behavior

Every promoted capability must map to:

1. one or more positive fixtures that should emit review cards or evidence,
2. one or more negative fixtures that should not emit false cards,
3. golden JSON output for canonical DTO shape,
4. human or Markdown smoke output for reviewer usability,
5. support-tier documentation listing proof and known limits.

Fixture and calibration data are product evidence, not incidental tests. Changes to
expected output must be reviewed as behavior changes.

## Fixture contract

A fixture directory should contain:

- `Cargo.toml` and source files for the minimal Rust crate under review
- `change.diff` when testing diff-mode behavior
- `expected.cards.json` for canonical review-card output
- optional receipt, policy, baseline, suppression, SARIF, packet, or LSP golden files
- a short README or inline metadata when the fixture exercises a subtle limitation

Fixtures must be small, deterministic, and free of network access. They should prefer
realistic unsafe idioms over synthetic parser traps unless parser behavior is the
point of the fixture.

## Calibration metrics

Calibration runs should report:

- expected card count versus actual card count
- expected operation and hazard classification accuracy
- obligation evidence precision and recall for known fixtures
- false-positive and false-negative notes by fixture
- unsupported or static-limitation cases
- support-tier deltas caused by the run

A failure to detect a fixture's expected card is a release blocker for any capability
that claims support for that operation family.

## Support-tier rules

Supported tiers are:

- `planned`: specified but not implemented.
- `scaffold`: implementation exists but is not calibrated enough for reliable signal.
- `experimental`: fixture-backed, useful for dogfood, still conservative.
- `deferred`: intentionally out of current scope.

A capability may move from `scaffold` to `experimental` only when fixture evidence,
JSON golden tests, and at least one dogfood run exist. Claims beyond `experimental`
require a separate spec or ADR.

## Non-goals

- no claim that fixtures prove analyzer completeness
- no hidden benchmark or private corpus as the only promotion evidence
- no support-tier promotion without updating documentation
- no generated tests as a substitute for reviewed fixtures
- no duplicate truth outside fixtures, golden outputs, and support-tier docs

## Required evidence

- fixture harness covering every currently emitted operation family
- negative fixtures for safe code and comment-only false positives
- golden update process documented in contributor docs or xtask help
- support-tier check that rejects unknown tier names
- dogfood receipt or CI artifact for each promoted experimental capability

## Acceptance examples

- Adding support for a new unsafe operation requires a positive fixture, an expected
  card, and a support-tier row before promotion.
- A safe-code fixture with no unsafe seams emits an empty card list and stays empty in
  golden output.
- Updating expected JSON after changing the schema requires updating the related spec
  or documenting compatibility behavior.
- A capability listed as experimental has at least one fixture proof and one known
  limits entry.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

The calibration system itself can move from scaffold to experimental only after the
fixture harness enforces expected card output and support-tier documentation in CI.
