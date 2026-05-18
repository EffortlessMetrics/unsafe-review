# unsafe-review

Install handle and product façade for `unsafe-review`.

`unsafe-review` is a PR-time unsafe Rust review assistant. It scans changed
unsafe-adjacent code and emits review cards for missing safety contracts, local guards,
test reachability, or witness routing. It is static review evidence, not a memory-safety
proof and not a replacement for Miri, sanitizers, Loom, Kani, or other witnesses.

## Install and run

```bash
cargo install unsafe-review
unsafe-review check --base origin/main
```

For local development from a repository checkout:

```bash
cargo run -p unsafe-review -- check --base origin/main
```

Useful commands:

```bash
unsafe-review check --diff change.diff --format json
unsafe-review repo --format json
unsafe-review badges --out badges/
unsafe-review explain <card-id>
unsafe-review context <card-id>
```

For embedding or custom tooling, use `unsafe-review-core` directly.
