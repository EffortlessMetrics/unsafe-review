# UNSAFE-REVIEW-SPEC-0014: Repo inventory and badges

Status: accepted
Owner: core/spec
Created: 2026-05-17
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

`unsafe-review` needs a precise, checkable behavior contract for repo inventory and badges.

## Behavior

Repo mode is a static posture snapshot projected from `ReviewCard`s. It reports
repo-scope summary counts, card JSON, advisory policy, and the static-review
trust boundary.

Badge JSON is a small open-gap summary for shields-compatible consumers:

- `unsafe-review.json` reports `<n> open gaps`
- `unsafe-review-plus.json` reports contract, guard, and current
  guarded-unwitnessed summary counts

Badges count unresolved review evidence. They never claim the repository is
safe, UB-free, Miri-clean, or policy-compliant.

Baseline-known items, suppressions, and no-new-debt deltas are planned later
policy surfaces and are not part of the current badge proof.

## Non-goals

- no soundness claim
- no hidden blocking unless policy mode explicitly enables it
- no duplicate truth outside this spec and linked policy files
- no safety badge
- no baseline, suppression, or no-new-debt policy in the badge JSON

## Required evidence

- fixture-backed examples for positive and negative cases
- JSON output contract coverage
- CLI e2e coverage for repo JSON and badge JSON
- policy documentation when behavior is configurable

## Acceptance examples

- Repo JSON for a fixture reports `scope = repo`, advisory policy, open-gap
  counts, cards, and the trust boundary.
- Badge JSON for a fixture reports open unsafe-review gaps rather than raw
  unsafe count or safe/unsafe status.
- If evidence is not knowable statically, repo output and badges count the
  card state instead of overclaiming.

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
cargo test -p unsafe-review --test e2e repo_inventory_and_badges_count_open_gaps_without_safety_claim
```

## Promotion rule

Move from experimental to usable alpha only after fixture, golden, and dogfood receipts exist.
