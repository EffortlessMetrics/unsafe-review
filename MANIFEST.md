# unsafe-review workspace manifest

Generated: 2026-05-17

## Included

- Rust 2024 / MSRV 1.95 workspace
- `unsafe-review` product façade crate
- `unsafe-review-cli` CLI adapter crate
- `unsafe-review-core` SDK and static analyzer crate
- `xtask` automation crate
- full proposal/spec/ADR/status/plan documentation system
- policy ledgers for unsafe-review, Clippy, no-panic, non-Rust, generated, executable, workflow, process, and network surfaces
- GitHub workflow and settings scaffold
- raw-pointer-alignment fixture

## Implemented analyzer capabilities

- source-only unsafe seam scanning
- unified diff parsing
- repo inventory mode
- hazard taxonomy and safety obligation mapping
- `# Safety` / `SAFETY:` contract evidence mining
- simple guard evidence mining
- static test mention reachability
- witness route suggestions for Miri, cargo-careful, sanitizers, Loom, and human review
- human / JSON / Markdown output
- badge JSON output
- context packet output scaffold

## Known limitations

- not compiled in this environment because Rust/Cargo are unavailable
- no MIR or `rustc_private` integration
- baseline/suppression matching is specified but not yet implemented
- SARIF/LSP/receipt import are specified but not yet implemented
- static reachability is heuristic and should not be treated as execution proof
