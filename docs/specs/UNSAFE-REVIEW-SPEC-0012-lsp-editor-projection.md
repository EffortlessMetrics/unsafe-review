# UNSAFE-REVIEW-SPEC-0012: LSP and editor projection

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for lsp and editor projection.

## Behavior

LSP v1 is read-only: diagnostics, hover, status, copy agent packet, copy witness command, open related test.

The editor projection consumes saved workspace state and ReviewCard artifacts. It
must not run heavyweight witnesses, rewrite code, or infer additional hazards
inside the language server.

## Inputs

The LSP server accepts:

- repository root and workspace configuration;
- saved Rust source files from disk;
- optional precomputed `unsafe-review.cards.json` artifacts;
- policy files used to classify card state;
- explicit user command to refresh static analysis.

Unsaved buffer analysis is deferred. Diagnostics may be stale until the file is
saved or the refresh command completes, and the status item must make that clear.

## Diagnostics

- Diagnostic ranges point to the unsafe seam when known.
- Severity is derived from policy classification and support tier, not from a
  separate editor-only severity mapping.
- Diagnostic codes use hazard or obligation taxonomy IDs.
- Suppressed or baseline-known cards are shown as lower-noise diagnostics or code
  lenses according to editor configuration, but remain discoverable.
- Diagnostic data includes card ID and artifact path so commands can open the
  canonical card.

## Hover and commands

Hover text includes the operation, hazard, obligations, missing evidence, next
action, and a short limitation statement. It links to the canonical artifact when
available.

Read-only commands include:

- copy review card JSON;
- copy agent packet JSON;
- copy witness command;
- open related test or fixture location;
- open policy entry or baseline match;
- refresh static cards for saved files.

## Staleness and failure behavior

- If source changed after the card artifact was generated, diagnostics are marked
  stale rather than silently discarded.
- If analysis fails, the server reports a workspace status error and leaves the
  last known cards visible as stale.
- If a file is outside configured input scope, the server reports no diagnostic
  and explains the exclusion in status/log output.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no automatic edits or generated fixes in v1
- no unsaved-buffer scanner until saved-workspace behavior is proven

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- protocol fixtures for diagnostics, hover, commands, stale artifacts, and
  analysis failure

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- Opening a file with a matching saved card shows a diagnostic at the seam and a
  hover with missing obligation evidence.
- Editing a file without saving marks previous diagnostics stale rather than
  implying the issue disappeared.
- Copy-agent-packet returns the same packet that the CLI would render for the card.

## Implementation backlog

1. Define LSP DTO projection from ReviewCard and policy state.
2. Add saved-artifact loading and staleness detection.
3. Implement diagnostics and hover protocol fixtures.
4. Implement read-only commands for card, packet, witness, related test, and
   policy navigation.
5. Add editor smoke documentation after the protocol fixtures are stable.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
