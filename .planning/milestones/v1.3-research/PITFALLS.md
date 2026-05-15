# Domain Pitfalls

**Domain:** ctxpack v1.1 Packaging & Adoption
**Researched:** 2026-05-13
**Overall confidence:** HIGH

## Scope

v1.1 should make the completed v1 local context broker installable, configurable, documented, and smoke-testable through real agent clients. It should not add retrieval features, durable production storage, cloud/vector search, team sync, a UI, autonomous editing, or broad parser work. Those belong to v1.2+ or v2.

## Critical Pitfalls

### Pitfall 1: Install Works Only From the Source Checkout

**What goes wrong:** Users follow the README, but the installed `ctxpack` binary is missing, not on `PATH`, points MCP clients at `cargo run`, or only works from this repository.
**Why it happens:** v1 docs are developer-oriented. Cargo installs binaries into an install-root `bin` directory, and `$HOME/.cargo/bin` must be on `PATH`. `cargo install` also ignores packaged `Cargo.lock` unless `--locked` is used.
**Consequences:** First-run adoption fails with `spawn ctxpack ENOENT`, stale source-checkout paths in MCP config, or unreproducible dependency resolution.
**Prevention:** Make one blessed v1.1 install path and test it from a clean temp directory. Prefer release binaries plus `cargo install --locked --path .` as a developer fallback. Generate MCP snippets that use an absolute resolved binary path, not `cargo run`, unless explicitly in development mode.
**Detection:** A release gate should run `ctxpack --version`, `ctxpack --help`, `ctxpack serve-mcp` handshake, `ctxpack init`, and MCP smoke from outside the repo with a stripped-down `PATH`.

### Pitfall 2: MCP Config Points at the Wrong Repo or Working Directory

**What goes wrong:** The MCP server starts, but `prepare_task` reads the MCP server cwd instead of the active project, or generated setup assumes every client launches servers from the repo root.
**Why it happens:** Agent clients have different cwd and project-root behavior. Claude Code sets `CLAUDE_PROJECT_DIR` for stdio servers, but ctxpack already validated that explicit `repo` is the safer contract across clients.
**Consequences:** ctxpack recommends files from the wrong checkout, creates cache entries for the wrong repo, or looks broken even though the server is healthy.
**Prevention:** Keep every v1.1 setup guide and adapter instruction repo-explicit: call `prepare_task` and `get_pack` with `repo`. `ctxpack init` should show the active repo path and generated snippets should avoid relying on cwd.
**Detection:** CI should keep the existing wrong-cwd protocol smoke and add packaging-mode smoke where the server starts from a temp directory while `repo` points at a fixture project.

### Pitfall 3: MCP Client Version Drift Breaks Adoption

**What goes wrong:** Codex CLI, Claude Code, Cursor, or OpenCode changes MCP config syntax, protocol behavior, output limits, approval behavior, or scope precedence after docs are written.
**Why it happens:** MCP clients are moving quickly. The MCP spec negotiates protocol versions during initialization, Codex documents shared CLI/IDE MCP config, and Claude Code documents local/project/user MCP scopes with precedence rules and evolving stdio behavior.
**Consequences:** A published "works with Codex/Claude" claim becomes stale; users can list the server but cannot get machine-checkable `prepare_task` / `get_pack` calls.
**Prevention:** Publish a small compatibility matrix with exact client versions last verified. Keep deterministic JSON-RPC protocol smoke as the hard gate and treat real-client smokes as versioned adoption evidence, not the only proof.
**Detection:** Release CI should support `CTXPACK_SKIP_REAL_CLIENT=1` for portable protocol proof and `CTXPACK_REQUIRE_REAL_CLIENT=1` for maintainer machines before tagging.

### Pitfall 4: PATH and Shell Differences Are Hidden Until Runtime

**What goes wrong:** MCP config uses `ctxpack`, but the GUI/agent process cannot find it because it has a different login shell, launch environment, or PATH than the user's terminal.
**Why it happens:** Local stdio MCP clients spawn commands directly. Claude's docs explicitly advise using a full executable path when a command is not on PATH.
**Consequences:** Setup appears correct in text but fails only when the agent tries to start the MCP server.
**Prevention:** `ctxpack init` should resolve and print the absolute binary path used in snippets. Add troubleshooting for `which ctxpack`, `ctxpack --version`, and replacing `command: "ctxpack"` with the absolute path.
**Detection:** Add a smoke that empties or narrows `PATH` and verifies generated configs either still work through absolute paths or fail with a clear diagnostic.

### Pitfall 5: Docs Rot Faster Than the Product

**What goes wrong:** README, AGENTS guidance, Claude snippet, Codex snippet, scripts, and release notes disagree on install commands, verified client versions, supported tools, or session-scoped pack resources.
**Why it happens:** Packaging changes touch docs more than code, and client docs change independently.
**Consequences:** Users copy stale setup, over-trust pack resource durability, or miss the durable `get_pack` path.
**Prevention:** Make docs part of the release gate: one quickstart, one troubleshooting section, one compatibility matrix, and generated adapter text checked by tests. Keep pack resources documented as same-session only; recommend `get_pack` after reconnect.
**Detection:** Add tests or a script that greps docs for old `cargo run -p ctxpack -- serve-mcp` production snippets, obsolete client versions, and missing `repo` guidance.

### Pitfall 6: Packaging Secrets, Caches, or Local State

**What goes wrong:** Release artifacts, tarballs, generated cards, docs examples, or CI uploads include `.ctxpack`, `~/.ctxpack`-style caches, traces, real repo paths, tokens, request logs, or local MCP configs.
**Why it happens:** v1.1 focuses on packaging and smoke logs; those flows create temporary homes, request logs, generated cards, adapter snippets, and release archives.
**Consequences:** Privacy regression and loss of trust in the local-first claim.
**Prevention:** Release packaging must start from a clean checkout and explicit include list. Exclude `.ctxpack/`, `.codex/`, `.claude/`, temp smoke logs, traces, caches, target artifacts unless intentionally packaging binaries, and all local config files.
**Detection:** Add an artifact audit step that lists archive contents and fails on `.ctxpack`, `traces.jsonl`, `.env`, key/token-looking filenames, absolute `/Users/` paths, and request-log files.

### Pitfall 7: CI Proves Build Health, Not Adoption Health

**What goes wrong:** `cargo test --workspace` passes, but the release cannot be installed, the binary is not executable, docs commands fail, or MCP clients cannot launch it.
**Why it happens:** Existing validation is implementation-centered. v1.1 needs release/adoption gates: install, init, smoke, docs, and artifact audit.
**Consequences:** A tagged release is technically correct but unusable for new developers.
**Prevention:** Add a v1.1 release checklist or script that runs from a clean temp home: build/package, install into temp prefix, verify `PATH`, run `ctxpack init`, run deterministic MCP smoke, run docs command snippets where practical, and audit artifacts.
**Detection:** CI should fail if the smoke uses the workspace binary instead of the packaged/installed binary.

### Pitfall 8: v1.1 Becomes v1.2/v2 By Accident

**What goes wrong:** Packaging work expands into Homebrew taps, crates.io publishing, signed installers, durable storage redesign, embeddings, UI, team policy, or new MCP tools before the basic install path is reliable.
**Why it happens:** Adoption issues naturally reveal product wishes, but this milestone is about reducing time-to-first-pack for the current product.
**Consequences:** The release slips and the most important failure modes, install and MCP setup, remain unproven.
**Prevention:** Define v1.1 success as: installable artifact, absolute-path MCP setup, first-pack quickstart, client compatibility matrix, deterministic smoke, optional real-client smoke, artifact audit, and troubleshooting. Defer everything else.
**Detection:** Any task requiring new retrieval signals, cloud services, durable storage migration, a GUI, or editing behavior should be moved to v1.2+ unless it directly blocks install/setup verification.

## Moderate Pitfalls

### Pitfall 1: Overwriting User Agent Config

**What goes wrong:** Setup commands mutate global Codex or Claude config unexpectedly.
**Prevention:** Keep generated snippets repo-local or copy/paste-oriented. For smoke tests, use temp homes/configs only. Document exactly which files `ctxpack init` writes.

### Pitfall 2: Confusing "Configured" With "Used"

**What goes wrong:** A docs step passes after `codex mcp list` or `claude mcp get`, but the agent never calls ctxpack.
**Prevention:** Adoption proof must include server-side evidence for `prepare_task` and `get_pack` with explicit `repo`, or a deterministic protocol smoke when real clients are unavailable.

### Pitfall 3: Release Artifacts Omit License, README, or Version Identity

**What goes wrong:** Users install a binary but cannot tell what version it is, where docs live, or what license applies.
**Prevention:** Package `README`, license metadata, and `ctxpack --version`; tag release artifacts with version, commit, target triple, and checksum.

## Phase-Specific Warnings

| v1.1 Topic | Likely Pitfall | Mitigation |
|------------|----------------|------------|
| Install path | Binary not on `PATH` or MCP cannot spawn it | Absolute binary path in snippets; temp-prefix install smoke |
| Agent setup | Wrong repo/cwd | Explicit `repo` in all examples and smokes |
| Client support | Codex/Claude version drift | Versioned compatibility matrix plus deterministic protocol gate |
| Docs | Stale command snippets | Docs grep/checklist in release gate |
| Artifacts | Caches/secrets included | Explicit include list and archive audit |
| CI | Tests skip packaged binary | Run smoke against installed artifact, not workspace target |
| Scope | v1.2/v2 features creep in | Defer retrieval/storage/UI/cloud/team work |

## Release Gate Recommendations

1. Build/package from a clean checkout.
2. Install into a temp prefix and verify `ctxpack --version` and `ctxpack --help`.
3. Run `ctxpack init` in a fixture repo and verify generated guidance uses explicit `repo`.
4. Run deterministic MCP protocol smoke from wrong cwd using the installed binary.
5. Run optional Codex/Claude real-client smokes only when available, with exact versions recorded.
6. Audit archive contents for secrets, caches, traces, request logs, and absolute local paths.
7. Check docs for one current quickstart, troubleshooting, and compatibility matrix.

## Sources

- Local project context: `.planning/PROJECT.md`, `.planning/STATE.md`, `README.md`, `scripts/smoke-mcp-protocol.sh`, `scripts/smoke-codex-mcp.sh`, `scripts/smoke-claude-mcp.sh` - HIGH confidence for current ctxpack v1.1 scope and existing smoke behavior.
- Cargo install docs - HIGH confidence for install root, `PATH`, `--locked`, and `--path` behavior: https://doc.rust-lang.org/cargo/commands/cargo-install.html
- Rust Book `cargo install` chapter - HIGH confidence for `$HOME/.cargo/bin` needing to be on `PATH`: https://doc.rust-lang.org/book/ch14-04-installing-binaries.html
- MCP specification lifecycle - HIGH confidence for protocol version negotiation and capability negotiation: https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
- OpenAI Docs MCP quickstart - MEDIUM confidence for current Codex MCP setup shape and shared CLI/IDE configuration: https://developers.openai.com/learn/docs-mcp
- Claude Code MCP docs - HIGH confidence for MCP scopes, stdio process behavior, environment expansion, output limits, and full-path executable troubleshooting: https://code.claude.com/docs/en/mcp
