# Dogfood-calibrated evidence loop v0.6 closeout

Date: 2026-05-18
Status: experimental evidence loop closed; broader calibration continues
Owner: core/status

## What closed

This handoff closes the dogfood-calibrated evidence lane as an experimental,
repeatable evidence loop:

```text
scan fixture or selected real-crate target
-> emit ReviewCards
-> verify artifacts
-> compare saved outcomes
-> audit scoped receipts
-> report repo and policy posture
-> record support posture without policy promotion
```

This does not close the overall product objective. `unsafe-review` remains an
experimental static unsafe-review evidence router, not a UB prover, Miri
replacement, security scanner, safety badge, or policy gate.

## Landed PRs

Core lane PRs:

- `#218 docs: define dogfood-calibrated evidence lane`
- `#219 docs: close stale candidate queue for current lane`
- `#220 dogfood: add real-crate corpus manifest`
- `#221 schema: pin outcome comparison JSON`
- `#222 outcome: explain card movement reasons`
- `#224 receipts: audit witness receipt matching`
- `#225 outcome: classify witness receipt movement`
- `#226 repo: render posture markdown`
- `#227 policy: add advisory no-new-debt report`
- `#228 docs(status): add support posture summary`

Related adjacent readiness work:

- `#223 release: prepare initial crates.io publication`

The release-readiness work is useful for publication, but publication, tagging,
and post-publish receipts are separate operations.

## Done Criteria Audit

| Criterion | Evidence | Status |
|---|---|---|
| Dogfood has a manifest-backed corpus of selected real crates and PR diffs | `dogfood/corpus.toml` is validated by `cargo xtask check-dogfood`; current `check-pr` reports 24 targets across 6 repositories | Done |
| Dogfood artifacts are mechanically validated | `cargo xtask check-pr` includes `check-dogfood` | Done |
| Saved-snapshot outcome JSON and Markdown are pinned and explain movement | `#221` pinned outcome JSON; `#222` added movement reasons; support tiers list outcome renderer/e2e proof | Done |
| Receipt matching reports matched, unmatched, expired, stale, wrong identity, wrong tool, and weaker receipts | `#224` added `receipt audit` JSON/Markdown and core/e2e proof | Done |
| Outcome comparison reports receipt-strength movement without overclaiming | `#225` added saved witness receipt strength movement classification | Done |
| Repo inventory JSON and Markdown are pinned for later posture reporting | `#226` added repo posture Markdown on top of repo JSON and badges | Done |
| Advisory no-new-debt can emit a non-blocking policy report | `#227` added `policy report` JSON/Markdown; it is advisory-only and does not change `--policy no-new-debt` enforcement | Done |
| Support tiers distinguish fixture-backed, dogfood-backed, and calibrated surfaces | `#228` added `docs/status/SUPPORT_SUMMARY.md` and kept `SUPPORT_TIERS.md` as the detailed ledger | Done |
| No output claims soundness, UB-free status, Miri-clean status, target-feature availability, site execution, or policy readiness without exact evidence | Trust-boundary text remains in support tiers, artifacts, handoffs, and the support summary | Done for this lane |

## Current Support Posture

The lane supports these statements:

- `unsafe-review` can run fixture and selected real-crate dogfood targets.
- The tool can emit `ReviewCard`s and project them into advisory PR, saved LSP,
  bounded agent, repo posture, receipt audit, outcome, and policy-report
  surfaces.
- Receipt import and audit are scoped metadata paths; they do not execute
  witness tools.
- Outcome comparison is saved-snapshot comparison; it does not rerun analysis or
  make a policy decision.
- Repo posture and badge JSON count open unsafe-review gaps, not raw unsafe
  usage and not safety status.
- `policy report` is advisory-only. Explicit `--policy no-new-debt` remains
  opt-in and separate.

The lane must not be used to claim:

- memory safety
- UB-free status
- Miri-clean status
- target-feature availability
- site execution without a receipt
- calibrated precision or recall
- default no-new-debt or blocking readiness
- release publication

## Proof Commands

The closeout PR should pass:

```bash
rtk cargo fmt --check
rtk cargo run --locked -p xtask -- check-pr
rtk git diff --check
```

Use the broader proof pass before release or policy promotion:

```bash
rtk cargo check --workspace --all-targets --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo run --locked -p xtask -- check-calibration
```

## Known Limits

- The dogfood corpus is selected and capped; it is not ecosystem-wide
  calibration.
- The fixture manifest is a proof index, not real-world precision/recall.
- Receipt adapters import saved outputs only and do not run tools.
- Outcome comparison reads saved snapshots only.
- Repo posture is advisory and not release-grade governance.
- Exact baseline and suppression matching can still be brittle under broader
  code movement.
- No default blocking policy is justified.

## Next Useful Work

Keep the next work narrow:

- decide whether to execute the initial crates.io publication as a separate
  manual release operation
- after publication, tighten install and quickstart smoke docs
- dogfood outcome comparison and explicit receipts on more real PRs
- measure card usefulness and false-positive notes on more unsafe-heavy crates
- preserve exact-card matching, visible limitations, and advisory-only policy

Defer:

- default blocking CI
- automatic comments
- automatic source edits
- witness execution by default
- broad suppressions
- calibrated support-tier promotion
