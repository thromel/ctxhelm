# Milestone 7: Materialized Context Packs

## Goal

Turn `prepare-task` pack options into a real context-pack compiler that can emit structured JSON and agent-readable Markdown.

## Scope

1. Add typed `ContextPack` and `PackSection` contracts in `ctxpack-core`.
2. Add compiler support for budgeted packs built from the current context plan.
3. Materialize safe source and test snippets from plan-selected files.
4. Add a `ctxpack get-pack` CLI command with Markdown and JSON output.
5. Add focused tests for pack structure and snippet inclusion.

## Out of Scope

- Cloud embeddings or reranking.
- Autonomous editing.
- Persistent pack storage.
- Deep MCP resource serving for packs.

## Verification

- `cargo fmt --all --check`
- `cargo test --workspace --locked --offline`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo run -p ctxpack -- --help`
- Smoke-test `ctxpack get-pack` on a temporary repository.
