# Phase 145: Public v1.1.9 Hardening Currentness

## Goal

Publish and verify a current public release after the bounded Git-history timeout hardening, so the latest GitHub archive and Homebrew tap include the same product code that passed the strict release gate.

## Public Release

- Repository: `thromel/ctxhelm`
- Tag: `v1.1.9`
- Target commit: final pushed `main` commit for this phase; verify with `gh release view v1.1.9 --repo thromel/ctxhelm --json targetCommitish`
- Release URL: `https://github.com/thromel/ctxhelm/releases/tag/v1.1.9`
- Archive: `ctxhelm-v1.1.9-aarch64-apple-darwin.tar.gz`
- Archive SHA-256: `d9f4b0a4b38fcdfd1702873994b8f3ed4c63af6e88e0cd33b3635f104beb3c7d`

## Proof Artifacts

- Public release freshness: verified live after release retargeting
  - `status = current`
  - `productStatus = current`
  - `commitsAhead = 0`
  - `productCommitsAhead = 0`
- Public archive install: `.ctxhelm/e2e/phase145-public-archive-install.json`
  - downloads public GitHub release assets
  - verifies archive checksum
  - installs to a temporary bin path
  - verifies `ctxhelm 1.1.9`
  - runs help, doctor, and first-pack smoke checks
- Homebrew tap: `.ctxhelm/e2e/phase145-homebrew-tap.json`
  - verifies `thromel/tap/ctxhelm`
  - verifies formula URL and SHA-256
  - installs and tests `ctxhelm 1.1.9`
- Public real-client smoke: `.ctxhelm/e2e/phase145-public-real-client-smoke.json`
  - Claude Code `2.1.159` passed with machine-checkable `prepare_task` and `get_pack` calls
  - Codex CLI `0.44.0` remained optional/skipped because it did not produce machine-checkable tool-call evidence
- Clean fixture product proof: `.ctxhelm/e2e/phase145-clean-fixture-product-proof.json`
  - `releaseGate.decision = promote`
  - four repositories evaluated
  - 16 commits evaluated
  - average File Recall@10 delta versus lexical: `+0.14154172`
  - average agent-evidence Recall@10 delta versus lexical: `+0.19379663`
  - average context Recall@10 delta versus lexical: `+0.23717105`

## Validation

- `cargo fmt --all -- --check`
- `bash scripts/check-release-docs.sh`
- `cargo run -p ctxhelm --locked -- --version`
- `cargo test -p ctxhelm --test release_packaging --locked`
- `CTXHELM_CLEAN_FIXTURE_CONFIG=/tmp/ctxhelm-phase144-fixture-config.XXXXXX.json CTXHELM_BIN="$PWD/target/release/ctxhelm" CTXHELM_REQUIRE_CLEAN_FIXTURE_PROOF=1 bash scripts/release-gate.sh`
- `bash scripts/verify-github-release.sh --repo thromel/ctxhelm --tag v1.1.9 --target "$(git rev-parse HEAD)" --assets-dir /tmp/ctxhelm-v119-assets-NO7TKq`
- `bash scripts/verify-public-archive-install.sh --repo thromel/ctxhelm --tag v1.1.9 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.9"`
- `bash scripts/verify-homebrew-tap.sh --tap thromel/tap --formula ctxhelm --expected-version "ctxhelm 1.1.9"`
- `bash scripts/smoke-public-real-clients.sh --repo thromel/ctxhelm --tag v1.1.9 --target-label aarch64-apple-darwin --expected-version "ctxhelm 1.1.9"`
