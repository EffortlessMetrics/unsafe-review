# Codex web candidate PR intake

Date: 2026-05-18
Status: candidate intake mostly drained; later-lane candidates parked
Base snapshot: `main` at `fa9584f`

## Operating model

The main agent lane remains sequential and merge-ready. Codex web PRs are
candidate branches, not failures and not active-lane blockers.

Use this intake rule for every candidate:

- Does it improve the current active lane?
- Does it preserve the `ReviewCard` contract?
- Does it avoid new policy authority or default blocking?
- Does it avoid creating another source of truth?
- Is it narrow enough to review?
- Can it be rebased cleanly onto current `main`?

Disposition choices:

- `merge`: review, rebase if needed, validate, and merge now.
- `rework`: useful idea, but split or adjust before merge.
- `park`: useful later-lane branch; leave open or convert to issue.
- `close-duplicate`: close only after a canonical PR for the theme is chosen.

## Current lane context

The original intake snapshot covered PR/CI projection from existing review
cards. The repo has since closed PR/CI projection, saved LSP/agent projection,
repo policy, and first witness receipt import slices. This document remains the
candidate inventory ledger; newer lane handoffs own the current product state.

Already landed in this lane:

- PR summary artifact
- SARIF artifact
- advisory GitHub workflow
- inline comment planning artifact
- scanner false-positive hardening that protects PR/LSP/agent projections
- fixture validation, focused unit coverage, and CLI artifact e2e coverage
- advisory artifact verifier and projection consistency checks
- raw pointer write, unaligned read, and partial-parse scanner hardening
- CLI diff-input hardening for stdin diffs, root-relative diffs, and local `--out`
- repo-mode dogfood false-positive hardening for deref assignments in product code
- CLI parser hardening for `--flag=value` artifact commands and missing values
- core property-test hardening for diff-coordinate and identity-token invariants
- mutation-sensitive test extraction for obligation mappings and scanner scope behavior

Do not use candidate intake as permission to jump to LSP, agent packets, repo
badges, receipts, Miri execution, or blocking policy.

## Candidate labels

Use these labels on open candidate PRs:

- `candidate`
- `candidate:tests`
- `candidate:docs`
- `candidate:ci`
- `candidate:fuzz`
- `candidate:mutation`
- `candidate:refactor`
- `candidate:scanner`
- `candidate:pr-projection`

As of this snapshot, these labels have been created and applied to the open
candidate PRs in the theme inventory below.

## Theme inventory

| Theme | Candidate PRs | Current canonical | Disposition | Target lane | Reason |
|---|---:|---|---|---|---|
| Scanner false-positive hardening | #29, #32 | merged as #100 | closed | current hardening | PR #27 merged the broad class; #100 extracted the remaining syntax-declaration false-positive guard without reopening broad scanner changes. |
| Scanner partial-parse recovery | #48 | merged as #103 | closed | current hardening | #103 keeps syntax-backed concrete operation detection available when unrelated parse errors exist, reducing fake unknown wrapper cards in PR artifacts. |
| Raw pointer write detection | #49, #62 | merged as #95 | closed | current hardening | #95 kept syntax-target behavior and fixture proof, rebuilt narrowly on current main as raw pointer assignment-write detection. |
| xtask fixture validation | #33, #50, #63, #64 | merged as #92 | closed | current hardening | Fixture validation protects the support-tier proof mechanism by validating fixture layout, golden JSON shape, diff shape, and package naming without broad xtask policy changes. |
| CLI e2e coverage | #39, #58, #80, #81 | merged as #94 | closed | current PR/CI projection | #94 kept #81's user-path shape but updated it for landed PR artifacts: JSON, PR summary, SARIF, comment plan, context, and explain. |
| Focused unit coverage | #44, #61, #86, #87 | merged as #93 | closed | current hardening | #93 is the canonical core-only slice for classifier, evidence, and diff parser invariants. |
| Property testing | #43, #60, #84, #85 | merged as #115 | closed | later hardening | #115 extracted the narrow core invariant slice: unified-diff new-file coordinates, removed-only file tracking, slug token stability, and path-display normalization. It added no fuzz workflow, mutation workflow, product surface, or policy authority. |
| Fuzzing | #42, #59, #82, #83 | none yet | park | later hardening | Keep as candidate inventory. Avoid scheduled or blocking fuzz workflows in the current lane. |
| Mutation-sensitive tests | #41, #79 | merged as #119 | closed | current hardening | #119 extracted the current-lane-safe tests for obligation/hazard mapping, concrete-operation suppression, and diff-vs-repo scanner filtering. #41 and #79 were closed as superseded. |
| Mutation workflow/config | #57, #78 | none yet | park | later hardening | Keep optional mutation config and workflow work parked until a later hardening lane. Do not add mutation workflow surface to the advisory PR artifact loop. |
| CLI ergonomics and diff handling | #31, #45, #46, #47 | merged as #104 and #112 | closed | current PR/CI projection | #104 kept current-lane fixes: stdin diffs, root-relative diff files, current-directory `--out`, JSON aliases, duplicate card-id rejection, and no `--fail-on-gaps` policy behavior. #112 extracted the useful current-lane slice from stale #46: `--flag=value` parsing, stricter missing-value handling, and help text aligned with advisory artifact outputs. #46 was closed as superseded. |
| Documentation usage guides | #36, #37, #53, #56, #72, #73, #76, #77 | merged as #143 and #144 | closed except #76 parked | current docs | #143 added the canonical current-surface CLI guide. #144 extracted the crate README link. Direct CLI/usage duplicates #36, #37, #53, #56, #72, #73, and #77 were closed as superseded. #76 remains parked as broader docs-overview work. |
| Diataxis docs structure | #35, #54, #70, #71 | none yet | park | later docs | Broad docs restructuring is not active-lane work. |
| Spec expansion | #38, #52, #68, #69 | none yet | park | later source-of-truth | Specs should follow concrete behavior gaps, not outrun implementation. |
| CI hardening | #34, #51, #65, #66 | merged as #96 | closed | current hardening | #96 kept the narrow workflow reliability pieces: read-only permissions, no persisted checkout credentials, locked Cargo commands, docs build, timeout, manual dispatch, and PR-run cancellation. |
| Broad module refactors | #40, #55, #74, #75 | none yet | park | later refactor | Avoid broad SRP churn unless it directly unblocks a reviewed implementation slice. |
| Public JSON/visibility API | #28 | merged as #101 | closed | current hardening | `UnsafeSite` already tracked visibility and public API surface; #101 projected those fields into JSON and updated fixture goldens. |
| Unaligned raw pointer read behavior | #30 | merged as #102 | closed | current hardening | #102 kept the useful distinction that `read_unaligned` does not require alignment evidence while preserving other raw pointer read obligations. |

## Immediate intake order

1. The current-lane candidate intake is drained through #144 for PR artifacts,
   projection docs, and CLI guide coverage.
2. Direct duplicate CLI/usage docs candidates are closed as superseded by #143
   and #144.
3. Leave fuzz, mutation workflow/config, broad docs, spec expansion, and broad
   refactor candidates parked until their target lanes open.
4. Continue using candidate branches as option inventory. Rebuild useful slices
   narrowly on current `main` instead of merging stale broad branches.

## Review protocol

For each canonical candidate:

```bash
rtk git fetch origin
rtk git switch <candidate-branch>
rtk git rebase origin/main
rtk cargo fmt --check
rtk cargo check --workspace --all-targets
rtk cargo clippy --workspace --all-targets -- -D warnings
rtk cargo test --workspace
rtk cargo xtask check-pr
```

If the branch is valuable but too broad, rebuild the useful slice on a fresh
branch instead of merging it as-is.
