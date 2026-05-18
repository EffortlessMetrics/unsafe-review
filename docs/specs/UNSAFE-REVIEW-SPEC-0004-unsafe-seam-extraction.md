# UNSAFE-REVIEW-SPEC-0004: Unsafe seam extraction

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for unsafe seam extraction.

## Behavior

Detect unsafe blocks, unsafe functions, unsafe impls, FFI, static mut, raw pointer operations, MaybeUninit, transmute, set_len, Pin, Send/Sync, and related seams.
Extraction should use a stable syntax substrate that records syntax node kind,
byte range, line/column, and snippet text before card-specific classifiers consume
those facts. Line-based scanning may remain as a compatibility path while syntax
facts are adopted behind fixture-backed card output.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
