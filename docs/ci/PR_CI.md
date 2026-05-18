# PR and CI model

Default PR runs cheap static review:

```text
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
unsafe-review check --base origin/main --format json
unsafe-review check --base origin/main \
  --format pr-summary \
  --out target/unsafe-review/pr-summary.md
```

The PR summary artifact is Markdown for GitHub job summaries or uploaded
artifacts. It projects existing review cards only: counts, top card, card table,
witness plan, and the trust boundary. It must not add PR-specific analyzer truth
and must not imply a blocking policy.

Witness tools are routed, not run everywhere. Miri, sanitizers, Loom, and Kani
belong in targeted PR, nightly, or release lanes unless repo policy says
otherwise.
