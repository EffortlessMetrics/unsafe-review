# Advisory PR artifacts v0.2 handoff

Date: 2026-05-18
Status: closed as experimental advisory artifact loop
Owner: CLI/CI/product

## What landed

The PR/CI projection lane now projects existing `ReviewCard`s into advisory PR
artifacts without creating new analyzer truth or policy authority.

The lane now has:

- local PR summary Markdown output
- local SARIF output
- advisory GitHub workflow artifact upload
- artifact-only inline comment plan JSON
- scanner false-positive hardening for comments and string literals
- fixture validation in `xtask check-pr`
- focused classifier, evidence, and diff parser unit coverage
- CLI e2e coverage for JSON, PR summary, SARIF, comment plan, context, and
  explain outputs
- raw pointer assignment-write fixture coverage
- minimal CI hardening for locked checks, docs build, read-only checkout
  credentials, PR-run cancellation, manual dispatch, and job timeout

The workflow uploads:

```text
target/unsafe-review/cards.json
target/unsafe-review/pr-summary.md
target/unsafe-review/cards.sarif
target/unsafe-review/comment-plan.json
```

## Proof

The merged lane was validated with the recurring workspace gates:

```bash
rtk cargo fmt --check
rtk cargo check --workspace --all-targets --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --all-targets --locked
rtk cargo run --locked -p xtask -- check-pr
```

Targeted proof added during the lane includes:

```bash
rtk cargo xtask check-fixtures
rtk cargo test -p unsafe-review-core fixture_card_goldens_match_rendered_json
rtk cargo test -p unsafe-review-core raw_pointer_v1_operation_cards_are_concrete
rtk cargo test -p unsafe-review-core pr_summary
rtk cargo test -p unsafe-review-core sarif
rtk cargo test -p unsafe-review-core comment_plan
rtk cargo test -p unsafe-review --test e2e
```

The CI workflow now also runs a docs build:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
```

## Current support posture

The PR artifact surfaces are experimental and advisory. They are suitable for
dogfood on real PRs, not for release-grade policy gating.

The repo may claim:

- PR artifacts are projected from existing `ReviewCard`s
- the advisory workflow uploads cards JSON, PR summary Markdown, SARIF, and
  comment-plan JSON
- SARIF is a code-scanning-compatible projection of static review evidence
- the comment plan is artifact-only and does not post comments
- CI and advisory workflows remain read-only and non-blocking

The repo must not claim:

- memory-safety proof
- UB-free status
- Miri, sanitizer, Loom, or Kani success without imported receipts
- automatic PR comments
- default blocking or branch-protection policy
- that SARIF upload means code-scanning approval or safety proof

## Known limits

- Artifacts are only as good as the underlying `ReviewCard`s.
- False positives and false negatives remain possible.
- Docs-only PR quieting depends on the advisory workflow path filters and should
  be watched during dogfood.
- Comment-plan JSON is not a posting policy.
- SARIF is advisory and should not be interpreted as a gate.
- No witness tools run by default.
- No receipt import, baseline, suppression, repo badge, LSP, or agent packet
  surface is part of this lane.

## Dogfood path

Use real PRs to inspect artifact usefulness before adding more product surface:

1. Open or update a PR that changes unsafe-adjacent Rust.
2. Confirm the advisory workflow uploads all four artifacts.
3. Read `pr-summary.md` first for reviewer guidance.
4. Inspect `cards.json` for the canonical card data.
5. Inspect `cards.sarif` for code-scanning projection shape.
6. Inspect `comment-plan.json` for proposed inline comments, without posting
   them.
7. Record noisy cards, missing cards, and unclear wording as fixture or renderer
   follow-ups.

## Next lane

The next durable lane should be dogfood-driven stabilization, not new product
surface. Prefer narrow PRs that reduce observed artifact noise or ambiguity.

Defer these until dogfood evidence justifies them:

- LSP projection
- agent packets
- baseline or no-new-debt policy
- repo badges
- witness receipt import
- fuzz or mutation workflows
- blocking CI policy
