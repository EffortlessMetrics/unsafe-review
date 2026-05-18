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

## Dogfood receipt

The advisory artifact loop has been checked against a real PR workflow artifact,
not only local renderer tests.

Receipt:

- PR: `#104 cli: harden diff input ergonomics`
- Workflow: `unsafe-review`
- Run: `26013330481`
- Branch: `cli/diff-input-ergonomics`
- Artifact: `unsafe-review`
- Artifact id: `7049477825`
- Artifact digest: `sha256:6368cbe9372a84ae23e8ad587ee75715026fd322bc2fcce692778673c700b10c`

Downloaded artifact contents:

```text
cards.json
cards.sarif
comment-plan.json
pr-summary.md
```

Verification command:

```bash
rtk cargo run --locked -p xtask -- check-advisory-artifacts target/advisory-artifact-104/unsafe-review
```

Result:

```text
check-advisory-artifacts: ok (target/advisory-artifact-104/unsafe-review)
```

This receipt proves the current advisory workflow produced and uploaded the four
expected artifacts for a real PR, and that the repo verifier accepted their
advisory policy, plan-only comment mode, projection card identity consistency,
result counts, parseability, and trust-boundary text. It does not prove
memory safety, witness execution, code-scanning approval, or policy readiness.

## Unsafe-adjacent dogfood follow-up

The `#102 analysis: distinguish unaligned raw pointer reads` PR provided a
stronger unsafe-adjacent dogfood sample because it changed raw-pointer fixture
and scanner code.

Receipt:

- PR: `#102 analysis: distinguish unaligned raw pointer reads`
- Workflow: `unsafe-review`
- Run: `26012554821`
- Branch: `analysis/read-unaligned-card-v1`
- Artifact: `unsafe-review`
- Artifact id: `7049234818`
- Artifact digest: `sha256:afb743e2547cc06cf406f56e9a85f5e0620f8bf5561dd267dc7e62d5c28a0fae`

The downloaded artifact passed the artifact-contract verifier:

```bash
rtk cargo run --locked -p xtask -- check-advisory-artifacts target/advisory-artifact-102/unsafe-review
```

The artifact also exposed dogfood noise: it contained five cards, four of which
came from detector implementation strings in `scanner.rs`, such as
`line.contains("get_unchecked")`, rather than from unsafe operations. That
finding was fixed by `#107 fix: ignore syntax string literal detector text`.

After `#107`, rerunning the `#102` diff locally produced one card: the intended
`raw_pointer_read_unaligned` fixture card. This confirms the PR artifact loop is
useful as a noise-finding dogfood path, while the support tier should remain
experimental and advisory.

The `#107` advisory workflow artifact then confirmed the scanner-only fix no
longer generated unsafe-review noise:

- PR: `#107 fix: ignore syntax string literal detector text`
- Workflow: `unsafe-review`
- Run: `26013722196`
- Branch: `fix/syntax-string-literal-detection`
- Artifact: `unsafe-review`
- Artifact id: `7049611451`
- Artifact digest: `sha256:d4f0730666ea5f3a7a8a243b6d6957c037cefbddd3da802b8e2d2e9fcd4445c5`

Verification command:

```bash
rtk cargo run --locked -p xtask -- check-advisory-artifacts target/advisory-artifact-107/unsafe-review
```

Result:

```text
check-advisory-artifacts: ok (target/advisory-artifact-107/unsafe-review)
```

The downloaded `cards.json` summary for `#107` reported:

```text
changed_rust_files: 1
cards: 0
open_actionable_gaps: 0
```

## Docs-only quieting receipt

The advisory workflow is configured to skip docs-only pull requests:

```yaml
paths-ignore:
  - "docs/**"
  - "**/*.md"
```

The `#109 docs: record clean string-literal artifact dogfood` PR changed only:

```text
docs/handoffs/2026-05-18-advisory-pr-artifacts-v0.2.md
```

Its status rollup contained the CI, CodeRabbit, and GitGuardian checks, but no
`unsafe-review advisory` workflow check. A run-list query for the branch also
returned no `unsafe-review` workflow runs:

```bash
rtk gh run list --workflow unsafe-review --branch docs/string-literal-dogfood-receipt --json databaseId,headBranch,status,conclusion,event,displayTitle,url --limit 10
```

Result:

```json
[]
```

This receipt proves docs-only PRs can be quiet for the advisory artifact
workflow. It does not change the advisory workflow's manual `workflow_dispatch`
behavior, and it does not imply any policy gate.

## Post-closeout stabilization receipts

The first dogfood stabilization fixes after lane closeout stayed within the
advisory artifact loop: they reduced observed noise and made artifact commands
less error-prone without adding new analyzer truth, comments, witnesses, or
blocking policy.

### Repo-mode deref-assignment false positive

Repo-mode self-dogfood found a product-code card in
`crates/unsafe-review-core/src/analysis/pipeline.rs` from ordinary
mutable-reference code:

```text
*next += 1;
```

That line updates an identity counter. It is not a raw-pointer write seam. The
fix in `#111 fix: avoid text fallback deref write cards` removed bare
deref-assignment classification from text fallback detection while preserving
syntax-backed detection for real unsafe raw pointer assignments such as:

```text
unsafe { *ptr = value; }
```

Validation included:

```bash
rtk cargo test -p unsafe-review-core text_detection_does_not_classify_deref_assignments_as_writes --locked
rtk cargo test -p unsafe-review-core syntax_detection_classifies_unsafe_raw_pointer_assignments_as_writes --locked
rtk cargo test -p unsafe-review-core fixture_card_goldens_match_rendered_json --locked
rtk cargo run --quiet --locked -p unsafe-review -- repo --format json --out target/dogfood/repo-self-after-fix.json
```

The repo-mode result after the fix was:

```text
cards: 23
open_actionable_gaps: 23
product_code_cards: 0
fixture_cards: 23
```

### Equals-style artifact flags

Candidate PR `#46` contained useful CLI parser hardening but was stale and
overlapped with already-landed diff-input ergonomics. The current-lane slice was
rebuilt as `#112 cli: accept equals-style flag values`.

The merged behavior accepts artifact-oriented invocations such as:

```bash
rtk cargo run --quiet --locked -p unsafe-review -- check --root=fixtures/raw_pointer_alignment --diff=change.diff --format=sarif --out=target/dogfood/equals-flags/cards.sarif
```

It also rejects missing flag values such as `--diff --json` while preserving
`--diff -` for stdin diff input. This is CLI stabilization for the advisory
artifact path, not new policy authority.

The advisory workflow artifact for `#112` was then downloaded and verified:

- PR: `#112 cli: accept equals-style flag values`
- Workflow: `unsafe-review`
- Run: `26014327438`
- Branch: `cli/equals-style-flags`
- Artifact: `unsafe-review`
- Artifact id: `7049824978`
- Artifact digest: `sha256:d9743e8b29c709c356cc1c1d12a3a1368071561a6ccea20e173d6ce2dcf35a92`

Verification command:

```bash
rtk cargo run --locked -p xtask -- check-advisory-artifacts target/advisory-artifact-112
```

Result:

```text
check-advisory-artifacts: ok (target/advisory-artifact-112)
```

The downloaded `cards.json` summary reported:

```text
changed_rust_files: 2
cards: 0
open_actionable_gaps: 0
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

For downloaded or locally rendered artifacts, run:

```bash
rtk cargo xtask check-advisory-artifacts target/unsafe-review
```

This verifies the four-file artifact contract, advisory policy, plan-only
comment mode, JSON/SARIF parseability, projection card IDs, result counts, and
trust-boundary text.

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
