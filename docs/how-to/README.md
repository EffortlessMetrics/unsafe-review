# How-to guides

How-to guides are task-oriented recipes. They assume you already know what
`unsafe-review` is and need the shortest safe path to a result.

## Review the current PR diff

Use this when a branch should be compared with the merge base of `origin/main`:

```bash
unsafe-review check --base origin/main
```

For machine-readable output:

```bash
unsafe-review check --base origin/main --format json --out target/unsafe-review/cards.json
```

## Review a supplied diff file

Use this in CI systems that already export a unified diff:

```bash
unsafe-review check --diff change.diff --format markdown
```

## Keep an early rollout small

Use `pilot` to keep the default output advisory and capped while introducing the
tool to a repository:

```bash
unsafe-review pilot --base origin/main --max-cards 5
```

Triage the highest-signal cards first, then raise or remove the cap once the team
has a baseline.

## Explain a card

Use `explain` when a reviewer needs a focused Markdown explanation for one card:

```bash
unsafe-review explain UR-src-lib-rs-8-read-header-raw_pointer_read
```

Use `context` when an agent or another tool needs a JSON packet for the same card:

```bash
unsafe-review context UR-src-lib-rs-8-read-header-raw_pointer_read
```

## Generate repo badges

Use badges for a repository-level status surface rather than a PR annotation:

```bash
unsafe-review badges --out badges/
```

The command writes badge JSON files that can be published by the repository's
normal documentation or CI artifact flow.

## Decide where to put new documentation

Use this rule before adding a page:

- Teaching a first successful path? Put it in `docs/tutorials/`.
- Solving a concrete task? Put it in `docs/how-to/`.
- Defining behavior, schema, support, or policy? Put it in reference docs.
- Explaining why the project chose an approach? Put it in explanation docs.
