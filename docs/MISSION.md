# Mission

`unsafe-review` makes unsafe Rust review cheap enough to run on every pull request.

The tool does not try to prove memory safety. It finds changed unsafe-adjacent
code, names the safety contract it appears to rely on, checks whether that
contract is documented and locally discharged, estimates whether tests reach the
safe wrapper, and routes the seam to the cheapest useful witness.

```text
unsafe seam
-> hazard
-> safety contract
-> discharge evidence
-> reach evidence
-> witness route
-> review card
```

## Users

- Rust library maintainers with small but important unsafe islands.
- Reviewers approving PRs that touch raw pointers, FFI, layout, atomics, `Pin`,
  `MaybeUninit`, or safe wrappers over unsafe internals.
- CI owners who want Miri, sanitizer, Loom, or Kani runs only where they buy
  signal.
- IDEs and coding agents that need bounded unsafe repair packets rather than
  broad “fix this unsafe code” prompts.

## Non-goals

- no soundness claim
- no UB-free badge
- no automatic unsafe rewrites in v1
- no Miri replacement
- no default blocking before calibration
