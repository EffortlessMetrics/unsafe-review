# Documentation map

| Layer | Owns | Path |
|---|---|---|
| Mission / vision | product purpose and end state | `docs/MISSION.md`, `docs/VISION.md` |
| Usage guide | practical CLI workflows and troubleshooting | `docs/USAGE.md` |
| Roadmap | release direction | `docs/ROADMAP.md` |
| Proposals | why a workstream exists | `docs/proposals/` |
| Specs | behavior contracts | `docs/specs/` |
| ADRs | durable architecture decisions | `docs/adr/` |
| Implementation plans | PR-sized sequence and proof commands | `plans/` |
| Support tiers | product claim to proof mapping | `docs/status/SUPPORT_TIERS.md` |
| Policies | ledgers, baselines, suppressions | `policy/` |

Rule: do not make every document do every job. Proposals say why, specs say what,
ADRs say why this architecture, plans say how, and policies hold exceptions.

## Start here

- New users should start with the [usage guide](USAGE.md) and then the [architecture overview](ARCHITECTURE.md).
- Contributors changing behavior should update the relevant spec and, when the design rationale changes, add or amend an ADR.
- Release planning belongs in `docs/ROADMAP.md` and `plans/`, not in individual specs.
