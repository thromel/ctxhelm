---
phase: 35
plan: 01
type: implementation
wave: 1
depends_on: []
files_modified:
  - crates/ctxpack-core/src/contracts.rs
  - crates/ctxpack-index/src/lib.rs
  - crates/ctxpack-index/src/workspace.rs
  - crates/ctxpack/src/main.rs
  - crates/ctxpack/tests/cli_compat.rs
  - docs/workspace.md
  - README.md
  - docs/release.md
  - scripts/check-release-docs.sh
  - scripts/release-gate.sh
  - scripts/smoke-workspace.sh
autonomous: true
requirements:
  - WORK-01
  - WORK-02
  - WORK-03
  - WORK-04
---

# Phase 35 Plan: Workspace Manifest & Inventory

<objective>
Add the v2.0 workspace foundation: a local multi-repo workspace manifest plus source-free workspace inventory/status reporting. Keep this phase strictly foundational; cross-repo task routing and packs stay in Phase 36.
</objective>

<context>
There is no Phase 35 `CONTEXT.md`; the user moved directly from discuss to plan. Use conservative defaults:

- Manifest file: `.ctxpack/workspace.json`.
- Repositories are local paths with optional explicit IDs, display labels, and tags.
- Workspace status aggregates source-free metadata only.
- Existing single-repo commands remain unchanged when no workspace manifest is present.
- No hosted sync, source sharing, or MCP tool expansion in this phase.
</context>

<tasks>

<task id="35-01-01" type="execute">
<title>Add workspace public contracts</title>
<files>
- `crates/ctxpack-core/src/contracts.rs`
</files>
<read_first>
- `crates/ctxpack-core/src/contracts.rs`
- Existing source-free contracts near `FeedbackSummary`, `PolicyQualityReport`, and `Storage*`-adjacent report types
</read_first>
<action>
Add stable camelCase serializable contracts for workspace manifests and status reports. Suggested contracts:

- `WorkspaceManifest`
- `WorkspaceRepo`
- `WorkspaceInventoryReport`
- `WorkspaceRepoStatus`
- `WorkspaceRepoDiagnostic`
- `WorkspaceRepoPrivacyStatus` if the existing `PrivacyStatus` shape is not sufficient

Contracts must avoid source-bearing fields. Use path labels, repo IDs, counts, timestamps, compatibility/status enums, diagnostics, and privacy metadata.
</action>
<acceptance_criteria>
- `contracts.rs` contains `WorkspaceManifest`.
- `contracts.rs` contains `WorkspaceInventoryReport`.
- Public JSON field names serialize as camelCase.
- Contract tests assert no fields named `source`, `sourceText`, `prompt`, `terminalLog`, or `transcript`.
- Existing contract tests still pass.
</acceptance_criteria>
</task>

<task id="35-01-02" type="execute">
<title>Add workspace manifest loading and source-free status aggregation</title>
<files>
- `crates/ctxpack-index/src/workspace.rs`
- `crates/ctxpack-index/src/lib.rs`
</files>
<read_first>
- `crates/ctxpack-index/src/lib.rs`
- `crates/ctxpack-index/src/storage.rs`
- `crates/ctxpack-index/src/policy.rs`
- `crates/ctxpack-core/src/repo.rs`
</read_first>
<action>
Create a focused `workspace.rs` module and re-export its public API from `ctxpack-index`. Implement:

- default workspace manifest path resolution: `<repo>/.ctxpack/workspace.json`
- manifest loading from explicit path or default path
- validation for duplicate IDs/labels, missing paths, non-git roots, inaccessible paths, ignored/generated/sensitive labels, and unsafe strings
- source-free per-repo status aggregation using existing inventory and storage status helpers
- no raw source reads beyond existing safe inventory metadata
</action>
<acceptance_criteria>
- `crates/ctxpack-index/src/workspace.rs` exists.
- `ctxpack-index/src/lib.rs` exports workspace APIs.
- Missing repo path yields a diagnostic instead of panicking.
- Duplicate repo ID yields a diagnostic.
- Status report includes per-repo inventory counts and storage compatibility when available.
- Test fixture containing a sentinel source string does not leak that sentinel into workspace manifest/status JSON.
</acceptance_criteria>
</task>

<task id="35-01-03" type="execute">
<title>Add CLI workspace init/status surface</title>
<files>
- `crates/ctxpack/src/main.rs`
- `crates/ctxpack/tests/cli_compat.rs`
</files>
<read_first>
- `crates/ctxpack/src/main.rs`
- `crates/ctxpack/tests/cli_compat.rs`
- Existing `storage`, `memory`, and `eval feedback` command patterns
</read_first>
<action>
Add a small CLI surface:

- `ctxpack workspace init --repo <path> [--member <path>] [--label <label>]`
- `ctxpack workspace status --repo <path> [--manifest <path>] [--format json|markdown]`

Keep output source-free and local-only. The initial `init` may create a minimal `.ctxpack/workspace.json` containing the current repo and any provided members. It must not scan source text beyond existing safe inventory behavior.
</action>
<acceptance_criteria>
- `ctxpack --help` lists `workspace`.
- `ctxpack workspace status --format json` emits `workspaceRoot`, `repos`, diagnostics, and `sourceTextLogged: false` or equivalent source-free privacy signal.
- `ctxpack workspace init` writes `.ctxpack/workspace.json` without modifying source files outside `.ctxpack`.
- Existing CLI compatibility tests still pass.
- New CLI compatibility test covers init/status with two temp repos and a source sentinel.
</acceptance_criteria>
</task>

<task id="35-01-04" type="execute">
<title>Add workspace docs and release smoke</title>
<files>
- `docs/workspace.md`
- `README.md`
- `docs/release.md`
- `scripts/check-release-docs.sh`
- `scripts/release-gate.sh`
- `scripts/smoke-workspace.sh`
</files>
<read_first>
- `docs/feedback.md`
- `docs/storage.md`
- `docs/release.md`
- `scripts/smoke-feedback.sh`
- `scripts/release-gate.sh`
- `scripts/check-release-docs.sh`
- `README.md`
</read_first>
<action>
Document workspace manifests and source-free workspace status. Add a deterministic smoke script that:

1. creates two temp git repos,
2. writes a source sentinel in one repo,
3. initializes a workspace manifest,
4. runs workspace status JSON,
5. verifies the sentinel is absent from workspace storage/output,
6. verifies single-repo commands still work without a workspace manifest.

Wire the smoke into release docs and release gate.
</action>
<acceptance_criteria>
- `docs/workspace.md` explains manifest shape, status, privacy boundaries, and Phase 35 limitations.
- `scripts/smoke-workspace.sh` exists and is executable.
- `scripts/check-release-docs.sh` checks workspace docs/smoke references.
- `scripts/release-gate.sh` runs workspace smoke.
- README links to workspace docs.
</acceptance_criteria>
</task>

</tasks>

<verification>
Run these checks before marking Phase 35 implementation complete:

```bash
cargo fmt --all --check
bash scripts/check-release-docs.sh
CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-workspace.sh
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo run -p ctxpack -- --help
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-core workspace -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack-index workspace -- --test-threads=1 --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test -p ctxpack workspace --test cli_compat -- --nocapture
CARGO_TARGET_DIR=/tmp/ctxpack-target cargo test --workspace
```
</verification>

<success_criteria>
- [ ] WORK-01: local workspace manifest can define multiple repos with IDs, labels, and tags.
- [ ] WORK-02: source-free workspace inventory/status report summarizes per-repo metadata.
- [ ] WORK-03: missing, duplicate, stale, inaccessible, sensitive/generated, and invalid entries produce diagnostics.
- [ ] WORK-04: workspace state remains local and source-free.
- [ ] No existing single-repo behavior changes without an explicit workspace manifest.
- [ ] Release smoke proves no source sentinel leakage.
</success_criteria>

<must_haves>
- Keep workspace manifest/status local-first.
- Preserve repo boundaries; do not merge repo inventories.
- Do not add cross-repo `prepare-task` routing in this phase.
- Do not add hosted sync, cloud upload, or source-bearing shared artifacts.
- Prefer typed contracts over ad hoc JSON values.
</must_haves>
