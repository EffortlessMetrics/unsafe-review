# UNSAFE-REVIEW-PROP-0001: Product contract

Status: accepted
Owner: core/product
Created: 2026-05-17
Linked specs: UNSAFE-REVIEW-SPEC-0001 through UNSAFE-REVIEW-SPEC-0017
Linked ADRs: UNSAFE-REVIEW-ADR-0001 through UNSAFE-REVIEW-ADR-0005
Linked plan: ../../plans/0.1.0/implementation-plan.md

## Problem

Rust teams want Miri-like confidence but cannot afford broad Miri execution on
every change. Existing tools answer partial questions: where unsafe exists,
whether a concrete execution hits UB, whether a proof harness passes, or whether
a specific bug family appears. They do not cheaply answer the PR review question:

```text
What unsafe contract changed, what evidence is missing, and what witness should run next?
```

## Users and surfaces

- PR reviewers
- unsafe library maintainers
- CI owners
- IDE users
- LLM coding agents

## Success criteria

- changed unsafe seams become compact review cards
- every actionable card names missing contract, guard, reach, or witness evidence
- PR output is sparse and actionable
- LSP projection is read-only in v1
- LLM packets include allowed repairs, do-not-do list, verify command, and stop conditions
- repo badges count open review gaps, not raw unsafe usage

## Alternatives considered

- Run Miri everywhere: too expensive and not always compatible.
- Build a proof system: too annotation-heavy for the PR-time wedge.
- Build another cargo-geiger: too shallow; inventory is not review.
- Build a Rust compiler plugin: too unstable for v0.1.

## Non-goals

- memory-safety proof
- UB-free claim
- default blocking
- automatic code edits
- broad generated tests

## Exit criteria

The proposal is done when v0.1 emits review cards from stable source analysis and
the spec system, policy ledgers, and initial CLI are present.
