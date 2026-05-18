# Phase 28 Plan: Memory Selection In Plans And Packs

## Goal

Select relevant memory cards in `prepare-task`, render them in `get-pack`, and
expose memory additively through MCP.

## Work

- Score fresh approved/deterministic cards by task overlap and selected source links.
- Add `selectedMemory` to plans.
- Add a capped `Selected memory` pack section.
- Add `ctxpack://repo/memory` without growing the MCP tool surface.

## Verification

- Compiler test proves selected memory appears in plans and packs.
- MCP resource tests cover the new URI.
