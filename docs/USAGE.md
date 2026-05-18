# Using `unsafe-review`

This guide shows how to run the current `unsafe-review` CLI in local development,
CI, and review follow-up workflows.

## What the tool reports

`unsafe-review` is a static PR-time review helper for unsafe-adjacent Rust code. It
looks for unsafe seams, classifies the operation family, and reports whether the
change has the review evidence that makes an unsafe-code review credible:

- a nearby safety contract (`# Safety`, `SAFETY:`, or equivalent prose),
- local guard or discharge evidence for the operation,
- related test reachability, and
- a plausible witness route such as Miri, sanitizers, Loom, Shuttle, Kani, Crux, or
  `cargo-careful`.

The output is advisory. A clean report means the analyzer did not find an open
review-evidence gap in its current support tier; it does **not** prove memory safety
or replace witness execution.

## Install and smoke-test

```bash
cargo install unsafe-review
unsafe-review --version
unsafe-review doctor --root .
```

From a checkout of this repository, use Cargo directly while developing:

```bash
cargo run -p unsafe-review -- --version
cargo run -p unsafe-review -- doctor --root .
```

`doctor` verifies the root path and reports whether basic external tools are visible
on `PATH`. It does not install Miri, sanitizers, or model-checking tools for you.

## Review a pull request diff

Use `check` for the normal PR workflow. With `--base`, the CLI asks Git for a
three-dot diff from the base ref to `HEAD`:

```bash
unsafe-review check --base origin/main
```

If CI has already produced a patch file, pass it explicitly:

```bash
unsafe-review check --diff change.diff --format json --out unsafe-review.json
```

Useful options for `check`, `repo`, and `pilot`:

| Option | Meaning |
|---|---|
| `--root <dir>` | Repository root to analyze. Defaults to the current directory. |
| `--base <ref>` | Build a Git diff with `<ref>...HEAD`. |
| `--diff <file>` | Read a unified diff from a file instead of invoking Git. |
| `--format human\|json\|markdown` | Select output format. Defaults to human text. |
| `--json` | Shortcut for `--format json`. |
| `--markdown` | Shortcut for `--format markdown`. |
| `--out <file>` | Write rendered output to a file. Parent directories are created. |
| `--max-cards <n>` | Limit the number of emitted cards. |

When neither `--base` nor `--diff` is supplied, `check` falls back to a repository
scan. Prefer an explicit base or diff in CI so the result is tied to the reviewed
change.

## Start with pilot mode

`pilot` uses the same diff analyzer as `check`, but defaults to at most five cards so
teams can trial the signal without flooding a PR:

```bash
unsafe-review pilot --base origin/main --format markdown --out unsafe-review.md
```

Treat pilot output as triage: fix obvious missing contracts or guards, route valuable
witnesses, and use suppressions or baselines only for intentional exceptions.

## Inventory a whole repository

Use `repo` when you want the current open-gap inventory instead of a PR diff:

```bash
unsafe-review repo --format json --out unsafe-review-repo.json
```

Use `badges` to generate Shields-compatible JSON badge data from that inventory:

```bash
unsafe-review badges --out badges/
```

The badge files are:

- `badges/unsafe-review.json` for the total open actionable gap count, and
- `badges/unsafe-review-plus.json` for contract, guard, and witness breakdowns.

Badges intentionally describe review gaps only. They must not be presented as
"UB-free" or "safe" badges.

## Follow up on a card

Each finding has a stable card id in the rendered output. Use that id to get a
review-focused explanation:

```bash
unsafe-review explain UR-src-lib-rs-42-raw-pointer-read
```

For agent or LLM handoff workflows, request the structured packet:

```bash
unsafe-review context UR-src-lib-rs-42-raw-pointer-read > packet.json
# or
unsafe-review explain --format json UR-src-lib-rs-42-raw-pointer-read > packet.json
```

The packet includes the card, related evidence, and next actions; it is meant to help
a reviewer or agent inspect the code, not to authorize automatic edits.

## CI pattern

A minimal advisory CI step writes JSON for machines and Markdown for PR comments:

```bash
unsafe-review check --base origin/main --format json --out unsafe-review.json
unsafe-review check --base origin/main --format markdown --out unsafe-review.md
```

Recommended rollout order:

1. Run in advisory mode and publish artifacts.
2. Pilot on a few unsafe-heavy crates with `--max-cards`.
3. Add baselines for known debt after human review.
4. Consider blocking only on newly introduced, high-confidence gaps once support tiers
   and fixtures cover your use case.

## Common fixes for findings

- Add or tighten a `# Safety` section on unsafe public APIs.
- Add `SAFETY:` comments immediately before unsafe blocks to connect preconditions to
  the operation being performed.
- Move local validation close to raw pointer dereferences, slice construction,
  `NonNull` construction, FFI calls, or concurrency-sensitive unsafe code.
- Add targeted tests that reach the unsafe seam.
- Route the card to an appropriate witness and attach a receipt when witness evidence
  is available.

Keep suppressions narrow and documented. A suppression should explain why the missing
evidence is acceptable, not hide unknown unsafe behavior.
