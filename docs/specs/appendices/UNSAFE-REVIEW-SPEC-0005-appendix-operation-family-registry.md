# Operation-family registry appendix

Status: accepted
Owner: core/spec
Updated: 2026-05-19
Normative owner: ../UNSAFE-REVIEW-SPEC-0005-hazard-taxonomy-and-obligations.md

This appendix provides the canonical operation-family table referenced by Spec 0005.

## Registry table

| operation_family | detected syntax shapes | hazards | not hazards | evidence keys | witness route | fixture proof | known false-positive controls | known limits |
|---|---|---|---|---|---|---|---|---|
| `raw_pointer_read` | `ptr.read()`; `core::ptr::read(ptr)` | pointer_validity, alignment, initialized_memory, same_allocation | — | bounds_guard, alignment_guard, initialized_source, allocation_relation | miri -> cargo-careful | `raw_pointer_read` | reject comment-only guards; reject bare `align_of` mentions | macro-expanded forms may degrade to unknown |
| `raw_pointer_read_unaligned` | `ptr.read_unaligned()`; `core::ptr::read_unaligned(ptr)` | pointer_validity, initialized_memory, same_allocation | alignment | bounds_guard, initialized_source, allocation_relation | miri -> cargo-careful | `raw_pointer_read_unaligned`, `raw_pointer_alignment` | alignment-only checks do not discharge missing bounds/init | cannot prove dynamic aliasing |
| `raw_pointer_write` | `ptr.write(v)`; `core::ptr::write(ptr, v)` | pointer_validity, alignment, write_permission, same_allocation | initialized_destination | bounds_guard, alignment_guard, unique_write_access | miri -> sanitizers | `raw_pointer_write` | `MaybeUninit` destination cannot be used as write-permission proof | interprocedural uniqueness is limited |
| `raw_pointer_write_unaligned` | `ptr.write_unaligned(v)`; `core::ptr::write_unaligned(ptr, v)` | pointer_validity, write_permission, same_allocation | alignment | bounds_guard, unique_write_access | miri -> sanitizers | `raw_pointer_write_unaligned` | reject alignment obligations for this family | aliasing remains witness-heavy |
| `nonnull_new_unchecked` | `NonNull::new_unchecked(ptr)` | pointer_validity, non_null | — | same_operand_nonnull_guard | miri -> cargo-careful | `nonnull_new_guard`, `nonnull_new_guard_other_pointer` | `NonNull::new(other)` is not evidence for `ptr` | operand normalization limited across macro rewrites |
| `unwrap_unchecked` | `opt.unwrap_unchecked()`; `res.unwrap_unchecked()` | validity_of_assumed_variant | — | same_receiver_state_guard, early_return_on_none_or_err | tests+receipt | `unwrap_unchecked_is_some_guard` | post-call checks do not count; different receiver does not count | path-sensitive flow is local-only |
| `str_from_utf8_unchecked` | `str::from_utf8_unchecked(buf)` | utf8_validity, bounds | — | same_buffer_utf8_check, returning_error_branch | miri -> cargo-careful | `str_from_utf8_validation` | validation on different buffer does not count | may miss indirect wrapper validations |
| `mem_zeroed` | `mem::zeroed()` | initialized_memory, valid_bit_pattern | — | type_wrapper_safety, explicit-type-allowance | miri -> human-review | `mem_zeroed` | comments are contract-only, not discharge | semantic type validity is partly manual |
| `transmute` | `mem::transmute::<T,U>(v)` | layout_compatibility, validity, ownership_semantics | — | size_equality_guard, repr_contract, invariant_doc | miri -> kani/crux -> human-review | `transmute` | reject name-only “same type” claims | deep invariants often require manual review |
| `vec_set_len` | `vec.set_len(n)` | initialized_memory, bounds, length_capacity_consistency | — | n_le_capacity_guard, initialized_prefix_proof | miri -> cargo-careful | `vec_set_len` | bare `capacity()` mention is not guard | cannot prove initialization beyond local writes |

## Precedence

1. Syntax-backed detections are authoritative when available.
2. Text fallback must not duplicate syntax-backed rows for the same operation span.
3. Parent unknown-unsafe-block cards are suppressed when a concrete operation-family card exists and the block has no independent contract-only risk.

## Fixture and dogfood expectations

- Each registry row must reference at least one fixture calibration case before promotion.
- Any support-tier claim for a row must map to a dogfood note or explicit “fixture-only” limitation.
