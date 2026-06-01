# Architecture Research: v1.1 Packaging & Adoption

**Domain:** local-first repository context broker for coding agents
**Researched:** 2026-05-13
**Confidence:** HIGH for repo architecture and contract impacts; MEDIUM for release tooling choice until a dry-run release is performed.

## Recommended Architecture

v1.1 should package the existing product without changing the runtime architecture. Keep the Rust workspace layers intact and add a thin release/adoption layer around them:

```text
docs/ + README
  -> install, first pack, agent setup, smoke/troubleshooting

scripts/
  -> release gates, deterministic MCP smoke, optional real-client smoke

crates/ctxhelm
  -> installed binary: init, serve-mcp, smoke-friendly CLI commands

crates/ctxhelm-core
  -> stable contracts, repo-local setup artifacts, adapter templates

crates/ctxhelm-mcp
  -> local stdio MCP runtime with no non-JSON stdout
```

Do not introduce a hosted service, background daemon, editor plugin, or autonomous setup writer for v1.1. The release should distribute one `ctxhelm` binary and repo-local adapter artifacts; MCP clients launch `ctxhelm serve-mcp`.

## Component Boundaries

| Component | Responsibility | v1.1 Impact |
|-----------|----------------|-------------|
| `crates/ctxhelm/src/main.rs` | CLI entry point and `serve-mcp` launcher. | Add only packaging-safe commands or flags if required by install/smoke flows; keep command output contract-tested. |
| `crates/ctxhelm-core/src/init.rs` | `AGENTS.md`, `.ctxhelm/ctxhelm.toml`, and adapter templates. | Treat adapter text as product contract. Add new setup artifacts here, not in CLI string literals. |
| `crates/ctxhelm-mcp/src/*` | MCP stdio protocol, tools, resources, prompts, session cache. | Preserve existing tool/resource names and structuredContent shapes. No stdout logging. |
| `scripts/smoke-mcp-protocol.sh` | Deterministic protocol smoke. | Make it the required packaging gate because it does not depend on Codex/Claude auth. |
| `scripts/smoke-codex-mcp.sh`, `scripts/smoke-claude-mcp.sh` | Optional real-client smokes with required mode. | Keep optional by default; CI/release can set `CTXHELM_REQUIRE_REAL_CLIENT=1` only on machines with clients/auth. |
| Release scripts/workflow | Build, package, checksum, smoke installed binary. | New boundary. Should call the installed binary, not `cargo run`, after artifact installation. |
| Docs | Install and adoption path. | Keep docs task-oriented: install -> init -> MCP config -> first `prepare_task`/`get_pack` -> smoke/troubleshooting. |

## Release Module And Script Boundaries

Use a `scripts/release-check.sh` style gate as the architectural join point between code and packaging. It should be boring and repeatable:

1. `cargo test --workspace`
2. `cargo run -p ctxhelm -- --help`
3. `cargo package --list` / `cargo publish --dry-run` when publishing to crates.io
4. Build release binary
5. Install into a temp prefix or use the packaged artifact
6. Run deterministic CLI and MCP smoke against the installed `ctxhelm`
7. Run optional real-client smokes only when explicitly required

Prefer crates.io `cargo install ctxhelm` as the first adoption path if the crate name and metadata are available. It matches Cargo's native binary install model and keeps v1.1 small. Add prebuilt GitHub Release artifacts after that gate is reliable. `cargo-dist` is attractive for later because it plans/builds installers and GitHub Release artifacts, but do not make it the first v1.1 dependency unless a local dry run proves it does not complicate the workspace.

## Setup And Adapter Architecture

`ctxhelm init` should remain repo-local and non-invasive:

- Writes `.ctxhelm/ctxhelm.toml`, managed `AGENTS.md` section, and optional Cursor/Claude/OpenCode adapter snippets.
- Prints Codex setup guidance but must not mutate global Codex config.
- Keeps Claude MCP config as a mergeable repo-local snippet, not an automatic global write.
- Keeps adapter guidance thin: call `prepare_task`, pass explicit `repo`, read files natively, request `get_pack` progressively.

If v1.1 adds setup helpers, make them generated artifacts or validation commands, not client-specific global mutators. This preserves the product rule that ctxhelm brokers context but does not take over agent configuration or editing authority.

## Docs Layout

Recommended concise docs structure:

```text
README.md                         # shortest install-to-first-pack path
docs/install.md                   # cargo install, binary artifacts, PATH, uninstall
docs/agent-setup.md               # Codex, Claude, Cursor, OpenCode setup snippets
docs/smoke-tests.md               # protocol smoke, real-client smoke, env vars
docs/troubleshooting.md           # PATH, wrong repo cwd, CTXHELM_HOME, MCP stdout, stale cache
docs/contracts.md                 # stable CLI/MCP/JSON compatibility notes
```

Avoid long static repository maps in docs and adapters. The docs should teach users to invoke dynamic ctxhelm calls, not paste ctxhelm output into rule files.

## Smoke-Test Harnesses

The release smoke architecture should have three tiers:

| Tier | Gate | Why |
|------|------|-----|
| Deterministic binary smoke | `ctxhelm --help`, `ctxhelm init`, `ctxhelm prepare-task --no-trace`, `ctxhelm get-pack --format json --no-trace` on a fixture repo. | Proves installed binary, CLI contract, and repo-local writes. |
| Deterministic MCP protocol smoke | `scripts/smoke-mcp-protocol.sh` against installed `ctxhelm serve-mcp`. | Proves stdio JSON-RPC behavior without client auth/model variability. |
| Optional real-client smoke | Codex/Claude wrappers with server-side request logs and explicit `repo`. | Proves adoption path when clients are available; should fail only in required mode. |

The current protocol smoke uses `cargo run`; v1.1 should allow `CTXHELM_BIN=/path/to/ctxhelm` or equivalent so packaging can test the artifact users will run.

## Read-Only And Contract Preservation

v1.1 packaging must not weaken existing safety boundaries:

- No source edits, commits, test execution, dependency installation, global config mutation, cloud indexing, cloud embeddings, or hosted backend.
- Repo-local writes are limited to ctxhelm-owned setup/cache/card artifacts already allowed by product rules.
- Public JSON remains camelCase; existing CLI commands, MCP tool names, resource URI shapes, and structuredContent fields remain compatible.
- Any new release metadata should be additive and outside `ContextPlan`/`ContextPack` unless contract tests are updated deliberately.
- MCP stdout must contain only valid MCP JSON-RPC messages; logs and diagnostics go to stderr or structured responses.

## Patterns To Follow

### Artifact-First Release Gate

Build once, install the artifact into an isolated prefix, then smoke that installed binary. This catches missing files, PATH assumptions, and `cargo run`-only behavior before users hit it.

### Repo-Local Adapter Generation

Keep all generated agent guidance in `ctxhelm-core::init`, covered by tests for size, dynamic wording, explicit `repo`, and no static context dumps.

### Optional Real-Client Proof

Real Codex/Claude smokes are valuable, but they are operationally flaky because auth, model availability, and client versions change. Keep the deterministic protocol gate mandatory and the real-client gate optional unless the release machine is provisioned.

## Anti-Patterns To Avoid

| Anti-Pattern | Why Bad | Instead |
|--------------|---------|---------|
| Release scripts call `cargo run` only | Tests the source tree, not the installed product. | Smoke the packaged `ctxhelm` binary. |
| Auto-edit global Codex/Claude/Cursor config | Surprising side effect and hard to undo. | Generate snippets and docs; let user/client own global config. |
| Add a daemon or hosted service for adoption | Expands trust and support surface beyond v1.1. | Local stdio MCP launched by clients. |
| Put repo maps into adapters/docs | Stale, token-heavy, undermines dynamic context. | Thin guidance that calls `prepare_task`/`get_pack`. |
| Change MCP/JSON contracts during packaging | Destabilizes already-validated client durability. | Add compatibility tests first; keep release work additive. |

## Scalability Considerations

| Concern | v1.1 Approach |
|---------|---------------|
| Multi-platform install | Start with Cargo install plus release tarballs/checksums; add full installer automation after dry-run confidence. |
| Client variability | Separate deterministic protocol smoke from optional real-client smoke. |
| Support burden | Put PATH, explicit `repo`, wrong cwd, `CTXHELM_HOME`, and stdout logging failures in troubleshooting docs. |
| Contract drift | Run CLI compatibility, public JSON, MCP protocol/resource/prompt, and adapter guidance tests in the release gate. |
| Read-only trust | Keep setup writes repo-local and all runtime behavior source-read/context-only. |

## Sources

- Local: `.planning/PROJECT.md`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`
- Local: `crates/ctxhelm-core/src/init.rs`, `crates/ctxhelm/src/main.rs`, `scripts/smoke-*.sh`, `crates/ctxhelm/tests/cli_compat.rs`
- Official MCP transport spec, 2025-06-18: https://modelcontextprotocol.io/specification/2025-06-18/basic/transports
- Cargo Book, `cargo install`: https://doc.rust-lang.org/stable/cargo/commands/cargo-install.html
- Cargo Book, publishing and dry-run packaging: https://doc.rust-lang.org/cargo/reference/publishing.html
- dist/cargo-dist book, release automation context: https://axodotdev.github.io/cargo-dist/book/introduction.html
