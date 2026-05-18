# UNSAFE-REVIEW-SPEC-0011: PR and CI output

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for pr and ci output.

The current implementation can render human, JSON, and Markdown output. CI still
needs stable artifacts, SARIF, GitHub summary text, inline-comment selection, and
policy-aware exit behavior.

## Implementation status

Partially implemented. Human, JSON, and Markdown renderers exist. SARIF, GitHub
summary rendering, inline comment selection, and no-new-debt enforcement are still
planned.

## Behavior

PR output must be sparse: summary first, at most a few high-confidence inline comments, durable JSON/Markdown/SARIF artifacts.

### Artifact set

A CI run must be able to emit the following files in a caller-selected output
folder:

- `unsafe-review.cards.json`: canonical review-card output;
- `unsafe-review.summary.md`: human-readable PR summary;
- `unsafe-review.sarif.json`: SARIF 2.1.0 results for code-scanning consumers;
- `unsafe-review.policy.json`: policy decisions, exit intent, stale suppressions,
  and baseline drift;
- `unsafe-review.witness-plan.md`: suggested witness commands grouped by route.

The JSON card artifact remains the canonical truth. Markdown, SARIF, and GitHub
summary output must be projections from the same analyzed output and policy
decisions.

### GitHub summary

The GitHub summary must include, in order:

1. policy result and selected mode;
2. changed unsafe seam count;
3. open actionable gap count;
4. top hazards by severity;
5. witness routes to run next;
6. links or paths to durable artifacts;
7. limitations statement that the tool is static review evidence, not proof.

The summary must avoid dumping every card when the card count exceeds the
configured summary limit. Truncation must be explicit.

### Inline comments

Inline comments are optional and must be selected conservatively:

- only cards with exact changed-line mapping are eligible;
- at most one comment per changed unsafe seam;
- comments must include a card id, hazard, missing evidence, and next action;
- comments must not include speculative language beyond the card's evidence;
- duplicate comments from previous runs must be detectable by embedded card id.

### SARIF mapping

SARIF output must use SARIF 2.1.0 and map:

- card id to `result.fingerprints`;
- hazard id to `rule.id`;
- severity to SARIF level;
- source span to `physicalLocation`;
- missing obligations to `message.text`;
- witness routes to `help.markdown` or `properties`.

SARIF must not claim dynamic execution, memory-safety proof, or exploitability.

### Exit behavior

CI exit status follows the policy engine in SPEC-0010. Renderer failures are
command failures even when policy would otherwise pass.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no mandatory GitHub dependency for local CI
- no automatic code modification

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- SARIF schema validation against representative cards
- GitHub summary golden tests for small and truncated outputs
- inline-comment selection tests for exact and inexact line mappings

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity and one
  SARIF result with the same fingerprint.
- A PR with twenty cards renders a bounded summary and points to the full JSON
  artifact.
- A card without exact changed-line mapping appears in artifacts but does not
  produce an inline comment.
- If evidence is not knowable statically, every renderer names the limitation
  instead of overclaiming.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
