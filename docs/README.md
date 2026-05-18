# Documentation map

This documentation set follows the [Diataxis](https://diataxis.fr/) split:
tutorials teach, how-to guides help you complete a task, reference defines exact
contracts, and explanation records why the product and architecture work this way.

Use the first table to choose what to read. Use the second table to decide where a
new document belongs.

## Start here

| Need | Diataxis mode | Read |
|---|---|---|
| Understand the product promise before trying it | Explanation | [`MISSION.md`](MISSION.md), [`VISION.md`](VISION.md) |
| Run the tool on a small diff | Tutorial | [`tutorials/review-a-diff.md`](tutorials/review-a-diff.md) |
| Complete a specific operator or maintainer task | How-to | [`how-to/README.md`](how-to/README.md) |
| Look up exact product behavior or schemas | Reference | [`specs/README.md`](specs/README.md), [`reference/README.md`](reference/README.md) |
| Understand architecture decisions | Explanation | [`ARCHITECTURE.md`](ARCHITECTURE.md), [`adr/README.md`](adr/README.md) |
| See release direction and support status | Explanation / Reference | [`ROADMAP.md`](ROADMAP.md), [`status/SUPPORT_TIERS.md`](status/SUPPORT_TIERS.md) |

## Diataxis roles in this repository

| Diataxis mode | Reader question | Repository home | What belongs here |
|---|---|---|---|
| Tutorial | “Can you walk me through a safe first success?” | `docs/tutorials/` | Learning-oriented paths with concrete commands, fixtures, expected observations, and cleanup notes. |
| How-to | “How do I do this one job?” | `docs/how-to/` | Task recipes for CI, reviewers, maintainers, release work, and docs/spec updates. |
| Reference | “What is the contract?” | `docs/specs/`, `docs/reference/`, `policy/`, `MANIFEST.md` | Stable behavior, schemas, policy keys, support tiers, and inventory facts. |
| Explanation | “Why is it built this way?” | `docs/MISSION.md`, `docs/VISION.md`, `docs/ARCHITECTURE.md`, `docs/adr/`, `docs/proposals/`, `docs/ROADMAP.md` | Product rationale, alternatives, tradeoffs, durable decisions, and future direction. |

## Authoring rule

Do not make every document do every job:

- Tutorials optimize for learning and should avoid exhaustive option lists.
- How-to guides optimize for task completion and should link to reference instead
  of restating schemas.
- Reference documents optimize for precision and should avoid narrative rationale.
- Explanation documents optimize for context and should avoid becoming runbooks.

When a document starts mixing modes, split the extra material into the appropriate
home and link between the pages.
