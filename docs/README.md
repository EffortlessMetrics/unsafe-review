# Documentation map

| Layer | Owns | Path |
|---|---|---|
| Mission / vision | product purpose and end state | `docs/MISSION.md`, `docs/VISION.md` |
| Roadmap | release direction | `docs/ROADMAP.md` |
| Proposals | why a workstream exists | `docs/proposals/` |
| CLI reference | command usage, options, output, and exit status | `docs/CLI.md` |
| Specs | behavior contracts | `docs/specs/` |
| ADRs | durable architecture decisions | `docs/adr/` |
| Implementation plans | PR-sized sequence and proof commands | `plans/` |
| Support tiers | product claim to proof mapping | `docs/status/SUPPORT_TIERS.md` |
| Policies | ledgers, baselines, suppressions | `policy/` |

Rule: do not make every document do every job. Proposals say why, specs say what,
ADRs say why this architecture, plans say how, the CLI reference says how to run the product, and policies hold exceptions.
