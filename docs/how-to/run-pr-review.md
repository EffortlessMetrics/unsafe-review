# Run unsafe-review on a pull request

Use this guide when a branch changes unsafe-adjacent Rust code and you want a
review report for the diff.

## Prerequisites

- Run commands from a Git checkout.
- Ensure the comparison branch exists locally, for example `origin/main`.
- Build or install the CLI. In this repository, `cargo run -p unsafe-review --` is
  equivalent to the installed `unsafe-review` binary.

## Review the current branch diff

```bash
cargo run -p unsafe-review -- check --base origin/main
```

`--base origin/main` makes the CLI run `git diff origin/main...HEAD` from the
selected root. The default output is human-readable text for local review.

## Produce Markdown for a PR summary

```bash
cargo run -p unsafe-review -- check \
  --base origin/main \
  --format markdown \
  --out target/unsafe-review/pr-summary.md
```

Attach the generated Markdown file to the pull request or CI job summary.

## Produce JSON for automation

```bash
cargo run -p unsafe-review -- check \
  --base origin/main \
  --format json \
  --out target/unsafe-review/cards.json
```

Use JSON for CI annotations, dashboards, editor projections, or agent workflows.

## Review a saved diff

When CI or another tool already produced a unified diff, pass it directly:

```bash
cargo run -p unsafe-review -- check \
  --diff change.diff \
  --format json
```

A supplied `--diff` is useful for reproducible debugging because it decouples the
analysis from the current Git state.

## Limit an exploratory pilot run

```bash
cargo run -p unsafe-review -- pilot \
  --base origin/main \
  --max-cards 5
```

Use `pilot` when introducing the tool to a repository. It keeps the report small
while teams calibrate expectations and suppression policy.

## If a card needs deeper context

First run a repository scan so the card can be found by id, then ask for the
explanation or JSON context packet:

```bash
cargo run -p unsafe-review -- explain <card-id>
cargo run -p unsafe-review -- context <card-id>
```

`explain` is for humans. `context` is for tools and coding agents that need a
bounded packet for one unsafe seam.
