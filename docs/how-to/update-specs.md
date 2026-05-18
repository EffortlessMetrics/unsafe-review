# How to update the specification set

Use this guide when product behavior changes or a new durable contract is needed.

## Choose the document type

- Add or edit a proposal when the product reason, alternatives, risks, or success
  criteria change.
- Add or edit a spec when observable behavior, schemas, policy semantics, or CI
  proof obligations change.
- Add or edit an ADR when the team makes a durable architecture choice.
- Add or edit a plan when the work needs a PR-sized sequence and proof commands.

## Preserve source-of-truth boundaries

Specs say what must happen. Proposals and ADRs explain why. Plans say how the
work will land. Policy files record configured exceptions and baselines.

When a spec needs context, link to the proposal or ADR instead of copying the
rationale into the spec.

## Update indexes

Add new files to their index:

- proposals: [`../proposals/README.md`](../proposals/README.md)
- specs: [`../specs/README.md`](../specs/README.md)
- ADRs: [`../adr/README.md`](../adr/README.md)

`cargo xtask check-pr` fails if a proposal, spec, or ADR file exists without a
matching index entry.

## Verify

```bash
cargo xtask check-pr
```
