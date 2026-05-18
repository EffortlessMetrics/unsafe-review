# Documentation map

| Layer | Owns | Path |
|---|---|---|
| Mission / vision | product purpose and end state | `docs/MISSION.md`, `docs/VISION.md` |
| Roadmap | release direction | `docs/ROADMAP.md` |
| CLI guide | day-to-day commands, output formats, and CI usage | `docs/CLI.md` |
| Proposals | why a workstream exists | `docs/proposals/` |
| Specs | behavior contracts | `docs/specs/` |
| ADRs | durable architecture decisions | `docs/adr/` |
| Implementation plans | PR-sized sequence and proof commands | `plans/` |
| Support tiers | product claim to proof mapping | `docs/status/SUPPORT_TIERS.md` |
| Policies | ledgers, baselines, suppressions | `policy/` |

Rule: do not make every document do every job. Proposals say why, specs say what,
ADRs say why this architecture, plans say how, and policies hold exceptions.

Start with the [CLI guide](CLI.md) if you want to run the tool before reading the product specs.
