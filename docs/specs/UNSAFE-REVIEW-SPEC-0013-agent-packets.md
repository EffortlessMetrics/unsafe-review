# UNSAFE-REVIEW-SPEC-0013: Agent packets

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for agent packets.
The current context output must become a hardened, least-authority instruction packet that helps an
agent repair evidence gaps without broad, unsafe rewrites or unverifiable claims.

## Behavior

Agent packets constrain LLMs with task, contract, missing evidence, allowed repairs, do-not-do list, verify commands, and stop conditions.

Packets are derived from review cards and must be reproducible from canonical JSON. They are guidance
for human-reviewed agent work, not authorization to change code. Every packet must preserve the card
id, site, obligations, missing evidence, and trust boundary.

## Packet schema

An agent packet must include:

- schema version and card id;
- repository root hint and relative file paths only;
- unsafe site snippet with bounded context;
- hazard and obligation summary;
- current contract, discharge, reachability, witness, policy, and receipt state;
- missing evidence grouped by obligation;
- allowed repair categories;
- disallowed actions;
- verification commands and expected artifacts;
- stop conditions and escalation conditions.

Packets must be serializable as JSON and renderable as Markdown. JSON is canonical.

## Allowed and disallowed work

Allowed repair categories should be narrow and explicit, for example:

- add or clarify a `# Safety` contract;
- add a local guard or assertion that discharges a listed obligation;
- add or route a focused test to a relevant witness;
- add a receipt reference when the witness has already run;
- split broad unsafe blocks only when it improves reviewability without changing behavior.

Disallowed actions must include:

- deleting unsafe code only to silence the card;
- broad refactors unrelated to the card;
- weakening public API contracts;
- adding blanket suppressions;
- claiming soundness or UB freedom from static evidence;
- changing generated or vendored files unless the card explicitly targets them.

## Context bounds

Packets must be small enough to review:

- include bounded source context around the primary site;
- include only related tests and witness commands relevant to the card;
- include policy and receipt excerpts only when they affect the card;
- redact or omit absolute paths, environment variables, and secrets;
- state when context was truncated.

## Implementation still required

- Formalize the packet JSON DTO and schema version.
- Replace ad hoc context rendering with serde-backed JSON plus Markdown projection.
- Add context bounding and truncation notices.
- Add allowed/disallowed repair templates by hazard and obligation family.
- Add stop-condition text for inconclusive static analysis, missing tests, and witness failures.
- Add packet golden tests for representative hazards and no-related-test cases.
- Ensure LSP copy-agent-packet and CLI context output use the same packet builder.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no autonomous merge or commit authority implied by a packet
- no secret scanning beyond refusing to include known sensitive process data

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- packet schema/golden coverage
- tests that packets preserve card id and missing evidence

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- A packet for a missing safety contract includes contract-focused allowed repairs and verification commands.
- A packet for an inconclusive witness route includes a stop condition instead of asking the agent to assert success.
- The CLI and LSP packet projections are byte-equivalent for the same saved card JSON.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
