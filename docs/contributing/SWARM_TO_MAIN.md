# Swarm to main promotion policy

`EffortlessMetrics/unsafe-review-swarm` is the default implementation and
dogfood proving repo.

`EffortlessMetrics/unsafe-review` is the public source-of-record repo for the
published crates, release preparation, public documentation, badge endpoints,
support posture, and curated promotion.

The split is intentional:

```text
swarm proves
source absorbs
release publishes
```

Do not treat the two repositories as equal work targets. Routine analyzer,
evidence, dogfood, candidate extraction, and experiment work starts in
`unsafe-review-swarm`. Source repo PRs should be quieter, narrower, and tied to
public record or deliberate promotion.

## Default routing

Open new work in `unsafe-review-swarm` by default when it changes:

- analyzer or evidence behavior,
- fixtures, goldens, calibration, or dogfood controls,
- artifact verifier experiments,
- policy simulation internals,
- candidate branch extraction,
- broad refactors or risky implementation cleanup.

Open work directly in `unsafe-review` only when it is one of:

- release preparation or publication receipt work,
- public README, badge, docs.rs, crate metadata, or package surface work,
- urgent source hotfix for published users,
- source-only repository hygiene,
- curated promotion from a green swarm PR or commit.

Every source repo PR must declare its route:

```text
Source: swarm PR #NN / commit SHA
Source: direct-main public or release surface
Source: direct-main urgent source hotfix
Source: direct-main source-only repo hygiene
```

If a source PR is not swarm-originated, it must say why the work belongs in
`unsafe-review` now.

## Promotion eligibility

A swarm slice is eligible for source promotion only after it has:

- green routed CI,
- fixture, golden, calibration, dogfood, receipt, schema, or docs proof
  appropriate to the claim,
- support-tier or status text updated when the public claim changes,
- no unsupported safety, UB-free, Miri-clean, site-execution, or policy-gate
  claim,
- no witness execution by default,
- no automatic comments,
- no blocking policy by default.

Promotion should recreate or cherry-pick the exact behavior onto current source
`main`. Do not merge a stale swarm branch directly into source.

## Source PR naming

Use `port(swarm #NN): ...` for promoted implementation slices, for example:

```text
port(swarm #90): reject stale copy range guards
```

Use normal public-surface prefixes for source-only work:

```text
docs(readme): add public unsafe-review badges
release: prepare 0.1.1 polish
```

## Promotion batches

When promoting more than one related swarm slice, add or update a short handoff
that records:

- swarm PR or commit,
- source PR,
- behavior carried over,
- proof carried over,
- support-tier or spec impact,
- release impact.

Promotion batches should stay reviewable. Prefer several narrow source PRs over
one large release-time cutover.

## Standing boundaries

The source repo remains conservative:

- `ReviewCard` remains the source of truth.
- Public badges and docs map to checked evidence.
- No support-tier promotion without proof.
- No default witness execution.
- No automatic comments.
- No source edits by the tool.
- No default blocking policy.
- No output may imply safety, UB-free status, Miri-clean status, site execution,
  or policy readiness without exact evidence.
