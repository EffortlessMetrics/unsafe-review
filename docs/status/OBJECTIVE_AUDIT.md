# Objective audit

Date: 2026-05-18
Status: active objective partially achieved; continue dogfood measurement before
any release or policy promotion

This audit maps the current product objective to concrete repo evidence. It is a
status artifact, not a support-tier promotion. `docs/status/SUPPORT_TIERS.md`
remains the authority for public claim wording.

## Objective

`unsafe-review` should be the cheap PR-time unsafe contract reviewer for Rust:
it identifies changed unsafe-adjacent seams, emits `ReviewCard`s, projects those
cards into review/editor/agent/repo surfaces, and routes reviewers to the
cheapest credible next witness without claiming soundness or running expensive
witnesses by default.

## Evidence Checklist

| Requirement | Current evidence | Status | Gap |
|---|---|---|---|
| Canonical product unit is `ReviewCard`; projections must not create parallel truth | JSON, PR summary, SARIF, comment-plan, saved LSP, agent packet, repo, badge, policy, and receipt surfaces are all listed in `SUPPORT_TIERS.md` as card projections; handoffs record lane boundaries | Experimental | Continue watching new surfaces for reclassification logic |
| Card correctness before breadth | Fixture goldens cover raw pointer alignment/deref/read/write, split syntax, public unsafe contracts, `MaybeUninit`, `Vec::set_len`, `transmute`, `get_unchecked_mut`, `Pin::new_unchecked`, FFI, unsafe impl Send, and negative safe/comment cases; `fixtures/calibration.toml` indexes the core positive, negative, and false-positive-control claims | Experimental | Fixture corpus is curated; no broad semantic proof |
| Obligation-level evidence | `ReviewCard` output and fixture goldens distinguish contract, discharge, reach, and witness evidence per obligation | Experimental | Guard patterns remain sparse |
| Length guard does not discharge alignment; comments do not count as guards | Raw-pointer alignment and comment-not-guard fixtures are listed as proof in support tiers | Experimental | More real-world guard idioms need calibration |
| Stable-first implementation; no mandatory MIR or `rustc_private` | Workspace uses stable source parsing and `ra_ap_syntax`; support tiers mark MIR/nightly facts as deferred | Met for current lanes | Optional adapters still need ADR before promotion |
| Advisory PR artifact loop | Handoff `2026-05-18-advisory-pr-artifacts-v0.2.md` records cards JSON, PR summary, SARIF, and comment-plan artifact proof plus in-workflow artifact verification | Experimental/dogfoodable | No automatic comments or blocking policy by design |
| Saved IDE projection | Handoff `2026-05-18-lsp-agent-projection-v0.3.md` records `--format lsp` saved diagnostics, hovers, and copy-command data | Experimental | No live LSP server or editor extension |
| Bounded LLM packet | Handoff `2026-05-18-lsp-agent-projection-v0.3.md` records `context <card-id> --json` bounded packet proof | Experimental | Copy-only; no automated repair or source edits |
| Repo posture and badges count open review gaps, not raw unsafe or safety status | Handoff `2026-05-18-repo-policy-v0.4.md` and support tiers cover repo JSON, badge JSON, and saved-snapshot outcome comparison | Experimental | Not release-grade posture or calibrated governance |
| Baselines and suppressions use exact counted identity | Repo policy handoff records exact baseline/suppression matching and explicit no-new-debt mode | Experimental | Exact identity only; no broad suppressions; no calibrated blocking |
| Witness routing recommends cheap next action | Support tiers cover route-table tests and fixture routes for raw pointer, FFI, unsafe impl Send, Pin, and invalid-value cases | Experimental | Recommendation only unless a receipt is attached |
| Witness receipts attach external evidence without executing tools | Receipt docs and tests cover exact-card JSON import, metadata validation, tool/strength validation, DTO shape, template, validate command, Miri saved-output adapter, cargo-careful saved-output adapter, sanitizer saved-output adapter, Loom/Shuttle saved-output adapter, Kani/Crux proof saved-output adapter, and witness-plan output | Experimental | Saved-output adapters read success logs only; no witness tool is executed by `unsafe-review` |
| Explicit receipts can be authored and validated safely | `receipt template` and `receipt validate` are covered by CLI e2e tests and support tiers | Experimental | Template output does not verify that the recorded command ran |
| Public claims map to proof | `SUPPORT_TIERS.md` maps every current surface to proof and limits | In place | Keep updating for every new lane |
| No soundness, UB-free, Miri-clean, site-execution, or default-blocking claim | Trust-boundary text is enforced across artifacts; support tiers and handoffs repeat limits | In place | Must remain part of all new projections |
| First real-crate dogfood measurement | Handoff `2026-05-18-real-crate-dogfood-v0.6.md` records top-50 capped `rust-smallvec`, `arrayvec`, and `memchr` runs; dogfood found and fixed import/declaration false positives, `cfg(target_feature)` false positives, and capped repo scan timeout behavior | Experimental | More crates, real PR diffs, uncapped/sampled runs, and human review are still needed before calibration claims |

## Current Gaps

These are not failures; they are the next unsupported or weakly verified areas:

- Live LSP server and editor extension remain planned.
- The first saved-output adapters only import saved Miri, cargo-careful, sanitizer,
  Loom, Shuttle, Kani, and Crux success logs.
- Witness tools are not executed by `unsafe-review`, and no lane should add
  default execution without a separate plan.
- Schema compatibility is not yet a public promise.
- Broader calibration on real unsafe-heavy crates is still needed before any
  support tier promotion toward usable alpha. The first dogfood slice covered
  three top-50 capped repo snapshots; the fixture calibration
  manifest remains a proof index, not real-world calibration.
- No default no-new-debt or blocking branch-protection policy is justified yet.
- Outcome comparison is saved-snapshot only and still needs dogfood on real
  repo posture snapshots.

## Current Gates

Use these commands for a broad local proof pass:

```bash
rtk cargo fmt --check
rtk cargo check --workspace --all-targets --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk cargo run --locked -p xtask -- check-pr
rtk cargo run --locked -p xtask -- check-calibration
rtk git diff --check
```

Targeted proof commands added by recent receipt work:

```bash
rtk cargo test -p unsafe-review-core receipt --locked
rtk cargo test -p unsafe-review-core imported_receipt --locked
rtk cargo test -p unsafe-review-cli receipt_template --locked
rtk cargo test -p unsafe-review-cli receipt_validate --locked
rtk cargo test -p unsafe-review --test e2e receipt_template --locked
rtk cargo test -p unsafe-review --test e2e receipt_validate --locked
```

## Recommended Next Lane

Continue dogfood measurement before policy promotion:

1. Run `unsafe-review` on more selected real unsafe-heavy crates and record
   false-positive and false-negative notes.
2. Measure card usefulness on real PR diffs, not only repo snapshots.
3. Dogfood explicit receipts and outcome comparison on real unsafe-review PRs.
4. Preserve exact-card matching, visible limitations, and advisory-only policy.
5. Keep support tiers experimental until dogfood and calibration justify a
   stronger claim.
