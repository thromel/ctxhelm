# Phase 144: Public v1.1.8 Post-Rename Currentness

## Goal

Prove that the public post-rename `ctxhelm` release and package-manager path are current with the renamed repository, installable from public artifacts, and usable by a real agent client without relying on stale `ctxpack` or superseded brand surfaces.

## Public Release

- Repository: `thromel/ctxhelm`
- Tag: `v1.1.8`
- Target commit: `311ada8b04be9037fac0b4cea2882f9b271d9be6`
- Release URL: `https://github.com/thromel/ctxhelm/releases/tag/v1.1.8`
- Archive: `ctxhelm-v1.1.8-aarch64-apple-darwin.tar.gz`
- Archive SHA-256: `e9aca16008e16f352beb0a39c35c3d757cfc56235172a5f8f4872b3a6ab6f6da`

## Proof Artifacts

- Public release freshness: `.ctxhelm/e2e/phase144-public-release-freshness.json`
  - `status = current`
  - `productStatus = current`
  - `commitsAhead = 0`
  - `productCommitsAhead = 0`
- Public archive install: `.ctxhelm/e2e/phase144-public-archive-install.json`
  - downloads public GitHub release assets
  - verifies archive checksum
  - installs to a temporary bin path
  - verifies `ctxhelm 1.1.8`
  - runs help, doctor, and first-pack smoke checks
- Homebrew tap: `.ctxhelm/e2e/phase144-homebrew-tap.json`
  - verifies `thromel/tap/ctxhelm`
  - verifies formula URL and SHA-256
  - installs and tests `ctxhelm 1.1.8`
- Public real-client smoke: `.ctxhelm/e2e/phase144-public-real-client-smoke.json`
  - Claude Code `2.1.159` passed with machine-checkable `prepare_task` and `get_pack` calls
  - Codex CLI `0.44.0` remained optional/skipped because it did not produce machine-checkable tool-call evidence

## Reliability Fix

The strict release gate initially surfaced a resource-sensitive eval flake under low-disk pressure: the historical eval path could skip the lone sampled commit when per-commit Git metadata or diff-tree calls exceeded a 250ms timeout.

The fix keeps Git calls bounded but raises the default history metadata and diff timeouts to one second, while preserving the 250ms minimum for tests that intentionally pass a zero timeout.

## Validation

- `cargo fmt --all -- --check`
- `cargo test -p ctxhelm --test cli_compat eval_baselines_reports_paired_variants_source_free --locked -- --nocapture`
- `cargo test -p ctxhelm --test release_packaging --locked`
- `CTXHELM_PROOF_FIXTURE_ROOT=/Users/romel/Documents/GitHub/ctxpack-proof-fixtures CTXHELM_BIN="$PWD/target/release/ctxhelm" CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1 bash scripts/release-gate.sh`

