# Architecture

**Analysis Date:** 2026-05-13

## Pattern Overview

**Overall:** Layered Rust workspace with local-only repository indexing, task-conditioned context planning, pack compilation, and transport adapters.

**Key Characteristics:**
- Keep public contracts centralized in `crates/ctxpack-core/src/contracts.rs` and re-exported from `crates/ctxpack-core/src/lib.rs`.
- Keep repository scanning and retrieval in `crates/ctxpack-index/src/lib.rs`; callers receive structured values, not formatted CLI strings.
- Keep context-plan and context-pack construction in `crates/ctxpack-compiler/src/lib.rs`; CLI and MCP entry points call the same compiler APIs.
- Keep protocol handling in `crates/ctxpack-mcp/src/lib.rs`; it translates JSON-RPC/MCP requests into core/compiler/index calls.
- Keep command parsing and terminal rendering in `crates/ctxpack/src/main.rs`; it is the outer orchestration layer.

## Layers

**Core Contracts:**
- Purpose: Define stable data contracts, repository root discovery, privacy metadata, and init artifacts shared across crates.
- Location: `crates/ctxpack-core/src/`
- Contains: Serializable structs and enums in `crates/ctxpack-core/src/contracts.rs`, repo discovery in `crates/ctxpack-core/src/repo.rs`, privacy status in `crates/ctxpack-core/src/privacy.rs`, and init file generation in `crates/ctxpack-core/src/init.rs`.
- Depends on: `serde`, `thiserror`, and `uuid` from `crates/ctxpack-core/Cargo.toml`.
- Used by: `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, `crates/ctxpack-mcp/src/lib.rs`, and `crates/ctxpack/src/main.rs`.

**Repository Index And Retrieval:**
- Purpose: Build a safe file inventory, classify files, search files and symbols, infer related tests, derive dependency edges, read current diff paths, sample git history, and persist eval traces.
- Location: `crates/ctxpack-index/src/lib.rs`
- Contains: Inventory APIs such as `build_inventory`, `write_inventory`, and `load_or_build_inventory`; retrieval APIs such as `lexical_search`, `symbol_search`, `related_tests`, `co_change_hints`, `dependency_edges`, `related_dependency_edges`, `current_diff_summary`, `historical_commit_samples`, `append_eval_trace`, and `list_eval_traces`.
- Depends on: `ctxpack-core`, `ignore`, `blake3`, `serde`, `serde_json`, `thiserror`, `uuid`, local filesystem access, and local `git` commands.
- Used by: `crates/ctxpack-compiler/src/lib.rs`, `crates/ctxpack-mcp/src/lib.rs`, and `crates/ctxpack/src/main.rs`.

**Context Compiler:**
- Purpose: Convert a task and optional anchors into a `ContextPlan`, compile budgeted `ContextPack` values, render packs and context cards, and run source-free historical retrieval evaluation.
- Location: `crates/ctxpack-compiler/src/lib.rs`
- Contains: Plan APIs such as `prepare_context_plan_with_paths`, pack APIs such as `compile_context_pack_with_plan_and_paths_for_agent`, rendering APIs such as `render_pack_markdown`, eval APIs such as `eval_trace_for_plan`, `eval_trace_for_pack`, and `evaluate_historical_commits`, and card generation in `generate_context_cards`.
- Depends on: `ctxpack-core`, `ctxpack-index`, `serde`, `tempfile`, `uuid`, local filesystem access, and local `git` commands for historical worktree evaluation.
- Used by: `crates/ctxpack-mcp/src/lib.rs` and `crates/ctxpack/src/main.rs`.

**MCP Transport:**
- Purpose: Expose ctxpack through a local stdio JSON-RPC server with MCP tools, resources, and prompts.
- Location: `crates/ctxpack-mcp/src/lib.rs`
- Contains: Server loop in `run_stdio_server` and `run_server`, request dispatch in `handle_request`, MCP tool handlers such as `call_prepare_task`, `call_search`, `call_related`, `call_get_pack`, `call_related_tests`, and `call_current_diff`, resource handlers such as `read_resource`, and prompt handlers such as `get_prompt`.
- Depends on: `ctxpack-core`, `ctxpack-index`, `ctxpack-compiler`, `serde`, and `serde_json`.
- Used by: The CLI command `serve-mcp` in `crates/ctxpack/src/main.rs`.

**CLI Application:**
- Purpose: Parse terminal commands, discover repository roots, call library APIs, append eval traces where applicable, and render JSON or Markdown/text output.
- Location: `crates/ctxpack/src/main.rs`
- Contains: Clap command definitions, argument structs, mode/budget/format enums, `main`, helper conversion functions, output renderers, and CLI-specific anchor handling.
- Depends on: `ctxpack-core`, `ctxpack-index`, `ctxpack-compiler`, `ctxpack-mcp`, `anyhow`, `clap`, and `serde_json`.
- Used by: Developers running `cargo run -p ctxpack -- ...`; MCP clients launch the same binary through `ctxpack serve-mcp`.

**Planning Documentation:**
- Purpose: Preserve product specs and milestone implementation plans.
- Location: `docs/superpowers/`
- Contains: Product spec in `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md` and milestone plans in `docs/superpowers/plans/`.
- Depends on: Not applicable.
- Used by: Human and agent planning workflows; runtime code does not import these documents.

## Data Flow

**CLI Prepare Task Flow:**

1. `crates/ctxpack/src/main.rs` parses `ctxpack prepare-task` into `PrepareTaskArgs`.
2. `RepoRoot::discover_from` in `crates/ctxpack-core/src/repo.rs` resolves the active repository from `--repo` or the current directory.
3. `context_anchor_paths` in `crates/ctxpack/src/main.rs` merges explicit `--path` anchors with safe paths from `current_diff_summary` in `crates/ctxpack-index/src/lib.rs` when `--current-diff` is present.
4. `prepare_context_plan_with_paths` in `crates/ctxpack-compiler/src/lib.rs` builds a `ContextPlan` using `symbol_search`, `lexical_search`, `related_tests`, `co_change_hints`, and `related_dependency_edges` from `crates/ctxpack-index/src/lib.rs`.
5. `eval_trace_for_plan` in `crates/ctxpack-compiler/src/lib.rs` creates a source-free trace, and `append_eval_trace` in `crates/ctxpack-index/src/lib.rs` appends it under `~/.ctxpack/repos/<repo-id>/traces.jsonl`.
6. `crates/ctxpack/src/main.rs` prints the `ContextPlan` as pretty JSON.

**CLI Get Pack Flow:**

1. `crates/ctxpack/src/main.rs` parses `ctxpack get-pack` into task, mode, budget, format, anchors, current-diff inclusion, and target-agent arguments.
2. `compile_context_pack_with_plan_and_paths_for_agent` in `crates/ctxpack-compiler/src/lib.rs` calls `prepare_context_plan_with_paths`, then `compile_pack_from_plan`.
3. `compile_pack_from_plan` in `crates/ctxpack-compiler/src/lib.rs` creates a `ContextPack` with task, target file, validation, risk flag, source snippet, test snippet, and final checklist sections according to `PackBudget`.
4. `eval_trace_for_pack` and `append_eval_trace` record source-free retrieval metadata through `crates/ctxpack-compiler/src/lib.rs` and `crates/ctxpack-index/src/lib.rs`.
5. `crates/ctxpack/src/main.rs` prints Markdown via `render_pack_markdown` or structured JSON via `serde_json`.

**MCP Prepare Task Flow:**

1. `ctxpack serve-mcp` dispatches from `crates/ctxpack/src/main.rs` to `ctxpack_mcp::run_stdio_server` in `crates/ctxpack-mcp/src/lib.rs`.
2. `run_server` reads newline-delimited JSON-RPC requests from stdio and passes each request through `handle_line` and `handle_request`.
3. `tools/call` requests with `name = "prepare_task"` call `call_prepare_task` in `crates/ctxpack-mcp/src/lib.rs`.
4. `call_prepare_task` validates arguments, discovers the repo, resolves anchors, calls `prepare_context_plan_with_paths`, records an eval trace, and calls `cache_pack_resources`.
5. `cache_pack_resources` compiles brief, standard, and deep packs from the plan and stores them in a process-local `OnceLock<Mutex<BTreeMap<...>>>` keyed by `ctxpack://pack/<task-id>/<budget>`.
6. The MCP response returns both text JSON and `structuredContent` for the `ContextPlan`.

**MCP Resource Flow:**

1. `resources/list` in `crates/ctxpack-mcp/src/lib.rs` advertises repository summary, test map, dependency graph, pack guide, pack resources, file slices, and symbol resources.
2. `resources/read` calls `read_resource` in `crates/ctxpack-mcp/src/lib.rs`.
3. Repository resources call `repo_summary`, `repo_test_map`, or `repo_dependency_graph`, which delegate to `load_or_build_inventory`, `test_map`, or `dependency_edges` in `crates/ctxpack-index/src/lib.rs`.
4. Pack resources call `read_pack_resource`, which serves only packs cached in the current MCP server process by `prepare_task`.
5. File and symbol resources call `read_file_resource` and `read_symbol_resource`, both gated by repo discovery and safe inventory/search behavior.

**Index Persistence Flow:**

1. `write_inventory` in `crates/ctxpack-index/src/lib.rs` calls `build_inventory`.
2. `build_inventory` uses `ignore::WalkBuilder` with `.gitignore`, `.ctxpackignore`, and `.cursorignore`, classifies each safe file, hashes bytes with `blake3`, and excludes generated and sensitive files unless explicitly opted in.
3. Inventory JSON is written by `write_inventory` to `inventory_path(repo_id)`, which resolves to `~/.ctxpack/repos/<repo-id>/inventory.json` through `ctxpack_home`.
4. Retrieval functions call `load_or_build_inventory`, which reuses a stored inventory when present and otherwise builds one with default safe options.

**State Management:**
- Runtime domain state is passed as serialized structs such as `ContextPlan`, `ContextPack`, `RepoInventory`, and `EvalTrace`.
- Persistent local state lives outside the repo under `~/.ctxpack/repos/<repo-id>/inventory.json` and `~/.ctxpack/repos/<repo-id>/traces.jsonl` via `crates/ctxpack-index/src/lib.rs`.
- Repo-local init state is written under `.ctxpack/` and `AGENTS.md` by `run_init` in `crates/ctxpack-core/src/init.rs`.
- MCP pack resources are in-memory and session-scoped in `crates/ctxpack-mcp/src/lib.rs`.

## Key Abstractions

**ContextPlan:**
- Purpose: Compact task preparation result with target files, related tests, commands, pack options, risk flags, and privacy status.
- Examples: `ContextPlan`, `TargetFile`, `RelatedTest`, `RiskFlag`, and `PackOption` in `crates/ctxpack-core/src/contracts.rs`.
- Pattern: Serializable contract with camelCase field names for tool/client interoperability.

**ContextPack:**
- Purpose: Budgeted materialization of a plan into ordered sections and optional snippets.
- Examples: `ContextPack` and `PackSection` in `crates/ctxpack-core/src/contracts.rs`; construction in `compile_pack_from_plan` in `crates/ctxpack-compiler/src/lib.rs`.
- Pattern: Structured core value rendered to Markdown or JSON at the transport boundary.

**Safe Inventory:**
- Purpose: Normalize repository files into safe, classified, hash-addressable entries for all retrieval operations.
- Examples: `RepoInventory`, `FileInventoryEntry`, `InventoryOptions`, and `InventoryReport` in `crates/ctxpack-index/src/lib.rs`.
- Pattern: Build or load a local JSON cache; exclude sensitive/generated files by default; keep source text out of search summaries where possible.

**Retrieval Signals:**
- Purpose: Provide ranked candidate files, symbols, tests, co-change hints, dependency edges, and current-diff anchors for plans and MCP tools.
- Examples: `SearchResult`, `SymbolSearchResult`, `RelatedTestResult`, `CoChangeHint`, `DependencyEdge`, and `CurrentDiffSummary` in `crates/ctxpack-index/src/lib.rs`.
- Pattern: Each signal has a typed options struct and typed result struct; consumers merge signals in `crates/ctxpack-compiler/src/lib.rs` and `crates/ctxpack-mcp/src/lib.rs`.

**Repository Root:**
- Purpose: Resolve user-supplied paths and MCP server working directories to the nearest git repository.
- Examples: `RepoRoot` and `RepoRoot::discover_from` in `crates/ctxpack-core/src/repo.rs`.
- Pattern: Walk ancestors until `.git` exists; return typed `RepoRootError` when no repository is found.

**Initialization Artifacts:**
- Purpose: Generate repo-local AGENTS guidance, `.ctxpack/ctxpack.toml`, and optional native adapter snippets.
- Examples: `run_init`, `InitOptions`, `AgentAdapter`, `adapter_files`, and `upsert_agents_section` in `crates/ctxpack-core/src/init.rs`.
- Pattern: Validate path safety, reject symlink components, upsert managed AGENTS section between marker comments, and report per-file actions.

**MCP Boundary:**
- Purpose: Translate MCP JSON-RPC methods into typed library calls and structured MCP responses.
- Examples: `JsonRpcRequest`, `CallToolParams`, `PrepareTaskArgs`, `GetPackArgs`, `RelatedArgs`, `ReadResourceParams`, and `GetPromptParams` in `crates/ctxpack-mcp/src/lib.rs`.
- Pattern: Deserialize request params, validate required fields, call lower layers, serialize both text content and structured content.

## Entry Points

**Workspace Manifest:**
- Location: `Cargo.toml`
- Triggers: Cargo workspace commands.
- Responsibilities: Declare workspace members and shared dependency versions for `crates/ctxpack`, `crates/ctxpack-core`, `crates/ctxpack-index`, `crates/ctxpack-compiler`, and `crates/ctxpack-mcp`.

**CLI Binary:**
- Location: `crates/ctxpack/src/main.rs`
- Triggers: `cargo run -p ctxpack -- <command>` or installed `ctxpack`.
- Responsibilities: Implement `init`, `index`, `prepare-task`, `get-pack`, `search`, `symbols`, `related-tests`, `co-changes`, `dependencies`, `cards generate`, `eval traces`, `eval checklist`, `eval history`, and `serve-mcp`.

**Core Library:**
- Location: `crates/ctxpack-core/src/lib.rs`
- Triggers: Rust crate imports from other workspace crates.
- Responsibilities: Re-export core contracts, init APIs, privacy status, and repo root helpers.

**Index Library:**
- Location: `crates/ctxpack-index/src/lib.rs`
- Triggers: Calls from `crates/ctxpack/src/main.rs`, `crates/ctxpack-compiler/src/lib.rs`, and `crates/ctxpack-mcp/src/lib.rs`.
- Responsibilities: Provide safe local repository inventory, retrieval, graph, diff, history, and trace APIs.

**Compiler Library:**
- Location: `crates/ctxpack-compiler/src/lib.rs`
- Triggers: Calls from the CLI and MCP crates.
- Responsibilities: Fuse retrieval signals into plans, compile packs, generate cards, render Markdown, and evaluate historical commits.

**MCP Server Library:**
- Location: `crates/ctxpack-mcp/src/lib.rs`
- Triggers: `ctxpack serve-mcp`.
- Responsibilities: Serve MCP tools/resources/prompts over stdio JSON-RPC and keep session-scoped pack resources.

## Error Handling

**Strategy:** Use typed errors inside library layers, convert to `anyhow::Result` at the CLI boundary and JSON-RPC error payloads at the MCP boundary.

**Patterns:**
- Use `thiserror::Error` enums for domain errors in `crates/ctxpack-core/src/repo.rs`, `crates/ctxpack-core/src/init.rs`, and `crates/ctxpack-index/src/lib.rs`.
- Use `anyhow::Result` in `crates/ctxpack/src/main.rs` so CLI commands can use `?` across lower-layer error types.
- Use `RpcError` in `crates/ctxpack-mcp/src/lib.rs` with JSON-RPC codes for parse errors, invalid params, and missing methods.
- Let optional history signals degrade gracefully: `prepare_context_plan_with_paths_and_history` in `crates/ctxpack-compiler/src/lib.rs` adds `co_change_unavailable` or `dependency_graph_unavailable` risk flags where appropriate.
- Let MCP `related` continue without local git history by adding a warning in `call_related` in `crates/ctxpack-mcp/src/lib.rs`.

## Cross-Cutting Concerns

**Logging:** There is no logging framework. CLI commands print user-facing summaries or JSON/Markdown from `crates/ctxpack/src/main.rs`; MCP responses are serialized JSON values from `crates/ctxpack-mcp/src/lib.rs`.

**Validation:** Argument validation is performed at the entry layer in `crates/ctxpack/src/main.rs` and `crates/ctxpack-mcp/src/lib.rs`; repository safety validation is centralized in `crates/ctxpack-index/src/lib.rs` and path/symlink validation for init is in `crates/ctxpack-core/src/init.rs`.

**Authentication:** Not applicable. The project is local-first and read-only by design; MCP and CLI calls operate on local filesystem and local git state.

**Privacy:** Preserve local-only behavior. Use `PrivacyStatus::local_only()` from `crates/ctxpack-core/src/privacy.rs`; keep generated/sensitive paths excluded by default in `crates/ctxpack-index/src/lib.rs`; keep eval traces source-free via `EvalTrace` in `crates/ctxpack-core/src/contracts.rs`.

---

*Architecture analysis: 2026-05-13*
