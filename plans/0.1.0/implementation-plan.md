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
10. Policy/baseline matching — planned; behavior specified in ../../docs/specs/UNSAFE-REVIEW-SPEC-0010-policy-baseline-suppressions.md.
11. SARIF/GitHub output — planned; behavior specified in ../../docs/specs/UNSAFE-REVIEW-SPEC-0011-pr-ci-output.md.
12. LSP projection — planned; behavior specified in ../../docs/specs/UNSAFE-REVIEW-SPEC-0012-lsp-editor-projection.md.
13. Agent packet hardening — planned; behavior specified in ../../docs/specs/UNSAFE-REVIEW-SPEC-0013-agent-packets.md.
14. Receipt import — planned; behavior specified in ../../docs/specs/UNSAFE-REVIEW-SPEC-0009-witness-receipts.md.
15. Repo inventory and badge hardening — planned; behavior specified in ../../docs/specs/UNSAFE-REVIEW-SPEC-0014-repo-inventory-badges.md.
16. Fixture calibration and support-tier gates — planned; behavior specified in ../../docs/specs/UNSAFE-REVIEW-SPEC-0016-fixtures-calibration-support.md.

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
