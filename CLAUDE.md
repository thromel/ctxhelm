<!-- GSD:project-start source:PROJECT.md -->
## Project

**Repo Context Packer**

Repo Context Packer is a local-first, read-only context broker that helps existing coding agents choose better repository context. It does not replace Codex, Claude Code, Cursor, OpenCode, Aider, or similar tools; it exposes task-conditioned file, test, graph, history, and pack guidance through agent-native surfaces such as MCP, AGENTS.md, and thin adapter files.

The current codebase is a Rust workspace with a CLI, MCP server, safe repository inventory, lexical and symbol retrieval, related-test inference, dependency hints, current-diff anchors, context packs, generated context cards, local eval traces, and historical retrieval evaluation.

**Core Value:** Given a coding task, ctxpack should return the smallest safe evidence set that makes an existing coding agent more likely to inspect the right files, run the right tests, and avoid irrelevant context.

### Constraints

- **Privacy**: Default behavior must stay local-only and source-safe for inventory, plans, traces, historical eval reports, and generated cards. Packs may contain safe snippets, but every snippet path must remain filtered through the safe inventory policy.
- **Product surface**: AGENTS.md, MCP, and thin native rules/adapters remain the primary surfaces. CLI exists for setup, debugging, and automation, not as the daily product center.
- **Read-only scope**: ctxpack should not edit source code, run user project tests, install dependencies, or auto-commit user work. It can write its own local caches, traces, generated cards, adapter files, and planning/docs artifacts.
- **Implementation stack**: Keep the current Rust workspace architecture and typed contracts unless there is a clear measured reason to change.
- **Evaluation**: New retrieval work should be checked against source-free historical evals, with RefactoringMiner as a large-history external smoke target when practical.
- **Validation**: Run `cargo test --workspace` before claiming implementation work complete, and `cargo run -p ctxpack -- --help` after CLI changes.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- Rust 2021 edition - all executable and library code lives in the Cargo workspace rooted at `Cargo.toml`.
- Markdown - product docs, roadmap, milestone plans, and generated agent instruction surfaces.
- JSON and TOML - generated configuration and protocol payload formats.
## Runtime
- Rust toolchain from the local development environment.
- No pinned toolchain file is present. `rust-toolchain`, `rust-toolchain.toml`, `.nvmrc`, and `.python-version` were not detected.
- Runtime entrypoints are local command-line processes, not web servers.
- Cargo 1.87.0
- Lockfile: present at `Cargo.lock`
- Workspace resolver: `resolver = "2"` in `Cargo.toml`
- Workspace members:
## Frameworks
- Cargo workspace - crate organization and shared dependency versioning in `Cargo.toml`.
- clap 4.6.1 - derive-based CLI parsing in `crates/ctxpack/src/main.rs`.
- serde 1.0.228 and serde_json 1.0.149 - typed JSON contracts, CLI output, MCP payloads, inventory files, traces, and generated reports across `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, and `crates/ctxpack-mcp/src/lib.rs`.
- Model Context Protocol over JSON-RPC stdio - implemented directly with `serde_json`, `std::io::BufRead`, and `std::io::Write` in `crates/ctxpack-mcp/src/lib.rs`; no external MCP SDK is used.
- Rust built-in test harness - tests are co-located in `#[cfg(test)] mod tests` blocks inside crate source files such as `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-mcp/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, and `crates/ctxpack-core/src/init.rs`.
- tempfile 3.27.0 - temporary repositories, temporary homes, and fixture workspaces in crate tests.
- Cargo commands documented in `README.md` and `AGENTS.md`:
- External process tools used by the implementation:
## Key Dependencies
- `anyhow` 1.0.102 - CLI-level error propagation in `crates/ctxpack/src/main.rs`.
- `clap` 4.6.1 - subcommands and typed arguments for `init`, `index`, `prepare-task`, `get-pack`, `search`, `symbols`, `related-tests`, `co-changes`, `dependencies`, `cards`, `eval`, and `serve-mcp` in `crates/ctxpack/src/main.rs`.
- `serde` 1.0.228 - derives stable structured contracts in `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, and `crates/ctxpack-mcp/src/lib.rs`.
- `serde_json` 1.0.149 - CLI JSON output, MCP JSON-RPC input/output, inventory serialization, trace serialization, and adapter snippet validation.
- `ignore` 0.4.25 - safe repository walking with `.gitignore`, `.ctxpackignore`, and `.cursorignore` support in `crates/ctxpack-index/src/lib.rs`.
- `blake3` 1.8.5 - file content hashes and task hashes in `crates/ctxpack-index/src/lib.rs`.
- `uuid` 1.23.1 - repo IDs, plan IDs, pack IDs, and trace IDs in `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-index/src/lib.rs`, and `crates/ctxpack-compiler/src/lib.rs`.
- `thiserror` 2.0.18 - typed error enums in `crates/ctxpack-core/src/init.rs`, `crates/ctxpack-core/src/repo.rs`, and `crates/ctxpack-index/src/lib.rs`.
- `tempfile` 3.27.0 - dev/test dependency for isolated filesystem and git fixtures in crate tests; it is also used by `crates/ctxpack-compiler/src/lib.rs` for temporary historical eval worktrees.
- `tokio` 1 is declared in workspace dependencies in `Cargo.toml`, but no crate currently consumes it in its `Cargo.toml`; it does not appear in the active `cargo tree --workspace --depth 2` output.
## Configuration
- Primary runtime configuration is CLI arguments and repo-local files.
- `CTXPACK_HOME` controls private local state location in `crates/ctxpack-index/src/lib.rs`.
- No `.env`, `.env.*`, `*.env`, secret, credential, PEM, key, or `.npmrc` files were detected at mapping depth.
- `Cargo.toml` defines the Rust workspace, edition, license, member crates, and workspace dependency constraints.
- Each crate has its own manifest:
- `Cargo.lock` pins resolved dependency versions.
- `.gitignore` excludes `.worktrees/`, `/target/`, `/.ctxpack/cache/`, `/.ctxpack/index/`, `*.log`, and `.DS_Store`.
## Platform Requirements
- Rust/Cargo toolchain compatible with Rust 2021 edition.
- `git` CLI available on `PATH`.
- `tar` CLI available on `PATH`.
- Local filesystem access for scanning repositories, writing `~/.ctxpack/repos/<repo-id>/inventory.json`, appending `~/.ctxpack/repos/<repo-id>/traces.jsonl`, and optionally writing `.ctxpack/cards/*.md`.
- Local-first CLI binary and stdio MCP server.
- No hosted service, container runtime, HTTP server, cloud storage, cloud embeddings, cloud reranking, or remote database deployment target is detected in the repository.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- Use Rust crate and module names in kebab-case for packages and snake_case for source files.
- Workspace crates live under `crates/ctxpack/Cargo.toml`, `crates/ctxpack-core/Cargo.toml`, `crates/ctxpack-index/Cargo.toml`, `crates/ctxpack-compiler/Cargo.toml`, and `crates/ctxpack-mcp/Cargo.toml`.
- Library crates use `src/lib.rs`; the CLI crate uses `crates/ctxpack/src/main.rs`.
- Domain modules in `ctxpack-core` are split into small files: `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-core/src/init.rs`, `crates/ctxpack-core/src/privacy.rs`, and `crates/ctxpack-core/src/repo.rs`.
- Larger implementation crates keep implementation in a single `src/lib.rs`: `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, and `crates/ctxpack-mcp/src/lib.rs`.
- Use snake_case for functions and methods: `prepare_context_plan`, `compile_context_pack_with_plan_and_paths_for_agent`, `render_pack_markdown`, `current_diff_summary`, and `run_stdio_server`.
- Public API functions use action-oriented names and explicit suffixes when behavior differs: `compile_context_pack`, `compile_context_pack_with_plan`, `compile_context_pack_with_plan_for_agent`, and `compile_context_pack_from_plan_for_agent` in `crates/ctxpack-compiler/src/lib.rs`.
- Helper functions stay private unless they are part of the crate contract: `context_anchor_paths` in `crates/ctxpack/src/main.rs`, `bounded_limit` in `crates/ctxpack-mcp/src/lib.rs`, and `normalize_score` in `crates/ctxpack-compiler/src/lib.rs`.
- Test names are behavior sentences in snake_case: `context_plan_serializes_with_camel_case_contract_fields`, `inventory_respects_ignores_and_default_exclusions`, and `prepare_task_call_returns_structured_context_plan`.
- Use snake_case for locals and fields: `task_hash`, `target_agent`, `repo_root`, `safe_changed_files`, and `source_text_logged`.
- Use `repo` for a discovered `RepoRoot` and `repo_root` for a `Path` or `PathBuf` root path, matching `crates/ctxpack/src/main.rs` and `crates/ctxpack-index/src/lib.rs`.
- Use `args` for parsed CLI/MCP arguments in command handlers: `Command::PrepareTask(args)` in `crates/ctxpack/src/main.rs` and `call_prepare_task(arguments)` in `crates/ctxpack-mcp/src/lib.rs`.
- Use `output` for incrementally rendered Markdown/text buffers, as in `render_eval_checklist`, `render_historical_eval_report`, and `render_cards_report` in `crates/ctxpack/src/main.rs`.
- Use PascalCase for structs and enums: `ContextPlan`, `ContextPack`, `InventoryError`, `SearchOptions`, `RpcError`, `HistoricalEvalReport`, and `AgentAdapter`.
- Use `Options` suffix for input configuration structs: `InventoryOptions`, `SearchOptions`, `SymbolOptions`, `DependencyOptions`, `HistoricalEvalOptions`, and `ContextCardsOptions`.
- Use `Report`, `Result`, `Summary`, `Entry`, and `Hint` suffixes for output contracts: `InventoryReport`, `SearchResult`, `CurrentDiffSummary`, `FileInventoryEntry`, and `CoChangeHint`.
- Use `Error` suffix for typed errors: `RepoRootError`, `InitError`, `InventoryError`, and `RpcError`.
- Use SCREAMING_SNAKE_CASE for constants: `PREPARE_TASK_TARGET_LIMIT`, `JSONRPC_VERSION`, `MCP_PROTOCOL_VERSION`, `AGENTS_SECTION_START`, and `AGENTS_SECTION_END`.
## Code Style
- Use standard `rustfmt` formatting.
- No repo-specific `rustfmt.toml` or `.rustfmt.toml` is detected.
- Preserve rustfmt import formatting and line wrapping. Examples include grouped imports in `crates/ctxpack/src/main.rs` and `crates/ctxpack-mcp/src/lib.rs`.
- Keep generated Markdown/text rendering readable with explicit `push_str`, `format!`, and small helper functions, as in `render_historical_eval_report` and `push_plain_path_list` in `crates/ctxpack/src/main.rs`.
- No checked-in `clippy.toml`, `deny.toml`, or CI lint config is detected.
- Prefer code that passes default compiler warnings and common Clippy expectations.
- Avoid introducing unused public APIs; public exports in `crates/ctxpack-core/src/lib.rs` are intentionally limited to core contracts and top-level entry points.
- Use typed contracts instead of unstructured strings for behavior surfaces. Public structs in `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-index/src/lib.rs`, and `crates/ctxpack-compiler/src/lib.rs` are serializable data contracts.
## Import Organization
- No Rust path aliases are configured.
- Use workspace crate names directly: `ctxpack_core`, `ctxpack_index`, `ctxpack_compiler`, and `ctxpack_mcp`.
- Use package names with hyphens in `Cargo.toml` and underscore crate names in Rust imports: `ctxpack-core` in `crates/ctxpack-core/Cargo.toml` imports as `ctxpack_core` in `crates/ctxpack/src/main.rs`.
## Error Handling
- Use `thiserror::Error` for library/domain error enums in `crates/ctxpack-core/src/repo.rs`, `crates/ctxpack-core/src/init.rs`, and `crates/ctxpack-index/src/lib.rs`.
- Include the failing path and source error in IO variants:
#[error("failed to read {path}: {source}")]
- Map IO, serialization, and git failures at the boundary where context is available. `write_inventory` in `crates/ctxpack-index/src/lib.rs` maps create, serialize, and write failures to `InventoryError`.
- Use `anyhow::Result` only in the CLI binary at `crates/ctxpack/src/main.rs`; keep library crates on typed errors.
- Convert library errors into JSON-RPC errors at the MCP boundary in `crates/ctxpack-mcp/src/lib.rs` with `RpcError::invalid_params(format!("failed to ...: {error}"))`.
- Return JSON-RPC method errors through `RpcError::method_not_found` and parse errors through `RpcError::parse_error` in `crates/ctxpack-mcp/src/lib.rs`.
- Validate empty required strings before running expensive work. `call_prepare_task`, `call_get_pack`, and `call_search` in `crates/ctxpack-mcp/src/lib.rs` reject empty task/query values.
- In tests, `unwrap()` and `unwrap_err()` are common and acceptable for fixture setup and direct failure assertions.
## Logging
- CLI commands print user-facing reports or JSON to stdout with `println!` in `crates/ctxpack/src/main.rs`.
- MCP server writes one JSON-RPC response per line to its writer in `run_server` in `crates/ctxpack-mcp/src/lib.rs`.
- Git subprocess stderr is captured and converted into `InventoryError::Git` in `crates/ctxpack-index/src/lib.rs`.
- Do not add ad hoc debug logging to library functions. Return structured reports or typed errors instead.
## Comments
- Prefer self-describing function and test names over comments.
- Use comments sparingly for protocol-facing or generated text only when the literal content requires context.
- Avoid comments that restate simple Rust operations.
- Not applicable. The codebase is Rust.
- Rustdoc comments are not used on the existing public API. Match the existing style unless adding a public API that needs external documentation.
## Function Design
- Keep core-domain modules small where possible, as in `crates/ctxpack-core/src/contracts.rs`, `crates/ctxpack-core/src/privacy.rs`, and `crates/ctxpack-core/src/repo.rs`.
- Large orchestration modules exist in `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, and `crates/ctxpack-mcp/src/lib.rs`; add new helpers near related behavior rather than creating unrelated utility sections.
- Keep public functions focused on one operation and push rendering, filtering, and path normalization into private helpers.
- Accept `impl AsRef<Path>` for public functions that take repo paths: `prepare_context_plan`, `compile_context_pack`, `write_inventory`, `lexical_search`, and `current_diff_summary`.
- Accept options structs for configurable operations: `InventoryOptions`, `SearchOptions`, `SymbolOptions`, `CoChangeOptions`, `DependencyOptions`, and `CurrentDiffOptions`.
- Use explicit typed enums for mode/budget rather than strings in Rust APIs: `TaskType` and `PackBudget` in `crates/ctxpack-core/src/contracts.rs`.
- Use `Option<T>` for optional JSON/CLI inputs and apply defaults at the boundary, as in `PrepareTaskArgs`, `GetPackArgs`, and `SearchArgs` in `crates/ctxpack-mcp/src/lib.rs`.
- Return `Result<T, InventoryError>` for index/compiler operations that touch the filesystem, git, or serialized state.
- Return `Result<T, InitError>` for repo initialization operations in `crates/ctxpack-core/src/init.rs`.
- Return `Result<Value, RpcError>` for MCP request handlers in `crates/ctxpack-mcp/src/lib.rs`.
- Return serializable structs for behavior contracts, not formatted strings. Formatting belongs in boundary helpers such as `render_pack_markdown`, `render_eval_checklist`, and `tool_json_result`.
## Module Design
- `crates/ctxpack-core/src/lib.rs` is the only barrel-style module. It exposes domain contracts via `pub use contracts::*`, selected init types, `PrivacyStatus`, `FileRole`, and `RepoRoot`.
- `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, and `crates/ctxpack-mcp/src/lib.rs` expose public functions and structs directly from their crate root.
- Keep internal helper structs private unless they are consumed across crate boundaries. Examples include `RpcError` in `crates/ctxpack-mcp/src/lib.rs` and `HistoricalEvalWorktree` in `crates/ctxpack-compiler/src/lib.rs`.
- Use a barrel file only for `ctxpack-core`, where multiple small modules form the shared contract layer.
- Do not add new barrel files for `ctxpack-index`, `ctxpack-compiler`, or `ctxpack-mcp` unless those crates are split into modules.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- Keep public contracts centralized in `crates/ctxpack-core/src/contracts.rs` and re-exported from `crates/ctxpack-core/src/lib.rs`.
- Keep repository scanning and retrieval in `crates/ctxpack-index/src/lib.rs`; callers receive structured values, not formatted CLI strings.
- Keep context-plan and context-pack construction in `crates/ctxpack-compiler/src/lib.rs`; CLI and MCP entry points call the same compiler APIs.
- Keep protocol handling in `crates/ctxpack-mcp/src/lib.rs`; it translates JSON-RPC/MCP requests into core/compiler/index calls.
- Keep command parsing and terminal rendering in `crates/ctxpack/src/main.rs`; it is the outer orchestration layer.
## Layers
- Purpose: Define stable data contracts, repository root discovery, privacy metadata, and init artifacts shared across crates.
- Location: `crates/ctxpack-core/src/`
- Contains: Serializable structs and enums in `crates/ctxpack-core/src/contracts.rs`, repo discovery in `crates/ctxpack-core/src/repo.rs`, privacy status in `crates/ctxpack-core/src/privacy.rs`, and init file generation in `crates/ctxpack-core/src/init.rs`.
- Depends on: `serde`, `thiserror`, and `uuid` from `crates/ctxpack-core/Cargo.toml`.
- Used by: `crates/ctxpack-index/src/lib.rs`, `crates/ctxpack-compiler/src/lib.rs`, `crates/ctxpack-mcp/src/lib.rs`, and `crates/ctxpack/src/main.rs`.
- Purpose: Build a safe file inventory, classify files, search files and symbols, infer related tests, derive dependency edges, read current diff paths, sample git history, and persist eval traces.
- Location: `crates/ctxpack-index/src/lib.rs`
- Contains: Inventory APIs such as `build_inventory`, `write_inventory`, and `load_or_build_inventory`; retrieval APIs such as `lexical_search`, `symbol_search`, `related_tests`, `co_change_hints`, `dependency_edges`, `related_dependency_edges`, `current_diff_summary`, `historical_commit_samples`, `append_eval_trace`, and `list_eval_traces`.
- Depends on: `ctxpack-core`, `ignore`, `blake3`, `serde`, `serde_json`, `thiserror`, `uuid`, local filesystem access, and local `git` commands.
- Used by: `crates/ctxpack-compiler/src/lib.rs`, `crates/ctxpack-mcp/src/lib.rs`, and `crates/ctxpack/src/main.rs`.
- Purpose: Convert a task and optional anchors into a `ContextPlan`, compile budgeted `ContextPack` values, render packs and context cards, and run source-free historical retrieval evaluation.
- Location: `crates/ctxpack-compiler/src/lib.rs`
- Contains: Plan APIs such as `prepare_context_plan_with_paths`, pack APIs such as `compile_context_pack_with_plan_and_paths_for_agent`, rendering APIs such as `render_pack_markdown`, eval APIs such as `eval_trace_for_plan`, `eval_trace_for_pack`, and `evaluate_historical_commits`, and card generation in `generate_context_cards`.
- Depends on: `ctxpack-core`, `ctxpack-index`, `serde`, `tempfile`, `uuid`, local filesystem access, and local `git` commands for historical worktree evaluation.
- Used by: `crates/ctxpack-mcp/src/lib.rs` and `crates/ctxpack/src/main.rs`.
- Purpose: Expose ctxpack through a local stdio JSON-RPC server with MCP tools, resources, and prompts.
- Location: `crates/ctxpack-mcp/src/lib.rs`
- Contains: Server loop in `run_stdio_server` and `run_server`, request dispatch in `handle_request`, MCP tool handlers such as `call_prepare_task`, `call_search`, `call_related`, `call_get_pack`, `call_related_tests`, and `call_current_diff`, resource handlers such as `read_resource`, and prompt handlers such as `get_prompt`.
- Depends on: `ctxpack-core`, `ctxpack-index`, `ctxpack-compiler`, `serde`, and `serde_json`.
- Used by: The CLI command `serve-mcp` in `crates/ctxpack/src/main.rs`.
- Purpose: Parse terminal commands, discover repository roots, call library APIs, append eval traces where applicable, and render JSON or Markdown/text output.
- Location: `crates/ctxpack/src/main.rs`
- Contains: Clap command definitions, argument structs, mode/budget/format enums, `main`, helper conversion functions, output renderers, and CLI-specific anchor handling.
- Depends on: `ctxpack-core`, `ctxpack-index`, `ctxpack-compiler`, `ctxpack-mcp`, `anyhow`, `clap`, and `serde_json`.
- Used by: Developers running `cargo run -p ctxpack -- ...`; MCP clients launch the same binary through `ctxpack serve-mcp`.
- Purpose: Preserve product specs and milestone implementation plans.
- Location: `docs/superpowers/`
- Contains: Product spec in `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md` and milestone plans in `docs/superpowers/plans/`.
- Depends on: Not applicable.
- Used by: Human and agent planning workflows; runtime code does not import these documents.
## Data Flow
- Runtime domain state is passed as serialized structs such as `ContextPlan`, `ContextPack`, `RepoInventory`, and `EvalTrace`.
- Persistent local state lives outside the repo under `~/.ctxpack/repos/<repo-id>/inventory.json` and `~/.ctxpack/repos/<repo-id>/traces.jsonl` via `crates/ctxpack-index/src/lib.rs`.
- Repo-local init state is written under `.ctxpack/` and `AGENTS.md` by `run_init` in `crates/ctxpack-core/src/init.rs`.
- MCP pack resources are in-memory and session-scoped in `crates/ctxpack-mcp/src/lib.rs`.
## Key Abstractions
- Purpose: Compact task preparation result with target files, related tests, commands, pack options, risk flags, and privacy status.
- Examples: `ContextPlan`, `TargetFile`, `RelatedTest`, `RiskFlag`, and `PackOption` in `crates/ctxpack-core/src/contracts.rs`.
- Pattern: Serializable contract with camelCase field names for tool/client interoperability.
- Purpose: Budgeted materialization of a plan into ordered sections and optional snippets.
- Examples: `ContextPack` and `PackSection` in `crates/ctxpack-core/src/contracts.rs`; construction in `compile_pack_from_plan` in `crates/ctxpack-compiler/src/lib.rs`.
- Pattern: Structured core value rendered to Markdown or JSON at the transport boundary.
- Purpose: Normalize repository files into safe, classified, hash-addressable entries for all retrieval operations.
- Examples: `RepoInventory`, `FileInventoryEntry`, `InventoryOptions`, and `InventoryReport` in `crates/ctxpack-index/src/lib.rs`.
- Pattern: Build or load a local JSON cache; exclude sensitive/generated files by default; keep source text out of search summaries where possible.
- Purpose: Provide ranked candidate files, symbols, tests, co-change hints, dependency edges, and current-diff anchors for plans and MCP tools.
- Examples: `SearchResult`, `SymbolSearchResult`, `RelatedTestResult`, `CoChangeHint`, `DependencyEdge`, and `CurrentDiffSummary` in `crates/ctxpack-index/src/lib.rs`.
- Pattern: Each signal has a typed options struct and typed result struct; consumers merge signals in `crates/ctxpack-compiler/src/lib.rs` and `crates/ctxpack-mcp/src/lib.rs`.
- Purpose: Resolve user-supplied paths and MCP server working directories to the nearest git repository.
- Examples: `RepoRoot` and `RepoRoot::discover_from` in `crates/ctxpack-core/src/repo.rs`.
- Pattern: Walk ancestors until `.git` exists; return typed `RepoRootError` when no repository is found.
- Purpose: Generate repo-local AGENTS guidance, `.ctxpack/ctxpack.toml`, and optional native adapter snippets.
- Examples: `run_init`, `InitOptions`, `AgentAdapter`, `adapter_files`, and `upsert_agents_section` in `crates/ctxpack-core/src/init.rs`.
- Pattern: Validate path safety, reject symlink components, upsert managed AGENTS section between marker comments, and report per-file actions.
- Purpose: Translate MCP JSON-RPC methods into typed library calls and structured MCP responses.
- Examples: `JsonRpcRequest`, `CallToolParams`, `PrepareTaskArgs`, `GetPackArgs`, `RelatedArgs`, `ReadResourceParams`, and `GetPromptParams` in `crates/ctxpack-mcp/src/lib.rs`.
- Pattern: Deserialize request params, validate required fields, call lower layers, serialize both text content and structured content.
## Entry Points
- Location: `Cargo.toml`
- Triggers: Cargo workspace commands.
- Responsibilities: Declare workspace members and shared dependency versions for `crates/ctxpack`, `crates/ctxpack-core`, `crates/ctxpack-index`, `crates/ctxpack-compiler`, and `crates/ctxpack-mcp`.
- Location: `crates/ctxpack/src/main.rs`
- Triggers: `cargo run -p ctxpack -- <command>` or installed `ctxpack`.
- Responsibilities: Implement `init`, `index`, `prepare-task`, `get-pack`, `search`, `symbols`, `related-tests`, `co-changes`, `dependencies`, `cards generate`, `eval traces`, `eval checklist`, `eval history`, and `serve-mcp`.
- Location: `crates/ctxpack-core/src/lib.rs`
- Triggers: Rust crate imports from other workspace crates.
- Responsibilities: Re-export core contracts, init APIs, privacy status, and repo root helpers.
- Location: `crates/ctxpack-index/src/lib.rs`
- Triggers: Calls from `crates/ctxpack/src/main.rs`, `crates/ctxpack-compiler/src/lib.rs`, and `crates/ctxpack-mcp/src/lib.rs`.
- Responsibilities: Provide safe local repository inventory, retrieval, graph, diff, history, and trace APIs.
- Location: `crates/ctxpack-compiler/src/lib.rs`
- Triggers: Calls from the CLI and MCP crates.
- Responsibilities: Fuse retrieval signals into plans, compile packs, generate cards, render Markdown, and evaluate historical commits.
- Location: `crates/ctxpack-mcp/src/lib.rs`
- Triggers: `ctxpack serve-mcp`.
- Responsibilities: Serve MCP tools/resources/prompts over stdio JSON-RPC and keep session-scoped pack resources.
## Error Handling
- Use `thiserror::Error` enums for domain errors in `crates/ctxpack-core/src/repo.rs`, `crates/ctxpack-core/src/init.rs`, and `crates/ctxpack-index/src/lib.rs`.
- Use `anyhow::Result` in `crates/ctxpack/src/main.rs` so CLI commands can use `?` across lower-layer error types.
- Use `RpcError` in `crates/ctxpack-mcp/src/lib.rs` with JSON-RPC codes for parse errors, invalid params, and missing methods.
- Let optional history signals degrade gracefully: `prepare_context_plan_with_paths_and_history` in `crates/ctxpack-compiler/src/lib.rs` adds `co_change_unavailable` or `dependency_graph_unavailable` risk flags where appropriate.
- Let MCP `related` continue without local git history by adding a warning in `call_related` in `crates/ctxpack-mcp/src/lib.rs`.
## Cross-Cutting Concerns
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
