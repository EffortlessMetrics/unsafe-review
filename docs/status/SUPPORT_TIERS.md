# Support tiers

All tiers describe static review evidence. None means memory-safety proof.

| Capability | Tier | Surface | Proof | Known limits |
|---|---|---|---|---|
| Diff unsafe site inventory | scaffold | CLI JSON/human | compile gate and one fixture smoke | source-based, not MIR |
| Review cards | scaffold | CLI / PR artifacts | fixture golden tests and JSON contract tests | fixture corpus is still small |
| Contract evidence mining | scaffold | cards / hovers | `# Safety` and `SAFETY:` smoke fixture | comment quality is heuristic |
| Guard evidence mining | scaffold | cards | raw-pointer alignment smoke fixture | card-wide evidence; obligation-level evidence is planned |
| Witness routing | scaffold | cards / packets | route-table code smoke | route may be incomplete |
| Repo inventory | scaffold | repo JSON / badges | compile gate only | badge is not UB-free claim |
| LSP projection | planned | editor | saved-card fixtures | read-only first |
| Agent packets | planned | JSON packet | packet schema tests | agents still require review |
| Receipt import | planned | witness receipts | Miri/careful/sanitizer fixtures | receipt strength must be explicit |
| MIR/nightly facts | deferred | optional adapter | ADR needed | not v0.1 product default |
