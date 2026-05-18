# How to review a working-tree diff

Use this guide when you want `unsafe-review` to inspect your current changes.

## Review against the default branch

```bash
unsafe-review check --base origin/main
```

The command compares your checkout against `origin/main` and emits review cards
for changed unsafe-adjacent seams.

## Review a saved diff

```bash
unsafe-review check --diff change.diff --format json
```

Saved diffs are useful for CI jobs, fixtures, and reproducing reports from
another machine.

## Generate a focused context packet

```bash
unsafe-review context UR-src-lib-rs-42-raw-pointer-read --json
```

Use a context packet when handing one bounded repair to an editor integration or
coding agent. The packet should include the card, allowed repair shape, commands
to verify the repair, and stop conditions.
