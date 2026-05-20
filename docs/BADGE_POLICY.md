# Badge policy

`unsafe-review` badges are public evidence signals, not safety claims.

## Principles

- Badges are advisory and repo-scoped.
- Badges summarize generated evidence surfaces where possible.
- No badge implies memory safety, soundness, UB-freedom, or Miri execution.

## Meaning table

| Badge | Meaning | Not meaning |
|---|---|---|
| CI | Current GitHub Actions CI status. | Analyzer correctness or safety proof. |
| Codecov | Uploaded coverage signal when a coverage workflow publishes. | Test adequacy, completeness, or safety proof. |
| `ripr+` | Static oracle-exposure evidence imported from `ripr` posture. | Mutation testing execution or runtime mutation confirmation. |
| GitHub release | Latest published GitHub release tag. | crates.io availability or release quality proof. |
| crates.io downloads | Public crates.io download count for the install crate. | Adoption quality or safety proof. |
| docs.rs | Current docs.rs build badge for the install crate. | API stability guarantee. |
| `unsafe-review` | Open static review gaps in repo posture. | Safety/unsafety status. |
| `unsafe-review+` | Contract/guard/witness gap summary posture. | Miri-clean status or formal proof. |
| VS Code planned | Editor extension is planned and documented. | Published marketplace extension. |
| Open VSX planned | Editor extension is planned and documented. | Published Open VSX extension. |
| MSRV | Declared minimum supported Rust version. | Toolchain-wide compatibility guarantee. |
| License | Declared project license expression. | Legal advice. |

Badge endpoints are repo-scoped static evidence projections from ReviewCards.
They are not safety badges.
They must be generated or checked by `xtask`.

## Generation contract

When present, Shields endpoint JSON under `badges/` is generated content and must
be checked, not hand-edited.

- Generate/refresh endpoint files: `cargo run --locked -p xtask -- badges`
- Validate endpoint files and README links: `cargo run --locked -p xtask -- badges --check`
- Run the repo gate: `cargo run --locked -p xtask -- check-pr`

Until endpoint JSON is generated and covered by the validation path above,
endpoint badges should not be added to README rows.
