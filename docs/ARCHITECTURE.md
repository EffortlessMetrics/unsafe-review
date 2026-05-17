# Architecture

`unsafe-review` is three published crates plus internal module families.

```text
unsafe-review          # product facade / install handle
  -> unsafe-review-cli # command parsing, UX, rendering
      -> unsafe-review-core # SDK, domain, analyzer, policy, output schemas
```

This follows the boundary doctrine: design seams like microcrates, implement most
as module families, and publish only seams that deserve a support promise.

## Core pipeline

```text
input scope
-> diff/workspace discovery
-> Rust source seam extraction
-> hazard classification
-> safety obligation mapping
-> contract evidence mining
-> discharge evidence mining
-> test reach estimation
-> witness routing
-> review-card classification
-> output projection
```

## Stability posture

v0.1 is stable-only and avoids `rustc_private`. That keeps the tool cheap,
portable, and predictable while the product loop is proven. A later optional
nightly adapter can add MIR facts once the review-card contract is stable.

## Canonical unit

The review card is the single truth object. PR summaries, SARIF, LSP diagnostics,
hovers, badges, and agent packets must project from the same card. No second
truth surface is allowed.
