# Coding Conventions

**Analysis Date:** 2026-05-13

## Naming Patterns

**Files:**
- Use Rust crate and module names in kebab-case for packages and snake_case for source files.
- Workspace crates live under `crates/ctxhelm/Cargo.toml`, `crates/ctxhelm-core/Cargo.toml`, `crates/ctxhelm-index/Cargo.toml`, `crates/ctxhelm-compiler/Cargo.toml`, and `crates/ctxhelm-mcp/Cargo.toml`.
- Library crates use `src/lib.rs`; the CLI crate uses `crates/ctxhelm/src/main.rs`.
- Domain modules in `ctxhelm-core` are split into small files: `crates/ctxhelm-core/src/contracts.rs`, `crates/ctxhelm-core/src/init.rs`, `crates/ctxhelm-core/src/privacy.rs`, and `crates/ctxhelm-core/src/repo.rs`.
- Larger implementation crates keep implementation in a single `src/lib.rs`: `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, and `crates/ctxhelm-mcp/src/lib.rs`.

**Functions:**
- Use snake_case for functions and methods: `prepare_context_plan`, `compile_context_pack_with_plan_and_paths_for_agent`, `render_pack_markdown`, `current_diff_summary`, and `run_stdio_server`.
- Public API functions use action-oriented names and explicit suffixes when behavior differs: `compile_context_pack`, `compile_context_pack_with_plan`, `compile_context_pack_with_plan_for_agent`, and `compile_context_pack_from_plan_for_agent` in `crates/ctxhelm-compiler/src/lib.rs`.
- Helper functions stay private unless they are part of the crate contract: `context_anchor_paths` in `crates/ctxhelm/src/main.rs`, `bounded_limit` in `crates/ctxhelm-mcp/src/lib.rs`, and `normalize_score` in `crates/ctxhelm-compiler/src/lib.rs`.
- Test names are behavior sentences in snake_case: `context_plan_serializes_with_camel_case_contract_fields`, `inventory_respects_ignores_and_default_exclusions`, and `prepare_task_call_returns_structured_context_plan`.

**Variables:**
- Use snake_case for locals and fields: `task_hash`, `target_agent`, `repo_root`, `safe_changed_files`, and `source_text_logged`.
- Use `repo` for a discovered `RepoRoot` and `repo_root` for a `Path` or `PathBuf` root path, matching `crates/ctxhelm/src/main.rs` and `crates/ctxhelm-index/src/lib.rs`.
- Use `args` for parsed CLI/MCP arguments in command handlers: `Command::PrepareTask(args)` in `crates/ctxhelm/src/main.rs` and `call_prepare_task(arguments)` in `crates/ctxhelm-mcp/src/lib.rs`.
- Use `output` for incrementally rendered Markdown/text buffers, as in `render_eval_checklist`, `render_historical_eval_report`, and `render_cards_report` in `crates/ctxhelm/src/main.rs`.

**Types:**
- Use PascalCase for structs and enums: `ContextPlan`, `ContextPack`, `InventoryError`, `SearchOptions`, `RpcError`, `HistoricalEvalReport`, and `AgentAdapter`.
- Use `Options` suffix for input configuration structs: `InventoryOptions`, `SearchOptions`, `SymbolOptions`, `DependencyOptions`, `HistoricalEvalOptions`, and `ContextCardsOptions`.
- Use `Report`, `Result`, `Summary`, `Entry`, and `Hint` suffixes for output contracts: `InventoryReport`, `SearchResult`, `CurrentDiffSummary`, `FileInventoryEntry`, and `CoChangeHint`.
- Use `Error` suffix for typed errors: `RepoRootError`, `InitError`, `InventoryError`, and `RpcError`.
- Use SCREAMING_SNAKE_CASE for constants: `PREPARE_TASK_TARGET_LIMIT`, `JSONRPC_VERSION`, `MCP_PROTOCOL_VERSION`, `AGENTS_SECTION_START`, and `AGENTS_SECTION_END`.

## Code Style

**Formatting:**
- Use standard `rustfmt` formatting.
- No repo-specific `rustfmt.toml` or `.rustfmt.toml` is detected.
- Preserve rustfmt import formatting and line wrapping. Examples include grouped imports in `crates/ctxhelm/src/main.rs` and `crates/ctxhelm-mcp/src/lib.rs`.
- Keep generated Markdown/text rendering readable with explicit `push_str`, `format!`, and small helper functions, as in `render_historical_eval_report` and `push_plain_path_list` in `crates/ctxhelm/src/main.rs`.

**Linting:**
- No checked-in `clippy.toml`, `deny.toml`, or CI lint config is detected.
- Prefer code that passes default compiler warnings and common Clippy expectations.
- Avoid introducing unused public APIs; public exports in `crates/ctxhelm-core/src/lib.rs` are intentionally limited to core contracts and top-level entry points.
- Use typed contracts instead of unstructured strings for behavior surfaces. Public structs in `crates/ctxhelm-core/src/contracts.rs`, `crates/ctxhelm-index/src/lib.rs`, and `crates/ctxhelm-compiler/src/lib.rs` are serializable data contracts.

## Import Organization

**Order:**
1. External crate and sibling crate imports first, grouped by crate.
2. Standard library imports after external/project imports.
3. Test-only imports inside `#[cfg(test)] mod tests`.

**Examples:**
```rust
use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ctxhelm_compiler::{prepare_context_plan_with_paths, render_pack_markdown};
use ctxhelm_core::{PackBudget, RepoRoot, TaskType};
use std::collections::BTreeSet;
use std::path::PathBuf;
```

This pattern is used in `crates/ctxhelm/src/main.rs`. `crates/ctxhelm-mcp/src/lib.rs` follows the same shape with `ctxhelm_compiler`, `ctxhelm_core`, `ctxhelm_index`, `serde`, `serde_json`, then `std`.

**Path Aliases:**
- No Rust path aliases are configured.
- Use workspace crate names directly: `ctxhelm_core`, `ctxhelm_index`, `ctxhelm_compiler`, and `ctxhelm_mcp`.
- Use package names with hyphens in `Cargo.toml` and underscore crate names in Rust imports: `ctxhelm-core` in `crates/ctxhelm-core/Cargo.toml` imports as `ctxhelm_core` in `crates/ctxhelm/src/main.rs`.

## Error Handling

**Patterns:**
- Use `thiserror::Error` for library/domain error enums in `crates/ctxhelm-core/src/repo.rs`, `crates/ctxhelm-core/src/init.rs`, and `crates/ctxhelm-index/src/lib.rs`.
- Include the failing path and source error in IO variants:
```rust
#[error("failed to read {path}: {source}")]
Read { path: PathBuf, source: io::Error },
```
- Map IO, serialization, and git failures at the boundary where context is available. `write_inventory` in `crates/ctxhelm-index/src/lib.rs` maps create, serialize, and write failures to `InventoryError`.
- Use `anyhow::Result` only in the CLI binary at `crates/ctxhelm/src/main.rs`; keep library crates on typed errors.
- Convert library errors into JSON-RPC errors at the MCP boundary in `crates/ctxhelm-mcp/src/lib.rs` with `RpcError::invalid_params(format!("failed to ...: {error}"))`.
- Return JSON-RPC method errors through `RpcError::method_not_found` and parse errors through `RpcError::parse_error` in `crates/ctxhelm-mcp/src/lib.rs`.
- Validate empty required strings before running expensive work. `call_prepare_task`, `call_get_pack`, and `call_search` in `crates/ctxhelm-mcp/src/lib.rs` reject empty task/query values.
- In tests, `unwrap()` and `unwrap_err()` are common and acceptable for fixture setup and direct failure assertions.

## Logging

**Framework:** console/stdout/stderr only

**Patterns:**
- CLI commands print user-facing reports or JSON to stdout with `println!` in `crates/ctxhelm/src/main.rs`.
- MCP server writes one JSON-RPC response per line to its writer in `run_server` in `crates/ctxhelm-mcp/src/lib.rs`.
- Git subprocess stderr is captured and converted into `InventoryError::Git` in `crates/ctxhelm-index/src/lib.rs`.
- Do not add ad hoc debug logging to library functions. Return structured reports or typed errors instead.

## Comments

**When to Comment:**
- Prefer self-describing function and test names over comments.
- Use comments sparingly for protocol-facing or generated text only when the literal content requires context.
- Avoid comments that restate simple Rust operations.

**JSDoc/TSDoc:**
- Not applicable. The codebase is Rust.
- Rustdoc comments are not used on the existing public API. Match the existing style unless adding a public API that needs external documentation.

## Function Design

**Size:** 
- Keep core-domain modules small where possible, as in `crates/ctxhelm-core/src/contracts.rs`, `crates/ctxhelm-core/src/privacy.rs`, and `crates/ctxhelm-core/src/repo.rs`.
- Large orchestration modules exist in `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, and `crates/ctxhelm-mcp/src/lib.rs`; add new helpers near related behavior rather than creating unrelated utility sections.
- Keep public functions focused on one operation and push rendering, filtering, and path normalization into private helpers.

**Parameters:** 
- Accept `impl AsRef<Path>` for public functions that take repo paths: `prepare_context_plan`, `compile_context_pack`, `write_inventory`, `lexical_search`, and `current_diff_summary`.
- Accept options structs for configurable operations: `InventoryOptions`, `SearchOptions`, `SymbolOptions`, `CoChangeOptions`, `DependencyOptions`, and `CurrentDiffOptions`.
- Use explicit typed enums for mode/budget rather than strings in Rust APIs: `TaskType` and `PackBudget` in `crates/ctxhelm-core/src/contracts.rs`.
- Use `Option<T>` for optional JSON/CLI inputs and apply defaults at the boundary, as in `PrepareTaskArgs`, `GetPackArgs`, and `SearchArgs` in `crates/ctxhelm-mcp/src/lib.rs`.

**Return Values:** 
- Return `Result<T, InventoryError>` for index/compiler operations that touch the filesystem, git, or serialized state.
- Return `Result<T, InitError>` for repo initialization operations in `crates/ctxhelm-core/src/init.rs`.
- Return `Result<Value, RpcError>` for MCP request handlers in `crates/ctxhelm-mcp/src/lib.rs`.
- Return serializable structs for behavior contracts, not formatted strings. Formatting belongs in boundary helpers such as `render_pack_markdown`, `render_eval_checklist`, and `tool_json_result`.

## Module Design

**Exports:** 
- `crates/ctxhelm-core/src/lib.rs` is the only barrel-style module. It exposes domain contracts via `pub use contracts::*`, selected init types, `PrivacyStatus`, `FileRole`, and `RepoRoot`.
- `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, and `crates/ctxhelm-mcp/src/lib.rs` expose public functions and structs directly from their crate root.
- Keep internal helper structs private unless they are consumed across crate boundaries. Examples include `RpcError` in `crates/ctxhelm-mcp/src/lib.rs` and `HistoricalEvalWorktree` in `crates/ctxhelm-compiler/src/lib.rs`.

**Barrel Files:** 
- Use a barrel file only for `ctxhelm-core`, where multiple small modules form the shared contract layer.
- Do not add new barrel files for `ctxhelm-index`, `ctxhelm-compiler`, or `ctxhelm-mcp` unless those crates are split into modules.

---

*Convention analysis: 2026-05-13*
