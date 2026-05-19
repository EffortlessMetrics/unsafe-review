# unsafe-review-cli

CLI adapter crate for `unsafe-review`.

Most users should install the product facade:

```bash
cargo install unsafe-review
```

This crate owns command parsing, process-facing output, and the
`cargo-unsafe-review` integration binary. It depends on `unsafe-review-core` for
the ReviewCard engine and does not define an independent analyzer truth.

Current status: experimental advisory tooling. It does not run Miri,
`cargo-careful`, sanitizers, Loom, Shuttle, Kani, or Crux by default; it does
not post PR comments; and it does not enable blocking policy by default.

Repository documentation:
https://github.com/EffortlessMetrics/unsafe-review
