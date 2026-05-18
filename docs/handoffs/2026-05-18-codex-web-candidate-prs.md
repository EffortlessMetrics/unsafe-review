# Codex web candidate PR intake

Date: 2026-05-18
Status: active candidate inventory
Base snapshot: `main` at `27b4c46`

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

## Current active lane

PR/CI projection from existing review cards.

Already landed in this lane:

- PR summary artifact
- SARIF artifact
- advisory GitHub workflow
- inline comment planning artifact
- scanner false-positive hardening that protects PR/LSP/agent projections

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
| Scanner false-positive hardening | #29, #32 | reworked declaration-prefix slice from #29/#32 | merge | current hardening | PR #27 merged the broad class; this follow-up extracts the remaining syntax-declaration false-positive guard without reopening broad scanner changes. |
| Scanner partial-parse recovery | #48 | reworked from #48 on current main | merge | current hardening | The current slice keeps syntax-backed concrete operation detection available when unrelated parse errors exist, which reduces fake unknown wrapper cards in PR artifacts. |
| Raw pointer write detection | #49, #62 | reworked from #49/#62 on current main | merge | current hardening | The current slice keeps the syntax-target behavior and fixture proof, but rebuilds them narrowly on current main as raw pointer assignment-write detection. |
| xtask fixture validation | #33, #50, #63, #64 | reworked from #64 on current main | merge | current hardening | Fixture validation protects the support-tier proof mechanism. The reworked slice validates fixture layout, golden JSON shape, diff shape, and package naming without broad xtask policy changes. |
| CLI e2e coverage | #39, #58, #80, #81 | reworked from #81 on current main | merge | current PR/CI projection | The current slice keeps #81's user-path shape but updates it for landed PR artifacts: JSON, PR summary, SARIF, comment plan, context, and explain. |
| Focused unit coverage | #44, #61, #86, #87 | #86 | merge | current hardening | #86 is the canonical core-only slice for classifier, evidence, and diff parser invariants. Keep broader CLI parser coverage for the CLI e2e/ergonomics queue. |
| Property testing | #43, #60, #84, #85 | none yet | park | later hardening | Valuable later, but do not put property infrastructure ahead of current PR projection unless it is tiny and directly protects a current parser invariant. |
| Fuzzing | #42, #59, #82, #83 | none yet | park | later hardening | Keep as candidate inventory. Avoid scheduled or blocking fuzz workflows in the current lane. |
| Mutation testing | #41, #57, #78, #79 | none yet | park | later hardening | Useful after the card and PR artifact surfaces settle. Keep non-blocking and manual/scheduled later. |
| CLI ergonomics and diff handling | #31, #45, #46, #47 | none yet | park or rework | later UX | Review only if it fixes a current workflow bug. Avoid broad CLI semantics changes during projection hardening. |
| Documentation usage guides | #36, #37, #53, #56, #72, #73, #76, #77 | choose one only after doc-map review | park or rework | later docs | Pick one canonical CLI usage guide. Avoid multiple overlapping docs pages. |
| Diataxis docs structure | #35, #54, #70, #71 | none yet | park | later docs | Broad docs restructuring is not active-lane work. |
| Spec expansion | #38, #52, #68, #69 | none yet | park | later source-of-truth | Specs should follow concrete behavior gaps, not outrun implementation. |
| CI hardening | #34, #51, #65, #66 | reworked from #66 on current main | merge | current hardening | The current slice keeps the narrow workflow reliability pieces: read-only permissions, no persisted checkout credentials, locked Cargo commands, docs build, timeout, manual dispatch, and PR-run cancellation. |
| Broad module refactors | #40, #55, #74, #75 | none yet | park | later refactor | Avoid broad SRP churn unless it directly unblocks a reviewed implementation slice. |
| Public JSON/visibility API | #28 | reworked from #28 on current main | merge | current hardening | `UnsafeSite` already tracked visibility and public API surface; the reworked slice projects those fields into JSON and updates fixture goldens. |
| Unaligned raw pointer read behavior | #30 | reworked from #30 on current main | merge | current hardening | The current slice keeps the useful distinction that `read_unaligned` does not require alignment evidence while preserving other raw pointer read obligations. |

## Immediate intake order

1. Complete the reworked xtask fixture-validation slice that was extracted from
   #64.
2. Complete the focused unit-coverage slice from #86.
3. Complete the reworked CLI e2e artifact slice from #81.
4. Complete the reworked raw pointer write slice from #49/#62.
5. Complete the minimal CI hardening slice reworked from #66.
6. Park property, fuzz, mutation, broad docs, and broad refactor PRs until their
   target lane opens.

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
