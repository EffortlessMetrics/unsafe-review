# Repo posture and policy v0.4 receipt

Date: 2026-05-18
Status: initial experimental posture and policy slice landed
Owner: CLI/core/policy

## What landed

The first repo posture and policy slices now use existing `ReviewCard` identity
and summary data without adding broad policy authority.

Merged PRs:

- `#133 repo: prove inventory badge posture output`
- `#134 policy: validate ledger entry shape`
- `#135 policy: match exact advisory ledgers`
- `#136 policy: add explicit no-new-debt mode`

The repo posture surface includes:

- `unsafe-review repo --format json`
- `unsafe-review badges --out <dir>`

The policy surface includes:

- TOML shape validation for `policy/unsafe-review-baseline.toml`
- TOML shape validation for `policy/unsafe-review-suppressions.toml`
- exact counted `card_id` matching for `baseline_known` and `suppressed`
  classifications
- explicit opt-in `--policy no-new-debt`

## Proof

The merged PRs passed the hosted Rust workspace, advisory workflow,
CodeRabbit, and GitGuardian checks before merge.

Targeted local validation added during this slice included:

```bash
rtk cargo test -p unsafe-review --test e2e repo_inventory_and_badges_count_open_gaps_without_safety_claim --locked
rtk cargo test -p xtask ledger --locked
rtk cargo test -p unsafe-review-core baseline_policy --locked
rtk cargo test -p unsafe-review-core suppression_policy --locked
rtk cargo test -p unsafe-review-core policy_state --locked
rtk cargo test -p unsafe-review-cli no_new_debt --locked
rtk cargo test -p unsafe-review --test e2e no_new_debt_policy_fails_only_for_unbaselined_actionable_gaps --locked
```

The recurring workspace gate also passed for each code PR:

```bash
rtk cargo fmt --check
rtk cargo check --workspace --all-targets --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo run --locked -p xtask -- check-pr
rtk git diff --check
```

## Current support posture

These surfaces are experimental.

The repo may claim:

- repo JSON reports static open unsafe-review gaps from `ReviewCard`s
- badge JSON reports open review gaps, not raw unsafe count
- baseline and suppression ledgers require exact counted card identity plus
  owner, reason, evidence, and review or expiry dates
- exact baseline matches become `baseline_known`
- exact suppression matches become `suppressed`
- explicit `--policy no-new-debt` exits nonzero when unbaselined actionable gaps
  remain

The repo must not claim:

- memory-safety proof
- UB-free status
- Miri, sanitizer, Loom, Kani, or Crux success without receipts
- broad baseline or suppression patterns
- default no-new-debt behavior
- calibrated blocking policy
- branch-protection readiness

## Known limits

- Matching is exact `card_id` only.
- Line-stable identity exists, but broader drift behavior still needs dogfood.
- Suppression and baseline ledgers do not support glob, owner, path, class, or
  operation-family patterns.
- No outcome comparison exists yet.
- No witness receipt import exists yet.
- No calibrated blocking policy exists yet.

## Next useful work

Prefer dogfood before expanding policy authority:

- run `--policy no-new-debt` against real unsafe-review PRs and inspect noise
- record cases where exact identity is too brittle or too permissive
- add outcome comparison before making repo posture more dashboard-like
- add witness receipt import before claiming witness-backed resolution

Defer:

- default no-new-debt
- blocking branch protection
- broad suppressions
- release-grade safety badges
- automatic witness execution
