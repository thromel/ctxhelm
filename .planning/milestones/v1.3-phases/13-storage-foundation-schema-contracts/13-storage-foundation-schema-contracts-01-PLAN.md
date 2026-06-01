---
phase: 13
plan: 1
title: Define storage contracts and initialize SQLite store
status: complete
wave: 1
depends_on: []
requirements_addressed: [STORE-01, STORE-03]
files_modified:
  - Cargo.toml
  - crates/ctxhelm-index/Cargo.toml
  - crates/ctxhelm-index/src/lib.rs
  - crates/ctxhelm-index/src/storage.rs
autonomous: true
---

# Plan 1: Storage Contracts And Initialization

## Objective

Add the first source-free SQLite storage module with explicit path resolution, store metadata, and idempotent initialization.

## Must Haves

- D-01: Default durable storage remains under `CTXHELM_HOME` or `~/.ctxhelm`.
- D-02: Store path is explicit in metadata and diagnostics.
- D-04: Explicit store path override exists for tests and advanced workflows.
- D-10: Storage-facing values are typed Rust contracts, not raw row maps.

## Tasks

<task id="13-01-T1">
<title>Add rusqlite dependency at the index layer</title>
<read_first>
- Cargo.toml
- crates/ctxhelm-index/Cargo.toml
- .planning/phases/13-storage-foundation-schema-contracts/13-CONTEXT.md
- .planning/phases/13-storage-foundation-schema-contracts/13-RESEARCH.md
</read_first>
<action>Add workspace dependency `rusqlite = { version = "0.39.0", features = ["bundled"] }` in root `Cargo.toml`, then add `rusqlite = { workspace = true }` to `crates/ctxhelm-index/Cargo.toml`. Do not add SQLite dependencies to `crates/ctxhelm`, `ctxhelm-core`, `ctxhelm-compiler`, or `ctxhelm-mcp` in this plan.</action>
<acceptance_criteria>
- Root `Cargo.toml` contains workspace dependency key `rusqlite`.
- `crates/ctxhelm-index/Cargo.toml` contains `rusqlite = { workspace = true }`.
- `cargo metadata --no-deps` succeeds.
</acceptance_criteria>
</task>

<task id="13-01-T2">
<title>Create typed storage path and metadata contracts</title>
<read_first>
- crates/ctxhelm-index/src/inventory.rs
- crates/ctxhelm-index/src/lib.rs
- crates/ctxhelm-index/src/storage.rs
</read_first>
<action>Create `crates/ctxhelm-index/src/storage.rs` with `STORAGE_SCHEMA_VERSION: u32 = 1`, `StoreConfig`, `StorePaths`, `StorageMetadata`, `StoragePrivacyMode`, `StorageOpenMode`, and `StorageReport`. Implement default path resolution that uses `CTXHELM_HOME/repos/<repo-id>/ctxhelm.sqlite3`, then `HOME/.ctxhelm/repos/<repo-id>/ctxhelm.sqlite3`, then `.ctxhelm/repos/<repo-id>/ctxhelm.sqlite3`. Include explicit override support through `StoreConfig { path_override: Option<PathBuf> }`.</action>
<acceptance_criteria>
- `crates/ctxhelm-index/src/storage.rs` defines `STORAGE_SCHEMA_VERSION`.
- `StorePaths` exposes the concrete SQLite path as `database_path`.
- Default path tests assert the path ends with `repos/<repo-id>/ctxhelm.sqlite3`.
- Explicit override test asserts the override path is used exactly.
</acceptance_criteria>
</task>

<task id="13-01-T3">
<title>Implement idempotent store initialization</title>
<read_first>
- crates/ctxhelm-index/src/storage.rs
- crates/ctxhelm-index/src/inventory.rs
</read_first>
<action>Implement `initialize_store(repo_root: impl AsRef<Path>, config: &StoreConfig) -> Result<StorageReport, StorageError>`. It must canonicalize the repo root, reuse `repo_id_for_path`, create the parent directory, open the SQLite database with `rusqlite::Connection`, enable foreign keys with `PRAGMA foreign_keys = ON`, create metadata and migration tables, insert or update the repo metadata row, and return a `StorageReport` containing repo ID, repo root, database path, schema version, and privacy mode.</action>
<acceptance_criteria>
- Calling `initialize_store` twice on the same repo succeeds without duplicate-key errors.
- `StorageReport.schema_version == 1`.
- SQLite query `PRAGMA foreign_keys` returns `1` in a unit test.
- Initialization does not create or modify `.ctxhelm/` inside the target repo unless an explicit override points there.
</acceptance_criteria>
</task>

<task id="13-01-T4">
<title>Expose storage module through index facade</title>
<read_first>
- crates/ctxhelm-index/src/lib.rs
- crates/ctxhelm-index/src/storage.rs
</read_first>
<action>Add `mod storage;` and re-export only the stable storage APIs needed by later plans: `initialize_store`, `StoreConfig`, `StorePaths`, `StorageMetadata`, `StoragePrivacyMode`, `StorageReport`, and `StorageError`.</action>
<acceptance_criteria>
- `ctxhelm_index::initialize_store` compiles from an external crate test or existing integration test.
- No CLI command changes are required in this plan.
- `cargo test -p ctxhelm-index storage` passes.
</acceptance_criteria>
</task>

## Verification

- `cargo test -p ctxhelm-index storage`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`

