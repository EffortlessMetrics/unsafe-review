# Reference

Reference pages define precise contracts and inventory facts. They are optimized
for lookup, not teaching or rationale.

## Primary references

| Subject | Reference |
|---|---|
| Product behavior and schemas | [`../specs/README.md`](../specs/README.md) |
| Support tier names and current claims | [`../status/SUPPORT_TIERS.md`](../status/SUPPORT_TIERS.md) |
| Policy ledgers, baselines, and suppressions | [`../../policy/`](../../policy/) |
| Workspace inventory | [`../../MANIFEST.md`](../../MANIFEST.md) |
| CLI facade crate notes | [`../../crates/unsafe-review/README.md`](../../crates/unsafe-review/README.md) |

## Reference-writing checklist

- State normative behavior with must/should language when appropriate.
- Keep examples small and label them as examples, not exhaustive behavior.
- Link to tutorials for learning paths and to ADRs or proposals for rationale.
- Include the proof command or fixture requirement when a contract needs CI
  evidence.
