# Phase 5: Release Identity & Binary Packaging - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Mode:** Autonomous smart-discuss defaults

<domain>
## Phase Boundary

This phase makes ctxpack installable and identifiable as a versioned binary product. It covers release metadata, binary version output, repeatable archive/checksum generation, source-build fallback documentation, and artifact audits. It does not publish the release, mutate user agent configuration, add package-manager ecosystems, change retrieval behavior, or add cloud/runtime services.

</domain>

<decisions>
## Implementation Decisions

### Release Channel
- Use GitHub Releases-style archives and SHA-256 checksums as the first public binary artifact shape.
- Defer crates.io, Homebrew, self-update, signed installers, and package-manager automation until after one reliable binary archive path is proven.
- Keep `cargo install --git ... --tag ... --locked` and local checkout installation as documented fallback paths.
- Treat any release automation as repeatable local scripts/workflows, not a hosted service dependency.

### Version Identity
- Add or verify `ctxpack --version` so support/debug starts with a simple installed-binary diagnostic.
- Keep workspace crate versions, release tag references, docs, README, and binary output consistent for v1.1.
- Add release metadata required for a credible public binary distribution, including license/readme/repository/package description where missing.
- Prefer additive metadata and CLI behavior; avoid changing existing plan/pack/MCP JSON contracts.

### Artifact Shape And Audit
- Build from a clean checkout into explicit release artifacts, then smoke the extracted or installed binary rather than only `cargo run`.
- Publish or generate checksums alongside archives and document verification.
- Audit archive contents for `.ctxpack`, traces, request logs, temp homes, secrets, absolute local paths, target debris, and unintended caches.
- Keep stdout cleanliness for `ctxpack serve-mcp`; packaging scripts must not introduce stdout logging into the MCP server.

### the agent's Discretion
- The planner may choose the exact script names and release metadata layout if they stay boring, testable, and consistent with existing repository style.
- The implementation may use shell scripts and standard Cargo tooling before adding dedicated release tooling, as long as repeatability and artifact audit criteria are met.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ctxpack/src/main.rs` owns the Clap CLI and already exposes the user-facing command tree.
- `Cargo.toml` and per-crate `Cargo.toml` files currently define workspace/package metadata and versions.
- `scripts/smoke-mcp-protocol.sh`, `scripts/smoke-codex-mcp.sh`, `scripts/smoke-claude-mcp.sh`, and `scripts/smoke-historical-eval.sh` provide existing shell smoke patterns.
- `README.md` already documents developer-oriented `cargo run` flows that can be updated or complemented with installed-binary release guidance.

### Established Patterns
- Keep ctxpack local-first, read-only, and agent-native.
- Prefer stable typed contracts and additive fields over breaking output changes.
- Prefer deterministic protocol-level smoke gates over real-client-only proof.
- Use scripts plus tests for behavior that crosses CLI, MCP, or release boundaries.

### Integration Points
- CLI version/help output belongs in `crates/ctxpack/src/main.rs` and package metadata.
- Release metadata belongs in root and crate `Cargo.toml` files plus repository docs.
- Packaging and audit gates belong under `scripts/` unless a workflow file is needed later.
- Documentation should keep normal-user install commands distinct from developer `cargo run` commands.

</code_context>

<specifics>
## Specific Ideas

- Start with a `scripts/release-package.sh` or equivalent that builds a release binary, creates a tarball/zip-style archive for the current platform, writes SHA-256 checksums, and audits contents.
- Add a release audit script or release-package audit step that fails on local-state and secret-looking paths.
- Add tests around `ctxpack --version` and package metadata where practical.
- Keep generated artifacts under a predictable ignored output directory such as `dist/`.

</specifics>

<deferred>
## Deferred Ideas

- crates.io publishing.
- Homebrew tap.
- Signed/notarized installers.
- Automatic update checks.
- Cloud telemetry or hosted release service.
- Retrieval, storage, parser, UI, or team features.

</deferred>
