# Phase 8: Release Gates & Smoke Proof - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning
**Mode:** Autonomous smart-discuss defaults

<domain>
## Phase Boundary

This phase creates the final v1.1 release gate that blocks publication unless installed-binary behavior, generated setup artifacts, docs consistency, release artifact audit, deterministic MCP protocol smoke, and optional real-client evidence expectations pass. It consolidates existing Phase 5-7 scripts and tests into a maintainer-facing gate. It does not publish a release, create package-manager taps, add cloud services, or require real Codex/Claude auth on every machine.

</domain>

<decisions>
## Implementation Decisions

### Release Gate Shape
- Add one maintainer-facing release gate command/script that orchestrates existing checks.
- The gate must smoke an installed/extracted or selected `ctxpack` binary, not only `cargo run`.
- It should check help/version, release packaging, artifact audit, init/setup artifacts, first-pack smoke, docs consistency, and deterministic MCP protocol behavior.
- It should remain local and deterministic by default.

### MCP And Client Proof
- Deterministic MCP protocol smoke is the hard required proof.
- The protocol smoke must support `CTXPACK_BIN` or equivalent selected-binary execution from the wrong cwd with explicit `repo`.
- Codex CLI and Claude Code real-client smokes remain optional unless `CTXPACK_REQUIRE_REAL_CLIENT=1` is set.
- When real-client smokes run, they must record machine-checkable `prepare_task` and `get_pack` evidence and exact client versions.

### Release Readiness Output
- The release gate should produce clear pass/fail output that tells maintainers what failed and why.
- It should not upload artifacts, mutate global agent configs, run user project tests, or create tags.
- It should document any required environment variables for optional client checks.
- The final verification should prove all SMOKE-01 through SMOKE-04 requirements.

### the agent's Discretion
- The planner may choose whether the top-level gate is a new `scripts/release-gate.sh` or an extension of existing release scripts, provided it remains composable and testable.
- The implementation may update existing smoke scripts if needed to centralize selected-binary handling.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scripts/release-package.sh` builds archives/checksums and smokes extracted binaries.
- `scripts/audit-release-artifact.sh` audits archives for local state, secrets, paths, and caches.
- `scripts/check-release-docs.sh` checks release/docs consistency.
- `scripts/smoke-mcp-protocol.sh` provides deterministic MCP proof and now supports selected binary execution.
- `scripts/smoke-first-pack.sh` validates install-to-first-pack behavior.
- `scripts/smoke-codex-mcp.sh` and `scripts/smoke-claude-mcp.sh` provide optional real-client wrappers.
- `crates/ctxpack/tests/release_packaging.rs` and `crates/ctxpack/tests/cli_compat.rs` guard script and protocol behavior.

### Established Patterns
- Prefer deterministic shell scripts with `set -euo pipefail`.
- Keep optional real-client checks skippable by default and required only via explicit env.
- Use temp directories and isolated `CTXPACK_HOME` for smokes.
- Do not let release gates rely on developer-local `.ctxpack`, `.codex`, `.claude`, or source checkout side effects.

### Integration Points
- Release gate tests can inspect script contracts and run lightweight checks.
- Full phase verification can run the actual release gate.
- ROADMAP/REQUIREMENTS should mark SMOKE-01 through SMOKE-04 complete when verified.

</code_context>

<specifics>
## Specific Ideas

- Add `scripts/release-gate.sh`.
- Add contract tests that it invokes `cargo test --workspace`, `check-release-docs.sh`, `release-package.sh`, `smoke-first-pack.sh`, and deterministic MCP smoke with a selected binary.
- Include optional real-client smoke hooks for Codex and Claude controlled by `CTXPACK_SKIP_REAL_CLIENT` / `CTXPACK_REQUIRE_REAL_CLIENT`.
- Ensure the gate avoids publishing, tagging, or uploading.

</specifics>

<deferred>
## Deferred Ideas

- GitHub Actions release workflow.
- Automatic GitHub release creation.
- crates.io/Homebrew publication.
- Mandatory real-client smokes on unprovisioned machines.
- Code signing, notarization, or hosted release dashboards.

</deferred>
