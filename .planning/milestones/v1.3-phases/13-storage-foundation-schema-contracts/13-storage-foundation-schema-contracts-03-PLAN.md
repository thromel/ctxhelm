---
phase: 13
plan: 3
title: Add schema version and migration history tracking
status: complete
wave: 3
depends_on: [13-storage-foundation-schema-contracts-02]
requirements_addressed: [STORE-03]
files_modified:
  - crates/ctxpack-index/src/storage.rs
autonomous: true
---

# Plan 3: Versioning And Migration History

## Objective

Make storage compatibility diagnosable by recording schema version, ctxpack version, ranking/compiler version, and migration history.

## Must Haves

- D-02: Diagnostics must show the concrete store path.
- D-10: Version and migration records are typed Rust values.
- STORE-03: Stale or incompatible storage can be diagnosed without opening source files.

## Tasks

<task id="13-03-T1">
<title>Add storage metadata records</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
- crates/ctxpack-index/src/inventory.rs
- crates/ctxpack-core/src/contracts.rs
</read_first>
<action>Add `StorageMetadata` persistence in a `storage_metadata` table with keys for `schema_version`, `ctxpack_version`, `ranking_version`, `compiler_version`, `privacy_mode`, `created_at_unix_seconds`, and `updated_at_unix_seconds`. Use `env!("CARGO_PKG_VERSION")` for the crate-local version value where appropriate and define explicit constants for ranking/compiler storage contract versions.</action>
<acceptance_criteria>
- `initialize_store` inserts or updates exactly one metadata row.
- `StorageReport` includes `ctxpack_version`, `ranking_version`, `compiler_version`, `privacy_mode`, and `database_path`.
- Re-running initialization updates `updated_at_unix_seconds` without changing `created_at_unix_seconds`.
</acceptance_criteria>
</task>

<task id="13-03-T2">
<title>Add migration history table and initializer migration record</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
</read_first>
<action>Create a `schema_migrations` table with `version`, `name`, `applied_at_unix_seconds`, and `checksum` fields. Insert migration version `1` named `initial_source_free_storage_schema` when schema creation completes. Ensure insertion is idempotent.</action>
<acceptance_criteria>
- Fresh initialization creates one migration row with version `1`.
- Second initialization still reports one row for version `1`.
- Migration checksum is deterministic for the initial schema string or migration identifier.
</acceptance_criteria>
</task>

<task id="13-03-T3">
<title>Detect incompatible schema versions</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
</read_first>
<action>Add `open_store_report(path: impl AsRef<Path>) -> Result<StorageReport, StorageError>` or equivalent inspection helper that reads metadata and reports compatible, missing metadata, or incompatible schema status. Do not auto-migrate beyond version `1` in this phase.</action>
<acceptance_criteria>
- Test database with `schema_version = 1` reports compatible.
- Test database with `schema_version = 999` returns or reports incompatible schema.
- Incompatible schema diagnostic includes database path and version numbers but no source-derived content.
</acceptance_criteria>
</task>

<task id="13-03-T4">
<title>Add stable diagnostics for missing or partial stores</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
- crates/ctxpack-core/src/contracts.rs
</read_first>
<action>Add storage diagnostic values for missing database, missing metadata, missing required tables, incompatible schema, and privacy mode mismatch. Keep these diagnostics source-free and suitable for later CLI/MCP surfacing.</action>
<acceptance_criteria>
- Unit tests assert diagnostic codes include `storage_missing`, `storage_missing_metadata`, `storage_missing_tables`, and `storage_incompatible_schema`.
- Diagnostic messages include `ctxpack.sqlite3` path information where relevant.
- Diagnostics do not include file contents, source snippets, prompt text, or commit subjects.
</acceptance_criteria>
</task>

## Verification

- `cargo test -p ctxpack-index storage`
- `cargo test --workspace`

