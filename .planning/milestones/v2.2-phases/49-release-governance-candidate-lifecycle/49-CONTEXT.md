# Phase 49 Context: Release Governance & Candidate Lifecycle

Milestone: v2.2 Release & Distribution Hardening

Goal: make release readiness explicit and reversible while preserving local-only,
read-only, source-free release operations.

## Inputs

- Phase 45 release proof bundle and package audit.
- Phase 46 install doctor and stale-binary troubleshooting.
- Phase 47 public demo artifacts and public summary.
- Phase 48 distribution metadata, update metadata, and clean extraction archive
  verification.

## Constraints

- Candidate status metadata must be source-free.
- Deterministic MCP protocol proof is required; real-client proof remains
  optional for Codex CLI and Claude Code.
- Cursor and OpenCode real-client proof is not claimed for v1.1.0.
- Rollback must remove only marked local candidate artifacts and optionally
  restore previous metadata; it must not touch repo source.

