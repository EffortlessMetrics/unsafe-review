# unsafe-review-core

Core SDK and static analysis engine for `unsafe-review`.

Use this crate when embedding the ReviewCard engine programmatically. Most
command-line users should install the product facade instead:

```bash
cargo install unsafe-review
```

The public SDK centers on `AnalyzeInput`, `AnalyzeOutput`, and `ReviewCard`.
All PR summaries, SARIF, saved LSP JSON, agent packets, repo posture, outcome
reports, and receipt-aware output should project from those cards rather than
reclassifying findings independently.

Current status: experimental static unsafe contract review evidence. The engine
is source-based, stable-first, and intentionally conservative. It is not a
memory-safety proof, not a UB-free claim, and not a Miri result unless a scoped
witness receipt is attached.

Repository documentation:
https://github.com/EffortlessMetrics/unsafe-review
