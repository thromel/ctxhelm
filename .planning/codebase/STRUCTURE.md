# Codebase Structure

**Analysis Date:** 2026-05-13

## Directory Layout

```text
Agent Memory/
|-- AGENTS.md                         # Project working rules and validation commands
|-- Cargo.toml                        # Rust workspace manifest
|-- Cargo.lock                        # Workspace dependency lockfile
|-- README.md                         # User-facing product and command documentation
|-- crates/
|   |-- ctxhelm/                      # CLI binary crate
|   |   |-- Cargo.toml
|   |   `-- src/main.rs
|   |-- ctxhelm-core/                 # Shared contracts, repo discovery, privacy, init artifacts
|   |   |-- Cargo.toml
|   |   `-- src/
|   |       |-- contracts.rs
|   |       |-- init.rs
|   |       |-- lib.rs
|   |       |-- privacy.rs
|   |       `-- repo.rs
|   |-- ctxhelm-index/                # Safe inventory, retrieval, symbols, tests, git signals
|   |   |-- Cargo.toml
|   |   `-- src/lib.rs
|   |-- ctxhelm-compiler/             # Context plan, pack, cards, and historical eval compiler
|   |   |-- Cargo.toml
|   |   `-- src/lib.rs
|   `-- ctxhelm-mcp/                  # Stdio MCP JSON-RPC server
|       |-- Cargo.toml
|       `-- src/lib.rs
|-- docs/
|   `-- superpowers/
|       |-- specs/                    # Product specs
|       `-- plans/                    # Milestone implementation plans
|-- .planning/
|   `-- codebase/                     # Generated codebase map documents
|-- .worktrees/                       # Local worktree area
`-- target/                           # Cargo build output, generated
```

## Directory Purposes

**Project Root:**
- Purpose: Workspace-level configuration, product docs, and project guidance.
- Contains: `Cargo.toml`, `Cargo.lock`, `README.md`, `AGENTS.md`, `crates/`, `docs/`, `.planning/`, `.worktrees/`, and `target/`.
- Key files: `Cargo.toml`, `README.md`, `AGENTS.md`.

**`crates/`:**
- Purpose: All Rust packages that make up the ctxhelm product.
- Contains: One directory per crate: `crates/ctxhelm/`, `crates/ctxhelm-core/`, `crates/ctxhelm-index/`, `crates/ctxhelm-compiler/`, and `crates/ctxhelm-mcp/`.
- Key files: `crates/ctxhelm/src/main.rs`, `crates/ctxhelm-core/src/lib.rs`, `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`.

**`crates/ctxhelm/`:**
- Purpose: CLI binary crate and top-level runtime orchestrator.
- Contains: `crates/ctxhelm/Cargo.toml` and `crates/ctxhelm/src/main.rs`.
- Key files: `crates/ctxhelm/src/main.rs` defines Clap commands, maps CLI enum values into core contracts, discovers repositories, calls lower-layer APIs, and renders terminal output.

**`crates/ctxhelm-core/`:**
- Purpose: Shared domain contracts and repo-local initialization behavior.
- Contains: `crates/ctxhelm-core/src/contracts.rs`, `crates/ctxhelm-core/src/init.rs`, `crates/ctxhelm-core/src/privacy.rs`, `crates/ctxhelm-core/src/repo.rs`, and `crates/ctxhelm-core/src/lib.rs`.
- Key files: `crates/ctxhelm-core/src/contracts.rs` for serializable types, `crates/ctxhelm-core/src/init.rs` for generated AGENTS/adapter artifacts, `crates/ctxhelm-core/src/repo.rs` for git root discovery.

**`crates/ctxhelm-index/`:**
- Purpose: Repository indexing and local retrieval signals.
- Contains: `crates/ctxhelm-index/Cargo.toml` and `crates/ctxhelm-index/src/lib.rs`.
- Key files: `crates/ctxhelm-index/src/lib.rs` contains inventory, classification, search, symbol extraction, related tests, co-changes, dependency graph, current diff, historical commit samples, and eval trace persistence.

**`crates/ctxhelm-compiler/`:**
- Purpose: Task-conditioned context planning, pack materialization, context card generation, and historical eval.
- Contains: `crates/ctxhelm-compiler/Cargo.toml` and `crates/ctxhelm-compiler/src/lib.rs`.
- Key files: `crates/ctxhelm-compiler/src/lib.rs` contains `prepare_context_plan_with_paths`, `compile_context_pack_with_plan_and_paths_for_agent`, `generate_context_cards`, and `evaluate_historical_commits`.

**`crates/ctxhelm-mcp/`:**
- Purpose: MCP transport layer for tools, resources, and prompts.
- Contains: `crates/ctxhelm-mcp/Cargo.toml` and `crates/ctxhelm-mcp/src/lib.rs`.
- Key files: `crates/ctxhelm-mcp/src/lib.rs` contains JSON-RPC request handling, MCP method dispatch, tool handlers, resource handlers, prompt handlers, and session pack-resource caching.

**`docs/superpowers/`:**
- Purpose: Planning artifacts for product and milestone work.
- Contains: Product specs in `docs/superpowers/specs/` and milestone plans in `docs/superpowers/plans/`.
- Key files: `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md` and `docs/superpowers/plans/2026-05-09-repo-context-packer-implementation-roadmap.md`.

**`.planning/codebase/`:**
- Purpose: Generated codebase map consumed by GSD planning and execution workflows.
- Contains: `ARCHITECTURE.md` and `STRUCTURE.md` for this architecture mapping pass.
- Key files: `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md`.

## Key File Locations

**Entry Points:**
- `crates/ctxhelm/src/main.rs`: CLI binary entry point and `serve-mcp` launcher.
- `crates/ctxhelm-mcp/src/lib.rs`: MCP stdio server entry point through `run_stdio_server`.
- `crates/ctxhelm-core/src/lib.rs`: Public re-export surface for core contracts and helpers.
- `Cargo.toml`: Workspace entry point for Cargo builds, tests, and crate membership.

**Configuration:**
- `Cargo.toml`: Workspace members, resolver, workspace package metadata, and shared dependencies.
- `crates/ctxhelm/Cargo.toml`: CLI crate dependencies.
- `crates/ctxhelm-core/Cargo.toml`: Core crate dependencies.
- `crates/ctxhelm-index/Cargo.toml`: Index/retrieval crate dependencies.
- `crates/ctxhelm-compiler/Cargo.toml`: Compiler crate dependencies.
- `crates/ctxhelm-mcp/Cargo.toml`: MCP crate dependencies.
- `AGENTS.md`: Project rules, MVP boundaries, and validation commands.

**Core Logic:**
- `crates/ctxhelm-core/src/contracts.rs`: Task, plan, pack, file, command, privacy, and eval trace contracts.
- `crates/ctxhelm-core/src/init.rs`: Repo-local init artifacts and safe file writes.
- `crates/ctxhelm-core/src/repo.rs`: Repository root discovery.
- `crates/ctxhelm-core/src/privacy.rs`: Local-only privacy status.
- `crates/ctxhelm-index/src/lib.rs`: Safe inventory, retrieval, dependency, current diff, git history, and trace persistence.
- `crates/ctxhelm-compiler/src/lib.rs`: Context plan construction, context pack compilation, card generation, eval traces, and historical eval.
- `crates/ctxhelm-mcp/src/lib.rs`: MCP protocol boundary and session-scoped pack resources.
- `crates/ctxhelm/src/main.rs`: CLI command routing and rendering.

**Testing:**
- `crates/ctxhelm-core/src/contracts.rs`: Inline unit tests for contract serialization.
- `crates/ctxhelm-core/src/repo.rs`: Inline unit tests for git ancestor discovery.
- `crates/ctxhelm-core/src/init.rs`: Inline unit tests for init output, AGENTS upsert behavior, adapter artifacts, and path safety.
- `crates/ctxhelm-index/src/lib.rs`: Inline unit tests for inventory, search, symbols, related tests, co-changes, dependency edges, current diff, history parsing, and trace behavior.
- `crates/ctxhelm-compiler/src/lib.rs`: Inline unit tests for plan construction, pack compilation, context cards, eval traces, and historical eval behavior.
- `crates/ctxhelm-mcp/src/lib.rs`: Inline unit tests for JSON-RPC/MCP methods, tools, resources, prompts, current diff, pack caching, and related expansions.
- `crates/ctxhelm/src/main.rs`: Inline unit tests for CLI rendering and helper behavior.

**Documentation:**
- `README.md`: Product behavior, commands, MCP runtime, safe inventory, retrieval, context packs, context cards, and eval workflows.
- `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md`: Product specification.
- `docs/superpowers/plans/`: Milestone implementation plans named by date and milestone.

## Naming Conventions

**Files:**
- Rust crates use hyphenated package names under `crates/`: `ctxhelm-core`, `ctxhelm-index`, `ctxhelm-compiler`, `ctxhelm-mcp`.
- Rust library roots use `src/lib.rs`: `crates/ctxhelm-core/src/lib.rs`, `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`.
- The CLI binary uses `src/main.rs`: `crates/ctxhelm/src/main.rs`.
- Core modules use short noun filenames: `contracts.rs`, `init.rs`, `privacy.rs`, and `repo.rs` under `crates/ctxhelm-core/src/`.
- Planning docs use date-prefixed Markdown filenames: `docs/superpowers/plans/2026-05-12-repo-context-packer-milestone-24-pack-provenance.md`.
- Codebase map docs use uppercase names: `.planning/codebase/ARCHITECTURE.md` and `.planning/codebase/STRUCTURE.md`.

**Directories:**
- New Rust crates belong under `crates/<crate-name>/`.
- New crate modules belong under the owning crate's `src/` directory.
- Product specs belong under `docs/superpowers/specs/`.
- Milestone implementation plans belong under `docs/superpowers/plans/`.
- Generated GSD codebase mapping documents belong under `.planning/codebase/`.

## Where to Add New Code

**New CLI Command:**
- Primary code: `crates/ctxhelm/src/main.rs`
- Add command enum variants, `Args` structs, mode/format conversions, command dispatch, and CLI output rendering in `crates/ctxhelm/src/main.rs`.
- Shared behavior should move to `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, or `crates/ctxhelm-core/src/` instead of staying in CLI-only helpers.
- Tests: Add inline tests in `crates/ctxhelm/src/main.rs` for CLI helper/rendering behavior; add lower-layer tests in the crate that owns the business behavior.

**New MCP Tool:**
- Primary code: `crates/ctxhelm-mcp/src/lib.rs`
- Add the tool name to `IMPLEMENTED_MCP_TOOL_NAMES`, add schema in `tools_list_result`, dispatch from `call_tool`, define a typed args struct near the existing MCP args structs, and implement `call_<tool_name>`.
- Business behavior should live in `crates/ctxhelm-index/src/lib.rs` or `crates/ctxhelm-compiler/src/lib.rs`; the MCP handler should validate/deserialize and serialize responses.
- Tests: Add inline MCP request/response tests in `crates/ctxhelm-mcp/src/lib.rs`.

**New MCP Resource Or Prompt:**
- Primary code: `crates/ctxhelm-mcp/src/lib.rs`
- Resources: add descriptor in `resources_list_result`, dispatch from `read_resource`, and delegate to safe inventory/compiler APIs.
- Prompts: add descriptor in `prompts_list_result`, dispatch from `get_prompt`, and use `workflow_prompt` for consistent prompt shape.
- Tests: Add inline tests in `crates/ctxhelm-mcp/src/lib.rs`.

**New Core Contract:**
- Primary code: `crates/ctxhelm-core/src/contracts.rs`
- Export from `crates/ctxhelm-core/src/lib.rs` if it is part of the public core surface.
- Use `serde` derives and existing rename conventions: `camelCase` for struct fields, `snake_case` or `lowercase` for enums according to neighboring contracts.
- Tests: Add serialization tests in `crates/ctxhelm-core/src/contracts.rs`.

**New Repository Retrieval Signal:**
- Primary code: `crates/ctxhelm-index/src/lib.rs`
- Add typed options and result structs near related retrieval types, implement safe inventory filtering, and keep source text out of public summaries unless the API is explicitly a safe file-slice resource.
- Connect plan usage in `crates/ctxhelm-compiler/src/lib.rs` when it affects context selection.
- Connect MCP/CLI exposure in `crates/ctxhelm-mcp/src/lib.rs` and `crates/ctxhelm/src/main.rs` only after the library API exists.
- Tests: Add inline tests in `crates/ctxhelm-index/src/lib.rs` and compiler/MCP tests where the signal changes plan or tool output.

**New Context Planning Or Pack Behavior:**
- Primary code: `crates/ctxhelm-compiler/src/lib.rs`
- Update `prepare_context_plan_with_paths_and_history` for target selection, test selection, risk flags, or confidence behavior.
- Update `compile_pack_from_plan`, `pack_limits`, or section render helpers for pack content changes.
- Tests: Add inline tests in `crates/ctxhelm-compiler/src/lib.rs`.

**New Init Adapter:**
- Primary code: `crates/ctxhelm-core/src/init.rs`
- Add the adapter to `AgentAdapter`, `adapter_path`, `adapter_content`, `adapter_files`, and `config_toml`.
- Connect CLI flags in `crates/ctxhelm/src/main.rs`.
- Tests: Add init artifact and CLI option tests in `crates/ctxhelm-core/src/init.rs` and `crates/ctxhelm/src/main.rs`.

**Utilities:**
- Shared repo, privacy, and contract helpers: `crates/ctxhelm-core/src/`.
- Shared retrieval and filesystem/git helpers: `crates/ctxhelm-index/src/lib.rs`.
- Shared plan, pack, rendering, card, and eval helpers: `crates/ctxhelm-compiler/src/lib.rs`.
- Transport-specific helpers: `crates/ctxhelm-mcp/src/lib.rs`.
- CLI-only helpers: `crates/ctxhelm/src/main.rs`.

## Special Directories

**`target/`:**
- Purpose: Cargo build output.
- Generated: Yes.
- Committed: No.

**`.worktrees/`:**
- Purpose: Local working-tree area.
- Generated: Local workflow dependent.
- Committed: No.

**`.planning/codebase/`:**
- Purpose: GSD-generated codebase mapping documents.
- Generated: Yes.
- Committed: Project workflow dependent.

**`docs/superpowers/`:**
- Purpose: Durable planning/spec artifacts.
- Generated: No for runtime; maintained as project documentation.
- Committed: Yes.

**`.ctxhelm/` in target repositories:**
- Purpose: Repo-local ctxhelm config, optional adapter snippets, and generated context cards from `run_init` and `cards generate`.
- Generated: Yes.
- Committed: Depends on target repository policy; generated cards are described as optional repo-committable artifacts in `README.md`.

**`~/.ctxhelm/repos/<repo-id>/`:**
- Purpose: User-local inventory and trace persistence for a repository.
- Generated: Yes.
- Committed: No.

---

*Structure analysis: 2026-05-13*
