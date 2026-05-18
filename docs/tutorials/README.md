# Tutorials

Tutorials are learning-oriented. They should produce a working result before they
explain every option.

## First unsafe-review pass

This tutorial runs `unsafe-review` against a bundled fixture and reads the first
review card.

### Goal

By the end, you will know how to:

1. run a diff-scoped review,
2. recognize the summary and card table,
3. identify the recommended next action, and
4. keep the trust boundary clear.

### Prerequisites

- A Rust toolchain compatible with this workspace.
- A checkout of this repository.

### 1. Run the fixture review

From the workspace root:

```bash
cargo run -q -p unsafe-review -- check \
  --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff \
  --format markdown
```

### 2. Read the summary

The fixture should report one unsafe seam card:

```text
1 changed/repo unsafe seam card(s) found.
```

That means the analyzer found one unsafe-adjacent change that needs review
evidence. It does not mean the code is unsound.

### 3. Read the recommended next action

The recommendation tells the reviewer which evidence gap to close first. For this
fixture, the next action is to add or expose the local guard for the raw pointer
read obligation.

The suggested witness route is Miri:

```bash
cargo +nightly miri test read_header
```

Treat this as routing guidance. A static `unsafe-review` card is not itself a
Miri result.

### 4. Read the card table

The Markdown output includes a row like this:

| ID | Class | Hazard | Missing | Route |
|---|---|---|---|---|
| `UR-src-lib-rs-8-read-header-raw_pointer_read` | `guard_missing` | `pointer_validity` | `guard` | `miri` |

The important fields are:

- **Class**: the type of review gap.
- **Hazard**: the safety risk category.
- **Missing**: the evidence that should be added or exposed.
- **Route**: the witness lane most likely to validate the obligation.

### 5. Next steps

After the first pass, use the how-to guides to run against a real PR, cap output
for pilot adoption, or produce JSON for CI.
