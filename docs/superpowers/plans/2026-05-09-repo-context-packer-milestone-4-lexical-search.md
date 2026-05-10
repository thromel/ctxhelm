# Repo Context Packer Milestone 4: Lexical Search

**Goal:** Replace the `ctxpack search` stub with the first real retrieval path over the safe file inventory.

This milestone keeps search lightweight. It does not introduce Tantivy yet; it adds deterministic local lexical scoring over safe inventoried files so agents can retrieve exact paths, identifiers, and error-message-like text before the full BM25 layer exists.

## Scope

- Load an existing inventory from `~/.ctxpack/repos/<repo-id>/inventory.json`.
- Build the inventory automatically when the search command has no inventory yet.
- Search only files included in the safe inventory.
- Score path/name/token/content matches with deterministic weights.
- Return small typed `SearchResult` JSON objects.
- Wire `ctxpack search <query> --repo <path> --limit <n>`.

## Out of Scope

- Tantivy.
- Persistent inverted index files.
- Symbol extraction.
- Snippet line ranges.
- MCP `search` tool implementation.
- Vector search or reranking.

## Acceptance Checks

- Search finds source files by exact identifier content.
- Search finds files by path/name terms.
- Search does not read files excluded from inventory by ignore/sensitive/generated rules.
- Missing inventory is built automatically from the safe inventory defaults.
- `cargo test --workspace --locked` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
