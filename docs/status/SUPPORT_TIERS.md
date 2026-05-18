# Support tiers

All tiers describe static review evidence. None means memory-safety proof.

| Capability | Tier | Surface | Proof | Known limits |
|---|---|---|---|---|
| Diff unsafe site inventory | experimental | CLI JSON/human | syntax-backed fixture goldens for unsafe blocks, split unsafe blocks, raw pointer operations, and negative safe-code cases | source-based, not MIR |
| Review-card JSON schema | experimental | CLI JSON | serde-backed DTOs, `schema_version`, top-level trust boundary, site visibility/public API surface fields, and `fixture_card_goldens_match_rendered_json` | fixture corpus is still small; no schema compatibility promise yet |
| Review-card identity | experimental | card `id` | `card_identity` tests cover line drift and duplicate counted identities | exact identity is consumed by baselines, suppressions, and receipts, but broader drift behavior still needs dogfood |
| Advisory baseline/suppression matching | experimental | policy ledgers / card class | xtask ledger tests plus analyzer tests for exact `baseline_known` and `suppressed` card identity matches | exact identity only; no broad suppressions and no blocking policy |
| Explicit no-new-debt mode | experimental | `--policy no-new-debt` | CLI parser tests and e2e cover nonzero exit for unbaselined actionable gaps and success for exact baseline matches | opt-in only; not default, not calibrated blocking, and no broad suppression patterns |
| Raw pointer card slice | experimental | cards | `raw_pointer_alignment`, `raw_pointer_deref`, `raw_pointer_read_unaligned`, `raw_pointer_write_assignment`, `split_raw_pointer_read_call`, `split_unsafe_block`, and safe-reference negative fixtures | source-level review evidence only |
| Core operation smoke slice | experimental | cards | `maybeuninit_assume_init`, `vec_set_len`, `transmute_invalid_value`, `get_unchecked_mut_bounds`, and `pin_new_unchecked` fixture goldens | curated fixtures, not broad semantic proof |
| Contract evidence mining | experimental | cards | public unsafe fn/trait fixtures and private helper `SAFETY:` fixture | comment quality is heuristic |
| Guard evidence mining | experimental | cards | raw-pointer alignment and comment-not-guard fixtures prove bounds evidence does not discharge alignment | obligation-specific patterns are still sparse |
| Witness routing | experimental | cards | route-table tests plus raw pointer, FFI, unsafe impl Send, Pin, and invalid-value fixture routes | route recommendation only; receipt import is a separate surface and does not execute witnesses |
| Witness plan output | experimental | `--format witness-plan` | renderer tests and CLI e2e cover card-sourced route plans and trust-boundary wording | route artifact only; does not execute witnesses or prove witness success |
| Repo inventory and badge JSON | experimental | `repo --format json` / `badges` | CLI e2e covers repo-scope open-gap counts, trust boundary, and badge messages for a fixture card | static open-gap counts only; not calibrated, not a safety badge, and not policy gating |
| PR Markdown summary | experimental | PR artifact Markdown | `pr_summary` renderer tests, CLI `--format pr-summary`, CLI e2e, and advisory workflow upload | advisory artifact only; no comments or blocking policy |
| SARIF projection | experimental | PR artifact SARIF | `sarif` renderer tests, CLI `--format sarif`, CLI e2e, and advisory workflow upload | advisory static review evidence; no default blocking |
| Advisory PR workflow | experimental | GitHub Actions artifacts | workflow renders cards JSON, PR summary, SARIF, and comment plan; runs `cargo xtask check-advisory-artifacts target/unsafe-review` before upload; downloaded artifacts are verified for cards JSON trust boundary and projection card identity consistency | no comments, witnesses, or blocking policy |
| Inline comment plan | experimental | PR artifact JSON | `comment_plan` renderer tests, CLI `--format comment-plan`, CLI e2e, and advisory workflow upload | artifact-only; no posting by default |
| Saved LSP JSON projection | experimental | `--format lsp` JSON | `lsp_projection` renderer tests and CLI e2e cover read-only diagnostics, hovers, and copy-command action data projected from ReviewCards | no LSP server, no editor integration, no source edits |
| LSP server/editor integration | planned | editor | saved-card fixtures | read-only first |
| Agent packets | experimental | `context <card-id> --json` | `agent_packet` renderer tests and CLI e2e cover bounded read-only packets projected from ReviewCards | copy-only; no agent execution, source edits, comments, witness execution, or repair success claim |
| Witness receipt import | experimental | `.unsafe-review/receipts/*.json` | receipt parser tests cover exact identity, strength, author, timestamp, and expiry validation; analyzer tests cover exact-card import; `raw_pointer_alignment_receipted` golden covers rendered card output | imports receipts only; does not execute witnesses, does not prove repository safety, and matches exact card identity only |
| MIR/nightly facts | deferred | optional adapter | ADR needed | not v0.1 product default |
