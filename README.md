# unsafe-review

`unsafe-review` is a cheap PR-time review pass for unsafe Rust. It is designed to
make unsafe changes easier to review before they merge, not to prove that a crate
is memory-safe.

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

## What a review card means

A card is a reviewer-facing prompt for one unsafe-adjacent seam. Treat it as a
checklist item, not as a compiler diagnostic:

- **Contract**: is the `# Safety` documentation or nearby `SAFETY:` rationale visible?
- **Discharge**: is there a local guard, assertion, type invariant, or other evidence
  that the contract is enforced at the seam?
- **Reach**: is there a test or fixture that appears to exercise the owner of the seam?
- **Witness**: is the seam routed to a dynamic or formal witness that is appropriate
  for the hazard class?

The intended workflow is to read the card, improve the code or evidence, run the
suggested witness command when it applies, and attach any witness receipt to the
review.

## Quick start

```bash
cargo install unsafe-review

# Confirm the local environment can run the tool.
unsafe-review doctor

# Review the current diff against origin/main.
unsafe-review check --base origin/main

# Review a supplied unified diff and emit machine-readable output.
unsafe-review check --diff change.diff --format json

# Try the bundled smoke fixture.
unsafe-review check --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff \
  --format json

# Full repo inventory and badge data.
unsafe-review repo --format json
unsafe-review badges --out badges/

# Explain one card and produce an LLM-ready context packet.
unsafe-review explain UR-src-lib-rs-42-raw-pointer-read
unsafe-review context UR-src-lib-rs-42-raw-pointer-read
```

## CLI command reference

| Command | Purpose | Common options |
|---|---|---|
| `unsafe-review doctor` | Print basic environment information. | `--root .` |
| `unsafe-review check` | Analyze a diff or, with no diff source, scan the current repo. | `--root .`, `--base origin/main`, `--diff file`, `--format human\|json\|markdown`, `--out file` |
| `unsafe-review pilot` | Analyze a diff with a small default card budget for trial adoption. | `--root .`, `--base origin/main`, `--max-cards 5` |
| `unsafe-review repo` | Analyze the whole repository inventory. | `--root .`, `--format json`, `--out file` |
| `unsafe-review badges` | Write Shields-compatible JSON badge files. | `--root .`, `--out badges` |
| `unsafe-review explain` | Render human-readable detail for a card. | `--root .`, `--format markdown\|json` |
| `unsafe-review context` | Emit the JSON context packet for a card. | `--root .` |

When both `--diff` and `--base` are supplied to `check`, the explicit diff file is
used. `--base` runs `git diff <base>...HEAD` from the selected root. Output defaults
to human text for `check` and Markdown for `explain`.

## Current implementation status

This initial workspace includes the specification system and an experimental
stable-only v0.1 analyzer scaffold. The analyzer is intentionally conservative:

- no `rustc_private`
- no MIR dependency
- no automatic source edits
- no default blocking
- no soundness claims

The current static engine detects common unsafe seams and operations from source text,
maps them to hazard classes and safety obligations, mines nearby `# Safety` / `SAFETY:`
contract evidence, looks for simple local guards, and routes cards to likely witnesses.
It is a scaffold, not a calibrated review signal; the support tiers stay conservative
until schema fixtures and golden tests prove each claim.

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
