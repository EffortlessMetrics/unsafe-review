# Review cards and trust boundary

`unsafe-review` sits between unsafe inventory tools and execution witnesses.
It answers a review-process question: does this unsafe-adjacent change have the
evidence a reviewer needs?

```text
unsafe seam
-> hazard
-> safety obligation
-> contract evidence
-> discharge evidence
-> test reach evidence
-> witness route
-> review card
```

## What a review card means

A review card is a focused request for human or tool attention. It names an unsafe
seam, classifies the likely hazard, lists the safety obligations, and records the
evidence found near the changed code.

The card is the canonical unit for every output surface: terminal text, JSON,
Markdown summaries, badges, editor diagnostics, and agent packets should all be
projections of the same card rather than independent interpretations.

## What a review card does not mean

A card is not a proof obligation discharged by the tool. It is also not evidence
that undefined behavior exists. Missing evidence means the reviewer may need a
better safety comment, a local guard, a reachable test, or a witness run.

Likewise, no open cards does not make a repository UB-free. It only means the
configured analyzer and policy did not find open unsafe-review gaps in the chosen
scope.

## Why witnesses are routed, not replaced

Miri, `cargo-careful`, sanitizers, Loom, Kani, and Crux answer execution or model
questions for specific seams and harnesses. `unsafe-review` routes changed seams
to likely witnesses and records receipts when available. It does not replace those
witnesses or claim their results without attached evidence.

## Why policy is advisory first

Unsafe review signals need calibration before they become gates. The default
posture is advisory so teams can measure false positives, tune suppressions, and
promote only well-supported claims to stricter CI policy.
