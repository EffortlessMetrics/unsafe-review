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

## Implementation backlog covered by specs

The following areas are specified but still need implementation work. The linked
specs define behavior, acceptance examples, and fixture expectations so future
implementation PRs can stay small and testable.

| Area | Spec | First implementation slice |
| --- | --- | --- |
| Obligation-level evidence | [SPEC-0006](UNSAFE-REVIEW-SPEC-0006-contract-and-discharge-evidence.md) | Persist lane-level evidence and replace card-wide guard summaries. |
| Witness receipt import | [SPEC-0009](UNSAFE-REVIEW-SPEC-0009-witness-receipts.md) | Add receipt DTOs and explicit artifact import. |
| Policy/baseline/suppression matching | [SPEC-0010](UNSAFE-REVIEW-SPEC-0010-policy-baseline-suppressions.md) | Parse policy ledgers and classify cards before exit-code decisions. |
| PR and CI projections | [SPEC-0011](UNSAFE-REVIEW-SPEC-0011-pr-ci-output.md) | Render Markdown summary, SARIF, and witness-plan artifacts from cards. |
| LSP/editor projection | [SPEC-0012](UNSAFE-REVIEW-SPEC-0012-lsp-editor-projection.md) | Load saved card artifacts and expose diagnostics/hover read-only. |
| Agent packet hardening | [SPEC-0013](UNSAFE-REVIEW-SPEC-0013-agent-packets.md) | Stabilize packet DTOs and generate packets from missing evidence. |
| Calibration and promotion gates | [SPEC-0016](UNSAFE-REVIEW-SPEC-0016-fixtures-calibration-support.md) | Add fixture metadata and support-tier consistency checks. |

These backlog rows are intentionally high-level. Detailed work sequencing belongs
in `plans/`, while behavioral requirements belong in the individual specs.
