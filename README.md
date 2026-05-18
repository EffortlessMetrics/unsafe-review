# unsafe-review

`unsafe-review` is a cheap PR-time review pass for unsafe Rust.

It scans changed unsafe-adjacent code, identifies the safety conditions that matter,
checks whether those conditions are documented, locally discharged, reached by tests,
and routed to an appropriate witness such as Miri, `cargo-careful`, sanitizers, Loom,
Shuttle, Kani, or Crux. It then emits one focused review card per gap.

```text
Miri:
  Did this concrete execution hit UB?

unsafe-review:
  Does this unsafe change have the safety contract, guard, test reach,
  and witness route needed to make review credible?
```

## Trust boundary

`unsafe-review` is **static unsafe contract review**. It reports missing or present
review evidence. It is not a proof of memory safety, not a claim that the repository is
UB-free, and not a Miri result unless a witness receipt is attached.

## Quick start

```bash
cargo install unsafe-review

# Review the current diff against origin/main
unsafe-review check --base origin/main

# Fail only when explicit no-new-debt mode finds unbaselined actionable gaps
unsafe-review check --base origin/main --policy no-new-debt

# Review a supplied unified diff from a file or stdin
unsafe-review check --diff change.diff --format json
git diff origin/main...HEAD | unsafe-review check --diff - --format json

# Write a sparse GitHub-ready PR summary artifact
unsafe-review check --base origin/main \
  --format pr-summary \
  --out target/unsafe-review/pr-summary.md

# Write SARIF for code scanning upload or CI artifacts
unsafe-review check --base origin/main \
  --format sarif \
  --out target/unsafe-review/cards.sarif

# Plan high-signal inline review comments without posting them
unsafe-review check --base origin/main \
  --format comment-plan \
  --out target/unsafe-review/comment-plan.json

# Write a read-only saved editor/LSP projection
unsafe-review check --base origin/main \
  --format lsp \
  --out target/unsafe-review/lsp.json

# Try the bundled smoke fixture
unsafe-review check --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff \
  --format json

# Static repo inventory and badge JSON; not a safety badge
unsafe-review repo --format json
unsafe-review badges --out badges/

# Explain one card and produce an experimental context packet
unsafe-review explain UR-src-lib-rs-42-raw-pointer-read
unsafe-review context UR-src-lib-rs-42-raw-pointer-read --json
```

## Current implementation status

This workspace includes the specification system, fixture-backed experimental
review cards, and an advisory PR artifact loop. The analyzer is intentionally
conservative:

- no `rustc_private`
- no MIR dependency
- no automatic source edits
- no default blocking
- no soundness claims

The current static engine detects common unsafe seams and operations from source text,
maps them to hazard classes and safety obligations, mines nearby `# Safety` / `SAFETY:`
contract evidence, looks for simple local guards, and routes cards to likely witnesses.

The current PR projection renders `cards.json`, `pr-summary.md`, `cards.sarif`, and
`comment-plan.json` as advisory artifacts. It does not post comments, run witness
tools, or enable blocking policy. Support tiers stay conservative: fixture-backed
surfaces are experimental, the saved editor/LSP projection and bounded agent
packets are read-only and experimental, repo posture, exact policy-ledger
matching, explicit no-new-debt mode, and exact witness receipt import are
experimental. Witness tools are not executed by default, and blocking policy plus
broad suppression controls remain later lanes until they have proof.

## Crate surface

```text
unsafe-review          # product facade / install handle
unsafe-review-cli      # CLI adapter and rendering
unsafe-review-core     # SDK / analysis engine
xtask                  # repo automation, not product surface
```

The crate boundary policy is: design seams like microcrates, implement most as module
families, and publish only seams that deserve a support promise.

## Development

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask check-pr
```

## Documentation map

- [Mission and vision](docs/MISSION.md)
- [Roadmap](docs/ROADMAP.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Specifications](docs/specs/README.md)
- [ADRs](docs/adr/README.md)
- [Implementation plan](plans/0.1.0/implementation-plan.md)
- [Support tiers](docs/status/SUPPORT_TIERS.md)
- [Policy ledgers](policy/)
