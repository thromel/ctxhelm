---
phase: 08-release-gates-smoke-proof
status: passed
verified: 2026-05-13T19:35:36Z
requirements: [SMOKE-01, SMOKE-02, SMOKE-03, SMOKE-04]
---

# Phase 08 Verification: Release Gates & Smoke Proof

## Status

Passed. All three Phase 8 plans completed with committed summaries and final release-gate verification. No releases were published, no tags were created, and no artifacts were uploaded.

## Verified Outcomes

- `scripts/release-gate.sh` runs one local pre-publication gate covering workspace tests, docs consistency, packaging/audit, binary identity, first-pack smoke, wrong-cwd MCP protocol proof, and optional real-client wrappers.
- The gate proves a selected `CTXHELM_BIN` when provided and otherwise extracts the packaged artifact for installed-binary proof.
- `scripts/smoke-codex-mcp.sh` and `scripts/smoke-claude-mcp.sh` use the selected binary for protocol proof and `serve-mcp`, and emit source-free versioned evidence when real-client checks are explicitly enabled.
- `docs/release.md` documents required deterministic gates, optional Codex/Claude evidence, selected-binary usage, and no-publish/no-tag/no-global-config boundaries.
- `scripts/check-release-docs.sh` enforces release-gate docs coverage and rejects unsupported publish/tag/upload and Cursor/OpenCode real-client proof claims.

## Final Verification Commands

- `bash scripts/check-release-docs.sh` - passed
- `cargo test -p ctxhelm --test release_packaging release_ -- --nocapture` - passed
- `cargo test -p ctxhelm --test cli_compat real_client -- --nocapture` - passed
- `cargo test --workspace` - passed
- `cargo run -p ctxhelm -- --help` - passed
- `cargo build -p ctxhelm && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/release-gate.sh` - passed

## Plan Summaries

- `08-release-gates-smoke-proof-01-SUMMARY.md` - core local release gate
- `08-release-gates-smoke-proof-02-SUMMARY.md` - optional real-client evidence wrappers
- `08-release-gates-smoke-proof-03-SUMMARY.md` - release docs and docs checker

## Blockers

None.

## Notes

Real Codex/Claude client checks remain opt-in. The verified final gate used `CTXHELM_SKIP_REAL_CLIENT=1`, so it proved deterministic protocol behavior without requiring local auth.

## Audit Follow-up

The milestone integration audit found and fixed two release-gate hardening gaps after the original phase verification:

- `scripts/release-gate.sh` now preserves `scripts/release-package.sh` clean-checkout enforcement by default instead of defaulting `CTXHELM_ALLOW_DIRTY=1`.
- `scripts/smoke-mcp-protocol.sh` now proves `current_diff` against an isolated temporary git repo instead of temporarily writing a smoke file into the target repository.

Additional verification after these fixes:

- `cargo test -p ctxhelm --test release_packaging -- --nocapture` - passed
- `cargo test -p ctxhelm --test cli_compat -- --nocapture` - passed
- `bash scripts/check-release-docs.sh` - passed
- `cargo build -p ctxhelm` - passed
- `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" bash scripts/smoke-first-pack.sh` - passed
- `CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_ROOT="$(pwd)" CTXHELM_SMOKE_REPO="$(pwd)" CTXHELM_SMOKE_TASK="verify isolated current diff smoke" CTXHELM_SMOKE_PATH="crates/ctxhelm-mcp/src/lib.rs" CTXHELM_SMOKE_QUERY="prepare_task" bash scripts/smoke-mcp-protocol.sh` - passed and left no `ctxhelm_smoke_current_diff` file in the target repo
- `CTXHELM_ALLOW_DIRTY=1 CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/release-gate.sh` - passed for in-flight validation
