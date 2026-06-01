# Phase 13: Storage Foundation & Schema Contracts - Research

**Researched:** 2026-05-14
**Status:** Complete

## Research Summary

Phase 13 should add a small SQLite storage layer under `crates/ctxhelm-index` while preserving the current JSON/JSONL state paths as fallback. The best dependency choice for the current Rust CLI is `rusqlite = "0.39.0"` with the `bundled` feature so the local CLI remains portable and does not require users to have a system SQLite development package installed.

## Relevant Existing Patterns

- `crates/ctxhelm-index/src/inventory.rs` already owns `CTXHELM_HOME`, repo IDs, inventory path resolution, source-free inventory metadata, and JSON inventory persistence.
- `crates/ctxhelm-index/src/traces.rs` already owns source-free trace append/list behavior under `~/.ctxhelm/repos/<repo-id>/traces.jsonl`.
- `crates/ctxhelm-index/src/freshness.rs` already compares stored inventory metadata against current repo state and emits stale diagnostics.
- Workspace dependency versions are centralized in root `Cargo.toml` and consumed from crate manifests.
- Tests use `tempfile` plus isolated `CTXHELM_HOME`; storage tests should follow that pattern.

## Dependency Notes

- `rusqlite 0.39.0` is the current crates.io version checked during planning.
- Use `rusqlite` in `crates/ctxhelm-index`, not the CLI crate, so storage is available to compiler and MCP consumers through the index facade.
- Prefer `bundled` for portability. This is a local CLI binary, not a server deployment; avoiding system SQLite setup friction is worth the slightly larger build.

## Planning Implications

- Add a `storage` module with explicit path resolution and typed metadata before wiring any runtime behavior.
- Create broad source-free tables early, but keep writes narrow and testable in Phase 13.
- Keep JSON/JSONL fallback intact. Do not make SQLite mandatory for existing `index`, `prepare-task`, `get-pack`, or MCP flows in this phase.
- Add privacy tests that inspect the SQLite database bytes/text and prove raw source text and prompt text are absent.

## Validation Architecture

Phase 13 validation should include:

- Unit tests for store path resolution under `CTXHELM_HOME`, `HOME`, and explicit override.
- Unit tests for schema initialization and idempotent migration records.
- Unit tests or integration tests that initialize a store and assert all required table names exist.
- Privacy fixture test that creates repo files containing unique source strings and verifies the SQLite file does not contain those strings.
- Workspace validation with `cargo test --workspace` and `cargo run -p ctxhelm -- --help`.

## Open Risks

- Broad schema can become speculative if too many consumer-specific columns are added. Keep columns minimal and source-free.
- `rusqlite` bundled builds may increase compile time. Accept this for portability unless tests reveal unacceptable overhead.
- Directly replacing JSON persistence would be risky. Keep the transition additive in Phase 13.

---

*Phase: 13-Storage Foundation & Schema Contracts*
