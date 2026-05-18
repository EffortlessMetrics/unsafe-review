# PR and CI model

Default PR runs cheap static review:

```text
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
unsafe-review check --base origin/main --format json --out target/unsafe-review/pr.json
```

Witness tools are routed, not run everywhere. Miri, sanitizers, Loom, and Kani
belong in targeted PR, nightly, or release lanes unless repo policy says
otherwise.

Keep this lane advisory until the repository has calibrated fixtures, baselines, and suppression policy. See the [CLI guide](../CLI.md) for pilot, repo inventory, badge, and card-explanation workflows.
