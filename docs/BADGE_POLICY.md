# Badge policy

`unsafe-review` badges are public evidence signals, not safety claims.

## Principles

- Badges are advisory and repo-scoped.
- Badges summarize generated evidence surfaces where possible.
- No badge implies memory safety, soundness, UB-freedom, or Miri execution.

## Meaning table

| Badge | Meaning | Not meaning |
|---|---|---|
| CI | Current GitHub Actions CI status. | Product correctness or safety proof. |
| GitHub release | Latest published GitHub release tag. | crates.io availability or release quality proof. |
| crates.io downloads | Public crates.io download count for the install crate. | Adoption quality or safety proof. |
| docs.rs | Current docs.rs build badge for the install crate. | API stability guarantee. |
| `unsafe-review` | Open static review gaps in repo posture. | Safety/unsafety status. |
| `unsafe-review+` | Contract/guard/witness gap summary posture. | Miri-clean status or formal proof. |
| VS Code planned | Editor extension is planned and documented. | Published marketplace extension. |
| Open VSX planned | Editor extension is planned and documented. | Published Open VSX extension. |
| MSRV | Declared minimum supported Rust version. | Toolchain-wide compatibility guarantee. |
| License | Declared project license expression. | Legal advice. |

## Generation contract

When present, Shields endpoint JSON under `badges/` is generated content and must
be checked, not hand-edited.

- Generate from an installed or locally built CLI: `unsafe-review badges --out badges/`
- Validate badge behavior: `cargo test -p unsafe-review --test e2e repo_inventory_and_badges_count_open_gaps_without_safety_claim --locked`
- Run the repo gate: `cargo run --locked -p xtask -- check-pr`

Until endpoint JSON is generated and covered by the validation path above,
endpoint badges should not be added to README rows.
