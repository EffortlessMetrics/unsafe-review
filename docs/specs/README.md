# Specifications

Specs define behavior. They do not carry the PR queue.

## Index

1. [Product contract](UNSAFE-REVIEW-SPEC-0001-product-contract.md)
2. [Review card schema](UNSAFE-REVIEW-SPEC-0002-review-card-schema.md)
3. [Input scope and diff model](UNSAFE-REVIEW-SPEC-0003-input-scope-and-diff-model.md)
4. [Unsafe seam extraction](UNSAFE-REVIEW-SPEC-0004-unsafe-seam-extraction.md)
5. [Hazard taxonomy and obligations](UNSAFE-REVIEW-SPEC-0005-hazard-taxonomy-and-obligations.md)
6. [Contract and discharge evidence](UNSAFE-REVIEW-SPEC-0006-contract-and-discharge-evidence.md)
7. [Test reachability](UNSAFE-REVIEW-SPEC-0007-test-reachability.md)
8. [Witness routing](UNSAFE-REVIEW-SPEC-0008-witness-routing.md)
9. [Witness receipts](UNSAFE-REVIEW-SPEC-0009-witness-receipts.md)
10. [Policy, baseline, suppressions](UNSAFE-REVIEW-SPEC-0010-policy-baseline-suppressions.md)
11. [PR and CI output](UNSAFE-REVIEW-SPEC-0011-pr-ci-output.md)
12. [LSP and editor projection](UNSAFE-REVIEW-SPEC-0012-lsp-editor-projection.md)
13. [Agent packets](UNSAFE-REVIEW-SPEC-0013-agent-packets.md)
14. [Repo inventory and badges](UNSAFE-REVIEW-SPEC-0014-repo-inventory-badges.md)
15. [Public API and crate surface](UNSAFE-REVIEW-SPEC-0015-public-api-crate-surface.md)
16. [Fixtures, calibration, support tiers](UNSAFE-REVIEW-SPEC-0016-fixtures-calibration-support.md)
17. [Security and file policy](UNSAFE-REVIEW-SPEC-0017-security-file-policy.md)

## Implementation backlog map

These specs are accepted behavior contracts, not a PR queue. The following map
highlights the specs that still describe planned or partially implemented work:

| Spec | Implementation state | Remaining implementation scope |
|---|---|---|
| [Witness receipts](UNSAFE-REVIEW-SPEC-0009-witness-receipts.md) | planned | receipt schema, importers for Miri/cargo-careful/sanitizers/Loom/Kani/Crux, conservative receipt-to-card matching |
| [Policy, baseline, suppressions](UNSAFE-REVIEW-SPEC-0010-policy-baseline-suppressions.md) | planned | policy decisions, baseline drift detection, suppression validation, policy exit codes |
| [PR and CI output](UNSAFE-REVIEW-SPEC-0011-pr-ci-output.md) | partially implemented | SARIF, GitHub summary, inline comment selection, policy artifact emission |
| [LSP and editor projection](UNSAFE-REVIEW-SPEC-0012-lsp-editor-projection.md) | planned | saved-workspace diagnostics, hover cards, read-only code actions, timeout/partial-analysis reporting |
| [Agent packets](UNSAFE-REVIEW-SPEC-0013-agent-packets.md) | partially implemented | hardened packet schema, redaction, schema tests, CLI/LSP packet parity |
| [Fixtures, calibration, support tiers](UNSAFE-REVIEW-SPEC-0016-fixtures-calibration-support.md) | partially implemented | golden harness, calibration classes, support-tier promotion checks, false-positive/false-negative ledger |

Implementation should keep JSON review cards as the canonical truth and treat
Markdown, SARIF, LSP diagnostics, PR comments, and agent packets as projections.

