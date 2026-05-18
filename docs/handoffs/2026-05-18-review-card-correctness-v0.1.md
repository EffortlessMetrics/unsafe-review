# Review-card correctness v0.1 handoff

Date: 2026-05-18
Status: closed as fixture-backed experimental card engine
Owner: core/product

## What landed

The review-card correctness lane moved `unsafe-review` from scaffold to a
fixture-backed analyzer core. The canonical product object remains
`ReviewCard`; downstream PR, LSP, agent, badge, baseline, and receipt surfaces
must continue to project from that object instead of creating parallel truth.

The lane now has:

- serde-backed review-card JSON with `schema_version`
- fixture golden rendering for card JSON
- obligation-level contract, discharge, reach, and witness evidence
- stable-first syntax extraction through `ra_ap_syntax`
- unsafe-block and split unsafe-block site extraction
- raw pointer read and deref cards, including split-call spelling
- public unsafe fn and trait contract evidence fixtures
- core operation smoke fixtures for `MaybeUninit::assume_init`,
  `Vec::set_len`, `transmute`, `get_unchecked_mut`, and `Pin::new_unchecked`
- FFI, unsafe impl Send, and route-table witness routing fixtures
- explanation and reach wording guardrails
- exact counted review-card identity without line-number churn
- support-tier language that maps card claims to fixture proof

## Proof

The merged lane was validated with the recurring workspace gate:

```bash
rtk cargo fmt --check
rtk cargo check --workspace --all-targets
rtk cargo clippy --workspace --all-targets -- -D warnings
rtk cargo test --workspace
rtk cargo xtask check-pr
```

Targeted proof added during the lane includes:

```bash
rtk cargo test -p unsafe-review-core fixture_card_goldens_match_rendered_json
rtk cargo test -p unsafe-review-core card_identity
rtk cargo test -p unsafe-review-core reach_wording
rtk cargo test -p unsafe-review-core witness_routing
```

## Current support posture

The local review-card surfaces are fixture-backed and experimental. This is not
a usable-alpha or release claim. The current support-tier table is the authority
for exact claim wording.

The repo may claim:

- review-card JSON is serde-backed and fixture-golden tested
- selected raw pointer, public unsafe API, and core operation slices are
  fixture-backed
- witness routes are recommendations, not receipts
- card identity is stable across line drift and counted for duplicate sites

The repo must not claim:

- memory-safety proof
- UB-free status
- Miri success without an imported receipt
- site execution or test coverage without a receipt
- default blocking policy
- broad operation-family support beyond fixture-backed slices

## Known limits

- Evidence remains source-level and heuristic.
- Fixture coverage is curated and still small.
- Reach evidence is static wording only, not execution proof.
- Witness routing explains recommendations but does not import receipts.
- Baseline and suppression policy do not consume exact card identity yet.
- PR/SARIF, LSP, agent packets, repo badges, and no-new-debt policy are planned
  projections, not completed product surfaces.

## Next lane

The next durable lane should be PR/CI projection after this handoff:

- GitHub Markdown summary artifact
- SARIF output
- witness plan artifact
- advisory workflow only
- inline comment planner as summary-only or artifact-only by default

Keep the same trust boundary in that lane: project existing review cards, do not
invent new analyzer truth in PR-specific code.
