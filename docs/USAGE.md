# Usage guide

This guide covers the day-to-day `unsafe-review` workflow: running a diff scan,
reading review cards, producing machine-readable output, and knowing what the tool
is allowed to claim.

## What `unsafe-review` checks

`unsafe-review` is a static review assistant for unsafe-adjacent Rust changes. It
looks for unsafe seams in changed files, classifies the likely hazard, and reports
whether the review evidence around the seam is present:

| Evidence | What the tool looks for | Why reviewers care |
|---|---|---|
| Contract | nearby `# Safety` docs or `SAFETY:` comments | states the caller or maintainer obligation |
| Guard | nearby checks that appear to discharge the obligation | shows the code path enforces the contract locally |
| Test reach | tests or fixture paths that appear to exercise the seam | makes the unsafe path visible to normal test runs |
| Witness route | a likely external checker such as Miri, sanitizers, Loom, Kani, or Crux | gives reviewers a concrete next verification step |

A card is a review prompt, not a proof. A clean run means the scanner did not find
an actionable evidence gap under the selected inputs; it does not mean the crate is
memory-safe or undefined-behavior-free.

## Install and run from this repository

Until you are using a published package, run the CLI from the workspace checkout:

```bash
cargo run -q -p unsafe-review -- check --base origin/main
```

If the package is installed, the same command becomes:

```bash
unsafe-review check --base origin/main
```

The packaged binary is named `unsafe-review`. The CLI crate also provides a
`cargo-unsafe-review` binary for cargo-style integration.

## Common workflows

### Review the current branch diff

```bash
unsafe-review check --base origin/main
```

This shells out to `git diff origin/main...HEAD` from `--root` and scans unsafe
seams in the diff. Use the branch that your pull request targets as the base.

### Review a saved unified diff

```bash
unsafe-review check --diff change.diff --format json
```

Use this for CI jobs that already produce a patch file, for reproducing reports, or
for fixture-driven debugging.

### Scan the whole repository

```bash
unsafe-review repo --format json --out unsafe-review.repo.json
```

Repository mode ignores diff filtering and inventories unsafe seams across the
workspace root. This is useful for baselines, badges, and choosing migration work.

### Limit a pilot rollout

```bash
unsafe-review pilot --base origin/main --max-cards 5 --format markdown
```

Pilot mode is advisory and diff-scoped like `check`, but `--max-cards` keeps early
rollouts focused on the first few findings.

### Generate badge JSON

```bash
unsafe-review badges --out badges/
```

This writes `unsafe-review.json` and `unsafe-review-plus.json` Shields-compatible
badge payloads for repository dashboards.

### Explain a card or produce an agent packet

```bash
unsafe-review explain UR-src-lib-rs-42-raw-pointer-read
unsafe-review context UR-src-lib-rs-42-raw-pointer-read
```

`explain` produces reviewer-facing detail. `context` emits JSON intended for an
LLM or automation agent that needs the card, obligation, nearby evidence, and route
in one packet.

## Output formats

`check`, `repo`, and `pilot` accept:

- `--format human` for local terminal review.
- `--format markdown` for pull request comments or job summaries.
- `--format json` for automation, archival, and downstream projections.

Most scan commands also accept `--out <path>` to write the rendered output to a
file instead of standard output.

## Reading a review card

Each card has a stable ID and enough context for a reviewer to decide the next
action. Treat the classification as a triage hint:

- `contract_missing`: add or improve the `# Safety` contract or `SAFETY:` comment.
- `guard_missing`: add a local check or document why the caller upholds it.
- `guarded_unwitnessed`: route the path to a witness such as Miri or a sanitizer.
- `covered`: evidence was found, but reviewers should still inspect the unsafe
  invariant.

When a card points at a witness, run the witness separately and keep its receipt or
CI log with the change. `unsafe-review` does not execute Miri, sanitizers, Loom,
Kani, Crux, or `cargo-careful` itself.

## Fixture smoke test

The repository includes calibrated fixtures. This command scans one fixture diff and
prints the JSON card stream:

```bash
cargo run -q -p unsafe-review -- check \
  --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff \
  --format json
```

For full development validation, prefer the workspace checks in the main README.

## Troubleshooting

| Symptom | Check |
|---|---|
| `git diff failed` | Confirm `--base` exists locally and `--root` points at the git repository. |
| No cards in diff mode | Confirm the unsafe lines are included in the patch, or try `repo` mode. |
| Too many cards during rollout | Use `pilot --max-cards <n>` and add a baseline before enforcing policy. |
| Need machine-readable data | Use `--format json --out <path>`. |
| Unsure whether local tools are available | Run `unsafe-review doctor --root .`. |
