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

| Evidence rule | Counts when | Does not count when | Fixture proof |
|---|---|---|---|
| Alignment guard | executable check like `is_aligned`, `align_offset`, modulo/equality check over same pointer before use | bare `align_of` mention; comment text; unrelated pointer | `raw_pointer_alignment`, `align_of_only_not_guard`, `alignment_other_pointer_not_guard`, `comment_alignment_not_guard` |
| NonNull guard | `NonNull::new(ptr)` or equivalent non-null check for the same pointer before `new_unchecked(ptr)` | guard applies to different pointer, a non-returning error branch, or a post-constructor check | `nonnull_new_guard`, `nonnull_other_guard_not_evidence`, `nonnull_is_null_nonreturning_not_guard`, `nonnull_post_check_not_guard` |
| UTF-8 validation | `str::from_utf8(buf).is_ok()`, a returning error path, or `str::from_utf8(buf)?` before `from_utf8_unchecked(buf)` | no validation, validation after use, or validation for a different buffer | `str_from_utf8_unchecked`, `str_from_utf8_unchecked_is_ok_guard`, `str_from_utf8_unchecked_is_err_return_guard`, `str_from_utf8_unchecked_question_mark_guard`, `str_from_utf8_unchecked_post_validation_not_guard`, `str_from_utf8_unchecked_other_buffer_not_guard` |
| unwrap_unchecked state | same receiver has pre-check (`is_some` / `is_ok`) or returning `None` / `Err` path before the unchecked call | other receiver; check after unchecked call; unrelated infallible expression | `unwrap_unchecked_is_some_guard`, `unwrap_unchecked_is_ok_guard`, `unwrap_unchecked_is_none_return_guard`, `unwrap_unchecked_is_err_return_guard`, `unwrap_unchecked_other_infallible_not_guard` |
| Bounds guard | `len/capacity` relation executable and relevant to operation family | unrelated length variable; comment-only claim; post-access check; capacity observation without a guard | `vec_set_len`, `raw_pointer_read_len_capacity_assert`, `get_unchecked_mut_len_guard`, `get_unchecked_mut_other_len_not_guard`, `get_unchecked_mut_post_check_not_guard`, `vec_set_len_capacity_observed_not_guard` |

## Does not count

- Comments as discharge evidence.
- Policy receipt metadata as contract/discharge substitution (receipts only populate witness lane).
- Family-incompatible rules (e.g., requiring alignment for `write_unaligned`).

## Fixtures

Every evidence rule in this spec must name at least one positive and one negative fixture (or explicit limitation).

## Output examples

```json
{
  "obligation_evidence": [
    {
      "key": "alignment",
      "description": "pointer is aligned for the accessed type",
      "contract": {"present": true, "state": "present", "summary": "SAFETY comment explains alignment contract"},
      "discharge": {"present": false, "state": "missing", "summary": "No alignment guard code was detected"},
      "reach": {"present": false, "state": "missing", "summary": "No static test relation found"},
      "witness": {"present": false, "state": "missing", "summary": "No imported witness receipt was found"}
    }
  ],
  "missing": ["alignment evidence is missing"],
  "verify_commands": ["cargo +nightly miri test"]
}
```

## CI proof

```bash
cargo xtask check-pr
cargo test --workspace
```

## Promotion rule

Rule changes are promotable only when fixture/golden coverage includes at least one "does not count" case proving false-positive control.
