# Phase 44 Context: Agent Preview & Inspector Release Gates

**Created:** 2026-05-18
**Status:** Active

## Goal

Users can preview how ctxhelm will present context to Codex, Claude Code,
Cursor, OpenCode, and generic MCP clients while preserving the product boundary:
ctxhelm recommends files, packs, tests, tools, and constraints; coding agents
own file reads, edits, shell commands, and approvals.

## Requirements

- AGPREV-01: Preview Codex, Claude Code, Cursor, OpenCode, and generic MCP.
- AGPREV-02: Show MCP tools/resources, AGENTS.md guidance, native rules, pack
  resource URIs, and recommended next steps.
- AGPREV-03: Preserve the read-only context-broker boundary.
- AGPREV-04: Add release-gate smoke coverage proving source-free metadata,
  explicit source-bearing pack paths, no hidden cloud use, and compatibility.

## Existing Dependencies

- Phase 39: `PackInspectorView`
- Phase 40: static/local inspector HTML export
- Phase 41: retrieval-health reports
- Phase 42: graph neighborhoods
- Phase 43: semantic provider and policy experiment reports
- Existing adapter guidance from `ctxhelm_core::init`

## Non-goals

- No agent automation.
- No global agent config mutation.
- No source-bearing preview unless the user explicitly exports a pack.
- No real-client proof expansion beyond existing Codex/Claude smoke hooks.
