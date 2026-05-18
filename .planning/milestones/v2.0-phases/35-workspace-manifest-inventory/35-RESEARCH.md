# Phase 35 Research: Workspace Manifest & Inventory

## Goal

Research what the planner needs to know before implementing Phase 35: multi-repo workspace manifests and source-free workspace inventory/status.

## Phase Boundary

Phase 35 should only establish the workspace foundation:

- typed workspace manifest contracts
- local manifest parsing/loading
- source-free workspace status aggregation
- diagnostics for bad workspace entries
- CLI-facing status/init surface
- tests proving source-free behavior and single-repo compatibility

Cross-repo task routing, workspace packs, shared artifact import/export, team policy templates, and MCP resources belong to later phases.

## Existing Architecture To Reuse

- `crates/ctxpack-core/src/contracts.rs` owns public JSON-compatible contracts.
- `crates/ctxpack-index/src/lib.rs` re-exports index/storage/privacy functions from focused modules.
- `crates/ctxpack-index/src/storage.rs` already exposes source-free storage status APIs that can be reused per repo.
- `crates/ctxpack/src/main.rs` owns clap command definitions and Markdown/JSON renderers.
- Existing tests prefer temp repositories, `CTXPACK_HOME`, and sentinel checks that prove source text is not persisted.

## Recommended Defaults

- Manifest location: `.ctxpack/workspace.json` for the first implementation.
- Paths: allow repo-relative and absolute local paths; normalize/canonicalize during load.
- Repo IDs: allow optional explicit IDs; otherwise derive the existing stable repo ID through `RepoRoot`.
- Labels/tags: source-free user-provided strings with validation against newlines/control text.
- Status scope: include repo root label, repo ID, manifest path, inventory counts, storage compatibility, memory card count, feedback/profile paths/status where cheap, privacy status, and diagnostics.
- Privacy: workspace reports may include local paths in local CLI output, but all shareable/exportable workspace artifacts must keep source text absent and mark `localOnly`.

## Risks

- Accidentally flattening several repos into one inventory. Keep per-repo records separate.
- Adding workspace logic directly into compiler/retrieval too early. Phase 36 owns routing.
- Leaking source text or raw prompts from cards/feedback while aggregating status. Only aggregate metadata/counts.
- Breaking existing single-repo commands. Add compatibility tests that no manifest is required.

## Plan Input

The plan should create one executable implementation plan covering contracts, index helpers, CLI surface, tests, and docs/release checks only if needed for Phase 35.
