# CLI reference

`unsafe-review` is a static, advisory review tool for unsafe-adjacent Rust changes. It
looks for unsafe seams, maps them to review obligations, and emits review cards for
missing contracts, local guards, test reachability, or witness routing. It does not
execute witness tools by default and does not prove memory safety.

## Installation

```bash
cargo install unsafe-review
```

During local development, run the same commands through Cargo:

```bash
cargo run -p unsafe-review -- check --base origin/main
cargo run -p unsafe-review -- doctor
```

The workspace also builds a Cargo subcommand binary, so an installed build can be
invoked as either `unsafe-review ...` or `cargo unsafe-review ...`.

## Common workflows

### Review a pull request diff

Use `--base` when the repository has the target branch available locally. The CLI
runs `git diff <base>...HEAD` from `--root` and analyzes the resulting patch.

```bash
unsafe-review check --base origin/main
```

### Review a saved unified diff

Use `--diff` in CI systems that already provide a patch file, or when reproducing a
specific fixture locally.

```bash
unsafe-review check --diff change.diff --format json
```

### Scan the whole repository

Omit diff input, or use the `repo` command, when you want an inventory of all current
unsafe-review cards rather than only cards associated with changed lines.

```bash
unsafe-review repo --format markdown --out unsafe-review.md
```

### Start with a bounded pilot

`pilot` behaves like `check` but caps output to five cards unless `--max-cards` is set.
This is useful when introducing the tool to a repository that may have many existing
unsafe seams.

```bash
unsafe-review pilot --base origin/main
unsafe-review pilot --base origin/main --max-cards 10
```

### Generate badge JSON

`badges` writes Shields-compatible JSON badge files to the requested output directory.

```bash
unsafe-review badges --out badges/
```

### Explain and package one card

Use `explain` for a human-readable explanation of a card, and `context` for an
LLM-ready JSON packet containing the card and nearby evidence.

```bash
unsafe-review explain UR-src-lib-rs-42-raw-pointer-read
unsafe-review context UR-src-lib-rs-42-raw-pointer-read
```

## Commands

| Command | Purpose |
|---|---|
| `check` | Analyze a diff or, when no diff is supplied, the current repository. |
| `repo` | Analyze the repository inventory. |
| `pilot` | Analyze with a default `--max-cards 5` cap. |
| `badges` | Write badge JSON files. |
| `explain` | Explain one card by id. |
| `context` | Emit one card's context packet as JSON. |
| `doctor` | Print basic environment and policy diagnostics. |
| `help` | Print command summary. |

## Options

| Option | Commands | Description |
|---|---|---|
| `--root <dir>` | `check`, `repo`, `pilot`, `badges`, `explain`, `context`, `doctor` | Repository root. Defaults to `.`. |
| `--base <rev>` | `check`, `repo`, `pilot` | Generate a diff with `git diff <rev>...HEAD`. |
| `--diff <file>` | `check`, `repo`, `pilot` | Analyze a supplied unified diff file. Takes precedence over `--base`. |
| `--format human\|json\|markdown` | `check`, `repo`, `pilot`, `explain` | Select output format. `check`, `repo`, and `pilot` default to `human`; `explain` defaults to `markdown`. |
| `--json` | `check`, `repo`, `pilot` | Shortcut for `--format json`. |
| `--markdown` | `check`, `repo`, `pilot` | Shortcut for `--format markdown`. |
| `--out <file>` | `check`, `repo`, `pilot` | Write rendered output to a file instead of stdout. Parent directories are created. |
| `--max-cards <n>` | `check`, `repo`, `pilot` | Limit emitted cards. `pilot` defaults this to `5`. |
| `--out <dir>` | `badges` | Badge output directory. Defaults to `badges`. |

## Output model

Every finding is represented as a review card. A card is a review prompt, not a verdict:

- **Contract evidence** asks whether the unsafe seam documents its safety preconditions.
- **Guard evidence** asks whether nearby code locally checks or establishes those
  preconditions.
- **Test reachability** asks whether changed unsafe-adjacent code is exercised by tests.
- **Witness routing** suggests external tools such as Miri, sanitizers, Loom, Shuttle,
  Kani, Crux, or `cargo-careful` when they are relevant.

JSON and Markdown output are intended for CI annotations, dashboards, or follow-up
agent packets. Human output is optimized for terminal review.

## Exit status

The CLI exits with status `0` when command parsing and analysis complete, even if it
emits cards. It exits with status `2` for command, input, or analysis errors. The tool is
advisory by default; make a CI job blocking only after your repository has calibrated
fixtures, baselines, and suppression policy.
