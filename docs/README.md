# Documentation map

The documentation follows the [Diátaxis](https://diataxis.fr/) model so each
page has one job: teach, guide, describe, or explain. If a page starts doing
more than one of those jobs, split it or link to the page that owns the other
job.

## Start here

| Need | Read | Why |
|---|---|---|
| Understand the product goal | [`MISSION.md`](MISSION.md), [`VISION.md`](VISION.md) | Explains the problem, users, non-goals, and intended end state. |
| Install and run the tool | [`../README.md`](../README.md#quick-start) | Gives the shortest runnable path for a first check. |
| See the system shape | [`ARCHITECTURE.md`](ARCHITECTURE.md) | Explains crates, pipeline stages, and the canonical review-card surface. |
| Implement against a contract | [`specs/`](specs/) | Defines normative behavior and schemas. |
| Understand past decisions | [`adr/`](adr/) | Records durable architecture choices and trade-offs. |
| Plan or review delivery work | [`../plans/`](../plans/) | Breaks releases into PR-sized work with proof commands. |
| Check support claims | [`status/SUPPORT_TIERS.md`](status/SUPPORT_TIERS.md) | Maps product claims to evidence and known limits. |

## Diátaxis quadrants

| Quadrant | Reader question | This repo uses | Current paths |
|---|---|---|---|
| Tutorials | “Teach me by doing one complete path.” | First-run and learning journeys. | [`../README.md#quick-start`](../README.md#quick-start) |
| How-to guides | “Help me complete a specific task.” | Operational recipes and release/CI procedures. | [`ci/PR_CI.md`](ci/PR_CI.md), [`handoffs/`](handoffs/) |
| Reference | “Tell me exactly how it behaves.” | Stable contracts, schemas, policies, and generated ledgers. | [`specs/`](specs/), [`status/SUPPORT_TIERS.md`](status/SUPPORT_TIERS.md), [`../policy/`](../policy/) |
| Explanation | “Help me understand why it is this way.” | Mission, architecture, proposals, ADRs, and roadmap context. | [`MISSION.md`](MISSION.md), [`VISION.md`](VISION.md), [`ARCHITECTURE.md`](ARCHITECTURE.md), [`ROADMAP.md`](ROADMAP.md), [`proposals/`](proposals/), [`adr/`](adr/) |

## Ownership rules

| Layer | Owns | Path | Diátaxis role |
|---|---|---|---|
| Mission / vision | product purpose, users, non-goals, and end state | `docs/MISSION.md`, `docs/VISION.md` | Explanation |
| Roadmap | release direction and deferred work | `docs/ROADMAP.md` | Explanation |
| Proposals | why a workstream exists, alternatives, risks, success criteria | `docs/proposals/` | Explanation |
| Specs | behavior contracts and schemas | `docs/specs/` | Reference |
| ADRs | durable architecture decisions and trade-offs | `docs/adr/` | Explanation |
| CI guides | operational commands and PR lanes | `docs/ci/` | How-to |
| Handoffs | closeout evidence and next-step constraints | `docs/handoffs/` | How-to |
| Implementation plans | PR-sized sequence and proof commands | `plans/` | How-to |
| Support tiers | product claim to proof mapping | `docs/status/SUPPORT_TIERS.md` | Reference |
| Policies | ledgers, baselines, suppressions, and exceptions | `policy/` | Reference |

## Writing guidelines

- Pick one quadrant before writing. Name the reader’s question in the opening
  paragraph or heading.
- Keep tutorials and how-to guides task-oriented. They may link to reference
  pages, but they should not restate full schemas or policy tables.
- Keep reference pages terse, complete, and normative. Avoid motivation unless it
  changes how the contract is interpreted.
- Keep explanation pages focused on context and trade-offs. Avoid step-by-step
  operational instructions except as examples.
- Prefer links over duplication. If two pages need the same rule, one page owns
  the rule and the other page links to it.
- Keep support claims tied to proof commands, fixtures, or policy ledgers.

## Where new material goes

| If you are adding... | Put it in... | Not in... |
|---|---|---|
| A first-run walkthrough | `README.md` or a tutorial page | specs, ADRs |
| A command recipe for CI, releases, or handoff work | `docs/ci/`, `docs/handoffs/`, or `plans/` | architecture docs |
| A CLI flag, JSON field, policy key, or output contract | `docs/specs/` or `policy/` | roadmap, proposals |
| A rationale for a major design choice | `docs/adr/` | specs |
| Product motivation or user pain | `docs/proposals/`, `docs/MISSION.md`, or `docs/VISION.md` | reference pages |
| Claim maturity, known limits, or proof expectations | `docs/status/SUPPORT_TIERS.md` | README marketing copy |

## Anti-patterns

- Do not make every document do every job. Proposals say why, specs say what,
  ADRs say why this architecture, plans say how, and policies hold exceptions.
- Do not hide normative behavior in tutorials or proposals; promote it to a spec.
- Do not put active work queues in specs; use implementation plans or issues.
- Do not turn support tiers into safety claims. They describe static review
  evidence, not proof that a repository is UB-free.
