# Phase 147: Public v1.1.10 Source Distribution Currentness

## Goal

Publish and verify a current public archive and Homebrew tap after Phase 146 made
the crates source-distribution path publish-order ready. The public artifact
should include the explicit internal dependency versions, package-boundary
checks, release-doc contracts, and source-distribution readiness metadata.

## Public Release

- Repository: `thromel/ctxhelm`
- Tag: `v1.1.10`
- Target commit: `375b30f4cd1001f81d8e33f9a58460c922170a83`
- Release URL: `https://github.com/thromel/ctxhelm/releases/tag/v1.1.10`
- Archive: `ctxhelm-v1.1.10-aarch64-apple-darwin.tar.gz`
- Archive SHA-256: `3eea9f0b85bf5973462c6cfff0dc6effe025059464640b954a540d9e739e3e8c`

## Proof Artifacts

- Public release freshness: `.ctxhelm/e2e/phase147-public-release-freshness.json`
  - `status = current`
  - `productStatus = current`
  - `commitsAhead = 0`
  - `productCommitsAhead = 0`
- Public archive install: `.ctxhelm/e2e/phase147-public-archive-install.json`
  - downloads public GitHub release assets
  - verifies archive checksum and archive manifest
  - installs to a temporary bin path
  - verifies `ctxhelm 1.1.10`
  - runs help, doctor, and first-pack smoke checks
- Homebrew tap: `.ctxhelm/e2e/phase147-homebrew-tap.json`
  - verifies `thromel/tap/ctxhelm`
  - verifies formula URL and SHA-256
  - installs and tests `ctxhelm 1.1.10`
- Public real-client smoke: `.ctxhelm/e2e/phase147-public-real-client-smoke.json`
  - Claude Code `2.1.159` passed with machine-checkable `prepare_task` and
    `get_pack` calls
  - Codex CLI `0.44.0` remained optional/skipped because it did not produce
    machine-checkable tool-call evidence

## Validation

- `cargo fmt --all -- --check`
- `bash scripts/check-release-docs.sh`
- `cargo test -p ctxhelm --test release_packaging --locked`
- `cargo test -p ctxhelm --test cli_compat --locked -- --nocapture`
- `bash scripts/release-gate.sh`
- Public CI on `375b30f4cd1001f81d8e33f9a58460c922170a83`
  - workspace validation passed
  - release gate smoke passed
- `bash scripts/verify-github-release.sh --repo thromel/ctxhelm --tag v1.1.10 --target 375b30f4cd1001f81d8e33f9a58460c922170a83 --assets-dir /tmp/ctxhelm-v1110-assets`
- `bash scripts/verify-public-archive-install.sh --repo thromel/ctxhelm --tag v1.1.10 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.10" --output .ctxhelm/e2e/phase147-public-archive-install.json`
- `bash scripts/verify-homebrew-tap.sh --tap thromel/tap --formula ctxhelm --expected-version "ctxhelm 1.1.10" --expected-url https://github.com/thromel/ctxhelm/releases/download/v1.1.10/ctxhelm-v1.1.10-aarch64-apple-darwin.tar.gz --expected-sha256 3eea9f0b85bf5973462c6cfff0dc6effe025059464640b954a540d9e739e3e8c --output .ctxhelm/e2e/phase147-homebrew-tap.json`
- `bash scripts/smoke-public-real-clients.sh --repo thromel/ctxhelm --tag v1.1.10 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.10" --output .ctxhelm/e2e/phase147-public-real-client-smoke.json`

## Notes

- crates.io publication remains deferred. Phase 146 made the source
  distribution path publish-order ready, and this phase made that readiness
  available in the public archive and Homebrew release.
- The Homebrew tap was updated and pushed separately in
  `/Users/romel/Documents/GitHub/homebrew-tap` at commit `0d6a51f`.
