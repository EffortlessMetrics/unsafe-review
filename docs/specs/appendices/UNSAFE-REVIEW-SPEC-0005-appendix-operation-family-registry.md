# Operation-family registry appendix

Status: accepted
Owner: core/spec
Updated: 2026-05-19
Normative owner: ../UNSAFE-REVIEW-SPEC-0005-hazard-taxonomy-and-obligations.md

This appendix provides the canonical operation-family table referenced by Spec 0005 for promoted or fixture-backed rows. It is intentionally scoped to the current `ReviewCard` registry and must not invent operation names, hazards, evidence lanes, or witness claims that the implementation does not emit.

## Registry table

| operation_family | detected syntax shapes | hazards | not hazards | obligation / evidence keys | witness route | fixture proof | known false-positive controls | known limits |
|---|---|---|---|---|---|---|---|---|
| `raw_pointer_read` | `ptr.read()`; `core::ptr::read(ptr)` | pointer_validity, alignment, initialized_memory, same_allocation | none | pointer-live, bounds, alignment, initialized, allocation | miri -> cargo-careful | `raw_pointer_alignment`, `split_raw_pointer_read_call`, `raw_pointer_read_len_capacity_assert` | reject comment-only guards; reject bare `align_of`; reject unrelated pointer alignment | macro-expanded forms may degrade to unknown; provenance remains witness-heavy |
| `raw_pointer_read_unaligned` | `ptr.read_unaligned()`; `core::ptr::read_unaligned(ptr)` | pointer_validity, initialized_memory, same_allocation | alignment | pointer-live, bounds, initialized, allocation | miri -> cargo-careful | `raw_pointer_read_unaligned` | reject alignment obligations for this family | cannot prove dynamic aliasing or provenance |
| `raw_pointer_write` | `*ptr = value`; `ptr.write(v)`; `ptr.write_bytes(v, n)`; volatile write forms when recognized | pointer_validity, alignment, initialized_memory, same_allocation | none | pointer-live, bounds, alignment, initialized, allocation | miri -> cargo-careful; sanitizers when projected | `raw_pointer_write_assignment`, `raw_pointer_write_bytes`, `raw_pointer_write_maybeuninit`, `raw_pointer_write_volatile` | `MaybeUninit` or `u8` evidence must match the written target; unrelated destination evidence does not count | interprocedural uniqueness and aliasing remain limited |
| `raw_pointer_write_unaligned` | `ptr.write_unaligned(v)`; `core::ptr::write_unaligned(ptr, v)` | pointer_validity, initialized_memory, same_allocation | alignment | pointer-live, bounds, initialized, allocation | miri -> cargo-careful; sanitizers when projected | `raw_pointer_write_unaligned` | reject alignment obligations for this family | aliasing remains witness-heavy |
| `nonnull_unchecked` | `NonNull::new_unchecked(ptr)` | pointer_validity | none | non-null | miri -> cargo-careful | `nonnull_new_guard`, `nonnull_other_guard_not_evidence`, `nonnull_is_null_nonreturning_not_guard` | `NonNull::new(other)` is not evidence for `ptr`; non-returning error branches do not discharge | operand normalization is local and source-based |
| `unwrap_unchecked` | `opt.unwrap_unchecked()`; `res.unwrap_unchecked()` | invalid_value | none | valid-value | miri -> human-deep-review, with tests when statically related | `unwrap_unchecked_is_some_guard`, `unwrap_unchecked_is_ok_guard`, `unwrap_unchecked_is_none_return_guard`, `unwrap_unchecked_is_err_return_guard`, `unwrap_unchecked_other_infallible_not_guard` | post-call checks do not count; different receivers do not count | path-sensitive flow is local-only |
| `str_from_utf8_unchecked` | `str::from_utf8_unchecked(buf)` | invalid_value | none | utf8 | miri -> cargo-careful | `str_from_utf8_unchecked`, `str_from_utf8_unchecked_is_ok_guard`, `str_from_utf8_unchecked_is_err_return_guard` | validation must dominate the unchecked conversion | may miss indirect wrapper validations |
| `zeroed` | `mem::zeroed()`; `MaybeUninit::zeroed().assume_init()` when classified as zeroed | invalid_value, layout_or_repr, aliasing_or_provenance | none | valid-zero | miri -> human-deep-review | `zeroed_invalid_value`, `zeroed_valid_u32` | comments are contract-only, not discharge | semantic type validity is partly manual |
| `transmute` | `mem::transmute::<T, U>(v)`; `mem::transmute_copy::<T, U>(&v)` when classified as transmute | invalid_value, layout_or_repr, aliasing_or_provenance | none | layout, valid-value | miri -> kani/crux -> human-deep-review | `transmute_invalid_value`, `transmute_layout_size_guard`, `transmute_bool_valid_value_guard`, `transmute_bool_invalid_return_guard`, `transmute_copy_invalid_value`, `transmute_copy_layout_size_guard` | reject name-only type claims; valid-value and layout evidence are obligation-specific | deep invariants often require manual review |
| `vec_set_len` | `vec.set_len(n)` | initialized_memory, bounds | none | capacity, initialized | miri -> cargo-careful | `vec_set_len`, `vec_set_len_capacity_observed_not_guard`, `vec_set_len_call_result_init`, `vec_set_len_shrink` | bare `capacity()` observation is not a guard; initialization evidence must match the extended range | cannot prove initialization beyond local writes |

## Precedence

1. Syntax-backed detections are authoritative when available.
2. Text fallback must not duplicate syntax-backed rows for the same operation span.
3. Parent unknown-unsafe-block cards are suppressed when a concrete operation-family card exists and the block has no independent contract-only risk.

## Fixture and dogfood expectations

- Each registry row must reference at least one fixture calibration case before promotion.
- Any support-tier claim for a row must map to a dogfood note or explicit "fixture-only" limitation.
