---
phase: 05-release-identity-binary-packaging
plan: 03
type: execute
wave: 3
depends_on: [02]
files_modified:
  - scripts/audit-release-artifact.sh
  - scripts/release-package.sh
  - crates/ctxpack/tests/release_packaging.rs
autonomous: true
requirements: [PKG-05]
must_haves:
  truths:
    - "Maintainer can audit release archives and fail fast on ctxpack local state, traces, request logs, temp homes, secrets, absolute local paths, and caches."
    - "Release packaging invokes the artifact audit before reporting success."
    - "Audit checks are source-free and inspect archive file names plus text payloads that are intentionally included in the release artifact."
  artifacts:
    - path: "scripts/audit-release-artifact.sh"
      provides: "Archive-content and text-payload release audit"
    - path: "scripts/release-package.sh"
      provides: "Packaging integration that calls the audit before success"
    - path: "crates/ctxpack/tests/release_packaging.rs"
      provides: "Contract tests for audit patterns and release-package integration"
  key_links:
    - from: "scripts/release-package.sh"
      to: "scripts/audit-release-artifact.sh"
      via: "post-archive audit invocation"
      pattern: "audit-release-artifact"
    - from: "scripts/audit-release-artifact.sh"
      to: "dist/*.tar.gz"
      via: "tar listing and extracted payload inspection"
      pattern: "tar.*-tf"
---

<objective>
Add the release artifact privacy and hygiene audit.

Purpose: A local-first context broker loses trust if its first release archive contains local state, traces, logs, secrets, or machine-specific paths.
Output: An audit script integrated into packaging, with tests that guard the forbidden artifact patterns.
</objective>

<execution_context>
@/Users/romel/.codex/get-shit-done/workflows/execute-plan.md
@/Users/romel/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/research/PITFALLS.md
@.planning/phases/05-release-identity-binary-packaging/05-CONTEXT.md
@.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-02-SUMMARY.md
@scripts/release-package.sh

<decision_trace>
- D-05: Audit archive contents for `.ctxpack`, traces, request logs, temp homes, secrets, absolute local paths, target debris, and unintended caches.
- D-06: Keep stdout cleanliness for `ctxpack serve-mcp`; packaging scripts must not introduce MCP stdout logging.
- D-08: Do not add telemetry, cloud indexing, cloud embeddings, update checks, or hosted release dependencies.
</decision_trace>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add release artifact audit contract tests</name>
  <files>crates/ctxpack/tests/release_packaging.rs, scripts/audit-release-artifact.sh</files>
  <behavior>
    - Test 1: `scripts/audit-release-artifact.sh` exists, passes `bash -n`, and documents/implements forbidden patterns for `.ctxpack`, `traces.jsonl`, request logs, temp homes, `.env`, key/token-looking paths, `target/`, `.git/`, and absolute `/Users/` paths.
    - Test 2: A synthetic archive containing `.ctxpack/repos/x/traces.jsonl` fails the audit.
    - Test 3: A synthetic archive containing only `ctxpack`, `README.md`, `LICENSE`, and `VERSION` passes the audit.
  </behavior>
  <action>Extend `crates/ctxpack/tests/release_packaging.rs` with audit-script contract tests and synthetic archive tests. Create `scripts/audit-release-artifact.sh` as the executable under test. Keep the tests local and deterministic; use temporary directories and `tar`, not real release artifacts or network downloads.</action>
  <verify>
    <automated>cargo test -p ctxpack --test release_packaging release_artifact_audit -- --nocapture</automated>
  </verify>
  <done>Artifact audit behavior is test-covered before integration with the release package script.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement and integrate artifact audit</name>
  <files>scripts/audit-release-artifact.sh, scripts/release-package.sh, crates/ctxpack/tests/release_packaging.rs</files>
  <behavior>
    - Test 1: `scripts/audit-release-artifact.sh <archive>` lists archive contents and exits non-zero for forbidden path patterns.
    - Test 2: The audit extracts text payloads to a temp directory and exits non-zero for absolute local paths like `/Users/romel`, temp-home markers, `.ctxpack`, `traces.jsonl`, request logs, `.env`, private key names, or token-looking filenames.
    - Test 3: `scripts/release-package.sh` calls the audit for each generated archive before writing a success message.
    - Test 4: A full packaging run with `CTXPACK_ALLOW_DIRTY=1` succeeds only after audit passes.
  </behavior>
  <action>Implement `scripts/audit-release-artifact.sh` per PKG-05. It must accept one or more archive paths, support `.tar.gz` at minimum, use `tar -tf` to inspect members, extract into a temp directory for text scans, and fail on forbidden patterns without printing archive source contents. Integrate it into `scripts/release-package.sh` immediately after archive creation and before checksum success output. Do not add cloud scanning, secret-upload services, or broad source tree linting outside the archive.</action>
  <verify>
    <automated>cargo test -p ctxpack --test release_packaging release_artifact_audit -- --nocapture && CTXPACK_ALLOW_DIRTY=1 CTXPACK_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh</automated>
  </verify>
  <done>Every locally generated release archive is audited for local-state and secret/path leakage before packaging succeeds.</done>
</task>

</tasks>

<verification>
- `cargo test -p ctxpack --test release_packaging release_artifact_audit -- --nocapture`
- `CTXPACK_ALLOW_DIRTY=1 CTXPACK_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh`
- `cargo test --workspace`
</verification>

<success_criteria>
Plan 03 is complete when the release archive path has a deterministic, local-only artifact audit that blocks privacy and machine-specific leakage.
</success_criteria>

<output>
After completion, create `.planning/phases/05-release-identity-binary-packaging/05-release-identity-binary-packaging-03-SUMMARY.md`
</output>
