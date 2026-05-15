---
phase: 13
plan: 4
title: Add source-free storage privacy tests and fixtures
status: complete
wave: 4
depends_on: [13-storage-foundation-schema-contracts-03]
requirements_addressed: [STORE-04]
files_modified:
  - crates/ctxpack-index/src/storage.rs
  - crates/ctxpack-index/src/lib.rs
autonomous: true
---

# Plan 4: Source-Free Privacy Tests

## Objective

Prove the new SQLite storage foundation does not persist source snippets, prompt text, secrets, or raw file contents by default.

## Must Haves

- D-05: Strict source-free schema is enforced by tests.
- D-07: Future source-bearing persistence requires a separate explicit opt-in design.
- D-11: Existing JSON/JSONL behavior remains non-breaking.
- D-12: JSON/JSONL fallback remains available during the transition.

## Tasks

<task id="13-04-T1">
<title>Add fixture that attempts to leak source text into storage</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
- crates/ctxpack-index/src/inventory.rs
- crates/ctxpack-index/src/traces.rs
</read_first>
<action>Create a storage privacy test fixture with a temporary git repo containing a safe source file whose body includes unique sentinel strings such as `CTXPACK_SHOULD_NOT_STORE_SOURCE_BODY` and `CTXPACK_SHOULD_NOT_STORE_PROMPT_TEXT`. Initialize the store and insert or sync only source-free metadata supported by Phase 13.</action>
<acceptance_criteria>
- Test fixture writes a source file containing `CTXPACK_SHOULD_NOT_STORE_SOURCE_BODY`.
- Test fixture includes a simulated prompt/task sentinel `CTXPACK_SHOULD_NOT_STORE_PROMPT_TEXT` only in test input, not in expected persisted fields.
- The test initializes SQLite storage successfully.
</acceptance_criteria>
</task>

<task id="13-04-T2">
<title>Assert SQLite database bytes do not contain forbidden sentinels</title>
<read_first>
- crates/ctxpack-index/src/storage.rs
</read_first>
<action>After store initialization and metadata writes, read the SQLite database bytes from disk and assert the forbidden sentinel strings are absent. Also query text columns from all required tables and assert sentinel strings are absent from returned values.</action>
<acceptance_criteria>
- Test fails if `CTXPACK_SHOULD_NOT_STORE_SOURCE_BODY` appears in database bytes.
- Test fails if `CTXPACK_SHOULD_NOT_STORE_PROMPT_TEXT` appears in database bytes.
- Test fails if either sentinel appears in any SQLite text column.
</acceptance_criteria>
</task>

<task id="13-04-T3">
<title>Verify JSON and JSONL fallback behavior remains intact</title>
<read_first>
- crates/ctxpack-index/src/inventory.rs
- crates/ctxpack-index/src/traces.rs
- crates/ctxpack-index/src/storage.rs
</read_first>
<action>Add regression tests proving `write_inventory`, `load_or_build_inventory`, `append_eval_trace`, and `list_eval_traces` still operate with their current JSON/JSONL paths when SQLite storage is initialized but not required by the caller.</action>
<acceptance_criteria>
- Existing inventory path remains `repos/<repo-id>/inventory.json`.
- Existing trace path remains `repos/<repo-id>/traces.jsonl`.
- Initializing SQLite does not delete, require, or rewrite the JSON/JSONL files.
- `cargo test -p ctxpack-index write_inventory_persists_under_ctxpack_home` still passes.
</acceptance_criteria>
</task>

<task id="13-04-T4">
<title>Run phase-level validation and update planning status</title>
<read_first>
- .planning/ROADMAP.md
- .planning/REQUIREMENTS.md
- .planning/phases/13-storage-foundation-schema-contracts/13-CONTEXT.md
- .planning/phases/13-storage-foundation-schema-contracts/13-RESEARCH.md
</read_first>
<action>Run the full validation commands for Phase 13 after all storage foundation changes land. If implementation changes CLI help or docs, include the relevant help/doc checks. Update only the plan summary after execution, not during planning.</action>
<acceptance_criteria>
- `cargo test -p ctxpack-index storage` exits 0.
- `cargo test --workspace` exits 0.
- `cargo run -p ctxpack -- --help` exits 0.
- No source-bearing strings from privacy fixtures appear in SQLite database files.
</acceptance_criteria>
</task>

## Verification

- `cargo test -p ctxpack-index storage`
- `cargo test --workspace`
- `cargo run -p ctxpack -- --help`
