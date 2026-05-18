# UNSAFE-REVIEW-SPEC-0013: Agent packets

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for agent packets.

The current context output is a scaffold. It still needs a hardened schema that
keeps coding agents focused on reviewable repairs and prevents them from treating
static findings as proof obligations they can silently waive.

## Implementation status

Partially implemented. CLI context output can render a packet-like JSON document
from a card. Schema validation, hardening, redaction, and packet-specific golden
tests are still planned.

## Behavior

Agent packets constrain LLMs with task, contract, missing evidence, allowed repairs, do-not-do list, verify commands, and stop conditions.

### Packet schema

Each packet must include:

- schema version;
- card id;
- repository-relative file path and span;
- unsafe operation and hazard;
- known evidence;
- missing obligations;
- suggested witness route;
- allowed repair classes;
- forbidden repair classes;
- verification commands;
- stop conditions;
- reviewer handoff checklist.

The packet must be self-contained enough for an agent to work on one card without
needing the full analyzer output, but it must not include unrelated repository
content.

### Allowed repairs

Allowed repair classes are advisory and may include:

- add or improve `# Safety` documentation for public unsafe APIs;
- add local guard checks before an unsafe operation;
- narrow an unsafe block;
- add targeted tests or witness commands;
- replace an unsafe operation with an equivalent safe API when the behavior is
  unchanged and reviewable.

### Forbidden repairs

Packets must tell agents not to:

- delete unsafe code just to remove a finding;
- add broad suppressions or baseline entries;
- weaken tests;
- change public behavior without calling it out;
- claim Miri, sanitizer, Loom, Kani, or Crux coverage unless a receipt proves it;
- invent safety invariants not supported by code, docs, or reviewer-provided
  context.

### Verification commands

Verification commands must be concrete and copyable. They may include `cargo
fmt`, `cargo test`, fixture-specific tests, witness commands, or project-specific
commands from policy. Commands that require unavailable tooling must be marked as
optional or environment-dependent.

### Stop conditions

The packet must require the agent to stop and ask for human review when:

- an invariant is ambiguous;
- a repair would change public API or semantics;
- a witness command fails for reasons unrelated to the edited code;
- the card points to generated or vendored code;
- the agent would need secrets, network credentials, or external services.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no autonomous merge approval
- no packet that spans unrelated unsafe sites in v1

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- JSON schema tests for packets
- redaction tests for absolute paths and environment-specific data
- golden tests for allowed/forbidden repair text
- round-trip tests from CLI context and LSP copy-packet action

## Acceptance examples

- A changed unsafe seam produces one agent packet with the same stable card id as
  the review card.
- The packet includes missing evidence and at least one concrete next action.
- The packet forbids broad suppression and unsupported proof claims.
- Environment-dependent witness commands are labeled as such instead of being
  presented as completed evidence.
- If evidence is not knowable statically, the packet names the limitation instead
  of overclaiming.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
