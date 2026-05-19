# UNSAFE-REVIEW-SPEC-0006: Contract and discharge evidence

Status: accepted
Owner: core/spec
Created: 2026-05-17
Updated: 2026-05-19
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Contract

`unsafe-review` must classify evidence into explicit lanes and apply operand/receiver-sensitive rules before counting discharge evidence.

## Evidence lanes

- `contract`: docs, `# Safety`, `SAFETY:` comments, unsafe API precondition text.
- `discharge`: local executable guards, matching wrappers, matching constructor/privacy boundaries.
- `reach`: static relation from tests/harness inventory only.
- `witness`: imported receipt evidence only.

## Matching / precedence rules

1. Contract evidence never auto-discharges obligations by itself.
2. Discharge evidence must match target operand/receiver/buffer identity when the rule is identity-sensitive.
3. Later checks do not retroactively discharge earlier unsafe operations.
4. Non-returning error branches do not count as guard discharge.

## Counts as evidence

| Evidence rule | Counts when | Does not count when | Fixture |
|---|---|---|---|
| Alignment guard | executable check like `is_aligned`, `align_offset`, modulo/equality check over same pointer before use | bare `align_of` mention; comment text; unrelated pointer | `align_of_only_not_guard` |
| NonNull guard | `NonNull::new(ptr)` or equivalent non-null check for the same pointer then `new_unchecked(ptr)` | guard applies to different pointer | `nonnull_new_guard`, `nonnull_new_guard_other_pointer` |
| UTF-8 validation | `str::from_utf8(buf).is_ok()` (or equivalent returning error path) before `from_utf8_unchecked(buf)` | different buffer; non-returning error branch | `str_from_utf8_validation` |
| unwrap_unchecked state | same receiver has pre-check (`is_some` / `is_ok`) on the dominating path | other receiver; check after unchecked call | `unwrap_unchecked_is_some_guard` |
| Bounds guard | `len/capacity` relation executable and relevant to operation family | unrelated length variable; comment-only claim | `vec_set_len`, `raw_pointer_read` |

## Does not count

- Comments as discharge evidence.
- Policy receipt metadata as contract/discharge substitution (receipts only populate witness lane).
- Family-incompatible rules (e.g., requiring alignment for `write_unaligned`).

## Fixtures

Every evidence rule in this spec must name at least one positive and one negative fixture (or explicit limitation).

## Output examples

```json
{
  "contract_evidence": ["SAFETY comment explains pointer provenance"],
  "discharge_evidence": ["ptr alignment checked with addr % align == 0"],
  "missing_obligations": ["initialized_memory"],
  "limitations": ["no witness receipt imported"]
}
```

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Rule changes are promotable only when fixture/golden coverage includes at least one “does not count” case proving false-positive control.
