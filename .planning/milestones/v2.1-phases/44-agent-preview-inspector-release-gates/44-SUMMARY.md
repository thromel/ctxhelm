# Phase 44 Summary: Agent Preview & Inspector Release Gates

**Completed:** 2026-05-18
**Status:** Complete

## Delivered

- Added `AgentPreviewReport` contracts for source-free agent consumption previews.
- Added `ctxpack agent preview` with JSON and Markdown output.
- Preview output covers Codex CLI, Claude Code, Cursor, OpenCode, generic MCP, and custom agent fallbacks.
- Previews show MCP tools/resources, `AGENTS.md`, native guidance paths, pack resource URIs, next steps, and the ctxpack-vs-agent ownership boundary.
- Added `docs/agent-preview.md`, connected architecture/components/data-contract docs, and `scripts/smoke-agent-preview.sh`.
- Wired agent preview checks into release docs, docs consistency checks, release gate, and release packaging contract tests.

## Validation

- `cargo fmt --all --check`
- `bash scripts/smoke-agent-preview.sh`
- `bash scripts/check-release-docs.sh`
- `bash -n scripts/release-gate.sh`
- `bash -n scripts/smoke-agent-preview.sh`
- `cargo run -p ctxpack -- agent preview --help`
- `cargo run -p ctxpack -- --help`
- `cargo test -p ctxpack release_gate_script_contract -- --nocapture`
- `cargo test -p ctxpack release_docs_check_passes -- --nocapture`
- `cargo test --workspace`

## Notes

- Preview artifacts remain source-free and do not include pack snippets.
- Source-bearing context still requires explicit pack materialization.
- ctxpack remains read-only: agents own file reads, edits, shell commands, and permission decisions.
