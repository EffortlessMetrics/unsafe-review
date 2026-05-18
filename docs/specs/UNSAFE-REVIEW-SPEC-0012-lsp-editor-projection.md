# UNSAFE-REVIEW-SPEC-0012: LSP and editor projection

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for LSP and editor projection.
Developers should be able to inspect review cards while editing without granting the tool authority to
rewrite code or claim stronger evidence than the saved analysis contains.

## Behavior

LSP v1 is read-only: diagnostics, hover, status, copy agent packet, copy witness command, open related test.

The LSP server projects saved analysis artifacts into editor features. It must not run expensive repo
analysis on every keystroke. The first implementation should load an existing cards JSON artifact,
optionally refresh on file save or explicit command, and expose diagnostics derived from review cards.

## Data model

The editor projection consumes the canonical review-card JSON plus optional policy and receipt data.
Each editor diagnostic must retain the card id and enough properties for the user to find the same
card in CLI output.

Diagnostic mapping:

- range: primary unsafe site range, falling back to the first line of the site snippet;
- severity: priority plus policy state, never stronger than the CLI decision;
- source: `unsafe-review`;
- code: stable card id;
- message: concise missing-evidence or next-action summary;
- related information: related tests, witness routes, policy entry, and receipt notes.

## Editor commands

LSP v1 must expose commands that copy or open generated text rather than mutate source code:

- copy card id;
- copy agent packet;
- copy witness command for a selected route;
- open related test path when available;
- open rendered Markdown detail for the card;
- refresh from saved artifact.

Code actions may present these commands, but they must be marked non-preferred when they require human
review. No automatic fixes are part of v1.

## Refresh and trust boundary

The server must make freshness explicit:

- show artifact path and timestamp in status;
- mark diagnostics stale when the source file is newer than the artifact;
- avoid silently mixing cards from different repository roots;
- report parse/import failures as LSP status diagnostics, not as unsafe-code findings.

## Implementation still required

- Add an LSP crate or binary that can load canonical card JSON.
- Define an editor-facing DTO for diagnostics, hover content, status, and commands.
- Implement saved-artifact watching or explicit refresh.
- Add hover rendering for contract, discharge, reachability, witness, and policy evidence.
- Add code actions for copy-agent-packet, copy-witness-command, and open-related-test.
- Add fixtures for no-card, stale-artifact, malformed-artifact, and multi-card files.
- Document setup for at least VS Code-compatible clients without committing to one editor extension.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no automatic source edits in LSP v1
- no background network access

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- LSP protocol golden tests or DTO snapshot tests
- stale-artifact behavior tests

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- Loading a cards JSON artifact produces one diagnostic with the same card id as CLI JSON.
- A stale artifact is visibly marked stale and does not pretend to describe unsaved edits.
- The copy-agent-packet command returns the same packet content as the CLI context command.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
