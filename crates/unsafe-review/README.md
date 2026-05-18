# unsafe-review

Install handle and product façade for `unsafe-review`.

```bash
cargo install unsafe-review
unsafe-review doctor
unsafe-review check --base origin/main
```

Use this crate when you want the end-user CLI. For programmatic integrations, depend
on `unsafe-review-core` directly; for command parsing and output adapters, see
`unsafe-review-cli`.

## Output contract

The CLI reports review evidence around unsafe-adjacent seams. A finding means that
review evidence is missing or should be inspected; it does not prove undefined
behavior and it does not prove the crate is memory-safe when no cards are emitted.
