# AGENTS.md

## Project Goal

Build ctxhelm, powered by the `ctxhelm` CLI, as a local-first, read-only
context broker for coding agents.

## Working Rules

- Keep the MVP agent-native: AGENTS.md, MCP, and thin native rules are the product surface.
- Do not add autonomous editing behavior to ctxhelm.
- Do not add cloud indexing, cloud embeddings, or cloud reranking by default.
- Prefer small typed contracts over stringly typed command output.
- Add focused tests for behavior that affects context selection, privacy, or generated agent instructions.

## Validation

- Run `cargo test --workspace` before claiming implementation work is complete.
- Run `cargo run -p ctxhelm -- --help` after CLI changes.
