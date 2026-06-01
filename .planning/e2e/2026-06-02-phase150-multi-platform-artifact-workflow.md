# Phase 150 - Multi-Platform Artifact Workflow

## Goal

Move ctxhelm distribution readiness beyond a single local Apple Silicon archive
by making the release package script build an explicit Rust target and adding a
non-publishing GitHub Actions workflow for Linux x64, macOS Intel, and macOS
Apple Silicon archives.

## Research

GitHub's hosted-runner reference currently lists:

- `ubuntu-latest` for Linux x64 runners.
- `macos-15-intel` for macOS Intel runners.
- `macos-14` / `macos-15` style labels for macOS arm64 runners.

Phase 150 therefore uses `ubuntu-latest`, `macos-15-intel`, and `macos-14`.
The workflow remains non-publishing: it uploads Actions artifacts only and does
not create tags, releases, Homebrew commits, crates.io packages, signed
installers, or self-update metadata.

## Changes

- Added `CTXHELM_BUILD_TARGET` support to `scripts/release-package.sh`.
- Made release packaging copy from `target/<target>/release/ctxhelm` instead of
  treating `CTXHELM_TARGET_LABEL` as a cosmetic archive name.
- Added `.github/workflows/release-artifacts.yml` with a three-target packaging
  matrix:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
- Each matrix job runs `scripts/release-package.sh`, verifies the generated
  archive with `scripts/verify-release-archive.sh`, and uploads workflow
  artifacts with `actions/upload-artifact@v5`.
- Updated release, distribution, release-governance, and checklist docs so the
  new workflow is a gated distribution capability rather than an unsupported
  publication claim.
- Refreshed the source-free Claude Code workflow integration report at
  `.ctxhelm/e2e/phase150-claude-workflow.json`.

## Verification

- `cargo fmt --check`
- `bash -n scripts/release-package.sh`
- `bash -n scripts/release-candidate-status.sh`
- `bash -n scripts/smoke-release-governance.sh`
- `bash scripts/check-release-docs.sh`
- `bash scripts/smoke-release-governance.sh`
- `cargo test -p ctxhelm --test release_packaging --locked`
- `CTXHELM_ALLOW_DIRTY=1 CTXHELM_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh`
- `cargo test --workspace --locked --no-fail-fast`
- `cargo run -p ctxhelm --locked -- --help`
- `CTXHELM_REQUIRE_REAL_CLIENT=1 CTXHELM_RUN_REAL_CLIENT=1 CTXHELM_CLAUDE_WORKFLOW_REPORT=.ctxhelm/e2e/phase150-claude-workflow.json bash scripts/e2e-claude-workflow.sh`

## Results

- Local host packaging still produces and verifies
  `ctxhelm-v1.1.11-aarch64-apple-darwin.tar.gz`.
- Release docs and release-governance smokes pass with the new workflow and
  distribution-state language.
- Workspace tests pass.
- Claude Code `2.1.159` passed required real-client workflow evidence with
  explicit-repo `prepare_task` and `get_pack` calls.

## Privacy

The new workflow and proof artifacts are source-free. The workflow builds from
the public repository state and archives only `ctxhelm`, `README.md`, `LICENSE`,
and `VERSION`. The Claude report stores hashes, counts, client version, and
sanitized tool-call metadata only; it does not store raw prompt text, task text,
source snippets, or raw MCP traffic.
