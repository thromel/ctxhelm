# Public Project Summary

Repo Context Packer, `ctxpack`, is a local-first context broker for AI coding
agents. Given a coding task, it compiles a small, task-conditioned evidence set:
likely target files, related tests, validation commands, graph/history/memory
signals, and agent-specific context guidance.

## Current Capabilities

- Agent-native setup through AGENTS.md, MCP, and thin Codex, Claude Code,
  Cursor, and OpenCode guidance.
- Source-free repository inventory, lexical retrieval, graph/test/history
  signals, local semantic metadata, memory cards, retrieval-health reports,
  pack inspector exports, and agent previews.
- Read-only MCP server with compact tools such as `prepare_task` and `get_pack`.
- Local release packaging, artifact audit, install doctor, deterministic MCP
  protocol proof, and release-gate smoke coverage.

## Accurate Non-claims

The public posture is source-free by default.

- ctxpack does not edit source code.
- ctxpack does not run user project tests automatically.
- ctxpack does not require cloud embeddings or cloud reranking.
- ctxpack does not mutate global agent configuration.
- ctxpack does not provide hosted sync, enterprise admin, signed installers, or
  self-update in the current release.

## One-sentence Portfolio Copy

`ctxpack` is a local, read-only context compiler that makes Codex, Claude Code,
Cursor, OpenCode, and other coding agents better at finding the right files,
tests, and constraints before they edit.
