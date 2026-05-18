# UNSAFE-REVIEW-SPEC-0016: Fixtures, calibration, support tiers

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for fixtures, calibration, support tiers.

## Behavior

Every promoted claim must map to fixture/golden/receipt proof and a support-tier entry.

Fixtures and calibration data are the promotion gate for analyzer claims. A claim
can be documented before it is fully implemented, but support-tier tables must
make its status visible.

## Fixture types

- `positive`: changed unsafe seam should produce one or more review cards.
- `negative`: safe or irrelevant change should produce no cards.
- `classification`: a seam should map to a specific operation, hazard, and
  obligation set.
- `evidence`: contract, discharge, reach, or witness evidence should attach to a
  specific obligation lane.
- `policy`: baseline and suppression state should classify cards and exit codes.
- `projection`: JSON, Markdown, SARIF, LSP, or agent-packet output should remain
  stable.
- `receipt`: imported witness artifacts should map to expected receipt strength.

Each fixture includes the source, diff or repo mode input, expected cards or
artifacts, and a short README when the safety scenario is not obvious.

## Calibration workflow

- New analyzer behavior starts as `experimental` until positive and negative
  fixtures exist.
- A behavior can become `supported` only when golden output, support-tier entry,
  and at least one dogfood or external-repo receipt exist.
- False positives and false negatives are tracked against fixture IDs or linked
  repo receipts.
- Calibration updates preserve old fixtures unless the spec intentionally changes;
  intentional changes must explain migration impact.
- Metrics report counts and rates, not broad safety claims.

## Support tiers

Support-tier entries include:

- feature or hazard name;
- tier: `planned`, `experimental`, `supported`, or `deprecated`;
- input surface covered;
- proof required for promotion;
- known limitations and non-goals.

A feature listed as `planned` may have a spec and backlog but must not be marketed
as available. A `supported` feature can still have limitations, and those
limitations must appear in output when relevant.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no benchmark leaderboard as a proxy for proof
- no promotion based only on happy-path fixtures

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- support-tier checks that fail when documented support lacks matching fixtures

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- A new hazard classifier cannot move to supported until positive, negative, and
  projection fixtures exist.
- A SARIF renderer remains planned until at least one projection golden fixture is
  checked in.
- A feature with known false positives names those limitations in support-tier
  documentation and output.

## Implementation backlog

1. Add fixture metadata describing type, claim covered, and support-tier target.
2. Extend `cargo xtask check-pr` to verify support-tier/fixture consistency.
3. Add calibration reports for false-positive and false-negative tracking.
4. Add projection and receipt fixtures for the unimplemented output surfaces.
5. Document promotion requests in handoff notes or PR descriptions.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
