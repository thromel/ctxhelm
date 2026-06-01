# Phase 120: Public CI Release Gate

## Goal

Make the public repository enforce the local production checks continuously
instead of relying only on manual release-gate runs.

## Gap

The project had a strong local release gate, public archive proof, and
source-free release artifacts, but no `.github/workflows` automation. That left
pushes to `main` and external pull requests without a machine-checkable default
gate for formatting, clippy, workspace tests, CLI help, release docs, packaging,
archive audit, MCP protocol smoke, and release-gate smokes.

## Changes

- Added `.github/workflows/ci.yml` with two jobs:
  - `workspace`: checkout, Rust install, cargo cache, format, clippy with
    `-D warnings`, locked workspace tests, CLI help, and release-doc checks.
  - `release-gate`: checkout, Rust install, cargo cache, and
    `scripts/release-gate.sh` with clean-fixture and real-client proofs
    explicitly skipped.
- Added a release-packaging contract test that verifies the workflow exists,
  runs the required commands, and contains no publish/tag/push behavior.
- Updated release docs and release-doc consistency checks to mention the CI
  workflow and clippy gate.
- Cleared existing clippy warnings across the workspace so the new CI gate is
  actionable instead of immediately red.
- Fixed the first public CI run's workspace-job install failure by passing
  separate `rustup --component` flags for `rustfmt` and `clippy`.
- Fixed the second public CI run's newer-stable clippy failures by removing
  redundant `.into_iter()` calls in MCP current-diff anchor chaining.
- Fixed the third public CI run's newer-stable clippy failures by applying the
  same redundant `.into_iter()` cleanup to CLI current-diff anchor chaining.

## Validation

Passed:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p ctxhelm --test release_packaging -- --nocapture
bash scripts/check-release-docs.sh
cargo test --workspace --locked --no-fail-fast
cargo run -p ctxhelm --locked -- --help
CTXHELM_ALLOW_DIRTY=1 CTXHELM_SKIP_CLEAN_FIXTURE_PROOF=1 CTXHELM_SKIP_REAL_CLIENT=1 bash scripts/release-gate.sh
```

The release-gate smoke passed through archive packaging/audit, clean extraction,
installed-binary smokes, deterministic MCP protocol checks, Cursor/OpenCode
setup protocol proof, and skipped optional benchmark/clean-fixture/real-client
proofs with explicit source-free skip evidence.

## Boundary

The CI workflow does not publish releases, create tags, push commits, run real
agent clients, require detached proof fixtures, or upload source artifacts.
Full release-candidate proof with clean fixtures and optional real-client proof
remains a maintainer-run local gate.
