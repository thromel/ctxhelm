# Phase 138 E2E: Public v1.1.5 Release Currentness

## Goal

Refresh the public archive release after Phase 137 made the Homebrew tap a real
install channel, so the public archive and the public tap are both current with
the latest product commit.

## Release

- Tag: `v1.1.5`
- URL: `https://github.com/thromel/ctxpack/releases/tag/v1.1.5`
- Target commit: `3efa8c18d9f186c7e6a91f19c4171c3c3224158d`
- Archive: `ctxpack-v1.1.5-aarch64-apple-darwin.tar.gz`
- Archive SHA-256: `0ca4ce00012a767c5a400e79b9c81471398d5bd94c2cfccb2b8264e8f74f3d9c`
- Binary SHA-256: `38eae7123813a451bf9505c4eb26294d9505f7ab7ea570f32365ef7bfd196005`

## Homebrew Tap

- Tap repository: `https://github.com/thromel/homebrew-tap`
- Tap commit: `d49a94a48c0be46391ad92fd3d872e35f3a00378`
- Formula: `thromel/tap/ctxpack`
- Formula URL: `https://github.com/thromel/ctxpack/releases/download/v1.1.5/ctxpack-v1.1.5-aarch64-apple-darwin.tar.gz`
- Formula SHA-256: `0ca4ce00012a767c5a400e79b9c81471398d5bd94c2cfccb2b8264e8f74f3d9c`
- Platform: Apple Silicon macOS (`depends_on arch: :arm64`)

## Proof

Durable source-free proof artifacts:

- `.ctxpack/e2e/phase138-github-release.json`
- `.ctxpack/e2e/phase138-github-release-verify.json`
- `.ctxpack/e2e/phase138-public-release-freshness.json`
- `.ctxpack/e2e/phase138-public-archive-install.json`
- `.ctxpack/e2e/phase138-homebrew-tap-proof.json`
- `.ctxpack/e2e/phase138-public-real-client-smoke.json`

Commands run:

```bash
bash scripts/verify-github-release.sh \
  --repo thromel/ctxpack \
  --tag v1.1.5 \
  --target 3efa8c18d9f186c7e6a91f19c4171c3c3224158d \
  --assets-dir dist

bash scripts/check-public-release-freshness.sh \
  --repo thromel/ctxpack \
  --tag v1.1.5 \
  --require-product-current

bash scripts/verify-public-archive-install.sh \
  --repo thromel/ctxpack \
  --tag v1.1.5 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxpack 1.1.5"

brew uninstall ctxpack
brew untap thromel/tap
bash scripts/verify-homebrew-tap.sh \
  --tap thromel/tap \
  --formula ctxpack \
  --expected-version "ctxpack 1.1.5" \
  --expected-url https://github.com/thromel/ctxpack/releases/download/v1.1.5/ctxpack-v1.1.5-aarch64-apple-darwin.tar.gz \
  --expected-sha256 0ca4ce00012a767c5a400e79b9c81471398d5bd94c2cfccb2b8264e8f74f3d9c

bash scripts/smoke-public-real-clients.sh \
  --repo thromel/ctxpack \
  --tag v1.1.5 \
  --target-label aarch64-apple-darwin \
  --expected-version "ctxpack 1.1.5"
```

All required checks passed.

## Client Evidence

- Claude Code `2.1.159` passed explicit-repo `prepare_task` and `get_pack`
  evidence against the public v1.1.5 archive binary.
- Codex CLI `0.44.0` remained an optional source-free skip because it still did
  not produce machine-checkable `prepare_task` / `get_pack` evidence.

## Non-Goals

No crates.io package, signed installer, self-update path, cloud indexing, cloud
embedding, hosted service, global agent config mutation, or additional platform
archive was added.
