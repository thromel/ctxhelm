---
phase: 08-release-gates-smoke-proof
plan: 03
type: execute
wave: 2
depends_on: [01, 02]
files_modified:
  - docs/release.md
  - scripts/check-release-docs.sh
  - scripts/release-gate.sh
  - crates/ctxpack/tests/release_packaging.rs
autonomous: true
requirements: [SMOKE-01, SMOKE-02, SMOKE-03, SMOKE-04]
must_haves:
  truths:
    - "Maintainer docs name the release gate as the local pre-publication blocker and document required/optional checks."
    - "Docs explain selected-binary usage, wrong-cwd explicit-repo MCP proof, optional Codex/Claude evidence, and no publish/tag behavior."
    - "Docs consistency checks cover the release gate command, environment variables, proof boundaries, and deferred publish channels."
    - "The final gate can be run after Plans 01 and 02 and proves all SMOKE-01 through SMOKE-04 requirements."
  artifacts:
    - path: "docs/release.md"
      provides: "Maintainer release gate instructions and no-publish boundary"
    - path: "scripts/check-release-docs.sh"
      provides: "Docs consistency coverage for release gate and proof boundaries"
    - path: "scripts/release-gate.sh"
      provides: "Final gate wiring to optional real-client evidence wrappers"
    - path: "crates/ctxpack/tests/release_packaging.rs"
      provides: "Release docs/gate contract coverage"
  key_links:
    - from: "docs/release.md"
      to: "scripts/release-gate.sh"
      via: "documented maintainer command"
      pattern: "scripts/release-gate\\.sh"
    - from: "scripts/check-release-docs.sh"
      to: "docs/release.md"
      via: "grep checks for release gate, CTXPACK_BIN, CTXPACK_REQUIRE_REAL_CLIENT, and no publish/tag language"
      pattern: "release-gate|CTXPACK_BIN|CTXPACK_REQUIRE_REAL_CLIENT|publish|tag"
    - from: "scripts/release-gate.sh"
      to: "scripts/smoke-codex-mcp.sh and scripts/smoke-claude-mcp.sh"
      via: "optional real-client evidence hooks"
      pattern: "smoke-codex-mcp|smoke-claude-mcp"
---

<objective>
Document and verify the completed Phase 8 release gate.

Purpose: The release gate should be discoverable, accurately documented, and covered by docs consistency checks so maintainers know exactly what blocks publication and what remains optional.
Output: Release docs/checker updates, final release-gate wiring, and full Phase 8 verification commands.
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
@.planning/phases/08-release-gates-smoke-proof/08-release-gates-smoke-proof-01-SUMMARY.md
@.planning/phases/08-release-gates-smoke-proof/08-release-gates-smoke-proof-02-SUMMARY.md
@.planning/phases/05-release-identity-binary-packaging/VERIFICATION.md
@.planning/phases/06-agent-setup-first-pack-adoption/VERIFICATION.md
@.planning/phases/07-documentation-troubleshooting/VERIFICATION.md
@docs/release.md
@docs/quickstart.md
@docs/agent-setup.md
@docs/troubleshooting.md
@scripts/check-release-docs.sh
@scripts/release-gate.sh
@scripts/smoke-codex-mcp.sh
@scripts/smoke-claude-mcp.sh
@crates/ctxpack/tests/release_packaging.rs

<decision_trace>
- SMOKE-01 through SMOKE-04 are the only Phase 8 requirements in scope.
- Phase 8 context: The final verification should prove all SMOKE requirements without publishing, tagging, uploading, or requiring real-client auth on every machine.
- Phase 7 decision: Docs checks remain narrow and grep-based; do not introduce a Markdown parser.
- Deferred: GitHub Actions release workflow, automatic GitHub release creation, crates.io/Homebrew publication, mandatory real-client smokes on unprovisioned machines, signing/notarization.
</decision_trace>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Document release gate usage and proof boundaries</name>
  <files>docs/release.md</files>
  <action>Update `docs/release.md` with a maintainer "Release gate" section. Document `bash scripts/release-gate.sh` as the local pre-publication blocker, `CTXPACK_BIN=/absolute/path/to/ctxpack bash scripts/release-gate.sh` for selected installed/extracted binaries, default package/extract behavior when no binary is selected, required checks (`cargo test --workspace`, docs checker, release packaging/audit, `--version`, `--help`, first-pack smoke, wrong-cwd MCP protocol smoke), optional checks (`scripts/smoke-codex-mcp.sh`, `scripts/smoke-claude-mcp.sh`), and env vars `CTXPACK_SKIP_REAL_CLIENT`, `CTXPACK_REQUIRE_REAL_CLIENT`, and `CTXPACK_REAL_CLIENT_EVIDENCE_DIR`. State explicitly that this gate does not publish, upload, create tags, mutate global agent config, or run user project tests.</action>
  <verify>
    <automated>python3 -c "from pathlib import Path; d=Path('docs/release.md').read_text(); required=['scripts/release-gate.sh','CTXPACK_BIN','CTXPACK_REQUIRE_REAL_CLIENT','CTXPACK_SKIP_REAL_CLIENT','CTXPACK_REAL_CLIENT_EVIDENCE_DIR','cargo test --workspace','scripts/check-release-docs.sh','scripts/release-package.sh','scripts/smoke-first-pack.sh','scripts/smoke-mcp-protocol.sh','scripts/smoke-codex-mcp.sh','scripts/smoke-claude-mcp.sh','does not publish','does not create tags']; missing=[s for s in required if s not in d]; assert not missing, missing"</automated>
  </verify>
  <done>Maintainer release docs explain the complete Phase 8 gate and proof boundaries.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Expand docs checker for release gate consistency</name>
  <files>scripts/check-release-docs.sh, crates/ctxpack/tests/release_packaging.rs</files>
  <behavior>
    - Test 1: Release docs checker requires `docs/release.md` to name `scripts/release-gate.sh` and all release gate component scripts.
    - Test 2: Release docs checker requires `CTXPACK_BIN`, `CTXPACK_REQUIRE_REAL_CLIENT`, `CTXPACK_SKIP_REAL_CLIENT`, and `CTXPACK_REAL_CLIENT_EVIDENCE_DIR` documentation.
    - Test 3: Release docs checker rejects claims that the gate publishes, tags, uploads, creates GitHub releases, or requires Cursor/OpenCode real-client proof.
    - Test 4: Rust release docs contract expects the new release-gate checks and still passes without parsing Markdown.
  </behavior>
  <action>Extend `scripts/check-release-docs.sh` with grep-style release gate checks and forbidden-claim checks. Update `crates/ctxpack/tests/release_packaging.rs` release docs contract to require the checker to cover the release gate and proof-boundary strings. Keep the checker small and deterministic; do not add a Markdown parser or network/client probing.</action>
  <verify>
    <automated>bash -n scripts/check-release-docs.sh && bash scripts/check-release-docs.sh && cargo test -p ctxpack --test release_packaging release_docs -- --nocapture</automated>
  </verify>
  <done>Docs consistency checks guard the Phase 8 release gate documentation and proof boundaries.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Wire optional real-client hooks into final gate and run full verification</name>
  <files>scripts/release-gate.sh, crates/ctxpack/tests/release_packaging.rs</files>
  <behavior>
    - Test 1: `scripts/release-gate.sh` invokes `scripts/smoke-codex-mcp.sh` and `scripts/smoke-claude-mcp.sh` after deterministic first-pack and MCP protocol gates.
    - Test 2: The gate passes through `CTXPACK_BIN`, `CTXPACK_SKIP_REAL_CLIENT`, `CTXPACK_REQUIRE_REAL_CLIENT`, and `CTXPACK_REAL_CLIENT_EVIDENCE_DIR` to optional real-client wrappers.
    - Test 3: With `CTXPACK_SKIP_REAL_CLIENT=1`, the final gate can complete without real-client auth after deterministic proof passes.
    - Test 4: With `CTXPACK_REQUIRE_REAL_CLIENT=1`, missing Codex/Claude evidence fails through the wrappers.
  </behavior>
  <action>Update `scripts/release-gate.sh` if Plan 01 did not already wire optional real-client hooks after deterministic release checks. Keep default behavior portable: real clients are attempted/skipped according to wrapper semantics, and required only when `CTXPACK_REQUIRE_REAL_CLIENT=1`. Update release gate contract tests accordingly. Run the full release gate with a selected local binary and `CTXPACK_SKIP_REAL_CLIENT=1` to prove the deterministic Phase 8 gate without requiring auth.</action>
  <verify>
    <automated>cargo test -p ctxpack --test release_packaging release_gate -- --nocapture && cargo build -p ctxpack && CTXPACK_BIN="$(pwd)/target/debug/ctxpack" CTXPACK_SKIP_REAL_CLIENT=1 bash scripts/release-gate.sh</automated>
  </verify>
  <done>The final release gate ties together installed-binary proof, docs consistency, packaging/audit, first-pack/MCP protocol proof, and optional versioned real-client evidence hooks.</done>
</task>

</tasks>

<verification>
- `bash scripts/check-release-docs.sh`
- `cargo test -p ctxpack --test release_packaging release_docs release_gate -- --nocapture`
- `cargo test -p ctxpack --test cli_compat real_client -- --nocapture`
- `cargo build -p ctxpack && CTXPACK_BIN="$(pwd)/target/debug/ctxpack" CTXPACK_SKIP_REAL_CLIENT=1 bash scripts/release-gate.sh`
- `cargo test --workspace`
- `cargo run -p ctxpack -- --help`
</verification>

<success_criteria>
Plan 03 is complete when release docs, docs checks, and the final release gate demonstrate all SMOKE-01 through SMOKE-04 requirements without publishing or tagging a release.
</success_criteria>

<output>
After completion, create `.planning/phases/08-release-gates-smoke-proof/08-release-gates-smoke-proof-03-SUMMARY.md`
</output>
