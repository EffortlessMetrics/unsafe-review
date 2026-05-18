# UNSAFE-REVIEW-SPEC-0013: Agent packets

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for agent packets.

## Behavior

Agent packets constrain LLMs with task, contract, missing evidence, allowed repairs, do-not-do list, verify commands, and stop conditions.

Packets are bounded work orders for human-supervised agents. They carry enough
context to repair or gather evidence for one card or obligation, but they do not
ask an agent to certify safety.

## Packet schema

A packet contains:

- `packet_id`: stable digest over card ID, obligation ID when present, and packet
  kind.
- `card_id`: canonical review-card identity.
- `obligation_id`: optional obligation targeted by this packet.
- `task`: one of `document_contract`, `add_guard`, `add_or_route_test`,
  `run_witness`, `explain_limitation`, or `manual_review`.
- `context`: operation, hazard, relevant source spans, and existing evidence.
- `missing_evidence`: precise lanes that need work.
- `allowed_repairs`: narrow list of permitted code or documentation changes.
- `do_not_do`: project-specific and universal forbidden actions.
- `verify_commands`: commands the agent or reviewer should run after changes.
- `stop_conditions`: when to stop and ask for human review instead of continuing.

## Safety rails

- Packets must include repository-relative paths only.
- Packets must not include secrets, unbounded source dumps, or unrelated files.
- Packets must not request broad rewrites, automatic unsafe removal, or generated
  tests unless a separate policy explicitly permits them.
- A packet may propose guard or documentation work, but the reviewer remains
  responsible for validating correctness.
- When evidence is statically unknown, the packet must preserve that uncertainty
  and ask for investigation rather than asserting a fix.

## Packet kinds

- `document_contract`: add or clarify `# Safety` contract language for public
  unsafe APIs.
- `add_guard`: add a concrete check or wrapper that discharges a named obligation.
- `add_or_route_test`: connect existing tests or add narrowly scoped test reach
  when project policy allows tests.
- `run_witness`: run or import a dynamic/formal witness and attach the receipt.
- `explain_limitation`: document why a static lane remains unknown.
- `manual_review`: collect context when the tool cannot suggest a safe bounded
  change.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no autonomous merge, approval, or certification workflow
- no prompt that instructs an agent to ignore project policy or tests

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- packet schema tests for each packet kind and redaction tests for unrelated
  context

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- A missing public unsafe `# Safety` contract produces a `document_contract`
  packet with the item span and verify commands.
- A missing bounds guard produces an `add_guard` packet that names bounds evidence
  only and does not imply alignment was fixed.
- A generated-code card either omits repair packets or includes policy-limited
  manual-review instructions.

## Implementation backlog

1. Stabilize packet DTOs and JSON schema fixtures.
2. Generate packet kinds from obligation-level missing evidence.
3. Add redaction and path-normalization tests.
4. Expose packet rendering through CLI and LSP commands.
5. Add project-policy hooks for allowed repair classes.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
