# Phase 7: Documentation & Troubleshooting - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Mode:** Autonomous smart-discuss defaults

<domain>
## Phase Boundary

This phase makes the current v1.1 install, setup, smoke, and support story understandable from docs alone. It covers README flow, troubleshooting docs, agent setup matrix, deterministic-vs-real-client proof explanation, PATH/CTXHELM_HOME/wrong-cwd/MCP startup guidance, and docs consistency checks. It does not add new product behavior unless a small validation script is required to keep docs accurate.

</domain>

<decisions>
## Implementation Decisions

### Documentation Structure
- Keep README as the short install-to-first-pack path for normal users.
- Put deeper operational reference in dedicated docs rather than overloading README.
- Prefer task-oriented docs: install, initialize, validate setup, run first pack, troubleshoot.
- Keep command snippets synchronized with actual CLI flags and scripts.

### Troubleshooting Scope
- Cover PATH and GUI/client environment differences, including absolute `ctxhelm` binary paths.
- Cover `CTXHELM_HOME`, local state cleanup, uninstall, wrong cwd, explicit `repo`, MCP startup failures, stdout cleanliness, and session-scoped pack resources.
- Explain that setup-check validates repo-local artifacts and does not run or mutate real agent clients.
- Keep troubleshooting local-only and privacy-aware.

### Agent Setup Matrix
- Include Codex CLI, Claude Code, Cursor, and OpenCode in one comparison table.
- Show write scope, default mutation behavior, generated artifact/snippet, smoke support, and verified-client-evidence status.
- Distinguish deterministic MCP protocol proof from optional real-client Codex/Claude proof.
- Do not claim Cursor/OpenCode real-client tool-call proof unless a machine-checkable smoke exists.

### the agent's Discretion
- The planner may choose exact doc filenames, but docs should remain easy to find from README.
- The implementation may extend existing docs checks if that is cleaner than introducing another docs test script.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `README.md` now includes release install guidance from Phase 5.
- `docs/release.md` documents binary packaging, checksum, fallback install, and release audit behavior.
- Phase 6 added `setup-check`, improved init reporting, and `scripts/smoke-first-pack.sh`.
- `scripts/check-release-docs.sh` already validates some release documentation consistency.
- `scripts/smoke-mcp-protocol.sh`, `scripts/smoke-codex-mcp.sh`, and `scripts/smoke-claude-mcp.sh` provide deterministic and optional proof paths.

### Established Patterns
- Docs should state what ctxhelm does locally and what it deliberately does not do.
- Do not present `cargo run` as the normal user path after Phase 5.
- Keep real-client claims tied to exact smoke evidence and client version context.
- Maintain source-free/privacy language where docs describe traces, cards, setup checks, and artifacts.

### Integration Points
- README links should point to docs files that exist in the repo.
- Docs checks should validate required command phrases and prevent unsupported claims from creeping in.
- Phase 8 will use the docs as part of final release gates, so Phase 7 should leave machine-checkable docs consistency hooks.

</code_context>

<specifics>
## Specific Ideas

- Add `docs/agent-setup.md` for the agent setup matrix and per-client instructions.
- Add `docs/troubleshooting.md` for PATH, CTXHELM_HOME, wrong-cwd, MCP startup, setup-check, and state cleanup.
- Update README to link release, setup, troubleshooting, and first-pack smoke docs.
- Extend docs checks to cover agent setup matrix, deterministic-vs-real-client proof language, and no unsupported Cursor/OpenCode real-client claims.

</specifics>

<deferred>
## Deferred Ideas

- UI or website documentation redesign.
- Generated man pages.
- Hosted docs site automation.
- Cursor/OpenCode real-client smoke proof.
- Any retrieval, packaging, or setup behavior outside doc accuracy.

</deferred>
