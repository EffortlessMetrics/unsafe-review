# 2026-06-01 source repo dogfood batch sync

Status: source-to-swarm history sync

This handoff records source PR #508 and advances the swarm source-sync
checkpoint after source imported the repo dogfood usability batch with history
preserved. This is not a release, analyzer expansion, Bun lane, or policy-gate
promotion.

Source PR and commits:

| Source PR / commit | Source commit | Surface | Swarm status |
|---|---|---|---|
| `EffortlessMetrics/unsafe-review#508` | `b399ece` | Imported the reviewed swarm repo-dogfood usability batch through `edbb6da`, including repo-mode dogfood usability, manual candidates, related sink clustering, and advisory JS getter reentry detection | Acknowledged by this sync as source history imported from swarm |

Swarm sync:

- `unsafe-review-swarm` merges source main `b399ece06275b1789a229e93578558f7b018472c`.
- `policy/source-sync.toml` acknowledges source main at
  `b399ece06275b1789a229e93578558f7b018472c`.
- Existing swarm-only work remains workbench state until deliberately promoted.

Boundaries:

- no release publication
- no analyzer breadth beyond the source-imported reviewed batch
- no Bun finding or vulnerability claim
- no witness execution
- no automatic comments
- no source edits by `unsafe-review`
- no default blocking policy
- no safety, UB-free, Miri-clean, site-execution, precision, recall, or
  policy-readiness claim

Validation:

- `cargo fmt --check`
- `cargo run --locked -p xtask -- check-calibration`
- `cargo run --locked -p xtask -- check-dogfood`
- `cargo run --locked -p xtask -- check-doc-artifacts`
- `cargo run --locked -p xtask -- check-goals`
- `cargo run --locked -p xtask -- check-pr`
- `git diff --check`

Expected after merge:

- `cargo run --locked -p xtask -- source-divergence` reports
  `new_source_commits=0`.
