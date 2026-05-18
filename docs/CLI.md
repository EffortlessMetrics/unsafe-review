# CLI guide

`unsafe-review` is an advisory review tool for unsafe Rust changes. It reads a
repository or a diff, emits review cards for unsafe-adjacent gaps, and keeps the
result intentionally separate from any claim of memory-safety proof.

## Review workflow

1. Run `doctor` to confirm the local checkout and basic tools are visible.
2. Run `check` on the pull request diff during development or CI.
3. Use a card ID from the output with `explain` or `context` when a reviewer or
   agent needs the evidence bundle behind a finding.
4. Run `repo` and `badges` when you want an inventory of current unsafe-review
   debt for dashboards or release tracking.

```bash
unsafe-review doctor --root .
unsafe-review check --base origin/main --format markdown
unsafe-review explain --root . <card-id>
unsafe-review context --root . <card-id>
unsafe-review repo --format json --out target/unsafe-review/repo.json
unsafe-review badges --out target/unsafe-review/badges
```

## Commands

| Command | Purpose | Common options |
|---|---|---|
| `doctor` | Prints environment and policy diagnostics. | `--root .` |
| `check` | Reviews a diff or, when no diff source is supplied, scans the repository. | `--root .`, `--base origin/main`, `--diff file`, `--format human\|json\|markdown`, `--out file`, `--max-cards n` |
| `repo` | Produces repository inventory output. | Same options as `check`; JSON is the intended machine-readable format. |
| `pilot` | Runs a limited advisory diff review for trial adoption. | Same options as `check`; defaults to `--max-cards 5` when omitted. |
| `badges` | Writes Shields-compatible badge JSON files. | `--root .`, `--out badges` |
| `explain` | Prints a reviewer-facing explanation for one card. | `--root .`, `--format markdown\|json`, `<card-id>` |
| `context` | Prints the LLM-ready context packet for one card. | `--root .`, `<card-id>` |

## Diff sources

`check`, `repo`, and `pilot` accept either a Git base or a unified diff file.

```bash
# Compare the current branch to origin/main.
unsafe-review check --base origin/main

# Analyze a saved unified diff.
unsafe-review check --diff change.diff --format json

# Scan without an explicit diff source.
unsafe-review check --root .
```

When `--base` is used, the CLI shells out to `git diff <base>...HEAD` from the
selected root. When `--diff` is used, the CLI reads that file directly.

## Output formats

- `human` is the default console-oriented format.
- `markdown` is intended for PR comments, handoffs, and review notes.
- `json` is intended for CI, dashboards, badges, and agent integrations.

Use `--out <file>` with `check`, `repo`, or `pilot` when CI should persist the
rendered output instead of printing it to standard output.

## Interpreting results

A review card is a prompt to inspect evidence around an unsafe seam. It can point
to missing contracts, missing local guards, missing test reachability, or a route
to an external witness such as Miri, sanitizers, Loom, Shuttle, Kani, or Crux.

Treat cards as advisory review evidence:

- A clean run is not a proof that the code is memory safe.
- A witness route is not a witness result unless a receipt is attached.
- Badge counts report open review gaps, not raw unsafe usage and not UB freedom.
- Suppressions and baselines should explain accepted review debt rather than hide
  findings without context.

## CI sketch

```bash
cargo check --workspace --all-targets
unsafe-review check --base origin/main --format json --out target/unsafe-review/check.json
unsafe-review badges --out target/unsafe-review/badges
```

Keep CI policy advisory until the repository has calibrated fixtures, golden
outputs, and an agreed suppression process for accepted debt.
