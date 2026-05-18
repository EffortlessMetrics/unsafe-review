# Support tiers

All tiers describe static review evidence. None means memory-safety proof.

| Capability | Tier | Surface | Proof | Known limits |
|---|---|---|---|---|
| Diff unsafe site inventory | experimental | CLI JSON/human | syntax-backed fixture goldens for unsafe blocks, split unsafe blocks, raw pointer operations, and negative safe-code cases | source-based, not MIR |
| Review-card JSON schema | experimental | CLI JSON | serde-backed DTOs, `schema_version`, top-level trust boundary, site visibility/public API surface fields, and `fixture_card_goldens_match_rendered_json` | fixture corpus is still small; no schema compatibility promise yet |
| Review-card identity | experimental | card `id` | `card_identity` tests cover line drift and duplicate counted identities | baseline and suppression policy do not consume identities yet |
| Raw pointer card slice | experimental | cards | `raw_pointer_alignment`, `raw_pointer_deref`, `raw_pointer_read_unaligned`, `raw_pointer_write_assignment`, `split_raw_pointer_read_call`, `split_unsafe_block`, and safe-reference negative fixtures | source-level review evidence only |
| Core operation smoke slice | experimental | cards | `maybeuninit_assume_init`, `vec_set_len`, `transmute_invalid_value`, `get_unchecked_mut_bounds`, and `pin_new_unchecked` fixture goldens | curated fixtures, not broad semantic proof |
| Contract evidence mining | experimental | cards | public unsafe fn/trait fixtures and private helper `SAFETY:` fixture | comment quality is heuristic |
| Guard evidence mining | experimental | cards | raw-pointer alignment and comment-not-guard fixtures prove bounds evidence does not discharge alignment | obligation-specific patterns are still sparse |
| Witness routing | experimental | cards | route-table tests plus raw pointer, FFI, unsafe impl Send, Pin, and invalid-value fixture routes | route recommendation only; no witness receipts |
| Repo inventory | scaffold | repo JSON / badges | compile gate only | badge is not UB-free claim |
| PR Markdown summary | experimental | PR artifact Markdown | `pr_summary` renderer tests, CLI `--format pr-summary`, CLI e2e, and advisory workflow upload | advisory artifact only; no comments or blocking policy |
| SARIF projection | experimental | PR artifact SARIF | `sarif` renderer tests, CLI `--format sarif`, CLI e2e, and advisory workflow upload | advisory static review evidence; no default blocking |
| Advisory PR workflow | experimental | GitHub Actions artifacts | workflow renders cards JSON, PR summary, SARIF, and comment plan; runs `cargo xtask check-advisory-artifacts target/unsafe-review` before upload; downloaded artifacts are verified for cards JSON trust boundary and projection card identity consistency | no comments, witnesses, or blocking policy |
| Inline comment plan | experimental | PR artifact JSON | `comment_plan` renderer tests, CLI `--format comment-plan`, CLI e2e, and advisory workflow upload | artifact-only; no posting by default |
| Saved LSP JSON projection | experimental | `--format lsp` JSON | `lsp_projection` renderer tests and CLI e2e cover read-only diagnostics, hovers, and copy-command action data projected from ReviewCards | no LSP server, no editor integration, no source edits |
| LSP server/editor integration | planned | editor | saved-card fixtures | read-only first |
| Agent packets | experimental | `context <card-id> --json` | `agent_packet` renderer tests and CLI e2e cover bounded read-only packets projected from ReviewCards | copy-only; no agent execution, source edits, comments, witness execution, or repair success claim |
| Receipt import | planned | witness receipts | Miri/careful/sanitizer fixtures | receipt strength must be explicit |
| MIR/nightly facts | deferred | optional adapter | ADR needed | not v0.1 product default |
