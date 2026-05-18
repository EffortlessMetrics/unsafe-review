# CLI guide

`unsafe-review` is a PR-time reviewer for unsafe-adjacent Rust changes. It does
not prove that code is safe. It turns changed unsafe seams into review cards that
show the contract, guard, test reach, and witness route reviewers should inspect.

## Mental model

```text
diff or repo scan
-> unsafe seam
-> inferred hazards and obligations
-> local evidence checks
-> witness routing
-> review card
```

A **review card** is the unit of work. Each card names one unsafe seam, explains
what evidence is present or missing, and suggests the next review action.

| Term | Meaning |
|---|---|
| Contract | The written `# Safety` or `SAFETY:` explanation for why the unsafe operation is valid. |
| Guard / discharge | Nearby code evidence that checks or establishes the inferred obligation. |
| Reach | Tests that appear to exercise the owner or nearby symbol. |
| Witness route | A suggested specialized checker such as Miri, `cargo-careful`, sanitizers, Loom, Shuttle, Kani, or Crux. |
| Witness receipt | Evidence imported from a witness run. Without a receipt, a route is only a recommendation. |

## Trust boundary

`unsafe-review` reports review evidence. It is intentionally advisory by default:

- it is not a soundness proof;
- it is not a replacement for manual unsafe-code review;
- it is not a Miri, sanitizer, Loom, Shuttle, Kani, or Crux result unless a
  witness receipt is attached;
- the v0.1 analyzer is stable-only and conservative, so cards should be treated
  as prompts for review rather than calibrated pass/fail verdicts.

## Common workflows

### Preflight a checkout

Run `doctor` first when setting up CI or debugging local behavior:

```bash
unsafe-review doctor --root .
```

It verifies the root path and reports whether basic tools such as `git` and
`cargo` are available.

### Review a pull request diff

```bash
unsafe-review check --base origin/main
```

`--base` shells out to `git diff <base>...HEAD` from `--root`. Use it in PR CI
when the repository has enough history for the merge-base comparison.

### Review a supplied patch file

```bash
unsafe-review check --diff change.diff --format json
```

Use `--diff` when CI already produced a unified diff or when testing fixtures.
`--diff` is read as a file path from the current process working directory.

### Keep early pilots focused

```bash
unsafe-review pilot --base origin/main --max-cards 5
```

`pilot` is a diff review mode that defaults to at most five cards. It is useful
while introducing the tool to a repository and tuning local policy.

### Inventory the full repository

```bash
unsafe-review repo --format json --out target/unsafe-review/repo.json
```

`repo` scans the repository instead of only changed lines. Use it to establish a
baseline, feed dashboards, or inspect older unsafe seams that are not part of the
current PR.

### Generate badge data

```bash
unsafe-review badges --out badges/
```

This writes Shields-compatible JSON files:

- `badges/unsafe-review.json` reports total open gaps;
- `badges/unsafe-review-plus.json` breaks out contract, guard, and witness gaps.

### Explain a card

```bash
unsafe-review explain UR-src-lib-rs-8-read-header-raw_pointer_read
unsafe-review context UR-src-lib-rs-8-read-header-raw_pointer_read
```

`explain` prints reviewer-facing detail for one card. `context` prints an
LLM-ready packet for agent-assisted review. Both commands rescan the repository
and look up the supplied card ID, so run them from the same root used to produce
the card.

## Command reference

| Command | Purpose | Important options |
|---|---|---|
| `check` | Review the current diff, a supplied diff, or the repo when no diff source is supplied. | `--root`, `--base`, `--diff`, `--format`, `--json`, `--markdown`, `--out`, `--max-cards` |
| `pilot` | Diff review with a default card cap of five. | Same options as `check` |
| `repo` | Full repository inventory. | Same options as `check`; normally use `--format json` and `--out` |
| `badges` | Write badge JSON from a full repo scan. | `--root`, `--out` |
| `explain` | Explain one card for a human reviewer. | `--root`, `--format markdown\|json`, `<card-id>` |
| `context` | Emit one card's context packet. | `--root`, `<card-id>` |
| `doctor` | Print local environment diagnostics. | `--root` |

Supported output formats for review commands are `human`, `json`, and
`markdown`. `--json` and `--markdown` are shorthands for `--format json` and
`--format markdown`.

## Reading human output

A typical card includes:

```text
GUARD_MISSING src/lib.rs:8
  id: UR-src-lib-rs-8-read-header-raw_pointer_read
  operation: unsafe { ptr.cast::<Header>().read() }
  contract: Nearby `SAFETY:` comment was detected
  discharge: Some inferred safety obligations are missing local guard evidence
  reach: 1 related test file(s) mention owner `read_header`
  missing:
    - Missing visible local guard for inferred safety obligations
    - No witness receipt imported for this card
  next: Add or expose the local guard that discharges the `raw_pointer_read` safety obligation.
  verify:
    cargo +nightly miri test read_header
    cargo +nightly careful test read_header
```

Review the `missing` section first, then decide whether to add documentation,
make a guard visible to reviewers, add or connect tests, run the suggested
witness, or suppress/baseline the card according to repository policy.

## Suggested CI shape

Start advisory and artifact-only:

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
unsafe-review check --base origin/main --format json --out target/unsafe-review/pr.json
```

Promote individual witness routes to blocking jobs only after the repository has
calibrated fixtures, baselines, and suppression policy.
