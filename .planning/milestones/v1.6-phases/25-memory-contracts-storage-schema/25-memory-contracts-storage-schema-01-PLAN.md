# Phase 25 Plan: Memory Contracts & Storage Schema

## Goal

Add stable source-free contracts and SQLite persistence for repo memory cards.

## Work

- Add `MemoryCard`, freshness, review status, and selected-memory contracts.
- Add `memory_cards` storage schema under schema version 3.
- Persist, list, and update memory card review status without raw source or prompts.
- Report memory-card counts in storage status.

## Verification

- Contract serialization tests.
- Storage source-free persistence and review-control tests.
