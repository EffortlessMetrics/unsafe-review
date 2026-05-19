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
| Codecov | Uploaded coverage signal from CI artifacts. | Complete test adequacy or proof. |
| `ripr+` | Static mutation-exposure evidence produced by `ripr` policy. | Runtime mutation confirmation, mutation testing score, or coverage. |
| `unsafe-review` | Open static review gaps in repo posture. | Safety/unsafety status. |
| `unsafe-review+` | Contract/guard/witness gap summary posture. | Miri-clean status or formal proof. |
| VS Code planned | Editor extension is planned and documented. | Published marketplace extension. |
| Open VSX planned | Editor extension is planned and documented. | Published Open VSX extension. |
| MSRV | Declared minimum supported Rust version. | Toolchain-wide compatibility guarantee. |
| License | Declared project license expression. | Legal advice. |

## Generation contract

When present, Shields endpoint JSON under `badges/` is generated content and must
be checked, not hand-edited.

- Generate: `cargo run --locked -p xtask -- badges`
- Verify: `cargo run --locked -p xtask -- badges --check`

Until `xtask badges` is implemented, endpoint badges should not be added to
README rows.
