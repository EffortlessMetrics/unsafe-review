# Documentation map

| Layer | Owns | Path |
|---|---|---|
| Mission / vision | product purpose and end state | `docs/MISSION.md`, `docs/VISION.md` |
| Roadmap | release direction | `docs/ROADMAP.md` |
| CLI guide | command behavior, output formats, and CI examples | `docs/CLI.md` |
| Proposals | why a workstream exists | `docs/proposals/` |
| Specs | behavior contracts | `docs/specs/` |
| ADRs | durable architecture decisions | `docs/adr/` |
| Implementation plans | PR-sized sequence and proof commands | `plans/` |
| Support tiers | product claim to proof mapping | `docs/status/SUPPORT_TIERS.md` |
| Policies | ledgers, baselines, suppressions | `policy/` |

Rule: do not make every document do every job. Proposals say why, specs say what,
ADRs say why this architecture, plans say how, and policies hold exceptions.


## Common entry points

- New users should start with the repository [`README.md`](../README.md), then use
  [`docs/CLI.md`](CLI.md) for day-to-day commands.
- Product and trust-boundary questions belong in [`docs/MISSION.md`](MISSION.md),
  [`docs/VISION.md`](VISION.md), and [`docs/ROADMAP.md`](ROADMAP.md).
- Durable behavior changes should update the relevant spec in [`docs/specs/`](specs/)
  and add or update an ADR in [`docs/adr/`](adr/) when the architecture changes.
- Release or implementation sequencing belongs under [`plans/`](../plans/), with
  proof commands that can be copied into a PR.
