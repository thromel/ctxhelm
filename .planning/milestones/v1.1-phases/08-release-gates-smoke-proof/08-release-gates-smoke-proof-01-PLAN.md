---
phase: 08-release-gates-smoke-proof
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - scripts/release-gate.sh
  - crates/ctxhelm/tests/release_packaging.rs
autonomous: true
requirements: [SMOKE-01, SMOKE-02, SMOKE-04]
must_haves:
  truths:
    - "Maintainer can run one local release gate that proves a selected, installed, or extracted ctxhelm binary, not only `cargo run`."
    - "The gate runs deterministic MCP protocol proof from a wrong cwd using explicit repo and selected `CTXHELM_BIN` behavior."
    - "The gate blocks release readiness on CLI help/version, packaging/audit, setup/first-pack smoke, MCP protocol smoke, docs consistency, and workspace tests."
    - "The gate never publishes, uploads, creates git tags, runs `cargo publish`, or mutates global agent configuration."
  artifacts:
    - path: "scripts/release-gate.sh"
      provides: "Maintainer-facing local release gate orchestrating packaging, installed-binary smoke, docs, first-pack, and MCP protocol checks"
    - path: "crates/ctxhelm/tests/release_packaging.rs"
      provides: "Contract tests for release gate script behavior and no-publish boundary"
  key_links:
    - from: "scripts/release-gate.sh"
      to: "scripts/release-package.sh"
      via: "build/extract release artifact path when CTXHELM_BIN is not provided"
      pattern: "release-package\\.sh"
    - from: "scripts/release-gate.sh"
      to: "scripts/smoke-first-pack.sh"
      via: "selected-binary first-pack smoke"
      pattern: "CTXHELM_BIN=.*smoke-first-pack"
    - from: "scripts/release-gate.sh"
      to: "scripts/smoke-mcp-protocol.sh"
      via: "selected-binary wrong-cwd MCP protocol proof"
      pattern: "CTXHELM_BIN=.*smoke-mcp-protocol"
    - from: "scripts/release-gate.sh"
      to: "scripts/check-release-docs.sh"
      via: "docs command consistency gate"
      pattern: "check-release-docs\\.sh"
---

<objective>
Create the core Phase 8 release gate script.

Purpose: Phase 8 should give maintainers a single local command that blocks publication unless the installed/extracted binary path, deterministic MCP proof, first-pack setup, release artifact audit, docs consistency, and workspace tests pass.
Output: `scripts/release-gate.sh` plus script-contract tests.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/research/SUMMARY.md
@.planning/research/PITFALLS.md
@.planning/phases/08-release-gates-smoke-proof/08-CONTEXT.md
@.planning/phases/05-release-identity-binary-packaging/VERIFICATION.md
@.planning/phases/06-agent-setup-first-pack-adoption/VERIFICATION.md
@.planning/phases/07-documentation-troubleshooting/VERIFICATION.md
@scripts/release-package.sh
@scripts/audit-release-artifact.sh
@scripts/check-release-docs.sh
@scripts/smoke-first-pack.sh
@scripts/smoke-mcp-protocol.sh
@crates/ctxhelm/tests/release_packaging.rs

<decision_trace>
- SMOKE-01: Release gate must smoke an installed/extracted or selected `ctxhelm` binary, not only `cargo run`.
- SMOKE-02: Deterministic MCP protocol smoke must run from a wrong cwd with explicit `repo` and selected `CTXHELM_BIN`.
- SMOKE-04: Gate must check help/version, repo-local init/setup artifacts, MCP protocol behavior, artifact audit, and docs command consistency.
- Phase 8 context: Add one maintainer-facing release gate command/script, local and deterministic by default.
- Deferred: Do not publish, tag, upload, add GitHub Actions release automation, require crates.io/Homebrew/signing, or make real-client smokes mandatory on unprovisioned machines.
</decision_trace>

<interfaces>
Existing script contracts the executor must preserve:

```text
scripts/release-package.sh:
- builds `cargo build -p ctxhelm --release --locked`
- writes versioned tar.gz archives and SHA-256 files under `dist/` or `CTXHELM_DIST_DIR`
- runs `scripts/audit-release-artifact.sh`
- smokes the extracted binary with `--version` and `--help`
```

```text
scripts/smoke-first-pack.sh:
- honors `CTXHELM_BIN`
- runs init, setup-check, deterministic MCP protocol smoke, prepare-task, and get-pack on a temp repo
- validates machine-readable JSON without real agent auth
```

```text
scripts/smoke-mcp-protocol.sh:
- honors `CTXHELM_BIN`
- launches `serve-mcp` from a wrong cwd
- sends explicit `repo` in repo-accepting MCP calls
- validates prepare_task, get_pack, search, related, related_tests, current_diff, and same-session pack resource reads
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add release gate script contract tests</name>
  <files>scripts/release-gate.sh, crates/ctxhelm/tests/release_packaging.rs</files>
  <behavior>
    - Test 1: `scripts/release-gate.sh` exists and passes `bash -n`.
    - Test 2: The script invokes `cargo test --workspace`, `scripts/check-release-docs.sh`, `scripts/release-package.sh`, `scripts/smoke-first-pack.sh`, and `scripts/smoke-mcp-protocol.sh`.
    - Test 3: The script supports `CTXHELM_BIN` as a selected executable and passes that same binary path into the first-pack and MCP protocol smokes.
    - Test 4: If `CTXHELM_BIN` is unset, the script obtains an extracted binary from a release package artifact instead of using `cargo run` as the release proof.
    - Test 5: The script contains no `git tag`, `git push`, `gh release`, upload, `cargo publish`, crates.io, Homebrew, signing, or notarization behavior.
  </behavior>
  <action>Extend `crates/ctxhelm/tests/release_packaging.rs` with focused release-gate contract tests. Add a minimal executable `scripts/release-gate.sh` placeholder only if needed for the RED test, then implement enough structure for the contract tests to fail for missing behavior before the GREEN step. Keep tests deterministic and file-content based; do not run real release packaging inside the contract test.</action>
  <verify>
    <automated>cargo test -p ctxhelm --test release_packaging release_gate -- --nocapture</automated>
  </verify>
  <done>Release gate script expectations are captured before implementing the full gate.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement installed-binary release gate orchestration</name>
  <files>scripts/release-gate.sh, crates/ctxhelm/tests/release_packaging.rs</files>
  <behavior>
    - Test 1: With `CTXHELM_BIN=/absolute/path/to/ctxhelm`, the gate verifies executable status, canonicalizes the path, runs `--version` and `--help`, and uses that binary for all binary-facing smoke scripts.
    - Test 2: Without `CTXHELM_BIN`, the gate runs `scripts/release-package.sh` into a temp or configured `CTXHELM_DIST_DIR`, extracts the generated archive to a temp install directory, and uses the extracted `ctxhelm` binary for smoke proof.
    - Test 3: The gate runs `cargo test --workspace`, `bash scripts/check-release-docs.sh`, `scripts/release-package.sh`, `scripts/smoke-first-pack.sh`, and `scripts/smoke-mcp-protocol.sh` in fail-fast order with clear pass/fail step names.
    - Test 4: The gate honors existing `CTXHELM_REQUIRE_REAL_CLIENT` / `CTXHELM_SKIP_REAL_CLIENT` semantics without making real clients mandatory by default.
    - Test 5: The gate produces a concise final success line naming the binary path and does not create tags, uploads, releases, or global agent config changes.
  </behavior>
  <action>Implement `scripts/release-gate.sh` with `set -euo pipefail`, explicit step logging, temp directory cleanup, selected-binary resolution, and no publish/tag/upload behavior. Use existing scripts rather than duplicating their logic. When packaging is needed, rely on `scripts/release-package.sh` so artifact audit remains the single source of truth. After resolving the binary, export `CTXHELM_BIN` into `scripts/smoke-first-pack.sh`, `scripts/smoke-mcp-protocol.sh`, and the optional Codex/Claude smoke wrappers if invoked. Do not run user project tests; workspace tests here are ctxhelm's own release guard.</action>
  <verify>
    <automated>bash -n scripts/release-gate.sh && cargo build -p ctxhelm && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" bash scripts/release-gate.sh</automated>
  </verify>
  <done>Maintainer can run one local gate proving release readiness against a selected or extracted ctxhelm binary without publishing anything.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxhelm --test release_packaging release_gate -- --nocapture`
- `bash -n scripts/release-gate.sh`
- `cargo build -p ctxhelm && CTXHELM_BIN="$(pwd)/target/debug/ctxhelm" bash scripts/release-gate.sh`
- `cargo test --workspace`
- `cargo run -p ctxhelm -- --help`
</verification>

<success_criteria>
Plan 01 is complete when SMOKE-01, SMOKE-02, and the required deterministic portions of SMOKE-04 are represented by a single local gate that uses selected/extracted binaries, existing package/audit/docs/smoke scripts, and no publish/tag behavior.
</success_criteria>

<output>
After completion, create `.planning/phases/08-release-gates-smoke-proof/08-release-gates-smoke-proof-01-SUMMARY.md`
</output>
