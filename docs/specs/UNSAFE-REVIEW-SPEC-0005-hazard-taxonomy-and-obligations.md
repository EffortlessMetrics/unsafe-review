# UNSAFE-REVIEW-SPEC-0005: Hazard taxonomy and obligations

Status: accepted
Owner: core/spec
Created: 2026-05-17
Updated: 2026-05-19
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md
Linked plan: ../../plans/0.1.0/implementation-plan.md
Registry appendix: ./appendices/UNSAFE-REVIEW-SPEC-0005-appendix-operation-family-registry.md

## Contract

`unsafe-review` must map each detected unsafe operation to an `operation_family`, hazard set, and obligation set using the registry appendix as the canonical table for promoted rows.

## States / schema

This spec constrains fields already carried by `ReviewCard` and its JSON projections. It must not create a second PR/SARIF/LSP/agent truth.

Required current JSON card fields for this spec surface:

- `operation_family` (string; required)
- `hazards[]` (non-empty array; required)
- `obligations[]` (non-empty array; required)
- `obligation_evidence[]`, with per-obligation `contract`, `discharge`, `reach`, and `witness` states
- `missing[]` for unresolved obligation messages
- `verify_commands[]` for reviewer witness routes

Internal route objects and downstream projections may carry richer witness metadata, but those fields must be derived from the `ReviewCard` rather than reclassifying the finding.

## Matching / precedence rules

1. Match syntax-backed operation shape first.
2. If syntax match fails, text fallback may classify only when it can identify a known registry family.
3. If syntax and fallback both match the same site, syntax-backed result wins for location/snippet and family ID.
4. If a concrete family is detected within an `unsafe` block, suppress parent "unknown unsafe block" cards unless the block has independent contract risk.

## Counts as evidence

- Operation-family entries listed in the registry appendix with fixture references.
- Hazard/obligation sets that exactly match the selected family row.
- Witness commands or projection route fields derived from family-hazard routing.

## Does not count

- Free-form hazard labels not in registry.
- Family-specific hazard leakage (for example alignment hazard on `*_unaligned` families).
- Comment-only statements used as discharge evidence.

## Fixtures

All promoted families must cite fixture names in the registry appendix and corresponding calibration entries.

## Dogfood

Support-tier claims for a family must include dogfood proof or explicit "fixture-only" limitations.

## Output examples

```json
{
  "operation_family": "raw_pointer_read",
  "hazards": ["pointer_validity", "alignment", "initialized_memory", "same_allocation"],
  "obligations": [
    "pointer is live and dereferenceable for the accessed type",
    "buffer has enough bytes for the accessed type",
    "pointer is aligned for the accessed type",
    "memory is initialized for the accessed type",
    "access remains inside one live allocation"
  ],
  "obligation_evidence": [
    {
      "key": "alignment",
      "description": "pointer is aligned for the accessed type",
      "contract": {"present": false, "state": "missing", "summary": "No contract evidence found"},
      "discharge": {"present": false, "state": "missing", "summary": "No guard evidence found"},
      "reach": {"present": false, "state": "missing", "summary": "No static test relation found"},
      "witness": {"present": false, "state": "missing", "summary": "No witness receipt imported"}
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

A new or changed family is promotable only with:

1. registry row,
2. fixture + calibration coverage,
3. golden output assertion, and
4. support-tier update (or explicit fixture-only limitation).
