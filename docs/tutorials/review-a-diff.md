# Tutorial: review a supplied diff

This tutorial walks through one complete `unsafe-review` loop using a bundled
fixture. It is intentionally small: the goal is to understand what a review card
is, not to learn every CLI option.

## What you will learn

- how to run `unsafe-review check` on a known diff
- how to recognize the unsafe seam, hazard, obligations, evidence, and witness route
- how to read the result without treating it as a memory-safety proof

## Prerequisites

- a checked-out copy of this repository
- Rust toolchain from `rust-toolchain.toml`
- commands run from the repository root

## 1. Build the workspace

```bash
cargo check --workspace --all-targets
```

This verifies that the CLI, core analyzer, facade crate, and `xtask` compile in
the current checkout.

## 2. Run the raw-pointer fixture

```bash
cargo run -p unsafe-review -- check \
  --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff
```

The fixture contains a small unsafe-adjacent change. The command asks
`unsafe-review` to review only that supplied diff rather than your working tree.

## 3. Read the card as review evidence

Look for these parts in the output:

| Output part | Meaning |
|---|---|
| Seam location | the changed unsafe-adjacent source range under review |
| Hazard | the kind of unsafe risk, such as raw pointer validity or alignment |
| Obligations | conditions reviewers expect the code or contract to establish |
| Evidence | nearby `# Safety` or `SAFETY:` comments and simple local guards the analyzer found |
| Missing evidence | what the reviewer should ask for next |
| Witness route | the cheapest useful external tool or human review path to run next |

A card is a focused review prompt. It is not a proof that the code is safe and it
is not a substitute for Miri, sanitizers, Loom, Kani, Crux, or careful manual
review.

## 4. Compare JSON output

```bash
cargo run -p unsafe-review -- check \
  --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff \
  --format json
```

Use JSON when another tool needs to consume review cards. The same card contract
must back human output, JSON, Markdown, PR comments, LSP diagnostics, badges, and
agent packets.

## 5. Next steps

- Use [`../how-to/run-pr-checks.md`](../how-to/run-pr-checks.md) when preparing a
  pull request.
- Use [`../specs/README.md`](../specs/README.md) when you need exact behavior.
- Use [`../MISSION.md`](../MISSION.md) to revisit the trust boundary.
