# Support tiers

All tiers describe static review evidence. None means memory-safety proof.

| Capability | Tier | Surface | Proof | Known limits |
|---|---|---|---|---|
| Diff unsafe site inventory | usable alpha | CLI JSON/human | fixture examples | source-based, not MIR |
| Review cards | usable alpha | CLI / PR artifacts | schema and golden checks | classification can be conservative |
| Contract evidence mining | experimental | cards / hovers | `# Safety` and `SAFETY:` fixtures | comment quality is heuristic |
| Guard evidence mining | experimental | cards | simple guard fixtures | not semantic proof |
| Witness routing | experimental | cards / packets | route-table fixtures | route may be incomplete |
| Repo inventory | usable alpha | repo JSON / badges | repo fixture | badge is not UB-free claim |
| LSP projection | planned | editor | saved-card fixtures | read-only first |
| Agent packets | planned | JSON packet | packet schema tests | agents still require review |
| Receipt import | planned | witness receipts | Miri/careful/sanitizer fixtures | receipt strength must be explicit |
| MIR/nightly facts | deferred | optional adapter | ADR needed | not v0.1 product default |
