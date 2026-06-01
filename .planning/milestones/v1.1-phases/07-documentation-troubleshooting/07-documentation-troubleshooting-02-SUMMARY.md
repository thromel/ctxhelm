---
phase: 07-documentation-troubleshooting
plan: 02
subsystem: docs
tags: [agent-setup, mcp, codex, claude, cursor, opencode]
requires:
  - phase: 06-agent-setup-first-pack-adoption
    provides: repo-local adapter generation, setup-check, deterministic MCP smoke, optional client smokes
provides:
  - Agent setup matrix for Codex CLI, Claude Code, Cursor, and OpenCode
  - Proof taxonomy separating artifact checks, deterministic protocol proof, and optional real-client proof
affects: [phase-08-release-gates, docs, agent-adoption]
tech-stack:
  added: []
  patterns: [proof taxonomy documentation, explicit repo MCP examples]
key-files:
  created: [docs/agent-setup.md]
  modified: []
key-decisions:
  - "Client version availability is recorded separately from proof status; version presence is not treated as tool-call proof."
  - "Cursor and OpenCode are documented as generated-artifact plus deterministic-protocol supported, without claiming machine-checkable real-client tool-call proof."
patterns-established:
  - "Agent docs must distinguish generated artifact checks, deterministic protocol proof, and optional real-client proof."
requirements-completed: [DOCS-03, DOCS-04]
duration: 3min
completed: 2026-05-13
---

# Phase 07 Plan 02: Agent Setup Summary

**Agent setup matrix with explicit proof boundaries for Codex, Claude, Cursor, and OpenCode**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-13T19:08:30Z
- **Completed:** 2026-05-13T19:11:19Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added `docs/agent-setup.md` with generated artifacts, write scope, global-config mutation status, setup-check coverage, and proof status for all four target clients.
- Defined deterministic protocol proof as direct JSON-RPC/MCP smoke through `ctxhelm serve-mcp` with explicit-repo `prepare_task` and `get_pack` evidence.
- Documented optional real-client proof for Codex CLI and Claude Code while avoiding unsupported Cursor/OpenCode tool-call claims.

## Task Commits

1. **Task 1: Create agent setup matrix** - `3d6a5b0` (docs)
2. **Task 2: Explain proof boundaries and per-client setup notes** - `3f4302d` (docs)

## Files Created/Modified

- `docs/agent-setup.md` - Agent setup matrix, proof taxonomy, common MCP flow, and per-client setup notes.

## Decisions Made

- Local client version probes were used only as availability/evidence notes: Codex CLI `0.130.0`, Claude Code `2.1.140`, Cursor `3.3.30`, and OpenCode `1.14.25`.
- Cursor/OpenCode setup support remains artifact validation plus deterministic protocol proof in v1.1; docs do not claim machine-checkable client tool-call validation for those clients.

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03 can link troubleshooting and docs consistency checks against `docs/agent-setup.md` proof taxonomy terms.

## Verification

- `python3 -c 'from pathlib import Path; d=Path("docs/agent-setup.md").read_text(); required=["Codex CLI","Claude Code","Cursor","OpenCode","Generated artifact","Default write scope","global config","setup-check","deterministic protocol proof","real-client proof"]; missing=[s for s in required if s not in d]; assert not missing, missing'`
- `python3 -c 'from pathlib import Path; d=Path("docs/agent-setup.md").read_text(); required=["prepare_task","native file reads","get_pack","explicit `repo`","same MCP server session","Codex CLI","Claude Code","Cursor","OpenCode","does not claim machine-checkable Cursor","does not claim machine-checkable OpenCode"]; missing=[s for s in required if s not in d]; assert not missing, missing; forbidden=["Cursor real-client proof","OpenCode real-client proof","Cursor tool-call validation is verified","OpenCode tool-call validation is verified"]; assert not any(s in d for s in forbidden)'`
- `python3 -c 'from pathlib import Path; d=Path("docs/agent-setup.md").read_text(); assert "Codex CLI" in d and "OpenCode" in d and "deterministic protocol proof" in d'`

## Self-Check: PASSED

- Found created file: `docs/agent-setup.md`
- Found task commit: `3d6a5b0`
- Found task commit: `3f4302d`

---
*Phase: 07-documentation-troubleshooting*
*Completed: 2026-05-13*
