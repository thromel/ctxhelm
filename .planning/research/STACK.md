# Stack Research

**Domain:** Local-first, agent-native repository context compiler for coding agents
**Researched:** 2026-05-13
**Confidence:** HIGH for current stack hardening; MEDIUM for optional new libraries until measured in this codebase

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended | Confidence |
|------------|---------|---------|-----------------|------------|
| Rust + Cargo workspace | Rust 2021 edition; current local baseline rustc/cargo 1.87.0 | CLI, indexer, compiler, MCP server, tests | Keep the existing workspace. It matches the product's local-first, fast binary, read-only filesystem/git workload and already has crate boundaries for core contracts, index, compiler, MCP, and CLI. Do not change language or edition as part of retrieval hardening. | HIGH |
| Typed serde contracts | serde 1.0.228, serde_json 1.0.149 | Stable JSON contracts for CLI, MCP, packs, traces, and eval reports | This is the right contract layer. Roadmap work should add fields compatibly and preserve camelCase public JSON. Avoid stringly CLI output as an internal API. | HIGH |
| clap derive CLI | clap 4.6.1 | CLI command parsing and help | Keep. The CLI is a setup/debug/automation surface, and clap provides polished help, derive parsing, and semver-compatible Rust CLI behavior. Add binary-level tests before expanding command flags. | HIGH |
| ignore safe walker | ignore 0.4.25 | Safe repository inventory and ignore-rule semantics | Keep as the inventory front door. It explicitly supports fast recursive traversal with `.gitignore`-style filters and fine-grained matchers. Build freshness/privacy policy around it instead of replacing it with raw walkdir. | HIGH |
| blake3 hashes | blake3 1.8.5 | Fast file and task fingerprints | Keep. The next stack direction is streaming and metadata-aware hashing, not switching hash algorithms. Hash file bytes with size/mtime/ignore-policy metadata for freshness checks. | HIGH |
| MCP over JSON-RPC stdio | MCP specification 2025-11-25; current hand-rolled serde_json server | Agent-native tools/resources/prompts | Keep the small MCP surface and stdio default. MCP's official spec is JSON-RPC based and defines tools, resources, and prompts; ctxpack already maps cleanly to that model. Add protocol conformance tests before changing transport internals. | HIGH |
| rmcp SDK | rmcp 1.6.0, gated migration | Future MCP implementation library | Direction: evaluate migration of `ctxpack-mcp` to rmcp after current contracts are locked by tests. rmcp is the official Rust SDK and supports server tools, resources, prompts, stdio, and streamable HTTP. Do not migrate before retrieval/cache hardening unless protocol drift becomes the main risk. | MEDIUM |
| tree-sitter parsers | tree-sitter 0.26.8; tree-sitter-typescript 0.23.2; tree-sitter-python 0.25.0; tree-sitter-go 0.25.0; tree-sitter-rust 0.24.2 | Parser-backed symbols and dependency edges | Add parser-backed implementations behind existing `CodeSymbol` and `DependencyEdge` contracts. Start with TS/TSX, Python, Rust, and Go because those are already covered by heuristics. Keep heuristic parsing as a fallback and fixture-test every grammar. | HIGH |
| System git with centralized runner | git CLI on PATH | current diff, co-change hints, history samples, historical eval | Keep git CLI as source of truth for now. Centralize process execution, timeout handling, stderr capture, version diagnostics, and batched `git log --name-only` paths. Do not replace with gitoxide/gix yet. | HIGH |
| Local cache manifests | JSON manifest now; rusqlite 0.39.0 only when query/retention needs justify it | Inventory freshness, trace retention, pack/cache metadata | First add typed cache metadata next to the current JSON inventory. Move to SQLite only if retention queries, historical eval metadata, or large-repo incremental indexes outgrow simple files. SQLite is appropriate for local metadata, not as the first retrieval-quality fix. | MEDIUM |

### Supporting Libraries

| Library | Version | Purpose | When to Use | Confidence |
|---------|---------|---------|-------------|------------|
| Tantivy | 0.26.1 | Persistent full-text index and BM25-style lexical retrieval | Use after inventory freshness exists and historical eval shows linear lexical search is the bottleneck. Keep it behind an index abstraction and compare Recall@5/10 against the current lexical baseline before making it default. | MEDIUM |
| notify | 8.2.0 | Cross-platform filesystem change notifications | Use only for an explicit `ctxpack watch` or long-running MCP cache freshness mode. Do not require a daemon for default CLI/MCP use; many filesystems have watcher caveats, so polling/freshness checks must remain available. | MEDIUM |
| rayon | 1.12.0 | Bounded parallel inventory/search/parser work | Use after profiling shows CPU-bound scanning/parsing. Prefer per-operation budgets and deterministic output ordering over broad parallelism. | MEDIUM |
| rusqlite | 0.39.0 | Durable local metadata store | Use for cache metadata, trace retention, and eval run indexes if JSON/JSONL becomes awkward. Avoid using SQLite FTS as the main retrieval layer; Tantivy is a better Rust-native full-text engine if persistent search is needed. | MEDIUM |
| assert_cmd | 2.2.1 | CLI integration tests | Add for binary-level tests covering `index`, `search`, `prepare-task`, `get-pack`, `cards`, and eval commands. This directly addresses the current test gap around Clap wiring and side effects. | HIGH |
| trycmd | 1.2.0 | Snapshot-style CLI tests | Use for help output and stable example commands that should also feed documentation. Avoid over-snapshotting dynamic JSON unless values are normalized. | MEDIUM |
| tempfile | 3.27.0 | Isolated fixture repositories and cache homes | Keep. Existing tests already use it correctly for repo and home isolation. | HIGH |
| thiserror + anyhow | thiserror 2.0.18, anyhow 1.0.102 | Layered error handling | Keep typed domain errors in libraries and anyhow at the CLI boundary. Add structured diagnostics rather than free-text warnings for cache, git, unreadable files, and skipped files. | HIGH |
| which | 8.0.2 | External tool discovery | Add when centralizing `git` and `tar` diagnostics. It is better than discovering missing tools only after process spawn failure. | MEDIUM |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo test --workspace` | Required project validation | Keep as the completion gate for implementation work. Add focused crate tests plus binary tests before large refactors. |
| `cargo run -p ctxpack -- --help` | CLI smoke after command changes | Required by project rules. Snapshot with trycmd once command output stabilizes. |
| `cargo clippy --workspace --all-targets` | Refactor safety | Not currently mandated in AGENTS.md, but should become a roadmap validation gate before large module splits and parser integrations. |
| Historical eval on real repos | Retrieval-quality proof | Keep RefactoringMiner as the large-history smoke. Every scoring/parser/index change should report recall delta against lexical baseline, not just pass unit tests. |
| Live MCP smoke with explicit `repo` | Client integration proof | Keep Codex CLI and Claude Code smoke paths separated from protocol unit tests. The README already documents real client smoke status; make this repeatable. |

## Installation

Do not add all optional libraries at once. The stack should grow in the order below so each dependency has a measurable reason.

```bash
# Immediate hardening: CLI tests and external-tool diagnostics
cargo add --workspace --dev assert_cmd@2.2.1 trycmd@1.2.0
cargo add --workspace which@8.0.2

# Parser-backed symbols/dependencies, added behind feature-gated modules or narrow crate boundaries
cargo add -p ctxpack-index tree-sitter@0.26.8
cargo add -p ctxpack-index tree-sitter-typescript@0.23.2 tree-sitter-python@0.25.0 tree-sitter-go@0.25.0 tree-sitter-rust@0.24.2

# Optional after measurement
cargo add -p ctxpack-index tantivy@0.26.1
cargo add -p ctxpack-index rayon@1.12.0
cargo add -p ctxpack-index notify@8.2.0
cargo add -p ctxpack-index rusqlite@0.39.0

# Optional MCP migration after protocol contract tests are in place
cargo add -p ctxpack-mcp rmcp@1.6.0 --features server,transport-io,macros,schemars
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Keep Rust workspace | TypeScript/Node service | Only if the product becomes an editor extension first. That conflicts with the current local binary and multi-agent MCP/CLI surface. |
| MCP stdio default | Remote HTTP MCP server | Use HTTP only for an explicit future multi-machine/team mode. For current privacy and local-first trust, stdio is the right default. |
| rmcp gated migration | Continue hand-rolled JSON-RPC forever | Continue hand-rolled while the surface is small and tested. Move to rmcp if spec evolution, cancellation/progress/logging, or schema maintenance starts consuming roadmap time. |
| tree-sitter narrow parser set | LSP/LSIF/SCIP as default | Use LSP/SCIP later for deep cross-reference precision in supported languages. For next phases, tree-sitter gives local, fast, read-only syntax structure without running language servers. |
| git CLI plus better runner | gitoxide/gix | Consider gix only for a narrow hot path after profiling. Official gitoxide docs still describe `gix` as unstable and not a git replacement. |
| Tantivy optional persistent index | SQLite FTS or cloud/vector search | Use SQLite only for metadata. Use vector search only as an opt-in experiment after source-free evals prove lexical+symbol+graph is insufficient. |
| JSON manifest first | SQLite immediately | Use SQLite when retention and incremental metadata need real queries. For cache freshness, a typed manifest is simpler and less migration-heavy. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Cloud embeddings, cloud indexing, or cloud reranking by default | Violates the local-first trust contract and creates source-exposure risk. It also does not address current stale-cache/privacy gaps. | Local lexical, parser, graph, and history signals; optional local-only experiments behind explicit config. |
| Autonomous editing or test execution inside ctxpack | The product is a context broker. Editing, approvals, shell execution, and commits belong to host agents. | Emit precise context packs, validation commands, risk flags, and diagnostics. |
| Expanding MCP tool count for each new capability | More tools increase agent decision overhead. The current project explicitly keeps MCP small. | Extend existing `prepare_task`, `search`, `related`, `get_pack`, `related_tests`, and `current_diff` inputs/structured outputs compatibly. |
| Replacing git CLI with gitoxide/gix in the near term | gitoxide docs say `gix` is unstable and not a git replacement. History correctness matters more than avoiding subprocesses right now. | Centralize git CLI calls, batch them, add timeouts, and surface partial-result diagnostics. |
| Whole-repo vector databases as the next retrieval fix | Current eval weakness is graph/history lift and stale/heuristic signals, not lack of embeddings. Vector search also complicates privacy, determinism, and packaging. | Parser-backed dependency/symbol extraction, measured graph expansion, Tantivy only when lexical indexing becomes a scale bottleneck. |
| Generic all-language parser bundles by default | Increases binary/build complexity and test surface without evidence of value. | Add language parsers only for languages covered by fixtures and eval targets. |
| SQLite FTS as the primary search engine | It is convenient, but Tantivy is the stronger Rust search library when full-text search becomes a first-class index. | Tantivy for persistent lexical search; rusqlite for metadata and traces. |
| Long-running background daemon as required architecture | It weakens the simple local CLI/MCP product shape and creates lifecycle/debug complexity. | On-demand CLI/MCP with freshness checks; optional `watch` mode later. |

## Stack Patterns by Variant

**If the next phase is correctness and trust hardening:**
- Use the current Rust/serde/ignore/blake3 stack.
- Add typed freshness metadata, broader privacy policy tests, structured diagnostics, and binary CLI tests.
- Do not add Tantivy, rmcp, SQLite, or notify in the same phase unless the phase explicitly measures them.

**If the next phase is retrieval-quality lift:**
- Add tree-sitter-backed symbols and dependency edges behind existing contracts.
- Improve graph/history fusion in the compiler and prove recall lift with source-free historical evals.
- Keep hand-rolled lexical search until parser/graph changes prove where the remaining bottleneck is.

**If the next phase is large-repo performance:**
- Add operation budgets, text read limits, batched git metadata, and possibly rayon.
- Add Tantivy only behind an index abstraction and evaluate against the current lexical baseline.
- Add notify only for explicit watch mode; cache freshness must not depend on file watchers.

**If the next phase is MCP protocol maturity:**
- Preserve the six-tool surface.
- Add conformance tests for initialize, capabilities, tools/list, tools/call, resources/list/read, prompts/list/get, errors, structuredContent, and client reconnection behavior.
- Then evaluate rmcp migration to reduce protocol drift.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| Current workspace | Rust 2021, cargo resolver 2 | Keep edition unchanged. Add `rust-toolchain.toml` only after choosing a release/MSRV policy. |
| tree-sitter 0.26.8 | tree-sitter-language based grammar crates | Grammar crate versions are not uniform. Introduce a parser adapter module and fixture tests before relying on tags/queries in planning. |
| rmcp 1.6.0 | tokio 1, serde 1, serde_json 1 | The workspace already declares tokio but does not consume it. rmcp migration will make async runtime choices real; isolate it inside `ctxpack-mcp`. |
| Tantivy 0.26.1 | Local cache directory under CTXPACK_HOME | Treat the index as derived data. Rebuild on inventory policy/version/hash changes and keep source snippets out of eval traces. |
| notify 8.2.0 | Optional watch mode | Docs list watcher caveats for network filesystems, Docker/macOS, editor save behavior, and large directories. Polling/freshness checks remain required. |
| rusqlite 0.39.0 | Local metadata only | Use migrations and schema versioning if introduced. Do not put public contracts behind ad hoc SQL rows; serialize typed structs at boundaries. |
| assert_cmd 2.2.1 | tempfile 3.27.0 | Use for isolated CLI tests with `CTXPACK_HOME` and fixture repos. |
| trycmd 1.2.0 | clap 4.6.1 | Use for stable help/example snapshots; normalize paths and dynamic IDs. |

## Recommended Technical Direction

The right 2026 direction is not "add semantic search." It is to make ctxpack a more reliable local compiler of repository evidence: fresh inventory, stronger privacy policy, parser-backed graph signals, measurable retrieval lift, and protocol-compatible agent surfaces.

The next roadmap should sequence stack changes as follows:

1. **Trust and diagnostics first** - keep the current stack, add freshness manifests, privacy corpus tests, read-error/skipped-file diagnostics, centralized process execution, and binary CLI tests.
2. **Parser-backed retrieval second** - add tree-sitter adapters for TS/TSX, Python, Rust, and Go; keep heuristic fallback; measure graph/symbol lift on historical evals.
3. **Index/runtime upgrades third** - add Tantivy, rayon, rusqlite, notify, or rmcp only when a phase has a measured need and contract tests are already protecting behavior.
4. **Agent-native compatibility always** - preserve MCP/AGENTS/native-rule surfaces, keep the MCP tool list small, and keep source-free traces/evals as a non-negotiable constraint.

## Sources

- `.planning/PROJECT.md` - current product goals, validated/active requirements, constraints, and key decisions. Confidence: HIGH.
- `.planning/codebase/STACK.md` - current workspace, dependency, runtime, and platform inventory. Confidence: HIGH.
- `.planning/codebase/ARCHITECTURE.md` - crate boundaries, data flow, MCP/session behavior, and contract locations. Confidence: HIGH.
- `.planning/codebase/CONCERNS.md` - stale cache, privacy, parser, CLI test, git/history, and performance gaps. Confidence: HIGH.
- https://modelcontextprotocol.io/specification/2025-11-25 - MCP JSON-RPC architecture, tools/resources/prompts, and security principles. Confidence: HIGH.
- https://docs.rs/rmcp/latest/rmcp/ - official Rust MCP SDK 1.6.0 and stdio/server feature support. Confidence: HIGH.
- https://docs.rs/ignore/latest/ignore/ - ignore 0.4.25 traversal and gitignore semantics. Confidence: HIGH.
- https://docs.rs/tree-sitter/latest/tree_sitter/ - tree-sitter 0.26.8 Rust bindings. Confidence: HIGH.
- https://docs.rs/tree-sitter-typescript/latest/tree_sitter_typescript/ - TypeScript/TSX grammar crate 0.23.2. Confidence: HIGH.
- https://docs.rs/tree-sitter-python/latest/tree_sitter_python/ - Python grammar crate 0.25.0. Confidence: HIGH.
- https://docs.rs/tree-sitter-go/latest/tree_sitter_go/ - Go grammar crate 0.25.0. Confidence: HIGH.
- https://docs.rs/tree-sitter-rust/latest/tree_sitter_rust/ - Rust grammar crate 0.24.2. Confidence: HIGH.
- https://docs.rs/tantivy/latest/tantivy/ - Tantivy 0.26.1 search engine library. Confidence: HIGH.
- https://docs.rs/gitoxide/latest/gitoxide/ - gitoxide/gix stability and non-replacement caveat. Confidence: HIGH.
- https://docs.rs/notify/latest/notify/ - notify 8.2.0 filesystem watcher capabilities and caveats. Confidence: HIGH.
- https://docs.rs/rusqlite/latest/rusqlite/ - rusqlite 0.39.0 SQLite wrapper. Confidence: HIGH.
- https://docs.rs/assert_cmd/latest/assert_cmd/ - assert_cmd 2.2.1 CLI test support. Confidence: HIGH.
- https://docs.rs/trycmd/latest/trycmd/ - trycmd 1.2.0 CLI snapshot support. Confidence: HIGH.

---
*Stack research for: local-first agent-native repository context compiler*
*Researched: 2026-05-13*
