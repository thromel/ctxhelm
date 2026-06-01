# Technology Stack

**Analysis Date:** 2026-05-13

## Languages

**Primary:**
- Rust 2021 edition - all executable and library code lives in the Cargo workspace rooted at `Cargo.toml`.
  - CLI binary: `crates/ctxhelm/src/main.rs`
  - Shared contracts and init logic: `crates/ctxhelm-core/src/lib.rs`, `crates/ctxhelm-core/src/contracts.rs`, `crates/ctxhelm-core/src/init.rs`, `crates/ctxhelm-core/src/privacy.rs`, `crates/ctxhelm-core/src/repo.rs`
  - Repository inventory, search, symbols, git history, and dependency graph logic: `crates/ctxhelm-index/src/lib.rs`
  - Context plan, pack, cards, and eval compilation: `crates/ctxhelm-compiler/src/lib.rs`
  - Stdio MCP runtime: `crates/ctxhelm-mcp/src/lib.rs`

**Secondary:**
- Markdown - product docs, roadmap, milestone plans, and generated agent instruction surfaces.
  - Project instructions: `AGENTS.md`
  - User documentation: `README.md`
  - Product spec: `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md`
  - Milestone plans: `docs/superpowers/plans/`
- JSON and TOML - generated configuration and protocol payload formats.
  - Cargo manifests: `Cargo.toml`, `crates/ctxhelm/Cargo.toml`, `crates/ctxhelm-core/Cargo.toml`, `crates/ctxhelm-index/Cargo.toml`, `crates/ctxhelm-compiler/Cargo.toml`, `crates/ctxhelm-mcp/Cargo.toml`
  - Generated repo config template: `.ctxhelm/ctxhelm.toml` from `crates/ctxhelm-core/src/init.rs`
  - Generated Claude MCP snippet: `.ctxhelm/adapters/claude-mcp.json` from `crates/ctxhelm-core/src/init.rs`
  - Generated OpenCode snippet: `.ctxhelm/adapters/opencode.jsonc.snippet` from `crates/ctxhelm-core/src/init.rs`

## Runtime

**Environment:**
- Rust toolchain from the local development environment.
  - `rustc 1.87.0 (17067e9ac 2025-05-09) (Homebrew)` was detected while mapping.
  - `cargo 1.87.0 (Homebrew)` was detected while mapping.
- No pinned toolchain file is present. `rust-toolchain`, `rust-toolchain.toml`, `.nvmrc`, and `.python-version` were not detected.
- Runtime entrypoints are local command-line processes, not web servers.
  - `ctxhelm` CLI command dispatch is in `crates/ctxhelm/src/main.rs`.
  - `ctxhelm serve-mcp` starts a JSON-RPC-over-stdio MCP server via `ctxhelm_mcp::run_stdio_server()` in `crates/ctxhelm/src/main.rs` and `crates/ctxhelm-mcp/src/lib.rs`.

**Package Manager:**
- Cargo 1.87.0
- Lockfile: present at `Cargo.lock`
- Workspace resolver: `resolver = "2"` in `Cargo.toml`
- Workspace members:
  - `crates/ctxhelm`
  - `crates/ctxhelm-core`
  - `crates/ctxhelm-index`
  - `crates/ctxhelm-compiler`
  - `crates/ctxhelm-mcp`

## Frameworks

**Core:**
- Cargo workspace - crate organization and shared dependency versioning in `Cargo.toml`.
- clap 4.6.1 - derive-based CLI parsing in `crates/ctxhelm/src/main.rs`.
- serde 1.0.228 and serde_json 1.0.149 - typed JSON contracts, CLI output, MCP payloads, inventory files, traces, and generated reports across `crates/ctxhelm-core/src/contracts.rs`, `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, and `crates/ctxhelm-mcp/src/lib.rs`.
- Model Context Protocol over JSON-RPC stdio - implemented directly with `serde_json`, `std::io::BufRead`, and `std::io::Write` in `crates/ctxhelm-mcp/src/lib.rs`; no external MCP SDK is used.

**Testing:**
- Rust built-in test harness - tests are co-located in `#[cfg(test)] mod tests` blocks inside crate source files such as `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-mcp/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, and `crates/ctxhelm-core/src/init.rs`.
- tempfile 3.27.0 - temporary repositories, temporary homes, and fixture workspaces in crate tests.

**Build/Dev:**
- Cargo commands documented in `README.md` and `AGENTS.md`:
  - `cargo test --workspace`
  - `cargo run -p ctxhelm -- --help`
  - `cargo run -p ctxhelm -- serve-mcp`
- External process tools used by the implementation:
  - `git` is invoked with `std::process::Command` for co-change history, current diff, historical commit samples, and historical evals in `crates/ctxhelm-index/src/lib.rs` and `crates/ctxhelm-compiler/src/lib.rs`.
  - `tar` is invoked when extracting historical revision snapshots in `crates/ctxhelm-compiler/src/lib.rs`.

## Key Dependencies

**Critical:**
- `anyhow` 1.0.102 - CLI-level error propagation in `crates/ctxhelm/src/main.rs`.
- `clap` 4.6.1 - subcommands and typed arguments for `init`, `index`, `prepare-task`, `get-pack`, `search`, `symbols`, `related-tests`, `co-changes`, `dependencies`, `cards`, `eval`, and `serve-mcp` in `crates/ctxhelm/src/main.rs`.
- `serde` 1.0.228 - derives stable structured contracts in `crates/ctxhelm-core/src/contracts.rs`, `crates/ctxhelm-index/src/lib.rs`, `crates/ctxhelm-compiler/src/lib.rs`, and `crates/ctxhelm-mcp/src/lib.rs`.
- `serde_json` 1.0.149 - CLI JSON output, MCP JSON-RPC input/output, inventory serialization, trace serialization, and adapter snippet validation.
- `ignore` 0.4.25 - safe repository walking with `.gitignore`, `.ctxhelmignore`, and `.cursorignore` support in `crates/ctxhelm-index/src/lib.rs`.
- `blake3` 1.8.5 - file content hashes and task hashes in `crates/ctxhelm-index/src/lib.rs`.
- `uuid` 1.23.1 - repo IDs, plan IDs, pack IDs, and trace IDs in `crates/ctxhelm-core/src/contracts.rs`, `crates/ctxhelm-index/src/lib.rs`, and `crates/ctxhelm-compiler/src/lib.rs`.

**Infrastructure:**
- `thiserror` 2.0.18 - typed error enums in `crates/ctxhelm-core/src/init.rs`, `crates/ctxhelm-core/src/repo.rs`, and `crates/ctxhelm-index/src/lib.rs`.
- `tempfile` 3.27.0 - dev/test dependency for isolated filesystem and git fixtures in crate tests; it is also used by `crates/ctxhelm-compiler/src/lib.rs` for temporary historical eval worktrees.
- `tokio` 1 is declared in workspace dependencies in `Cargo.toml`, but no crate currently consumes it in its `Cargo.toml`; it does not appear in the active `cargo tree --workspace --depth 2` output.

## Configuration

**Environment:**
- Primary runtime configuration is CLI arguments and repo-local files.
  - CLI argument parsing is centralized in `crates/ctxhelm/src/main.rs`.
  - Repository discovery requires a `.git` ancestor and is implemented in `crates/ctxhelm-core/src/repo.rs`.
- `CTXHELM_HOME` controls private local state location in `crates/ctxhelm-index/src/lib.rs`.
  - If `CTXHELM_HOME` is absent, `HOME/.ctxhelm` is used.
  - If neither is available, `.ctxhelm` relative to the current process is used.
- No `.env`, `.env.*`, `*.env`, secret, credential, PEM, key, or `.npmrc` files were detected at mapping depth.

**Build:**
- `Cargo.toml` defines the Rust workspace, edition, license, member crates, and workspace dependency constraints.
- Each crate has its own manifest:
  - `crates/ctxhelm/Cargo.toml`
  - `crates/ctxhelm-core/Cargo.toml`
  - `crates/ctxhelm-index/Cargo.toml`
  - `crates/ctxhelm-compiler/Cargo.toml`
  - `crates/ctxhelm-mcp/Cargo.toml`
- `Cargo.lock` pins resolved dependency versions.
- `.gitignore` excludes `.worktrees/`, `/target/`, `/.ctxhelm/cache/`, `/.ctxhelm/index/`, `*.log`, and `.DS_Store`.

## Platform Requirements

**Development:**
- Rust/Cargo toolchain compatible with Rust 2021 edition.
- `git` CLI available on `PATH`.
  - Required for `co-changes`, `current_diff`, historical commit samples, and historical retrieval evals in `crates/ctxhelm-index/src/lib.rs` and `crates/ctxhelm-compiler/src/lib.rs`.
- `tar` CLI available on `PATH`.
  - Required for historical eval snapshot extraction in `crates/ctxhelm-compiler/src/lib.rs`.
- Local filesystem access for scanning repositories, writing `~/.ctxhelm/repos/<repo-id>/inventory.json`, appending `~/.ctxhelm/repos/<repo-id>/traces.jsonl`, and optionally writing `.ctxhelm/cards/*.md`.

**Production:**
- Local-first CLI binary and stdio MCP server.
- No hosted service, container runtime, HTTP server, cloud storage, cloud embeddings, cloud reranking, or remote database deployment target is detected in the repository.

---

*Stack analysis: 2026-05-13*
