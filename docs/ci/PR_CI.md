# PR and CI model

Default PR runs cheap static review on the pinned Rust toolchain:

```text
cargo fmt --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo run --locked -p xtask -- check-pr
unsafe-review first-pr --base origin/main --out-dir target/unsafe-review
```

The CI workflow keeps repository permissions read-only, avoids persisted checkout
credentials, cancels superseded pull request runs, supports manual dispatch for
ad hoc verification, and bounds the Rust job with a timeout.
Dependabot opens weekly Cargo and GitHub Actions update PRs as maintenance
signals; those PRs still pass through the same advisory CI and review process.
The `dtolnay/rust-toolchain` action ref is intentionally pinned to the repo
toolchain version and is not Dependabot-managed.

The PR summary artifact is Markdown for GitHub job summaries or uploaded
artifacts. It projects existing review cards only: counts, top card, card table,
witness plan, and the trust boundary. It must not add PR-specific analyzer truth
and must not imply a blocking policy.

The SARIF artifact projects the same review cards into code-scanning shape. It
is still advisory static review evidence; uploading SARIF must not be treated as
proof that the changed code is memory-safe.

The advisory GitHub workflow writes and uploads:

```text
target/unsafe-review/cards.json
target/unsafe-review/pr-summary.md
target/unsafe-review/github-summary.md
target/unsafe-review/cards.sarif
target/unsafe-review/comment-plan.json
target/unsafe-review/witness-plan.md
target/unsafe-review/receipt-audit.md
target/unsafe-review/lsp.json
target/unsafe-review/repair-queue.json
```

Before upload, the workflow runs:

```text
cargo run --locked -p xtask -- check-advisory-artifacts target/unsafe-review
```

The comment plan is an artifact of candidate high-signal inline comments. It is
not posted by the workflow.

The PR gate fails on infrastructure and contract failures, not on advisory
findings by default:

- tool invocation failed,
- required artifact missing,
- machine-readable artifact malformed,
- card IDs inconsistent across projections,
- trust boundary missing,
- output contains positive safety/proof wording,
- comment plan violates its artifact contract.

The comment-plan contract is intentionally narrow:

- at most three planned comments,
- changed lines only,
- high-confidence actionable cards only,
- no `static_unknown`, baseline-known, or suppressed cards,
- no posting by default.

A future trusted poster must consume `comment-plan.json` and keep the same
ReviewCard identity, witness route, verify-command, and trust-boundary fields.
It must not rerun analysis and create a second comment truth.

The workflow does not run Miri, sanitizers, Loom, Kani, or other witness tools.
It does not post comments and does not enable blocking policy.

After downloading or rendering an advisory artifact set, verify the artifact
contract with:

```text
cargo xtask check-advisory-artifacts target/unsafe-review
```

This checks that the first-pr review kit artifacts exist, machine-readable
artifacts parse, the policy remains advisory, the comment plan remains
plan-only, the receipt audit stays metadata-only, repair queue boundaries are
present, projected card IDs match `cards.json`, result counts stay consistent,
and the trust boundary is present.

Witness tools are routed, not run everywhere. Miri, sanitizers, Loom, and Kani
belong in targeted PR, nightly, or release lanes unless repo policy says
otherwise.
