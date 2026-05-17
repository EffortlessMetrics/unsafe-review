# UNSAFE-REVIEW-ADR-0004: Three-crate surface

Status: accepted
Date: 2026-05-17
Owner: core/architecture
Linked proposal: ../proposals/UNSAFE-REVIEW-PROP-0001-product-contract.md

## Decision

Publish unsafe-review as product facade, unsafe-review-cli as adapter, and unsafe-review-core as SDK/engine. Keep implementation seams as module families.

## Context

The product must be cheap enough for PR-time use while staying honest about what
static analysis can and cannot prove.

## Consequences

- fewer false claims
- lower CI cost
- stronger handoff to dynamic/proof tools
- stable public surface before deeper compiler integration

## Alternatives considered

- default Miri execution
- compiler plugin first
- one giant crate
- default blocking gate

## Follow-up

Track follow-up implementation in `plans/0.1.0/implementation-plan.md` and support-tier proof in `docs/status/SUPPORT_TIERS.md`.
