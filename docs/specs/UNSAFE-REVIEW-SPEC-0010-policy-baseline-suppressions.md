# UNSAFE-REVIEW-SPEC-0010: Policy, baseline, suppressions

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for policy, baseline, suppressions.
Without that contract the tool cannot distinguish new debt from previously accepted debt,
and CI users cannot opt into enforcement without surprising developers.

## Behavior

Use exact counted identity for baselines and suppressions. Default advisory, then no-new-debt, then calibrated blocking.

Policy evaluation is a projection over review cards, not a second analyzer. The analyzer produces
cards first; policy then annotates each card with one of these states:

- `new`: the card is not present in the active baseline or suppression set.
- `baseline`: the card identity is present in the active baseline and has remaining count budget.
- `suppressed`: the card identity is covered by an explicit suppression entry.
- `expired_suppression`: the card matched a suppression whose expiry, owner, or reason is invalid.
- `policy_error`: policy files could not be parsed or contained unsupported semantics.

Policy mode determines the process result:

| Mode | Default | Failing condition |
| --- | --- | --- |
| `advisory` | yes | never fails only because cards exist |
| `no-new-debt` | no | any actionable `new` card exists |
| `blocking` | no | any actionable non-suppressed card exceeds configured severity or class thresholds |

## Inputs

Implementations must read policy inputs from the repository root unless explicitly overridden:

- `policy/unsafe-review.toml`: mode, thresholds, and enabled policy files.
- `policy/unsafe-review-baseline.toml`: counted card identities accepted as existing debt.
- `policy/unsafe-review-suppressions.toml`: temporary, reasoned exceptions.

Baseline entries must be keyed by the stable card identity and include at least `count`, `first_seen`,
`reason`, and `owner`. Suppression entries must include `id` or scoped matcher, `reason`, `owner`,
`expires`, and an optional `replacement_action`.

## Identity and matching

Card identity matching must be deterministic and explainable:

1. Exact `card.id` match is preferred.
2. If a future migration changes identity format, the previous identity may be listed as an alias.
3. Counts are decremented per matched emitted card; excess cards with the same identity are `new`.
4. Suppressions must not hide parse errors, policy errors, or cards with unknown identity.
5. Every non-`new` match must be visible in JSON, Markdown, human output, and CI summaries.

## Output contract

Policy-aware output must add a policy section to the top-level summary and per-card annotations:

- active mode and policy file paths used;
- counts for `new`, `baseline`, `suppressed`, `expired_suppression`, and `policy_error`;
- per-card policy state, matched policy entry id, owner, reason, and expiry when present;
- exit decision and the reason for that decision.

No output format may silently drop suppressed or baseline cards unless the user explicitly requests a
filtered view; the default view must preserve auditability.

## Implementation still required

- Parse the three policy TOML files into a typed policy model.
- Wire policy evaluation into `AnalyzeOutput` after card generation.
- Add CLI flags for policy mode and policy file overrides.
- Implement counted baseline matching with duplicate-card accounting.
- Implement suppression validation, expiry checks, and owner/reason requirements.
- Make process exit codes depend on policy mode.
- Render policy annotations in human, JSON, Markdown, and later SARIF/GitHub outputs.
- Add fixture coverage for advisory, no-new-debt, blocking, duplicate baseline counts,
  expired suppressions, malformed policy files, and explicit opt-out behavior.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no broad glob suppressions that cannot be audited from emitted output

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- human output smoke coverage
- policy documentation when behavior is configurable
- exit-code tests for each policy mode

## Acceptance examples

- A changed unsafe seam produces one review card with stable identity.
- The card includes missing evidence and a next action.
- If evidence is not knowable statically, the card names the limitation instead of overclaiming.
- In advisory mode, a new actionable card is reported but exits successfully.
- In no-new-debt mode, the same new actionable card exits non-zero.
- If a baseline entry has `count = 1` and two matching cards are emitted, one card is baseline and
  one card is new.
- An expired suppression is reported as `expired_suppression` and does not hide the card.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
