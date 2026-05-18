# Documentation map

The documentation is split by decision level so each document has one job. Start with
the root [`README.md`](../README.md) if you want to run the tool, then use this map to
find the product intent, durable contracts, and implementation sequence.

| Layer | Owns | Path | Read when you need to... |
|---|---|---|---|
| Mission / vision | product purpose and end state | `docs/MISSION.md`, `docs/VISION.md` | understand why unsafe review is separate from dynamic UB detection. |
| Roadmap | release direction | `docs/ROADMAP.md` | see what must be proven before stronger claims or stricter CI behavior. |
| Proposals | why a workstream exists | `docs/proposals/` | review ideas before they become binding specs. |
| Specs | behavior contracts | `docs/specs/` | implement, test, or audit user-visible behavior. |
| ADRs | durable architecture decisions | `docs/adr/` | understand why the workspace is shaped this way. |
| Implementation plans | PR-sized sequence and proof commands | `plans/` | pick up an implementation task or verify a milestone. |
| Support tiers | product claim to proof mapping | `docs/status/SUPPORT_TIERS.md` | decide whether a finding can be advisory, no-new-debt, or blocking. |
| Policies | ledgers, baselines, suppressions | `policy/` | inspect exceptions and repository-specific policy data. |

## Suggested reading paths

### I want to try the tool

1. Root [`README.md`](../README.md) for installation, commands, and trust boundary.
2. [`docs/status/SUPPORT_TIERS.md`](status/SUPPORT_TIERS.md) before treating output as a
   release gate.
3. [`docs/ci/PR_CI.md`](ci/PR_CI.md) when wiring advisory output into CI.

### I want to implement or review analyzer behavior

1. [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) for the crate and pipeline shape.
2. [`docs/specs/README.md`](specs/README.md) for the normative behavior index.
3. The matching ADR in [`docs/adr/`](adr/) when a design tradeoff is surprising.
4. The active plan under [`plans/`](../plans/) for the PR-sized sequence and proof
   commands.

### I want to change product policy

1. Start with a proposal in [`docs/proposals/`](proposals/) if the change alters user
   expectations.
2. Update the relevant spec once the behavior becomes accepted.
3. Update [`docs/status/SUPPORT_TIERS.md`](status/SUPPORT_TIERS.md) if the change affects
   claim strength, gating, or calibration evidence.
4. Record durable architecture consequences in an ADR.

## Maintenance rules

- Proposals say **why a workstream should exist**.
- Specs say **what behavior the product promises**.
- ADRs say **why this architecture or policy choice won**.
- Plans say **how to deliver and verify the next slice**.
- Policies hold **exceptions, ledgers, baselines, and suppressions**.

Do not make every document do every job. When a change crosses layers, update the
smallest set of documents that keeps this separation true.
