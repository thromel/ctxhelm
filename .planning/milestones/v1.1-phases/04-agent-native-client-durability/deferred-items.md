# Deferred Items

## 2026-05-13 - Plan 02 out-of-scope workspace failure

- **Found during:** `cargo test --workspace`
- **Scope:** Out of scope for Plan 02; the failing file is `crates/ctxhelm/tests/cli_compat.rs`, which Plan 02 must not edit because Plan 01 may own it in parallel.
- **Failure:** `mcp_protocol_uses_explicit_repo_from_wrong_cwd` returned `pack resource is not available in this MCP session; call prepare_task first: ctxhelm://pack/brief.json`.
- **Action:** Not fixed in Plan 02. Keep with Plan 01 or the MCP durability follow-up plans that own CLI compatibility and protocol smoke behavior.

## 2026-05-13 - Plan 02 out-of-scope formatting drift

- **Found during:** `cargo fmt --all --check`
- **Scope:** Out of scope for Plan 02; diffs are in `crates/ctxhelm/tests/cli_compat.rs` and `crates/ctxhelm-mcp/src/lib.rs`, while Plan 02 owns only `crates/ctxhelm-core/src/init.rs`.
- **Failure:** rustfmt reported formatting changes outside the Plan 02 implementation file.
- **Action:** Not fixed in Plan 02 to avoid editing files owned by other Phase 4 plans or pre-existing formatting drift.
