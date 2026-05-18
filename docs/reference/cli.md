# CLI reference

`unsafe-review` is an advisory CLI for static unsafe contract review. It reports
review evidence and gaps; it does not prove memory safety.

## Global commands

```text
unsafe-review --help
unsafe-review --version
```

## `check`

Review the selected diff and emit review cards.

```text
unsafe-review check [--root .] [--base origin/main | --diff file]
                    [--format human|json|markdown] [--out file]
                    [--max-cards n]
```

Options:

- `--root <dir>`: repository or fixture root. Defaults to the current directory.
- `--base <rev>`: analyze `git diff <rev>...HEAD` from `--root`.
- `--diff <file>`: analyze a supplied unified diff file.
- `--format <human|json|markdown>`: choose output format. Defaults to `human`.
- `--json`: shortcut for `--format json`.
- `--markdown`: shortcut for `--format markdown`.
- `--out <file>`: write output to a file instead of standard output.
- `--max-cards <n>`: cap the number of emitted cards.

If neither `--base` nor `--diff` is supplied, `check` performs a repository scan
without a diff source.

## `repo`

Scan the repository instead of only a pull request diff.

```text
unsafe-review repo [--root .] [--format human|json|markdown] [--out file]
```

Use `repo` for inventory, calibration, card lookup, badges, and context packet
collection.

## `pilot`

Run a small diff review for adoption experiments.

```text
unsafe-review pilot [--root .] [--base origin/main | --diff file]
                    [--format human|json|markdown] [--max-cards n]
```

`pilot` defaults to at most five cards when no explicit `--max-cards` value is
provided.

## `badges`

Write badge JSON files from a repository scan.

```text
unsafe-review badges [--root .] [--out badges]
```

The command writes:

- `unsafe-review.json`
- `unsafe-review-plus.json`

## `explain`

Print a human-oriented explanation for one card id.

```text
unsafe-review explain [--root .] [--format markdown|json] <card-id>
```

Markdown is the default. JSON output returns the same context packet as
`context`.

## `context`

Print an LLM/tool-oriented JSON packet for one card id.

```text
unsafe-review context [--root .] <card-id>
```

Use this command to hand one bounded unsafe-review task to an editor integration
or coding agent.

## `doctor`

Check basic local environment assumptions.

```text
unsafe-review doctor [--root .]
```

`doctor` verifies that the root is a directory, reports whether common tools are
available, and reminds users that policy is advisory by default.
