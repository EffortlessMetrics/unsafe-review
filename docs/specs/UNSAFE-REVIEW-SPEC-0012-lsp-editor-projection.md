# UNSAFE-REVIEW-SPEC-0012: LSP and editor projection

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for lsp and editor projection.

The current product is CLI-first. Editors still need a read-only projection that
lets reviewers inspect cards, copy packets, and run witness commands without
changing code automatically.

## Implementation status

Planned. No LSP server is implemented yet.

## Behavior

LSP v1 is read-only: diagnostics, hover, status, copy agent packet, copy witness command, open related test.

### Server model

The LSP server must run analysis from saved workspace files. It must not analyze
unsaved editor buffers unless a later spec defines the trust and performance
model. The server may reuse cached analysis results, but diagnostics must expose
which file revision or workspace snapshot produced them.

### Diagnostics

Each review card maps to one primary diagnostic when the card has a concrete
source location. Diagnostics must include:

- card id;
- hazard id and label;
- operation id and label;
- severity;
- missing evidence summary;
- policy state when policy data is available;
- limitation statement for static-only evidence.

Cards without precise source locations must be exposed through a workspace status
or virtual document, not pinned to arbitrary lines.

### Hover

Hover content must show a compact card:

- title;
- why this unsafe seam matters;
- present evidence;
- missing obligations;
- suggested witness route;
- card id for durable reference.

Hover text must be derived from the canonical card model and must not introduce
new findings.

### Code actions

Read-only code actions may:

- copy an agent packet;
- copy a witness command;
- open the detailed Markdown explanation;
- open a related test or fixture when known;
- copy a baseline or suppression draft only when the draft is explicitly marked
  incomplete and requires owner, reason, and expiry.

The LSP server must not apply code edits, insert safety comments, or add
suppressions in v1.

### Refresh and performance

Analysis should be debounced on save. Large workspaces may fall back to changed
files only. Timeouts must be reported as partial analysis, not as an all-clear.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no automatic fixes
- no unsaved-buffer data model in v1
- no background network access

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- saved-workspace diagnostic golden tests
- hover snapshot tests
- code-action tests proving v1 actions are read-only
- partial-analysis and timeout tests

## Acceptance examples

- A changed unsafe seam produces one diagnostic whose card id matches CLI JSON.
- A hover for that diagnostic lists missing evidence and the next witness route.
- Copy-agent-packet returns the same packet as CLI context output for the card.
- A timeout reports partial analysis and does not clear existing findings as safe.
- If evidence is not knowable statically, the diagnostic names the limitation
  instead of overclaiming.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
