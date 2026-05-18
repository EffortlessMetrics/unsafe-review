# UNSAFE-REVIEW-SPEC-0009: Witness receipts

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for witness receipts.

## Behavior

Receipts record configured, ran, test-targeted, or site-reached witness strength and limitations.

Witness receipts are imported facts about external checks. They improve review
routing and evidence display, but they do not prove memory safety and they do not
silence cards unless policy explicitly allows that transition.

## Receipt schema

A receipt contains:

- `receipt_id`: stable digest of tool, command, normalized target, and source
  artifact path.
- `tool`: one of `miri`, `cargo-careful`, `sanitizer`, `loom`, `kani`, `crux`, or
  `manual`.
- `strength`: one of `configured`, `ran`, `test_targeted`, or `site_reached`.
- `command`: normalized command or documented manual procedure.
- `target`: package, test, binary, example, or harness that produced the receipt.
- `matched_cards`: review-card IDs or obligation IDs the receipt claims to cover.
- `artifact`: path to machine output, log, or attestation file.
- `timestamp`: ISO-8601 timestamp supplied by the runner or import command.
- `limitations`: explicit caveats such as ignored tests, unsupported platform,
  missing feature flags, or partial harness reachability.

## Strength rules

- `configured`: configuration exists, but there is no evidence that the witness
  ran for this revision.
- `ran`: the witness command completed for the workspace or package, but no card
  or obligation reachability was established.
- `test_targeted`: the witness ran a target that static reachability maps to the
  unsafe seam or enclosing item.
- `site_reached`: the witness artifact includes a site marker, coverage marker,
  or harness assertion that reaches the specific unsafe seam or obligation.

Receipts may only increase strength when the import can show the required fact in
the artifact. A failed witness run is still imported when useful, but it must be
reported as failing evidence and cannot be treated as discharge.

## Import behavior

- Importers accept explicit artifact paths and never scan unbounded directories by
  default.
- Importers normalize local absolute paths to repository-relative paths before
  writing durable output.
- Unknown tool versions are allowed but recorded as limitations.
- Receipts from generated files or non-Rust harnesses require a policy allowlist
  entry before they affect routing.
- Duplicate receipts collapse by `receipt_id`; changed limitations or artifacts
  produce a new receipt.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no requirement that every repository uses every witness tool
- no automatic execution of heavyweight witnesses during normal `check`

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- importer fixtures for at least one passing, failing, and partially matched
  receipt artifact

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- A Miri log that ran the package but lacks seam reachability imports as `ran`,
  not `site_reached`.
- A sanitizer artifact with a site marker for the changed seam attaches witness
  evidence to the matching obligation.
- A stale receipt from a different card ID is preserved as historical data but
  does not satisfy current card evidence.

## Implementation backlog

1. Define receipt DTOs and JSON schema fixtures.
2. Add `unsafe-review receipt import` for explicit artifact paths.
3. Wire receipt matching into obligation-level witness evidence.
4. Add fixture artifacts for Miri, sanitizer, and one formal witness before
   promoting beyond planned support.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
