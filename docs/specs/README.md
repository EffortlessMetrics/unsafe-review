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

## Implementation backlog coverage

The implementation plan still lists several planned slices. Their detailed behavior
contracts now live in these specs:

| Planned slice | Spec | Implementation focus |
| --- | --- | --- |
| Policy/baseline matching | [Policy, baseline, suppressions](UNSAFE-REVIEW-SPEC-0010-policy-baseline-suppressions.md) | policy TOML parsing, counted baselines, suppression validation, mode-based exit codes |
| SARIF/GitHub output | [PR and CI output](UNSAFE-REVIEW-SPEC-0011-pr-ci-output.md) | SARIF, GitHub summaries, CI artifact set, inline-comment payloads |
| LSP projection | [LSP and editor projection](UNSAFE-REVIEW-SPEC-0012-lsp-editor-projection.md) | saved-artifact diagnostics, hover content, copy/open commands, staleness reporting |
| Agent packet hardening | [Agent packets](UNSAFE-REVIEW-SPEC-0013-agent-packets.md) | canonical packet schema, bounded context, allowed/disallowed repairs, stop conditions |
| Receipt import | [Witness receipts](UNSAFE-REVIEW-SPEC-0009-witness-receipts.md) | receipt DTOs, validation, matching, stale/malformed receipt handling |

Each linked spec includes an **Implementation still required** section that can be
used as the implementation checklist for that slice.
