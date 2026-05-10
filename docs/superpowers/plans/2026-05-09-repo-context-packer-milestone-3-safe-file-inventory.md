# Repo Context Packer Milestone 3: Safe File Inventory

**Goal:** Add the first real repository-intelligence layer: a local-only file inventory that includes useful repo files and excludes sensitive/generated noise by default.

This milestone is intentionally pre-search. The output is a typed inventory that later lexical indexing, test mapping, and context compilation can consume.

## Scope

- Walk a repository using safe ignore rules.
- Respect `.gitignore`, `.ctxpackignore`, and `.cursorignore`.
- Classify files as source, test, config, schema, docs, generated, sensitive, or unknown.
- Exclude sensitive and generated files from the default inventory.
- Compute a stable content hash for included files.
- Persist inventory JSON under `~/.ctxpack/repos/<repo-id>/inventory.json`.
- Wire `ctxpack index` to build and write the inventory.

## Out of Scope

- SQLite storage.
- Tantivy/BM25 indexing.
- Tree-sitter parsing.
- Embeddings.
- Git history.
- Test command inference.

## Acceptance Checks

- A temp repo with `.gitignore`, `.ctxpackignore`, and `.cursorignore` excludes matching files.
- Sensitive files such as `.env`, keys, certs, and dumps are excluded.
- Generated/dependency/build outputs are excluded.
- Source, test, config, schema, and docs files are classified.
- `ctxpack index --repo <repo>` writes `inventory.json` in a private ctxpack state directory.
- `cargo test --workspace --locked` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
