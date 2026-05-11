# Repo Context Packer Milestone 14: Active Context Anchors

## Goal

Finish the MVP agent-native loop by letting host agents pass active/open files into `prepare-task` and `get-pack` as ranked retrieval anchors.

## Scope

- Add repeatable CLI `--path <file>` anchors to `prepare-task` and `get-pack`.
- Add MCP `paths` arrays to `prepare_task` and `get_pack`.
- Keep anchor handling read-only and filtered through the safe inventory.
- Report unavailable anchors as risk flags instead of failing broad task planning.
- Prefer explicit anchors before symbol and lexical matches.

## Out of Scope

- Editor plugins.
- Cloud embeddings, cloud reranking, or remote indexing.
- Any file-editing behavior inside ctxpack.

## Verification

- `cargo fmt --all --check`
- `cargo test --workspace --locked --offline`
- `cargo clippy --locked --workspace --all-targets --offline -- -D warnings`
- `cargo run -p ctxpack -- --help`
- CLI smoke for `prepare-task --path`.
