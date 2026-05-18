# UNSAFE-REVIEW-SPEC-0013: Agent packets

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

Agent packets are intended to give LLMs enough context to repair review evidence
without broad, unsafe, or unrelated edits. The packet schema and safety rails still
need implementation detail before packets are exposed through CLI, PR, and LSP
surfaces.

## Behavior

An agent packet is a bounded task derived from exactly one review card unless policy
explicitly groups cards. It must describe the missing review evidence, allowed repair
scope, prohibited actions, and verification commands. It should make the smallest
credible repair easy and make risky behavior explicit.

## Packet data contract

Every packet must include:

- `packet_id`: stable identifier derived from the card id and packet version.
- `card_id`: review-card id being repaired.
- `task`: concise repair objective.
- `repository_scope`: repository root, allowed paths, and changed files.
- `unsafe_context`: operation, hazard class, obligations, snippet, and location.
- `present_evidence`: contract, discharge, reach, receipt, and policy evidence already
  found.
- `missing_evidence`: obligations or witness states that still need work.
- `allowed_repairs`: enumerated repair families such as add safety docs, add guard,
  add targeted test, add witness route, or update policy.
- `do_not_do`: prohibited edits such as widening unsafe, deleting checks, muting lints,
  broad suppressions, or claiming proof without receipt.
- `verify_commands`: commands required before returning work.
- `stop_conditions`: when to stop and ask for human review.
- `output_contract`: required final response fields for the agent.

Packet JSON must be serde-backed and versioned.

## Allowed repair families

- Add or refine `# Safety` documentation for public unsafe APIs.
- Add or refine local `SAFETY:` comments for unsafe blocks.
- Add explicit guards that discharge an obligation already implied by code.
- Add targeted tests that reach the safe wrapper or unsafe seam.
- Add witness routing configuration or import a receipt.
- Add a narrow suppression only when policy requires owner, reason, and expiry.

## Prohibited actions

Packets must instruct agents not to:

- remove unsafe code solely to silence the card unless the user asked for a refactor
- broaden unsafe scope or relax preconditions
- delete tests, guards, asserts, or safety comments
- add blanket allow attributes or broad suppressions
- claim UB freedom or memory-safety proof
- modify generated, vendored, or out-of-scope files
- ignore failing verification commands

## Grouping rules

Default packet generation is one packet per card. Grouping is allowed only when cards
share the same file, operation family, and repair objective. Grouped packets must keep
per-card missing evidence and verification commands visible.

## Non-goals

- no autonomous merge or deploy workflow
- no automatic source edits by unsafe-review itself
- no packet that hides policy or witness limitations
- no use of agent output as receipt evidence unless separately verified
- no duplicate truth outside review cards and policy decisions

## Required evidence

- JSON schema or DTO tests for packet serialization
- golden packet fixture for a missing safety contract
- golden packet fixture for a missing guard and witness route
- tests that prohibited actions and stop conditions are always present
- CLI, Markdown, and LSP projection smoke coverage for packet ids

## Acceptance examples

- A public unsafe function missing `# Safety` docs produces a packet whose primary
  allowed repair is documentation and whose verify commands include the repository
  checks.
- A raw-pointer card missing an alignment guard produces a packet that names the
  obligation, shows the snippet, prohibits deleting the unsafe block, and asks for a
  targeted test or witness receipt when appropriate.
- A card in an excluded generated file does not produce an edit packet; it produces a
  stop condition explaining the file policy.
- A grouped packet preserves separate card ids and missing evidence for each card.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Agent packets can move from planned to experimental only after packet DTOs, golden
fixtures, and mandatory safety-rail tests exist.
