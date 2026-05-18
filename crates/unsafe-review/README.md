# unsafe-review

Install handle and product façade for `unsafe-review`.

`unsafe-review` is a static unsafe Rust review assistant. It scans unsafe-adjacent
changes, emits review cards for missing safety contracts, guards, tests, or witness
routes, and keeps the trust boundary explicit: findings are review evidence, not a
soundness proof.

## Install

```bash
cargo install unsafe-review
```

When working from the source repository before installing, run:

```bash
cargo run -q -p unsafe-review -- check --base origin/main
```

## Common commands

```bash
# Review a pull-request diff
unsafe-review check --base origin/main

# Review a saved unified diff
unsafe-review check --diff change.diff --format json

# Inventory the whole repository
unsafe-review repo --format json

# Explain a card or create an automation packet
unsafe-review explain <card-id>
unsafe-review context <card-id>
```

For embedding, use `unsafe-review-core`. For full project documentation, see the
repository README and `docs/USAGE.md`.
