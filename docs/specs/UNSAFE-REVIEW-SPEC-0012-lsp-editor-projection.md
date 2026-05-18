# UNSAFE-REVIEW-SPEC-0012: LSP and editor projection

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

Editor integration should help authors repair unsafe-review cards before CI, but it
must not fork analysis behavior or make unverified source edits. The LSP projection
still needs implementation boundaries for diagnostics, hovers, commands, and saved
workspace state.

## Behavior

LSP v1 is read-only with copy-oriented commands. It projects saved or freshly computed
review cards into editor features:

- diagnostics for changed unsafe seams and missing obligations
- hover cards with contract, discharge, reach, witness, and policy status
- code actions that copy agent packets or witness commands to the clipboard
- commands that open related tests or policy entries when known
- status notifications for analyzer limitations and stale saved-card data

The LSP server must use the same core review-card DTOs as CLI and CI output.

## Workspace model

- The server may analyze the saved workspace, a supplied diff, or the current file.
- Unsaved buffers may be used only for best-effort diagnostics and must be marked as
  provisional.
- Diagnostics must be cleared or downgraded when saved-card data is stale relative to
  the buffer version.
- Multi-root workspaces must keep policy, baseline, and receipt state scoped to the
  matching repository root.

## Diagnostic mapping

- One diagnostic corresponds to one review card or one missing obligation on a card,
  according to user configuration.
- Diagnostic severity is derived from policy decision, not hazard class alone.
- Diagnostic code must include the stable card id or obligation id.
- Diagnostic message must include missing evidence and next action.
- Diagnostic related information may link safety comments, guards, tests, receipts,
  policy entries, and witness routes.

## Hover and commands

Hover output must include:

- operation and hazard class
- safety obligations
- present and missing evidence
- strongest witness receipt or route
- policy decision and support tier
- trust-boundary wording for static evidence

Commands must never apply source edits in v1. They may copy:

- an agent repair packet
- a witness command
- a suppression template
- a baseline template

## Non-goals

- no automatic fixes or code generation
- no editor-only analyzer behavior
- no blocking language-server diagnostics unless policy mode marks a card blocking
- no claim that an absence of diagnostics means the file is safe
- no duplicate truth outside review cards and policy decisions

## Required evidence

- saved-card fixture for diagnostics and hover rendering
- stale-buffer fixture showing provisional or cleared diagnostics
- command fixture for copy-agent-packet and copy-witness-command payloads
- multi-root scoping test for policy and baseline state
- documentation for editor trust boundary and read-only first behavior

## Acceptance examples

- Opening a file with a saved raw-pointer card produces one diagnostic at the unsafe
  seam and a hover that lists alignment and validity obligations.
- Editing the file without saving marks the diagnostic provisional or stale rather
  than claiming exact current analysis.
- Invoking copy-agent-packet returns the same packet JSON that the CLI would emit for
  the card.
- A baseline-known card is shown with lower severity than a policy-blocking new card.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

LSP projection can move from planned to experimental only after saved-card fixtures,
diagnostic mapping tests, and read-only command tests exist.
