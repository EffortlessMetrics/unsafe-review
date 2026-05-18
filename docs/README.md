# Documentation map

This documentation is organized with the Diátaxis model so each page has one job:

- **Tutorials** teach the first successful path through the tool.
- **How-to guides** solve specific operational tasks.
- **Reference** states stable contracts, schemas, policies, and support claims.
- **Explanation** records why the product and architecture are shaped this way.

If you are not sure where to start, read the tutorial first, then use the how-to
recipes during PR work, and treat reference pages as the source of truth when
behavior or compatibility is in question.

## Start here

| Need | Diátaxis type | Start with |
|---|---|---|
| Run the tool once and understand the output | Tutorial | [`docs/tutorials/`](tutorials/) |
| Solve a PR, CI, or triage task | How-to | [`docs/how-to/`](how-to/) |
| Check a schema, policy rule, support claim, or CLI contract | Reference | [`docs/reference/`](reference/) |
| Understand product intent, architecture, or decision history | Explanation | [`docs/explanation/`](explanation/) |

## Full documentation inventory

| Layer | Diátaxis type | Owns | Path |
|---|---|---|---|
| Tutorials | Tutorial | learning-oriented walkthroughs | `docs/tutorials/` |
| How-to guides | How-to | task recipes for PR and CI use | `docs/how-to/` |
| Specs | Reference | behavior contracts | `docs/specs/` |
| Support tiers | Reference | product claim to proof mapping | `docs/status/SUPPORT_TIERS.md` |
| Policies | Reference | ledgers, baselines, suppressions | `policy/` |
| CI model | Reference | default PR checks and witness lanes | `docs/ci/PR_CI.md` |
| Mission / vision | Explanation | product purpose and end state | `docs/MISSION.md`, `docs/VISION.md` |
| Architecture | Explanation | system shape and invariants | `docs/ARCHITECTURE.md` |
| Roadmap | Explanation | release direction | `docs/ROADMAP.md` |
| Proposals | Explanation | why a workstream exists | `docs/proposals/` |
| ADRs | Explanation | durable architecture decisions | `docs/adr/` |
| Implementation plans | How-to / planning | PR-sized sequence and proof commands | `plans/` |
| Handoffs | How-to / operations | closeout evidence and next-step warnings | `docs/handoffs/` |

## Placement rules

Do not make every document do every job:

- Put learning paths in tutorials, not specs.
- Put repeatable task steps in how-to guides, not ADRs.
- Put normative behavior in specs, support tiers, and policy ledgers.
- Put reasoning, tradeoffs, and historical choices in proposals and ADRs.
- Put active PR sequencing in plans, and closeout evidence in handoffs.
