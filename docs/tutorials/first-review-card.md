# First review card

This tutorial walks through one local `unsafe-review` run using a bundled fixture.
It is the fastest way to learn what a review card is before wiring the tool into a
real pull request.

## 1. Build the workspace

From the repository root, build the CLI and libraries:

```bash
cargo check --workspace --all-targets
```

## 2. Run the fixture review

Run `unsafe-review check` against the raw-pointer alignment fixture and ask for
Markdown output:

```bash
cargo run -p unsafe-review -- check \
  --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff \
  --format markdown
```

The fixture contains a changed unsafe-adjacent operation. The command should emit a
small PR-style report instead of trying to prove the code sound.

## 3. Read the card as a reviewer

A card is useful when it answers four review questions:

1. **What changed?** The unsafe seam and operation under review.
2. **What can go wrong?** The hazard class and safety obligations.
3. **What evidence exists?** Nearby contract comments, local guards, tests, and
   witness routes.
4. **What is missing?** The smallest actionable gap for the PR author or reviewer.

Treat the card as a review checklist, not as a verdict. `unsafe-review` reports
review evidence; it does not claim the repository is memory safe.

## 4. Try JSON output

Machine consumers should use JSON:

```bash
cargo run -p unsafe-review -- check \
  --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff \
  --format json
```

Use this output when integrating with CI annotations, dashboards, editor tooling,
or agent packets.

## Next steps

- Use [Run unsafe-review on a pull request](../how-to/run-pr-review.md) for a real
  branch diff.
- Use [CLI reference](../reference/cli.md) for the complete command and flag list.
- Read [Review cards and trust boundary](../explanation/review-cards-and-trust-boundary.md)
  to understand what the tool does and does not prove.
