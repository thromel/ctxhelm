# Phase 151: Public v1.1.12 Multi-Platform Currentness

## Scope

Prepare the public `v1.1.12` release identity and close the gap between the
proven multi-platform artifact workflow and public release assets.

## Changes

- Bumped workspace release identity from `1.1.11` to `1.1.12`.
- Updated release, quickstart, troubleshooting, distribution, governance, and
  package-boundary docs for the new release line.
- Changed `.github/workflows/release-artifacts.yml` so manual dispatch remains a
  workflow-artifact smoke, while version-tag pushes create or update the GitHub
  release and upload verified assets for:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
- Updated release-candidate metadata so published additional platform assets are
  `ready` only when the candidate is ready.
- Refreshed Claude Code workflow evidence at
  `.ctxhelm/e2e/phase151-claude-workflow.json`.

## Local Validation

- `cargo fmt --check`
- `bash scripts/check-release-docs.sh`
- `cargo test -p ctxhelm --test release_packaging --locked`
- `cargo test -p ctxhelm --test cli_compat --locked`
- `bash scripts/smoke-release-governance.sh`
- `CTXHELM_ALLOW_DIRTY=1 CTXHELM_DIST_DIR="$(mktemp -d)" bash scripts/release-package.sh`
- `cargo run -p ctxhelm --locked -- --help`
- `cargo test --workspace --locked --no-fail-fast`
- `CTXHELM_ALLOW_DIRTY=1 CTXHELM_SKIP_CLEAN_FIXTURE_PROOF=1 CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/release-gate.sh`
- `CTXHELM_REQUIRE_REAL_CLIENT=1 CTXHELM_RUN_REAL_CLIENT=1 CTXHELM_CLAUDE_WORKFLOW_REPORT=.ctxhelm/e2e/phase151-claude-workflow.json bash scripts/e2e-claude-workflow.sh`

## Claude Code Evidence

The refreshed source-free report records:

- `status = passed`
- `ctxhelmVersion = ctxhelm 1.1.12`
- `clientVersion = 2.1.159 (Claude Code)`
- `explicitRepoToolCallCount = 2`
- observed tool calls: `prepare_task`, `get_pack`
- `rawPromptStored = false`
- `rawMcpTrafficStored = false`
- `sourceTextLogged = false`
- `userProjectCommandsRun = false`

## Pending Public Evidence

After this change is pushed:

1. CI must pass on `main`.
2. Tag `v1.1.12` must run the release-artifact workflow.
3. The tag workflow must publish all three target archives, manifests, audit
   reports, and per-archive checksums to the GitHub release.
4. `scripts/verify-github-release.sh` must verify release asset digests.
5. `scripts/check-public-release-freshness.sh --tag v1.1.12 --require-product-current`
   must report current.

## Decision

Local release readiness is complete. Public currentness is intentionally pending
until the pushed commit and tag workflow finish.
