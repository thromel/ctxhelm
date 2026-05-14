---
phase: 13
plan: 2
title: Create source-free schema tables and record contracts
status: complete
wave: 2
depends_on: [13-storage-foundation-schema-contracts-01]
requirements_addressed: [STORE-02, STORE-04]
files_modified:
  - crates/ctxpack-index/src/storage.rs
autonomous: true
---

# Plan 2: Source-Free Schema Tables

## Objective

Create the broad, source-free SQLite schema skeleton required by v1.3 without wiring every runtime consumer yet.

## Must Haves

- D-05: No raw source text, prompt text, snippet bodies, commit subjects, or secret-bearing columns.
- D-06: Store only stable handles, paths, roles, hashes, line ranges, IDs, warnings, privacy status, and metrics.
- D-08: Define broad future-ready tables for repository intelligence and proof artifacts.
- D-09: Keep columns minimal; full consumer wiring happens later.

## Tasks

<task id="13-02-T1">
<title>Add schema creation for repository intelligence tables</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
- crates/ctxpack-index/src/inventory.rs
- crates/ctxpack-core/src/contracts.rs
</read_first>
<action>Extend store initialization to create source-free tables named `repos`, `files`, `symbols`, `chunks`, `edges`, `tests`, and `git_history`. Include primary keys, repo ID foreign keys, safe relative path fields, role/kind fields, hash fields, line-number fields, weight/confidence fields where needed, and metadata JSON fields only for source-free structured metadata.</action>
<acceptance_criteria>
- A test opens the initialized database and finds tables `repos`, `files`, `symbols`, `chunks`, `edges`, `tests`, and `git_history`.
- None of those tables contains columns named `source`, `content`, `snippet`, `prompt`, `secret`, or `raw_text`.
- Foreign key constraints reference `repos(repo_id)` where records are repo-scoped.
</acceptance_criteria>
</task>

<task id="13-02-T2">
<title>Add schema creation for trace, pack, benchmark, and proof tables</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
- crates/ctxpack-core/src/contracts.rs
- crates/ctxpack-compiler/src/eval.rs
</read_first>
<action>Extend store initialization to create source-free tables named `eval_traces`, `context_plans`, `context_packs`, `benchmark_runs`, `benchmark_metrics`, `retrieval_gaps`, and `proof_reports`. Store task hashes, repo snapshot hashes, budgets, target agents, selected candidate IDs, warnings, confidence, suite IDs, revision IDs, metric names, metric values, and privacy status. Do not store task text, prompt text, snippets, commit subjects, or source text.</action>
<acceptance_criteria>
- A test finds all seven tables after initialization.
- `context_packs` includes `pack_id`, `task_hash`, `budget`, `target_agent`, `confidence`, and `privacy_status`.
- `benchmark_metrics` stores metric names and numeric values without source labels beyond safe paths or IDs.
- No created table contains a prohibited source-bearing column name.
</acceptance_criteria>
</task>

<task id="13-02-T3">
<title>Add typed table-name and schema-inspection helpers</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
</read_first>
<action>Add a typed constant list or enum-backed helper for required table names and a `inspect_store_schema(path: impl AsRef<Path>) -> Result<StorageSchemaReport, StorageError>` helper that reports database path, schema version, privacy mode, and present/missing table names.</action>
<acceptance_criteria>
- `StorageSchemaReport.missing_tables` is empty immediately after `initialize_store`.
- A test against an empty temporary SQLite file reports missing required tables.
- The report contains no source-derived strings.
</acceptance_criteria>
</task>

<task id="13-02-T4">
<title>Document source-free schema invariants in code tests</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
- .planning/phases/13-storage-foundation-schema-contracts/13-CONTEXT.md
</read_first>
<action>Add tests that collect `PRAGMA table_info` for every required table and assert prohibited column names are absent. Use the prohibited fragments `source`, `content`, `snippet`, `prompt`, `secret`, `raw`, and `subject` unless the column is exactly `content_hash`.</action>
<acceptance_criteria>
- Test fails if a future developer adds `source_text`, `prompt`, `snippet_body`, `secret_value`, `raw_content`, or `commit_subject`.
- `content_hash` remains allowed.
- `cargo test -p ctxpack-index storage` passes.
</acceptance_criteria>
</task>

## Verification

- `cargo test -p ctxpack-index storage`
- `cargo test --workspace`

