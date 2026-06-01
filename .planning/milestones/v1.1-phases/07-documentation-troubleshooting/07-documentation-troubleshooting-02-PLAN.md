---
phase: 07-documentation-troubleshooting
plan: 02
type: execute
wave: 1
depends_on: []
files_modified:
  - docs/agent-setup.md
autonomous: true
requirements: [DOCS-03, DOCS-04]
must_haves:
  truths:
    - "User can compare Codex CLI, Claude Code, Cursor, and OpenCode setup options in one matrix."
    - "User can see each agent's write scope, default mutation behavior, generated artifact or snippet, smoke support, and verified proof status."
    - "Docs distinguish deterministic protocol proof from real-client proof and avoid unsupported Cursor/OpenCode real-client validation claims."
  artifacts:
    - path: "docs/agent-setup.md"
      provides: "Agent setup matrix and per-client setup notes"
  key_links:
    - from: "docs/agent-setup.md"
      to: "Phase 6 generated guidance"
      via: "documents Codex/Claude/Cursor/OpenCode artifacts created by `ctxhelm init`"
      pattern: "Codex CLI|Claude Code|Cursor|OpenCode"
    - from: "docs/agent-setup.md"
      to: "deterministic protocol proof"
      via: "separate support row/section from optional real-client proof"
      pattern: "deterministic protocol proof|real-client proof"
---

<objective>
Document agent setup capabilities and proof boundaries.

Purpose: Users need one accurate comparison of supported agent surfaces, what ctxhelm writes, and what has machine-checkable proof.
Output: `docs/agent-setup.md` with matrix, per-agent setup notes, and deterministic-vs-real-client proof explanation.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/research/SUMMARY.md
@.planning/research/FEATURES.md
@.planning/research/PITFALLS.md
@.planning/phases/07-documentation-troubleshooting/07-CONTEXT.md
@.planning/phases/06-agent-setup-first-pack-adoption/VERIFICATION.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-02-SUMMARY.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-03-SUMMARY.md
@.planning/phases/06-agent-setup-first-pack-adoption/06-agent-setup-first-pack-adoption-04-SUMMARY.md
@README.md
@scripts/smoke-mcp-protocol.sh
@scripts/smoke-codex-mcp.sh
@scripts/smoke-claude-mcp.sh

<decision_trace>
- DOCS-03: Documentation must include an agent setup matrix for Codex CLI, Claude Code, Cursor, and OpenCode with write scope, smoke support, and verified client versions.
- DOCS-04: Documentation must distinguish deterministic protocol proof from real-client proof and must not claim unsupported Cursor/OpenCode real-client tool-call validation.
- Phase 6 decision: Generated setup remains repo-local or copy/paste-oriented and does not mutate global agent configuration by default.
- Phase 6 decision: Deterministic MCP protocol proof is the hard gate; Codex/Claude real-client smokes are optional/versioned.
- Scope guard: Do not implement Cursor/OpenCode real-client smoke proof in this docs phase.
</decision_trace>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create agent setup matrix</name>
  <files>docs/agent-setup.md</files>
  <action>Create `docs/agent-setup.md` with a matrix covering Codex CLI, Claude Code, Cursor, and OpenCode. Columns must include: generated artifact/snippet, default write scope, whether ctxhelm mutates global config by default, setup-check coverage, deterministic MCP protocol proof support, optional real-client proof status, and verified-version/evidence notes. Document that Codex setup is copy/paste-oriented, Claude has repo/project-local command/snippet guidance, Cursor uses `.cursor/rules/ctxhelm.mdc`, and OpenCode uses `.ctxhelm/adapters/opencode.jsonc.snippet`. Tell implementer to verify local client version commands when recording exact version strings; if a client is unavailable, state "not verified on this machine" rather than inventing a version.</action>
  <verify>
    <automated>python3 -c "from pathlib import Path; d=Path('docs/agent-setup.md').read_text(); required=['Codex CLI','Claude Code','Cursor','OpenCode','Generated artifact','Default write scope','global config','setup-check','deterministic protocol proof','real-client proof']; missing=[s for s in required if s not in d]; assert not missing, missing"</automated>
  </verify>
  <done>Agent setup docs expose the current setup support surface in one scannable matrix.</done>
</task>

<task type="auto">
  <name>Task 2: Explain proof boundaries and per-client setup notes</name>
  <files>docs/agent-setup.md</files>
  <action>Extend `docs/agent-setup.md` with per-client setup notes and a clear proof taxonomy. Include exact examples for explicit `repo` usage and progressive `prepare_task` -> native file reads -> `get_pack` flow. Define deterministic protocol proof as direct JSON-RPC/MCP smoke through `ctxhelm serve-mcp` with machine-checkable `prepare_task`/`get_pack` evidence. Define real-client proof as optional Codex CLI or Claude Code smoke evidence tied to exact client versions and request logs. Explicitly state that Cursor and OpenCode setup can be validated through generated artifact checks plus deterministic protocol proof, but v1.1 docs do not claim machine-checkable Cursor/OpenCode real-client tool-call proof.</action>
  <verify>
    <automated>python3 -c "from pathlib import Path; d=Path('docs/agent-setup.md').read_text(); required=['prepare_task','native file reads','get_pack','explicit `repo`','same MCP server session','Codex CLI','Claude Code','Cursor','OpenCode','does not claim machine-checkable Cursor','does not claim machine-checkable OpenCode']; missing=[s for s in required if s not in d]; assert not missing, missing; forbidden=['Cursor real-client proof','OpenCode real-client proof','Cursor tool-call validation is verified','OpenCode tool-call validation is verified']; assert not any(s in d for s in forbidden)"</automated>
  </verify>
  <done>`docs/agent-setup.md` satisfies DOCS-03 and DOCS-04 without overstating unsupported real-client validation.</done>
</task>

</tasks>

<verification>
- `python3 -c "from pathlib import Path; d=Path('docs/agent-setup.md').read_text(); assert 'Codex CLI' in d and 'OpenCode' in d and 'deterministic protocol proof' in d"`
</verification>

<success_criteria>
Plan 02 is complete when users can compare all four agent setup paths and understand exactly which proof level is deterministic, optional real-client, or not claimed.
</success_criteria>

<output>
After completion, create `.planning/phases/07-documentation-troubleshooting/07-documentation-troubleshooting-02-SUMMARY.md`
</output>
