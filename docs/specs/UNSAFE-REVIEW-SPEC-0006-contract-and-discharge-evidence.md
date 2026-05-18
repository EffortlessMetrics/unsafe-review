# UNSAFE-REVIEW-SPEC-0006: Contract and discharge evidence

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for contract and discharge evidence.

## Behavior

Mine # Safety docs, SAFETY comments, asserts, length/capacity/alignment/nullability guards, type wrappers, privacy boundaries, and policy receipts.

Contract and discharge evidence must be attached to the obligation it supports,
not only to the card that contains the obligation. A single unsafe seam can carry
multiple obligations, and evidence for one obligation must not discharge another
obligation by accident.

## Obligation-level evidence model

Each obligation entry records four independent evidence lanes:

- `contract`: documentation or API shape that states the caller/callee safety
  precondition.
- `discharge`: local code that checks, preserves, or constructs the precondition
  before the unsafe operation executes.
- `reach`: static evidence that relevant tests or examples can reach the seam.
- `witness`: imported dynamic/formal receipt evidence that exercised or checked
  the seam.

Each lane has a state of `present`, `missing`, `unknown`, or `not_applicable`.
`unknown` is used when the stable scanner cannot determine the answer without
claiming proof. `not_applicable` is only allowed when the obligation category does
not need that lane.

## Matching rules

- Evidence is matched by source range first, then by enclosing item, then by
  nearest preceding safety comment or guard expression.
- A guard can satisfy only the obligation kinds it names or structurally checks;
  for example, a length guard may support bounds obligations but not alignment.
- `# Safety` docs satisfy public unsafe API contract obligations when attached to
  the public unsafe item, not when found on an unrelated helper.
- `SAFETY:` comments explain discharge intent, but do not become `present`
  discharge evidence unless a nearby expression or wrapper also performs a check
  or construction step.
- When multiple obligations share one expression, the card repeats the evidence
  reference per obligation so downstream surfaces can filter without re-inferring.

## Output requirements

Review-card JSON includes `obligation_evidence` entries with stable obligation
IDs, lane states, source spans when known, and a limitation string for every
`missing` or `unknown` lane. Human and Markdown output may summarize, but must
preserve enough wording for a reviewer to know which obligation is unresolved.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no dataflow proof across arbitrary control flow in the stable scanner

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- golden tests showing evidence attached to the correct obligation when one card
  has multiple obligations

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- A `# Safety` section on a public unsafe function marks the contract lane present
  for the public API obligation.
- A `SAFETY:` comment without a corresponding guard or wrapper remains advisory
  and does not mark discharge present.
- A bounds check for `get_unchecked` does not satisfy an alignment obligation on a
  neighboring raw-pointer dereference.

## Implementation backlog

1. Persist lane-level evidence in the domain model and serde DTOs.
2. Replace card-wide guard summaries with obligation-specific evidence matching.
3. Add multi-obligation fixtures that prove evidence does not bleed between
   obligations.
4. Update human and Markdown renderers to group missing lanes by obligation.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
