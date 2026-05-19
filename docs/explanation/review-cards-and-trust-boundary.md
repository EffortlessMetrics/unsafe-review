# Review Cards And Trust Boundary

`unsafe-review` sits between changed unsafe Rust and expensive witnesses. It
answers a review-process question:

```text
Does this unsafe-adjacent change have the evidence a reviewer needs?
```

It does not answer the stronger memory-safety question:

```text
Can this code never exhibit undefined behavior?
```

## What A ReviewCard Means

A `ReviewCard` is a small evidence object for one unsafe-adjacent seam. It names:

- the operation family under review;
- the hazard class;
- the inferred safety obligations;
- contract evidence such as `# Safety` docs or local `SAFETY:` comments;
- guard or discharge evidence near the changed operation;
- static test-reach evidence;
- witness routes such as Miri, `cargo-careful`, sanitizers, Loom/Shuttle,
  Kani/Crux, or human deep review.

Every output surface should project this same card. Terminal text, JSON,
Markdown PR summaries, SARIF, comment plans, saved LSP data, agent packets, repo
posture, badges, baselines, suppressions, and witness receipts must not create
parallel classifications.

## What A ReviewCard Does Not Mean

A card is not a vulnerability report and not a soundness verdict. Missing
evidence means a reviewer may need a better contract, a local guard, a targeted
test, or a witness run.

No open cards does not mean a repository is safe, sound, UB-free, or Miri-clean.
It only means the configured static review pass did not emit open
unsafe-review gaps for the selected scope.

Static related-test evidence is also limited. A test mention can show that a
wrapper or owner appears reachable by name, but it is not proof that the unsafe
site executed unless a matching receipt records that evidence.

## Why Witnesses Are Routed

Witness tools answer different questions:

- Miri and `cargo-careful` are useful for many pure-Rust pointer and
  invalid-value hazards.
- Sanitizers are often better for FFI and runtime memory diagnostics.
- Loom and Shuttle are better fits for concurrency and Send/Sync invariants.
- Kani and Crux can help with small explicit proof harnesses.
- Human deep review remains the right route for unsupported, provenance-heavy,
  or architecture-specific seams.

`unsafe-review` recommends the cheapest credible next route and can import
receipts after those tools run elsewhere. It does not run witnesses by default
and does not claim witness success without an attached receipt.

## Why Policy Is Advisory First

Unsafe review signals need calibration before they become gates. The default
posture is advisory so maintainers can inspect false positives, tune exact
baselines and suppressions, and promote only fixture-backed or dogfood-backed
claims to stricter policy later.

The core sentence stays:

```text
unsafe-review finds unsafe Rust changes missing a safety contract, guard, test,
or witness.
```
