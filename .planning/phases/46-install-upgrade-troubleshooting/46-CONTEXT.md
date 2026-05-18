# Phase 46 Context: Install, Upgrade & Troubleshooting

**Created:** 2026-05-18
**Status:** Complete

## Goal

Users can install, upgrade, verify, and troubleshoot ctxpack without hidden
global mutation.

## Requirements

- INSTALL-01: Install docs cover source build and release archive install with
  checksum verification and absolute binary-path guidance.
- INSTALL-02: Upgrade docs verify active binary version, binary location,
  release manifest, and compatibility with existing local `.ctxpack` state.
- INSTALL-03: Agent setup docs cover Codex CLI, Claude Code, Cursor, OpenCode,
  and generic MCP clients while preserving manual config review.
- INSTALL-04: Troubleshooting flow covers PATH, MCP startup, stale binary,
  incompatible local state, wrong cwd, and setup-check failures.

## Decisions

- Add `ctxpack doctor` as the read-only install/upgrade verification command.
- Keep `setup-check` scoped to generated repo-local guidance artifacts.
- Use `doctor` for binary-path, manifest, and local-state compatibility checks.
- Do not mutate global agent config from any install or troubleshooting path.

## Non-goals

- No self-update.
- No package-manager install automation.
- No global MCP config writes.
- No real-client proof expansion beyond existing optional Codex/Claude smokes.
