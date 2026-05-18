# UNSAFE-REVIEW-SPEC-0009: Witness receipts

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for witness receipts.

The analyzer can recommend witness routes, but it still needs a way to import and
compare proof-of-work from dynamic tools such as Miri, cargo-careful, sanitizers,
Loom, Kani, and Crux. Receipts prevent the UI from confusing "you should run this"
with "this was run and covered the site."

## Implementation status

Planned. Witness routing exists; receipt import and receipt-to-card matching are
not implemented yet.

## Behavior

Receipts record configured, ran, test-targeted, or site-reached witness strength and limitations.

### Receipt strengths

Receipt strength is ordered from weakest to strongest:

1. `configured`: the repository config mentions a relevant witness tool;
2. `command_generated`: `unsafe-review` generated a route-specific command;
3. `ran`: the command ran and produced a parseable result;
4. `targeted`: the run targeted a package, test, module, or feature that is
   plausibly connected to the card;
5. `site_reached`: the witness output proves the card's unsafe seam was executed
   or model-checked;
6. `passed`: the witness completed successfully at the recorded strength;
7. `failed`: the witness found a failure or undefined-behavior signal;
8. `inconclusive`: the witness ran but timed out, skipped the target, or used an
   unsupported configuration.

A stronger receipt may satisfy weaker display needs, but the original strength
must be preserved.

### Receipt schema

Each receipt must include:

- receipt schema version;
- tool name and version when available;
- command line or normalized invocation;
- working directory relative to repo root;
- timestamp;
- exit status;
- environment limitations that affect interpretation;
- target package/test/module/features;
- matched card ids, if known;
- coverage or reachability evidence, if known;
- raw artifact path or digest;
- parser confidence;
- limitations and unsupported cases.

Receipts must be importable from durable files. The analyzer must not infer a
successful receipt from a command recommendation alone.

### Tool-specific minimums

- Miri: record cargo subcommand, target, flags, unsupported operations, and any
  UB diagnostics.
- cargo-careful: record target, feature set, and whether the run reached tests or
  examples relevant to the card.
- Sanitizers: record sanitizer kind, compiler/toolchain, target, and whether the
  unsafe seam's test was exercised.
- Loom: record model name/test, explored branch or permutation summary when
  available, and incompleteness bounds.
- Kani/Crux: record proof harness, unwind/bound settings, result, and properties
  checked.

### Matching

Receipt-to-card matching must be explicit and conservative. A receipt may attach
to a card through:

- embedded card id;
- generated witness command id;
- test target known to mention the unsafe seam;
- exact source span in the witness artifact.

Weak or ambiguous matches must be shown as `candidate_receipt`, not as evidence
that discharges an obligation.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no claim that passing one witness proves the unsafe code correct
- no parsing of opaque logs without a confidence label

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- per-tool receipt parser fixtures for pass, fail, timeout, unsupported, and
  inconclusive cases
- matching tests for embedded id, command id, target mention, exact span, and
  ambiguous candidate matches
- renderer tests proving receipt strength and limitations are visible

## Acceptance examples

- A Miri pass with an embedded card id attaches to that card as `passed` at the
  recorded strength.
- A sanitizer timeout imports as `inconclusive` and does not discharge missing
  witness evidence.
- A Kani proof for a different harness is shown as a candidate only, not as
  site-reached evidence.
- A witness route recommendation without a receipt is displayed as planned work,
  never as completed evidence.
- If receipt coverage is not knowable, the card names the limitation instead of
  overclaiming.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
