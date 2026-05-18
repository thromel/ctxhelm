# Phase 26 Plan: Freshness-Aware Domain Cards

## Goal

Generate deterministic local-only domain cards with source links, input hashes,
freshness, review metadata, and regeneration reasons.

## Work

- Extend `ctxpack cards generate`.
- Persist deterministic domain memory records.
- Keep Markdown cards source-snippet-free.

## Verification

- Existing card generation tests now assert domain cards and memory persistence.
