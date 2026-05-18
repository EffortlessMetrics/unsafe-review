# Documentation map

The docs are organized by decision layer. Start with the product docs when you want
to understand the user promise, then move down to specs and ADRs when you need the
exact contract that implementation work must preserve.

| Layer | Owns | Path |
|---|---|---|
| Mission / vision | product purpose and end state | `docs/MISSION.md`, `docs/VISION.md` |
| Roadmap | release direction | `docs/ROADMAP.md` |
| Proposals | why a workstream exists | `docs/proposals/` |
| Specs | behavior contracts | `docs/specs/` |
| ADRs | durable architecture decisions | `docs/adr/` |
| Implementation plans | PR-sized sequence and proof commands | `plans/` |
| Support tiers | product claim to proof mapping | `docs/status/SUPPORT_TIERS.md` |
| Policies | ledgers, baselines, suppressions | `policy/` |

Rule: do not make every document do every job. Proposals say why, specs say what,
ADRs say why this architecture, plans say how, and policies hold exceptions.

## Recommended reading paths

- **New users**: read the root [`README.md`](../README.md), then
  [`docs/MISSION.md`](MISSION.md), then [`docs/ROADMAP.md`](ROADMAP.md).
- **Implementers**: read [`docs/ARCHITECTURE.md`](ARCHITECTURE.md), the relevant
  spec in [`docs/specs/`](specs/), and the active plan under [`plans/`](../plans/).
- **Reviewers**: read [`docs/status/SUPPORT_TIERS.md`](status/SUPPORT_TIERS.md)
  before relying on a finding as a blocking signal.
- **Decision archaeology**: start with [`docs/adr/README.md`](adr/README.md), then
  follow the numbered ADRs for the design area you are changing.

## Writing and update rules

- Keep CLI examples in sync with the implemented parser before publishing them.
- Update support-tier docs whenever a claim becomes more or less reliable.
- Prefer adding a spec for externally visible behavior and an ADR for irreversible
  architecture choices.
- Put temporary sequencing notes in `plans/`, not in durable specs.
