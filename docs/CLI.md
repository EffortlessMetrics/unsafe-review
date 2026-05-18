# CLI guide

`unsafe-review` is a PR-time review helper for Rust changes that add or modify
unsafe-adjacent code. It is intentionally advisory by default: the CLI reports
review evidence and gaps, but it does not prove that code is sound.

## Command overview

| Command | Typical use | Notes |
|---|---|---|
| `unsafe-review check` | Review the current change set. | Uses `--base` or `--diff` when provided; otherwise scans the repository. |
| `unsafe-review pilot` | Run a small, review-friendly diff pass. | Defaults `--max-cards` to `5` unless another limit is supplied. |
| `unsafe-review repo` | Inventory unsafe-adjacent code in the whole repository. | Useful for baselines, dashboards, and release planning. |
| `unsafe-review badges` | Write Shields-compatible badge JSON files. | Produces `unsafe-review.json` and `unsafe-review-plus.json`. |
| `unsafe-review explain` | Render a detailed explanation for one card. | Takes a card id from a previous `check` or `repo` run. |
| `unsafe-review context` | Emit an LLM-ready context packet for one card. | Takes the same card id used by `explain`. |
| `unsafe-review doctor` | Check basic local prerequisites. | Reports root, Git availability, Cargo availability, and policy mode. |

## Review a pull request diff

Use `--base` when the repository has the target branch available locally:

```bash
unsafe-review check --base origin/main
```

The CLI runs `git diff <base>...HEAD` from `--root` and analyzes the resulting
unified diff. If your CI checkout uses a different target branch, pass that branch
or commit explicitly:

```bash
unsafe-review check --root . --base origin/release-1.2
```

Use `--diff` when CI has already produced a patch file or when you want a stable
fixture input:

```bash
unsafe-review check --diff change.diff --format json
```

## Choose an output format

`check`, `pilot`, and `repo` support human-readable text, JSON, and Markdown:

```bash
unsafe-review check --base origin/main --format human
unsafe-review check --base origin/main --format json
unsafe-review check --base origin/main --format markdown
```

Short aliases are also accepted:

```bash
unsafe-review check --base origin/main --json
unsafe-review check --base origin/main --markdown
```

Use `--out` to write the rendered report instead of printing it to stdout:

```bash
unsafe-review check --base origin/main --format markdown --out unsafe-review.md
```

## Interpret the summary

Every run returns a summary plus zero or more review cards. The most important
summary counters are:

| Counter | Meaning |
|---|---|
| `unsafe_sites` | Unsafe-adjacent operations found in the selected scope. |
| `cards` | Review cards emitted for actionable or explanatory findings. |
| `open_actionable_gaps` | Cards that still need reviewer attention. |
| `contract_missing` | Unsafe API or block lacks a nearby safety contract. |
| `guard_missing` | The analyzer did not find a nearby local guard for an obligation. |
| `guarded_unwitnessed` | A guard was found, but no witness route or receipt was found. |
| `unsafe_unreached` | The changed unsafe code does not appear to be reached by tests. |
| `requires_loom` | The finding likely needs concurrency exploration such as Loom or Shuttle. |
| `miri_unsupported` | The finding likely cannot be fully exercised by Miri alone. |
| `static_unknown` | The static analyzer could not classify enough evidence confidently. |

Treat these counters as review triage, not as pass/fail proof. A zero-card run
means the current static checks did not find a gap in the selected scope; it does
not mean the code is UB-free.

## Work with cards

A card id is stable enough for a reviewer workflow within the same revision. Use
`explain` to expand the card into reviewer-facing Markdown:

```bash
unsafe-review explain UR-src-lib-rs-42-raw-pointer-read
```

Use `context` when handing one finding to an assistant or another review tool:

```bash
unsafe-review context UR-src-lib-rs-42-raw-pointer-read
```

`explain --format json` emits the same context packet as `context`.

## Suggested CI jobs

Start with advisory reporting and an uploaded artifact:

```bash
unsafe-review check --base origin/main --format markdown --out unsafe-review.md
```

For dashboards, add a whole-repository inventory and badge files:

```bash
unsafe-review repo --format json --out unsafe-review.repo.json
unsafe-review badges --out badges/
```

Keep the trust boundary visible in CI summaries: `unsafe-review` checks review
evidence, not Rust memory safety itself.
