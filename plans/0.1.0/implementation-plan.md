# unsafe-review 0.1.0 implementation plan

Status: active
Owner: core/product
Linked proposal: ../../docs/proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md

## Work item ladder

1. Source-of-truth scaffold — done in this workspace.
2. Three-crate workspace — done in this workspace.
3. Domain model and review-card schema — initial implementation present.
4. Diff/workspace input — initial implementation present.
5. Source unsafe seam scanner — initial implementation present.
6. Hazard and obligation mapping — initial implementation present.
7. Contract and guard evidence — initial implementation present.
8. Witness routing — initial implementation present.
9. Human/JSON/Markdown output — initial implementation present.
10. Policy/baseline matching — planned.
11. SARIF/GitHub output — planned.
12. LSP projection — planned.
13. Agent packet hardening — planned.
14. Receipt import — planned.

## Proof commands

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask check-pr
```

## Release gate

0.1.0 may ship when fixture/golden tests cover the current analyzer behavior and
all README trust-boundary claims have support-tier entries.
