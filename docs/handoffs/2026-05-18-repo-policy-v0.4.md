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
- `#170 repo: add saved snapshot outcome comparison`

The repo posture surface includes:

- `unsafe-review repo --format json`
- `unsafe-review badges --out <dir>`
- `unsafe-review outcome --before <cards.json> --after <cards.json>`

The policy surface includes:

- TOML shape validation for `policy/unsafe-review-baseline.toml`
- TOML shape validation for `policy/unsafe-review-suppressions.toml`
- exact counted `card_id` matching for `baseline_known` and `suppressed`
  classifications
- explicit opt-in `--policy no-new-debt`
- saved-snapshot outcome comparison for new, resolved, improved, regressed, and
  unchanged card identities

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
rtk cargo test -p unsafe-review-core outcome --locked
rtk cargo test -p unsafe-review-cli outcome --locked
rtk cargo test -p unsafe-review --test e2e outcome_compares_existing_json_snapshots_without_safety_claim --locked
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
- outcome comparison reports new, resolved, improved, regressed, and unchanged
  card identities from two existing unsafe-review JSON snapshots

The repo must not claim:

- memory-safety proof
- UB-free status
- Miri, sanitizer, Loom, Kani, or Crux success without receipts
- broad baseline or suppression patterns
- default no-new-debt behavior
- calibrated blocking policy
- branch-protection readiness
- outcome comparison that reruns analysis, executes witnesses, or makes policy
  decisions

## Subsequent status

This handoff records the repo posture and policy slice plus the later
saved-snapshot outcome follow-up. A later witness lane has also landed
exact-card witness receipt import and witness-plan output. The "Known limits"
below describe what remains outside the current repo posture proof.

For current posture, read this handoff together with:

- `docs/handoffs/2026-05-18-witness-receipt-import-v0.5.md`
- `docs/status/SUPPORT_TIERS.md`

## Subsequent dogfood

Outcome comparison was dogfooded on capped `memchr` repo snapshots from the
real-crate dogfood lane:

```bash
rtk cargo run --locked -p unsafe-review -- outcome --before target/dogfood-work/memchr.unsafe-review.after-slice-end-pointer-evidence.json --after target/dogfood-work/memchr.unsafe-review.after-target-feature-contract-evidence.json --format json --out target/dogfood-work/memchr.outcome.after-target-feature-contract-evidence.json
rtk cargo run --locked -p unsafe-review -- outcome --before target/dogfood-work/memchr.unsafe-review.after-slice-end-pointer-evidence.json --after target/dogfood-work/memchr.unsafe-review.after-target-feature-contract-evidence.json --format markdown --out target/dogfood-work/memchr.outcome.after-target-feature-contract-evidence.md
```

The saved-snapshot comparison reported:

```text
new: 0
resolved: 0
improved: 10
regressed: 0
unchanged: 40
```

Those 10 improved cards were the documented target-feature declaration cards
that moved from `guard_missing` to `guarded_unwitnessed`. The comparison did
not rerun analysis, execute witnesses, or make a policy decision. It preserved
the outcome trust boundary: static unsafe contract review outcome only, not
memory-safety proof, not UB-free status, and not witness execution.

## Known limits

- Matching is exact `card_id` only.
- Line-stable identity exists, but broader drift behavior still needs dogfood.
- Suppression and baseline ledgers do not support glob, owner, path, class, or
  operation-family patterns.
- Outcome comparison reads saved snapshots only; the first capped `memchr`
  snapshot dogfood is recorded, but more repos and PRs are needed before making
  it more dashboard-like. It does not rerun analysis, execute witnesses, or make
  policy decisions.
- No calibrated blocking policy exists yet.

## Next useful work

Prefer dogfood before expanding policy authority:

- run `--policy no-new-debt` against real unsafe-review PRs and inspect noise
- record cases where exact identity is too brittle or too permissive
- dogfood outcome comparison on more real repo snapshots and PR snapshots before
  making repo posture more dashboard-like
- keep witness receipt import separate from policy promotion

Defer:

- default no-new-debt
- blocking branch protection
- broad suppressions
- release-grade safety badges
- automatic witness execution
