# External Integrations

**Analysis Date:** 2026-05-13

## APIs & External Services

**Model Context Protocol:**
- MCP stdio server - local JSON-RPC interface for coding agents.
  - SDK/Client: no external MCP SDK; implemented directly with `serde_json`, `std::io::BufRead`, and `std::io::Write` in `crates/ctxhelm-mcp/src/lib.rs`.
  - Auth: none.
  - Entrypoint: `ctxhelm serve-mcp` dispatches to `ctxhelm_mcp::run_stdio_server()` in `crates/ctxhelm/src/main.rs`.
  - Protocol version: `2025-11-25` constant in `crates/ctxhelm-mcp/src/lib.rs`.
  - JSON-RPC methods: `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, and `prompts/get` in `crates/ctxhelm-mcp/src/lib.rs`.
  - Implemented tools: `prepare_task`, `search`, `related`, `get_pack`, `related_tests`, and `current_diff` in `crates/ctxhelm-mcp/src/lib.rs`.
  - Implemented static resources: `ctxhelm://repo/summary`, `ctxhelm://repo/test-map`, `ctxhelm://repo/dependency-graph`, and `ctxhelm://pack/guide` in `crates/ctxhelm-mcp/src/lib.rs`.
  - Implemented dynamic resources: `ctxhelm://pack/<task-id>/<budget>`, `ctxhelm://pack/<task-id>/<budget>.json`, `ctxhelm://file/<path>`, and `ctxhelm://symbol/<query>` in `crates/ctxhelm-mcp/src/lib.rs`.
  - Implemented prompts: `bugfix`, `feature`, `refactor`, `review_diff`, `write_tests`, and `explain_area` in `crates/ctxhelm-mcp/src/lib.rs`.

**Agent Configuration Surfaces:**
- Codex - setup guidance is printed by `ctxhelm init`.
  - SDK/Client: external Codex client configured by the user; this repo does not mutate global Codex config.
  - Auth: none in this repo.
  - Implementation: `CODEX_MCP_SETUP` in `crates/ctxhelm-core/src/init.rs`; `README.md` documents local smoke status for Codex CLI.
- Claude Code - optional project MCP snippet and slash-command file.
  - SDK/Client: `.mcp.json` consumer outside this repo.
  - Auth: none in this repo.
  - Implementation: `.claude/commands/ctxhelm-bugfix.md` and `.ctxhelm/adapters/claude-mcp.json` are generated from `crates/ctxhelm-core/src/init.rs`.
- Cursor - optional always-apply rules file.
  - SDK/Client: Cursor rules file consumed by Cursor.
  - Auth: none in this repo.
  - Implementation: `.cursor/rules/ctxhelm.mdc` is generated from `crates/ctxhelm-core/src/init.rs`.
- OpenCode - optional local MCP configuration snippet.
  - SDK/Client: OpenCode config file consumed outside this repo.
  - Auth: none in this repo.
  - Implementation: `.ctxhelm/adapters/opencode.jsonc.snippet` is generated from `crates/ctxhelm-core/src/init.rs`.

**Local Process Integrations:**
- Git CLI - local repository metadata and working-tree state.
  - SDK/Client: `std::process::Command`.
  - Auth: whatever local git already uses; no credentials are read or managed by ctxhelm.
  - Uses: `git diff --name-only`, `git diff --cached --name-only`, `git ls-files --others --exclude-standard`, `git log`, `git rev-list`, `git show`, `git ls-tree`, and `git archive` paths in `crates/ctxhelm-index/src/lib.rs` and `crates/ctxhelm-compiler/src/lib.rs`.
- tar CLI - extracts selected files from historical git archive streams.
  - SDK/Client: `std::process::Command`.
  - Auth: none.
  - Implementation: historical eval snapshot extraction in `crates/ctxhelm-compiler/src/lib.rs`.

**Remote APIs:**
- Not detected.
  - No `reqwest`, `hyper`, `ureq`, cloud SDK, OpenAI SDK, Anthropic SDK, Stripe SDK, Supabase SDK, AWS SDK, or HTTP server dependency appears in `Cargo.toml`, crate manifests, or source imports.
  - Product docs in `README.md`, `AGENTS.md`, and `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md` describe local-only behavior and no default cloud indexing, embeddings, or reranking.

## Data Storage

**Databases:**
- No database engine detected.
  - No SQLite, Postgres, MySQL, Redis, vector database, ORM, migration framework, or database connection string appears in manifests or source.
  - The product spec mentions no vector database for the default MVP in `docs/superpowers/specs/2026-05-09-repo-context-packer-product-spec.md`.

**File Storage:**
- Local filesystem only.
  - Safe inventory is written to `~/.ctxhelm/repos/<repo-id>/inventory.json` by `write_inventory()` in `crates/ctxhelm-index/src/lib.rs`.
  - Eval traces append to `~/.ctxhelm/repos/<repo-id>/traces.jsonl` by `append_eval_trace()` in `crates/ctxhelm-index/src/lib.rs`.
  - `CTXHELM_HOME` overrides the `~/.ctxhelm` state root in `crates/ctxhelm-index/src/lib.rs`.
  - `ctxhelm init` writes `.ctxhelm/ctxhelm.toml` and updates `AGENTS.md` in the target repo through `crates/ctxhelm-core/src/init.rs`.
  - Optional adapter files are written under `.cursor/rules/`, `.claude/commands/`, and `.ctxhelm/adapters/` by `crates/ctxhelm-core/src/init.rs`.
  - Optional context cards are written to `.ctxhelm/cards/repo-overview.md`, `.ctxhelm/cards/testing.md`, and `.ctxhelm/cards/dependency-graph.md` by `generate_context_cards()` in `crates/ctxhelm-compiler/src/lib.rs`.

**Caching:**
- Local session memory for MCP pack resources.
  - `ctxhelm://pack/<task-id>/<budget>` resources are cached in an in-process `OnceLock<Mutex<BTreeMap<...>>>` in `crates/ctxhelm-mcp/src/lib.rs`.
  - Pack resources are session-scoped and become available after `prepare_task` caches them in `crates/ctxhelm-mcp/src/lib.rs`.
- Local persistent state is JSON/JSONL under `CTXHELM_HOME` or `~/.ctxhelm`, not a cache service.

## Authentication & Identity

**Auth Provider:**
- Not detected.
  - There is no user account model, token validation, OAuth/OIDC, session handling, password storage, API key loading, or auth middleware in the Rust code.
  - MCP requests are accepted over the local stdio process boundary in `crates/ctxhelm-mcp/src/lib.rs`.

**Identity:**
- Repository identity is deterministic and local.
  - `repo_id_for_path()` derives repo IDs with UUID v5 from canonical repo paths in `crates/ctxhelm-index/src/lib.rs`.
  - Task hashes use BLAKE3 over task text in `task_hash()` in `crates/ctxhelm-index/src/lib.rs`; traces intentionally avoid storing source snippets.

## Monitoring & Observability

**Error Tracking:**
- None.
  - No Sentry, OpenTelemetry, Datadog, Honeycomb, Prometheus, or tracing subscriber dependencies were detected.

**Logs:**
- Structured command output and local trace files.
  - CLI commands print human-readable reports or pretty JSON in `crates/ctxhelm/src/main.rs`.
  - MCP responses return JSON-RPC success/error objects in `crates/ctxhelm-mcp/src/lib.rs`.
  - Source-free local eval traces are appended to `~/.ctxhelm/repos/<repo-id>/traces.jsonl` in `crates/ctxhelm-index/src/lib.rs`.
  - `README.md` documents `ctxhelm eval traces` and `ctxhelm eval checklist` for inspecting local traces.

## CI/CD & Deployment

**Hosting:**
- Not detected.
  - The repo is a local CLI/MCP project with no deployment manifests, Dockerfiles, Kubernetes configs, serverless configs, or hosting-provider config files detected.

**CI Pipeline:**
- Not detected.
  - No GitHub Actions, GitLab CI, CircleCI, Buildkite, or other CI configuration was found in the mapped file set.
  - Validation commands are documented in `AGENTS.md` and `README.md`: `cargo test --workspace` and `cargo run -p ctxhelm -- --help`.

## Environment Configuration

**Required env vars:**
- None required for default operation.
- Optional:
  - `CTXHELM_HOME` - overrides the private local state root for inventory and trace files in `crates/ctxhelm-index/src/lib.rs`.
  - `HOME` - fallback base for `~/.ctxhelm` when `CTXHELM_HOME` is absent in `crates/ctxhelm-index/src/lib.rs`.

**Secrets location:**
- Not applicable.
  - No secret-bearing config files were detected.
  - No `.env`, `.env.*`, `*.env`, `credentials.*`, `secrets.*`, `*.pem`, `*.key`, or `.npmrc` files were read or found during mapping.
  - The code excludes sensitive files from safe inventory by default in `crates/ctxhelm-index/src/lib.rs`.

## Webhooks & Callbacks

**Incoming:**
- None.
  - There is no HTTP listener, route framework, webhook endpoint, or network callback handler.
  - The only request surface is the local stdio MCP loop in `crates/ctxhelm-mcp/src/lib.rs`.

**Outgoing:**
- None.
  - The implementation invokes local `git` and `tar` processes but does not make network calls, send webhooks, or call hosted APIs.

---

*Integration audit: 2026-05-13*
