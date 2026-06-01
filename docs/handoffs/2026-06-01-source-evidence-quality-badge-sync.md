# 2026-06-01 source evidence-quality badge sync

Status: source-to-swarm public badge surface sync

This handoff records the source-only badge count adjustment from
`EffortlessMetrics/unsafe-review#512` and mirrors it into the swarm workbench so
routine swarm work is not based on stale public badge semantics.

Source PRs and commits:

| Source PR / commit | Source commit | Surface | Swarm status |
|---|---|---|---|
| `EffortlessMetrics/unsafe-review#512` | `bee8e234` | Changes `unsafe-review+` to report missing-or-weak evidence-quality findings without double-counting open actionable gaps | Mirrored by this sync as a source public-surface fix |

Swarm sync:

- `crates/unsafe-review-core/src/output/badges.rs` mirrors the source
  evidence-quality count semantics for `unsafe-review+`.
- `badges/unsafe-review-plus.json` mirrors the checked-in source badge JSON.
- `README.md`, `docs/BADGE_POLICY.md`, and badge-related specs mirror the
  source public-surface wording.
- `crates/unsafe-review/tests/e2e.rs` mirrors the source badge e2e assertion.
- `policy/source-sync.toml` acknowledges source main at
  `bee8e234d879316da3215d9ec2a42a5b02a8fbc2`.

Boundaries:

- no crates.io publication claim
- no tag or GitHub Release claim
- no analyzer behavior change claim beyond badge projection semantics
- no witness execution
- no automatic comments
- no source edits by `unsafe-review`
- no default blocking policy
- no safety, UB-free, Miri-clean, site-execution, precision, recall, or
  policy-readiness claim

Validation:

- `cargo fmt --check`
- `cargo test --locked -p unsafe-review-core badge`
- `cargo test --locked -p unsafe-review --test e2e repo_inventory_and_badges_count_open_gaps_without_safety_claim`
- `cargo test --locked -p xtask public_badge`
- `cargo run --locked -p unsafe-review -- badges --out badges/`
- `cargo run --locked -p xtask -- check-docs`
- `cargo run --locked -p xtask -- check-pr`
- `cargo run --locked -p xtask -- source-divergence`
- `git diff --check`

Expected after merge:

- `cargo run --locked -p xtask -- source-divergence` reports
  `new_source_commits=0`.
