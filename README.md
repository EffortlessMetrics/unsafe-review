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

The tool is intentionally advisory-first: it helps reviewers ask sharper questions,
collect missing evidence, and route risky changes to stronger tools. Treat every card
as a review prompt, not as a vulnerability report or a soundness verdict.

## What the tool looks for

A useful unsafe review usually needs four kinds of evidence:

| Evidence | Examples `unsafe-review` recognizes | Why it matters |
|---|---|---|
| Contract | `# Safety` docs, nearby `SAFETY:` comments | States the invariant the unsafe code relies on. |
| Local discharge | nearby assertions, bounds checks, alignment checks, null checks | Shows the precondition is checked close to the operation. |
| Test reach | tests or fixtures that reach the changed unsafe seam | Keeps the review from relying only on static inspection. |
| Witness route | Miri, `cargo-careful`, sanitizers, Loom, Shuttle, Kani, Crux | Sends the right hazard class to a tool that can exercise it. |

The current analyzer is conservative and source-oriented. It favors clear review gaps
over broad unsoundness claims.

## Quick start

```bash
# From a checkout of this repository
cargo run -p unsafe-review -- check --base origin/main

# After installation from crates.io or a local path
cargo install unsafe-review
unsafe-review check --base origin/main
```

Review a supplied unified diff:

```bash
unsafe-review check --diff change.diff --format json
```

Try the bundled smoke fixture:

```bash
unsafe-review check --root fixtures/raw_pointer_alignment \
  --diff fixtures/raw_pointer_alignment/change.diff \
  --format json
```

Generate a whole-repository inventory and badge data:

```bash
unsafe-review repo --format json
unsafe-review badges --out badges/
```

Explain one card and produce an LLM-ready packet:

```bash
unsafe-review explain UR-src-lib-rs-42-raw-pointer-read
unsafe-review context UR-src-lib-rs-42-raw-pointer-read
```

## Common review workflow

1. **Run on the PR diff.** Use `unsafe-review check --base origin/main` locally or in CI.
2. **Read the card summary first.** Prioritize missing contracts, missing guards, and
   guarded-but-unwitnessed seams.
3. **Fix evidence, not the card.** Add or improve `# Safety` docs, tighten local guards,
   add a targeted test, or route the seam to the recommended witness.
4. **Attach receipts.** If a witness ran, keep the command, status, and relevant output
   with the review or CI artifact.
5. **Escalate intentionally.** Only move from advisory to no-new-debt or blocking once
   the support tier for that claim is proven by fixtures and golden tests.

## Command reference

| Command | Purpose | Useful options |
|---|---|---|
| `check` | Analyze changed unsafe-adjacent code. | `--root`, `--base`, `--diff`, `--format`, `--out`, `--max-cards` |
| `repo` | Analyze the repository inventory instead of a diff. | `--root`, `--format`, `--out`, `--max-cards` |
| `pilot` | Run a capped advisory PR pass for early adoption. | `--root`, `--base`, `--diff`, `--max-cards` |
| `badges` | Write repository badge artifacts. | `--root`, `--out` |
| `explain` | Render detailed markdown or JSON for a card. | `--root`, `--format` |
| `context` | Emit an LLM-ready JSON packet for a card. | `--root` |
| `doctor` | Check whether the repository is ready for analysis. | `--root` |

Supported output formats are `human`, `json`, and `markdown` where the command accepts
`--format`.

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
