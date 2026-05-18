# Documentation map

The documentation follows the [Diátaxis](https://diataxis.fr/) model: each page
has one primary job. Tutorials teach, how-to guides solve tasks, reference pages
state facts, and explanation pages build understanding.

## Start by goal

| Goal | Diátaxis mode | Start here |
|---|---|---|
| Learn the product by completing a first run | Tutorial | [`tutorials/first-review-card.md`](tutorials/first-review-card.md) |
| Run the tool on a pull request | How-to | [`how-to/run-pr-review.md`](how-to/run-pr-review.md) |
| Look up command syntax or output contracts | Reference | [`reference/cli.md`](reference/cli.md), [`specs/`](specs/) |
| Understand the review model and trade-offs | Explanation | [`explanation/review-cards-and-trust-boundary.md`](explanation/review-cards-and-trust-boundary.md), [`ARCHITECTURE.md`](ARCHITECTURE.md) |

## Diátaxis sections

| Section | Reader question | Owns | Path |
|---|---|---|---|
| Tutorials | “Help me learn by doing.” | Guided first successes | [`docs/tutorials/`](tutorials/) |
| How-to guides | “Help me complete this task.” | Operational recipes | [`docs/how-to/`](how-to/) |
| Reference | “Tell me exactly what exists.” | Commands, schemas, specs, support promises, policy ledgers | [`docs/reference/`](reference/), [`docs/specs/`](specs/), [`docs/status/`](status/), [`policy/`](../policy/) |
| Explanation | “Help me understand why.” | Mission, vision, architecture, proposals, ADRs, trust boundary | [`docs/explanation/`](explanation/), [`docs/adr/`](adr/), [`docs/proposals/`](proposals/) |

## Product planning layers

Some documents are planning artifacts rather than user-facing product docs. Keep
them scoped to their planning job:

| Layer | Owns | Path |
|---|---|---|
| Mission / vision | product purpose and end state | [`MISSION.md`](MISSION.md), [`VISION.md`](VISION.md) |
| Roadmap | release direction | [`ROADMAP.md`](ROADMAP.md) |
| Proposals | why a workstream exists | [`proposals/`](proposals/) |
| Specs | behavior contracts | [`specs/`](specs/) |
| ADRs | durable architecture decisions | [`adr/`](adr/) |
| Implementation plans | PR-sized sequence and proof commands | [`../plans/`](../plans/) |
| Support tiers | product claim to proof mapping | [`status/SUPPORT_TIERS.md`](status/SUPPORT_TIERS.md) |
| Policies | ledgers, baselines, suppressions | [`../policy/`](../policy/) |

## Placement rules

- Put first-run learning paths in `docs/tutorials/`.
- Put task recipes and CI/operator procedures in `docs/how-to/`.
- Put exhaustive command, schema, policy, and support details in `docs/reference/`,
  `docs/specs/`, `docs/status/`, or `policy/`.
- Put rationale, mental models, trade-offs, proposals, and decisions in
  `docs/explanation/`, `docs/proposals/`, or `docs/adr/`.
- Do not make every document do every job. Link across quadrants instead of
  mixing tutorial, task, reference, and rationale content on one page.
